<script lang="ts">
  import { untrack, type Snippet } from "svelte";

  interface Props {
    left: Snippet;
    right: Snippet;
    /** 左侧初始占比 0.2 ~ 0.8 */
    ratio?: number;
    /** 不可拖动时退化为等宽两栏 */
    resizable?: boolean;
    gap?: string;
    children?: never;
  }

  let { left, right, ratio = 0.5, resizable = true, gap = "var(--sp-3)" }: Props = $props();

  // 仅取初值：之后由拖拽接管，不跟随 prop 变化
  let current = $state(untrack(() => ratio));
  let container: HTMLDivElement | undefined;
  let dragging = $state(false);

  const MIN = 0.2;
  const MAX = 0.8;

  function startDrag(event: PointerEvent) {
    if (!resizable) return;
    dragging = true;
    (event.currentTarget as HTMLElement).setPointerCapture(event.pointerId);
  }

  function onDrag(event: PointerEvent) {
    if (!dragging || !container) return;
    const rect = container.getBoundingClientRect();
    const next = (event.clientX - rect.left) / rect.width;
    current = Math.min(MAX, Math.max(MIN, next));
  }

  function endDrag(event: PointerEvent) {
    if (!dragging) return;
    dragging = false;
    (event.currentTarget as HTMLElement).releasePointerCapture(event.pointerId);
  }

  function onKeydown(event: KeyboardEvent) {
    if (event.key === "ArrowLeft") current = Math.max(MIN, current - 0.02);
    else if (event.key === "ArrowRight") current = Math.min(MAX, current + 0.02);
    else return;
    event.preventDefault();
  }
</script>

<div
  class="split"
  class:dragging
  bind:this={container}
  style="--ratio: {current}; --split-gap: {gap};"
>
  <div class="pane">{@render left()}</div>
  {#if resizable}
    <button
      class="divider"
      type="button"
      role="slider"
      aria-orientation="vertical"
      aria-valuemin={20}
      aria-valuemax={80}
      aria-valuenow={Math.round(current * 100)}
      aria-label="拖动调整两栏宽度"
      onpointerdown={startDrag}
      onpointermove={onDrag}
      onpointerup={endDrag}
      onpointercancel={endDrag}
      onkeydown={onKeydown}
    ></button>
  {/if}
  <div class="pane">{@render right()}</div>
</div>

<style>
  .split {
    display: grid;
    grid-template-columns: calc(var(--ratio) * 100% - var(--split-gap)) auto 1fr;
    gap: var(--split-gap);
    min-height: 0;
    flex: 1;
  }

  .pane {
    display: flex;
    flex-direction: column;
    min-width: 0;
    min-height: 0;
  }

  .divider {
    width: 5px;
    margin: 0 -2px;
    border-radius: var(--radius-full);
    background: transparent;
    cursor: col-resize;
    transition: background var(--motion);
  }

  .divider:hover,
  .dragging .divider {
    background: var(--color-primary);
  }

  /* 窄屏退化为上下堆叠 */
  @media (max-width: 900px) {
    .split {
      grid-template-columns: 1fr;
    }

    .divider {
      display: none;
    }
  }
</style>
