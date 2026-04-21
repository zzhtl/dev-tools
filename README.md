# Dev Tools · 开发者工具箱

一个用 Rust 打包成 **单文件可执行程序** 的本地 Web 应用：启动后自动在浏览器打开，内置常用开发工具，无需安装额外依赖，无外网调用，所有处理均在本机完成。

- 后端：[Axum](https://github.com/tokio-rs/axum) + Tokio，静态资源通过 [`rust-embed`](https://crates.io/crates/rust-embed) 直接嵌入二进制
- 前端：[Svelte 5](https://svelte.dev/) + [Vite 6](https://vite.dev/) + TypeScript
- 构建：`cargo build` 触发 `build.rs`，自动用 `bun` 构建前端产物并嵌入

## ✨ 特性

- 🧳 **单文件分发**：一个可执行文件即可运行，前端资源已打包进二进制
- 🌐 **本地 Web 形态**：启动后自动打开浏览器，界面用 Svelte 5 重写
- 🔒 **默认仅本机监听**：`127.0.0.1`，可切换 `0.0.0.0` 供同网段使用
- ⚡ **热切换端口**：默认端口被占用时自动向后顺延 20 个端口
- 🧩 **按需调用 Rust 能力**：DNS 解析、图片格式转换、JSON → Go struct 等计算密集工作交给后端

## 🛠️ 内置工具

| 分类 | 工具 | 说明 |
|------|------|------|
| 数据处理 | JSON 工具 | 格式化 / 压缩 / 校验 / 树形折叠 / 错误提示 / 转 Go struct |
| 数据处理 | HTML 工具 | 格式化 / 压缩 / 预览 |
| 数据处理 | 正则表达式 | 测试 / 验证 / 匹配高亮 |
| 编解码 | Base64 | 文本 / 图片 ↔ Base64 |
| 编解码 | URL 编解码 | URL encode / decode |
| 编解码 | 哈希 | MD5 / SHA 系列 |
| 编解码 | 加解密 | 常见对称 / 非对称加解密 |
| 网络 | DNS 解析 | A / AAAA / MX / CNAME / TXT / NS |
| 生成器 | UUID | 批量生成 UUID |
| 生成器 | 二维码 | 文本生成二维码 |
| 时间 | 时间转换 | 时间戳与日期格式互转、时区处理 |
| 时间 | Cron 表达式 | 解析并预测下一次执行时间 |
| 图像 | 图片转换 | 常见格式互转，支持大图压缩预览与原图无损转换 |
| 图像 | 颜色选择器 | 取色 / HEX / RGB / HSL |
| 参考 | Linux 命令 | 常用命令速查 |

## 🚀 快速开始

### 直接运行可执行文件

```bash
# 默认监听 127.0.0.1:8927，并自动打开浏览器
./dev-tools

# 常用参数
./dev-tools --port 9000            # 指定端口（占用时自动顺延）
./dev-tools --host 0.0.0.0         # 允许同网段访问
./dev-tools --no-browser           # 启动后不自动打开浏览器
./dev-tools --help                 # 查看全部参数
```

### 从源码构建

前置依赖：

- [Rust](https://www.rust-lang.org/tools/install)（stable，edition 2021）
- [Bun](https://bun.sh/)：用于构建前端。若未安装，`cargo build` 会跳过前端构建并写入占位页。

```bash
# 克隆并构建 release 版本
git clone https://github.com/zzhtl/dev-tools.git
cd dev-tools
cargo build --release

# 产物位置
./target/release/dev-tools
```

### 前端独立开发

在 `web/` 目录下可用 Vite 开发服务器独立调试前端：

```bash
cd web
bun install
bun run dev       # Vite dev server
bun run build     # 构建到 ../dist（供 rust-embed 嵌入）
bun run check     # svelte-check 类型检查
```

> Cargo 构建脚本 `build.rs` 会监听 `web/src`、`web/public` 等目录的变更；正常情况下直接 `cargo build` 即可自动完成前端构建与资源嵌入。

## 📁 项目结构

```
dev-tools/
├── src/                # Rust 后端
│   ├── main.rs         # 入口：CLI 解析、端口绑定、启动 axum
│   ├── server.rs       # 路由组装（/api/* 与静态资源回退）
│   ├── cli.rs          # clap 参数定义
│   ├── assets.rs       # rust-embed 静态资源服务
│   └── handlers/       # 后端 API 处理器
│       ├── json.rs     # POST /api/json/to-go
│       ├── dns.rs      # POST /api/dns/resolve
│       └── image.rs    # GET  /api/image/formats
│                       # POST /api/image/convert
├── web/                # Svelte 5 前端
│   └── src/tools/*     # 各工具独立模块
├── build.rs            # Cargo 构建脚本：驱动 bun 构建并嵌入产物
└── Cargo.toml
```

## 🔌 后端 API

所有接口前缀为 `/api`，需要调用的工具会通过这些接口委托给 Rust 处理：

| 方法 | 路径 | 用途 |
|------|------|------|
| GET  | `/api/healthz`         | 健康检查 |
| POST | `/api/json/to-go`      | JSON 转 Go struct |
| POST | `/api/dns/resolve`     | DNS 记录解析 |
| GET  | `/api/image/formats`   | 查询支持的图片格式 |
| POST | `/api/image/convert`   | 图片格式转换（multipart） |

## 📦 下载

最新构建产物见 [GitHub Releases](https://github.com/zzhtl/dev-tools/releases)。

## 📄 License

[MIT](./LICENSE)
