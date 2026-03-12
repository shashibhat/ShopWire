use actix_web::{http::StatusCode, test, web, App};
use shopwire::state::AppState;

#[actix_web::test]
async fn healthz_returns_ok() {
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(AppState::default()))
            .configure(shopwire::app::configure),
    )
    .await;
    let req = test::TestRequest::get().uri("/healthz").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
}

#[actix_web::test]
async fn discovery_returns_manifest() {
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(AppState::default()))
            .configure(shopwire::app::configure),
    )
    .await;
    let req = test::TestRequest::get()
        .uri("/agent/discovery")
        .to_request();
    let body: serde_json::Value = test::call_and_read_body_json(&app, req).await;
    assert_eq!(body["id"], "shopwire-v1");
    assert_eq!(body["name"], "BlueShoeMart");
}

#[actix_web::test]
async fn search_returns_structured_results() {
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(AppState::default()))
            .configure(shopwire::app::configure),
    )
    .await;
    let req = test::TestRequest::post()
        .uri("/v1/search")
        .set_json(serde_json::json!({
            "query": "cheap blue nike",
            "filters": {
              "brand": ["nike"],
              "color": "blue",
              "price": { "max": 60 },
              "size": "9"
            },
            "limit": 10,
            "page": 1
        }))
        .to_request();
    let body: serde_json::Value = test::call_and_read_body_json(&app, req).await;
    assert!(body["results"].is_array());
    assert!(body["meta"]["latency_ms"].as_u64().is_some());
}

#[actix_web::test]
async fn ingest_requires_auth_when_key_set() {
    std::env::set_var("INTERNAL_API_KEY", "secret");

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(AppState::default()))
            .configure(shopwire::app::configure),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/internal/ingest/upsert")
        .set_json(serde_json::json!({
          "source": "test",
          "products": [{
            "sku": "test-1",
            "name": "Test Shoe",
            "brand": "test",
            "price": 10.0,
            "walmart_url": "https://www.walmart.com/ip/1"
          }]
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

    std::env::remove_var("INTERNAL_API_KEY");
}

#[actix_web::test]
async fn ingest_requires_database() {
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(AppState::default()))
            .configure(shopwire::app::configure),
    )
    .await;

    let mut req_builder = test::TestRequest::post();
    req_builder = req_builder.uri("/internal/ingest/upsert");

    if let Ok(key) = std::env::var("INTERNAL_API_KEY") {
        if !key.is_empty() {
            req_builder = req_builder.insert_header(("x-internal-key", key));
        }
    }

    let req = req_builder
        .set_json(serde_json::json!({
          "source": "test",
          "products": [{
            "sku": "test-1",
            "name": "Test Shoe",
            "brand": "test",
            "price": 10.0,
            "walmart_url": "https://www.walmart.com/ip/1"
          }]
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::SERVICE_UNAVAILABLE);
}
