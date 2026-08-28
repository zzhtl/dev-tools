<script lang="ts">
  import { onMount, untrack, type Component } from "svelte";
  import { TOOLS, TOOL_BY_ID, groupByCategory, type ToolMeta } from "./lib/tools";
  import { initRouter, router } from "./lib/router.svelte";
  import { initTheme, theme, THEME_ICONS, THEME_LABELS } from "./lib/theme.svelte";
  import { prefs } from "./lib/prefs.svelte";
  import { isEditableTarget, matchesShortcut, MOD_LABEL } from "./lib/shortcuts";
  import CommandPalette from "./lib/ui/CommandPalette.svelte";
  import ShortcutHelp from "./lib/ui/ShortcutHelp.svelte";
  import Toast from "./lib/ui/Toast.svelte";

  type ToolComponent = Component<any>;

  onMount(() => {
    initTheme();
    initRouter();
  });

  const activeTool = $derived(TOOL_BY_ID.get(router.id) ?? TOOLS[0]);
  const categories = groupByCategory(TOOLS);

  const favoriteTools = $derived(
    prefs.favorites.map((id) => TOOL_BY_ID.get(id)).filter((tool): tool is ToolMeta => !!tool),
  );
  const recentTools = $derived(
    prefs.recents
      .map((id) => TOOL_BY_ID.get(id))
      .filter((tool): tool is ToolMeta => !!tool)
      .slice(0, 5),
  );
  // 只有一条时就是当前工具本身，与下方分类完全重复，没必要单独占一组
  const recentGroup = $derived(recentTools.length >= 2 ? recentTools : []);

  // ── 按需加载工具组件 ──
  const cache = new Map<string, ToolComponent>();
  let ToolView = $state<ToolComponent | null>(null);
  let loadFailed = $state(false);

  $effect(() => {
    const tool = activeTool;
    const cached = cache.get(tool.id);
    if (cached) {
      ToolView = cached;
      loadFailed = false;
      return;
    }
    ToolView = null;
    loadFailed = false;
    let cancelled = false;
    tool
      .load()
      .then((module) => {
        if (cancelled) return;
        cache.set(tool.id, module.default);
        ToolView = module.default;
      })
      .catch(() => {
        if (!cancelled) loadFailed = true;
      });
    return () => {
      cancelled = true;
    };
  });

  // 记录最近使用。touch 内部会读写同一份状态，必须 untrack 否则自触发成环。
  $effect(() => {
    const id = activeTool.id;
    untrack(() => prefs.touch(id));
  });

  let paletteOpen = $state(false);
  let helpOpen = $state(false);

  function handleKeydown(event: KeyboardEvent) {
    if (matchesShortcut(event, { key: "k", mod: true })) {
      event.preventDefault();
      paletteOpen = !paletteOpen;
      return;
    }
    if (matchesShortcut(event, { key: "/", mod: true })) {
      event.preventDefault();
      helpOpen = !helpOpen;
      return;
    }
    if (matchesShortcut(event, { key: "b", mod: true })) {
      event.preventDefault();
      prefs.toggleSidebar();
      return;
    }
    if (matchesShortcut(event, { key: "l", mod: true, shift: true })) {
      event.preventDefault();
      theme.cycle();
      return;
    }
    if (event.key === "Escape") {
      if (paletteOpen || helpOpen) {
        event.preventDefault();
        paletteOpen = false;
        helpOpen = false;
      }
      return;
    }
    // 其余按键交还给工具自身；输入框内更是完全不拦截
    if (isEditableTarget(event.target)) return;
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="app" class:collapsed={prefs.sidebarCollapsed}>
  <aside class="sidebar">
    <div class="brand">
      <span class="logo" aria-hidden="true">⚡</span>
      {#if !prefs.sidebarCollapsed}<span class="brand-name">开发工具箱</span>{/if}
      <button
        class="collapse"
        onclick={() => prefs.toggleSidebar()}
        title={`${prefs.sidebarCollapsed ? "展开" : "收起"}侧边栏 (${MOD_LABEL} B)`}
        aria-label={prefs.sidebarCollapsed ? "展开侧边栏" : "收起侧边栏"}
      >
        {prefs.sidebarCollapsed ? "»" : "«"}
      </button>
    </div>

    <nav class="nav">
      {#snippet navGroup(name: string, tools: ToolMeta[])}
        {#if tools.length > 0}
          <div class="group">
            {#if !prefs.sidebarCollapsed}<p class="group-name">{name}</p>{/if}
            {#each tools as tool (tool.id)}
              <button
                class="item"
                class:active={activeTool.id === tool.id}
                onclick={() => router.navigate(tool.id)}
                title={prefs.sidebarCollapsed ? `${tool.name} — ${tool.desc}` : tool.desc}
              >
                <span class="item-icon" aria-hidden="true">{tool.icon}</span>
                {#if !prefs.sidebarCollapsed}<span class="item-name">{tool.name}</span>{/if}
              </button>
            {/each}
          </div>
        {/if}
      {/snippet}

      {@render navGroup("收藏", favoriteTools)}
      {@render navGroup("最近使用", recentGroup)}
      {#each categories as category (category.name)}
        {@render navGroup(category.name, category.tools)}
      {/each}
    </nav>
  </aside>

  <div class="main">
    <header class="topbar">
      <div class="crumb">
        <span class="crumb-icon" aria-hidden="true">{activeTool.icon}</span>
        <span class="crumb-text">
          <span class="crumb-name">{activeTool.name}</span>
          <span class="crumb-desc">{activeTool.desc}</span>
        </span>
      </div>

      <div class="top-actions">
        <button class="search-entry" onclick={() => (paletteOpen = true)} title="搜索工具">
          <span aria-hidden="true">⌕</span>
          <span class="search-label">搜索工具</span>
          <kbd>{MOD_LABEL} K</kbd>
        </button>

        <button
          class="icon-action"
          class:starred={prefs.isFavorite(activeTool.id)}
          onclick={() => prefs.toggleFavorite(activeTool.id)}
          title={prefs.isFavorite(activeTool.id) ? "取消收藏" : "收藏此工具"}
          aria-label={prefs.isFavorite(activeTool.id) ? "取消收藏" : "收藏此工具"}
        >
          {prefs.isFavorite(activeTool.id) ? "★" : "☆"}
        </button>

        <button
          class="icon-action"
          onclick={() => theme.cycle()}
          title={`主题：${THEME_LABELS[theme.mode]}（${MOD_LABEL} ⇧ L）`}
          aria-label="切换主题"
        >
          {THEME_ICONS[theme.mode]}
        </button>

        <button
          class="icon-action"
          onclick={() => (helpOpen = true)}
          title={`快捷键 (${MOD_LABEL} /)`}
          aria-label="快捷键帮助"
        >
          ?
        </button>
      </div>
    </header>

    <main class="content">
      <div class="tool-host">
        {#if ToolView}
          <ToolView />
        {:else if loadFailed}
          <p class="load-state error">工具加载失败，请刷新页面重试。</p>
        {:else}
          <p class="load-state">正在加载 {activeTool.name}…</p>
        {/if}
      </div>
    </main>
  </div>
</div>

<CommandPalette
  open={paletteOpen}
  onclose={() => (paletteOpen = false)}
  onselect={(id) => router.navigate(id)}
/>
<ShortcutHelp open={helpOpen} onclose={() => (helpOpen = false)} />
<Toast />

<style>
  .app {
    display: flex;
    height: 100vh;
    width: 100vw;
    overflow: hidden;
  }

  /* ── 侧边栏 ── */
  .sidebar {
    display: flex;
    flex-direction: column;
    width: var(--sidebar-width);
    flex-shrink: 0;
    background: var(--color-surface);
    border-right: 1px solid var(--color-border);
    transition: width var(--motion-slow);
  }

  .collapsed .sidebar {
    width: var(--sidebar-width-collapsed);
  }

  .brand {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    height: var(--topbar-height);
    padding: 0 var(--sp-3);
    border-bottom: 1px solid var(--color-border);
    flex-shrink: 0;
  }

  .logo {
    font-size: var(--fs-lg);
    line-height: 1;
  }

  .brand-name {
    font-size: var(--fs-md);
    font-weight: var(--fw-semibold);
    white-space: nowrap;
    overflow: hidden;
  }

  .collapse {
    margin-left: auto;
    padding: var(--sp-1);
    border-radius: var(--radius-sm);
    color: var(--color-text-muted);
    font-size: var(--fs-sm);
    transition:
      color var(--motion),
      background var(--motion);
  }

  .collapse:hover {
    color: var(--color-text);
    background: var(--color-surface-2);
  }

  .nav {
    flex: 1;
    overflow-y: auto;
    padding: var(--sp-3) var(--sp-2);
  }

  .group + .group {
    margin-top: var(--sp-4);
  }

  .group-name {
    padding: 0 var(--sp-2);
    margin-bottom: var(--sp-1);
    font-size: var(--fs-xs);
    font-weight: var(--fw-semibold);
    letter-spacing: 0.06em;
    color: var(--color-text-muted);
  }

  .item {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    width: 100%;
    padding: 0.4rem var(--sp-2);
    border-radius: var(--radius-sm);
    color: var(--color-text-secondary);
    font-size: var(--fs-md);
    text-align: left;
    transition:
      background var(--motion),
      color var(--motion);
  }

  .item:hover {
    background: var(--color-surface-2);
    color: var(--color-text);
  }

  .item.active {
    background: var(--color-primary-bg);
    color: var(--color-primary);
    font-weight: var(--fw-medium);
  }

  .item-icon {
    width: 1.5rem;
    flex-shrink: 0;
    text-align: center;
    font-family: var(--font-mono);
    font-size: var(--fs-sm);
  }

  .item-name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .collapsed .item {
    justify-content: center;
  }

  /* ── 主区 ── */
  .main {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
    background: var(--color-bg);
  }

  .topbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--sp-4);
    height: var(--topbar-height);
    padding: 0 var(--sp-4);
    border-bottom: 1px solid var(--color-border);
    background: var(--color-surface);
    flex-shrink: 0;
  }

  .crumb {
    display: flex;
    align-items: center;
    gap: var(--sp-3);
    min-width: 0;
  }

  .crumb-icon {
    font-family: var(--font-mono);
    color: var(--color-text-muted);
  }

  .crumb-text {
    display: flex;
    align-items: baseline;
    gap: var(--sp-3);
    min-width: 0;
  }

  .crumb-name {
    font-size: var(--fs-lg);
    font-weight: var(--fw-semibold);
    white-space: nowrap;
  }

  .crumb-desc {
    font-size: var(--fs-xs);
    color: var(--color-text-muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .top-actions {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    flex-shrink: 0;
  }

  .search-entry {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    padding: 0.3rem var(--sp-2) 0.3rem var(--sp-3);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    background: var(--color-bg);
    color: var(--color-text-muted);
    font-size: var(--fs-sm);
    transition:
      border-color var(--motion),
      color var(--motion);
  }

  .search-entry:hover {
    border-color: var(--color-border-strong);
    color: var(--color-text-secondary);
  }

  .icon-action {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 30px;
    height: 30px;
    border-radius: var(--radius-sm);
    color: var(--color-text-muted);
    font-size: var(--fs-md);
    transition:
      color var(--motion),
      background var(--motion);
  }

  .icon-action:hover {
    color: var(--color-text);
    background: var(--color-surface-2);
  }

  .icon-action.starred {
    color: var(--color-warning);
  }

  kbd {
    padding: 1px 5px;
    border: 1px solid var(--color-border);
    border-radius: 4px;
    background: var(--color-surface-2);
    font-family: var(--font-mono);
    font-size: var(--fs-xs);
  }

  .content {
    flex: 1;
    min-height: 0;
    overflow: auto;
    padding: var(--sp-4);
    display: flex;
  }

  .tool-host {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
  }

  .load-state {
    margin: auto;
    color: var(--color-text-muted);
    font-size: var(--fs-md);
  }

  .load-state.error {
    color: var(--color-danger);
  }

  @media (max-width: 720px) {
    .search-label {
      display: none;
    }

    .crumb-desc {
      display: none;
    }
  }
</style>
