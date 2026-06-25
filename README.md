# Dev Tools

[English](#dev-tools) | [简体中文](#简体中文)

A local web application packaged as a **single executable binary**. It opens in your browser automatically after startup, bundles common developer utilities, requires no extra runtime dependencies, makes no external network calls, and processes everything locally on your machine.

- Backend: [Axum](https://github.com/tokio-rs/axum) + Tokio, with static assets embedded directly into the binary via [`rust-embed`](https://crates.io/crates/rust-embed)
- Frontend: [Svelte 5](https://svelte.dev/) + [Vite 6](https://vite.dev/) + TypeScript
- Build: `cargo build` triggers `build.rs`, which builds the frontend with `bun` and embeds the generated assets

## Features

- Single-file distribution: run everything from one executable with frontend assets already embedded
- Local web UI: opens automatically in a browser, with the interface rebuilt in Svelte 5
- Localhost by default: listens on `127.0.0.1`; can be switched to `0.0.0.0` for LAN access
- Automatic port fallback: if the default port is occupied, it retries the next 20 ports
- Rust-backed heavy lifting: compute-intensive tasks such as DNS resolution, image conversion, and JSON to Go struct generation are handled by the backend

## Built-in Tools

| Category | Tool | Description |
|------|------|------|
| Data Processing | JSON Tools | Format / minify / validate / tree view / diff / JSON Schema (validate & generate) / JSONPath & jq query / convert to YAML·CSV·XML·TOML / struct for Go·Java·Rust·TS·C++·C# |
| Data Processing | HTML Tools | Format / minify / preview |
| Data Processing | Regular Expressions | Test / validate / highlight matches |
| Data Processing | Text Diff | Side-by-side line-level diff of two texts |
| Data Processing | Markdown | Live preview, export HTML / MD |
| Encoding | Base64 | Text / image ↔ Base64 |
| Encoding | URL Encode/Decode | URL encode / decode |
| Encoding | Hash | MD5 / SHA family |
| Encoding | Encryption | Common symmetric / asymmetric encryption and decryption |
| Encoding | JWT | Decode header/payload, inspect claims, verify HS256/384/512 |
| Network | DNS Lookup | A / AAAA / MX / CNAME / TXT / NS |
| Generator | UUID | Batch UUID generation |
| Generator | QR Code | Generate QR codes from text |
| Time | Time Converter | Timestamp/date conversion and timezone handling |
| Time | Cron Expression | Parse cron expressions and predict next execution time |
| Image | Image Conversion | Convert common formats, with large-image compressed preview and lossless original conversion |
| Image | Color Picker | Pick colors / HEX / RGB / HSL |
| Reference | Linux Commands | Quick lookup for common commands |

## Quick Start

### Run the Executable

```bash
# Listen on 127.0.0.1:8927 by default and open the browser automatically
./dev-tools

# Common options
./dev-tools --port 9000            # Set port (auto-fallback if occupied)
./dev-tools --host 0.0.0.0         # Allow access from the local network
./dev-tools --no-browser           # Do not open a browser on startup
./dev-tools --help                 # Show all options
```

### Build from Source

Prerequisites:

- [Rust](https://www.rust-lang.org/tools/install) (stable, edition 2021)
- [Bun](https://bun.sh/): used to build the frontend. If it is not installed, `cargo build` will skip the frontend build and emit a placeholder page instead.

```bash
# Clone and build the release binary
git clone https://github.com/zzhtl/dev-tools.git
cd dev-tools
cargo build --release

# Output binary
./target/release/dev-tools
```

### Frontend Development

You can work on the frontend independently with the Vite dev server inside `web/`:

```bash
cd web
bun install
bun run dev       # Vite dev server
bun run build     # Build to ../dist for rust-embed
bun run check     # Type checking with svelte-check
```

> The Cargo build script `build.rs` watches `web/src`, `web/public`, and related directories. In normal cases, `cargo build` is enough to rebuild the frontend and embed the assets automatically.

## Project Structure

```text
dev-tools/
├── src/                # Rust backend
│   ├── main.rs         # Entry point: CLI parsing, port binding, axum startup
│   ├── server.rs       # Route assembly (/api/* and static asset fallback)
│   ├── cli.rs          # clap argument definitions
│   ├── assets.rs       # rust-embed static asset service
│   └── handlers/       # Backend API handlers
│       ├── error.rs    # Shared AppError (carries HTTP status)
│       ├── json.rs     # POST /api/json/{convert,schema,query}
│       ├── dns.rs      # POST /api/dns/resolve
│       └── image.rs    # POST /api/image/convert
├── web/                # Svelte 5 frontend
│   └── src/tools/*     # Individual tool modules
├── build.rs            # Cargo build script: runs bun build and embeds assets
└── Cargo.toml
```

## Backend API

All backend endpoints are prefixed with `/api`:

| Method | Path | Purpose |
|------|------|------|
| GET  | `/api/healthz`         | Health check |
| POST | `/api/json/convert`    | Convert between JSON / YAML / TOML / XML / CSV |
| POST | `/api/json/schema`     | Validate against / generate JSON Schema |
| POST | `/api/json/query`      | Query via JSONPath or jq |
| POST | `/api/dns/resolve`     | Resolve DNS records |
| POST | `/api/image/convert`   | Convert image formats (multipart) |

## Download

Latest builds are available on [GitHub Releases](https://github.com/zzhtl/dev-tools/releases).

## License

[Apache-2.0](./LICENSE)

---

## 简体中文

[English](#dev-tools) | [简体中文](#简体中文)

一个用 Rust 打包成 **单文件可执行程序** 的本地 Web 应用：启动后自动在浏览器打开，内置常用开发工具，无需安装额外依赖，无外网调用，所有处理均在本机完成。

- 后端：[Axum](https://github.com/tokio-rs/axum) + Tokio，静态资源通过 [`rust-embed`](https://crates.io/crates/rust-embed) 直接嵌入二进制
- 前端：[Svelte 5](https://svelte.dev/) + [Vite 6](https://vite.dev/) + TypeScript
- 构建：`cargo build` 触发 `build.rs`，自动用 `bun` 构建前端产物并嵌入

## 特性

- 单文件分发：一个可执行文件即可运行，前端资源已打包进二进制
- 本地 Web 形态：启动后自动打开浏览器，界面用 Svelte 5 重写
- 默认仅本机监听：`127.0.0.1`，可切换 `0.0.0.0` 供同网段使用
- 热切换端口：默认端口被占用时自动向后顺延 20 个端口
- 按需调用 Rust 能力：DNS 解析、图片格式转换、JSON → Go struct 等计算密集工作交给后端

## 内置工具

| 分类 | 工具 | 说明 |
|------|------|------|
| 数据处理 | JSON 工具 | 格式化 / 压缩 / 校验 / 树形 / 对比 / JSON Schema（校验与生成）/ JSONPath 与 jq 查询 / 转 YAML·CSV·XML·TOML / 转 Go·Java·Rust·TS·C++·C# 结构体 |
| 数据处理 | HTML 工具 | 格式化 / 压缩 / 预览 |
| 数据处理 | 正则表达式 | 测试 / 验证 / 匹配高亮 |
| 数据处理 | 文本对比 | 两段文本并排行级差异 |
| 数据处理 | Markdown | 实时预览，导出 HTML / MD |
| 编解码 | Base64 | 文本 / 图片 ↔ Base64 |
| 编解码 | URL 编解码 | URL encode / decode |
| 编解码 | 哈希 | MD5 / SHA 系列 |
| 编解码 | 加解密 | 常见对称 / 非对称加解密 |
| 编解码 | JWT | 解码 header/payload、查看声明、验签 HS256/384/512 |
| 网络 | DNS 解析 | A / AAAA / MX / CNAME / TXT / NS |
| 生成器 | UUID | 批量生成 UUID |
| 生成器 | 二维码 | 文本生成二维码 |
| 时间 | 时间转换 | 时间戳与日期格式互转、时区处理 |
| 时间 | Cron 表达式 | 解析并预测下一次执行时间 |
| 图像 | 图片转换 | 常见格式互转，支持大图压缩预览与原图无损转换 |
| 图像 | 颜色选择器 | 取色 / HEX / RGB / HSL |
| 参考 | Linux 命令 | 常用命令速查 |

## 快速开始

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

## 项目结构

```text
dev-tools/
├── src/                # Rust 后端
│   ├── main.rs         # 入口：CLI 解析、端口绑定、启动 axum
│   ├── server.rs       # 路由组装（/api/* 与静态资源回退）
│   ├── cli.rs          # clap 参数定义
│   ├── assets.rs       # rust-embed 静态资源服务
│   └── handlers/       # 后端 API 处理器
│       ├── error.rs    # 共享 AppError（携带 HTTP 状态码）
│       ├── json.rs     # POST /api/json/{convert,schema,query}
│       ├── dns.rs      # POST /api/dns/resolve
│       └── image.rs    # POST /api/image/convert
├── web/                # Svelte 5 前端
│   └── src/tools/*     # 各工具独立模块
├── build.rs            # Cargo 构建脚本：驱动 bun 构建并嵌入产物
└── Cargo.toml
```

## 后端 API

所有接口前缀为 `/api`，需要调用的工具会通过这些接口委托给 Rust 处理：

| 方法 | 路径 | 用途 |
|------|------|------|
| GET  | `/api/healthz`         | 健康检查 |
| POST | `/api/json/convert`    | JSON / YAML / TOML / XML / CSV 互转 |
| POST | `/api/json/schema`     | JSON Schema 校验 / 生成 |
| POST | `/api/json/query`      | JSONPath 或 jq 查询 |
| POST | `/api/dns/resolve`     | DNS 记录解析 |
| POST | `/api/image/convert`   | 图片格式转换（multipart） |

## 下载

最新构建产物见 [GitHub Releases](https://github.com/zzhtl/dev-tools/releases)。

## License

[Apache-2.0](./LICENSE)
