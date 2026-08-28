<script lang="ts">
  interface Tab {
    value: string;
    label: string;
    icon?: string;
  }

  interface Props {
    value?: string;
    tabs: Tab[];
    size?: "sm" | "md";
    ariaLabel?: string;
    onchange?: (value: string) => void;
  }

  let { value = $bindable(""), tabs, size = "md", ariaLabel, onchange }: Props = $props();

  function select(next: string) {
    if (next === value) return;
    value = next;
    onchange?.(next);
  }
</script>

<div class="tabs {size}" role="tablist" aria-label={ariaLabel}>
  {#each tabs as tab (tab.value)}
    <button
      class="tab"
      class:active={value === tab.value}
      role="tab"
      aria-selected={value === tab.value}
      onclick={() => select(tab.value)}
    >
      {#if tab.icon}<span class="icon" aria-hidden="true">{tab.icon}</span>{/if}
      {tab.label}
    </button>
  {/each}
</div>

<style>
  .tabs {
    display: flex;
    flex-wrap: wrap;
    gap: 2px;
    padding: 3px;
    background: var(--color-surface-2);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
  }

  .tab {
    display: inline-flex;
    align-items: center;
    gap: var(--sp-1);
    border-radius: calc(var(--radius-sm) - 2px);
    color: var(--color-text-secondary);
    font-weight: var(--fw-medium);
    white-space: nowrap;
    transition:
      background var(--motion),
      color var(--motion);
  }

  .md .tab {
    padding: 0.35rem var(--sp-3);
    font-size: var(--fs-md);
  }

  .sm .tab {
    padding: 0.25rem var(--sp-2);
    font-size: var(--fs-sm);
  }

  .tab:hover {
    color: var(--color-text);
    background: var(--color-surface-3);
  }

  .tab.active {
    background: var(--color-surface);
    color: var(--color-primary);
    box-shadow: var(--shadow-sm);
  }

  .icon {
    font-family: var(--font-mono);
  }
</style>
