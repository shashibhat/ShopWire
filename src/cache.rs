use crate::models::{SearchRequest, SearchResponse};
use redis::AsyncCommands;

const SEARCH_CACHE_TTL_SECONDS: u64 = 120;

pub fn search_cache_key(request: &SearchRequest) -> String {
    let filters = request.filters.as_ref();
    let brand = filters
        .and_then(|f| f.brand.as_ref())
        .map(|b| b.join(","))
        .unwrap_or_default();
    let color = filters
        .and_then(|f| f.color.clone())
        .unwrap_or_else(|| "_".to_string());
    let category = filters
        .and_then(|f| f.category.clone())
        .unwrap_or_else(|| "shoes".to_string());
    let size = filters
        .and_then(|f| f.size.clone())
        .unwrap_or_else(|| "_".to_string());
    let price_max = filters
        .and_then(|f| f.price.as_ref())
        .and_then(|p| p.max)
        .unwrap_or(0.0);
    let discount_min = filters.and_then(|f| f.discount_min).unwrap_or(0);
    let stock_min = filters.and_then(|f| f.stock_min).unwrap_or(0);

    format!(
        "search:v1:q={}:brand={}:color={}:category={}:size={}:price_max={}:discount_min={}:stock_min={}:sort={}:limit={}:page={}",
        request.query.clone().unwrap_or_default().to_lowercase(),
        brand.to_lowercase(),
        color.to_lowercase(),
        category.to_lowercase(),
        size.to_lowercase(),
        price_max,
        discount_min,
        stock_min,
        request
            .sort
            .clone()
            .unwrap_or_else(|| "relevance".to_string())
            .to_lowercase(),
        request.limit.unwrap_or(10).clamp(1, 100),
        request.page.unwrap_or(1),
    )
}

pub async fn get_cached_search(
    client: &redis::Client,
    key: &str,
) -> Result<Option<SearchResponse>, redis::RedisError> {
    let mut conn = client.get_multiplexed_async_connection().await?;
    let payload: Option<String> = conn.get(key).await?;
    match payload {
        Some(raw) => serde_json::from_str::<SearchResponse>(&raw)
            .map(Some)
            .map_err(|_| redis::RedisError::from((redis::ErrorKind::TypeError, "invalid json"))),
        None => Ok(None),
    }
}

pub async fn set_cached_search(
    client: &redis::Client,
    key: &str,
    response: &SearchResponse,
) -> Result<(), redis::RedisError> {
    let payload = serde_json::to_string(response)
        .map_err(|_| redis::RedisError::from((redis::ErrorKind::TypeError, "invalid json")))?;
    let mut conn = client.get_multiplexed_async_connection().await?;
    conn.set_ex::<_, _, ()>(key, payload, SEARCH_CACHE_TTL_SECONDS)
        .await?;
    Ok(())
}

pub async fn invalidate_search_cache(client: &redis::Client) -> Result<usize, redis::RedisError> {
    let mut conn = client.get_multiplexed_async_connection().await?;
    let keys: Vec<String> = conn.keys("search:v1:*").await?;
    if keys.is_empty() {
        return Ok(0);
    }
    let deleted: usize = conn.del(keys).await?;
    Ok(deleted)
}
