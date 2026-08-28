<script lang="ts">
  import type { DiffResult } from "../diff";

  interface Props {
    result: DiffResult;
    /** 左右两栏标题 */
    leftTitle?: string;
    rightTitle?: string;
    /** 只显示有差异的行及其上下文 */
    onlyChanges?: boolean;
    contextLines?: number;
    maxHeight?: string;
  }

  let {
    result,
    leftTitle = "原文",
    rightTitle = "对比",
    onlyChanges = false,
    contextLines = 2,
    maxHeight = "60vh",
  }: Props = $props();

  const visibleRows = $derived.by(() => {
    if (!onlyChanges) return result.rows.map((row, index) => ({ row, index }));
    const keep = new Set<number>();
    result.rows.forEach((row, index) => {
      if (row.type === "equal") return;
      for (let i = index - contextLines; i <= index + contextLines; i++) {
        if (i >= 0 && i < result.rows.length) keep.add(i);
      }
    });
    return result.rows
      .map((row, index) => ({ row, index }))
      .filter(({ index }) => keep.has(index));
  });

  const identical = $derived(result.added === 0 && result.removed === 0 && result.changed === 0);
</script>

<div class="diff">
  <div class="summary">
    <span class="stat add">+{result.added}</span>
    <span class="stat del">-{result.removed}</span>
    <span class="stat chg">~{result.changed}</span>
    {#if result.notice}
      <span class="notice">{result.notice}</span>
    {:else if identical}
      <span class="same">两侧内容完全一致</span>
    {/if}
  </div>

  <div class="headers">
    <span>{leftTitle}</span>
    <span>{rightTitle}</span>
  </div>

  <div class="rows" style="max-height: {maxHeight}">
    {#each visibleRows as { row, index } (index)}
      <div class="row {row.type}">
        <span class="ln">{row.leftNo ?? ""}</span>
        <span class="cell left">{row.left}</span>
        <span class="ln">{row.rightNo ?? ""}</span>
        <span class="cell right">{row.right}</span>
      </div>
    {/each}
  </div>
</div>

<style>
  .diff {
    display: flex;
    flex-direction: column;
    min-height: 0;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    overflow: hidden;
  }

  .summary {
    display: flex;
    align-items: center;
    gap: var(--sp-3);
    padding: var(--sp-2) var(--sp-3);
    background: var(--color-surface-2);
    border-bottom: 1px solid var(--color-border);
    font-size: var(--fs-sm);
    font-family: var(--font-mono);
  }

  .stat.add {
    color: var(--color-success);
  }

  .stat.del {
    color: var(--color-danger);
  }

  .stat.chg {
    color: var(--color-warning);
  }

  .same {
    margin-left: auto;
    font-family: var(--font-sans);
    color: var(--color-text-muted);
  }

  .notice {
    margin-left: auto;
    font-family: var(--font-sans);
    color: var(--color-warning);
  }

  .headers {
    display: grid;
    grid-template-columns: 1fr 1fr;
    background: var(--color-surface-2);
    border-bottom: 1px solid var(--color-border);
    font-size: var(--fs-xs);
    color: var(--color-text-muted);
  }

  .headers span {
    padding: var(--sp-1) var(--sp-3);
  }

  .headers span:first-child {
    border-right: 1px solid var(--color-border);
  }

  .rows {
    overflow: auto;
    background: var(--color-code-bg);
    font-family: var(--font-mono);
    font-size: var(--fs-sm);
    line-height: 1.65;
  }

  .row {
    display: grid;
    grid-template-columns: 3.2rem 1fr 3.2rem 1fr;
  }

  .ln {
    padding: 0 var(--sp-2);
    text-align: right;
    color: var(--color-text-muted);
    user-select: none;
    background: var(--color-surface-2);
    border-right: 1px solid var(--color-border);
  }

  .cell {
    padding: 0 var(--sp-2);
    white-space: pre-wrap;
    word-break: break-all;
    color: var(--color-text-secondary);
  }

  .row.insert .cell.right,
  .row.replace .cell.right {
    background: var(--color-diff-add-bg);
    color: var(--color-diff-add-text);
  }

  .row.delete .cell.left,
  .row.replace .cell.left {
    background: var(--color-diff-del-bg);
    color: var(--color-diff-del-text);
  }
</style>
