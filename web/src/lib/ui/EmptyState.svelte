<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    icon?: string;
    title: string;
    hint?: string;
    children?: Snippet;
  }

  let { icon = "○", title, hint, children }: Props = $props();
</script>

<div class="empty">
  <span class="icon" aria-hidden="true">{icon}</span>
  <p class="title">{title}</p>
  {#if hint}<p class="hint">{hint}</p>{/if}
  {#if children}<div class="extra">{@render children()}</div>{/if}
</div>

<style>
  .empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--sp-2);
    padding: var(--sp-8) var(--sp-4);
    text-align: center;
    color: var(--color-text-muted);
  }

  .icon {
    font-size: 1.75rem;
    opacity: 0.6;
    line-height: 1;
  }

  .title {
    font-size: var(--fs-md);
    color: var(--color-text-secondary);
  }

  .hint {
    font-size: var(--fs-sm);
    max-width: 34rem;
  }

  .extra {
    margin-top: var(--sp-2);
  }
</style>
