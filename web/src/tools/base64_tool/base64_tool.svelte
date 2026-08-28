<script lang="ts">
  import Button from "../../lib/ui/Button.svelte";
  import CopyButton from "../../lib/ui/CopyButton.svelte";
  import DropZone from "../../lib/ui/DropZone.svelte";
  import Panel from "../../lib/ui/Panel.svelte";
  import SplitPane from "../../lib/ui/SplitPane.svelte";
  import Tabs from "../../lib/ui/Tabs.svelte";
  import TextArea from "../../lib/ui/TextArea.svelte";
  import { formatFileSize } from "../../lib/format";
  import { toast } from "../../lib/toast.svelte";

  let source = $state<"text" | "file">("text");
  let mode = $state<"encode" | "decode">("encode");
  let urlSafe = $state(false);
  let input = $state("");

  // ── 文本 ⇄ Base64 ──
  // 用 TextEncoder 而不是 btoa(text)：btoa 只接受 Latin-1，中文会直接抛异常
  function encodeText(text: string): string {
    const bytes = new TextEncoder().encode(text);
    let binary = "";
    const CHUNK = 0x8000;
    for (let i = 0; i < bytes.length; i += CHUNK) {
      binary += String.fromCharCode(...bytes.subarray(i, i + CHUNK));
    }
    const encoded = btoa(binary);
    return urlSafe ? encoded.replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "") : encoded;
  }

  function decodeText(encoded: string): string {
    let normalized = encoded.trim().replace(/\s+/g, "").replace(/-/g, "+").replace(/_/g, "/");
    while (normalized.length % 4 !== 0) normalized += "=";
    const binary = atob(normalized);
    const bytes = Uint8Array.from(binary, (char) => char.charCodeAt(0));
    // fatal 让非法 UTF-8 直接报错，而不是静默产出乱码
    return new TextDecoder("utf-8", { fatal: true }).decode(bytes);
  }

  const textResult = $derived.by(() => {
    if (!input.trim()) return { text: "", error: "" };
    try {
      return { text: mode === "encode" ? encodeText(input) : decodeText(input), error: "" };
    } catch (err) {
      return {
        text: "",
        error: mode === "decode" ? "不是合法的 Base64 或不是 UTF-8 文本" : String(err),
      };
    }
  });

  // ── 文件 → Base64 ──
  interface FileResult {
    name: string;
    size: number;
    type: string;
    dataUrl: string;
  }

  let file = $state<FileResult | null>(null);
  let reading = $state(false);

  const MAX_FILE_BYTES = 20 * 1024 * 1024;

  function readFile(files: File[]) {
    const picked = files[0];
    if (!picked) return;
    if (picked.size > MAX_FILE_BYTES) {
      toast.error(`文件超过 ${formatFileSize(MAX_FILE_BYTES)}，Base64 会膨胀约 33%，已拒绝`);
      return;
    }
    reading = true;
    const reader = new FileReader();
    reader.onload = () => {
      file = {
        name: picked.name,
        size: picked.size,
        type: picked.type || "application/octet-stream",
        dataUrl: String(reader.result ?? ""),
      };
      reading = false;
    };
    reader.onerror = () => {
      reading = false;
      toast.error("读取文件失败");
    };
    reader.readAsDataURL(picked);
  }

  const rawBase64 = $derived(file ? file.dataUrl.slice(file.dataUrl.indexOf(",") + 1) : "");
  const isImage = $derived(!!file && file.type.startsWith("image/"));
</script>

