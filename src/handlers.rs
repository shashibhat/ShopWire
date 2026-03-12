use std::time::Instant;

use actix_web::{web, HttpRequest, HttpResponse, Responder};
use tracing::warn;

use crate::cache::{
    get_cached_search, invalidate_search_cache, search_cache_key, set_cached_search,
};
use crate::models::{
    DiscoveryResponse, IngestUpsertRequest, IngestUpsertResponse, ProductHit, SearchMeta,
    SearchRequest, SearchResponse,
};
use crate::repository::{search_products, upsert_products};
use crate::state::AppState;

const OPENAPI_YAML: &str = include_str!("../docs/openapi.yaml");

pub async fn healthz(state: web::Data<AppState>) -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "postgres": state.pg_pool.is_some(),
        "redis": state.redis_client.is_some(),
    }))
}

pub async fn discovery() -> impl Responder {
    let response = DiscoveryResponse {
        id: "blueshoemart-v1".to_string(),
        name: "BlueShoeMart".to_string(),
        version: "1.0".to_string(),
        description: "Agent-first shoe catalog. Walmart-style inventory, instant responses, redirect to Walmart for checkout.".to_string(),
        capabilities: vec![
            "search".to_string(),
            "inventory-check".to_string(),
            "price-history".to_string(),
        ],
        auth: "jwt-optional".to_string(),
        rate_limit: "200/min per IP".to_string(),
        schema_url: "/v1/schema".to_string(),
        docs: "https://docs.blueshoemart.com/agent".to_string(),
    };

    HttpResponse::Ok().json(response)
}

pub async fn schema() -> impl Responder {
    HttpResponse::Ok()
        .content_type("application/yaml")
        .body(OPENAPI_YAML)
}

pub async fn search(
    state: web::Data<AppState>,
    payload: web::Json<SearchRequest>,
) -> impl Responder {
    let started = Instant::now();
    let request = payload.into_inner();
    let page = request.page.unwrap_or(1);
    let mut source = "fallback".to_string();
    let cache_key = search_cache_key(&request);

    if let Some(redis) = &state.redis_client {
        match get_cached_search(redis, &cache_key).await {
            Ok(Some(mut cached)) => {
                cached.meta.latency_ms = started.elapsed().as_millis() as u32;
                cached.meta.source = "cache".to_string();
                cached.meta.page = page;
                return HttpResponse::Ok().json(cached);
            }
            Ok(None) => {}
            Err(err) => warn!("redis get failed: {err}"),
        }
    }

    let results = if let Some(pool) = &state.pg_pool {
        match search_products(pool, &request).await {
            Ok(rows) => {
                source = "db".to_string();
                rows
            }
            Err(err) => {
                warn!("db search failed, using fallback: {err}");
                fallback_products(&request)
            }
        }
    } else {
        fallback_products(&request)
    };

    let mut response = SearchResponse {
        meta: SearchMeta {
            total: results.len() as u32,
            page,
            latency_ms: started.elapsed().as_millis() as u32,
            source: source.clone(),
            accuracy: 0.98,
        },
        results,
        suggestions: vec!["Try 'navy' if no royal-blue".to_string()],
    };

    if let Some(redis) = &state.redis_client {
        if let Err(err) = set_cached_search(redis, &cache_key, &response).await {
            warn!("redis set failed: {err}");
        } else if source == "db" {
            response.meta.source = "db+cached".to_string();
        }
    }

    HttpResponse::Ok().json(response)
}

