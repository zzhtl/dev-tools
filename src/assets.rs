use axum::body::Body;
use axum::http::{header, HeaderValue, Response, StatusCode, Uri};
use rust_embed::Embed;

#[derive(Embed)]
#[folder = "$CARGO_MANIFEST_DIR/dist"]
pub struct Assets;

pub async fn serve(uri: Uri) -> Response<Body> {
    let path = uri.path().trim_start_matches('/');
    let key = if path.is_empty() { "index.html" } else { path };

    if let Some(file) = Assets::get(key) {
        return build_response(key, file.data.into_owned());
    }

    // SPA fallback：未命中的路径返回首页让前端路由接管
    if let Some(file) = Assets::get("index.html") {
        return build_response("index.html", file.data.into_owned());
    }

    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Body::from("404 Not Found"))
        .unwrap()
}

fn build_response(path: &str, bytes: Vec<u8>) -> Response<Body> {
    let mime = mime_guess::from_path(path).first_or_octet_stream();
    let mut res = Response::new(Body::from(bytes));
    res.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(mime.as_ref()).unwrap_or(HeaderValue::from_static("application/octet-stream")),
    );
    res
}
