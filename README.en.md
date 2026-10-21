# Dev Tools

[简体中文](./README.md) | English

[Download v0.4.0](https://github.com/zzhtl/dev-tools/releases/tag/v0.4.0) · [All releases](https://github.com/zzhtl/dev-tools/releases)

A local developer toolbox shipped as a **single Rust executable**. Run it and your browser opens with 26 everyday developer tools — no extra runtime, no install steps. Apart from DNS lookups and TCP port probes, everything runs on your machine: no uploads, no telemetry.

- **Backend**: [Axum](https://github.com/tokio-rs/axum) 0.8 + Tokio, static assets embedded via [`rust-embed`](https://crates.io/crates/rust-embed)
- **Frontend**: [Svelte 5](https://svelte.dev/) (runes) + [Vite 6](https://vite.dev/) + TypeScript
- **Build**: `cargo build` runs `build.rs`, which builds the frontend with [`bun`](https://bun.sh/) and embeds the output

## Features

- **Single-file distribution** — one executable, frontend assets baked in, no runtime dependencies
- **Fully offline** — the monospace font ships with the binary, so the UI is complete on air-gapped machines
- **Local first** — binds `127.0.0.1` by default; switch to `0.0.0.0` for LAN access
- **Image / PDF watermarks** — drop files to preview, customize text, position, angle, color, size and opacity; tile watermarks, save settings and export
- **Command palette** — `Ctrl/⌘ K` fuzzy-searches every tool
- **Light / dark themes** — can follow the system; your choice is remembered
- **Shareable links** — every tool has its own URL (`#/json`), survives reloads
- **Persistent state** — theme, favourites, recents and JSON history are stored locally
- **Lazy loading** — tools are code-split; the first paint pulls only ~149KB
- **Automatic port fallback** — tries up to 20 subsequent ports when the default is taken
- **Rust does the heavy lifting** — DNS, image conversion, JSON conversion / schema / queries run on the backend

## Keyboard shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl/⌘ K` | Open the command palette |
| `Ctrl/⌘ /` | Show keyboard shortcuts |
| `Ctrl/⌘ B` | Collapse / expand the sidebar |
| `Ctrl/⌘ ⇧ L` | Cycle dark / light / system theme |
| `Esc` | Close the current overlay |

## Included tools

| Category | Tool | Description |
|----------|------|-------------|
| Data | JSON | Format / minify / escape / validate / tree view / keyword jump; diff two documents; JSON Schema generation and validation; JSONPath and jq queries; convert to and from YAML·TOML·XML·CSV; generate Go·Java·Rust·TypeScript·C++·C# structs; 20 locally stored history entries |
| Data | Regex | Live testing, match highlighting, capture groups and replacement, with a built-in pattern library and syntax reference |
| Data | HTML | Format / minify / sandboxed preview (scripts disabled in the preview) |
| Data | Text diff | Side-by-side line diff with a changes-only view |
| Data | Markdown | Live GFM preview, export as HTML or Markdown |
| Data | Text utilities | Dedupe, drop blanks, sort, shuffle, number lines, add prefix/suffix, case conversion including camel/snake/kebab, word counts |
| Data | SQL formatter | Format and minify across 11 dialects, with keyword-case options |
| Data | YAML / TOML | Validate, format, and convert between YAML, TOML and JSON |
| Time | Timestamp | Convert timestamps ↔ dates (s / ms) and show local, UTC, ISO 8601 and relative time |
| Time | Cron | Field breakdown, presets, next-run prediction, timezone switching, plus natural-language generation |
| Encoding | Encrypt / decrypt | AES / DES / 3DES symmetric ciphers, RSA with 2048-bit key-pair generation |
| Encoding | JWT | Decode header / payload, inspect time claims and expiry, verify HS256 / 384 / 512 with an HMAC secret |
| Encoding | Base64 | UTF-8-safe text ↔ Base64 with a URL-safe variant; any file or image → data URL |
| Encoding | Hash | MD5 / SHA-1 / SHA-256 / 384 / 512 / SHA3-256 / RIPEMD-160; pick the algorithms you need; large files hashed in chunks with progress |
| Encoding | URL | `encodeURIComponent` and `encodeURI` modes with a reference table |
| Encoding | Escaping | Unicode / HTML entities / JSON strings / SQL literals / shell arguments, both directions |
| Encoding | Radix | Convert between base 2 / 8 / 10 / 16, bitwise AND·OR·XOR·shift·NOT, 8–64 bit widths, signed and unsigned |
| Generators | QR code | Generate from text with adjustable size, error correction and colours; download as PNG |
| Generators | UUID | Bulk generate UUID v4 / v7 / v1 backed by `crypto` |
| Generators | Password | Strong random passwords with selectable character sets, ambiguous-character exclusion, entropy and strength estimates |
| Generators | Colour picker | Pick colours, convert HEX / RGB(A) / HSL(A), colour schemes, WCAG contrast checks, gradient CSS |
| Network | DNS | Query A / AAAA / CNAME / MX / TXT / NS / SOA / SRV / CAA / PTR records with TTLs; optional upstream resolver |
| Network | IP / CIDR | IPv4 and IPv6 subnet maths: network address, mask, usable host range, address scope |
| System | Linux commands | 127 common commands across 15 categories, with examples and tips |
| System | Image conversion | PNG / JPEG / GIF / WEBP / BMP / ICO, batch mode, resizing and JPEG quality (100MB per file) |
| System | Image / PDF watermark | Drop files to preview; multiline text, drag positioning, rotation, color, size, opacity and tiling; save settings, export PNG / JPEG, a PDF page as an image, or all pages as PDF |

### Image / PDF watermark (new in v0.4.0)

Open `#/watermark`, drop an image or PDF, customize the watermark and export a new file. PDFs support page navigation; PDF export applies the settings to every page and preserves the original page content. Save watermark settings to reuse the text and style in the same browser.

Supports PNG / JPEG / WebP / BMP / GIF and unencrypted PDFs, up to 100MB per file. GIFs use the first frame. Images export at their original dimensions (up to 16384 pixels per side and 32 megapixels). PDF page images export at 144 DPI, capped at 4096 pixels on the longest side. PDF resources are bundled for offline use.

## Getting started

### One-line install (recommended)

**macOS / Linux**

```bash
curl -fsSL https://raw.githubusercontent.com/zzhtl/dev-tools/main/install.sh | sh
```

**Windows (PowerShell)**

```powershell
irm https://raw.githubusercontent.com/zzhtl/dev-tools/main/install.ps1 | iex
```

The installer detects your OS and architecture, downloads the matching artifact, **verifies it against `SHA256SUMS`**, and installs to `~/.local/bin` (`%LOCALAPPDATA%\Programs\dev-tools` on Windows) — no sudo required.

```bash
# install to a specific directory
curl -fsSL .../install.sh | sh -s -- --dir /usr/local/bin

# install a specific version
curl -fsSL .../install.sh | sh -s -- --version v0.4.0
```

### Manual download

Platform packages and the `SHA256SUMS` checksum file are available on the [v0.4.0 Release page](https://github.com/zzhtl/dev-tools/releases/tag/v0.4.0):

| Platform | Artifact |
|----------|----------|
| Linux x86_64 | `dev-tools-<version>-x86_64-unknown-linux-musl.tar.gz` |
| Linux ARM64 | `dev-tools-<version>-aarch64-unknown-linux-musl.tar.gz` |
| macOS Apple Silicon | `dev-tools-<version>-aarch64-apple-darwin.tar.gz` |
| macOS Intel | `dev-tools-<version>-x86_64-apple-darwin.tar.gz` |
| Windows x86_64 | `dev-tools-<version>-x86_64-pc-windows-msvc.zip` |

Linux builds are statically linked against musl, so they do not depend on the host glibc. Every release ships a `SHA256SUMS` file — verify before running.

### Running

```bash
# listens on 127.0.0.1:8927 and opens your browser
dev-tools

# common flags
dev-tools --port 9000            # pick a port (falls forward up to +20 if taken)
dev-tools --host 0.0.0.0         # allow LAN access
dev-tools --no-browser           # do not open a browser on startup
dev-tools --help                 # all options
```

> Reaching the app over a LAN IP puts the browser in a non-secure context, where some APIs (clipboard, `crypto.randomUUID`) are unavailable. The tools ship with fallbacks, so they keep working.

### Building from source

Prerequisites:

- [Rust](https://www.rust-lang.org/tools/install) (stable, edition 2021)
- [Bun](https://bun.sh/) to build the frontend. **`cargo build` fails outright when bun is missing**, rather than silently producing a binary with no frontend.

```bash
git clone https://github.com/zzhtl/dev-tools.git
cd dev-tools
cargo build --release

./target/release/dev-tools
```

If the frontend was already built elsewhere (for example built once in CI and reused across platforms), skip the frontend step in `build.rs`:

```bash
cd web && bun install && bun run build && cd ..
DEV_TOOLS_SKIP_WEB_BUILD=1 cargo build --release
```

That mode asserts `dist/index.html` exists and is not the placeholder page; otherwise the build fails.

### Frontend development

```bash
cd web
bun install
bun run dev       # Vite dev server (proxies /api to 127.0.0.1:8927)
bun run build     # build into ../dist for rust-embed
bun run check     # svelte-check
```

## Backend API

Every endpoint is prefixed with `/api`; the frontend delegates the heavy work to Rust:

| Method | Path | Purpose | Body |
|--------|------|---------|------|
| GET  | `/api/healthz`       | Health check | — |
| POST | `/api/json/convert`  | JSON / YAML / TOML / XML / CSV conversion | `{ input, from, to, indent? }` |
| POST | `/api/json/schema`   | JSON Schema generation or validation | `{ json, mode: "generate"\|"validate", schema? }` |
| POST | `/api/json/query`    | JSONPath or jq queries (3s timeout, output count and size caps) | `{ json, engine: "jsonpath"\|"jq", expr }` |
| POST | `/api/dns/resolve`   | DNS lookups (returns TTLs) | `{ domain, types: ["A","MX",...], server? }` |
| POST | `/api/net/port`      | Single host/port TCP reachability probe (3s timeout) | `{ host, port }` |
| POST | `/api/image/convert` | Image conversion with resize and quality | `multipart: file, format, options?` |

CORS only allows loopback origins, so exposing the app with `--host 0.0.0.0` does not let arbitrary websites drive the API. JSON diffing is implemented entirely in the frontend and has no endpoint.

## Tech stack

| Layer | Choice |
|-------|--------|
| Web framework | Axum 0.8 + Tokio |
| Asset embedding | rust-embed |
| Middleware | tower-http (gzip, CORS, tracing) |
| DNS | hickory-resolver |
| Images | image |
| JSON ecosystem | serde_json / serde_yaml / toml / quick-xml / csv / jsonschema / serde_json_path / jaq |
| Frontend | Svelte 5 (runes) + Vite 6 + TypeScript |
| Frontend libraries | marked + DOMPurify, qrcode, crypto-js, jsencrypt, cron-parser, date-fns, sql-formatter |
| Fonts | JetBrains Mono (OFL-1.1, bundled); system stack for CJK |
| Release profile | thin LTO, codegen-units=1, strip |

## License

[Apache-2.0](./LICENSE)