pub async fn ingest_upsert(
    state: web::Data<AppState>,
    request: HttpRequest,
    payload: web::Json<IngestUpsertRequest>,
) -> impl Responder {
    if let Some(expected) = std::env::var("INTERNAL_API_KEY")
        .ok()
        .filter(|v| !v.is_empty())
    {
        let supplied = request
            .headers()
            .get("x-internal-key")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        if supplied != expected {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "unauthorized"
            }));
        }
    }

    let Some(pool) = &state.pg_pool else {
        return HttpResponse::ServiceUnavailable().json(serde_json::json!({
            "error": "database unavailable"
        }));
    };

    let body = payload.into_inner();
    if body.products.is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "products cannot be empty"
        }));
    }

    let source = body
        .source
        .clone()
        .unwrap_or_else(|| "manual-prototype".to_string());

    let (upserted, failed) = match upsert_products(pool, &source, &body.products).await {
        Ok(summary) => summary,
        Err(err) => {
            warn!("ingest upsert failed: {err}");
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "failed to upsert products"
            }));
        }
    };

    let mut invalidated = 0usize;
    if body.invalidate_cache.unwrap_or(true) {
        if let Some(redis) = &state.redis_client {
            match invalidate_search_cache(redis).await {
                Ok(count) => invalidated = count,
                Err(err) => warn!("cache invalidation failed: {err}"),
            }
        }
    }

    let response = IngestUpsertResponse {
        source,
        received: body.products.len(),
        upserted,
        failed,
        invalidated_cache_keys: invalidated,
    };

    HttpResponse::Ok().json(response)
}

fn fallback_products(request: &SearchRequest) -> Vec<ProductHit> {
    let limit = request.limit.unwrap_or(10).clamp(1, 100) as usize;
    let sort = request
        .sort
        .clone()
        .unwrap_or_else(|| "relevance".to_string());
    let filters = request.filters.as_ref();
    let query_brand = filters
        .and_then(|f| f.brand.clone())
        .and_then(|brands| brands.first().cloned())
        .unwrap_or_else(|| "nike".to_string());
    let query_color = filters
        .and_then(|f| f.color.clone())
        .unwrap_or_else(|| "royal-blue".to_string());
    let size = filters
        .and_then(|f| f.size.clone())
        .unwrap_or_else(|| "9".to_string());
    let category = filters
        .and_then(|f| f.category.clone())
        .unwrap_or_else(|| "shoes".to_string());
    let max_price = filters
        .and_then(|f| f.price.as_ref())
        .and_then(|p| p.max)
        .unwrap_or(60.0);
    let discount_min = filters.and_then(|f| f.discount_min).unwrap_or(0);
    let stock_min = filters.and_then(|f| f.stock_min).unwrap_or(0);
    let matches_query = request
        .query
        .as_deref()
        .map(|q| q.to_lowercase().contains("nike"))
        .unwrap_or(true);

    let mut items = Vec::new();
    if matches_query && category.eq_ignore_ascii_case("shoes") {
        items.push(ProductHit {
            sku: "nike-pg40-royal".to_string(),
            name: "Nike Air Zoom Pegasus 40".to_string(),
            brand: query_brand.clone(),
            color: query_color,
            size: size.clone(),
            price: 49.99,
            original_price: 89.99,
            discount_pct: 44,
            stock: 18,
            image_url: "https://i5.walmartimages.com/asr/example1.jpg".to_string(),
            redirect_url: "https://www.walmart.com/ip/456789123".to_string(),
        });
        items.push(ProductHit {
            sku: "nike-rev7-game".to_string(),
            name: "Nike Revolution 7".to_string(),
            brand: query_brand,
            color: "game-royal".to_string(),
            size,
            price: 38.0,
            original_price: 55.0,
            discount_pct: 31,
            stock: 5,
            image_url: "https://i5.walmartimages.com/asr/example2.jpg".to_string(),
            redirect_url: "https://www.walmart.com/ip/987654321".to_string(),
        });
    }

    let mut filtered = items
        .into_iter()
        .filter(|item| item.price <= max_price)
        .filter(|item| item.discount_pct >= discount_min)
        .filter(|item| item.stock >= stock_min)
        .collect::<Vec<_>>();

    match sort.as_str() {
        "price_asc" => filtered.sort_by(|a, b| a.price.total_cmp(&b.price)),
        "discount_desc" => filtered.sort_by(|a, b| b.discount_pct.cmp(&a.discount_pct)),
        _ => {}
    }

    filtered.into_iter().take(limit).collect()
}
