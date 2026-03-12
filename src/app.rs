use actix_web::web;

use crate::handlers;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/healthz").route(web::get().to(handlers::healthz)))
        .service(web::resource("/agent/discovery").route(web::get().to(handlers::discovery)))
        .service(web::resource("/v1/schema").route(web::get().to(handlers::schema)))
        .service(web::resource("/v1/search").route(web::post().to(handlers::search)))
        .service(
            web::resource("/internal/ingest/upsert").route(web::post().to(handlers::ingest_upsert)),
        );
}
