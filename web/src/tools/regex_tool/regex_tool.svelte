<script lang="ts">
    import Button from "../../lib/ui/Button.svelte";
    import CopyButton from "../../lib/ui/CopyButton.svelte";
    import Panel from "../../lib/ui/Panel.svelte";
    import SplitPane from "../../lib/ui/SplitPane.svelte";
    import Tabs from "../../lib/ui/Tabs.svelte";
    import TextArea from "../../lib/ui/TextArea.svelte";
    import TextInput from "../../lib/ui/TextInput.svelte";
    import EmptyState from "../../lib/ui/EmptyState.svelte";
    import { escapeHtml } from "../../lib/format";
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
    let activeMode = $state<'match' | 'replace'>('match');
    let selectedCategory = $state<string>('all');

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
        handleRegexChange();
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

<div class="tool">
  <Panel title="正则表达式">
    {#snippet actions()}
      <CopyButton value={() => regexPattern} label="复制表达式" disabled={!regexPattern} />
      <CopyButton value={() => `/${regexPattern}/${regexFlags}`} label="复制完整" disabled={!regexPattern} />
      <Button size="sm" variant="ghost" onclick={clearAll}>清空</Button>
    {/snippet}

    <div class="pattern" class:invalid={!isValidRegex}>
      <span class="slash">/</span>
      <input
        type="text"
        bind:value={regexPattern}
        oninput={handleRegexChange}
        placeholder="输入正则表达式…"
        spellcheck="false"
        aria-label="正则表达式"
      />
      <span class="slash">/</span>
      <div class="flags">
        {#each [
          { flag: "g", title: "全局匹配 global" },
          { flag: "i", title: "忽略大小写 insensitive" },
          { flag: "m", title: "多行模式 multiline" },
          { flag: "s", title: "点号匹配换行 dotAll" },
        ] as item (item.flag)}
          <button
            class="flag"
            class:on={regexFlags.includes(item.flag)}
            onclick={() => toggleFlag(item.flag)}
            title={item.title}
          >
            {item.flag}
          </button>
        {/each}
      </div>
    </div>

    {#if errorMessage}
      <p class="error">{errorMessage}</p>
    {/if}

    <details class="library">
      <summary>常用正则模板</summary>
      <div class="cats">
        {#each categories as cat (cat)}
          <button class="cat" class:on={selectedCategory === cat} onclick={() => (selectedCategory = cat)}>
            {cat === "all" ? "全部" : cat}
          </button>
        {/each}
      </div>
      <div class="patterns">
        {#each getFilteredPatterns() as pattern (pattern.name)}
          <button class="pattern-item" onclick={() => selectPattern(pattern)}>
            <span class="p-name">{pattern.name}</span>
            <span class="p-desc">{pattern.description}</span>
            <code>{pattern.pattern}</code>
          </button>
        {/each}
      </div>
    </details>
  </Panel>

  <div class="mode">
    <Tabs
      bind:value={activeMode}
      tabs={[
        { value: "match", label: "匹配" },
        { value: "replace", label: "替换" },
      ]}
      ariaLabel="模式"
      onchange={() => activeMode === "replace" && performReplace()}
    />
    {#if activeMode === "replace"}
      <span class="hint">替换文本支持 $1、$2 引用捕获组</span>
    {/if}
  </div>

  <SplitPane>
    {#snippet left()}
      <Panel title="测试文本" grow>
        {#snippet actions()}
          <span class="count">{testString.length} 字符</span>
        {/snippet}
        <TextArea
          bind:value={testString}
          fill
          placeholder="输入需要测试的文本…"
          ariaLabel="测试文本"
          oninput={() => {
            testRegex();
            if (activeMode === "replace") performReplace();
          }}
        />
        {#if activeMode === "replace"}
          <div class="replace-input">
            <TextInput
              bind:value={replaceString}
              mono
              placeholder="替换为…"
              ariaLabel="替换文本"
              oninput={performReplace}
            />
          </div>
        {/if}
      </Panel>
    {/snippet}

    {#snippet right()}
      <Panel title={activeMode === "match" ? "匹配高亮" : "替换预览"} grow>
        {#snippet actions()}
          {#if activeMode === "match"}
            <span class="count">{matchResults.length} 个匹配</span>
          {:else}
            <CopyButton value={() => replacedText} disabled={!replacedText} />
          {/if}
        {/snippet}

        {#if activeMode === "match"}
          {#if !testString}
            <EmptyState icon=".*" title="输入测试文本查看匹配" />
          {:else}
            <div class="preview">{@html highlightedText}</div>
            {#if matchResults.length > 0}
              <ul class="matches">
                {#each matchResults as match, index (index)}
                  <li>
                    <div class="m-head">
                      <span class="m-no">#{index + 1}</span>
                      <span class="m-pos">{match.index}–{match.lastIndex}</span>
                      <CopyButton value={match.fullMatch} icon title="复制匹配内容" />
                    </div>
                    <code class="m-value">{match.fullMatch}</code>
                    {#if match.groups.length > 0}
                      <div class="groups">
                        {#each match.groups as group, groupIndex (groupIndex)}
                          <span class="group"><i>${groupIndex + 1}</i><code>{group ?? ""}</code></span>
                        {/each}
                      </div>
                    {/if}
                  </li>
                {/each}
              </ul>
            {:else if regexPattern && isValidRegex}
              <p class="none">没有找到匹配项</p>
            {/if}
          {/if}
        {:else if replacedText}
          <pre class="preview">{replacedText}</pre>
        {:else}
          <EmptyState icon="↻" title="输入替换文本查看结果" />
        {/if}
      </Panel>
    {/snippet}
  </SplitPane>

  <details class="reference">
    <summary>语法参考</summary>
    <div class="ref-grid">
      {#each syntaxReference as ref (ref.category)}
        <div class="ref-cat">
          <h3>{ref.category}</h3>
          {#each ref.items as item (item.pattern)}
            <div class="ref-item">
              <code>{item.pattern}</code>
              <span>{item.desc}</span>
            </div>
          {/each}
        </div>
      {/each}
    </div>
  </details>
</div>

<style>
  .tool {
    display: flex;
    flex-direction: column;
    gap: var(--sp-3);
    flex: 1;
    min-height: 0;
  }

  .pattern {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    padding: var(--sp-2) var(--sp-3);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    background: var(--color-bg);
  }

  .pattern:focus-within {
    border-color: var(--color-primary);
    box-shadow: 0 0 0 3px var(--color-focus-ring);
  }

  .pattern.invalid {
    border-color: var(--color-danger);
  }

  .slash {
    color: var(--color-text-muted);
    font-family: var(--font-mono);
    font-size: var(--fs-lg);
  }

  .pattern input {
    flex: 1;
    min-width: 0;
    background: none;
    border: none;
    outline: none;
    color: var(--color-text);
    font-family: var(--font-mono);
    font-size: var(--fs-md);
  }

  .flags {
    display: flex;
    gap: 2px;
  }

  .flag {
    width: 24px;
    height: 24px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border);
    background: var(--color-surface-2);
    color: var(--color-text-muted);
    font-family: var(--font-mono);
    font-size: var(--fs-sm);
    transition:
      background var(--motion),
      color var(--motion);
  }

  .flag.on {
    background: var(--color-primary);
    border-color: var(--color-primary);
    color: var(--color-text-inverse);
  }

  .library,
  .reference {
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    background: var(--color-surface-2);
  }

  .library {
    margin-top: var(--sp-3);
  }

  summary {
    padding: var(--sp-2) var(--sp-3);
    cursor: pointer;
    font-size: var(--fs-sm);
    color: var(--color-text-secondary);
  }

  .cats {
    display: flex;
    flex-wrap: wrap;
    gap: var(--sp-1);
    padding: 0 var(--sp-3) var(--sp-2);
  }

  .cat {
    padding: 0.15rem var(--sp-2);
    border-radius: var(--radius-full);
    border: 1px solid var(--color-border);
    font-size: var(--fs-xs);
    color: var(--color-text-muted);
  }

  .cat.on {
    border-color: var(--color-primary-border);
    background: var(--color-primary-bg);
    color: var(--color-primary);
  }

  .patterns {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
    gap: var(--sp-2);
    padding: 0 var(--sp-3) var(--sp-3);
    max-height: 260px;
    overflow-y: auto;
  }

  .pattern-item {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: var(--sp-2);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    background: var(--color-surface);
    text-align: left;
  }

  .pattern-item:hover {
    border-color: var(--color-primary-border);
  }

  .p-name {
    font-size: var(--fs-sm);
    color: var(--color-text);
  }

  .p-desc {
    font-size: var(--fs-xs);
    color: var(--color-text-muted);
  }

  .pattern-item code {
    font-family: var(--font-mono);
    font-size: var(--fs-xs);
    color: var(--color-accent);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .mode {
    display: flex;
    align-items: center;
    gap: var(--sp-3);
  }

  .hint,
  .count {
    font-size: var(--fs-xs);
    color: var(--color-text-muted);
  }

  .count {
    font-family: var(--font-mono);
  }

  .replace-input {
    margin-top: var(--sp-2);
  }

  .preview {
    padding: var(--sp-3);
    border-radius: var(--radius-sm);
    background: var(--color-code-bg);
    font-family: var(--font-mono);
    font-size: var(--fs-sm);
    line-height: var(--lh-relaxed);
    color: var(--color-text-secondary);
    white-space: pre-wrap;
    word-break: break-word;
    max-height: 40%;
    overflow: auto;
  }

  /* @html 注入的高亮片段不受 scoped 样式影响 */
  .preview :global(mark) {
    background: var(--color-highlight-bg);
    color: var(--color-text);
    border-radius: 3px;
    padding: 0 2px;
  }

  .matches {
    list-style: none;
    display: flex;
    flex-direction: column;
    gap: var(--sp-2);
    margin-top: var(--sp-3);
  }

  .matches li {
    padding: var(--sp-2);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    background: var(--color-surface-2);
  }

  .m-head {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
  }

  .m-no {
    font-family: var(--font-mono);
    font-size: var(--fs-xs);
    color: var(--color-primary);
  }

  .m-pos {
    flex: 1;
    font-family: var(--font-mono);
    font-size: var(--fs-xs);
    color: var(--color-text-muted);
  }

  .m-value {
    display: block;
    margin-top: var(--sp-1);
    font-family: var(--font-mono);
    font-size: var(--fs-sm);
    color: var(--color-text);
    word-break: break-all;
  }

  .groups {
    display: flex;
    flex-wrap: wrap;
    gap: var(--sp-2);
    margin-top: var(--sp-2);
  }

  .group {
    display: inline-flex;
    align-items: center;
    gap: var(--sp-1);
    padding: 0.1rem var(--sp-2);
    border-radius: var(--radius-sm);
    background: var(--color-surface-3);
    font-size: var(--fs-xs);
  }

  .group i {
    font-style: normal;
    font-family: var(--font-mono);
    color: var(--color-accent);
  }

  .group code {
    font-family: var(--font-mono);
    color: var(--color-text-secondary);
  }

  .none {
    margin-top: var(--sp-3);
    font-size: var(--fs-sm);
    color: var(--color-text-muted);
  }

  .error {
    margin-top: var(--sp-2);
    padding: var(--sp-2) var(--sp-3);
    border: 1px solid var(--color-danger-border);
    border-radius: var(--radius-sm);
    background: var(--color-danger-bg);
    color: var(--color-danger);
    font-size: var(--fs-sm);
  }

  .ref-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
    gap: var(--sp-4);
    padding: 0 var(--sp-3) var(--sp-3);
  }

  .ref-cat h3 {
    font-size: var(--fs-sm);
    color: var(--color-text);
    margin-bottom: var(--sp-2);
  }

  .ref-item {
    display: flex;
    gap: var(--sp-2);
    padding: 2px 0;
    font-size: var(--fs-xs);
  }

  .ref-item code {
    min-width: 3.5rem;
    font-family: var(--font-mono);
    color: var(--color-accent);
  }

  .ref-item span {
    color: var(--color-text-muted);
  }
</style>
