<script lang="ts">
  import { marked } from "marked";
  import DOMPurify from "dompurify";
  import { downloadBlob } from "../../lib/download";

  let input = $state("");
  let copied = $state(false);

  const html = $derived.by(() => {
    if (!input.trim()) return "";
    const raw = marked.parse(input, { async: false }) as string;
    return DOMPurify.sanitize(raw);
  });

  async function copyHtml() {
    if (!html) return;
    try {
      await navigator.clipboard.writeText(html);
      copied = true;
      setTimeout(() => (copied = false), 1500);
    } catch (e) {
      console.error("复制失败:", e);
    }
  }

  function downloadHtml() {
    if (!html) return;
    const doc = `<!doctype html><html><head><meta charset="utf-8"><title>markdown</title></head><body>${html}</body></html>`;
    downloadBlob(new Blob([doc], { type: "text/html;charset=utf-8" }), "document.html");
  }

  function downloadMd() {
    if (!input) return;
    downloadBlob(new Blob([input], { type: "text/markdown;charset=utf-8" }), "document.md");
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

  function clearAll() {
    input = "";
  }
</script>

<div class="markdown-tool">
  <div class="controls">
    <button class="primary-btn" onclick={loadExample}>📋 示例</button>
    <button class="secondary-btn" onclick={clearAll}>🗑️ 清空</button>
    <div class="spacer"></div>
    <button class="secondary-btn" onclick={copyHtml} disabled={!html}>
      {copied ? "✓ 已复制 HTML" : "📋 复制 HTML"}
    </button>
    <button class="secondary-btn" onclick={downloadMd} disabled={!input}>⬇ .md</button>
    <button class="secondary-btn" onclick={downloadHtml} disabled={!html}>⬇ .html</button>
  </div>

  <div class="split">
    <div class="pane">
      <div class="pane-header">Markdown · {input.split("\n").length} 行</div>
      <textarea bind:value={input} placeholder="在此输入 Markdown..." spellcheck="false"></textarea>
    </div>
    <div class="pane">
      <div class="pane-header">预览</div>
      <div class="md-preview">
        {#if html}
          {@html html}
        {:else}
          <p class="placeholder">预览将显示在这里...</p>
        {/if}
      </div>
    </div>
  </div>

  <div class="info-section">
    <h3>关于 Markdown 预览</h3>
    <p>实时渲染 Markdown（GFM：表格、代码块等），输出经 DOMPurify 净化以防 XSS。可复制 HTML 或导出 .md / .html，全程本地处理。</p>
  </div>
</div>

<style>
  .markdown-tool {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .controls {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    flex-wrap: wrap;
  }
  .spacer {
    flex: 1;
  }
  .secondary-btn {
    background: var(--bg-dark) !important;
    border: 1px solid var(--border) !important;
    color: var(--text-secondary) !important;
  }
  .secondary-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .split {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 0.75rem;
    min-height: 360px;
  }

  .pane {
    display: flex;
    flex-direction: column;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    overflow: hidden;
  }
  .pane-header {
    padding: 0.5rem 0.75rem;
    background: var(--bg-dark);
    border-bottom: 1px solid var(--border);
    font-size: 0.8rem;
    color: var(--text-secondary);
  }
  .pane textarea {
    flex: 1;
    border: none !important;
    border-radius: 0 !important;
    resize: none;
    font-family: "JetBrains Mono", monospace !important;
    font-size: 0.85rem;
    min-height: 320px;
  }

  .md-preview {
    flex: 1;
    padding: 1rem;
    overflow: auto;
    background: var(--bg-card);
    color: var(--text-primary);
    font-size: 0.9rem;
    line-height: 1.7;
  }
  .placeholder {
    color: var(--text-muted);
  }

  /* 渲染后的 Markdown（@html 内容需 :global） */
  .md-preview :global(h1),
  .md-preview :global(h2),
  .md-preview :global(h3) {
    margin: 1rem 0 0.6rem;
    color: var(--text-primary);
    border-bottom: 1px solid var(--border);
    padding-bottom: 0.3rem;
  }
  .md-preview :global(h1:first-child) {
    margin-top: 0;
  }
  .md-preview :global(p) {
    margin: 0.6rem 0;
  }
  .md-preview :global(ul),
  .md-preview :global(ol) {
    margin: 0.6rem 0;
    padding-left: 1.5rem;
  }
  .md-preview :global(a) {
    color: var(--accent);
  }
  .md-preview :global(code) {
    font-family: "JetBrains Mono", monospace;
    background: var(--bg-dark);
    padding: 0.1rem 0.4rem;
    border-radius: 4px;
    font-size: 0.85em;
    color: var(--accent-green);
  }
  .md-preview :global(pre) {
    background: var(--bg-dark);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    padding: 0.8rem;
    overflow-x: auto;
  }
  .md-preview :global(pre code) {
    background: none;
    padding: 0;
    color: var(--text-primary);
  }
  .md-preview :global(blockquote) {
    margin: 0.6rem 0;
    padding: 0.2rem 0.9rem;
    border-left: 3px solid var(--primary);
    color: var(--text-secondary);
    background: var(--bg-dark);
  }
  .md-preview :global(table) {
    border-collapse: collapse;
    margin: 0.6rem 0;
    width: 100%;
  }
  .md-preview :global(th),
  .md-preview :global(td) {
    border: 1px solid var(--border);
    padding: 0.4rem 0.7rem;
    text-align: left;
  }
  .md-preview :global(th) {
    background: var(--bg-dark);
  }
  .md-preview :global(img) {
    max-width: 100%;
  }
  .md-preview :global(hr) {
    border: none;
    border-top: 1px solid var(--border);
    margin: 1rem 0;
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
</style>
