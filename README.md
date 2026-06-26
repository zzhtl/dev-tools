# Dev Tools · 开发工具箱

简体中文 | [English](./README.en.md)

一个用 Rust 打包成 **单文件可执行程序** 的本地开发者工具箱：启动后自动在浏览器打开，内置 18 款常用开发工具，无需安装额外依赖，开箱即用。除 DNS 解析需向系统 DNS 服务器发起查询外，其余处理全部在本机完成，不上传数据、无遥测。

- **后端**：[Axum](https://github.com/tokio-rs/axum) 0.8 + Tokio，静态资源通过 [`rust-embed`](https://crates.io/crates/rust-embed) 直接嵌入二进制
- **前端**：[Svelte 5](https://svelte.dev/) + [Vite 6](https://vite.dev/) + TypeScript
- **构建**：`cargo build` 触发 `build.rs`，自动用 [`bun`](https://bun.sh/) 构建前端产物并嵌入

## 特性

- **单文件分发**：一个可执行文件即可运行，前端资源已打包进二进制，无运行时依赖
- **本地优先**：默认仅监听 `127.0.0.1`，可切换 `0.0.0.0` 供同网段访问；数据不出本机
- **自动打开浏览器**：启动后自动唤起默认浏览器，可用 `--no-browser` 关闭
- **端口自动顺延**：默认端口被占用时，自动向后尝试最多 20 个端口
- **Rust 承担重活**：DNS 解析、图片格式转换、JSON 互转 / Schema / 查询等计算交给后端处理

## 内置工具

| 分类 | 工具 | 说明 |
|------|------|------|
| 数据处理 | JSON 工具 | 格式化 / 压缩 / 转义 / 校验 / 树形视图 / 关键字定位；对比两段 JSON；JSON Schema 校验与生成；JSONPath 与 jq 查询；与 YAML·TOML·XML·CSV 互转；生成 Go·Java·Rust·TypeScript·C++·C# 结构体；保留 20 条历史记录 |
| 数据处理 | 正则表达式 | 实时测试、匹配高亮、分组与替换，内置常用正则预设 |
| 数据处理 | HTML 工具 | 格式化 / 压缩 / 实时预览 |
| 数据处理 | 文本对比 | 两段文本并排行级差异对比 |
| 数据处理 | Markdown | 实时预览（GFM）、代码高亮，导出 HTML / 复制 |
| 时间日期 | 时间转换 | 时间戳 ↔ 日期互转（秒 / 毫秒）、时区处理 |
| 时间日期 | Cron 表达式 | 字段拆解、常用预设、执行时间预测；支持**自然语言智能生成**（如「每天早上 8 点 30 分执行」→ `30 8 * * *`）与时区切换 |
| 编码加密 | 加密解密 | AES / DES / 3DES 对称加密，RSA 非对称加密（支持密钥对生成），Base64 编解码 |
| 编码加密 | JWT 解析 | 解码 header / payload，查看声明，使用 HMAC 密钥验签 HS256 / 384 / 512（RS / ES 仅解码） |
| 编码加密 | Base64 | 文本 ↔ Base64、图片 ↔ Base64 |
| 编码加密 | Hash 计算 | MD5 / SHA-1 / SHA-256 / SHA-384 / SHA-512 / SHA3 / RIPEMD |
| 编码加密 | URL 编解码 | URL encode / decode |
| 生成工具 | 二维码 | 文本生成二维码 |
| 生成工具 | UUID | 批量生成 UUID（v1 / v4） |
| 生成工具 | 颜色选择器 | 取色与 HEX / RGB(A) / HSL(A) 互转 |
| 网络工具 | DNS 解析 | 查询 A / AAAA / MX / CNAME / TXT / NS 记录（调用系统 DNS，单条 5 秒超时） |
| 系统工具 | Linux 命令 | 内置 15 个分类、127 条常用命令速查 |
| 系统工具 | 图片转换 | PNG / JPEG / GIF / WEBP / BMP / ICO 互转，支持缩放与 JPEG 质量调节（单文件上限 100MB） |

## 快速开始

### 直接运行可执行文件

```bash
# 默认监听 127.0.0.1:8927，并自动打开浏览器
./dev-tools

# 常用参数
./dev-tools --port 9000            # 指定端口（被占用时自动顺延，最多 +20）
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
bun run dev       # 启动 Vite 开发服务器
bun run build     # 构建到 ../dist（供 rust-embed 嵌入）
bun run check     # svelte-check 类型检查
```

> Cargo 构建脚本 `build.rs` 会监听 `web/src`、`web/public` 等目录的变更；正常情况下直接 `cargo build` 即可自动完成前端构建与资源嵌入。修改前端后需重新构建以重新嵌入产物。

## 项目结构

```text
dev-tools/
├── src/                # Rust 后端
│   ├── main.rs         # 入口：CLI 解析、端口绑定（自动顺延）、启动 axum
│   ├── server.rs       # 路由组装（/api/* 与静态资源回退）
│   ├── cli.rs          # clap 参数定义（--port / --host / --no-browser）
│   ├── assets.rs       # rust-embed 静态资源服务
│   └── handlers/       # 后端 API 处理器
│       ├── error.rs    # 共享 AppError（携带 HTTP 状态码）
│       ├── json.rs     # /api/json/{convert,schema,query}
│       ├── dns.rs      # /api/dns/resolve
│       └── image.rs    # /api/image/convert
├── web/                # Svelte 5 前端
│   ├── src/App.svelte  # 侧边栏分类与工具路由
│   └── src/tools/*     # 各工具独立模块
├── build.rs            # Cargo 构建脚本：驱动 bun 构建并嵌入产物
└── Cargo.toml
```

## 后端 API

所有接口前缀为 `/api`，前端在需要时将计算委托给 Rust 处理：

| 方法 | 路径 | 用途 | 请求体 |
|------|------|------|------|
| GET  | `/api/healthz`       | 健康检查 | — |
| POST | `/api/json/convert`  | JSON / YAML / TOML / XML / CSV 互转 | `{ input, from, to, indent? }` |
| POST | `/api/json/schema`   | JSON Schema 生成或校验 | `{ json, mode: "generate"\|"validate", schema? }` |
| POST | `/api/json/query`    | JSONPath 或 jq 查询 | `{ json, engine: "jsonpath"\|"jq", expr }` |
| POST | `/api/dns/resolve`   | DNS 记录解析 | `{ domain, types: ["A","MX",...] }` |
| POST | `/api/image/convert` | 图片格式转换（含缩放 / 质量） | `multipart: file, format, options?` |

## 技术栈

| 层 | 选型 |
|----|------|
| 后端框架 | Axum 0.8 + Tokio |
| 资源嵌入 | rust-embed（前端产物打包进二进制） |
| 中间件 | tower-http（gzip 压缩 / CORS / 请求追踪） |
| DNS | hickory-resolver |
| 图片 | image |
| JSON 生态 | serde_json / serde_yaml / toml / quick-xml / csv / jsonschema / serde_json_path / jaq |
| 前端 | Svelte 5 + Vite 6 + TypeScript |
| 前端库 | marked（Markdown）、highlight.js、qrcode、@noble/hashes、@noble/ciphers |
| Release 优化 | LTO（thin）、codegen-units=1、strip |

## 下载

最新构建产物见 [GitHub Releases](https://github.com/zzhtl/dev-tools/releases)。

## License

[Apache-2.0](./LICENSE)
