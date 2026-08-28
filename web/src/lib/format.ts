/** 字节数转可读体积。原先在 base64_tool 与 image_tool 各写了一份。 */
export function formatFileSize(bytes: number): string {
  if (!Number.isFinite(bytes) || bytes < 0) return "-";
  if (bytes === 0) return "0 B";
  const units = ["B", "KB", "MB", "GB", "TB"];
  const i = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), units.length - 1);
  const value = bytes / Math.pow(1024, i);
  return `${i === 0 ? value : value.toFixed(2)} ${units[i]}`;
}

/** HTML 转义。原先在 json_tool 与 regex_tool 各写了一份。 */
export function escapeHtml(text: string): string {
  return text
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#39;");
}

/** 数字千分位。 */
export function formatCount(n: number): string {
  return n.toLocaleString("zh-CN");
}
