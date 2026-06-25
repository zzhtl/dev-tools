const DEFAULT_TIMEOUT = 15000;

async function withTimeout(
  run: (signal: AbortSignal) => Promise<Response>,
  timeout = DEFAULT_TIMEOUT,
): Promise<Response> {
  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), timeout);
  try {
    const res = await run(controller.signal);
    if (!res.ok) {
      const text = await res.text().catch(() => "");
      throw new Error(text || `请求失败（HTTP ${res.status}）`);
    }
    return res;
  } catch (e) {
    if (e instanceof DOMException && e.name === "AbortError") {
      throw new Error(`请求超时（超过 ${Math.round(timeout / 1000)} 秒）`);
    }
    if (e instanceof TypeError) {
      throw new Error("无法连接到本地服务，请确认 dev-tools 正在运行");
    }
    throw e;
  } finally {
    clearTimeout(timer);
  }
}

export async function apiJson<T = unknown>(path: string, body: unknown): Promise<T> {
  const res = await withTimeout((signal) =>
    fetch(path, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(body),
      signal,
    }),
  );
  return (await res.json()) as T;
}

export async function apiMultipart(path: string, form: FormData): Promise<Response> {
  return withTimeout((signal) => fetch(path, { method: "POST", body: form, signal }));
}
