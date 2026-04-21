use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "dev-tools",
    about = "开发者工具箱：启动本地 Web 服务并在浏览器中使用",
    version
)]
pub struct Cli {
    /// 监听端口（占用时自动顺延）
    #[arg(long, default_value_t = 8927)]
    pub port: u16,

    /// 监听地址，默认仅本机。设为 0.0.0.0 可允许同网段访问
    #[arg(long, default_value = "127.0.0.1")]
    pub host: String,

    /// 启动后不自动打开浏览器
    #[arg(long)]
    pub no_browser: bool,
}
