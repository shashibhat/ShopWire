use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use tracing::{info, warn};

#[derive(Clone, Default)]
pub struct AppState {
    pub pg_pool: Option<PgPool>,
    pub redis_client: Option<redis::Client>,
}

impl AppState {
    pub async fn from_env() -> Self {
        let database_url = std::env::var("DATABASE_URL").ok();
        let redis_url = std::env::var("REDIS_URL").ok();

        let pg_pool = if let Some(url) = database_url {
            match PgPoolOptions::new().max_connections(10).connect(&url).await {
                Ok(pool) => {
                    info!("connected to postgres");
                    Some(pool)
                }
                Err(err) => {
                    warn!("postgres connection failed: {err}");
                    None
                }
            }
        } else {
            warn!("DATABASE_URL not set; using fallback search path");
            None
        };

        let redis_client = if let Some(url) = redis_url {
            match redis::Client::open(url) {
                Ok(client) => {
                    info!("configured redis client");
                    Some(client)
                }
                Err(err) => {
                    warn!("redis config failed: {err}");
                    None
                }
            }
        } else {
            warn!("REDIS_URL not set; cache disabled");
            None
        };

        Self {
            pg_pool,
            redis_client,
        }
    }
}
