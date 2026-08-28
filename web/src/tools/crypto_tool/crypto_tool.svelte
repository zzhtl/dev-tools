<script lang="ts">
  import JSEncrypt from "jsencrypt";
  import CryptoJS from "crypto-js";
  import Button from "../../lib/ui/Button.svelte";
  import CopyButton from "../../lib/ui/CopyButton.svelte";
  import Panel from "../../lib/ui/Panel.svelte";
  import SplitPane from "../../lib/ui/SplitPane.svelte";
  import Tabs from "../../lib/ui/Tabs.svelte";
  import TextArea from "../../lib/ui/TextArea.svelte";
  import TextInput from "../../lib/ui/TextInput.svelte";
  import { toast } from "../../lib/toast.svelte";

  type Algorithm = "AES" | "TripleDES" | "DES" | "RSA" | "Base64";

  const ALGORITHMS: { value: Algorithm; label: string }[] = [
    { value: "AES", label: "AES" },
    { value: "TripleDES", label: "3DES" },
    { value: "DES", label: "DES" },
    { value: "RSA", label: "RSA" },
    { value: "Base64", label: "Base64" },
  ];

  let algorithm = $state<Algorithm>("AES");
  let direction = $state<"encrypt" | "decrypt">("encrypt");
  let input = $state("");
  let output = $state("");
  let error = $state("");

  // 对称算法用口令；RSA 用公私钥
  let passphrase = $state("");
  let publicKey = $state("");
  let privateKey = $state("");
  let generating = $state(false);

  const isRsa = $derived(algorithm === "RSA");
  const isBase64 = $derived(algorithm === "Base64");

  function encodeBase64(text: string): string {
    const bytes = new TextEncoder().encode(text);
    let binary = "";
    for (const byte of bytes) binary += String.fromCharCode(byte);
    return btoa(binary);
  }

  function decodeBase64(text: string): string {
    const binary = atob(text.trim());
    const bytes = Uint8Array.from(binary, (char) => char.charCodeAt(0));
    return new TextDecoder("utf-8", { fatal: true }).decode(bytes);
  }

  function generateKeyPair() {
    generating = true;
    // getKey 是同步的重计算，先让出一帧让 loading 态渲染出来
    setTimeout(() => {
      try {
        const encryptor = new JSEncrypt({ default_key_size: "2048" });
        encryptor.getKey();
        privateKey = encryptor.getPrivateKey();
        publicKey = encryptor.getPublicKey();
        toast.success("已生成 2048 位 RSA 密钥对");
      } catch (err) {
        toast.error(`生成密钥对失败：${err instanceof Error ? err.message : String(err)}`);
      } finally {
        generating = false;
      }
    }, 0);
  }

  function run() {
    error = "";
    output = "";
    if (!input.trim()) {
      error = direction === "encrypt" ? "请输入要加密的文本" : "请输入要解密的文本";
      return;
    }

    try {
      if (isBase64) {
        output = direction === "encrypt" ? encodeBase64(input) : decodeBase64(input);
      } else if (isRsa) {
        if (direction === "encrypt") {
          if (!publicKey.trim()) {
            error = "请先生成或粘贴 RSA 公钥";
            return;
          }
          const encryptor = new JSEncrypt();
          encryptor.setPublicKey(publicKey);
          const encrypted = encryptor.encrypt(input);
          if (!encrypted) {
            error = "RSA 加密失败：请检查公钥格式，或明文超出密钥长度限制";
            return;
          }
          output = encrypted;
        } else {
          if (!privateKey.trim()) {
            error = "请先生成或粘贴 RSA 私钥";
            return;
          }
          const decryptor = new JSEncrypt();
          decryptor.setPrivateKey(privateKey);
          const decrypted = decryptor.decrypt(input);
          if (!decrypted) {
            error = "RSA 解密失败：请检查私钥与密文是否匹配";
            return;
          }
          output = decrypted;
        }
      } else {
        if (!passphrase) {
          error = `请输入 ${algorithm === "TripleDES" ? "3DES" : algorithm} 口令`;
          return;
        }
        const cipher =
          algorithm === "AES"
            ? CryptoJS.AES
            : algorithm === "DES"
              ? CryptoJS.DES
              : CryptoJS.TripleDES;
        if (direction === "encrypt") {
          output = cipher.encrypt(input, passphrase).toString();
        } else {
          const decrypted = cipher.decrypt(input, passphrase).toString(CryptoJS.enc.Utf8);
          if (!decrypted) {
            error = "解密失败：口令不正确，或密文不是本工具生成的格式";
            return;
          }
          output = decrypted;
        }
      }
    } catch (err) {
      error = `${direction === "encrypt" ? "加密" : "解密"}失败：${err instanceof Error ? err.message : String(err)}`;
    }
  }

  function swap() {
    if (!output) return;
    input = output;
    output = "";
    direction = direction === "encrypt" ? "decrypt" : "encrypt";
  }
