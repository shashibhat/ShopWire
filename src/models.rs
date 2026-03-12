use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchRequest {
    pub query: Option<String>,
    pub filters: Option<Filters>,
    pub sort: Option<String>,
    pub limit: Option<u32>,
    pub page: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Filters {
    pub category: Option<String>,
    pub brand: Option<Vec<String>>,
    pub color: Option<String>,
    pub price: Option<PriceFilter>,
    pub discount_min: Option<u32>,
    pub size: Option<String>,
    pub stock_min: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceFilter {
    pub max: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    pub results: Vec<ProductHit>,
    pub meta: SearchMeta,
    pub suggestions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductHit {
    pub sku: String,
    pub name: String,
    pub brand: String,
    pub color: String,
    pub size: String,
    pub price: f64,
    pub original_price: f64,
    pub discount_pct: u32,
    pub stock: u32,
    pub image_url: String,
    pub redirect_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchMeta {
    pub total: u32,
    pub page: u32,
    pub latency_ms: u32,
    pub source: String,
    pub accuracy: f64,
}

#[derive(Debug, Serialize)]
pub struct DiscoveryResponse {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub capabilities: Vec<String>,
    pub auth: String,
    pub rate_limit: String,
    pub schema_url: String,
    pub docs: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestUpsertRequest {
    pub source: Option<String>,
    pub products: Vec<IngestProduct>,
    pub invalidate_cache: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestProduct {
    pub sku: String,
    pub name: String,
    pub brand: String,
    pub category: Option<String>,
    pub color: Option<String>,
    pub size: Option<String>,
    pub price: f64,
    pub original_price: Option<f64>,
    pub stock: Option<u32>,
    pub image_url: Option<String>,
    pub walmart_url: String,
    pub active: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IngestUpsertResponse {
    pub source: String,
    pub received: usize,
    pub upserted: usize,
    pub failed: usize,
    pub invalidated_cache_keys: usize,
}
