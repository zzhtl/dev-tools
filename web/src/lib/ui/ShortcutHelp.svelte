<script lang="ts">
  import { GLOBAL_SHORTCUTS } from "../shortcuts";
  import Modal from "./Modal.svelte";

  interface Props {
    open: boolean;
    onclose: () => void;
  }

  let { open, onclose }: Props = $props();
</script>

<Modal {open} {onclose} title="快捷键" width="min(420px, 92vw)">
  <ul class="list">
    {#each GLOBAL_SHORTCUTS as item (item.keys)}
      <li>
        <span class="desc">{item.desc}</span>
        <kbd>{item.keys}</kbd>
      </li>
    {/each}
  </ul>
</Modal>

<style>
  .list {
    list-style: none;
    padding: var(--sp-2) var(--sp-4) var(--sp-4);
    overflow-y: auto;
  }

  li {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--sp-4);
    padding: var(--sp-2) 0;
  }

  li + li {
    border-top: 1px solid var(--color-border);
  }

  .desc {
    font-size: var(--fs-md);
    color: var(--color-text-secondary);
  }

  kbd {
    padding: 2px var(--sp-2);
    border: 1px solid var(--color-border-strong);
    border-radius: var(--radius-sm);
    background: var(--color-surface-2);
    font-family: var(--font-mono);
    font-size: var(--fs-sm);
    color: var(--color-text);
    white-space: nowrap;
  }
</style>
