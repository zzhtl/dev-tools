<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  let jsonInput = $state("");
  let jsonOutput = $state("");
  let goStructOutput = $state("");
  let errorMessage = $state("");
  let isProcessing = $state(false);
  let activeMode = $state("format");
  let copied = $state(false);

  const modes = [
    { id: "format", name: "格式化", icon: "{ }", desc: "美化JSON格式" },
    { id: "compress", name: "压缩", icon: "→", desc: "压缩为单行" },
    { id: "goStruct", name: "转Go结构体", icon: "Go", desc: "转换为Go语言结构体" },
    { id: "escape", name: "转义/反转义", icon: "\\", desc: "JSON字符串转义处理" },
  ];

  async function handleModeClick(mode: string) {
    if (isProcessing) return;
    activeMode = mode;
    if (jsonInput) {
      await processJson();
    }
  }

  async function processJson() {
    if (isProcessing || !jsonInput) return;
    isProcessing = true;
    errorMessage = "";
    
    try {
      if (activeMode === "format") {
        const parsed = JSON.parse(jsonInput);
        jsonOutput = JSON.stringify(parsed, null, 2);
        goStructOutput = "";
      } else if (activeMode === "compress") {
        const parsed = JSON.parse(jsonInput);
        jsonOutput = JSON.stringify(parsed);
        goStructOutput = "";
      } else if (activeMode === "goStruct") {
        // 先验证JSON格式
        JSON.parse(jsonInput);
        goStructOutput = await invoke("json_to_go_struct", { json: jsonInput });
        jsonOutput = "";
      } else if (activeMode === "escape") {
        // 检测输入是否是转义后的JSON字符串
        if (jsonInput.startsWith('"') && jsonInput.endsWith('"')) {
          // 反转义
          try {
            const unescaped = JSON.parse(jsonInput);
            if (typeof unescaped === "string") {
              jsonOutput = unescaped;
            } else {
              jsonOutput = JSON.stringify(unescaped, null, 2);
            }
          } catch {
            // 如果解析失败，尝试直接格式化
            const parsed = JSON.parse(jsonInput);
            jsonOutput = JSON.stringify(parsed, null, 2);
          }
        } else {
          // 转义为JSON字符串
          jsonOutput = JSON.stringify(jsonInput);
        }
        goStructOutput = "";
      }
    } catch (e: any) {
      errorMessage = `JSON 解析错误: ${e.message || e}`;
    } finally {
      isProcessing = false;
    }
  }

  async function copyOutput() {
    const text = activeMode === "goStruct" ? goStructOutput : jsonOutput;
    if (!text) return;
    
    try {
      await navigator.clipboard.writeText(text);
      copied = true;
      setTimeout(() => copied = false, 1500);
    } catch (err) {
      console.error("复制失败:", err);
    }
  }

  function clearAll() {
    jsonInput = "";
    jsonOutput = "";
    goStructOutput = "";
    errorMessage = "";
  }

  function loadExample() {
    jsonInput = `{
  "name": "开发工具箱",
  "version": "1.0.0",
  "features": ["JSON格式化", "代码转换", "加密解密"],
  "author": {
    "name": "Developer",
    "email": "dev@example.com"
  },
  "settings": {
    "theme": "dark",
    "language": "zh-CN",
    "autoSave": true
  }
}`;
    processJson();
  }

  // 实时处理
  $effect(() => {
    if (jsonInput && activeMode) {
      const timer = setTimeout(() => {
        processJson();
      }, 300);
      return () => clearTimeout(timer);
    }
  });
</script>

