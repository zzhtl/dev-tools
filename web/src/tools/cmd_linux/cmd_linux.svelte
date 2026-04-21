<script lang="ts">
    interface Command {
        command: string;
        description: string;
        keywords: string[];
        examples: string[];
        tips: string;
        category?: string;
        categoryIcon?: string;
    }

    interface Category {
        name: string;
        icon: string;
        description: string;
        commands: Omit<Command, 'category' | 'categoryIcon'>[];
    }

    let searchTerm = $state("");
    let commands: Command[] = $state([]);
    let categories: Category[] = $state([]);
    let selectedCategory = $state<string | null>(null);
    let isLoading = $state(false);
    let error = $state("");
    let copiedCommand = $state<string | null>(null);
    let viewMode = $state<'categories' | 'search'>('categories');

    // 加载命令数据
    async function loadCommands() {
        try {
            isLoading = true;
            error = "";
            const response = await fetch("/linux_commands.json");
            if (!response.ok) throw new Error('数据加载失败');
            const data = await response.json();
            categories = data.categories;
            commands = data.categories.flatMap((cat: Category) => 
                cat.commands.map(cmd => ({ 
                    ...cmd, 
                    category: cat.name,
                    categoryIcon: cat.icon
                }))
            );
        } catch (err) {
            error = '无法加载命令数据，请稍后重试';
        } finally {
            isLoading = false;
        }
    }

    // 搜索过滤逻辑 - 支持空格分隔的多个关键词
    function getFilteredCommands(): Command[] {
        const rawTerm = searchTerm.toLowerCase().trim();
        if (!rawTerm) return [];
        
        // 将搜索词按空格分割成多个关键词
        const searchTerms = rawTerm.split(/\s+/).filter(t => t.length > 0);
        
        return commands.filter(cmd => {
            // 合并所有可搜索内容
            const searchableContent = [
                cmd.command.toLowerCase(),
                cmd.description.toLowerCase(),
                cmd.tips.toLowerCase(),
                ...cmd.keywords.map(kw => kw.toLowerCase()),
                ...cmd.examples.map(ex => ex.toLowerCase())
            ].join(' ');
            
            // 任一关键词匹配即可（OR逻辑）
            return searchTerms.some(term => searchableContent.includes(term));
        });
    }

    // 获取当前分类的命令
    function getCategoryCommands(): Command[] {
        if (!selectedCategory) return [];
        return commands.filter(cmd => cmd.category === selectedCategory);
    }

    // 复制命令
    async function copyCommand(cmd: string) {
        try {
            await navigator.clipboard.writeText(cmd);
            copiedCommand = cmd;
            setTimeout(() => copiedCommand = null, 1500);
        } catch (err) {
            console.error('复制失败:', err);
        }
    }

    // 处理搜索输入
    function handleSearch() {
        if (searchTerm.trim()) {
            viewMode = 'search';
            selectedCategory = null;
        } else {
            viewMode = 'categories';
        }
    }

    // 选择分类
    function selectCategory(catName: string) {
        selectedCategory = selectedCategory === catName ? null : catName;
        viewMode = 'categories';
        searchTerm = '';
    }

    // 返回分类视图
    function backToCategories() {
        selectedCategory = null;
        searchTerm = '';
        viewMode = 'categories';
    }

    // 初始化加载
    loadCommands();
</script>

