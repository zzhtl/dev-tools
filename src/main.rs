use clap::Parser;
use std::net::SocketAddr;
use tokio::net::TcpListener;

mod assets;
mod cli;
mod handlers;
mod server;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,tower_http=warn".into()),
        )
        .init();

    let args = cli::Cli::parse();

    let (listener, addr) = bind(&args.host, args.port).await?;
    let url = format!("http://{addr}");

    let app = server::build_router();

    tracing::info!("dev-tools 已启动：{url}");
    eprintln!("Listening on {url}");

    if !args.no_browser {
        if let Err(e) = opener::open_browser(&url) {
            tracing::warn!("自动打开浏览器失败: {e}");
        }
    }

    axum::serve(listener, app).await?;
    Ok(())
}

async fn bind(host: &str, port: u16) -> anyhow::Result<(TcpListener, SocketAddr)> {
    let mut candidate = port;
    loop {
        let addr: SocketAddr = format!("{host}:{candidate}").parse()?;
        match TcpListener::bind(addr).await {
            Ok(l) => return Ok((l, addr)),
            Err(e) if candidate < port.saturating_add(20) => {
                tracing::warn!("端口 {candidate} 不可用 ({e})，尝试 {}", candidate + 1);
                candidate = candidate.checked_add(1).ok_or_else(|| anyhow::anyhow!("端口溢出"))?;
            }
            Err(e) => return Err(e.into()),
        }
    }
}
