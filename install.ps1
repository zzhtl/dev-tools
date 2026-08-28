<#
.SYNOPSIS
    dev-tools 安装脚本（Windows）

.EXAMPLE
    irm https://raw.githubusercontent.com/zzhtl/dev-tools/main/install.ps1 | iex

.PARAMETER Version
    指定版本（如 v0.3.0），默认取最新 release

.PARAMETER Dir
    安装目录，默认 %LOCALAPPDATA%\Programs\dev-tools
#>
[CmdletBinding()]
param(
    [string]$Version = $env:DEV_TOOLS_VERSION,
    [string]$Dir = $(if ($env:DEV_TOOLS_INSTALL_DIR) { $env:DEV_TOOLS_INSTALL_DIR } else { "$env:LOCALAPPDATA\Programs\dev-tools" })
)

$ErrorActionPreference = 'Stop'
$repo = 'zzhtl/dev-tools'

function Write-Step($msg) { Write-Host "==> $msg" -ForegroundColor Cyan }

if (-not [Environment]::Is64BitOperatingSystem) {
    throw '仅提供 64 位版本'
}
$target = 'x86_64-pc-windows-msvc'

if (-not $Version) {
    Write-Step '查询最新版本…'
    $latest = Invoke-RestMethod "https://api.github.com/repos/$repo/releases/latest"
    $Version = $latest.tag_name
    if (-not $Version) { throw '无法获取最新版本，请用 -Version 指定' }
}

$archive = "dev-tools-$Version-$target.zip"
$baseUrl = "https://github.com/$repo/releases/download/$Version"

Write-Step "版本 $Version - 平台 $target"

$tmp = Join-Path ([System.IO.Path]::GetTempPath()) ([System.Guid]::NewGuid().ToString())
New-Item -ItemType Directory -Path $tmp -Force | Out-Null

try {
    Write-Step "下载 $archive"
    Invoke-WebRequest "$baseUrl/$archive" -OutFile "$tmp\$archive"

    Write-Step '校验 SHA256'
    Invoke-WebRequest "$baseUrl/SHA256SUMS" -OutFile "$tmp\SHA256SUMS"

    $pattern = "\s$([regex]::Escape($archive))$"
    $line = Get-Content "$tmp\SHA256SUMS" | Where-Object { $_ -match $pattern } | Select-Object -First 1
    if (-not $line) { throw "SHA256SUMS 中没有 $archive 的记录" }

    $expected = ($line -split '\s+')[0]
    $actual = (Get-FileHash "$tmp\$archive" -Algorithm SHA256).Hash
    if ($expected.ToLower() -ne $actual.ToLower()) {
        throw "校验和不匹配！期望 $expected，实际 $actual。请勿使用该文件。"
    }

    Expand-Archive "$tmp\$archive" -DestinationPath $tmp -Force
    $exe = Join-Path $tmp "dev-tools-$Version-$target\dev-tools.exe"
    if (-not (Test-Path $exe)) { throw '压缩包中没有找到 dev-tools.exe' }

    New-Item -ItemType Directory -Path $Dir -Force | Out-Null
    Copy-Item $exe (Join-Path $Dir 'dev-tools.exe') -Force
    Write-Step "已安装到 $Dir\dev-tools.exe"

    $userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
    if ($userPath -notlike "*$Dir*") {
        Write-Host "警告: $Dir 不在 PATH 中" -ForegroundColor Yellow
        Write-Host '把它加入用户 PATH（新开终端后生效）：' -ForegroundColor Yellow
        Write-Host ('  [Environment]::SetEnvironmentVariable(''Path'', "$env:Path;' + $Dir + '", ''User'')')
    }
    else {
        Write-Host '运行 dev-tools 即可启动（默认监听 127.0.0.1:8927 并打开浏览器）' -ForegroundColor Cyan
    }
}
finally {
    Remove-Item $tmp -Recurse -Force -ErrorAction SilentlyContinue
}
