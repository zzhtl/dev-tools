use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

/// 统一错误类型：携带 HTTP 状态码，默认 500；输入校验类错误用 `bad_request` 返回 400。
#[derive(Debug)]
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
        // 底层错误可能含路径、依赖库内部细节等信息，只写日志，不回给客户端
        let err = e.into();
        tracing::error!("内部错误: {err:#}");
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: "内部错误，详情见服务端日志".to_string(),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (self.status, self.message).into_response()
    }
}
