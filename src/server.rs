use axum::routing::{get, post};
use axum::Router;
use tower_http::compression::CompressionLayer;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use crate::{assets, handlers};

pub fn build_router() -> Router {
    let api = Router::new()
        .route("/healthz", get(|| async { "ok" }))
        .route("/json/convert", post(handlers::json::convert))
        .route("/json/schema", post(handlers::json::schema))
        .route("/json/query", post(handlers::json::query))
        .route("/dns/resolve", post(handlers::dns::resolve))
        .route("/image/convert", post(handlers::image::convert));

    Router::new()
        .nest("/api", api)
        .fallback(assets::serve)
        .layer(CompressionLayer::new())
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
}
