<script lang="ts">
  import { marked } from "marked";
  import DOMPurify from "dompurify";
  import { downloadBlob } from "../../lib/download";
  import Button from "../../lib/ui/Button.svelte";
  import CopyButton from "../../lib/ui/CopyButton.svelte";
  import Panel from "../../lib/ui/Panel.svelte";
  import SplitPane from "../../lib/ui/SplitPane.svelte";
  import TextArea from "../../lib/ui/TextArea.svelte";
  import EmptyState from "../../lib/ui/EmptyState.svelte";

  let input = $state("");

  const html = $derived.by(() => {
    if (!input.trim()) return "";
    const raw = marked.parse(input, { async: false }) as string;
    // marked 不做净化，必须经 DOMPurify 后才能 @html 注入
    return DOMPurify.sanitize(raw);
  });

  function downloadHtml() {
    const doc = `<!doctype html><html><head><meta charset="utf-8"><title>markdown</title></head><body>${html}</body></html>`;
    downloadBlob(new Blob([doc], { type: "text/html;charset=utf-8" }), "document.html");
  }

  function loadExample() {
    input = `# 开发工具箱

支持 **加粗**、*斜体*、\`行内代码\` 与 [链接](https://example.com)。

## 列表
- 第一项
- 第二项
  - 嵌套项

## 代码块
\`\`\`js
console.log("hello dev-tools");
\`\`\`

## 表格
| 工具 | 用途 |
|------|------|
| JSON | 格式化 / 转换 |
| JWT  | 解析 / 验签 |

> 引用：本地处理，内容不离开你的机器。
`;
  }
</script>

<div class="tool">
  <div class="toolbar">
    <Button size="sm" onclick={loadExample}>示例</Button>
    <Button size="sm" variant="ghost" onclick={() => (input = "")}>清空</Button>
    <div class="spacer"></div>
    <Button
      size="sm"
      disabled={!input}
      onclick={() =>
        downloadBlob(new Blob([input], { type: "text/markdown;charset=utf-8" }), "document.md")}
    >
      下载 .md
    </Button>
    <Button size="sm" disabled={!html} onclick={downloadHtml}>下载 .html</Button>
  </div>

  <SplitPane>
    {#snippet left()}
      <Panel title="Markdown" grow>
        {#snippet actions()}
          <span class="count">{input.length} 字符</span>
        {/snippet}
        <TextArea bind:value={input} fill placeholder="在此输入 Markdown…" ariaLabel="Markdown 输入" />
      </Panel>
    {/snippet}

    {#snippet right()}
      <Panel title="预览" grow>
        {#snippet actions()}
          <CopyButton value={() => html} label="复制 HTML" disabled={!html} />
        {/snippet}
        {#if html}
          <div class="preview">{@html html}</div>
        {:else}
          <EmptyState icon="M↓" title="预览会显示在这里" hint="支持 GFM 表格、任务列表与代码块" />
        {/if}
      </Panel>
    {/snippet}
  </SplitPane>
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

  .preview {
    font-size: var(--fs-md);
    line-height: var(--lh-relaxed);
    color: var(--color-text-secondary);
  }

  /* @html 注入的内容不受 scoped 样式影响，只能用 :global */
  .preview :global(h1),
  .preview :global(h2),
  .preview :global(h3),
  .preview :global(h4) {
    color: var(--color-text);
    margin: var(--sp-4) 0 var(--sp-2);
    line-height: var(--lh-tight);
  }

  .preview :global(h1) {
    font-size: var(--fs-2xl);
    padding-bottom: var(--sp-2);
    border-bottom: 1px solid var(--color-border);
  }

  .preview :global(h2) {
    font-size: var(--fs-xl);
  }

  .preview :global(h3) {
    font-size: var(--fs-lg);
  }

  .preview :global(p) {
    margin: var(--sp-3) 0;
  }

  .preview :global(ul),
  .preview :global(ol) {
    margin: var(--sp-3) 0;
    padding-left: var(--sp-6);
  }

  .preview :global(li) {
    margin: var(--sp-1) 0;
  }

  .preview :global(a) {
    color: var(--color-primary);
  }

  .preview :global(strong) {
    color: var(--color-text);
  }

  .preview :global(code) {
    font-family: var(--font-mono);
    font-size: 0.9em;
    background: var(--color-code-bg);
    border-radius: 4px;
    padding: 0.15em 0.4em;
  }

  .preview :global(pre) {
    margin: var(--sp-3) 0;
    padding: var(--sp-3);
    background: var(--color-code-bg);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    overflow-x: auto;
  }

  .preview :global(pre code) {
    background: none;
    padding: 0;
  }

  .preview :global(blockquote) {
    margin: var(--sp-3) 0;
    padding: var(--sp-1) var(--sp-4);
    border-left: 3px solid var(--color-primary);
    background: var(--color-surface-2);
    color: var(--color-text-muted);
  }

  .preview :global(table) {
    width: 100%;
    margin: var(--sp-3) 0;
    border-collapse: collapse;
    font-size: var(--fs-sm);
  }

  .preview :global(th),
  .preview :global(td) {
    border: 1px solid var(--color-border);
    padding: var(--sp-2) var(--sp-3);
    text-align: left;
  }

  .preview :global(th) {
    background: var(--color-surface-2);
    color: var(--color-text);
    font-weight: var(--fw-semibold);
  }

  .preview :global(hr) {
    margin: var(--sp-4) 0;
    border: none;
    border-top: 1px solid var(--color-border);
  }

  .preview :global(img) {
    max-width: 100%;
  }
</style>
