<script lang="ts">
  import { apiJson } from "../../lib/api";
  import Button from "../../lib/ui/Button.svelte";
  import CopyButton from "../../lib/ui/CopyButton.svelte";
  import Panel from "../../lib/ui/Panel.svelte";
  import SplitPane from "../../lib/ui/SplitPane.svelte";
  import Tabs from "../../lib/ui/Tabs.svelte";
  import TextArea from "../../lib/ui/TextArea.svelte";

  type Format = "yaml" | "toml";

  let source = $state<Format>("yaml");
  let target = $state<"same" | "json" | "yaml" | "toml">("same");
  let input = $state("");
  let output = $state("");
  let error = $state("");
  let busy = $state(false);
  let validated = $state(false);

  const YAML_EXAMPLE = `name: dev-tools
version: 0.3.0
server:
  host: 127.0.0.1
  port: 8927
features:
  - json
  - dns
  - image
`;

  const TOML_EXAMPLE = `name = "dev-tools"
version = "0.3.0"

[server]
host = "127.0.0.1"
port = 8927

[features]
enabled = ["json", "dns", "image"]
`;

  const effectiveTarget = $derived(target === "same" ? source : target);

  async function run() {
    if (!input.trim()) {
      output = "";
      error = "";
      validated = false;
      return;
    }
    busy = true;
    error = "";
    validated = false;
    try {
      // 复用后端的格式互转：同格式进出即为「校验 + 规范化」
      const result = await apiJson<{ output: string }>("/api/json/convert", {
        input,
        from: source,
        to: effectiveTarget,
        indent: 2,
      });
      output = result.output;
      validated = true;
    } catch (e) {
      output = "";
      error = String(e);
    } finally {
      busy = false;
    }
  }

  function loadExample() {
    input = source === "yaml" ? YAML_EXAMPLE : TOML_EXAMPLE;
  }
</script>

<div class="tool">
  <div class="toolbar">
    <Tabs
      bind:value={source}
      tabs={[
        { value: "yaml", label: "YAML" },
        { value: "toml", label: "TOML" },
      ]}
      ariaLabel="输入格式"
      onchange={() => {
        output = "";
        error = "";
        validated = false;
      }}
    />

    <label class="opt">
      输出
      <select bind:value={target} aria-label="输出格式">
        <option value="same">格式化（同格式）</option>
        <option value="json">转 JSON</option>
        <option value="yaml">转 YAML</option>
        <option value="toml">转 TOML</option>
      </select>
    </label>

    <div class="spacer"></div>
    <Button size="sm" onclick={loadExample}>示例</Button>
    <Button
      size="sm"
      variant="ghost"
      onclick={() => {
        input = "";
        output = "";
        error = "";
        validated = false;
      }}
    >
      清空
    </Button>
    <Button size="sm" variant="primary" onclick={run} loading={busy} disabled={!input.trim()}>
      {target === "same" ? "校验并格式化" : "转换"}
    </Button>
  </div>

  <SplitPane>
    {#snippet left()}
      <Panel title={`${source.toUpperCase()} 输入`} grow>
        {#snippet actions()}
          <span class="count">{input.length} 字符</span>
        {/snippet}
        <TextArea
          bind:value={input}
          fill
          placeholder={source === "yaml" ? "在此粘贴 YAML…" : "在此粘贴 TOML…"}
          ariaLabel="输入"
        />
      </Panel>
    {/snippet}

    {#snippet right()}
      <Panel title={`${effectiveTarget.toUpperCase()} 输出`} grow>
        {#snippet actions()}
          {#if validated && !error}
            <span class="ok">✓ 语法有效</span>
          {/if}
          <CopyButton value={() => output} disabled={!output} />
        {/snippet}
        <TextArea
          value={output}
          fill
          readonly
          invalid={!!error}
          placeholder="点击右上角按钮开始校验或转换…"
          ariaLabel="输出"
        />
      </Panel>
    {/snippet}
  </SplitPane>

  {#if error}
    <p class="error">{error}</p>
  {/if}

  <p class="hint">
    解析在服务端完成（<code>serde_yaml</code> / <code>toml</code>）。TOML 要求顶层是表且不含 null，
    因此并非所有 YAML 都能转成 TOML。
  </p>
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

  .opt {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    font-size: var(--fs-sm);
    color: var(--color-text-secondary);
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

  .ok {
    font-size: var(--fs-xs);
    color: var(--color-success);
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
