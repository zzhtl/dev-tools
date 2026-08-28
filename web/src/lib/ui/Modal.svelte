<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    open: boolean;
    onclose: () => void;
    title?: string;
    width?: string;
    /** 命令面板靠上更符合手感，普通弹窗居中 */
    align?: "center" | "top";
    children: Snippet;
  }

  let { open, onclose, title, width = "min(560px, 92vw)", align = "center", children }: Props =
    $props();
</script>

{#if open}
  <!-- 背板：点击关闭。键盘用户走 Esc（在 App 的全局 keydown 里处理） -->
  <div
    class="backdrop"
    class:top={align === "top"}
    role="presentation"
    onclick={(event) => {
      if (event.target === event.currentTarget) onclose();
    }}
  >
    <div class="modal" style="width: {width}" role="dialog" aria-modal="true" aria-label={title}>
      {#if title}
        <header class="head">
          <h2>{title}</h2>
          <button class="close" onclick={onclose} aria-label="关闭">✕</button>
        </header>
      {/if}
      {@render children()}
    </div>
  </div>
{/if}

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    z-index: var(--z-modal);
    display: flex;
    align-items: center;
    justify-content: center;
    padding: var(--sp-5);
    background: rgba(0, 0, 0, 0.45);
    backdrop-filter: blur(2px);
    animation: fade 0.12s ease;
  }

  .backdrop.top {
    align-items: flex-start;
    padding-top: 12vh;
  }

  .modal {
    display: flex;
    flex-direction: column;
    max-height: 76vh;
    min-height: 0;
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-lg);
    overflow: hidden;
    animation: rise 0.14s ease;
  }

  .head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--sp-4);
    border-bottom: 1px solid var(--color-border);
  }

  .head h2 {
    font-size: var(--fs-lg);
    font-weight: var(--fw-semibold);
  }

  .close {
    color: var(--color-text-muted);
    padding: var(--sp-1);
    border-radius: var(--radius-sm);
    transition: color var(--motion), background var(--motion);
  }

  .close:hover {
    color: var(--color-text);
    background: var(--color-surface-2);
  }

  @keyframes fade {
    from {
      opacity: 0;
    }
  }

  @keyframes rise {
    from {
      opacity: 0;
      transform: translateY(-6px);
    }
  }
</style>
