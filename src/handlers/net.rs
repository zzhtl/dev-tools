use std::time::{Duration, Instant};

use axum::Json;
use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;

use super::error::AppError;

const CONNECT_TIMEOUT: Duration = Duration::from_secs(3);

#[derive(Deserialize)]
pub struct PortRequest {
    pub host: String,
    pub port: u16,
}

#[derive(Serialize)]
pub struct PortResponse {
    pub open: bool,
    /// 连接耗时（毫秒），失败时为 None
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency_ms: Option<u128>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// 单主机单端口的 TCP 连通性探测。
///
/// 刻意不做端口范围扫描：服务可能以 `--host 0.0.0.0` 暴露在局域网，
/// 提供批量扫描等于把本机变成扫描代理。
pub async fn port(Json(req): Json<PortRequest>) -> Result<Json<PortResponse>, AppError> {
    let host = req.host.trim();
    if host.is_empty() || host.len() > 253 {
        return Err(AppError::bad_request("无效的主机"));
    }
    if !host
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_' | ':'))
    {
        return Err(AppError::bad_request("主机包含非法字符"));
    }
    if req.port == 0 {
        return Err(AppError::bad_request("端口需要在 1..=65535 之间"));
    }

    let target = format!("{host}:{}", req.port);
    let started = Instant::now();
    let outcome = tokio::time::timeout(CONNECT_TIMEOUT, TcpStream::connect(&target)).await;

    let response = match outcome {
        Ok(Ok(_stream)) => PortResponse {
            open: true,
            latency_ms: Some(started.elapsed().as_millis()),
            error: None,
        },
        Ok(Err(e)) => PortResponse {
            open: false,
            latency_ms: None,
            error: Some(e.to_string()),
        },
        Err(_) => PortResponse {
            open: false,
            latency_ms: None,
            error: Some(format!("连接超时（{} 秒）", CONNECT_TIMEOUT.as_secs())),
        },
    };
    Ok(Json(response))
}
