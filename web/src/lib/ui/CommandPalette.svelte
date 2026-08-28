<script lang="ts">
  import { TOOLS } from "../tools";
  import { searchTools } from "../search";
  import { prefs } from "../prefs.svelte";
  import Modal from "./Modal.svelte";

  interface Props {
    open: boolean;
    onclose: () => void;
    onselect: (id: string) => void;
  }

  let { open, onclose, onselect }: Props = $props();

  let query = $state("");
  let cursor = $state(0);
  let input: HTMLInputElement | undefined = $state();

  const results = $derived(searchTools(TOOLS, query));

  // 每次打开重置，并把焦点交给输入框
  $effect(() => {
    if (open) {
      query = "";
      cursor = 0;
      queueMicrotask(() => input?.focus());
    }
  });

  // 结果集变短时避免游标越界
  $effect(() => {
    if (cursor > results.length - 1) cursor = Math.max(0, results.length - 1);
  });

  function choose(id: string) {
    onselect(id);
    onclose();
  }

  function onKeydown(event: KeyboardEvent) {
    if (event.key === "ArrowDown") {
      event.preventDefault();
      cursor = results.length === 0 ? 0 : (cursor + 1) % results.length;
    } else if (event.key === "ArrowUp") {
      event.preventDefault();
      cursor = results.length === 0 ? 0 : (cursor - 1 + results.length) % results.length;
    } else if (event.key === "Enter") {
      event.preventDefault();
      const hit = results[cursor];
      if (hit) choose(hit.id);
    }
  }
</script>

<Modal {open} {onclose} align="top" width="min(620px, 94vw)">
  <div class="search">
    <span class="glyph" aria-hidden="true">⌕</span>
    <input
      bind:this={input}
      bind:value={query}
      class="input"
      type="text"
      placeholder="搜索工具…"
      aria-label="搜索工具"
      onkeydown={onKeydown}
    />
  </div>

  <div class="results">
    {#each results as tool, index (tool.id)}
      <button
        class="hit"
        class:active={index === cursor}
        onclick={() => choose(tool.id)}
        onmouseenter={() => (cursor = index)}
      >
        <span class="icon" aria-hidden="true">{tool.icon}</span>
        <span class="text">
          <span class="name">
            {tool.name}
            {#if prefs.isFavorite(tool.id)}<span class="star" aria-label="已收藏">★</span>{/if}
          </span>
          <span class="desc">{tool.desc}</span>
        </span>
        <span class="cat">{tool.category}</span>
      </button>
    {:else}
      <p class="none">没有匹配的工具</p>
    {/each}
  </div>

  <footer class="foot">
    <span><kbd>↑</kbd><kbd>↓</kbd> 选择</span>
    <span><kbd>Enter</kbd> 打开</span>
    <span><kbd>Esc</kbd> 关闭</span>
  </footer>
</Modal>

<style>
  .search {
    display: flex;
    align-items: center;
    gap: var(--sp-3);
    padding: var(--sp-4);
    border-bottom: 1px solid var(--color-border);
  }

  .glyph {
    font-size: var(--fs-xl);
    color: var(--color-text-muted);
  }

  .input {
    flex: 1;
    background: none;
    border: none;
    outline: none;
    color: var(--color-text);
    font-size: var(--fs-lg);
  }

  .input::placeholder {
    color: var(--color-text-muted);
  }

  .results {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    padding: var(--sp-2);
  }

  .hit {
    display: flex;
    align-items: center;
    gap: var(--sp-3);
    width: 100%;
    padding: var(--sp-2) var(--sp-3);
    border-radius: var(--radius-sm);
    text-align: left;
    color: var(--color-text);
  }

  .hit.active {
    background: var(--color-primary-bg);
  }

  .icon {
    width: 1.75rem;
    flex-shrink: 0;
    text-align: center;
    font-family: var(--font-mono);
    color: var(--color-text-secondary);
  }

  .text {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
  }

  .name {
    font-size: var(--fs-md);
    font-weight: var(--fw-medium);
  }

  .star {
    color: var(--color-warning);
    font-size: var(--fs-xs);
  }

  .desc {
    font-size: var(--fs-xs);
    color: var(--color-text-muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .cat {
    flex-shrink: 0;
    font-size: var(--fs-xs);
    color: var(--color-text-muted);
  }

  .none {
    padding: var(--sp-6);
    text-align: center;
    color: var(--color-text-muted);
    font-size: var(--fs-md);
  }

  .foot {
    display: flex;
    gap: var(--sp-4);
    padding: var(--sp-2) var(--sp-4);
    border-top: 1px solid var(--color-border);
    background: var(--color-surface-2);
    font-size: var(--fs-xs);
    color: var(--color-text-muted);
  }

  kbd {
    display: inline-block;
    min-width: 1.2em;
    padding: 1px 4px;
    margin-right: 2px;
    border: 1px solid var(--color-border-strong);
    border-radius: 4px;
    background: var(--color-surface);
    font-family: var(--font-mono);
    font-size: 0.9em;
    text-align: center;
  }
</style>
