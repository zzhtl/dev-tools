<script lang="ts">
  import Button from "../../lib/ui/Button.svelte";
  import CopyButton from "../../lib/ui/CopyButton.svelte";
  import Panel from "../../lib/ui/Panel.svelte";
  import SplitPane from "../../lib/ui/SplitPane.svelte";
  import Tabs from "../../lib/ui/Tabs.svelte";
  import TextArea from "../../lib/ui/TextArea.svelte";
  import EmptyState from "../../lib/ui/EmptyState.svelte";

  type Mode = "format" | "minify" | "preview";

  let input = $state("");
  let mode = $state<Mode>("format");
  let indentSize = $state("2");

  /** HTML 空元素：不能有闭合标签，也不能写成自闭合的 `<div />` 形式。 */
  const VOID_ELEMENTS = new Set([
    "area", "base", "br", "col", "embed", "hr", "img", "input",
    "link", "meta", "param", "source", "track", "wbr",
  ]);

  /** 这些元素内部的文本必须原样保留，缩进会改变其语义。 */
  const RAW_TEXT_ELEMENTS = new Set(["pre", "script", "style", "textarea"]);

  function escapeAttr(value: string): string {
    return value.replace(/&/g, "&amp;").replace(/"/g, "&quot;");
  }

  function formatHtml(source: string, indent: number): { output: string; error: string } {
    if (!source.trim()) return { output: "", error: "" };
    try {
      const doc = new DOMParser().parseFromString(source, "text/html");
      if (doc.getElementsByTagName("parsererror").length > 0) {
        return { output: "", error: "HTML 解析失败" };
      }

      const unit = " ".repeat(indent);
      const lines: string[] = [];

      const walk = (node: Node, level: number): void => {
        const pad = unit.repeat(level);

        if (node.nodeType === Node.TEXT_NODE) {
          const text = node.textContent?.trim();
          if (text) lines.push(pad + text);
          return;
        }

        if (node.nodeType === Node.COMMENT_NODE) {
          lines.push(`${pad}<!--${node.textContent ?? ""}-->`);
          return;
        }

        if (node.nodeType !== Node.ELEMENT_NODE) return;

        const element = node as Element;
        const tag = element.tagName.toLowerCase();
        let open = `${pad}<${tag}`;
        for (const attr of Array.from(element.attributes)) {
          open += ` ${attr.name}="${escapeAttr(attr.value)}"`;
        }
        open += ">";

        if (VOID_ELEMENTS.has(tag)) {
          lines.push(open);
          return;
        }

        if (RAW_TEXT_ELEMENTS.has(tag)) {
          // 原样保留内部文本，只把整块按当前层级缩进
          const raw = (element.textContent ?? "").replace(/^\n+|\s+$/g, "");
          lines.push(open);
          if (raw) {
            for (const line of raw.split("\n")) lines.push(pad + unit + line.trim());
          }
          lines.push(`${pad}</${tag}>`);
          return;
        }

        if (element.childNodes.length === 0) {
          lines.push(`${open}</${tag}>`);
          return;
        }

        lines.push(open);
        for (const child of Array.from(element.childNodes)) walk(child, level + 1);
        lines.push(`${pad}</${tag}>`);
      };

      // 输入是完整文档时从 <html> 开始，否则只输出 body 内容 —— 旧版一律只取 body，
      // 会把 <head> 里的 meta/style 全部丢掉。
      const isFullDocument = /<html[\s>]/i.test(source);
      if (isFullDocument) {
        if (/<!doctype/i.test(source)) lines.push("<!DOCTYPE html>");
        walk(doc.documentElement, 0);
      } else {
        for (const child of Array.from(doc.body.childNodes)) walk(child, 0);
      }

      return { output: lines.join("\n"), error: "" };
    } catch (err) {
      return { output: "", error: err instanceof Error ? err.message : String(err) };
    }
  }

  function minifyHtml(source: string): string {
    return source
      .replace(/<!--[\s\S]*?-->/g, "")
      .replace(/\s+/g, " ")
      .replace(/>\s+</g, "><")
      .trim();
  }

  const result = $derived.by(() => {
    if (mode === "minify") return { output: minifyHtml(input), error: "" };
    if (mode === "format") {
      return formatHtml(input, Number.parseInt(indentSize, 10) || 2);
    }
    return { output: "", error: "" };
  });

  function loadExample() {
    input = `<!DOCTYPE html>
<html lang="zh-CN">
<head>
<meta charset="UTF-8">
<title>HTML示例</title>
<style>body { font-family: sans-serif; padding: 20px; }</style>
</head>
<body>
<div class="container"><h1>HTML示例页面</h1><p>用于测试格式化与压缩。</p>
<ul><li>格式化</li><li>压缩</li><li>预览</li></ul>
<img src="x.png" alt="示例"><br>
</div>
</body>
</html>`;
  }
</script>

<div class="tool">
  <div class="toolbar">
    <Tabs
      bind:value={mode}
      tabs={[
        { value: "format", label: "格式化" },
        { value: "minify", label: "压缩" },
        { value: "preview", label: "预览" },
      ]}
      ariaLabel="处理模式"
    />

    {#if mode === "format"}
      <label class="indent">
        缩进
        <select bind:value={indentSize} aria-label="缩进空格数">
          <option value="2">2 空格</option>
          <option value="4">4 空格</option>
        </select>
      </label>
    {/if}

    <div class="spacer"></div>
    <Button size="sm" onclick={loadExample}>示例</Button>
    <Button size="sm" variant="ghost" onclick={() => (input = "")}>清空</Button>
  </div>

  <SplitPane>
    {#snippet left()}
      <Panel title="HTML 输入" grow>
        {#snippet actions()}
          <span class="count">{input.length} 字符</span>
        {/snippet}
        <TextArea bind:value={input} fill placeholder="在此粘贴 HTML…" ariaLabel="HTML 输入" />
      </Panel>
    {/snippet}

    {#snippet right()}
      {#if mode === "preview"}
        <Panel title="预览" grow flush>
          {#if input.trim()}
            <!-- sandbox 置空：禁用脚本与表单提交，预览不会在应用源里执行任何代码 -->
            <iframe class="preview" title="HTML 预览" sandbox="" srcdoc={input}></iframe>
          {:else}
            <EmptyState icon="</>" title="预览会显示在这里" hint="脚本在预览中被禁用" />
          {/if}
        </Panel>
      {:else}
        <Panel title={mode === "format" ? "格式化输出" : "压缩输出"} grow>
          {#snippet actions()}
            {#if result.output && mode === "minify"}
              <span class="count">
                {Math.max(0, Math.round((1 - result.output.length / Math.max(1, input.length)) * 100))}% ↓
              </span>
            {/if}
            <CopyButton value={() => result.output} disabled={!result.output} />
          {/snippet}
          <TextArea
            value={result.output}
            fill
            readonly
            invalid={!!result.error}
            placeholder="处理结果将显示在这里…"
            ariaLabel="输出"
          />
        </Panel>
      {/if}
    {/snippet}
  </SplitPane>

  {#if result.error}
    <p class="error">{result.error}</p>
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
    gap: var(--sp-3);
    flex-wrap: wrap;
  }

  .spacer {
    flex: 1;
  }

  .indent {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    font-size: var(--fs-sm);
    color: var(--color-text-secondary);
  }

  .indent select {
    background: var(--color-bg);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    color: var(--color-text);
    padding: 0.25rem var(--sp-2);
    font-size: var(--fs-sm);
  }

  .count {
    font-size: var(--fs-xs);
    color: var(--color-text-muted);
    font-family: var(--font-mono);
  }

  .preview {
    flex: 1;
    width: 100%;
    border: none;
    /* 预览的是完整网页，白底是它本来的样子，不跟随应用主题 */
    background: #fff;
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
