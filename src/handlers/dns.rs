use axum::Json;
use hickory_resolver::proto::rr::RecordType;
use hickory_resolver::TokioResolver;
use serde::{Deserialize, Serialize};

use super::error::AppError;

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
    let domain = req.domain.trim();
    if domain.is_empty() || domain.len() > 253 {
        return Err(AppError::bad_request("无效的域名"));
    }
    if !domain
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_'))
    {
        return Err(AppError::bad_request("域名包含非法字符"));
    }
    if req.types.is_empty() {
        return Err(AppError::bad_request("请至少选择一种记录类型"));
    }
    if req.types.len() > 16 {
        return Err(AppError::bad_request("记录类型过多"));
    }

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
        let lookup = tokio::time::timeout(
            std::time::Duration::from_secs(5),
            resolver.lookup(domain.to_string(), rt),
        )
        .await;
        let records = match lookup {
            Ok(Ok(lookup)) => lookup.answers().iter().map(|r| r.data.to_string()).collect(),
            Ok(Err(e)) => vec![format!("查询失败: {e}")],
            Err(_) => vec!["查询超时".to_string()],
        };
        results.push(DnsResult { record_type: t, records });
    }
    Ok(Json(results))
}
