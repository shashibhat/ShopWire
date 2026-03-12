use crate::models::{IngestProduct, ProductHit, SearchRequest};
use sqlx::{postgres::PgRow, PgPool, QueryBuilder, Row, Transaction};

pub async fn search_products(
    pool: &PgPool,
    request: &SearchRequest,
) -> Result<Vec<ProductHit>, sqlx::Error> {
    let page = request.page.unwrap_or(1);
    let limit = request.limit.unwrap_or(10).clamp(1, 100);
    let offset = (page.saturating_sub(1) * limit) as i64;
    let sort = request
        .sort
        .clone()
        .unwrap_or_else(|| "relevance".to_string());
    let filters = request.filters.as_ref();

    let mut query_builder = QueryBuilder::new(
        "SELECT sku, name, brand, COALESCE(color, '') AS color, COALESCE(size, '') AS size, \
         price::float8 AS price, COALESCE(original_price, price)::float8 AS original_price, \
         COALESCE(discount_pct, 0) AS discount_pct, COALESCE(stock, 0) AS stock, \
         COALESCE(image_url, '') AS image_url, walmart_url \
         FROM products WHERE active = true",
    );

    if let Some(query) = request.query.as_ref() {
        if !query.trim().is_empty() {
            query_builder.push(" AND (name ILIKE ");
            query_builder.push_bind(format!("%{}%", query.trim()));
            query_builder.push(" OR brand ILIKE ");
            query_builder.push_bind(format!("%{}%", query.trim()));
            query_builder.push(")");
        }
    }

    if let Some(category) = filters.and_then(|f| f.category.as_ref()) {
        query_builder.push(" AND category = ");
        query_builder.push_bind(category);
    }

    if let Some(brands) = filters.and_then(|f| f.brand.as_ref()) {
        if !brands.is_empty() {
            query_builder.push(" AND lower(brand) = ANY(");
            let normalized = brands.iter().map(|b| b.to_lowercase()).collect::<Vec<_>>();
            query_builder.push_bind(normalized);
            query_builder.push(")");
        }
    }

    if let Some(color) = filters.and_then(|f| f.color.as_ref()) {
        query_builder.push(" AND color ILIKE ");
        query_builder.push_bind(format!("%{}%", color));
    }

    if let Some(size) = filters.and_then(|f| f.size.as_ref()) {
        query_builder.push(" AND size = ");
        query_builder.push_bind(size);
    }

    if let Some(max_price) = filters
        .and_then(|f| f.price.as_ref())
        .and_then(|price| price.max)
    {
        query_builder.push(" AND price <= ");
        query_builder.push_bind(max_price);
    }

    if let Some(discount_min) = filters.and_then(|f| f.discount_min) {
        query_builder.push(" AND COALESCE(discount_pct, 0) >= ");
        query_builder.push_bind(discount_min as i32);
    }

    if let Some(stock_min) = filters.and_then(|f| f.stock_min) {
        query_builder.push(" AND COALESCE(stock, 0) >= ");
        query_builder.push_bind(stock_min as i32);
    }

    match sort.as_str() {
        "price_asc" => query_builder.push(" ORDER BY price ASC, sku ASC"),
        "discount_desc" => query_builder.push(" ORDER BY discount_pct DESC, sku ASC"),
        _ => query_builder.push(" ORDER BY updated_at DESC, sku ASC"),
    };

    query_builder.push(" LIMIT ");
    query_builder.push_bind(limit as i64);
    query_builder.push(" OFFSET ");
    query_builder.push_bind(offset);

    let query = query_builder.build();
    let rows: Vec<PgRow> = query.fetch_all(pool).await?;
    Ok(rows.into_iter().map(map_row_to_product_hit).collect())
}

fn map_row_to_product_hit(row: PgRow) -> ProductHit {
    ProductHit {
        sku: row.get("sku"),
        name: row.get("name"),
        brand: row.get("brand"),
        color: row.get("color"),
        size: row.get("size"),
        price: row.get("price"),
        original_price: row.get("original_price"),
        discount_pct: row.get::<i32, _>("discount_pct").max(0) as u32,
        stock: row.get::<i32, _>("stock").max(0) as u32,
        image_url: row.get("image_url"),
        redirect_url: row.get("walmart_url"),
    }
}

pub async fn upsert_products(
    pool: &PgPool,
    source: &str,
    products: &[IngestProduct],
) -> Result<(usize, usize), sqlx::Error> {
    let mut tx: Transaction<'_, sqlx::Postgres> = pool.begin().await?;
    let mut upserted = 0usize;
    let mut failed = 0usize;

    for product in products {
        match upsert_single_product(&mut tx, product).await {
            Ok(product_id) => {
                upserted += 1;
                sqlx::query("INSERT INTO price_history (product_id, price) VALUES ($1, $2)")
                    .bind(product_id)
                    .bind(product.price)
                    .execute(tx.as_mut())
                    .await?;
            }
            Err(_) => {
                failed += 1;
            }
        }
    }

    sqlx::query(
        "INSERT INTO ingestion_runs (source, status, records_upserted, records_failed, finished_at)
         VALUES ($1, $2, $3, $4, NOW())",
    )
    .bind(source)
    .bind(if failed == 0 { "success" } else { "partial" })
    .bind(upserted as i32)
    .bind(failed as i32)
    .execute(tx.as_mut())
    .await?;

    tx.commit().await?;
    Ok((upserted, failed))
}

async fn upsert_single_product(
    tx: &mut Transaction<'_, sqlx::Postgres>,
    product: &IngestProduct,
) -> Result<i64, sqlx::Error> {
    let row = sqlx::query(
        "INSERT INTO products
        (sku, name, brand, category, color, size, price, original_price, stock, image_url, walmart_url, active, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, NOW())
        ON CONFLICT (sku) DO UPDATE SET
          name = EXCLUDED.name,
          brand = EXCLUDED.brand,
          category = EXCLUDED.category,
          color = EXCLUDED.color,
          size = EXCLUDED.size,
          price = EXCLUDED.price,
          original_price = EXCLUDED.original_price,
          stock = EXCLUDED.stock,
          image_url = EXCLUDED.image_url,
          walmart_url = EXCLUDED.walmart_url,
          active = EXCLUDED.active,
          updated_at = NOW()
        RETURNING id",
    )
    .bind(&product.sku)
    .bind(&product.name)
    .bind(&product.brand)
    .bind(product.category.clone().unwrap_or_else(|| "shoes".to_string()))
    .bind(&product.color)
    .bind(&product.size)
    .bind(product.price)
    .bind(product.original_price.unwrap_or(product.price))
    .bind(product.stock.unwrap_or(0) as i32)
    .bind(&product.image_url)
    .bind(&product.walmart_url)
    .bind(product.active.unwrap_or(true))
    .fetch_one(tx.as_mut())
    .await?;

    Ok(row.get::<i64, _>("id"))
}
