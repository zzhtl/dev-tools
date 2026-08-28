<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    label?: string;
    hint?: string;
    error?: string;
    /** 标签与控件横向排列 */
    inline?: boolean;
    children: Snippet;
  }

  let { label, hint, error, inline = false, children }: Props = $props();
</script>

<div class="field" class:inline>
  {#if label}<span class="label">{label}</span>{/if}
  <div class="control">
    {@render children()}
    {#if error}
      <p class="error">{error}</p>
    {:else if hint}
      <p class="hint">{hint}</p>
    {/if}
  </div>
</div>

<style>
  .field {
    display: flex;
    flex-direction: column;
    gap: var(--sp-1);
    min-width: 0;
  }

  .inline {
    flex-direction: row;
    align-items: center;
    gap: var(--sp-2);
  }

  .label {
    font-size: var(--fs-sm);
    font-weight: var(--fw-medium);
    color: var(--color-text-secondary);
    white-space: nowrap;
  }

  .control {
    min-width: 0;
  }

  .inline .control {
    flex: 1;
  }

  .hint {
    margin-top: var(--sp-1);
    font-size: var(--fs-xs);
    color: var(--color-text-muted);
  }

  .error {
    margin-top: var(--sp-1);
    font-size: var(--fs-xs);
    color: var(--color-danger);
  }
</style>
