use axum::body::Body;
use axum::http::{header, HeaderMap, HeaderValue, Response, StatusCode, Uri};
use rust_embed::Embed;

#[derive(Embed)]
#[folder = "$CARGO_MANIFEST_DIR/dist"]
pub struct Assets;

pub async fn serve(uri: Uri, headers: HeaderMap) -> Response<Body> {
    let path = uri.path().trim_start_matches('/');
    let key = if path.is_empty() { "index.html" } else { path };

    if let Some(file) = Assets::get(key) {
        return build_response(key, file, &headers);
    }

    // 带扩展名的资源没命中就是真的 404。此前一律回退 index.html 并返回 200，
    // 会让缺失的 .js/.png 看起来“加载成功”，掩盖构建产物不完整的问题。
    if has_extension(key) {
        return not_found();
    }

    // 无扩展名的路径视为前端路由，交回 index.html
    match Assets::get("index.html") {
        Some(file) => build_response("index.html", file, &headers),
        None => not_found(),
    }
}

fn has_extension(path: &str) -> bool {
    path.rsplit('/')
        .next()
        .is_some_and(|segment| segment.contains('.'))
}

fn not_found() -> Response<Body> {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Body::from("404 Not Found"))
        .unwrap()
}

fn build_response(
    path: &str,
    file: rust_embed::EmbeddedFile,
    headers: &HeaderMap,
) -> Response<Body> {
    let etag = format!("\"{}\"", hex(&file.metadata.sha256_hash()));

    if headers
        .get(header::IF_NONE_MATCH)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.split(',').any(|part| part.trim() == etag))
    {
        return Response::builder()
            .status(StatusCode::NOT_MODIFIED)
            .header(header::ETAG, etag)
            .body(Body::empty())
            .unwrap();
    }

    let mime = mime_guess::from_path(path).first_or_octet_stream();
    // Vite 给 assets/ 下的产物带内容哈希，可以长期强缓存；
    // index.html 必须每次校验，否则改版后用户会一直拿到旧壳。
    let cache_control = if path.starts_with("assets/") {
        "public, max-age=31536000, immutable"
    } else {
        "no-cache"
    };

    let mut res = Response::new(Body::from(file.data.into_owned()));
    let out = res.headers_mut();
    out.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(mime.as_ref())
            .unwrap_or(HeaderValue::from_static("application/octet-stream")),
    );
    out.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static(cache_control),
    );
    if let Ok(value) = HeaderValue::from_str(&etag) {
        out.insert(header::ETAG, value);
    }
    res
}

fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write;
    bytes
        .iter()
        .fold(String::with_capacity(bytes.len() * 2), |mut acc, b| {
            let _ = write!(acc, "{b:02x}");
            acc
        })
}

#[cfg(test)]
mod tests {
    use super::has_extension;

    #[test]
    fn detects_asset_paths() {
        assert!(has_extension("assets/index-abc.js"));
        assert!(has_extension("favicon.svg"));
        assert!(!has_extension("json"));
        assert!(!has_extension("some/deep/route"));
    }
}
