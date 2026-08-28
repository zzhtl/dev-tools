#!/bin/sh
# dev-tools 安装脚本
#
#   curl -fsSL https://raw.githubusercontent.com/zzhtl/dev-tools/main/install.sh | sh
#
# 环境变量：
#   DEV_TOOLS_VERSION      指定版本（如 v0.3.0），默认取最新 release
#   DEV_TOOLS_INSTALL_DIR  安装目录，默认 ~/.local/bin（不需要 sudo）

set -eu

REPO="zzhtl/dev-tools"
BIN="dev-tools"

info() { printf '\033[0;36m==>\033[0m %s\n' "$1"; }
warn() { printf '\033[0;33m警告:\033[0m %s\n' "$1" >&2; }
die() {
  printf '\033[0;31m错误:\033[0m %s\n' "$1" >&2
  exit 1
}

usage() {
  cat <<'USAGE'
用法: install.sh [选项]

选项:
  --dir <路径>       安装目录（默认 ~/.local/bin）
  --version <tag>    安装指定版本（如 v0.3.0）
  -h, --help         显示此帮助
USAGE
}

install_dir="${DEV_TOOLS_INSTALL_DIR:-$HOME/.local/bin}"
version="${DEV_TOOLS_VERSION:-}"

while [ $# -gt 0 ]; do
  case "$1" in
    --dir)
      [ $# -ge 2 ] || die "--dir 需要一个参数"
      install_dir="$2"
      shift 2
      ;;
    --version)
      [ $# -ge 2 ] || die "--version 需要一个参数"
      version="$2"
      shift 2
      ;;
    -h | --help)
      usage
      exit 0
      ;;
    *) die "未知参数: $1（用 --help 查看用法）" ;;
  esac
done

# ── 依赖 ──
if command -v curl >/dev/null 2>&1; then
  fetch() { curl -fsSL "$1"; }
  fetch_to() { curl -fsSL -o "$2" "$1"; }
elif command -v wget >/dev/null 2>&1; then
  fetch() { wget -qO- "$1"; }
  fetch_to() { wget -qO "$2" "$1"; }
else
  die "需要 curl 或 wget"
fi

if command -v sha256sum >/dev/null 2>&1; then
  checksum() { sha256sum "$1" | cut -d' ' -f1; }
elif command -v shasum >/dev/null 2>&1; then
  checksum() { shasum -a 256 "$1" | cut -d' ' -f1; }
else
  die "需要 sha256sum 或 shasum 来校验下载内容"
fi

# ── 平台识别 ──
os="$(uname -s)"
arch="$(uname -m)"

case "$os" in
  Linux) os_id="unknown-linux-musl" ;;
  Darwin) os_id="apple-darwin" ;;
  *) die "不支持的操作系统: $os（Windows 请使用 install.ps1）" ;;
esac

case "$arch" in
  x86_64 | amd64) arch_id="x86_64" ;;
  aarch64 | arm64) arch_id="aarch64" ;;
  *) die "不支持的架构: $arch" ;;
esac

target="${arch_id}-${os_id}"

# ── 版本 ──
if [ -z "$version" ]; then
  info "查询最新版本…"
  version="$(fetch "https://api.github.com/repos/$REPO/releases/latest" |
    grep -o '"tag_name"[[:space:]]*:[[:space:]]*"[^"]*"' |
    head -n 1 | cut -d'"' -f4)"
  [ -n "$version" ] || die "无法获取最新版本，请用 --version 指定，或检查网络"
fi

archive="dev-tools-${version}-${target}.tar.gz"
base_url="https://github.com/$REPO/releases/download/$version"

info "版本 $version · 平台 $target"

# ── 下载并校验 ──
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT INT TERM

info "下载 $archive"
fetch_to "$base_url/$archive" "$tmp/$archive" || die "下载失败: $base_url/$archive"

info "校验 SHA256"
fetch_to "$base_url/SHA256SUMS" "$tmp/SHA256SUMS" || die "下载 SHA256SUMS 失败"

expected="$(grep " $archive\$" "$tmp/SHA256SUMS" | cut -d' ' -f1 || true)"
[ -n "$expected" ] || die "SHA256SUMS 中没有 $archive 的记录"

actual="$(checksum "$tmp/$archive")"
if [ "$expected" != "$actual" ]; then
  die "校验和不匹配！期望 $expected，实际 $actual。请勿使用该文件。"
fi

# ── 安装 ──
tar -xzf "$tmp/$archive" -C "$tmp"
extracted="$tmp/dev-tools-${version}-${target}/$BIN"
[ -f "$extracted" ] || die "压缩包中没有找到 $BIN"

mkdir -p "$install_dir"
install -m 0755 "$extracted" "$install_dir/$BIN" 2>/dev/null || {
  cp "$extracted" "$install_dir/$BIN"
  chmod 0755 "$install_dir/$BIN"
}

info "已安装到 $install_dir/$BIN"

# ── PATH 提示 ──
case ":$PATH:" in
  *":$install_dir:"*)
    printf '\n运行 \033[0;36m%s\033[0m 即可启动（默认监听 127.0.0.1:8927 并打开浏览器）\n' "$BIN"
    ;;
  *)
    warn "$install_dir 不在 PATH 中"
    printf '把下面这行加进你的 shell 配置（~/.bashrc 或 ~/.zshrc）：\n\n'
    printf '  export PATH="%s:$PATH"\n\n' "$install_dir"
    printf '或直接运行：\033[0;36m%s/%s\033[0m\n' "$install_dir" "$BIN"
    ;;
esac
