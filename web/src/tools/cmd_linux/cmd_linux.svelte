<script lang="ts">
  import CopyButton from "../../lib/ui/CopyButton.svelte";
  import Panel from "../../lib/ui/Panel.svelte";
  import TextInput from "../../lib/ui/TextInput.svelte";
  import EmptyState from "../../lib/ui/EmptyState.svelte";

  interface RawCommand {
    command: string;
    description: string;
    keywords: string[];
    examples: string[];
    tips: string;
  }

  interface Category {
    name: string;
    icon: string;
    description: string;
    commands: RawCommand[];
  }

  interface Command extends RawCommand {
    category: string;
    categoryIcon: string;
  }

  let categories = $state<Category[]>([]);
  let commands = $state<Command[]>([]);
  let loading = $state(true);
  let error = $state("");

  let search = $state("");
  let activeCategory = $state<string | null>(null);

  async function load() {
    try {
      const response = await fetch("/linux_commands.json");
      if (!response.ok) throw new Error(`HTTP ${response.status}`);
      const data = (await response.json()) as { categories: Category[] };
      categories = data.categories;
      commands = data.categories.flatMap((category) =>
        category.commands.map((command) => ({
          ...command,
          category: category.name,
          categoryIcon: category.icon,
        })),
      );
    } catch {
      error = "无法加载命令数据";
    } finally {
      loading = false;
    }
  }

  load();

  // 空格分隔的多关键词，任一命中即可（OR）
  const visible = $derived.by(() => {
    const term = search.toLowerCase().trim();
    if (term) {
      const terms = term.split(/\s+/).filter(Boolean);
      return commands.filter((command) => {
        const haystack = [
          command.command,
          command.description,
          command.tips,
          ...command.keywords,
          ...command.examples,
        ]
          .join(" ")
          .toLowerCase();
        return terms.some((word) => haystack.includes(word));
      });
    }
    if (activeCategory) return commands.filter((c) => c.category === activeCategory);
    return [];
  });

  const title = $derived(
    search.trim()
      ? `搜索结果（${visible.length}）`
      : activeCategory
        ? `${activeCategory}（${visible.length}）`
        : `共 ${commands.length} 条命令`,
  );
</script>

<div class="tool">
  <TextInput
    bind:value={search}
    type="search"
    placeholder="搜索命令、说明或示例，支持空格分隔多个关键词…"
    ariaLabel="搜索命令"
    oninput={() => {
      if (search.trim()) activeCategory = null;
    }}
  />

  {#if !search.trim()}
    <div class="cats">
      {#each categories as category (category.name)}
        <button
          class="cat"
          class:on={activeCategory === category.name}
          onclick={() => (activeCategory = activeCategory === category.name ? null : category.name)}
          title={category.description}
        >
          <span aria-hidden="true">{category.icon}</span>
          {category.name}
          <span class="num">{category.commands.length}</span>
        </button>
      {/each}
    </div>
  {/if}

  <Panel {title} grow>
    {#if error}
      <p class="error">{error}</p>
    {:else if loading}
      <EmptyState icon="$_" title="加载中…" />
    {:else if visible.length === 0}
      <EmptyState
        icon="$_"
        title={search.trim() ? "没有匹配的命令" : "选择一个分类，或直接搜索"}
        hint={search.trim() ? "换个关键词试试" : `已收录 ${categories.length} 个分类、${commands.length} 条命令`}
      />
    {:else}
      <ul class="list">
        {#each visible as command (command.category + command.command)}
          <li>
            <div class="head">
              <code class="name">{command.command}</code>
              <span class="cat-tag">{command.categoryIcon} {command.category}</span>
              <CopyButton value={command.command} icon title="复制命令名" />
            </div>
            <p class="desc">{command.description}</p>

            {#if command.examples.length > 0}
              <ul class="examples">
                {#each command.examples as example (example)}
                  <li>
                    <code>{example}</code>
                    <CopyButton value={example} icon title="复制示例" />
                  </li>
                {/each}
              </ul>
            {/if}

            {#if command.tips}
              <p class="tips">💡 {command.tips}</p>
            {/if}
          </li>
        {/each}
      </ul>
    {/if}
  </Panel>
</div>

<style>
  .tool {
    display: flex;
    flex-direction: column;
    gap: var(--sp-3);
    flex: 1;
    min-height: 0;
  }

  .cats {
    display: flex;
    flex-wrap: wrap;
    gap: var(--sp-2);
  }

  .cat {
    display: inline-flex;
    align-items: center;
    gap: var(--sp-1);
    padding: 0.25rem var(--sp-3);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-full);
    background: var(--color-surface);
    font-size: var(--fs-sm);
    color: var(--color-text-secondary);
    transition:
      border-color var(--motion),
      color var(--motion);
  }

  .cat:hover {
    border-color: var(--color-border-strong);
    color: var(--color-text);
  }

  .cat.on {
    border-color: var(--color-primary-border);
    background: var(--color-primary-bg);
    color: var(--color-primary);
  }

  .num {
    font-family: var(--font-mono);
    font-size: var(--fs-xs);
    color: var(--color-text-muted);
  }

  .list {
    list-style: none;
    display: flex;
    flex-direction: column;
    gap: var(--sp-5);
  }

  .head {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
  }

  .name {
    font-family: var(--font-mono);
    font-size: var(--fs-lg);
    font-weight: var(--fw-semibold);
    color: var(--color-primary);
  }

  .cat-tag {
    flex: 1;
    font-size: var(--fs-xs);
    color: var(--color-text-muted);
  }

  .desc {
    margin-top: var(--sp-1);
    font-size: var(--fs-md);
    color: var(--color-text-secondary);
  }

  .examples {
    list-style: none;
    display: flex;
    flex-direction: column;
    gap: var(--sp-1);
    margin-top: var(--sp-2);
  }

  .examples li {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    padding: var(--sp-1) var(--sp-2);
    border-radius: var(--radius-sm);
    background: var(--color-code-bg);
  }

  .examples code {
    flex: 1;
    font-family: var(--font-mono);
    font-size: var(--fs-sm);
    color: var(--color-text);
    word-break: break-all;
  }

  .tips {
    margin-top: var(--sp-2);
    padding: var(--sp-2) var(--sp-3);
    border-left: 2px solid var(--color-warning);
    background: var(--color-warning-bg);
    border-radius: 0 var(--radius-sm) var(--radius-sm) 0;
    font-size: var(--fs-sm);
    color: var(--color-text-secondary);
  }

  .error {
    color: var(--color-danger);
    font-size: var(--fs-sm);
  }
</style>
