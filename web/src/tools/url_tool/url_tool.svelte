<script lang="ts">
  import Button from "../../lib/ui/Button.svelte";
  import CopyButton from "../../lib/ui/CopyButton.svelte";
  import Panel from "../../lib/ui/Panel.svelte";
  import SplitPane from "../../lib/ui/SplitPane.svelte";
  import Tabs from "../../lib/ui/Tabs.svelte";
  import TextArea from "../../lib/ui/TextArea.svelte";
  import { formatCount } from "../../lib/format";

  let inputText = $state("");
  let mode = $state<"encode" | "decode">("encode");
  let encodeType = $state<"component" | "standard">("component");

  const EXAMPLE = "https://example.com/search?q=中文测试&page=1&filter=特殊字符!@#$%";

  const COMMON_ENCODINGS = [
    { char: "空格", code: "%20 或 +" },
    { char: "!", code: "%21" },
    { char: "#", code: "%23" },
    { char: "$", code: "%24" },
    { char: "&", code: "%26" },
    { char: "=", code: "%3D" },
    { char: "?", code: "%3F" },
    { char: "@", code: "%40" },
  ];

  interface Outcome {
    text: string;
    error: string;
  }

  const outcome = $derived.by((): Outcome => {
    if (!inputText.trim()) return { text: "", error: "" };
    try {
      if (mode === "encode") {
        return {
          text: encodeType === "standard" ? encodeURI(inputText) : encodeURIComponent(inputText),
          error: "",
        };
      }
      try {
        return { text: decodeURIComponent(inputText), error: "" };
      } catch {
        // 含未转义 % 时 decodeURIComponent 会抛，退回 decodeURI 再试
        return { text: decodeURI(inputText), error: "" };
      }
    } catch (err) {
      return { text: "", error: `处理失败：${err instanceof Error ? err.message : String(err)}` };
    }
  });

  function swap() {
    const next = outcome.text;
    if (!next) return;
    inputText = next;
    mode = mode === "encode" ? "decode" : "encode";
  }
</script>

<div class="tool">
  <div class="toolbar">
    <Tabs
      bind:value={mode}
      tabs={[
        { value: "encode", label: "编码" },
        { value: "decode", label: "解码" },
      ]}
      ariaLabel="编解码模式"
    />

    {#if mode === "encode"}
      <Tabs
        bind:value={encodeType}
        size="sm"
        tabs={[
          { value: "component", label: "encodeURIComponent" },
          { value: "standard", label: "encodeURI" },
        ]}
        ariaLabel="编码方式"
      />
    {/if}

    <div class="spacer"></div>

    <Button size="sm" onclick={swap} title="把结果放回输入并切换模式">⇄ 交换</Button>
    <Button size="sm" onclick={() => (inputText = EXAMPLE)}>示例</Button>
    <Button size="sm" variant="ghost" onclick={() => (inputText = "")}>清空</Button>
  </div>

  <SplitPane>
    {#snippet left()}
      <Panel title={mode === "encode" ? "原文" : "编码后"} grow>
        {#snippet actions()}
          <span class="count">{formatCount(inputText.length)} 字符</span>
        {/snippet}
        <TextArea
          bind:value={inputText}
          fill
          placeholder={mode === "encode" ? "输入要编码的 URL 或文本…" : "输入要解码的 URL…"}
          ariaLabel="输入"
        />
      </Panel>
    {/snippet}

    {#snippet right()}
      <Panel title={mode === "encode" ? "编码后" : "解码后"} grow>
        {#snippet actions()}
          <span class="count">{formatCount(outcome.text.length)} 字符</span>
          <CopyButton value={() => outcome.text} disabled={!outcome.text} />
        {/snippet}
        <TextArea
          value={outcome.text}
          fill
          readonly
          invalid={!!outcome.error}
          placeholder="处理结果将显示在这里…"
          ariaLabel="输出"
        />
      </Panel>
    {/snippet}
  </SplitPane>

  {#if outcome.error}
    <p class="error">{outcome.error}</p>
  {/if}

  <details class="reference">
    <summary>URL 编码说明</summary>
    <div class="ref-body">
      <div class="ref-cards">
        <div class="ref-card">
          <h3>encodeURIComponent</h3>
          <p>编码所有非字母数字字符，适合编码单个参数值，会编码 <code>= &amp; ? /</code>。</p>
          <code class="sample">你好 → %E4%BD%A0%E5%A5%BD</code>
        </div>
        <div class="ref-card">
          <h3>encodeURI</h3>
          <p>保留 URL 结构字符，适合编码完整 URL，不会编码 <code>: / ? # [ ] @</code>。</p>
          <code class="sample">https://a.com/你好 → https://a.com/%E4%BD%A0%E5%A5%BD</code>
        </div>
      </div>
      <ul class="table">
        {#each COMMON_ENCODINGS as item (item.char)}
          <li><span>{item.char}</span><code>{item.code}</code></li>
        {/each}
      </ul>
    </div>
  </details>
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

  .error {
    padding: var(--sp-2) var(--sp-3);
    border: 1px solid var(--color-danger-border);
    border-radius: var(--radius-sm);
    background: var(--color-danger-bg);
    color: var(--color-danger);
    font-size: var(--fs-sm);
  }

  .reference {
    border: 1px solid var(--color-border);
    border-radius: var(--radius);
    background: var(--color-surface);
  }

  summary {
    padding: var(--sp-2) var(--sp-4);
    cursor: pointer;
    font-size: var(--fs-sm);
    font-weight: var(--fw-medium);
    color: var(--color-text-secondary);
  }

  .ref-body {
    display: flex;
    flex-direction: column;
    gap: var(--sp-3);
    padding: 0 var(--sp-4) var(--sp-4);
  }

  .ref-cards {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));
    gap: var(--sp-3);
  }

  .ref-card {
    padding: var(--sp-3);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    background: var(--color-surface-2);
  }

  .ref-card h3 {
    font-size: var(--fs-sm);
    font-family: var(--font-mono);
    color: var(--color-primary);
    margin-bottom: var(--sp-1);
  }

  .ref-card p {
    font-size: var(--fs-sm);
    color: var(--color-text-secondary);
  }

  .sample {
    display: block;
    margin-top: var(--sp-2);
    font-size: var(--fs-xs);
    color: var(--color-text-muted);
    word-break: break-all;
  }

  code {
    font-family: var(--font-mono);
    font-size: 0.95em;
  }

  .table {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
    gap: var(--sp-1) var(--sp-3);
    list-style: none;
  }

  .table li {
    display: flex;
    justify-content: space-between;
    gap: var(--sp-2);
    padding: var(--sp-1) var(--sp-2);
    border-radius: var(--radius-sm);
    background: var(--color-surface-2);
    font-size: var(--fs-xs);
  }

  .table span {
    color: var(--color-text-secondary);
  }

  .table code {
    color: var(--color-accent);
  }
</style>