<div class="tool">
  <div class="toolbar">
    <Tabs
      bind:value={source}
      tabs={[
        { value: "text", label: "文本" },
        { value: "file", label: "文件 / 图片" },
      ]}
      ariaLabel="输入来源"
    />

    {#if source === "text"}
      <Tabs
        bind:value={mode}
        size="sm"
        tabs={[
          { value: "encode", label: "编码" },
          { value: "decode", label: "解码" },
        ]}
        ariaLabel="编解码方向"
      />
      <label class="check">
        <input type="checkbox" bind:checked={urlSafe} />
        URL-safe
      </label>
    {/if}

    <div class="spacer"></div>

    {#if source === "text"}
      <Button size="sm" variant="ghost" onclick={() => (input = "")}>清空</Button>
    {:else if file}
      <Button size="sm" variant="ghost" onclick={() => (file = null)}>移除文件</Button>
    {/if}
  </div>

  {#if source === "text"}
    <SplitPane>
      {#snippet left()}
        <Panel title={mode === "encode" ? "原文" : "Base64"} grow>
          {#snippet actions()}
            <span class="count">{input.length} 字符</span>
          {/snippet}
          <TextArea
            bind:value={input}
            fill
            placeholder={mode === "encode" ? "输入要编码的文本…" : "粘贴 Base64…"}
            ariaLabel="输入"
          />
        </Panel>
      {/snippet}

      {#snippet right()}
        <Panel title={mode === "encode" ? "Base64" : "原文"} grow>
          {#snippet actions()}
            <span class="count">{textResult.text.length} 字符</span>
            <CopyButton value={() => textResult.text} disabled={!textResult.text} />
          {/snippet}
          <TextArea
            value={textResult.text}
            fill
            readonly
            invalid={!!textResult.error}
            placeholder="结果将显示在这里…"
            ariaLabel="输出"
          />
        </Panel>
      {/snippet}
    </SplitPane>

    {#if textResult.error}
      <p class="error">{textResult.error}</p>
    {/if}
  {:else}
    <SplitPane>
      {#snippet left()}
        <Panel title="选择文件" grow>
          <DropZone
            onfiles={readFile}
            disabled={reading}
            hint={reading ? "读取中…" : "拖拽任意文件到此处，或点击选择"}
          />
          {#if file}
            <dl class="meta">
              <div><dt>文件名</dt><dd>{file.name}</dd></div>
              <div><dt>类型</dt><dd>{file.type}</dd></div>
              <div><dt>原始大小</dt><dd>{formatFileSize(file.size)}</dd></div>
              <div><dt>Base64 后</dt><dd>{formatFileSize(rawBase64.length)}</dd></div>
            </dl>
            {#if isImage}
              <img class="preview" src={file.dataUrl} alt="预览：{file.name}" />
            {/if}
          {/if}
        </Panel>
      {/snippet}

      {#snippet right()}
        <Panel title="Data URL" grow>
          {#snippet actions()}
            <CopyButton value={() => rawBase64} label="仅 Base64" disabled={!file} />
            <CopyButton value={() => file?.dataUrl ?? ""} label="复制 Data URL" disabled={!file} />
          {/snippet}
          <TextArea
            value={file?.dataUrl ?? ""}
            fill
            readonly
            placeholder="选择文件后显示 data:… 形式的完整 URL"
            ariaLabel="Data URL"
          />
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
    gap: var(--sp-3);
  }

  .spacer {
    flex: 1;
  }

  .check {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    font-size: var(--fs-sm);
    color: var(--color-text-secondary);
  }

  .check input {
    accent-color: var(--color-primary);
  }

  .count {
    font-size: var(--fs-xs);
    color: var(--color-text-muted);
    font-family: var(--font-mono);
  }

  .meta {
    display: flex;
    flex-direction: column;
    gap: var(--sp-1);
    margin-top: var(--sp-3);
    font-size: var(--fs-sm);
  }

  .meta > div {
    display: flex;
    justify-content: space-between;
    gap: var(--sp-3);
    padding: var(--sp-1) var(--sp-2);
    border-radius: var(--radius-sm);
    background: var(--color-surface-2);
  }

  dt {
    color: var(--color-text-muted);
  }

  dd {
    font-family: var(--font-mono);
    color: var(--color-text);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .preview {
    margin-top: var(--sp-3);
    max-width: 100%;
    max-height: 220px;
    object-fit: contain;
    align-self: center;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    background: var(--color-surface-2);
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
