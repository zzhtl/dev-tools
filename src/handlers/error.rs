use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

/// 统一错误类型：携带 HTTP 状态码，默认 500；输入校验类错误用 `bad_request` 返回 400。
pub struct AppError {
    pub status: StatusCode,
    pub message: String,
}

impl AppError {
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message: message.into(),
        }
    }
}

impl<E: Into<anyhow::Error>> From<E> for AppError {
    fn from(e: E) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: e.into().to_string(),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (self.status, self.message).into_response()
    }
}
