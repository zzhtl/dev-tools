# Dev Tools

[简体中文](./README.md) | English

A local developer toolbox packaged as a **single executable binary** with Rust. It opens in your browser automatically after startup and bundles 18 common developer utilities — no extra runtime dependencies, ready to use out of the box. Everything is processed locally on your machine (the only exception is DNS lookup, which by nature queries your system's DNS servers); no data is uploaded and there is no telemetry.

- **Backend**: [Axum](https://github.com/tokio-rs/axum) 0.8 + Tokio, with static assets embedded directly into the binary via [`rust-embed`](https://crates.io/crates/rust-embed)
- **Frontend**: [Svelte 5](https://svelte.dev/) + [Vite 6](https://vite.dev/) + TypeScript
- **Build**: `cargo build` triggers `build.rs`, which builds the frontend with [`bun`](https://bun.sh/) and embeds the generated assets

## Features

- **Single-file distribution**: run everything from one executable with frontend assets already embedded, zero runtime dependencies
- **Local-first**: listens on `127.0.0.1` by default; switch to `0.0.0.0` for LAN access. Your data never leaves the machine
- **Auto-open browser**: launches your default browser on startup; disable with `--no-browser`
- **Automatic port fallback**: if the default port is occupied, it retries up to the next 20 ports
- **Rust does the heavy lifting**: DNS resolution, image conversion, and JSON convert / schema / query are handled by the backend

## Built-in Tools

| Category | Tool | Description |
|------|------|------|
| Data | JSON Tools | Format / minify / escape / validate / tree view / keyword locate; diff two JSON documents; JSON Schema validate & generate; JSONPath and jq query; convert to/from YAML·TOML·XML·CSV; generate structs for Go·Java·Rust·TypeScript·C++·C#; keeps 20 history entries |
| Data | Regular Expressions | Live testing, match highlighting, groups & replace, with built-in common-pattern presets |
| Data | HTML Tools | Format / minify / live preview |
| Data | Text Diff | Side-by-side line-level diff of two texts |
| Data | Markdown | Live preview (GFM), code highlighting, export HTML / copy |
| Time | Time Converter | Timestamp ↔ date conversion (seconds / milliseconds), timezone handling |
| Time | Cron Expression | Field breakdown, common presets, next-run prediction; supports **natural-language generation** (e.g. "run at 8:30 every morning" → `30 8 * * *`) and timezone switching |
| Encoding | Crypto | AES / DES / 3DES symmetric encryption, RSA asymmetric encryption (with key-pair generation), Base64 |
| Encoding | JWT | Decode header / payload, inspect claims, verify HS256 / 384 / 512 with an HMAC key (RS / ES decode-only) |
| Encoding | Base64 | Text ↔ Base64, image ↔ Base64 |
| Encoding | Hash | MD5 / SHA-1 / SHA-256 / SHA-384 / SHA-512 / SHA3 / RIPEMD |
| Encoding | URL Encode/Decode | URL encode / decode |
| Generator | QR Code | Generate QR codes from text |
| Generator | UUID | Batch UUID generation (v1 / v4) |
| Generator | Color Picker | Pick colors and convert between HEX / RGB(A) / HSL(A) |
| Network | DNS Lookup | Resolve A / AAAA / MX / CNAME / TXT / NS records (uses the system resolver, 5s timeout per record) |
| System | Linux Commands | Quick lookup across 15 categories and 127 common commands |
| System | Image Conversion | Convert between PNG / JPEG / GIF / WEBP / BMP / ICO, with resize and JPEG quality control (100MB upload limit) |

## Quick Start

### Run the Executable

```bash
# Listen on 127.0.0.1:8927 by default and open the browser automatically
./dev-tools

# Common options
./dev-tools --port 9000            # Set port (auto-fallback up to +20 if occupied)
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
bun run dev       # Start the Vite dev server
bun run build     # Build to ../dist (for rust-embed)
bun run check     # Type checking with svelte-check
```

> The Cargo build script `build.rs` watches `web/src`, `web/public`, and related directories. In normal cases, `cargo build` is enough to rebuild the frontend and embed the assets. After changing the frontend, rebuild to re-embed the assets.

## Project Structure

```text
dev-tools/
├── src/                # Rust backend
│   ├── main.rs         # Entry point: CLI parsing, port binding (auto-fallback), axum startup
│   ├── server.rs       # Route assembly (/api/* and static asset fallback)
│   ├── cli.rs          # clap argument definitions (--port / --host / --no-browser)
│   ├── assets.rs       # rust-embed static asset service
│   └── handlers/       # Backend API handlers
│       ├── error.rs    # Shared AppError (carries HTTP status)
│       ├── json.rs     # /api/json/{convert,schema,query}
│       ├── dns.rs      # /api/dns/resolve
│       └── image.rs    # /api/image/convert
├── web/                # Svelte 5 frontend
│   ├── src/App.svelte  # Sidebar categories and tool routing
│   └── src/tools/*     # Individual tool modules
├── build.rs            # Cargo build script: runs bun build and embeds assets
└── Cargo.toml
```

## Backend API

All backend endpoints are prefixed with `/api`; the frontend delegates computation to Rust when needed:

| Method | Path | Purpose | Request Body |
|------|------|------|------|
| GET  | `/api/healthz`       | Health check | — |
| POST | `/api/json/convert`  | Convert between JSON / YAML / TOML / XML / CSV | `{ input, from, to, indent? }` |
| POST | `/api/json/schema`   | Generate or validate JSON Schema | `{ json, mode: "generate"\|"validate", schema? }` |
| POST | `/api/json/query`    | Query via JSONPath or jq | `{ json, engine: "jsonpath"\|"jq", expr }` |
| POST | `/api/dns/resolve`   | Resolve DNS records | `{ domain, types: ["A","MX",...] }` |
| POST | `/api/image/convert` | Convert image formats (with resize / quality) | `multipart: file, format, options?` |

## Tech Stack

| Layer | Choice |
|----|------|
| Backend framework | Axum 0.8 + Tokio |
| Asset embedding | rust-embed (frontend bundled into the binary) |
| Middleware | tower-http (gzip compression / CORS / request tracing) |
| DNS | hickory-resolver |
| Image | image |
| JSON ecosystem | serde_json / serde_yaml / toml / quick-xml / csv / jsonschema / serde_json_path / jaq |
| Frontend | Svelte 5 + Vite 6 + TypeScript |
| Frontend libs | marked (Markdown), highlight.js, qrcode, @noble/hashes, @noble/ciphers |
| Release optimization | LTO (thin), codegen-units=1, strip |

## Download

Latest builds are available on [GitHub Releases](https://github.com/zzhtl/dev-tools/releases).

## License

[Apache-2.0](./LICENSE)
