<script lang="ts">
  import CryptoJS from "crypto-js";
  import Button from "../../lib/ui/Button.svelte";
  import CopyButton from "../../lib/ui/CopyButton.svelte";
  import Panel from "../../lib/ui/Panel.svelte";
  import SplitPane from "../../lib/ui/SplitPane.svelte";
  import TextArea from "../../lib/ui/TextArea.svelte";
  import TextInput from "../../lib/ui/TextInput.svelte";
  import EmptyState from "../../lib/ui/EmptyState.svelte";

  let token = $state("");
  let secret = $state("");

  interface Decoded {
    header: Record<string, unknown> | null;
    payload: Record<string, unknown> | null;
    headerRaw: string;
    payloadRaw: string;
    signature: string;
    error?: string;
  }

  function b64urlToBytes(value: string): Uint8Array {
    let normalized = value.replace(/-/g, "+").replace(/_/g, "/");
    while (normalized.length % 4) normalized += "=";
    const binary = atob(normalized);
    return Uint8Array.from(binary, (char) => char.charCodeAt(0));
  }

  function b64urlDecodeText(value: string): string {
    return new TextDecoder().decode(b64urlToBytes(value));
  }

  function hmacB64url(alg: string, signingInput: string, key: string): string | null {
    let digest;
    if (alg === "HS256") digest = CryptoJS.HmacSHA256(signingInput, key);
    else if (alg === "HS384") digest = CryptoJS.HmacSHA384(signingInput, key);
    else if (alg === "HS512") digest = CryptoJS.HmacSHA512(signingInput, key);
    else return null;
    return CryptoJS.enc.Base64.stringify(digest)
      .replace(/\+/g, "-")
      .replace(/\//g, "_")
      .replace(/=+$/, "");
  }

  const decoded = $derived.by((): Decoded | null => {
    const raw = token.trim();
    if (!raw) return null;
    const parts = raw.split(".");
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
    } catch (err) {
      return { ...empty, error: `解码失败：${(err as Error).message}` };
    }
  });

  const nowSec = Math.floor(Date.now() / 1000);

  const timeClaims = $derived.by(() => {
    const payload = decoded?.payload;
    if (!payload) return [];
    const labels: Record<string, string> = {
      iat: "签发时间 iat",
      nbf: "生效时间 nbf",
      exp: "过期时间 exp",
    };
    return Object.keys(labels)
      .filter((key) => typeof payload[key] === "number")
      .map((key) => {
        const sec = payload[key] as number;
        return { key, label: labels[key], when: new Date(sec * 1000).toLocaleString(), sec };
      });
  });

  const expired = $derived(
    typeof decoded?.payload?.exp === "number" && nowSec > (decoded.payload.exp as number),
  );
  const notYetValid = $derived(
    typeof decoded?.payload?.nbf === "number" && nowSec < (decoded.payload.nbf as number),
  );

  const verification = $derived.by(() => {
    if (!decoded || decoded.error || !decoded.signature) return null;
    const alg = (decoded.header?.alg as string) || "";
    if (!secret.trim()) return { status: "no-secret" as const, alg };
    if (!["HS256", "HS384", "HS512"].includes(alg)) return { status: "unsupported" as const, alg };
    const signingInput = token.trim().split(".").slice(0, 2).join(".");
    const expected = hmacB64url(alg, signingInput, secret);
    return {
      status: expected === decoded.signature ? ("valid" as const) : ("invalid" as const),
      alg,
    };
  });

  function loadExample() {
    // header {alg:HS256,typ:JWT} . payload . signature（secret 为 "dev-tools"）
    token =
      "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9" +
      ".eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkRldiBUb29scyIsImFkbWluIjp0cnVlLCJpYXQiOjE3MTYyMzkwMjIsImV4cCI6MjA2MzA3NTAyMn0" +
      ".q1T8I6v1DzOS-bCLEiTitk1AnfRU5Rce5AOFYrbxtdQ";
    secret = "dev-tools";
  }
</script>

