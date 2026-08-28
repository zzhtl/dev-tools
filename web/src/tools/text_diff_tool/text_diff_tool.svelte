<script lang="ts">
  import { diffLines } from "../../lib/diff";
  import Button from "../../lib/ui/Button.svelte";
  import DiffView from "../../lib/ui/DiffView.svelte";
  import Panel from "../../lib/ui/Panel.svelte";
  import SplitPane from "../../lib/ui/SplitPane.svelte";
  import TextArea from "../../lib/ui/TextArea.svelte";
  import { formatCount } from "../../lib/format";

  let textA = $state("");
  let textB = $state("");
  let onlyChanges = $state(false);

  const result = $derived(diffLines(textA, textB));
  const hasContent = $derived(textA.length > 0 || textB.length > 0);

  const lineCountA = $derived(textA ? textA.split("\n").length : 0);
  const lineCountB = $derived(textB ? textB.split("\n").length : 0);

  function loadExample() {
    textA = `name: dev-tools
version: 1.0.0
features:
  - json
  - regex
port: 8080`;
    textB = `name: dev-tools
version: 1.2.0
features:
  - json
  - regex
  - jwt
port: 9000`;
  }
</script>

<div class="tool">
  <div class="toolbar">
    <Button size="sm" onclick={loadExample}>示例</Button>
    <Button
      size="sm"
      variant="ghost"
      onclick={() => {
        textA = "";
        textB = "";
      }}
    >
      清空
    </Button>
    <label class="check">
      <input type="checkbox" bind:checked={onlyChanges} />
      只看差异
    </label>
  </div>

  <SplitPane>
    {#snippet left()}
      <Panel title="文本 A" grow>
        {#snippet actions()}
          <span class="count">{formatCount(lineCountA)} 行</span>
        {/snippet}
        <TextArea bind:value={textA} fill placeholder="在此粘贴文本 A…" ariaLabel="文本 A" />
      </Panel>
    {/snippet}

    {#snippet right()}
      <Panel title="文本 B" grow>
        {#snippet actions()}
          <span class="count">{formatCount(lineCountB)} 行</span>
        {/snippet}
        <TextArea bind:value={textB} fill placeholder="在此粘贴文本 B…" ariaLabel="文本 B" />
      </Panel>
    {/snippet}
  </SplitPane>

  {#if hasContent}
    <DiffView
      {result}
      leftTitle="文本 A"
      rightTitle="文本 B"
      {onlyChanges}
      maxHeight="42vh"
    />
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
</style>
