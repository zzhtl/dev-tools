use serde::Serialize;
use trust_dns_resolver::TokioAsyncResolver;
use trust_dns_resolver::config::*;
use trust_dns_resolver::proto::rr::RecordType;

#[derive(Serialize)]
pub struct DnsResult {
    pub record_type: String,
    pub records: Vec<String>,
}

#[tauri::command]
pub async fn resolve_dns(domain: String, types: Vec<String>) -> Result<Vec<DnsResult>, String> {
    let mut results = Vec::new();
    let resolver = TokioAsyncResolver::tokio(ResolverConfig::default(), ResolverOpts::default());

    for t in types {
        match t.as_str() {
            "A" => {
                let response = resolver.lookup(domain.clone(), RecordType::A).await
                    .map_err(|e| format!("DNS A 查询失败: {}", e))?;
                let records = response.iter().map(|r| r.to_string()).collect();
                results.push(DnsResult {
                    record_type: "A".into(),
                    records,
                });
            }
            "AAAA" => {
                let response = resolver.ipv6_lookup(domain.clone()).await;
                if let Ok(lookup) = response {
                    results.push(DnsResult {
                        record_type: "AAAA".to_string(),
                        records: lookup.iter().map(|ip| ip.to_string()).collect(),
                    });
                }
            }
            "MX" => {
                let response = resolver.mx_lookup(domain.clone()).await;
                if let Ok(lookup) = response {
                    results.push(DnsResult {
                        record_type: "MX".to_string(),
                        records: lookup.iter().map(|mx| format!("{} {}", mx.preference(), mx.exchange())).collect(),
                    });
                }
            }
            "CNAME" => {
                let response = resolver.lookup(domain.clone(), RecordType::CNAME).await;
                if let Ok(lookup) = response {
                    results.push(DnsResult {
                        record_type: "CNAME".to_string(),
                        records: lookup.iter().map(|r| r.to_string()).collect(),
                    });
                }
            }
            "TXT" => {
                let response = resolver.txt_lookup(domain.clone()).await;
                if let Ok(lookup) = response {
                    results.push(DnsResult {
                        record_type: "TXT".to_string(),
                        records: lookup.iter().flat_map(|txt| txt.txt_data().iter().map(|d| String::from_utf8_lossy(d).to_string())).collect(),
                    });
                }
            }
            "NS" => {
                let response = resolver.ns_lookup(domain.clone()).await;
                if let Ok(lookup) = response {
                    results.push(DnsResult {
                        record_type: "NS".to_string(),
                        records: lookup.iter().map(|ns| ns.to_string()).collect(),
                    });
                }
            }
            _ => {}
        }
    }
    Ok(results)
} 