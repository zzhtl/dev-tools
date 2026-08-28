/**
 * 复制到剪贴板。
 *
 * navigator.clipboard 只在安全上下文（https 或 localhost）可用。
 * 本工具支持 `--host 0.0.0.0`，用户经局域网 IP（http://192.168.x.x:8927）访问时
 * 该 API 不存在，旧实现会静默失败 —— 因此必须保留 execCommand 回退。
 */
export async function copyText(text: string): Promise<boolean> {
  if (window.isSecureContext && navigator.clipboard) {
    try {
      await navigator.clipboard.writeText(text);
      return true;
    } catch {
      /* 落到下面的回退 */
    }
  }
  return legacyCopy(text);
}

function legacyCopy(text: string): boolean {
  try {
    const ta = document.createElement("textarea");
    ta.value = text;
    ta.setAttribute("readonly", "");
    ta.style.position = "fixed";
    ta.style.top = "-9999px";
    ta.style.opacity = "0";
    document.body.appendChild(ta);
    ta.select();
    ta.setSelectionRange(0, ta.value.length);
    const ok = document.execCommand("copy");
    document.body.removeChild(ta);
    return ok;
  } catch {
    return false;
  }
}
