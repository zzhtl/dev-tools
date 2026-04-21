use axum::routing::{get, post};
use axum::Router;
use tower_http::compression::CompressionLayer;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use crate::{assets, handlers};

pub fn build_router() -> Router {
    let api = Router::new()
        .route("/healthz", get(|| async { "ok" }))
        .route("/json/to-go", post(handlers::json::to_go))
        .route("/dns/resolve", post(handlers::dns::resolve))
        .route("/image/formats", get(handlers::image::formats))
        .route("/image/convert", post(handlers::image::convert));

    Router::new()
        .nest("/api", api)
        .fallback(assets::serve)
        .layer(CompressionLayer::new())
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
}
