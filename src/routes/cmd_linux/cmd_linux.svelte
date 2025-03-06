<script lang="ts">
  let searchTerm = "";
  let commands: any[] = [];
  let selectedCategory = "all";
  let isLoading = false;
  let error = "";

  // 加载命令数据
  async function loadCommands() {
    try {
      isLoading = true;
      error = "";
      const response = await fetch("/linux_commands.json");
      if (!response.ok) throw new Error('数据加载失败');
      const data = await response.json();
      commands = data.categories.flatMap((cat: any) => 
        cat.commands.map((cmd: any) => ({ ...cmd, category: cat.name }))
      );
    } catch (err) {
      error = '无法加载命令数据，请稍后重试';
    } finally {
      isLoading = false;
    }
  }

  // 模糊搜索逻辑（支持中英文混合搜索）
  $: filteredCommands = commands.filter(cmd => {
    const term = searchTerm.toLowerCase().trim();
    if (!term) return true;
    
    const matchCommand = cmd.command.toLowerCase().includes(term);
    const matchDescription = cmd.description.toLowerCase().includes(term);
    const categoryMatch = selectedCategory === "all" || cmd.category === selectedCategory;
    
    // 支持拼音首字母搜索（如输入"ml"匹配"目录"）
    const pinyinMatch = cmd.pinyin?.some((py: string) => py.startsWith(term));
    
    return (matchCommand || matchDescription || pinyinMatch) && categoryMatch;
  });

  // 初始化加载
  loadCommands();
</script>

<div class="container">
  <h1>Linux命令查询工具</h1>
  <div class="search-box">
    <input 
      type="text" 
      bind:value={searchTerm}
      placeholder="输入命令或描述（支持中文/拼音）..."
      class="search-input"
    />
    
    <select bind:value={selectedCategory} class="category-filter">
      <option value="all">所有分类</option>
      {#each Array.from(new Set(commands.map(c => c.category))) as category}
        <option value={category}>{category}</option>
      {/each}
    </select>
  </div>

  {#if error}
    <div class="error-message">{error}</div>
  {:else if isLoading}
    <div class="loading">加载中...</div>
  {:else}
    <div class="results">
      {#each filteredCommands as cmd}
        <div class="command-card">
          <div class="header">
            <span class="command">{cmd.command}</span>
            <span class="category">{cmd.category}</span>
          </div>
          <div class="description">{cmd.description}</div>
          {#if cmd.examples?.length}
            <div class="examples">
              <div class="example-title">使用示例：</div>
              {#each cmd.examples as ex}
                <code>{ex}</code>
              {/each}
            </div>
          {/if}
          <button on:click={() => navigator.clipboard.writeText(cmd.command)}>
            📋 复制命令
          </button>
        </div>
      {:else}
        <div class="empty-state">未找到匹配命令</div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .container {
    padding: 2rem;
    max-width: 1000px;
    margin: 0 auto;
    font-family: system-ui, -apple-system, sans-serif;
  }

  h1 {
    text-align: center;
    color: #2c3e50;
    margin-bottom: 2rem;
  }

  .search-box {
    display: grid;
    grid-template-columns: 1fr auto;
    gap: 1rem;
    margin-bottom: 2rem;
  }

  .search-input {
    padding: 1rem;
    border: 2px solid #42b983;
    border-radius: 8px;
    font-size: 1rem;
    transition: border-color 0.3s;
  }

  .search-input:focus {
    outline: none;
    border-color: #2c3e50;
  }

  .category-filter {
    padding: 1rem;
    border-radius: 8px;
    border: 2px solid #42b983;
    background: white;
    font-size: 1rem;
  }

  .command-card {
    background: white;
    border-radius: 12px;
    padding: 1.5rem;
    margin-bottom: 1rem;
    box-shadow: 0 3px 6px rgba(0,0,0,0.1);
    transition: transform 0.2s;
  }

  .command-card:hover {
    transform: translateY(-2px);
  }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1rem;
  }

  .command {
    font-weight: 600;
    color: #2c3e50;
    font-size: 1.2rem;
  }

  .category {
    background: #42b983;
    color: white;
    padding: 0.3rem 0.8rem;
    border-radius: 20px;
    font-size: 0.9rem;
  }

  .description {
    color: #666;
    margin-bottom: 1rem;
    line-height: 1.6;
  }

  .examples {
    margin: 1.5rem 0;
    padding-left: 1rem;
    border-left: 3px solid #42b983;
  }

  .example-title {
    font-weight: 500;
    margin-bottom: 0.5rem;
    color: #444;
  }

  code {
    display: block;
    background: #f8f9fa;
    padding: 0.8rem;
    border-radius: 6px;
    margin: 0.5rem 0;
    font-family: 'Fira Code', monospace;
    font-size: 0.9rem;
    white-space: pre-wrap;
  }

  button {
    background: #42b983;
    color: white;
    border: none;
    padding: 0.8rem 1.5rem;
    border-radius: 6px;
    cursor: pointer;
    transition: background 0.3s;
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin-top: 1rem;
  }

  button:hover {
    background: #3aa876;
  }

  .empty-state, .loading, .error-message {
    text-align: center;
    padding: 3rem;
    font-size: 1.1rem;
    color: #666;
    border: 2px dashed #eee;
    border-radius: 12px;
    margin: 2rem 0;
  }

  .error-message {
    color: #e74c3c;
    border-color: #e74c3c;
  }
</style>