# Dev Tools · 开发工具箱

简体中文 | [English](./README.en.md)

一个用 Rust 打包成 **单文件可执行程序** 的本地开发者工具箱：启动后自动在浏览器打开，内置 26 款常用开发工具，无需安装额外依赖，开箱即用。除 DNS 解析、端口探测需要联网外，其余处理全部在本机完成，不上传数据、无遥测。

- **后端**：[Axum](https://github.com/tokio-rs/axum) 0.8 + Tokio，静态资源通过 [`rust-embed`](https://crates.io/crates/rust-embed) 直接嵌入二进制
- **前端**：[Svelte 5](https://svelte.dev/)（runes）+ [Vite 6](https://vite.dev/) + TypeScript
- **构建**：`cargo build` 触发 `build.rs`，自动用 [`bun`](https://bun.sh/) 构建前端产物并嵌入

## 特性

- **单文件分发**：一个可执行文件即可运行，前端资源已打包进二进制，无运行时依赖
- **完全离线**：等宽字体已随二进制分发，断网、内网环境下界面完整可用
- **本地优先**：默认仅监听 `127.0.0.1`，可切换 `0.0.0.0` 供同网段访问；数据不出本机
- **命令面板**：`Ctrl/⌘ K` 模糊搜索全部工具，键盘直达
- **亮色 / 暗色主题**：可跟随系统，选择会被记住
- **可分享的链接**：每个工具有独立 URL（`#/json`），刷新不丢、可直接发给同事
- **状态持久化**：主题、收藏、最近使用、JSON 历史记录都保存在本地浏览器
- **按需加载**：工具代码分块打包，首屏只加载约 149KB
- **端口自动顺延**：默认端口被占用时，自动向后尝试最多 20 个端口
- **Rust 承担重活**：DNS 解析、图片格式转换、JSON 互转 / Schema / 查询等计算交给后端处理

## 快捷键

| 快捷键 | 作用 |
|--------|------|
| `Ctrl/⌘ K` | 打开命令面板，搜索并跳转工具 |
| `Ctrl/⌘ /` | 显示快捷键帮助 |
| `Ctrl/⌘ B` | 折叠 / 展开侧边栏 |
| `Ctrl/⌘ ⇧ L` | 在暗色 / 亮色 / 跟随系统之间切换 |
| `Esc` | 关闭当前弹层 |

## 内置工具

| 分类 | 工具 | 说明 |
|------|------|------|
| 数据处理 | JSON 工具 | 格式化 / 压缩 / 转义 / 校验 / 树形视图 / 关键字定位；对比两段 JSON；Schema 生成与校验；JSONPath 与 jq 查询；与 YAML·TOML·XML·CSV 互转；生成 Go·Java·Rust·TypeScript·C++·C# 结构体；20 条历史记录（本地保存） |
| 数据处理 | 正则表达式 | 实时测试、匹配高亮、分组与替换，内置常用正则库与语法参考 |
| 数据处理 | HTML 工具 | 格式化 / 压缩 / 沙箱预览（预览中禁用脚本） |
| 数据处理 | 文本对比 | 两段文本并排行级差异对比，可只看差异 |
| 数据处理 | Markdown | 实时预览（GFM）、导出 HTML / Markdown |
| 数据处理 | 文本处理 | 去重、去空行、排序、随机打乱、加行号、前后缀、大小写与 camel/snake/kebab 转换、字数统计 |
| 数据处理 | SQL 格式化 | 11 种方言的格式化与压缩，关键字大小写可选 |
| 数据处理 | YAML / TOML | 语法校验、格式化，与 JSON 三向互转 |
| 时间日期 | 时间转换 | 时间戳 ↔ 日期互转（秒 / 毫秒），同时给出本地时间、UTC、ISO 8601 与相对时间 |
| 时间日期 | Cron 表达式 | 字段拆解、常用预设、未来执行时间预测；支持自然语言生成（如「每天早上 8 点 30 分执行」→ `30 8 * * *`）与时区切换 |
| 编码加密 | 加密解密 | AES / DES / 3DES 对称加密，RSA 非对称加密（支持 2048 位密钥对生成） |
| 编码加密 | JWT 解析 | 解码 header / payload，查看时间声明与过期状态，使用 HMAC 密钥验签 HS256 / 384 / 512 |
| 编码加密 | Base64 | 文本 ↔ Base64（UTF-8 安全、支持 URL-safe），任意文件 / 图片 → Data URL |
| 编码加密 | Hash 计算 | MD5 / SHA-1 / SHA-256 / 384 / 512 / SHA3-256 / RIPEMD-160，可按需勾选算法；大文件分块流式计算，带进度 |
| 编码加密 | URL 编解码 | `encodeURIComponent` 与 `encodeURI` 双模式，附常见编码对照 |
| 编码加密 | 转义工具 | Unicode / HTML 实体 / JSON 字符串 / SQL 字面量 / Shell 参数，五种转义与反转义 |
| 编码加密 | 进制转换 | 2 / 8 / 10 / 16 进制互转，AND·OR·XOR·移位·取反位运算，8–64 位宽与有无符号补码 |
| 生成工具 | 二维码 | 文本生成二维码，可调尺寸 / 容错等级 / 前景背景色，下载 PNG |
| 生成工具 | UUID | 批量生成 UUID v4 / v7 / v1，基于 `crypto` 强随机 |
| 生成工具 | 密码生成 | 强随机密码，字符集可选、可排除易混字符，给出熵值与强度评估 |
| 生成工具 | 颜色选择器 | 取色与 HEX / RGB(A) / HSL(A) 互转、配色方案、WCAG 对比度检查、渐变 CSS 生成 |
| 网络工具 | DNS 解析 | 查询 A / AAAA / CNAME / MX / TXT / NS / SOA / SRV / CAA / PTR 记录，显示 TTL，可指定上游 DNS 服务器 |
| 网络工具 | IP / CIDR | IPv4 与 IPv6 子网计算：网络地址、掩码、可用主机范围与地址类型 |
| 系统工具 | Linux 命令 | 内置 15 个分类、127 条常用命令速查，含示例与提示 |
| 系统工具 | 图片转换 | PNG / JPEG / GIF / WEBP / BMP / ICO 互转，支持批量、缩放与 JPEG 质量调节（单文件上限 100MB） |
| 系统工具 | 图片 / PDF 水印 | 拖入预览、自定义多行文字、拖动定位、倾斜、颜色 / 字号 / 透明度、整页平铺；保存设置，导出 PNG / JPEG、PDF 当前页图片或全部页 PDF |

### 图片 / PDF 水印

打开「系统工具 → 图片 / PDF 水印」（`#/watermark`），拖入图片或 PDF，调整水印后点击导出即可保存新文件。PDF 可翻页预览，整份导出时所有页面使用同一组设置，并保留原页面内容。「保存水印设置」只保存文字和样式到当前浏览器，供下次使用。

支持 PNG / JPEG / WebP / BMP / GIF 图片和未加密 PDF，单文件上限 100MB；GIF 取首帧。图片按原尺寸导出（边长最多 16384 像素、总像素最多 3200 万）；PDF 当前页图片按 144 DPI 导出，长边最多 4096 像素。PDF 解析资源随程序分发，无需联网。

## 快速开始

### 一键安装（推荐）

**macOS / Linux**

```bash
curl -fsSL https://raw.githubusercontent.com/zzhtl/dev-tools/main/install.sh | sh
```

**Windows（PowerShell）**

```powershell
irm https://raw.githubusercontent.com/zzhtl/dev-tools/main/install.ps1 | iex
```

安装脚本会识别系统与架构、下载对应产物、**用 `SHA256SUMS` 校验后**再安装；默认装到 `~/.local/bin`（Windows 为 `%LOCALAPPDATA%\Programs\dev-tools`），不需要 sudo。

可用参数：

```bash
# 安装到指定目录
curl -fsSL .../install.sh | sh -s -- --dir /usr/local/bin

# 安装指定版本
curl -fsSL .../install.sh | sh -s -- --version v0.3.0
```

### 手动下载

各平台产物见 [GitHub Releases](https://github.com/zzhtl/dev-tools/releases)：

| 平台 | 产物 |
|------|------|
| Linux x86_64 | `dev-tools-<版本>-x86_64-unknown-linux-musl.tar.gz` |
| Linux ARM64 | `dev-tools-<版本>-aarch64-unknown-linux-musl.tar.gz` |
| macOS Apple Silicon | `dev-tools-<版本>-aarch64-apple-darwin.tar.gz` |
| macOS Intel | `dev-tools-<版本>-x86_64-apple-darwin.tar.gz` |
| Windows x86_64 | `dev-tools-<版本>-x86_64-pc-windows-msvc.zip` |

Linux 版本为 musl 静态链接，不依赖系统 glibc。每次发布都附 `SHA256SUMS`，建议校验后再运行。

### 运行

```bash
# 默认监听 127.0.0.1:8927，并自动打开浏览器
dev-tools

# 常用参数
dev-tools --port 9000            # 指定端口（被占用时自动顺延，最多 +20）
dev-tools --host 0.0.0.0         # 允许同网段访问
dev-tools --no-browser           # 启动后不自动打开浏览器
dev-tools --help                 # 查看全部参数
```

> 通过局域网 IP 访问时浏览器处于非安全上下文，部分浏览器 API（剪贴板、`crypto.randomUUID`）不可用；工具已内置回退方案，功能仍然可用。

### 从源码构建

前置依赖：

- [Rust](https://www.rust-lang.org/tools/install)（stable，edition 2021）
- [Bun](https://bun.sh/)：用于构建前端。**未安装时 `cargo build` 会直接失败**，而不是静默产出一个没有前端的空壳二进制。

```bash
git clone https://github.com/zzhtl/dev-tools.git
cd dev-tools
cargo build --release

# 产物位置
./target/release/dev-tools
```

若前端已在别处构建好（如 CI 中只构建一次、多平台复用），可跳过 build.rs 里的前端步骤：

```bash
cd web && bun install && bun run build && cd ..
DEV_TOOLS_SKIP_WEB_BUILD=1 cargo build --release
```

该模式会校验 `dist/index.html` 存在且不是占位页，否则构建失败。

### 前端独立开发

```bash
cd web
bun install
bun run dev       # Vite 开发服务器（/api 代理到 127.0.0.1:8927）
bun run build     # 构建到 ../dist（供 rust-embed 嵌入）
bun run check     # svelte-check 类型检查
bun test          # 单元测试
bunx playwright install chromium  # 首次运行浏览器测试前安装 Chromium
bun run test:e2e   # 水印拖放、图片 / PDF 导出和布局测试
```

## 项目结构

```text
dev-tools/
├── src/                    # Rust 后端
│   ├── main.rs             # 入口：CLI 解析、端口绑定（自动顺延）、启动 axum
│   ├── server.rs           # 路由组装、请求体上限、CORS（仅放行本机来源）
│   ├── cli.rs              # clap 参数定义（--port / --host / --no-browser）
│   ├── state.rs            # 进程级共享状态（复用的 DNS resolver）
│   ├── assets.rs           # rust-embed 静态资源服务（ETag / Cache-Control / SPA 回退）
│   └── handlers/           # 后端 API 处理器
│       ├── error.rs        # 共享 AppError（内部错误不外泄细节）
│       ├── json.rs         # /api/json/{convert,schema,query}
│       ├── dns.rs          # /api/dns/resolve
│       ├── net.rs          # /api/net/port
│       └── image.rs        # /api/image/convert
├── web/                    # Svelte 5 前端
│   ├── src/App.svelte      # 应用外壳：顶栏、侧边栏、路由、主题、快捷键
│   ├── src/lib/design/     # 设计 token 与全局基线样式
│   ├── src/lib/ui/         # 共享组件库（Button/Panel/SplitPane/DiffView…）
│   ├── src/lib/tools.ts    # 工具注册表（单一数据源，动态 import）
│   └── src/tools/*         # 各工具独立模块
├── .github/workflows/      # CI 与按 tag 触发的多平台发布
├── install.sh              # macOS / Linux 安装脚本
├── install.ps1             # Windows 安装脚本
├── build.rs                # Cargo 构建脚本：驱动 bun 构建并嵌入产物
└── Cargo.toml
```

## 后端 API

所有接口前缀为 `/api`，前端在需要时将计算委托给 Rust 处理：

| 方法 | 路径 | 用途 | 请求体 |
|------|------|------|------|
| GET  | `/api/healthz`       | 健康检查 | — |
| POST | `/api/json/convert`  | JSON / YAML / TOML / XML / CSV 互转 | `{ input, from, to, indent? }` |
| POST | `/api/json/schema`   | JSON Schema 生成或校验 | `{ json, mode: "generate"\|"validate", schema? }` |
| POST | `/api/json/query`    | JSONPath 或 jq 查询（3 秒超时、输出条数与体积上限） | `{ json, engine: "jsonpath"\|"jq", expr }` |
| POST | `/api/dns/resolve`   | DNS 记录解析（返回 TTL） | `{ domain, types: ["A","MX",...], server? }` |
| POST | `/api/net/port`      | 单主机单端口 TCP 连通性探测（3 秒超时） | `{ host, port }` |
| POST | `/api/image/convert` | 图片格式转换（含缩放 / 质量） | `multipart: file, format, options?` |

CORS 仅放行本机来源（`localhost` / 回环地址），避免以 `--host 0.0.0.0` 暴露时被任意网站驱动。JSON Diff 为纯前端实现，没有对应接口。

## 技术栈

| 层 | 选型 |
|----|------|
| 后端框架 | Axum 0.8 + Tokio |
| 资源嵌入 | rust-embed（前端产物打包进二进制） |
| 中间件 | tower-http（gzip 压缩 / CORS / 请求追踪） |
| DNS | hickory-resolver |
| 图片 | image |
| JSON 生态 | serde_json / serde_yaml / toml / quick-xml / csv / jsonschema / serde_json_path / jaq |
| 前端 | Svelte 5（runes）+ Vite 6 + TypeScript |
| 前端库 | marked + DOMPurify（Markdown）、qrcode、crypto-js、jsencrypt、cron-parser、date-fns、sql-formatter |
| 字体 | JetBrains Mono（OFL-1.1，随二进制分发）；中文使用系统字体栈 |
| Release 优化 | LTO（thin）、codegen-units=1、strip |

## License

[Apache-2.0](./LICENSE)
