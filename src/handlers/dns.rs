use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;

use axum::extract::State;
use axum::Json;
use hickory_resolver::config::{NameServerConfig, ResolverConfig};
use hickory_resolver::net::runtime::TokioRuntimeProvider;
use hickory_resolver::proto::rr::RecordType;
use hickory_resolver::TokioResolver;
use serde::{Deserialize, Serialize};

use super::error::AppError;
use crate::state::AppState;

const QUERY_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Deserialize)]
pub struct ResolveRequest {
    pub domain: String,
    pub types: Vec<String>,
    /// 可选的上游 DNS 服务器 IP；留空则用系统配置
    #[serde(default)]
    pub server: Option<String>,
}

#[derive(Serialize)]
pub struct DnsRecord {
    pub value: String,
    pub ttl: u32,
}

#[derive(Serialize)]
pub struct DnsResult {
    pub record_type: String,
    pub records: Vec<DnsRecord>,
    /// 该类型查询失败/超时的原因。此前失败信息被塞进 records 里，前端无法区分。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

pub async fn resolve(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ResolveRequest>,
) -> Result<Json<Vec<DnsResult>>, AppError> {
    let domain = req.domain.trim();
    validate_domain(domain)?;
    if req.types.is_empty() {
        return Err(AppError::bad_request("请至少选择一种记录类型"));
    }
    if req.types.len() > 16 {
        return Err(AppError::bad_request("记录类型过多"));
    }

    // 指定了上游服务器才临时建 resolver，默认路径复用共享实例
    let custom = match req
        .server
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        Some(raw) => Some(build_resolver(raw)?),
        None => None,
    };
    let resolver = custom.as_ref().unwrap_or(&state.resolver);

    let mut results = Vec::new();
    for name in req.types {
        let Some(record_type) = parse_record_type(&name) else {
            continue;
        };
        let lookup = tokio::time::timeout(
            QUERY_TIMEOUT,
            resolver.lookup(domain.to_string(), record_type),
        )
        .await;

        let result = match lookup {
            Ok(Ok(lookup)) => DnsResult {
                record_type: name,
                records: lookup
                    .answers()
                    .iter()
                    .map(|record| DnsRecord {
                        value: record.data.to_string(),
                        ttl: record.ttl,
                    })
                    .collect(),
                error: None,
            },
            Ok(Err(e)) => DnsResult {
                record_type: name,
                records: Vec::new(),
                error: Some(format!("查询失败: {e}")),
            },
            Err(_) => DnsResult {
                record_type: name,
                records: Vec::new(),
                error: Some("查询超时".to_string()),
            },
        };
        results.push(result);
    }
    Ok(Json(results))
}

fn validate_domain(domain: &str) -> Result<(), AppError> {
    if domain.is_empty() || domain.len() > 253 {
        return Err(AppError::bad_request("无效的域名"));
    }
    if !domain
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_'))
    {
        return Err(AppError::bad_request("域名包含非法字符"));
    }
    Ok(())
}

fn parse_record_type(name: &str) -> Option<RecordType> {
    match name {
        "A" => Some(RecordType::A),
        "AAAA" => Some(RecordType::AAAA),
        "MX" => Some(RecordType::MX),
        "CNAME" => Some(RecordType::CNAME),
        "TXT" => Some(RecordType::TXT),
        "NS" => Some(RecordType::NS),
        "SOA" => Some(RecordType::SOA),
        "SRV" => Some(RecordType::SRV),
        "CAA" => Some(RecordType::CAA),
        "PTR" => Some(RecordType::PTR),
        _ => None,
    }
}

fn build_resolver(raw: &str) -> Result<TokioResolver, AppError> {
    let ip: IpAddr = raw
        .parse()
        .map_err(|_| AppError::bad_request(format!("DNS 服务器需要是 IP 地址，收到: {raw}")))?;
    let config =
        ResolverConfig::from_parts(None, Vec::new(), vec![NameServerConfig::udp_and_tcp(ip)]);
    TokioResolver::builder_with_config(config, TokioRuntimeProvider::default())
        .build()
        .map_err(|e| AppError::bad_request(format!("构建 DNS resolver 失败: {e}")))
}

#[cfg(test)]
mod tests {
    use super::{build_resolver, parse_record_type, validate_domain};
    use hickory_resolver::proto::rr::RecordType;

    #[test]
    fn accepts_normal_domains() {
        assert!(validate_domain("example.com").is_ok());
        assert!(validate_domain("a-b_c.example.co.uk").is_ok());
    }

    #[test]
    fn rejects_bad_domains() {
        assert!(validate_domain("").is_err());
        assert!(validate_domain(&"a".repeat(254)).is_err());
        // 注入类字符必须被挡住
        assert!(validate_domain("example.com;ls").is_err());
        assert!(validate_domain("exa mple.com").is_err());
        assert!(validate_domain("例子.com").is_err());
    }

    #[test]
    fn maps_supported_record_types() {
        assert_eq!(parse_record_type("A"), Some(RecordType::A));
        assert_eq!(parse_record_type("CAA"), Some(RecordType::CAA));
        assert_eq!(parse_record_type("a"), None);
        assert_eq!(parse_record_type("NOPE"), None);
    }

    #[test]
    fn custom_server_must_be_an_ip() {
        assert!(build_resolver("1.1.1.1").is_ok());
        assert!(build_resolver("dns.example.com").is_err());
    }
}