<div class="json-tool">
  <div class="mode-selector">
    {#each modes as mode}
      <button
        class="mode-btn"
        class:active={activeMode === mode.id}
        onclick={() => handleModeClick(mode.id)}
        disabled={isProcessing}
        title={mode.desc}
      >
        <span class="mode-icon">{mode.icon}</span>
        <span class="mode-name">{mode.name}</span>
      </button>
    {/each}
  </div>

  <div class="editor-container">
    <div class="editor-panel input-panel">
      <div class="panel-header">
        <span>JSON 输入</span>
        <div class="header-actions">
          <span class="char-count">{jsonInput.length} 字符</span>
          <button class="action-btn" onclick={loadExample}>示例</button>
          <button class="action-btn" onclick={clearAll}>清空</button>
        </div>
      </div>
      <textarea
        bind:value={jsonInput}
        placeholder="在此粘贴或输入JSON..."
        class="code-editor"
        spellcheck="false"
      ></textarea>
    </div>

    <div class="editor-panel output-panel">
      <div class="panel-header">
        <span>{activeMode === 'goStruct' ? 'Go 结构体输出' : 'JSON 输出'}</span>
        <div class="header-actions">
          <span class="char-count">
            {(activeMode === 'goStruct' ? goStructOutput : jsonOutput).length} 字符
          </span>
          {#if (activeMode === 'goStruct' ? goStructOutput : jsonOutput)}
            <button class="action-btn copy-btn" onclick={copyOutput}>
              {copied ? '✓ 已复制' : '复制'}
            </button>
          {/if}
        </div>
      </div>
      <textarea
        value={activeMode === 'goStruct' ? goStructOutput : jsonOutput}
        placeholder={isProcessing ? '处理中...' : '处理结果将显示在这里...'}
        class="code-editor"
        class:go-output={activeMode === 'goStruct'}
        readonly
        spellcheck="false"
      ></textarea>
    </div>
  </div>

  {#if errorMessage}
    <div class="error-message">
      <span class="error-icon">⚠️</span>
      <span>{errorMessage}</span>
    </div>
  {/if}

  <div class="tips-section">
    <h3>💡 使用提示</h3>
    <ul>
      <li><strong>格式化</strong>：将压缩的JSON美化为易读格式</li>
      <li><strong>压缩</strong>：将格式化的JSON压缩为单行，减小体积</li>
      <li><strong>转Go结构体</strong>：将JSON转换为Go语言的struct定义，支持嵌套结构</li>
      <li><strong>转义/反转义</strong>：在JSON字符串和原始文本之间转换</li>
    </ul>
  </div>
</div>

<style>
  .json-tool {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    height: 100%;
  }

  .mode-selector {
    display: flex;
    gap: 0.5rem;
    padding: 0.5rem;
    background: var(--bg-dark);
    border-radius: var(--radius-sm);
    border: 1px solid var(--border);
  }

  .mode-btn {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.65rem 1rem !important;
    background: transparent !important;
    border: 1px solid transparent !important;
    color: var(--text-secondary) !important;
    transition: all 0.2s;
  }

  .mode-btn:hover {
    background: var(--bg-hover) !important;
    color: var(--text-primary) !important;
    transform: none !important;
    box-shadow: none !important;
  }

  .mode-btn.active {
    background: var(--primary) !important;
    color: white !important;
    border-color: var(--primary) !important;
  }

  .mode-icon {
    font-family: 'JetBrains Mono', monospace;
    font-size: 0.85rem;
    font-weight: 600;
  }

  .mode-name {
    font-weight: 500;
  }

  .editor-container {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 1rem;
    flex: 1;
    min-height: 400px;
  }

  .editor-panel {
    display: flex;
    flex-direction: column;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    overflow: hidden;
  }

  .panel-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.75rem 1rem;
    background: var(--bg-dark);
    border-bottom: 1px solid var(--border);
    font-size: 0.85rem;
    font-weight: 500;
    color: var(--text-secondary);
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .char-count {
    font-size: 0.8rem;
    color: var(--text-muted);
  }

  .action-btn {
    padding: 0.35rem 0.75rem !important;
    font-size: 0.8rem !important;
    background: var(--bg-hover) !important;
    border: 1px solid var(--border) !important;
  }

  .action-btn:hover {
    border-color: var(--primary) !important;
    transform: none !important;
    box-shadow: none !important;
  }

  .copy-btn {
    min-width: 70px;
  }

  .code-editor {
    flex: 1;
    width: 100%;
    padding: 1rem;
    border: none !important;
    border-radius: 0 !important;
    background: var(--bg-card) !important;
    color: var(--text-primary);
    font-family: 'JetBrains Mono', monospace;
    font-size: 0.9rem;
    line-height: 1.6;
    resize: none;
    tab-size: 2;
  }

  .code-editor:focus {
    box-shadow: none !important;
  }

  .output-panel .code-editor {
    background: var(--bg-dark) !important;
    color: var(--accent);
  }

  .output-panel .code-editor.go-output {
    color: var(--accent-green);
  }

  .error-message {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.75rem 1rem;
    background: rgba(239, 68, 68, 0.1);
    border: 1px solid rgba(239, 68, 68, 0.3);
    border-radius: var(--radius-sm);
    color: #f87171;
    font-size: 0.9rem;
  }

  .error-icon {
    font-size: 1.1rem;
  }

  .tips-section {
    padding: 1rem;
    background: var(--bg-dark);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
  }

  .tips-section h3 {
    font-size: 0.9rem;
    margin-bottom: 0.75rem;
    color: var(--text-secondary);
  }

  .tips-section ul {
    list-style: none;
    padding: 0;
    margin: 0;
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
    gap: 0.5rem;
  }

  .tips-section li {
    font-size: 0.85rem;
    color: var(--text-muted);
    padding: 0.25rem 0;
  }

  .tips-section li strong {
    color: var(--accent);
  }

  @media (max-width: 900px) {
    .editor-container {
      grid-template-columns: 1fr;
    }

    .mode-selector {
      flex-wrap: wrap;
    }
  }
</style>
