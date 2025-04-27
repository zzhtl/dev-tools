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
  }

  let regexPattern = $state("");
  let testString = $state("");
  let matchResults = $state<MatchResult[]>([]);
  let errorMessage = $state("");
  let regexFlags = $state("g");
  let highlightedText = $state("");
  let isValidRegex = $state(true);
  let showPatternDropdown = $state(false);

  // 常用正则表达式集合
  const commonPatterns: RegexPattern[] = [
    {
      name: "邮箱",
      pattern: "\\w+@[a-zA-Z0-9]+\\.[a-zA-Z]{2,}",
      description: "匹配邮箱地址"
    },
    {
      name: "中文",
      pattern: "[\\u4e00-\\u9fa5]",
      description: "匹配中文字符"
    },
    {
      name: "双字节字符",
      pattern: "[^\\x00-\\xff]",
      description: "匹配双字节字符（包含汉字）"
    },
    {
      name: "时间",
      pattern: "([01]\\d|2[0-3]):[0-5]\\d:[0-5]\\d",
      description: "匹配时间（时:分:秒）"
    },
    {
      name: "IPV4",
      pattern: "((25[0-5]|2[0-4]\\d|1\\d{2}|[1-9]?\\d)\\.){3}(25[0-5]|2[0-4]\\d|1\\d{2}|[1-9]?\\d)",
      description: "匹配IP地址（IPV4）"
    },
    {
      name: "身份证",
      pattern: "(^\\d{15}$)|(^\\d{18}$)|(^\\d{17}(\\d|X|x)$)",
      description: "匹配身份证号码"
    },
    {
      name: "日期",
      pattern: "\\d{4}-(0[1-9]|1[0-2])-(0[1-9]|[12]\\d|3[01])",
      description: "匹配日期（年-月-日）"
    },
    {
      name: "正整数",
      pattern: "^[1-9]\\d*$",
      description: "匹配正整数"
    },
    {
      name: "负整数",
      pattern: "^-[1-9]\\d*$",
      description: "匹配负整数"
    },
    {
      name: "手机号",
      pattern: "^1[3-9]\\d{9}$",
      description: "匹配手机号码"
    },
    {
      name: "电驴链接",
      pattern: "ed2k://\\|file\\|.*\\|/",
      description: "匹配电驴链接"
    },
    {
      name: "车牌号",
      pattern: "[京津沪渝冀豫云辽黑湘皖鲁新苏浙赣鄂桂甘晋蒙陕吉闽贵粤青藏川宁琼使领][A-Z][A-HJ-NP-Z0-9]{4}[A-HJ-NP-Z0-9挂学警港澳]",
      description: "匹配车牌号"
    }
  ];

  function handleRegexChange() {
    validateRegex();
    if (isValidRegex) {
      testRegex();
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
    } catch (e: any) {
      isValidRegex = false;
      errorMessage = `无效的正则表达式: ${e.message}`;
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
      
    } catch (e: any) {
      errorMessage = `执行正则表达式时出错: ${e.message}`;
      matchResults = [];
      highlightedText = testString;
    }
  }
  
  function generateHighlightedText(text: string, matches: MatchResult[]): string {
    if (!matches.length) return text;
    
    // 对匹配进行排序，确保按照索引顺序处理
    const sortedMatches = [...matches].sort((a, b) => a.index - b.index);
    
    let result = '';
    let lastIndex = 0;
    
    for (const match of sortedMatches) {
      // 添加匹配前的文本
      result += text.substring(lastIndex, match.index);
      
      // 添加高亮的匹配文本
      result += `<span class="match-highlight">${text.substring(match.index, match.lastIndex)}</span>`;
      
      lastIndex = match.lastIndex;
    }
    
    // 添加最后一个匹配后的文本
    result += text.substring(lastIndex);
    
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
    }
  }

  function selectPattern(pattern: RegexPattern) {
    regexPattern = pattern.pattern;
    showPatternDropdown = false;
    handleRegexChange();
  }

  function togglePatternDropdown() {
    showPatternDropdown = !showPatternDropdown;
  }

  function handleClickOutside(event: MouseEvent) {
    const target = event.target as HTMLElement;
    if (!target.closest('.pattern-dropdown') && !target.closest('.select-pattern-btn')) {
      showPatternDropdown = false;
    }
  }