<div class="cmd-tool">
    <!-- 搜索栏 -->
    <div class="search-section">
        <div class="search-box">
            <span class="search-icon">🔍</span>
            <input 
                type="text" 
                bind:value={searchTerm}
                oninput={handleSearch}
                placeholder="输入命令名、功能描述或关键词搜索... (如: 磁盘, cpu, 网络, 进程)"
                class="search-input"
            />
            {#if searchTerm}
                <button class="clear-btn" onclick={() => { searchTerm = ''; viewMode = 'categories'; }}>
                    ✕
                </button>
            {/if}
        </div>
        <div class="search-tips">
            💡 提示: 可以输入功能描述如 "查看磁盘空间"、"监控CPU"、"网络连接" 等快速找到对应命令
        </div>
    </div>

    {#if error}
        <div class="error-message">
            ⚠️ {error}
        </div>
    {:else if isLoading}
        <div class="loading">
            <div class="loading-spinner"></div>
            <span>加载命令数据中...</span>
        </div>
    {:else if viewMode === 'search' && searchTerm.trim()}
        <!-- 搜索结果 -->
        <div class="search-results">
            <div class="results-header">
                <button class="back-btn" onclick={backToCategories}>
                    ← 返回分类
                </button>
                <span class="results-count">
                    找到 {getFilteredCommands().length} 个匹配命令
                </span>
            </div>
            
            <div class="commands-list">
                {#each getFilteredCommands() as cmd}
                    <div class="command-card">
                        <div class="card-header">
                            <div class="command-title">
                                <span class="category-badge">
                                    {cmd.categoryIcon} {cmd.category}
                                </span>
                                <code class="command-name">{cmd.command}</code>
                            </div>
                            <button 
                                class="copy-btn"
                                class:copied={copiedCommand === cmd.command}
                                onclick={() => copyCommand(cmd.command)}
                            >
                                {copiedCommand === cmd.command ? '✓ 已复制' : '📋 复制'}
                            </button>
                        </div>
                        <p class="command-desc">{cmd.description}</p>
                        
                        {#if cmd.examples.length > 0}
                            <div class="examples-section">
                                <span class="section-label">使用示例</span>
                                <div class="examples-list">
                                    {#each cmd.examples as example}
                                        <div class="example-item">
                                            <code>{example}</code>
                                            <button 
                                                class="copy-example-btn"
                                                onclick={() => copyCommand(example)}
                                            >
                                                {copiedCommand === example ? '✓' : '📋'}
                                            </button>
                                        </div>
                                    {/each}
                                </div>
                            </div>
                        {/if}
                        
                        {#if cmd.tips}
                            <div class="tips-section">
                                <span class="tip-icon">💡</span>
                                <span class="tip-text">{cmd.tips}</span>
                            </div>
                        {/if}
                        
                        <div class="keywords">
                            {#each cmd.keywords as kw}
                                <span class="keyword-tag">{kw}</span>
                            {/each}
                        </div>
                    </div>
                {:else}
                    <div class="empty-state">
                        <span class="empty-icon">🔍</span>
                        <p>未找到匹配 "{searchTerm}" 的命令</p>
                        <p class="empty-hint">尝试使用其他关键词，如: 内存、磁盘、网络、进程等</p>
                    </div>
                {/each}
            </div>
        </div>
    {:else if selectedCategory}
        <!-- 分类命令列表 -->
        <div class="category-detail">
            <div class="detail-header">
                <button class="back-btn" onclick={backToCategories}>
                    ← 返回分类
                </button>
                <h2 class="category-title">
                    {categories.find(c => c.name === selectedCategory)?.icon} {selectedCategory}
                </h2>
            </div>
            <p class="category-desc">
                {categories.find(c => c.name === selectedCategory)?.description}
            </p>
            
            <div class="commands-list">
                {#each getCategoryCommands() as cmd}
                    <div class="command-card">
                        <div class="card-header">
                            <code class="command-name">{cmd.command}</code>
                            <button 
                                class="copy-btn"
                                class:copied={copiedCommand === cmd.command}
                                onclick={() => copyCommand(cmd.command)}
                            >
                                {copiedCommand === cmd.command ? '✓ 已复制' : '📋 复制'}
                            </button>
                        </div>
                        <p class="command-desc">{cmd.description}</p>
                        
                        {#if cmd.examples.length > 0}
                            <div class="examples-section">
                                <span class="section-label">使用示例</span>
                                <div class="examples-list">
                                    {#each cmd.examples as example}
                                        <div class="example-item">
                                            <code>{example}</code>
                                            <button 
                                                class="copy-example-btn"
                                                onclick={() => copyCommand(example)}
                                            >
                                                {copiedCommand === example ? '✓' : '📋'}
                                            </button>
                                        </div>
                                    {/each}
                                </div>
                            </div>
                        {/if}
                        
                        {#if cmd.tips}
                            <div class="tips-section">
                                <span class="tip-icon">💡</span>
                                <span class="tip-text">{cmd.tips}</span>
                            </div>
                        {/if}
                    </div>
                {/each}
            </div>
        </div>
    {:else}
        <!-- 分类网格 -->
        <div class="categories-section">
            <h2 class="section-title">📂 按分类浏览命令</h2>
            <div class="categories-grid">
                {#each categories as cat}
                    <button 
                        class="category-card"
                        onclick={() => selectCategory(cat.name)}
                    >
                        <span class="cat-icon">{cat.icon}</span>
                        <span class="cat-name">{cat.name}</span>
                        <span class="cat-count">{cat.commands.length} 个命令</span>
                        <p class="cat-desc">{cat.description}</p>
                    </button>
                {/each}
            </div>
        </div>

        <!-- 常用场景快捷入口 -->
        <div class="quick-scenarios">
            <h2 class="section-title">🚀 常用场景</h2>
            <div class="scenarios-grid">
                <button class="scenario-btn" onclick={() => { searchTerm = 'cpu 使用率'; handleSearch(); }}>
                    🔧 查看 CPU 使用率
                </button>
                <button class="scenario-btn" onclick={() => { searchTerm = '内存'; handleSearch(); }}>
                    💾 查看内存使用
                </button>
                <button class="scenario-btn" onclick={() => { searchTerm = '磁盘 空间'; handleSearch(); }}>
                    💿 查看磁盘空间
                </button>
                <button class="scenario-btn" onclick={() => { searchTerm = 'io 磁盘'; handleSearch(); }}>
                    📊 监控磁盘 IO
                </button>
                <button class="scenario-btn" onclick={() => { searchTerm = '网络 流量'; handleSearch(); }}>
                    🌐 监控网络流量
                </button>
                <button class="scenario-btn" onclick={() => { searchTerm = '端口 占用'; handleSearch(); }}>
                    🔌 查看端口占用
                </button>
                <button class="scenario-btn" onclick={() => { searchTerm = '进程 终止'; handleSearch(); }}>
                    ⚙️ 终止进程
                </button>
                <button class="scenario-btn" onclick={() => { searchTerm = '日志 实时'; handleSearch(); }}>
                    📋 实时查看日志
                </button>
                <button class="scenario-btn" onclick={() => { searchTerm = '硬件 信息'; handleSearch(); }}>
                    🖥️ 查看硬件信息
                </button>
                <button class="scenario-btn" onclick={() => { searchTerm = '服务 启动'; handleSearch(); }}>
                    🔄 管理服务
                </button>
                <button class="scenario-btn" onclick={() => { searchTerm = '压缩 打包'; handleSearch(); }}>
                    📦 压缩打包文件
                </button>
                <button class="scenario-btn" onclick={() => { searchTerm = '查找 文件'; handleSearch(); }}>
                    📁 查找文件
                </button>
            </div>
        </div>
    {/if}
</div>

<style>
    .cmd-tool {
        display: flex;
        flex-direction: column;
        gap: 1.5rem;
    }

    /* 搜索区域 */
    .search-section {
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
    }

    .search-box {
        display: flex;
        align-items: center;
        background: var(--bg-dark);
        border: 2px solid var(--border);
        border-radius: var(--radius-sm);
        padding: 0 1rem;
        transition: border-color 0.2s;
    }

    .search-box:focus-within {
        border-color: var(--primary);
    }

    .search-icon {
        font-size: 1.2rem;
        margin-right: 0.75rem;
    }

    .search-input {
        flex: 1;
        border: none !important;
        background: transparent !important;
        padding: 1rem 0 !important;
        font-size: 1rem;
    }

    .search-input:focus {
        outline: none;
    }

    .clear-btn {
        padding: 0.25rem 0.5rem !important;
        background: var(--bg-hover) !important;
        border: none !important;
        font-size: 0.9rem;
        color: var(--text-muted) !important;
    }

    .clear-btn:hover {
        color: var(--text-primary) !important;
        transform: none !important;
        box-shadow: none !important;
    }

    .search-tips {
        font-size: 0.85rem;
        color: var(--text-muted);
        padding-left: 0.5rem;
    }

    /* 分类网格 */
    .categories-section, .quick-scenarios {
        background: var(--bg-dark);
        border: 1px solid var(--border);
        border-radius: var(--radius-sm);
        padding: 1.5rem;
    }

    .section-title {
        font-size: 1rem;
        font-weight: 600;
        margin-bottom: 1rem;
        color: var(--text-primary);
    }

    .categories-grid {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
        gap: 1rem;
    }

    .category-card {
        display: flex;
        flex-direction: column;
        align-items: flex-start;
        padding: 1rem !important;
        background: var(--bg-card) !important;
        border: 1px solid var(--border) !important;
        border-radius: var(--radius-sm) !important;
        text-align: left;
        transition: all 0.2s;
        cursor: pointer;
    }

    .category-card:hover {
        border-color: var(--primary) !important;
        transform: translateY(-2px) !important;
        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15) !important;
    }

    .cat-icon {
        font-size: 2rem;
        margin-bottom: 0.5rem;
    }

    .cat-name {
        font-weight: 600;
        font-size: 1rem;
        color: var(--text-primary);
    }

    .cat-count {
        font-size: 0.8rem;
        color: var(--text-muted);
        margin-top: 0.25rem;
    }

    .cat-desc {
        font-size: 0.8rem;
        color: var(--text-secondary);
        margin-top: 0.5rem;
        line-height: 1.4;
    }

    /* 常用场景 */
    .scenarios-grid {
        display: flex;
        flex-wrap: wrap;
        gap: 0.75rem;
    }

    .scenario-btn {
        padding: 0.6rem 1rem !important;
        background: var(--bg-card) !important;
        border: 1px solid var(--border) !important;
        border-radius: 20px !important;
        font-size: 0.9rem;
        color: var(--text-primary) !important;
        transition: all 0.2s;
    }

    .scenario-btn:hover {
        border-color: var(--primary) !important;
        background: var(--bg-hover) !important;
        transform: none !important;
        box-shadow: none !important;
    }

    /* 搜索结果 & 分类详情 */
    .search-results, .category-detail {
        display: flex;
        flex-direction: column;
        gap: 1rem;
    }

    .results-header, .detail-header {
        display: flex;
        align-items: center;
        gap: 1rem;
    }

    .back-btn {
        padding: 0.5rem 1rem !important;
        background: var(--bg-card) !important;
        border: 1px solid var(--border) !important;
        font-size: 0.9rem;
    }

    .back-btn:hover {
        border-color: var(--primary) !important;
        transform: none !important;
        box-shadow: none !important;
    }

    .results-count {
        font-size: 0.9rem;
        color: var(--text-secondary);
    }

    .category-title {
        font-size: 1.25rem;
        font-weight: 600;
        margin: 0;
    }

    .category-desc {
        font-size: 0.9rem;
        color: var(--text-secondary);
        margin: 0;
    }

    /* 命令卡片 */
    .commands-list {
        display: flex;
        flex-direction: column;
        gap: 1rem;
    }

    .command-card {
        background: var(--bg-dark);
        border: 1px solid var(--border);
        border-radius: var(--radius-sm);
        padding: 1.25rem;
        display: flex;
        flex-direction: column;
        gap: 0.75rem;
    }

    .card-header {
        display: flex;
        justify-content: space-between;
        align-items: flex-start;
        gap: 1rem;
    }

    .command-title {
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
    }

    .category-badge {
        font-size: 0.75rem;
        color: var(--text-muted);
    }

    .command-name {
        font-family: 'JetBrains Mono', monospace;
        font-size: 1.15rem;
        font-weight: 600;
        color: var(--accent) !important;
        background: transparent !important;
        padding: 0 !important;
    }

    .copy-btn {
        padding: 0.4rem 0.85rem !important;
        font-size: 0.85rem !important;
        background: var(--bg-card) !important;
        border: 1px solid var(--border) !important;
        white-space: nowrap;
    }

    .copy-btn.copied {
        background: var(--accent-green) !important;
        border-color: var(--accent-green) !important;
        color: white !important;
    }

    .copy-btn:hover:not(.copied) {
        border-color: var(--primary) !important;
        transform: none !important;
        box-shadow: none !important;
    }

    .command-desc {
        font-size: 0.95rem;
        color: var(--text-secondary);
        margin: 0;
        line-height: 1.5;
    }

    /* 示例区域 */
    .examples-section {
        background: var(--bg-card);
        border-radius: var(--radius-sm);
        padding: 0.75rem;
    }

    .section-label {
        font-size: 0.8rem;
        color: var(--text-muted);
        font-weight: 500;
        display: block;
        margin-bottom: 0.5rem;
    }

    .examples-list {
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
    }

    .example-item {
        display: flex;
        align-items: center;
        justify-content: space-between;
        background: var(--bg-dark);
        border-radius: 4px;
        padding: 0.5rem 0.75rem;
    }

    .example-item code {
        font-family: 'JetBrains Mono', monospace;
        font-size: 0.9rem;
        color: var(--text-primary);
        background: transparent !important;
        padding: 0 !important;
    }

    .copy-example-btn {
        padding: 0.2rem 0.4rem !important;
        background: transparent !important;
        border: none !important;
        font-size: 0.8rem;
        opacity: 0.6;
    }

    .copy-example-btn:hover {
        opacity: 1;
        transform: none !important;
        box-shadow: none !important;
    }

    /* 提示 */
    .tips-section {
        display: flex;
        align-items: flex-start;
        gap: 0.5rem;
        padding: 0.75rem;
        background: rgba(16, 185, 129, 0.1);
        border-radius: var(--radius-sm);
        border-left: 3px solid var(--accent-green);
    }

    .tip-icon {
        flex-shrink: 0;
    }

    .tip-text {
        font-size: 0.85rem;
        color: var(--text-secondary);
        line-height: 1.4;
    }

    /* 关键词标签 */
    .keywords {
        display: flex;
        flex-wrap: wrap;
        gap: 0.5rem;
        margin-top: 0.25rem;
    }

    .keyword-tag {
        padding: 0.2rem 0.5rem;
        background: var(--bg-card);
        border: 1px solid var(--border);
        border-radius: 4px;
        font-size: 0.75rem;
        color: var(--text-muted);
    }

    /* 空状态 */
    .empty-state {
        text-align: center;
        padding: 3rem 2rem;
        background: var(--bg-dark);
        border: 1px solid var(--border);
        border-radius: var(--radius-sm);
    }

    .empty-icon {
        font-size: 3rem;
        display: block;
        margin-bottom: 1rem;
    }

    .empty-state p {
        margin: 0.5rem 0;
        color: var(--text-secondary);
    }

    .empty-hint {
        font-size: 0.85rem;
        color: var(--text-muted) !important;
    }

    /* 加载 & 错误 */
    .loading {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 1rem;
        padding: 3rem;
        color: var(--text-muted);
    }

    .loading-spinner {
        width: 40px;
        height: 40px;
        border: 3px solid var(--border);
        border-top-color: var(--primary);
        border-radius: 50%;
        animation: spin 1s linear infinite;
    }

    @keyframes spin {
        to { transform: rotate(360deg); }
    }

    .error-message {
        text-align: center;
        padding: 2rem;
        background: rgba(239, 68, 68, 0.1);
        border: 1px solid rgba(239, 68, 68, 0.3);
        border-radius: var(--radius-sm);
        color: #f87171;
    }
</style>
