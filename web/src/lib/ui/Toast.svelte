<script lang="ts">
  import { toastStore } from "../toast.svelte";
</script>

<div class="toast-layer" role="status" aria-live="polite">
  {#each toastStore.items as item (item.id)}
    <button class="toast {item.kind}" onclick={() => toastStore.dismiss(item.id)} title="点击关闭">
      <span class="mark" aria-hidden="true">
        {item.kind === "success" ? "✓" : item.kind === "error" ? "✕" : "i"}
      </span>
      <span class="text">{item.text}</span>
    </button>
  {/each}
</div>

<style>
  .toast-layer {
    position: fixed;
    right: var(--sp-5);
    bottom: var(--sp-5);
    z-index: var(--z-toast);
    display: flex;
    flex-direction: column;
    gap: var(--sp-2);
    align-items: flex-end;
    pointer-events: none;
  }

  .toast {
    pointer-events: auto;
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    max-width: 380px;
    padding: var(--sp-2) var(--sp-4);
    border: 1px solid var(--color-border);
    border-radius: var(--radius);
    background: var(--color-surface);
    color: var(--color-text);
    font-size: var(--fs-md);
    text-align: left;
    box-shadow: var(--shadow);
    animation: slide-in 0.16s ease;
  }

  .mark {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 1.15rem;
    height: 1.15rem;
    flex-shrink: 0;
    border-radius: var(--radius-full);
    font-size: var(--fs-xs);
    font-weight: var(--fw-bold);
  }

  .success .mark {
    background: var(--color-success-bg);
    color: var(--color-success);
  }

  .error .mark {
    background: var(--color-danger-bg);
    color: var(--color-danger);
  }

  .info .mark {
    background: var(--color-primary-bg);
    color: var(--color-primary);
  }

  .success {
    border-color: var(--color-success-border);
  }

  .error {
    border-color: var(--color-danger-border);
  }

  @keyframes slide-in {
    from {
      opacity: 0;
      transform: translateY(6px);
    }
  }
</style>
