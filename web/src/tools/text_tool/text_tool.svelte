<script lang="ts">
  import Button from "../../lib/ui/Button.svelte";
  import CopyButton from "../../lib/ui/CopyButton.svelte";
  import Panel from "../../lib/ui/Panel.svelte";
  import SplitPane from "../../lib/ui/SplitPane.svelte";
  import TextArea from "../../lib/ui/TextArea.svelte";
  import TextInput from "../../lib/ui/TextInput.svelte";

  let input = $state("");

  // 行操作
  let dedupe = $state(false);
  let dropBlank = $state(false);
  let trimLines = $state(false);
  let sortMode = $state<"none" | "asc" | "desc" | "length" | "reverse" | "shuffle">("none");
  let numbering = $state(false);
  let prefix = $state("");
  let suffix = $state("");
  let caseMode = $state<"none" | "upper" | "lower" | "title" | "camel" | "snake" | "kebab">("none");

  function toWords(text: string): string[] {
    return text
      .replace(/([a-z0-9])([A-Z])/g, "$1 $2")
      .split(/[\s_\-.]+/)
      .filter(Boolean);
  }

  function applyCase(text: string): string {
    switch (caseMode) {
      case "upper":
        return text.toUpperCase();
      case "lower":
        return text.toLowerCase();
      case "title":
        return text.replace(/\b\w/g, (char) => char.toUpperCase());
      case "camel": {
        const words = toWords(text);
        return words
          .map((word, index) =>
            index === 0 ? word.toLowerCase() : word[0].toUpperCase() + word.slice(1).toLowerCase(),
          )
          .join("");
      }
      case "snake":
        return toWords(text).map((word) => word.toLowerCase()).join("_");
      case "kebab":
        return toWords(text).map((word) => word.toLowerCase()).join("-");
      default:
        return text;
    }
  }

  const output = $derived.by(() => {
    if (!input) return "";
    let lines = input.split("\n");

    if (trimLines) lines = lines.map((line) => line.trim());
    if (dropBlank) lines = lines.filter((line) => line.trim() !== "");
    if (dedupe) lines = [...new Set(lines)];

    switch (sortMode) {
      case "asc":
        lines = [...lines].sort((a, b) => a.localeCompare(b, "zh-CN"));
        break;
      case "desc":
        lines = [...lines].sort((a, b) => b.localeCompare(a, "zh-CN"));
        break;
      case "length":
        lines = [...lines].sort((a, b) => a.length - b.length);
        break;
      case "reverse":
        lines = [...lines].reverse();
        break;
      case "shuffle": {
        // Fisher-Yates
        lines = [...lines];
        for (let i = lines.length - 1; i > 0; i--) {
          const j = Math.floor(Math.random() * (i + 1));
          [lines[i], lines[j]] = [lines[j], lines[i]];
        }
        break;
      }
      default:
        break;
    }

    if (caseMode !== "none") lines = lines.map(applyCase);
    if (prefix || suffix) lines = lines.map((line) => `${prefix}${line}${suffix}`);
    if (numbering) {
      const width = String(lines.length).length;
      lines = lines.map((line, index) => `${String(index + 1).padStart(width, " ")}. ${line}`);
    }
    return lines.join("\n");
  });

  const stats = $derived.by(() => {
    const lines = input ? input.split("\n") : [];
    const nonEmpty = lines.filter((line) => line.trim() !== "").length;
    const chars = input.length;
    const charsNoSpace = input.replace(/\s/g, "").length;
    // 中文按字计，西文按词计
    const cjk = (input.match(/[一-龥]/g) ?? []).length;
    const words = (input.match(/[A-Za-z0-9_'-]+/g) ?? []).length;
    return {
      lines: lines.length,
      nonEmpty,
      chars,
      charsNoSpace,
      words: words + cjk,
      bytes: new Blob([input]).size,
    };
  });
</script>

<div class="tool">
  <Panel title="处理选项">
    {#snippet actions()}
      <Button
        size="sm"
        variant="ghost"
        onclick={() => {
          dedupe = false;
          dropBlank = false;
          trimLines = false;
          sortMode = "none";
          numbering = false;
          prefix = "";
          suffix = "";
          caseMode = "none";
        }}
      >
        重置选项
      </Button>
    {/snippet}

    <div class="opts">
      <label class="chip" class:on={dedupe}><input type="checkbox" bind:checked={dedupe} />去重</label>
      <label class="chip" class:on={dropBlank}><input type="checkbox" bind:checked={dropBlank} />去空行</label>
      <label class="chip" class:on={trimLines}><input type="checkbox" bind:checked={trimLines} />去首尾空格</label>
      <label class="chip" class:on={numbering}><input type="checkbox" bind:checked={numbering} />加行号</label>

      <label class="sel">
        排序
        <select bind:value={sortMode} aria-label="排序方式">
          <option value="none">不排序</option>
          <option value="asc">字典升序</option>
          <option value="desc">字典降序</option>
          <option value="length">按长度</option>
          <option value="reverse">反转顺序</option>
          <option value="shuffle">随机打乱</option>
        </select>
      </label>

      <label class="sel">
        大小写
        <select bind:value={caseMode} aria-label="大小写转换">
          <option value="none">不转换</option>
          <option value="upper">全大写</option>
          <option value="lower">全小写</option>
          <option value="title">首字母大写</option>
          <option value="camel">camelCase</option>
          <option value="snake">snake_case</option>
          <option value="kebab">kebab-case</option>
        </select>
      </label>
    </div>

    <div class="affix">
      <label>前缀<TextInput bind:value={prefix} size="sm" mono ariaLabel="每行前缀" /></label>
      <label>后缀<TextInput bind:value={suffix} size="sm" mono ariaLabel="每行后缀" /></label>
    </div>
  </Panel>

  <SplitPane>
    {#snippet left()}
      <Panel title="输入" grow>
        {#snippet actions()}
          <Button size="sm" variant="ghost" onclick={() => (input = "")}>清空</Button>
        {/snippet}
        <TextArea bind:value={input} fill mono={false} placeholder="在此粘贴文本…" ariaLabel="输入文本" />
      </Panel>
    {/snippet}

    {#snippet right()}
      <Panel title="结果" grow>
        {#snippet actions()}
          <CopyButton value={() => output} disabled={!output} />
        {/snippet}
        <TextArea value={output} fill readonly mono={false} placeholder="处理结果将显示在这里…" ariaLabel="结果" />
      </Panel>
    {/snippet}
  </SplitPane>

  <div class="stats">
    <span>行数 <b>{stats.lines}</b></span>
    <span>非空行 <b>{stats.nonEmpty}</b></span>
    <span>字符 <b>{stats.chars}</b></span>
    <span>不含空白 <b>{stats.charsNoSpace}</b></span>
    <span>词数 <b>{stats.words}</b></span>
    <span>字节 <b>{stats.bytes}</b></span>
  </div>
</div>

<style>
  .tool {
    display: flex;
    flex-direction: column;
    gap: var(--sp-3);
    flex: 1;
    min-height: 0;
  }

  .opts {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: var(--sp-2);
  }

  .chip {
    display: inline-flex;
    align-items: center;
    gap: var(--sp-1);
    padding: 0.25rem var(--sp-3);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-full);
    background: var(--color-surface-2);
    font-size: var(--fs-sm);
    color: var(--color-text-secondary);
    cursor: pointer;
  }

  .chip.on {
    border-color: var(--color-primary-border);
    background: var(--color-primary-bg);
    color: var(--color-primary);
  }

  .chip input {
    accent-color: var(--color-primary);
  }

  .sel {
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

  .affix {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
    gap: var(--sp-3);
    margin-top: var(--sp-3);
  }

  .affix label {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    font-size: var(--fs-sm);
    color: var(--color-text-secondary);
  }

  .stats {
    display: flex;
    flex-wrap: wrap;
    gap: var(--sp-2);
  }

  .stats span {
    padding: 0.15rem var(--sp-3);
    border-radius: var(--radius-full);
    background: var(--color-surface-2);
    font-size: var(--fs-xs);
    color: var(--color-text-muted);
  }

  .stats b {
    font-family: var(--font-mono);
    color: var(--color-text);
  }
</style>
