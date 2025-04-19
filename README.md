# Dev Tools - 开发者工具集

基于Tauri + SvelteKit + TypeScript构建的桌面应用，提供多种实用开发工具。

## 🛠️ 功能列表

| 工具名称       | 功能描述                     | 图标 |
|----------------|----------------------------|------|
| JSON工具       | JSON格式化/压缩/验证        | 📋   |
| Cron表达式     | Cron表达式解析和验证        | ⏰   |
| 编码加解密      | 常见编码转换和加解密工具     | 🔒   |
| Linux命令      | 常用Linux命令查询           | 💻   |
| 时间转换        | 时间戳与日期格式互转        | 🕒   |
| 二维码生成      | 文本生成二维码图片          | 📱   |
| 图片转Base64   | 图片与Base64互转            | 📊   |
| DNS解析       | DNS记录查询支持 A/AAAA/MX/CNAME/TXT/NS | 🌐   |

## 🚀 技术栈

- 前端框架: [SvelteKit](https://kit.svelte.dev/)
- UI组件: 原生CSS
- 打包工具: [Tauri](https://tauri.app/)
- 开发语言: TypeScript + Rust

## ⚙️ 开发环境搭建

```bash
# 安装依赖
pnpm install

# 开发模式运行
cargo tauri dev

# 生产环境构建
cargo tauri build
```
