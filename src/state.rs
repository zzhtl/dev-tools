use std::sync::Arc;

use hickory_resolver::TokioResolver;

/// 进程级共享状态。
///
/// DNS resolver 在这里创建一次并复用：此前每个请求都新建 resolver，
/// 既要重读系统配置，也拿不到任何缓存收益。
pub struct AppState {
    pub resolver: TokioResolver,
}

impl AppState {
    pub fn new() -> anyhow::Result<Arc<Self>> {
        let resolver = TokioResolver::builder_tokio()
            .map_err(|e| anyhow::anyhow!("读取系统 DNS 配置失败: {e}"))?
            .build()
            .map_err(|e| anyhow::anyhow!("构建 DNS resolver 失败: {e}"))?;
        Ok(Arc::new(Self { resolver }))
    }
}
