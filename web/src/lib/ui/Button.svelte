<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    variant?: "primary" | "secondary" | "ghost" | "danger";
    size?: "sm" | "md";
    disabled?: boolean;
    loading?: boolean;
    /** 撑满父容器宽度 */
    block?: boolean;
    /** 仅图标，收窄内边距并保持方形 */
    icon?: boolean;
    title?: string;
    ariaLabel?: string;
    type?: "button" | "submit" | "reset";
    onclick?: (event: MouseEvent) => void;
    children: Snippet;
  }

  let {
    variant = "secondary",
    size = "md",
    disabled = false,
    loading = false,
    block = false,
    icon = false,
    title,
    ariaLabel,
    type = "button",
    onclick,
    children,
  }: Props = $props();
</script>

<button
  class="btn {variant} {size}"
  class:block
  class:icon
  class:loading
  {type}
  {title}
  aria-label={ariaLabel}
  aria-busy={loading}
  disabled={disabled || loading}
  {onclick}
>
  {#if loading}<span class="spinner" aria-hidden="true"></span>{/if}
  {@render children()}
</button>

<style>
  .btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: var(--sp-2);
    border: 1px solid transparent;
    border-radius: var(--radius-sm);
    font-weight: var(--fw-medium);
    white-space: nowrap;
    cursor: pointer;
    transition:
      background var(--motion),
      border-color var(--motion),
      color var(--motion),
      opacity var(--motion);
  }

  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .md {
    padding: 0.5rem var(--sp-4);
    font-size: var(--fs-md);
  }

  .sm {
    padding: 0.3rem var(--sp-3);
    font-size: var(--fs-sm);
  }

  .block {
    width: 100%;
  }

  .icon.md {
    padding: 0.5rem;
    min-width: 34px;
  }

  .icon.sm {
    padding: 0.3rem;
    min-width: 26px;
  }

  .primary {
    background: var(--color-primary);
    color: var(--color-text-inverse);
  }

  .primary:not(:disabled):hover {
    background: var(--color-primary-hover);
  }

  .secondary {
    background: var(--color-surface-2);
    border-color: var(--color-border);
    color: var(--color-text);
  }

  .secondary:not(:disabled):hover {
    background: var(--color-surface-3);
    border-color: var(--color-border-strong);
  }

  .ghost {
    background: transparent;
    color: var(--color-text-secondary);
  }

  .ghost:not(:disabled):hover {
    background: var(--color-surface-2);
    color: var(--color-text);
  }

  .danger {
    background: var(--color-danger-bg);
    border-color: var(--color-danger-border);
    color: var(--color-danger);
  }

  .danger:not(:disabled):hover {
    background: var(--color-danger);
    color: var(--color-text-inverse);
  }

  .spinner {
    width: 0.85em;
    height: 0.85em;
    border: 2px solid currentColor;
    border-right-color: transparent;
    border-radius: var(--radius-full);
    animation: spin 0.6s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>
