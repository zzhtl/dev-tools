use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use hickory_resolver::proto::rr::RecordType;
use hickory_resolver::TokioResolver;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct ResolveRequest {
    pub domain: String,
    pub types: Vec<String>,
}

#[derive(Serialize)]
pub struct DnsResult {
    pub record_type: String,
    pub records: Vec<String>,
}

pub async fn resolve(Json(req): Json<ResolveRequest>) -> Result<Json<Vec<DnsResult>>, AppError> {
    let resolver = TokioResolver::builder_tokio()
        .map_err(|e| anyhow::anyhow!("读取系统 DNS 配置失败: {e}"))?
        .build()
        .map_err(|e| anyhow::anyhow!("构建 DNS resolver 失败: {e}"))?;

    let mut results = Vec::new();
    for t in req.types {
        let rt = match t.as_str() {
            "A" => RecordType::A,
            "AAAA" => RecordType::AAAA,
            "MX" => RecordType::MX,
            "CNAME" => RecordType::CNAME,
            "TXT" => RecordType::TXT,
            "NS" => RecordType::NS,
            _ => continue,
        };
        match resolver.lookup(req.domain.clone(), rt).await {
            Ok(lookup) => {
                let records: Vec<String> = lookup
                    .answers()
                    .iter()
                    .map(|r| r.data.to_string())
                    .collect();
                results.push(DnsResult { record_type: t, records });
            }
            Err(e) => {
                results.push(DnsResult {
                    record_type: t,
                    records: vec![format!("查询失败: {e}")],
                });
            }
        }
    }
    Ok(Json(results))
}

pub struct AppError(anyhow::Error);

impl<E: Into<anyhow::Error>> From<E> for AppError {
    fn from(e: E) -> Self {
        Self(e.into())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, self.0.to_string()).into_response()
    }
}
