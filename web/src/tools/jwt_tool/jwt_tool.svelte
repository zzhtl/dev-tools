<script lang="ts">
  import CryptoJS from "crypto-js";

  let token = $state("");
  let secret = $state("");
  let copied = $state<string | null>(null);

  interface Decoded {
    header: Record<string, unknown> | null;
    payload: Record<string, unknown> | null;
    headerRaw: string;
    payloadRaw: string;
    signature: string;
    error?: string;
  }

  function b64urlToBytes(s: string): Uint8Array {
    let b = s.replace(/-/g, "+").replace(/_/g, "/");
    while (b.length % 4) b += "=";
    const bin = atob(b);
    const bytes = new Uint8Array(bin.length);
    for (let i = 0; i < bin.length; i++) bytes[i] = bin.charCodeAt(i);
    return bytes;
  }

  function b64urlDecodeText(s: string): string {
    return new TextDecoder().decode(b64urlToBytes(s));
  }

  function hmacB64url(alg: string, signingInput: string, key: string): string | null {
    let h;
    if (alg === "HS256") h = CryptoJS.HmacSHA256(signingInput, key);
    else if (alg === "HS384") h = CryptoJS.HmacSHA384(signingInput, key);
    else if (alg === "HS512") h = CryptoJS.HmacSHA512(signingInput, key);
    else return null;
    return CryptoJS.enc.Base64.stringify(h)
      .replace(/\+/g, "-")
      .replace(/\//g, "_")
      .replace(/=+$/, "");
  }

  const decoded = $derived.by((): Decoded | null => {
    const t = token.trim();
    if (!t) return null;
    const parts = t.split(".");
    const empty = { header: null, payload: null, headerRaw: "", payloadRaw: "", signature: "" };
    if (parts.length < 2) {
      return { ...empty, error: "不是有效的 JWT（应为 header.payload.signature 三段）" };
    }
    try {
      const header = JSON.parse(b64urlDecodeText(parts[0]));
      const payload = JSON.parse(b64urlDecodeText(parts[1]));
      return {
        header,
        payload,
        headerRaw: JSON.stringify(header, null, 2),
        payloadRaw: JSON.stringify(payload, null, 2),
        signature: parts[2] ?? "",
      };
    } catch (e) {
      return { ...empty, error: "解码失败：" + (e as Error).message };
    }
  });

  const nowSec = Math.floor(Date.now() / 1000);

  // 时间类声明速览
  const timeClaims = $derived.by(() => {
    const p = decoded?.payload;
    if (!p) return [];
    const labels: Record<string, string> = {
      exp: "过期时间 (exp)",
      iat: "签发时间 (iat)",
      nbf: "生效时间 (nbf)",
    };
    return Object.keys(labels)
      .filter((k) => typeof p[k] === "number")
      .map((k) => {
        const sec = p[k] as number;
        return { key: k, label: labels[k], when: new Date(sec * 1000).toLocaleString(), sec };
      });
  });

  const expired = $derived.by(() => {
    const exp = decoded?.payload?.exp;
    return typeof exp === "number" && nowSec > exp;
  });

  const notYetValid = $derived.by(() => {
    const nbf = decoded?.payload?.nbf;
    return typeof nbf === "number" && nowSec < nbf;
  });

  const verification = $derived.by(() => {
    if (!decoded || decoded.error || !decoded.signature) return null;
    const alg = (decoded.header?.alg as string) || "";
    if (!secret.trim()) return { status: "no-secret" as const, alg };
    if (!["HS256", "HS384", "HS512"].includes(alg)) {
      return { status: "unsupported" as const, alg };
    }
    const signingInput = token.trim().split(".").slice(0, 2).join(".");
    const expected = hmacB64url(alg, signingInput, secret);
    return { status: expected === decoded.signature ? ("valid" as const) : ("invalid" as const), alg };
  });

  async function copy(text: string, tag: string) {
    if (!text) return;
    try {
      await navigator.clipboard.writeText(text);
      copied = tag;
      setTimeout(() => (copied = null), 1500);
    } catch (e) {
      console.error("复制失败:", e);
    }
  }

  function loadExample() {
    // header {alg:HS256,typ:JWT} . payload . sig（secret 为 "dev-tools"）
    token =
      "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9" +
      ".eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkRldiBUb29scyIsImFkbWluIjp0cnVlLCJpYXQiOjE3MTYyMzkwMjIsImV4cCI6MjA2MzA3NTAyMn0" +
      ".q1T8I6v1DzOS-bCLEiTitk1AnfRU5Rce5AOFYrbxtdQ";
    secret = "dev-tools";
  }

  function clearAll() {
    token = "";
    secret = "";
  }
</script>

<div class="jwt-tool">
  <div class="controls">
    <button class="primary-btn" onclick={loadExample}>📋 示例</button>
    <button class="secondary-btn" onclick={clearAll}>🗑️ 清空</button>
  </div>

  <div class="field">
    <label for="jwt-token">JWT Token</label>
    <textarea
      id="jwt-token"
      bind:value={token}
      placeholder="粘贴 JWT，例如 eyJhbGci...（header.payload.signature）"
      spellcheck="false"
    ></textarea>
  </div>

  <div class="field">
    <label for="jwt-secret">密钥（HS256/384/512 验签，可选）</label>
    <input id="jwt-secret" type="text" bind:value={secret} placeholder="HMAC 密钥" />
  </div>

  {#if decoded?.error}
    <div class="jwt-error">⚠️ {decoded.error}</div>
  {:else if decoded}
    <!-- 状态徽章 -->
    <div class="badges">
      {#if decoded.header?.alg}
        <span class="badge alg">{decoded.header.alg}</span>
      {/if}
      {#if expired}
        <span class="badge bad">已过期</span>
      {:else if decoded.payload?.exp}
        <span class="badge ok">未过期</span>
      {/if}
      {#if notYetValid}
        <span class="badge bad">尚未生效</span>
      {/if}
      {#if verification}
        {#if verification.status === "valid"}
          <span class="badge ok">✓ 签名有效</span>
        {:else if verification.status === "invalid"}
          <span class="badge bad">✕ 签名无效</span>
        {:else if verification.status === "unsupported"}
          <span class="badge muted">{verification.alg} 验签需公钥（暂不支持）</span>
        {:else}
          <span class="badge muted">输入密钥以验签</span>
        {/if}
      {/if}
    </div>

    {#if timeClaims.length}
      <div class="time-claims">
        {#each timeClaims as tc}
          <div class="tc-item">
            <span class="tc-label">{tc.label}</span>
            <span class="tc-when">{tc.when}</span>
          </div>
        {/each}
      </div>
    {/if}

    <div class="panels">
      <div class="panel">
        <div class="panel-header">
          <span>Header</span>
          <button class="copy-btn" onclick={() => copy(decoded!.headerRaw, "h")}>
            {copied === "h" ? "✓" : "📋"}
          </button>
        </div>
        <pre class="panel-body header-body">{decoded.headerRaw}</pre>
      </div>
      <div class="panel">
        <div class="panel-header">
          <span>Payload</span>
          <button class="copy-btn" onclick={() => copy(decoded!.payloadRaw, "p")}>
            {copied === "p" ? "✓" : "📋"}
          </button>
        </div>
        <pre class="panel-body payload-body">{decoded.payloadRaw}</pre>
      </div>
    </div>

    {#if decoded.signature}
      <div class="sig">
        <span class="sig-label">Signature</span>
        <code class="sig-val">{decoded.signature}</code>
      </div>
    {/if}
  {/if}

  <div class="info-section">
    <h3>关于 JWT</h3>
    <p>
      JWT 由 <span class="i-h">header</span>.<span class="i-p">payload</span>.<span class="i-s"
        >signature</span
      > 三段 Base64URL 组成。解码与验签均在本机完成，密钥不会上传。HS256/384/512 可用密钥直接验签；RS/ES
      等非对称算法需公钥，当前仅解码不验签。
    </p>
  </div>
</div>

<style>
  .jwt-tool {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .controls {
    display: flex;
    gap: 0.75rem;
  }
  .secondary-btn {
    background: var(--bg-dark) !important;
    border: 1px solid var(--border) !important;
    color: var(--text-secondary) !important;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
  }
  .field label {
    font-size: 0.8rem;
    color: var(--text-secondary);
  }
  .field textarea {
    min-height: 90px;
    resize: vertical;
    font-family: "JetBrains Mono", monospace !important;
    font-size: 0.85rem;
    word-break: break-all;
  }

  .jwt-error {
    padding: 0.6rem 0.8rem;
    background: rgba(239, 68, 68, 0.1);
    border: 1px solid #ef4444;
    border-radius: var(--radius-sm);
    color: #fca5a5;
    font-size: 0.85rem;
  }

  .badges {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
  }
  .badge {
    padding: 0.25rem 0.6rem;
    border-radius: 999px;
    font-size: 0.78rem;
    font-weight: 600;
  }
  .badge.alg {
    background: rgba(99, 102, 241, 0.2);
    color: var(--primary-light);
    font-family: "JetBrains Mono", monospace;
  }
  .badge.ok {
    background: rgba(16, 185, 129, 0.15);
    color: var(--accent-green);
  }
  .badge.bad {
    background: rgba(248, 113, 113, 0.15);
    color: #f87171;
  }
  .badge.muted {
    background: var(--bg-hover);
    color: var(--text-muted);
  }

  .time-claims {
    display: flex;
    flex-wrap: wrap;
    gap: 0.75rem;
  }
  .tc-item {
    display: flex;
    flex-direction: column;
    gap: 0.15rem;
    padding: 0.5rem 0.75rem;
    background: var(--bg-dark);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
  }
  .tc-label {
    font-size: 0.72rem;
    color: var(--text-muted);
  }
  .tc-when {
    font-size: 0.85rem;
    color: var(--text-primary);
    font-family: "JetBrains Mono", monospace;
  }

  .panels {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 0.75rem;
  }
  .panel {
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    overflow: hidden;
  }
  .panel-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.4rem 0.7rem;
    background: var(--bg-dark);
    border-bottom: 1px solid var(--border);
    font-size: 0.8rem;
    color: var(--text-secondary);
  }
  .copy-btn {
    padding: 0.15rem 0.4rem !important;
    background: var(--bg-card) !important;
    border: 1px solid var(--border) !important;
    font-size: 0.75rem;
  }
  .panel-body {
    margin: 0;
    padding: 0.7rem;
    background: var(--bg-card);
    font-family: "JetBrains Mono", monospace;
    font-size: 0.82rem;
    max-height: 260px;
    overflow: auto;
    white-space: pre-wrap;
    word-break: break-word;
  }
  .header-body {
    color: var(--accent);
  }
  .payload-body {
    color: var(--accent-green);
  }

  .sig {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    padding: 0.5rem 0.75rem;
    background: var(--bg-dark);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
  }
  .sig-label {
    font-size: 0.75rem;
    color: var(--text-muted);
    white-space: nowrap;
  }
  .sig-val {
    font-size: 0.8rem;
    color: #fbbf24;
    word-break: break-all;
  }

  .info-section {
    padding: 1rem;
    background: var(--bg-dark);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
  }
  .info-section h3 {
    margin-bottom: 0.5rem;
  }
  .info-section p {
    font-size: 0.85rem;
    line-height: 1.7;
    color: var(--text-secondary);
  }
  .i-h {
    color: var(--accent);
  }
  .i-p {
    color: var(--accent-green);
  }
  .i-s {
    color: #fbbf24;
  }
</style>