</script>

<div class="tool">
  <div class="toolbar">
    <Tabs bind:value={algorithm} tabs={ALGORITHMS} ariaLabel="算法" onchange={() => (output = "")} />
    <Tabs
      bind:value={direction}
      size="sm"
      tabs={[
        { value: "encrypt", label: isBase64 ? "编码" : "加密" },
        { value: "decrypt", label: isBase64 ? "解码" : "解密" },
      ]}
      ariaLabel="方向"
      onchange={() => (output = "")}
    />
    <div class="spacer"></div>
    <Button size="sm" onclick={swap} disabled={!output}>⇄ 结果转输入</Button>
    <Button
      size="sm"
      variant="ghost"
      onclick={() => {
        input = "";
        output = "";
        error = "";
      }}
    >
      清空
    </Button>
    <Button variant="primary" size="sm" onclick={run}>
      {direction === "encrypt" ? (isBase64 ? "编码" : "加密") : isBase64 ? "解码" : "解密"}
    </Button>
  </div>

  {#if !isBase64}
    <Panel title={isRsa ? "RSA 密钥对" : "口令"}>
      {#snippet actions()}
        {#if isRsa}
          <Button size="sm" onclick={generateKeyPair} loading={generating}>生成 2048 位密钥对</Button>
        {/if}
      {/snippet}

      {#if isRsa}
        <div class="keys">
          <label>
            <span>公钥（加密用）</span>
            <TextArea bind:value={publicKey} rows={4} placeholder="-----BEGIN PUBLIC KEY-----" ariaLabel="RSA 公钥" />
          </label>
          <label>
            <span>私钥（解密用）</span>
            <TextArea bind:value={privateKey} rows={4} placeholder="-----BEGIN RSA PRIVATE KEY-----" ariaLabel="RSA 私钥" />
          </label>
        </div>
        <p class="tip">
          RSA 只能加密远小于密钥长度的数据（2048 位约 245 字节）。大数据的常规做法是用 RSA 加密一个随机对称密钥，再用它加密正文。
        </p>
      {:else}
        <TextInput bind:value={passphrase} type="password" placeholder="输入口令" ariaLabel="口令" />
        <p class="tip">
          口令经 CryptoJS 的 OpenSSL 兼容 KDF 派生密钥与 IV，输出为 <code>Salted__</code> 格式，
          只能与同样使用该格式的实现互通；跨语言对接请改用明确指定 key/IV 的实现。
        </p>
      {/if}
    </Panel>
  {/if}

  <SplitPane>
    {#snippet left()}
      <Panel title={direction === "encrypt" ? "明文" : "密文"} grow>
        {#snippet actions()}
          <span class="count">{input.length} 字符</span>
        {/snippet}
        <TextArea bind:value={input} fill placeholder="在此输入…" ariaLabel="输入" />
      </Panel>
    {/snippet}

    {#snippet right()}
      <Panel title={direction === "encrypt" ? "密文" : "明文"} grow>
        {#snippet actions()}
          <span class="count">{output.length} 字符</span>
          <CopyButton value={() => output} disabled={!output} />
        {/snippet}
        <TextArea
          value={output}
          fill
          readonly
          invalid={!!error}
          placeholder="结果将显示在这里…"
          ariaLabel="输出"
        />
      </Panel>
    {/snippet}
  </SplitPane>

  {#if error}
    <p class="error">{error}</p>
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

  .spacer {
    flex: 1;
  }

  .count {
    font-size: var(--fs-xs);
    color: var(--color-text-muted);
    font-family: var(--font-mono);
  }

  .keys {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
    gap: var(--sp-3);
  }

  .keys label {
    display: flex;
    flex-direction: column;
    gap: var(--sp-1);
  }

  .keys span {
    font-size: var(--fs-sm);
    color: var(--color-text-secondary);
  }

  .tip {
    margin-top: var(--sp-2);
    font-size: var(--fs-xs);
    color: var(--color-text-muted);
    line-height: var(--lh-normal);
  }

  .tip code {
    font-family: var(--font-mono);
    background: var(--color-code-bg);
    padding: 0.1em 0.3em;
    border-radius: 3px;
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
