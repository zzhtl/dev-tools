<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    title?: string;
    subtitle?: string;
    /** 标题栏右侧操作区 */
    actions?: Snippet;
    /** 去掉内容内边距，交给内容自己控制（编辑器、表格等） */
    flush?: boolean;
    /** 内容区撑满剩余高度 */
    grow?: boolean;
    children: Snippet;
  }

  let { title, subtitle, actions, flush = false, grow = false, children }: Props = $props();
</script>

<section class="panel" class:grow>
  {#if title || actions}
    <header class="head">
      <div class="titles">
        {#if title}<h2 class="title">{title}</h2>{/if}
        {#if subtitle}<p class="subtitle">{subtitle}</p>{/if}
      </div>
      {#if actions}
        <div class="actions">{@render actions()}</div>
      {/if}
    </header>
  {/if}
  <div class="body" class:flush>
    {@render children()}
  </div>
</section>

<style>
  .panel {
    display: flex;
    flex-direction: column;
    min-height: 0;
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius);
    overflow: hidden;
  }

  .grow {
    flex: 1;
  }

  .head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--sp-3);
    padding: 0 var(--sp-4);
    border-bottom: 1px solid var(--color-border);
    /* 固定高度：多个 Panel 并排时，头部有无按钮都能对齐 */
    height: 44px;
    flex-shrink: 0;
  }

  .titles {
    min-width: 0;
  }

  .title {
    font-size: var(--fs-md);
    font-weight: var(--fw-semibold);
    color: var(--color-text);
    line-height: var(--lh-tight);
  }

  .subtitle {
    margin-top: 2px;
    font-size: var(--fs-xs);
    color: var(--color-text-muted);
  }

  .actions {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    flex-shrink: 0;
  }

  .body {
    flex: 1;
    min-height: 0;
    padding: var(--sp-4);
    overflow: auto;
    display: flex;
    flex-direction: column;
  }

  .body.flush {
    padding: 0;
  }
</style>