</script>

<svelte:window on:click={handleClickOutside} />

<main class="container">
  <h2>正则表达式测试工具</h2>

  <div class="regex-container">
    <div class="input-group">
      <div class="label-with-actions">
        <label for="regex-pattern">正则表达式</label>
        <button 
          class="select-pattern-btn" 
          onclick={togglePatternDropdown}
          title="选择常用正则表达式"
        >
          常用正则 ▾
        </button>
        
        {#if showPatternDropdown}
          <div class="pattern-dropdown">
            {#each commonPatterns as pattern}
              <div class="pattern-item" onclick={() => selectPattern(pattern)}>
                <strong>{pattern.name}</strong>
                <span>{pattern.description}</span>
              </div>
            {/each}
          </div>
        {/if}
      </div>
      
      <div class="regex-input-container">
        <input 
          id="regex-pattern" 
          type="text" 
          bind:value={regexPattern}
          onkeyup={handleRegexChange}
          placeholder="输入正则表达式..."
          class:error={!isValidRegex}
        />
        <div class="flags-container">
          <button 
            class:active={regexFlags.includes('g')}
            onclick={() => toggleFlag('g')} 
            class="flag-button" 
            title="全局匹配"
          >g</button>
          <button 
            class:active={regexFlags.includes('i')}
            onclick={() => toggleFlag('i')} 
            class="flag-button" 
            title="忽略大小写"
          >i</button>
          <button 
            class:active={regexFlags.includes('m')}
            onclick={() => toggleFlag('m')} 
            class="flag-button" 
            title="多行匹配"
          >m</button>
          <button 
            class:active={regexFlags.includes('s')}
            onclick={() => toggleFlag('s')} 
            class="flag-button" 
            title="点号匹配所有字符（包括换行符）"
          >s</button>
        </div>
      </div>
    </div>

    <div class="input-group">
      <label for="test-string">测试文本</label>
      <textarea 
        id="test-string" 
        bind:value={testString}
        onkeyup={testRegex}
        placeholder="输入需要测试的文本..."
      ></textarea>
    </div>

    {#if errorMessage}
      <div class="error-message">{errorMessage}</div>
    {/if}

    <div class="results">
      <h3>匹配结果</h3>
      
      {#if regexPattern && testString && isValidRegex}
        <div class="highlighted-text">
          <!-- 使用安全的方式渲染HTML -->
          {@html highlightedText}
        </div>
        
        {#if matchResults.length === 0}
          <p class="no-matches">没有找到匹配项</p>
        {:else}
          <div class="matches-container">
            <h4>找到 {matchResults.length} 个匹配</h4>
            <div class="matches-list">
              {#each matchResults as match, index}
                <div class="match-item">
                  <div class="match-header">匹配 #{index + 1}</div>
                  <div class="match-content">
                    <div>
                      <strong>完整匹配:</strong> 
                      <code>"{match.fullMatch}"</code>
                    </div>
                    <div>
                      <strong>位置:</strong> 
                      <code>{match.index} - {match.lastIndex}</code>
                    </div>
                    {#if match.groups.length > 0}
                      <div class="match-groups">
                        <strong>捕获组:</strong>
                        <ol>
                          {#each match.groups as group, groupIndex}
                            <li><code>"{group || '<空>'}"</code></li>
                          {/each}
                        </ol>
                      </div>
                    {/if}
                  </div>
                </div>
              {/each}
            </div>
          </div>
        {/if}
      {/if}
    </div>
  </div>

  <div class="regex-cheatsheet">
    <h3>正则表达式速查表</h3>
    <div class="cheatsheet-grid">
      <div class="cheatsheet-item">
        <div class="cheatsheet-pattern">.</div>
        <div class="cheatsheet-desc">匹配任意字符（除换行符外）</div>
      </div>
      <div class="cheatsheet-item">
        <div class="cheatsheet-pattern">\w</div>
        <div class="cheatsheet-desc">匹配字母、数字、下划线</div>
      </div>
      <div class="cheatsheet-item">
        <div class="cheatsheet-pattern">\d</div>
        <div class="cheatsheet-desc">匹配数字</div>
      </div>
      <div class="cheatsheet-item">
        <div class="cheatsheet-pattern">\s</div>
        <div class="cheatsheet-desc">匹配空白字符</div>
      </div>
      <div class="cheatsheet-item">
        <div class="cheatsheet-pattern">^</div>
        <div class="cheatsheet-desc">匹配行首</div>
      </div>
      <div class="cheatsheet-item">
        <div class="cheatsheet-pattern">$</div>
        <div class="cheatsheet-desc">匹配行尾</div>
      </div>
      <div class="cheatsheet-item">
        <div class="cheatsheet-pattern">[abc]</div>
        <div class="cheatsheet-desc">匹配a、b或c</div>
      </div>
      <div class="cheatsheet-item">
        <div class="cheatsheet-pattern">[^abc]</div>
        <div class="cheatsheet-desc">匹配除a、b、c以外的字符</div>
      </div>
      <div class="cheatsheet-item">
        <div class="cheatsheet-pattern">a|b</div>
        <div class="cheatsheet-desc">匹配a或b</div>
      </div>
      <div class="cheatsheet-item">
        <div class="cheatsheet-pattern">(abc)</div>
        <div class="cheatsheet-desc">捕获组</div>
      </div>
      <div class="cheatsheet-item">
        <div class="cheatsheet-pattern">a?</div>
        <div class="cheatsheet-desc">匹配0或1个a</div>
      </div>
      <div class="cheatsheet-item">
        <div class="cheatsheet-pattern">a*</div>
        <div class="cheatsheet-desc">匹配0或多个a</div>
      </div>
      <div class="cheatsheet-item">
        <div class="cheatsheet-pattern">a+</div>
        <div class="cheatsheet-desc">匹配1或多个a</div>
      </div>
      <div class="cheatsheet-item">
        <div class="cheatsheet-pattern">a&#123;3&#125;</div>
        <div class="cheatsheet-desc">匹配3个a</div>
      </div>
      <div class="cheatsheet-item">
        <div class="cheatsheet-pattern">a&#123;3,&#125;</div>
        <div class="cheatsheet-desc">匹配至少3个a</div>
      </div>
      <div class="cheatsheet-item">
        <div class="cheatsheet-pattern">a&#123;3,5&#125;</div>
        <div class="cheatsheet-desc">匹配3到5个a</div>
      </div>
    </div>
  </div>
</main>

<style>
  :root {
    --primary-color: #4f46e5;
    --primary-light: #6366f1;
    --text-color: #111827;
    --bg-color: #f9fafb;
    --card-bg: #ffffff;
    --border-radius: 8px;
    --shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -1px rgba(0, 0, 0, 0.06);
    --error-color: #ef4444;
  }

  .container {
    margin: 0 auto;
    max-width: 1200px;
    padding: 1rem;
    background: var(--bg-color);
    min-height: 100vh;
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }

  h2 {
    margin: 0 0 1rem 0;
    color: var(--primary-color);
    font-size: 1.5rem;
  }

  h3 {
    margin: 0.5rem 0;
    font-size: 1.2rem;
    color: var(--text-color);
  }

  h4 {
    margin: 0.5rem 0;
    font-size: 1rem;
    color: var(--text-color);
  }

  .regex-container {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .input-group {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  label {
    font-weight: 600;
    color: var(--text-color);
  }

  input[type="text"], textarea {
    padding: 0.75rem;
    border: 1px solid #e5e7eb;
    border-radius: var(--border-radius);
    font-family: monospace;
    font-size: 14px;
    width: 100%;
    background: var(--card-bg);
    color: var(--text-color);
  }

  textarea {
    min-height: 120px;
    resize: vertical;
  }

  .regex-input-container {
    display: flex;
    align-items: center;
  }

  .regex-delimiter {
    padding: 0 0.5rem;
    font-family: monospace;
    font-size: 18px;
    color: var(--primary-color);
  }

  input[type="text"].error {
    border-color: var(--error-color);
  }

  .error-message {
    color: var(--error-color);
    font-size: 14px;
    padding: 0.5rem;
    background-color: rgba(239, 68, 68, 0.1);
    border-radius: var(--border-radius);
  }

  .flags-container {
    display: flex;
    gap: 0.25rem;
    margin-left: 0.5rem;
  }

  .flag-button {
    width: 28px;
    height: 28px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-family: monospace;
    font-size: 14px;
    background: transparent;
    border: 1px solid #e5e7eb;
    border-radius: 4px;
    cursor: pointer;
    color: var(--text-color);
  }

  .flag-button.active {
    background-color: var(--primary-color);
    color: white;
    border-color: var(--primary-color);
  }

  .results {
    margin-top: 1rem;
    border: 1px solid #e5e7eb;
    border-radius: var(--border-radius);
    padding: 1rem;
    background: var(--card-bg);
  }

  .highlighted-text {
    padding: 0.75rem;
    border: 1px solid #e5e7eb;
    border-radius: var(--border-radius);
    white-space: pre-wrap;
    font-family: monospace;
    font-size: 14px;
    margin-top: 0.5rem;
    background: var(--bg-color);
    min-height: 80px;
    max-height: 200px;
    overflow-y: auto;
  }

  .match-highlight {
    background-color: #fef08a;
    color: #854d0e;
    padding: 0.1rem 0;
    border-radius: 2px;
  }

  .no-matches {
    font-style: italic;
    color: #6b7280;
  }

  .matches-container {
    margin-top: 1rem;
  }

  .matches-list {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    margin-top: 0.5rem;
    max-height: 300px;
    overflow-y: auto;
  }

  .match-item {
    border: 1px solid #e5e7eb;
    border-radius: var(--border-radius);
    overflow: hidden;
  }

  .match-header {
    background-color: var(--primary-light);
    color: white;
    padding: 0.5rem;
    font-weight: 600;
  }

  .match-content {
    padding: 0.75rem;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  code {
    background: var(--bg-color);
    padding: 0.2rem 0.4rem;
    border-radius: 4px;
    font-family: monospace;
  }

  .match-groups ol {
    margin: 0.5rem 0;
    padding-left: 1.5rem;
  }

  .regex-cheatsheet {
    margin-top: 2rem;
    border: 1px solid #e5e7eb;
    border-radius: var(--border-radius);
    padding: 1rem;
    background: var(--card-bg);
  }

  .cheatsheet-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
    gap: 0.75rem;
    margin-top: 0.75rem;
  }

  .cheatsheet-item {
    display: flex;
    gap: 0.75rem;
    align-items: center;
  }

  .cheatsheet-pattern {
    font-family: monospace;
    font-weight: bold;
    background: var(--bg-color);
    padding: 0.25rem 0.5rem;
    border-radius: 4px;
    width: 60px;
    text-align: center;
    color: var(--primary-color);
  }

  .cheatsheet-desc {
    font-size: 14px;
  }
  
  .label-with-actions {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 0.5rem;
    position: relative;
  }

  .select-pattern-btn {
    padding: 0.3rem 0.75rem;
    background: transparent;
    border: 1px solid #e5e7eb;
    border-radius: var(--border-radius);
    font-size: 0.85rem;
    cursor: pointer;
    color: var(--text-color);
    transition: all 0.2s ease;
  }

  .select-pattern-btn:hover {
    border-color: var(--primary-light);
    color: var(--primary-color);
  }

  .pattern-dropdown {
    position: absolute;
    top: 100%;
    right: 0;
    width: 300px;
    max-height: 400px;
    overflow-y: auto;
    background: var(--card-bg);
    border: 1px solid #e5e7eb;
    border-radius: var(--border-radius);
    box-shadow: var(--shadow);
    z-index: 10;
  }

  .pattern-item {
    padding: 0.75rem;
    border-bottom: 1px solid #e5e7eb;
    cursor: pointer;
    transition: background 0.2s ease;
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
  }

  .pattern-item:last-child {
    border-bottom: none;
  }

  .pattern-item:hover {
    background: rgba(99, 102, 241, 0.1);
  }

  .pattern-item strong {
    color: var(--primary-color);
    font-size: 0.9rem;
  }

  .pattern-item span {
    font-size: 0.8rem;
    color: #6b7280;
  }

  @media (prefers-color-scheme: dark) {
    :root {
      --bg-color: #1f2937;
      --text-color: #f3f4f6;
      --card-bg: #111827;
      --error-color: #ef4444;
    }

    .match-highlight {
      background-color: #854d0e;
      color: #fef08a;
    }
  }
</style> 