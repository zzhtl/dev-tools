use std::sync::Arc;

use axum::extract::DefaultBodyLimit;
use axum::http::{HeaderValue, Method};
use axum::routing::{get, post};
use axum::Router;
use tower_http::compression::CompressionLayer;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::trace::TraceLayer;

use crate::state::AppState;
use crate::{assets, handlers};

/// 图片上传上限。挂在路由层，超限的请求在读进内存前就被拒绝。
const IMAGE_BODY_LIMIT: usize = 100 * 1024 * 1024;
/// 其余 JSON 接口的请求体上限。
const JSON_BODY_LIMIT: usize = 32 * 1024 * 1024;

pub fn build_router(state: Arc<AppState>) -> Router {
    let api = Router::new()
        .route("/healthz", get(|| async { "ok" }))
        .route("/json/convert", post(handlers::json::convert))
        .route("/json/schema", post(handlers::json::schema))
        .route("/json/query", post(handlers::json::query))
        .route("/dns/resolve", post(handlers::dns::resolve))
        .route("/net/port", post(handlers::net::port))
        .layer(DefaultBodyLimit::max(JSON_BODY_LIMIT))
        .route(
            "/image/convert",
            post(handlers::image::convert).layer(DefaultBodyLimit::max(IMAGE_BODY_LIMIT)),
        )
        .with_state(state);

    Router::new()
        .nest("/api", api)
        .fallback(assets::serve)
        .layer(CompressionLayer::new())
        .layer(cors())
        .layer(TraceLayer::new_for_http())
}

/// 只放行本机来源。
///
/// 原先是 `CorsLayer::permissive()`：配合 `--host 0.0.0.0` 时，任意网站都能驱动
/// 本机 API（DNS 查询、端口探测），等于把本机变成对外代理。
/// 前端与后端同源，正常使用根本不需要 CORS；这里保留 localhost 仅为兼容 Vite 开发服务器。
fn cors() -> CorsLayer {
    CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(tower_http::cors::Any)
        .allow_origin(AllowOrigin::predicate(|origin: &HeaderValue, _| {
            origin.to_str().is_ok_and(is_local_origin)
        }))
}

fn is_local_origin(origin: &str) -> bool {
    let Some(rest) = origin
        .strip_prefix("http://")
        .or_else(|| origin.strip_prefix("https://"))
    else {
        return false;
    };
    // IPv6 字面量带方括号（[::1]:8927），不能直接按冒号切
    let host = match rest.strip_prefix('[') {
        Some(inner) => match inner.split(']').next() {
            Some(host) => host,
            None => return false,
        },
        None => rest.split(':').next().unwrap_or(""),
    };
    if host == "localhost" {
        return true;
    }
    // 覆盖整个 127.0.0.0/8 与 ::1，而不是只认 127.0.0.1
    host.parse::<std::net::IpAddr>()
        .is_ok_and(|ip| ip.is_loopback())
}

#[cfg(test)]
mod tests {
    use super::is_local_origin;

    #[test]
    fn accepts_loopback_origins() {
        assert!(is_local_origin("http://localhost:5173"));
        assert!(is_local_origin("http://127.0.0.1:8927"));
        assert!(is_local_origin("http://[::1]:8927"));
        assert!(is_local_origin("http://127.0.0.2:8927"));
    }

    #[test]
    fn rejects_remote_origins() {
        assert!(!is_local_origin("http://evil.example.com"));
        assert!(!is_local_origin("https://localhost.evil.com"));
        assert!(!is_local_origin("file://"));
        assert!(!is_local_origin("http://192.168.1.10:8927"));
    }
}