<div class="tool">
  <div class="toolbar">
    <Button size="sm" onclick={loadExample}>示例</Button>
    <Button
      size="sm"
      variant="ghost"
      onclick={() => {
        token = "";
        secret = "";
      }}
    >
      清空
    </Button>

    {#if verification}
      <span class="badge {verification.status}">
        {#if verification.status === "valid"}✓ 签名有效（{verification.alg}）
        {:else if verification.status === "invalid"}✕ 签名不匹配（{verification.alg}）
        {:else if verification.status === "unsupported"}{verification.alg || "未知算法"} 仅支持解码，本工具不验签
        {:else}填入密钥可验签 HS256 / 384 / 512{/if}
      </span>
    {/if}

    {#if expired}<span class="badge invalid">已过期</span>{/if}
    {#if notYetValid}<span class="badge warn">尚未生效</span>{/if}
  </div>

  <Panel title="JWT">
    {#snippet actions()}
      <CopyButton value={() => token} disabled={!token} />
    {/snippet}
    <TextArea bind:value={token} rows={4} placeholder="粘贴 JWT：header.payload.signature" ariaLabel="JWT" />
    <div class="secret">
      <TextInput
        bind:value={secret}
        type="password"
        placeholder="HMAC 密钥（仅用于本地验签，不会发送到任何地方）"
        ariaLabel="HMAC 密钥"
      />
    </div>
  </Panel>

  {#if decoded?.error}
    <p class="error">{decoded.error}</p>
  {:else if !decoded}
    <Panel grow>
      <EmptyState icon="🔑" title="粘贴一个 JWT 开始解析" hint="解码与验签全部在本地完成" />
    </Panel>
  {:else}
    <SplitPane ratio={0.42}>
      {#snippet left()}
        <Panel title="Header" grow>
          {#snippet actions()}
            <CopyButton value={() => decoded?.headerRaw ?? ""} />
          {/snippet}
          <pre>{decoded.headerRaw}</pre>
        </Panel>
      {/snippet}

      {#snippet right()}
        <Panel title="Payload" grow>
          {#snippet actions()}
            <CopyButton value={() => decoded?.payloadRaw ?? ""} />
          {/snippet}
          <pre>{decoded.payloadRaw}</pre>
          {#if timeClaims.length > 0}
            <ul class="claims">
              {#each timeClaims as claim (claim.key)}
                <li>
                  <span>{claim.label}</span>
                  <code>{claim.when}</code>
                </li>
              {/each}
            </ul>
          {/if}
        </Panel>
      {/snippet}
    </SplitPane>
  {/if}
</div>

<style>
  .tool {
    display: flex;
    flex-direction: column;
    gap: var(--sp-3);
    flex: 1;
    min-height: 0;
  }

  .toolbar {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: var(--sp-2);
  }

  .badge {
    padding: 0.2rem var(--sp-3);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-full);
    font-size: var(--fs-xs);
    color: var(--color-text-muted);
  }

  .badge.valid {
    border-color: var(--color-success-border);
    background: var(--color-success-bg);
    color: var(--color-success);
  }

  .badge.invalid {
    border-color: var(--color-danger-border);
    background: var(--color-danger-bg);
    color: var(--color-danger);
  }

  .badge.warn {
    border-color: var(--color-warning-border);
    background: var(--color-warning-bg);
    color: var(--color-warning);
  }

  .secret {
    margin-top: var(--sp-2);
  }

  pre {
    padding: var(--sp-3);
    border-radius: var(--radius-sm);
    background: var(--color-code-bg);
    font-family: var(--font-mono);
    font-size: var(--fs-sm);
    color: var(--color-text-secondary);
    white-space: pre-wrap;
    word-break: break-all;
  }

  .claims {
    list-style: none;
    display: flex;
    flex-direction: column;
    gap: var(--sp-1);
    margin-top: var(--sp-3);
  }

  .claims li {
    display: flex;
    justify-content: space-between;
    gap: var(--sp-3);
    padding: var(--sp-1) var(--sp-2);
    border-radius: var(--radius-sm);
    background: var(--color-surface-2);
    font-size: var(--fs-sm);
    color: var(--color-text-secondary);
  }

  .claims code {
    font-family: var(--font-mono);
    color: var(--color-text);
  }

  .error {
    padding: var(--sp-2) var(--sp-3);
    border: 1px solid var(--color-danger-border);
    border-radius: var(--radius-sm);
    background: var(--color-danger-bg);
    color: var(--color-danger);
    font-size: var(--fs-sm);
  }
</style>
