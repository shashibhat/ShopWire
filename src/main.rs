mod app;
mod cache;
mod handlers;
mod models;
mod repository;
mod state;

use actix_web::{web, App, HttpServer};
use dotenvy::dotenv;
use state::AppState;
use tracing_subscriber::EnvFilter;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let state = AppState::from_env().await;

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            .configure(app::configure)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
