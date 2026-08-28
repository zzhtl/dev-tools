<script lang="ts">
  import { format as formatSql, type FormatOptionsWithLanguage } from "sql-formatter";
  import Button from "../../lib/ui/Button.svelte";
  import CopyButton from "../../lib/ui/CopyButton.svelte";
  import Panel from "../../lib/ui/Panel.svelte";
  import SplitPane from "../../lib/ui/SplitPane.svelte";
  import TextArea from "../../lib/ui/TextArea.svelte";

  type Dialect = FormatOptionsWithLanguage["language"];

  const DIALECTS: { value: Dialect; label: string }[] = [
    { value: "sql", label: "标准 SQL" },
    { value: "mysql", label: "MySQL" },
    { value: "postgresql", label: "PostgreSQL" },
    { value: "sqlite", label: "SQLite" },
    { value: "mariadb", label: "MariaDB" },
    { value: "bigquery", label: "BigQuery" },
    { value: "snowflake", label: "Snowflake" },
    { value: "spark", label: "Spark SQL" },
    { value: "trino", label: "Trino" },
    { value: "tsql", label: "SQL Server" },
    { value: "db2", label: "DB2" },
  ];

  let input = $state("");
  let dialect = $state<Dialect>("sql");
  let keywordCase = $state<"upper" | "lower" | "preserve">("upper");
  let indentWidth = $state("2");
  let minify = $state(false);

  const EXAMPLE =
    "select u.id, u.name, count(o.id) as order_count from users u left join orders o on o.user_id = u.id where u.created_at >= '2024-01-01' and u.status in ('active','trial') group by u.id, u.name having count(o.id) > 3 order by order_count desc limit 20;";

  const result = $derived.by(() => {
    if (!input.trim()) return { text: "", error: "" };
    try {
      const text = formatSql(input, {
        language: dialect,
        keywordCase,
        tabWidth: Number.parseInt(indentWidth, 10) || 2,
        expressionWidth: minify ? 10000 : 50,
      });
      // sql-formatter 没有压缩模式，压缩时把换行折叠掉
      return { text: minify ? text.replace(/\s+/g, " ").trim() : text, error: "" };
    } catch (err) {
      return { text: "", error: err instanceof Error ? err.message : String(err) };
    }
  });
</script>

<div class="tool">
  <div class="toolbar">
    <label class="opt">
      方言
      <select bind:value={dialect} aria-label="SQL 方言">
        {#each DIALECTS as item (item.value)}
          <option value={item.value}>{item.label}</option>
        {/each}
      </select>
    </label>

    <label class="opt">
      关键字
      <select bind:value={keywordCase} aria-label="关键字大小写">
        <option value="upper">大写</option>
        <option value="lower">小写</option>
        <option value="preserve">保持原样</option>
      </select>
    </label>

    <label class="opt">
      缩进
      <select bind:value={indentWidth} aria-label="缩进宽度">
        <option value="2">2 空格</option>
        <option value="4">4 空格</option>
      </select>
    </label>

    <label class="check">
      <input type="checkbox" bind:checked={minify} />
      压缩为单行
    </label>

    <div class="spacer"></div>
    <Button size="sm" onclick={() => (input = EXAMPLE)}>示例</Button>
    <Button size="sm" variant="ghost" onclick={() => (input = "")}>清空</Button>
  </div>

  <SplitPane>
    {#snippet left()}
      <Panel title="SQL 输入" grow>
        {#snippet actions()}
          <span class="count">{input.length} 字符</span>
        {/snippet}
        <TextArea bind:value={input} fill placeholder="在此粘贴 SQL…" ariaLabel="SQL 输入" />
      </Panel>
    {/snippet}

    {#snippet right()}
      <Panel title={minify ? "压缩结果" : "格式化结果"} grow>
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

  <p class="hint">格式化在本地完成，SQL 不会发送到服务端。占位符 <code>?</code> / <code>:name</code> / <code>$1</code> 会被保留。</p>
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

  .opt,
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

  select {
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

  .error {
    padding: var(--sp-2) var(--sp-3);
    border: 1px solid var(--color-danger-border);
    border-radius: var(--radius-sm);
    background: var(--color-danger-bg);
    color: var(--color-danger);
    font-size: var(--fs-sm);
    white-space: pre-wrap;
  }

  .hint {
    font-size: var(--fs-xs);
    color: var(--color-text-muted);
  }

  .hint code {
    font-family: var(--font-mono);
  }
</style>
