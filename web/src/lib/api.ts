export async function apiJson<T = unknown>(path: string, body: unknown): Promise<T> {
  const res = await fetch(path, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  });
  if (!res.ok) throw new Error(await res.text());
  return (await res.json()) as T;
}

export async function apiMultipart(path: string, form: FormData): Promise<Response> {
  const res = await fetch(path, { method: "POST", body: form });
  if (!res.ok) throw new Error(await res.text());
  return res;
}
