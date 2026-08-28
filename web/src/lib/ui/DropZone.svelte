<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    onfiles: (files: File[]) => void;
    accept?: string;
    multiple?: boolean;
    disabled?: boolean;
    hint?: string;
    /** 自定义内容；不给就用默认的图标+提示 */
    children?: Snippet;
  }

  let {
    onfiles,
    accept,
    multiple = false,
    disabled = false,
    hint = "拖拽文件到此处，或点击选择",
    children,
  }: Props = $props();

  let dragging = $state(false);
  let input: HTMLInputElement | undefined;

  function emit(list: FileList | null) {
    if (!list || list.length === 0) return;
    onfiles(Array.from(list));
  }

  function onDrop(event: DragEvent) {
    event.preventDefault();
    dragging = false;
    if (disabled) return;
    emit(event.dataTransfer?.files ?? null);
  }

  function onDragOver(event: DragEvent) {
    event.preventDefault();
    if (!disabled) dragging = true;
  }
</script>

<div
  class="zone"
  class:dragging
  class:disabled
  role="button"
  tabindex="0"
  ondrop={onDrop}
  ondragover={onDragOver}
  ondragleave={() => (dragging = false)}
  onclick={() => !disabled && input?.click()}
  onkeydown={(event) => {
    if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      if (!disabled) input?.click();
    }
  }}
>
  {#if children}
    {@render children()}
  {:else}
    <span class="icon" aria-hidden="true">⇪</span>
    <span class="hint">{hint}</span>
  {/if}
  <input
    bind:this={input}
    class="file"
    type="file"
    {accept}
    {multiple}
    {disabled}
    onchange={(event) => {
      emit(event.currentTarget.files);
      event.currentTarget.value = "";
    }}
  />
</div>

<style>
  .zone {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--sp-2);
    padding: var(--sp-7) var(--sp-4);
    border: 1.5px dashed var(--color-border-strong);
    border-radius: var(--radius);
    background: var(--color-surface-2);
    color: var(--color-text-secondary);
    text-align: center;
    cursor: pointer;
    transition:
      border-color var(--motion),
      background var(--motion),
      color var(--motion);
  }

  .zone:hover:not(.disabled) {
    border-color: var(--color-primary);
    color: var(--color-text);
  }

  .dragging {
    border-color: var(--color-primary);
    background: var(--color-primary-bg);
    color: var(--color-text);
  }

  .disabled {
    opacity: 0.55;
    cursor: not-allowed;
  }

  .icon {
    font-size: var(--fs-2xl);
    line-height: 1;
  }

  .hint {
    font-size: var(--fs-sm);
  }

  .file {
    display: none;
  }
</style>
