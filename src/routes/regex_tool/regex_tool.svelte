<script lang="ts">
    interface MatchResult {
        fullMatch: string;
        groups: string[];
        index: number;
        lastIndex: number;
    }

    interface RegexPattern {
        name: string;
        pattern: string;
        description: string;
        example: string;
        category: string;
    }

    let regexPattern = $state("");
    let testString = $state("");
    let replaceString = $state("");
    let matchResults = $state<MatchResult[]>([]);
    let errorMessage = $state("");
    let regexFlags = $state("g");
    let highlightedText = $state("");
    let replacedText = $state("");
    let isValidRegex = $state(true);
    let showPatternDropdown = $state(false);
    let activeMode = $state<'match' | 'replace'>('match');
    let selectedCategory = $state<string>('all');
    let copiedPattern = $state<string | null>(null);

    // 常用正则表达式集合（按分类整理）
    const commonPatterns: RegexPattern[] = [
        // 基础验证
        { name: "邮箱地址", pattern: "\\w+([.-]?\\w+)*@\\w+([.-]?\\w+)*\\.\\w{2,}", description: "标准邮箱格式", example: "test@example.com", category: "验证" },
        { name: "手机号码", pattern: "^1[3-9]\\d{9}$", description: "中国大陆手机号", example: "13812345678", category: "验证" },
        { name: "身份证号", pattern: "(^\\d{15}$)|(^\\d{17}(\\d|X|x)$)", description: "15/18位身份证", example: "110101199001011234", category: "验证" },
        { name: "URL地址", pattern: "https?:\\/\\/(www\\.)?[-a-zA-Z0-9@:%._\\+~#=]{1,256}\\.[a-zA-Z0-9()]{1,6}\\b([-a-zA-Z0-9()@:%_\\+.~#?&//=]*)", description: "HTTP/HTTPS URL", example: "https://example.com", category: "验证" },
        { name: "IPV4地址", pattern: "((25[0-5]|2[0-4]\\d|1\\d{2}|[1-9]?\\d)\\.){3}(25[0-5]|2[0-4]\\d|1\\d{2}|[1-9]?\\d)", description: "IPv4格式", example: "192.168.1.1", category: "验证" },
        { name: "车牌号", pattern: "[京津沪渝冀豫云辽黑湘皖鲁新苏浙赣鄂桂甘晋蒙陕吉闽贵粤青藏川宁琼使领][A-Z][A-HJ-NP-Z0-9]{4}[A-HJ-NP-Z0-9挂学警港澳]", description: "中国车牌号", example: "京A12345", category: "验证" },

        // 日期时间
        { name: "日期 YYYY-MM-DD", pattern: "\\d{4}-(0[1-9]|1[0-2])-(0[1-9]|[12]\\d|3[01])", description: "ISO日期格式", example: "2024-01-15", category: "日期" },
        { name: "日期 DD/MM/YYYY", pattern: "(0[1-9]|[12]\\d|3[01])\\/(0[1-9]|1[0-2])\\/\\d{4}", description: "欧洲日期格式", example: "15/01/2024", category: "日期" },
        { name: "时间 HH:MM:SS", pattern: "([01]\\d|2[0-3]):[0-5]\\d:[0-5]\\d", description: "24小时制时间", example: "14:30:00", category: "日期" },
        { name: "时间 12小时制", pattern: "(0?[1-9]|1[0-2]):[0-5]\\d\\s?(AM|PM|am|pm)", description: "12小时制时间", example: "02:30 PM", category: "日期" },

        // 数字
        { name: "正整数", pattern: "^[1-9]\\d*$", description: "大于0的整数", example: "123", category: "数字" },
        { name: "负整数", pattern: "^-[1-9]\\d*$", description: "小于0的整数", example: "-123", category: "数字" },
        { name: "小数", pattern: "^-?\\d+\\.\\d+$", description: "正负小数", example: "3.14", category: "数字" },
        { name: "货币金额", pattern: "^\\d{1,3}(,\\d{3})*(\\.\\d{2})?$", description: "带千位分隔符", example: "1,234.56", category: "数字" },
        { name: "十六进制", pattern: "^0x[0-9A-Fa-f]+$", description: "十六进制数", example: "0x1A2B", category: "数字" },

        // 文本
        { name: "中文字符", pattern: "[\\u4e00-\\u9fa5]", description: "匹配中文", example: "中文", category: "文本" },
        { name: "英文单词", pattern: "\\b[a-zA-Z]+\\b", description: "纯英文单词", example: "Hello", category: "文本" },
        { name: "双字节字符", pattern: "[^\\x00-\\xff]", description: "包含中日韩等", example: "中文日本語", category: "文本" },
        { name: "空白行", pattern: "^\\s*$", description: "空行或纯空白", example: "   ", category: "文本" },
        { name: "HTML标签", pattern: "<[^>]+>", description: "任意HTML标签", example: "<div>", category: "文本" },
        { name: "HTML注释", pattern: "<!--[\\s\\S]*?-->", description: "HTML注释", example: "<!-- comment -->", category: "文本" },

        // 安全
        { name: "强密码", pattern: "^(?=.*[a-z])(?=.*[A-Z])(?=.*\\d)(?=.*[@$!%*?&])[A-Za-z\\d@$!%*?&]{8,}$", description: "大小写+数字+特殊字符", example: "Pass@123", category: "安全" },
        { name: "MD5哈希", pattern: "^[a-fA-F0-9]{32}$", description: "32位MD5", example: "d41d8cd98f00b204e9800998ecf8427e", category: "安全" },
        { name: "SHA256哈希", pattern: "^[a-fA-F0-9]{64}$", description: "64位SHA256", example: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855", category: "安全" },
    ];

    // 获取所有分类
    const categories = ['all', ...new Set(commonPatterns.map(p => p.category))];

    // 过滤后的模式
    function getFilteredPatterns(): RegexPattern[] {
        if (selectedCategory === 'all') return commonPatterns;
        return commonPatterns.filter(p => p.category === selectedCategory);
    }

    function handleRegexChange() {
        validateRegex();
        if (isValidRegex) {
            testRegex();
            if (activeMode === 'replace') {
                performReplace();
            }
        }
    }

    function validateRegex() {
        if (!regexPattern) {
            isValidRegex = true;
            errorMessage = "";
            return;
        }

        try {
            new RegExp(regexPattern, regexFlags);
            isValidRegex = true;
            errorMessage = "";
        } catch (e: unknown) {
            isValidRegex = false;
            errorMessage = `无效的正则表达式: ${(e as Error).message}`;
        }
    }

    function testRegex() {
        if (!regexPattern || !testString) {
            matchResults = [];
            highlightedText = testString;
            return;
        }

        try {
            const regex = new RegExp(regexPattern, regexFlags);
            const matches: MatchResult[] = [];
            let match;

            // 收集所有匹配结果
            if (regexFlags.includes('g')) {
                while ((match = regex.exec(testString)) !== null) {
                    matches.push({
                        fullMatch: match[0],
                        groups: match.slice(1),
                        index: match.index,
                        lastIndex: regex.lastIndex
                    });
                }
            } else {
                match = regex.exec(testString);
                if (match) {
                    matches.push({
                        fullMatch: match[0],
                        groups: match.slice(1),
                        index: match.index,
                        lastIndex: match.index + match[0].length
                    });
                }
            }

            matchResults = matches;

            // 生成高亮文本
            highlightedText = generateHighlightedText(testString, matches);

        } catch (e: unknown) {
            errorMessage = `执行正则表达式时出错: ${(e as Error).message}`;
            matchResults = [];
            highlightedText = testString;
        }
    }

    function performReplace() {
        if (!regexPattern || !testString) {
            replacedText = "";
            return;
        }

        try {
            const regex = new RegExp(regexPattern, regexFlags);
            replacedText = testString.replace(regex, replaceString);
        } catch (e: unknown) {
            replacedText = "";
        }
    }

    function generateHighlightedText(text: string, matches: MatchResult[]): string {
        if (!matches.length) return escapeHtml(text);

        const sortedMatches = [...matches].sort((a, b) => a.index - b.index);

        let result = '';
        let lastIndex = 0;

        for (const match of sortedMatches) {
            // 添加匹配前的文本
            result += escapeHtml(text.substring(lastIndex, match.index));

            // 添加高亮的匹配文本
            result += `<mark class="match-highlight">${escapeHtml(text.substring(match.index, match.lastIndex))}</mark>`;

            lastIndex = match.lastIndex;
        }

        // 添加最后一个匹配后的文本
        result += escapeHtml(text.substring(lastIndex));

        return result;
    }

    function escapeHtml(text: string): string {
        return text
            .replace(/&/g, '&amp;')
            .replace(/</g, '&lt;')
            .replace(/>/g, '&gt;')
            .replace(/"/g, '&quot;')
            .replace(/'/g, '&#039;');
    }

    function toggleFlag(flag: string) {
        if (regexFlags.includes(flag)) {
            regexFlags = regexFlags.replace(flag, '');
        } else {
            regexFlags += flag;
        }

        validateRegex();
        if (isValidRegex) {
            testRegex();
            if (activeMode === 'replace') {
                performReplace();
            }
        }
    }

    function selectPattern(pattern: RegexPattern) {
        regexPattern = pattern.pattern;
        testString = pattern.example;
        showPatternDropdown = false;
        handleRegexChange();
    }

    function copyToClipboard(text: string, type: string) {
        navigator.clipboard.writeText(text).then(() => {
            copiedPattern = type;
            setTimeout(() => copiedPattern = null, 2000);
        });
    }

    function clearAll() {
        regexPattern = "";
        testString = "";
        replaceString = "";
        matchResults = [];
        highlightedText = "";
        replacedText = "";
        errorMessage = "";
    }

    // 正则表达式语法参考
    const syntaxReference = [
        { category: "字符类", items: [
            { pattern: ".", desc: "任意字符（除换行符）" },
            { pattern: "\\w", desc: "字母、数字、下划线" },
            { pattern: "\\W", desc: "非字母数字下划线" },
            { pattern: "\\d", desc: "数字 [0-9]" },
            { pattern: "\\D", desc: "非数字" },
            { pattern: "\\s", desc: "空白字符" },
            { pattern: "\\S", desc: "非空白字符" },
        ]},
        { category: "边界", items: [
            { pattern: "^", desc: "行首" },
            { pattern: "$", desc: "行尾" },
            { pattern: "\\b", desc: "单词边界" },
            { pattern: "\\B", desc: "非单词边界" },
        ]},
        { category: "量词", items: [
            { pattern: "*", desc: "0 或更多" },
            { pattern: "+", desc: "1 或更多" },
            { pattern: "?", desc: "0 或 1" },
            { pattern: "{n}", desc: "恰好 n 次" },
            { pattern: "{n,}", desc: "至少 n 次" },
            { pattern: "{n,m}", desc: "n 到 m 次" },
        ]},
        { category: "分组", items: [
            { pattern: "(abc)", desc: "捕获组" },
            { pattern: "(?:abc)", desc: "非捕获组" },
            { pattern: "a|b", desc: "a 或 b" },
            { pattern: "[abc]", desc: "字符集" },
            { pattern: "[^abc]", desc: "排除字符集" },
            { pattern: "[a-z]", desc: "字符范围" },
        ]},
    ];
</script>

<div class="regex-tool">
    <!-- 标题区域 -->
    <div class="tool-header">
        <div class="header-content">
            <h2>🔍 正则表达式工具</h2>
            <p class="header-desc">在线测试、验证和替换，支持实时高亮匹配</p>
        </div>
        <button class="clear-btn" onclick={clearAll}>
            <span>🗑️</span> 清空
        </button>
    </div>

    <!-- 主编辑区域 -->
    <div class="editor-layout">
        <!-- 左侧：输入区域 -->
        <div class="input-panel">
            <!-- 正则表达式输入 -->
            <div class="input-section">
                <div class="section-header">
                    <label for="regex-input">正则表达式</label>
                    <div class="header-actions">
                        <div class="pattern-selector">
                            <button 
                                class="pattern-btn"
                                onclick={() => showPatternDropdown = !showPatternDropdown}
                            >
                                📚 常用模板
                                <span class="dropdown-arrow">{showPatternDropdown ? '▲' : '▼'}</span>
                            </button>
                            
                            {#if showPatternDropdown}
                                <div class="pattern-dropdown">
                                    <div class="category-tabs">
                                        {#each categories as cat}
                                            <button 
                                                class="cat-tab"
                                                class:active={selectedCategory === cat}
                                                onclick={() => selectedCategory = cat}
                                            >
                                                {cat === 'all' ? '全部' : cat}
                                            </button>
                                        {/each}
                                    </div>
                                    <div class="pattern-list">
                                        {#each getFilteredPatterns() as pattern}
                                            <button class="pattern-item" onclick={() => selectPattern(pattern)}>
                                                <div class="pattern-info">
                                                    <span class="pattern-name">{pattern.name}</span>
                                                    <span class="pattern-desc">{pattern.description}</span>
                                                </div>
                                                <code class="pattern-code">{pattern.pattern.length > 30 ? pattern.pattern.substring(0, 30) + '...' : pattern.pattern}</code>
                                            </button>
                                        {/each}
                                    </div>
                                </div>
                            {/if}
                        </div>
                    </div>
                </div>
                
                <div class="regex-input-wrapper" class:error={!isValidRegex}>
                    <span class="regex-delimiter">/</span>
                    <input 
                        id="regex-input"
                        type="text" 
                        bind:value={regexPattern}
                        oninput={handleRegexChange}
                        placeholder="输入正则表达式..."
                        spellcheck="false"
                    />
                    <span class="regex-delimiter">/</span>
                    <div class="flags-container">
                        {#each [
                            { flag: 'g', title: '全局匹配', desc: 'global' },
                            { flag: 'i', title: '忽略大小写', desc: 'insensitive' },
                            { flag: 'm', title: '多行模式', desc: 'multiline' },
                            { flag: 's', title: '点号匹配换行', desc: 'dotAll' },
                        ] as f}
                            <button 
                                class="flag-btn"
                                class:active={regexFlags.includes(f.flag)}
                                onclick={() => toggleFlag(f.flag)}
                                title={f.title}
                            >
                                {f.flag}
                            </button>
                        {/each}
                    </div>
                </div>

                {#if regexPattern && isValidRegex}
                    <div class="regex-actions">
                        <button class="action-btn" onclick={() => copyToClipboard(regexPattern, 'pattern')}>
                            {copiedPattern === 'pattern' ? '✓ 已复制' : '📋 复制表达式'}
                        </button>
                        <button class="action-btn" onclick={() => copyToClipboard(`/${regexPattern}/${regexFlags}`, 'full')}>
                            {copiedPattern === 'full' ? '✓ 已复制' : '📋 复制完整'}
                        </button>
                    </div>
                {/if}

                {#if errorMessage}
                    <div class="error-message">
                        <span class="error-icon">⚠️</span>
                        {errorMessage}
                    </div>
                {/if}
            </div>

            <!-- 模式切换 -->
            <div class="mode-tabs">
                <button 
                    class="mode-tab"
                    class:active={activeMode === 'match'}
                    onclick={() => activeMode = 'match'}
                >
                    🎯 匹配模式
                </button>
                <button 
                    class="mode-tab"
                    class:active={activeMode === 'replace'}
                    onclick={() => { activeMode = 'replace'; performReplace(); }}
                >
                    🔄 替换模式
                </button>
            </div>

            <!-- 测试文本 -->
            <div class="input-section">
                <label for="test-input">测试文本</label>
                <textarea 
                    id="test-input"
                    bind:value={testString}
                    oninput={() => { testRegex(); if (activeMode === 'replace') performReplace(); }}
                    placeholder="输入需要测试的文本..."
                    spellcheck="false"
                ></textarea>
            </div>

            <!-- 替换输入（替换模式） -->
            {#if activeMode === 'replace'}
                <div class="input-section">
                    <div class="section-header">
                        <label for="replace-input">替换为</label>
                        <span class="hint-badge">支持 $1, $2 捕获组引用</span>
                    </div>
                    <input 
                        id="replace-input"
                        type="text" 
                        bind:value={replaceString}
                        oninput={performReplace}
                        placeholder="替换文本..."
                        spellcheck="false"
                    />
                </div>
            {/if}
        </div>

        <!-- 右侧：结果区域 -->
        <div class="result-panel">
            <!-- 高亮预览 -->
            <div class="result-section">
                <div class="section-header">
                    <span class="section-title">
                        {activeMode === 'match' ? '🎨 匹配高亮' : '🔄 替换预览'}
                    </span>
                    {#if matchResults.length > 0}
                        <span class="match-count">{matchResults.length} 个匹配</span>
                    {/if}
                </div>
                
                <div class="preview-box">
                    {#if activeMode === 'match'}
                        {#if testString}
                            <div class="highlighted-content">
                                {@html highlightedText}
                            </div>
                        {:else}
                            <div class="empty-hint">输入测试文本查看匹配结果</div>
                        {/if}
                    {:else}
                        {#if replacedText}
                            <div class="replaced-content">{replacedText}</div>
                        {:else}
                            <div class="empty-hint">输入替换文本查看结果</div>
                        {/if}
                    {/if}
                </div>

                {#if activeMode === 'replace' && replacedText}
                    <button class="copy-result-btn" onclick={() => copyToClipboard(replacedText, 'result')}>
                        {copiedPattern === 'result' ? '✓ 已复制' : '📋 复制结果'}
                    </button>
                {/if}
            </div>

            <!-- 匹配详情 -->
            {#if activeMode === 'match' && matchResults.length > 0}
                <div class="matches-section">
                    <div class="section-header">
                        <span class="section-title">📊 匹配详情</span>
                    </div>
                    
                    <div class="matches-list">
                        {#each matchResults as match, index}
                            <div class="match-card">
                                <div class="match-header">
                                    <span class="match-number">#{index + 1}</span>
                                    <span class="match-position">位置: {match.index}-{match.lastIndex}</span>
                                </div>
                                <div class="match-body">
                                    <div class="match-row">
                                        <span class="match-label">匹配:</span>
                                        <code class="match-value">"{match.fullMatch}"</code>
                                    </div>
                                    {#if match.groups.length > 0}
                                        <div class="groups-section">
                                            <span class="match-label">捕获组:</span>
                                            <div class="groups-list">
                                                {#each match.groups as group, groupIndex}
                                                    <div class="group-item">
                                                        <span class="group-index">${groupIndex + 1}</span>
                                                        <code class="group-value">"{group || ''}"</code>
                                                    </div>
                                                {/each}
                                            </div>
                                        </div>
                                    {/if}
                                </div>
                            </div>
                        {/each}
                    </div>
                </div>
            {/if}

            {#if activeMode === 'match' && regexPattern && testString && matchResults.length === 0 && isValidRegex}
                <div class="no-matches">
                    <span class="no-match-icon">🔍</span>
                    <span>没有找到匹配项</span>
                </div>
            {/if}
        </div>
    </div>

    <!-- 语法参考 -->
    <div class="reference-section">
        <div class="reference-header">
            <h3>📖 正则表达式语法参考</h3>
        </div>
        <div class="reference-grid">
            {#each syntaxReference as ref}
                <div class="reference-category">
                    <h4>{ref.category}</h4>
                    <div class="reference-items">
                        {#each ref.items as item}
                            <div class="reference-item">
                                <code class="ref-pattern">{item.pattern}</code>
                                <span class="ref-desc">{item.desc}</span>
                            </div>
                        {/each}
                    </div>
                </div>
            {/each}
        </div>
    </div>
</div>

<style>
    .regex-tool {
        display: flex;
        flex-direction: column;
        gap: 1.5rem;
    }

    /* 标题区域 */
    .tool-header {
        display: flex;
        justify-content: space-between;
        align-items: flex-start;
        padding-bottom: 1rem;
        border-bottom: 1px solid var(--border);
    }

    .header-content h2 {
        font-size: 1.5rem;
        margin: 0 0 0.25rem 0;
        background: linear-gradient(135deg, #10b981 0%, #14b8a6 100%);
        -webkit-background-clip: text;
        -webkit-text-fill-color: transparent;
        background-clip: text;
    }

    .header-desc {
        font-size: 0.9rem;
        color: var(--text-muted);
        margin: 0;
    }

    .clear-btn {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        padding: 0.5rem 1rem !important;
        background: var(--bg-card) !important;
        border: 1px solid var(--border) !important;
        font-size: 0.85rem;
    }

    .clear-btn:hover {
        border-color: #ef4444 !important;
        color: #ef4444 !important;
    }

    /* 编辑器布局 */
    .editor-layout {
        display: grid;
        grid-template-columns: 1fr 1fr;
        gap: 1.5rem;
    }

    @media (max-width: 1000px) {
        .editor-layout {
            grid-template-columns: 1fr;
        }
    }

    /* 输入面板 */
    .input-panel {
        display: flex;
        flex-direction: column;
        gap: 1rem;
    }

    .input-section {
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
    }

    .input-section label {
        font-size: 0.9rem;
        font-weight: 600;
        color: var(--text-secondary);
    }

    .section-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
    }

    .header-actions {
        display: flex;
        gap: 0.5rem;
    }

    /* 正则表达式输入框 */
    .regex-input-wrapper {
        display: flex;
        align-items: center;
        background: var(--bg-dark);
        border: 2px solid var(--border);
        border-radius: var(--radius-sm);
        padding: 0.5rem;
        transition: border-color 0.2s;
    }

    .regex-input-wrapper:focus-within {
        border-color: var(--primary);
    }

    .regex-input-wrapper.error {
        border-color: #ef4444;
    }

    .regex-delimiter {
        color: var(--primary);
        font-size: 1.25rem;
        font-weight: bold;
        padding: 0 0.25rem;
        font-family: 'JetBrains Mono', monospace;
    }

    .regex-input-wrapper input {
        flex: 1;
        background: transparent !important;
        border: none !important;
        padding: 0.5rem !important;
        font-family: 'JetBrains Mono', monospace;
        font-size: 0.95rem;
    }

    .regex-input-wrapper input:focus {
        outline: none;
    }

    /* 标志按钮 */
    .flags-container {
        display: flex;
        gap: 0.25rem;
        padding-left: 0.5rem;
        border-left: 1px solid var(--border);
        margin-left: 0.5rem;
    }

    .flag-btn {
        width: 28px;
        height: 28px;
        display: flex;
        align-items: center;
        justify-content: center;
        padding: 0 !important;
        background: var(--bg-card) !important;
        border: 1px solid var(--border) !important;
        border-radius: 4px !important;
        font-family: 'JetBrains Mono', monospace;
        font-size: 0.85rem;
        font-weight: 600;
        color: var(--text-muted) !important;
        transition: all 0.2s;
    }

    .flag-btn:hover {
        border-color: var(--primary) !important;
    }

    .flag-btn.active {
        background: var(--primary) !important;
        border-color: var(--primary) !important;
        color: white !important;
    }

    /* 正则操作按钮 */
    .regex-actions {
        display: flex;
        gap: 0.5rem;
    }

    .action-btn {
        flex: 1;
        padding: 0.5rem !important;
        background: var(--bg-card) !important;
        border: 1px solid var(--border) !important;
        font-size: 0.8rem;
    }

    .action-btn:hover {
        border-color: var(--primary) !important;
    }

    /* 模式选项卡 */
    .pattern-selector {
        position: relative;
    }

    .pattern-btn {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        padding: 0.4rem 0.75rem !important;
        background: var(--bg-card) !important;
        border: 1px solid var(--border) !important;
        font-size: 0.85rem;
    }

    .pattern-btn:hover {
        border-color: var(--primary) !important;
    }

    .dropdown-arrow {
        font-size: 0.7rem;
    }

    .pattern-dropdown {
        position: absolute;
        top: calc(100% + 0.5rem);
        right: 0;
        width: 400px;
        max-height: 450px;
        background: var(--bg-dark);
        border: 1px solid var(--border);
        border-radius: var(--radius-sm);
        box-shadow: 0 10px 40px rgba(0,0,0,0.3);
        z-index: 100;
        overflow: hidden;
    }

    .category-tabs {
        display: flex;
        flex-wrap: wrap;
        gap: 0.25rem;
        padding: 0.75rem;
        background: var(--bg-card);
        border-bottom: 1px solid var(--border);
    }

    .cat-tab {
        padding: 0.3rem 0.6rem !important;
        background: transparent !important;
        border: 1px solid var(--border) !important;
        border-radius: 999px !important;
        font-size: 0.75rem;
    }

    .cat-tab.active {
        background: var(--primary) !important;
        border-color: var(--primary) !important;
        color: white !important;
    }

    .pattern-list {
        max-height: 350px;
        overflow-y: auto;
    }

    .pattern-item {
        display: flex;
        flex-direction: column;
        gap: 0.25rem;
        width: 100%;
        padding: 0.75rem !important;
        background: transparent !important;
        border: none !important;
        border-bottom: 1px solid var(--border) !important;
        border-radius: 0 !important;
        text-align: left;
    }

    .pattern-item:hover {
        background: rgba(99, 102, 241, 0.1) !important;
    }

    .pattern-info {
        display: flex;
        justify-content: space-between;
        align-items: center;
    }

    .pattern-name {
        font-weight: 600;
        font-size: 0.85rem;
    }

    .pattern-desc {
        font-size: 0.75rem;
        color: var(--text-muted);
    }

    .pattern-code {
        font-size: 0.75rem;
        padding: 0.25rem 0.5rem;
        background: var(--bg-card);
        border-radius: 4px;
        color: var(--primary);
        font-family: 'JetBrains Mono', monospace;
        word-break: break-all;
    }

    /* 错误消息 */
    .error-message {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        padding: 0.75rem;
        background: rgba(239, 68, 68, 0.1);
        border: 1px solid rgba(239, 68, 68, 0.3);
        border-radius: var(--radius-sm);
        color: #ef4444;
        font-size: 0.85rem;
    }

    .error-icon {
        font-size: 1rem;
    }

    /* 模式切换标签 */
    .mode-tabs {
        display: flex;
        background: var(--bg-dark);
        border: 1px solid var(--border);
        border-radius: var(--radius-sm);
        overflow: hidden;
    }

    .mode-tab {
        flex: 1;
        padding: 0.75rem !important;
        background: transparent !important;
        border: none !important;
        border-radius: 0 !important;
        font-size: 0.9rem;
        color: var(--text-muted) !important;
        transition: all 0.2s;
    }

    .mode-tab:hover {
        background: rgba(99, 102, 241, 0.05) !important;
    }

    .mode-tab.active {
        background: var(--primary) !important;
        color: white !important;
    }

    /* 测试文本输入 */
    .input-section textarea {
        min-height: 150px;
        resize: vertical;
        font-family: 'JetBrains Mono', monospace;
        font-size: 0.9rem;
        line-height: 1.6;
    }

    .hint-badge {
        font-size: 0.75rem;
        padding: 0.2rem 0.5rem;
        background: rgba(99, 102, 241, 0.1);
        border-radius: 4px;
        color: var(--primary);
    }

    /* 结果面板 */
    .result-panel {
        display: flex;
        flex-direction: column;
        gap: 1rem;
    }

    .result-section {
        background: var(--bg-dark);
        border: 1px solid var(--border);
        border-radius: var(--radius-sm);
        overflow: hidden;
    }

    .result-section .section-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 0.75rem 1rem;
        background: var(--bg-card);
        border-bottom: 1px solid var(--border);
    }

    .section-title {
        font-weight: 600;
        font-size: 0.9rem;
    }

    .match-count {
        font-size: 0.75rem;
        padding: 0.25rem 0.75rem;
        background: var(--accent-green);
        color: white;
        border-radius: 999px;
        font-weight: 500;
    }

    .preview-box {
        padding: 1rem;
        min-height: 120px;
        max-height: 250px;
        overflow-y: auto;
    }

    .highlighted-content, .replaced-content {
        font-family: 'JetBrains Mono', monospace;
        font-size: 0.9rem;
        line-height: 1.8;
        white-space: pre-wrap;
        word-break: break-all;
    }

    .highlighted-content :global(.match-highlight) {
        background: linear-gradient(135deg, rgba(234, 179, 8, 0.3), rgba(234, 179, 8, 0.2));
        color: #fbbf24;
        padding: 0.1rem 0.25rem;
        border-radius: 3px;
        border: 1px solid rgba(234, 179, 8, 0.5);
    }

    .empty-hint {
        color: var(--text-muted);
        font-style: italic;
        text-align: center;
        padding: 2rem;
    }

    .copy-result-btn {
        margin: 0 1rem 1rem;
        padding: 0.5rem !important;
        background: var(--bg-card) !important;
        border: 1px solid var(--border) !important;
        font-size: 0.85rem;
        width: calc(100% - 2rem);
    }

    .copy-result-btn:hover {
        border-color: var(--primary) !important;
    }

    /* 匹配详情 */
    .matches-section {
        background: var(--bg-dark);
        border: 1px solid var(--border);
        border-radius: var(--radius-sm);
        overflow: hidden;
    }

    .matches-section .section-header {
        padding: 0.75rem 1rem;
        background: var(--bg-card);
        border-bottom: 1px solid var(--border);
    }

    .matches-list {
        max-height: 300px;
        overflow-y: auto;
        padding: 0.75rem;
        display: flex;
        flex-direction: column;
        gap: 0.75rem;
    }

    .match-card {
        background: var(--bg-card);
        border: 1px solid var(--border);
        border-radius: var(--radius-sm);
        overflow: hidden;
    }

    .match-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 0.5rem 0.75rem;
        background: var(--primary);
        color: white;
        font-size: 0.8rem;
    }

    .match-number {
        font-weight: 600;
    }

    .match-position {
        font-family: 'JetBrains Mono', monospace;
        font-size: 0.75rem;
        opacity: 0.9;
    }

    .match-body {
        padding: 0.75rem;
    }

    .match-row {
        display: flex;
        align-items: flex-start;
        gap: 0.5rem;
        margin-bottom: 0.5rem;
    }

    .match-label {
        font-size: 0.8rem;
        color: var(--text-muted);
        white-space: nowrap;
    }

    .match-value {
        font-size: 0.85rem;
        padding: 0.25rem 0.5rem;
        background: var(--bg-dark);
        border-radius: 4px;
        word-break: break-all;
        color: #fbbf24;
    }

    .groups-section {
        margin-top: 0.5rem;
        padding-top: 0.5rem;
        border-top: 1px solid var(--border);
    }

    .groups-list {
        display: flex;
        flex-direction: column;
        gap: 0.375rem;
        margin-top: 0.375rem;
    }

    .group-item {
        display: flex;
        align-items: center;
        gap: 0.5rem;
    }

    .group-index {
        font-size: 0.75rem;
        padding: 0.15rem 0.4rem;
        background: var(--primary);
        color: white;
        border-radius: 4px;
        font-family: 'JetBrains Mono', monospace;
    }

    .group-value {
        font-size: 0.85rem;
        padding: 0.2rem 0.5rem;
        background: var(--bg-dark);
        border-radius: 4px;
        color: #10b981;
    }

    /* 无匹配 */
    .no-matches {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 0.5rem;
        padding: 2rem;
        background: var(--bg-dark);
        border: 1px solid var(--border);
        border-radius: var(--radius-sm);
        color: var(--text-muted);
    }

    .no-match-icon {
        font-size: 2rem;
        opacity: 0.5;
    }

    /* 语法参考 */
    .reference-section {
        background: var(--bg-dark);
        border: 1px solid var(--border);
        border-radius: var(--radius-sm);
        overflow: hidden;
    }

    .reference-header {
        padding: 0.875rem 1rem;
        background: var(--bg-card);
        border-bottom: 1px solid var(--border);
    }

    .reference-header h3 {
        margin: 0;
        font-size: 1rem;
    }

    .reference-grid {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
        gap: 1rem;
        padding: 1rem;
    }

    .reference-category h4 {
        font-size: 0.85rem;
        color: var(--primary);
        margin: 0 0 0.75rem 0;
        padding-bottom: 0.5rem;
        border-bottom: 1px solid var(--border);
    }

    .reference-items {
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
    }

    .reference-item {
        display: flex;
        align-items: center;
        gap: 0.75rem;
    }

    .ref-pattern {
        min-width: 60px;
        padding: 0.25rem 0.5rem;
        background: var(--bg-card);
        border-radius: 4px;
        font-size: 0.85rem;
        text-align: center;
        color: #fbbf24;
        font-family: 'JetBrains Mono', monospace;
    }

    .ref-desc {
        font-size: 0.8rem;
        color: var(--text-secondary);
    }
</style>
