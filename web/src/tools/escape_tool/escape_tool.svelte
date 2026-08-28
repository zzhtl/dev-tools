<script lang="ts">
  import CopyButton from "../../lib/ui/CopyButton.svelte";
  import Panel from "../../lib/ui/Panel.svelte";
  import SplitPane from "../../lib/ui/SplitPane.svelte";
  import Tabs from "../../lib/ui/Tabs.svelte";
  import TextArea from "../../lib/ui/TextArea.svelte";

  type Kind = "unicode" | "html" | "json" | "sql" | "shell";

  const KINDS: { value: Kind; label: string; note: string }[] = [
    { value: "unicode", label: "Unicode", note: "非 ASCII 字符转 \\uXXXX，代理对按 \\uXXXX\\uXXXX 输出" },
    { value: "html", label: "HTML 实体", note: "转义 & < > \" ' ，防止注入到 HTML 上下文" },
    { value: "json", label: "JSON 字符串", note: "按 JSON 规范转义并加引号，可直接嵌入 JSON" },
    { value: "sql", label: "SQL 字面量", note: "单引号加倍（标准 SQL）。注意：转义不能替代参数化查询" },
    { value: "shell", label: "Shell 参数", note: "用单引号包裹，内部单引号按 '\\'' 处理（POSIX sh）" },
  ];

  let kind = $state<Kind>("unicode");
  let direction = $state<"escape" | "unescape">("escape");
  let input = $state("");

  const HTML_ENTITIES: Record<string, string> = {
    "&": "&amp;",
    "<": "&lt;",
    ">": "&gt;",
    '"': "&quot;",
    "'": "&#39;",
  };

  function escapeUnicode(text: string): string {
    let out = "";
    for (const char of text) {
      const code = char.codePointAt(0) ?? 0;
      if (code < 0x80) {
        out += char;
      } else if (code > 0xffff) {
        // 星平面字符需要拆成代理对
        for (let i = 0; i < char.length; i++) {
          out += `\\u${char.charCodeAt(i).toString(16).padStart(4, "0")}`;
        }
      } else {
        out += `\\u${code.toString(16).padStart(4, "0")}`;
      }
    }
    return out;
  }

  function unescapeUnicode(text: string): string {
    return text
      .replace(/\\u\{([0-9a-fA-F]+)\}/g, (_, hex) => String.fromCodePoint(Number.parseInt(hex, 16)))
      .replace(/\\u([0-9a-fA-F]{4})/g, (_, hex) => String.fromCharCode(Number.parseInt(hex, 16)));
  }

  function escapeHtmlText(text: string): string {
    return text.replace(/[&<>"']/g, (char) => HTML_ENTITIES[char]);
  }

  function unescapeHtmlText(text: string): string {
    return text
      .replace(/&#(\d+);/g, (_, code) => String.fromCodePoint(Number(code)))
      .replace(/&#x([0-9a-fA-F]+);/g, (_, hex) => String.fromCodePoint(Number.parseInt(hex, 16)))
      .replace(/&lt;/g, "<")
      .replace(/&gt;/g, ">")
      .replace(/&quot;/g, '"')
      .replace(/&apos;/g, "'")
      .replace(/&nbsp;/g, "\u00a0")
      .replace(/&amp;/g, "&");
  }

  function escapeShell(text: string): string {
    return `'${text.replace(/'/g, `'\\''`)}'`;
  }

  function unescapeShell(text: string): string {
    const trimmed = text.trim();
    if (trimmed.startsWith("'") && trimmed.endsWith("'") && trimmed.length >= 2) {
      return trimmed.slice(1, -1).replace(/'\\''/g, "'");
    }
    return text;
  }

  const result = $derived.by(() => {
    if (!input) return { text: "", error: "" };
    try {
      if (direction === "escape") {
        switch (kind) {
          case "unicode":
            return { text: escapeUnicode(input), error: "" };
          case "html":
            return { text: escapeHtmlText(input), error: "" };
          case "json":
            return { text: JSON.stringify(input), error: "" };
          case "sql":
            return { text: `'${input.replace(/'/g, "''")}'`, error: "" };
          case "shell":
            return { text: escapeShell(input), error: "" };
        }
      }
      switch (kind) {
        case "unicode":
          return { text: unescapeUnicode(input), error: "" };
        case "html":
          return { text: unescapeHtmlText(input), error: "" };
        case "json": {
          const trimmed = input.trim();
          const source = trimmed.startsWith('"') ? trimmed : JSON.stringify(trimmed);
          const parsed = JSON.parse(source);
          if (typeof parsed !== "string") return { text: "", error: "不是 JSON 字符串字面量" };
          return { text: parsed, error: "" };
        }
        case "sql": {
          const trimmed = input.trim();
          const body = trimmed.startsWith("'") && trimmed.endsWith("'") ? trimmed.slice(1, -1) : trimmed;
          return { text: body.replace(/''/g, "'"), error: "" };
        }
        case "shell":
          return { text: unescapeShell(input), error: "" };
      }
    } catch (err) {
      return { text: "", error: err instanceof Error ? err.message : String(err) };
    }
  });

  const activeNote = $derived(KINDS.find((item) => item.value === kind)?.note ?? "");
</script>

<div class="tool">
  <div class="toolbar">
    <Tabs bind:value={kind} tabs={KINDS.map((k) => ({ value: k.value, label: k.label }))} ariaLabel="转义类型" />
    <Tabs
      bind:value={direction}
      size="sm"
      tabs={[
        { value: "escape", label: "转义" },
        { value: "unescape", label: "反转义" },
      ]}
      ariaLabel="方向"
    />
  </div>

  <p class="note">{activeNote}</p>

  <SplitPane>
    {#snippet left()}
      <Panel title={direction === "escape" ? "原文" : "已转义"} grow>
        {#snippet actions()}
          <span class="count">{input.length} 字符</span>
        {/snippet}
        <TextArea bind:value={input} fill placeholder="在此输入…" ariaLabel="输入" />
      </Panel>
    {/snippet}

    {#snippet right()}
      <Panel title={direction === "escape" ? "已转义" : "原文"} grow>
        {#snippet actions()}
          <span class="count">{result.text.length} 字符</span>
          <CopyButton value={() => result.text} disabled={!result.text} />
        {/snippet}
        <TextArea
          value={result.text}
          fill
          readonly
          invalid={!!result.error}
          placeholder="结果将显示在这里…"
          ariaLabel="输出"
        />
      </Panel>
    {/snippet}
  </SplitPane>

  {#if result.error}
    <p class="error">{result.error}</p>
  {/if}

  {#if kind === "sql"}
    <p class="warn">
      SQL 转义只是应急手段。生产代码请使用参数化查询 / 预编译语句，字符串拼接始终存在注入风险。
    </p>
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

  .note {
    font-size: var(--fs-xs);
    color: var(--color-text-muted);
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

  .warn {
    padding: var(--sp-2) var(--sp-3);
    border: 1px solid var(--color-warning-border);
    border-radius: var(--radius-sm);
    background: var(--color-warning-bg);
    color: var(--color-text-secondary);
    font-size: var(--fs-xs);
  }
</style>
