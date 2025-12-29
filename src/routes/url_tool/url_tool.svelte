<script lang="ts">
  let inputText = $state("");
  let outputText = $state("");
  let mode = $state<"encode" | "decode">("encode");
  let encodeType = $state<"standard" | "component">("component");
  let errorMessage = $state("");
  let copied = $state(false);

  function processUrl() {
    errorMessage = "";
    
    if (!inputText.trim()) {
      outputText = "";
      return;
    }

    try {
      if (mode === "encode") {
        if (encodeType === "standard") {
          outputText = encodeURI(inputText);
        } else {
          outputText = encodeURIComponent(inputText);
        }
      } else {
        try {
          outputText = decodeURIComponent(inputText);
        } catch {
          // 尝试 decodeURI
          outputText = decodeURI(inputText);
        }
      }
    } catch (err: any) {
      errorMessage = `处理失败: ${err.message}`;
      outputText = "";
    }
  }

  async function copyOutput() {
    if (!outputText) return;
    
    try {
      await navigator.clipboard.writeText(outputText);
      copied = true;
      setTimeout(() => {
        copied = false;
      }, 1500);
    } catch (err) {
      console.error('复制失败:', err);
    }
  }

  function swapInputOutput() {
    const temp = inputText;
    inputText = outputText;
    outputText = temp;
    mode = mode === "encode" ? "decode" : "encode";
  }

  function clearAll() {
    inputText = "";
    outputText = "";
    errorMessage = "";
  }

  function loadExample() {
    inputText = "https://example.com/search?q=中文测试&page=1&filter=特殊字符!@#$%";
    processUrl();
  }

  // 响应输入变化
  $effect(() => {
    if (inputText !== undefined) {
      processUrl();
    }
  });
</script>

<div class="url-tool">
  <div class="controls">
    <div class="mode-selector">
      <button 
        class:active={mode === "encode"}
        onclick={() => { mode = "encode"; processUrl(); }}
      >
        编码 (Encode)
      </button>
      <button 
        class:active={mode === "decode"}
        onclick={() => { mode = "decode"; processUrl(); }}
      >
        解码 (Decode)
      </button>
    </div>

    {#if mode === "encode"}
      <div class="encode-type">
        <label class="radio-label">
          <input type="radio" bind:group={encodeType} value="component" onchange={processUrl} />
          <span>encodeURIComponent</span>
          <span class="hint">(推荐，编码所有特殊字符)</span>
        </label>
        <label class="radio-label">
          <input type="radio" bind:group={encodeType} value="standard" onchange={processUrl} />
          <span>encodeURI</span>
          <span class="hint">(保留URL结构字符)</span>
        </label>
      </div>
    {/if}

    <div class="action-buttons">
      <button class="secondary-btn" onclick={loadExample}>加载示例</button>
      <button class="secondary-btn" onclick={clearAll}>清空</button>
    </div>
  </div>

  <div class="converter-area">
    <div class="input-panel">
      <div class="panel-header">
        <span>输入 ({mode === "encode" ? "原文" : "编码后"})</span>
        <span class="char-count">{inputText.length} 字符</span>
      </div>
      <textarea 
        bind:value={inputText}
        placeholder={mode === "encode" ? "输入要编码的URL或文本..." : "输入要解码的URL..."}
        rows="8"
      ></textarea>
    </div>

    <button class="swap-btn" onclick={swapInputOutput} title="交换输入输出">
      ⇄
    </button>

    <div class="output-panel">
      <div class="panel-header">
        <span>输出 ({mode === "encode" ? "编码后" : "解码后"})</span>
        <div class="output-actions">
          <span class="char-count">{outputText.length} 字符</span>
          {#if outputText}
            <button class="copy-inline-btn" onclick={copyOutput}>
              {copied ? '✓ 已复制' : '复制'}
            </button>
          {/if}
        </div>
      </div>
      <textarea 
        bind:value={outputText}
        placeholder="处理结果将显示在这里..."
        rows="8"
        readonly
      ></textarea>
    </div>
  </div>

  {#if errorMessage}
    <div class="error-message">
      ⚠️ {errorMessage}
    </div>
  {/if}

  <div class="info-section">
    <h3>URL 编码说明</h3>
    <div class="info-grid">
      <div class="info-card">
        <h4>encodeURIComponent</h4>
        <p>编码所有非字母数字字符，适合编码URL参数值。会编码 <code>= & ? /</code> 等字符。</p>
        <div class="example">
          <span>示例：</span>
          <code>你好 → %E4%BD%A0%E5%A5%BD</code>
        </div>
      </div>
      <div class="info-card">
        <h4>encodeURI</h4>
        <p>保留URL结构字符不编码，适合编码完整URL。不会编码 <code>: / ? # [ ] @</code> 等。</p>
        <div class="example">
          <span>示例：</span>
          <code>https://a.com/你好 → https://a.com/%E4%BD%A0%E5%A5%BD</code>
        </div>
      </div>
    </div>

    <div class="common-encodings">
      <h4>常见编码对照</h4>
      <div class="encoding-table">
        <div class="encoding-row">
          <span class="original">空格</span>
          <span class="encoded">%20 或 +</span>
        </div>
        <div class="encoding-row">
          <span class="original">!</span>
          <span class="encoded">%21</span>
        </div>
        <div class="encoding-row">
          <span class="original">#</span>
          <span class="encoded">%23</span>
        </div>
        <div class="encoding-row">
          <span class="original">$</span>
          <span class="encoded">%24</span>
        </div>
        <div class="encoding-row">
          <span class="original">&</span>
          <span class="encoded">%26</span>
        </div>
        <div class="encoding-row">
          <span class="original">=</span>
          <span class="encoded">%3D</span>
        </div>
        <div class="encoding-row">
          <span class="original">?</span>
          <span class="encoded">%3F</span>
        </div>
        <div class="encoding-row">
          <span class="original">@</span>
          <span class="encoded">%40</span>
        </div>
      </div>
    </div>
  </div>
</div>

<style>
  .url-tool {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }

  .controls {
    display: flex;
    flex-wrap: wrap;
    gap: 1.5rem;
    align-items: center;
    padding: 1rem;
    background: var(--bg-dark);
    border-radius: var(--radius-sm);
    border: 1px solid var(--border);
  }

  .mode-selector {
    display: flex;
    background: var(--bg-card);
    border-radius: var(--radius-sm);
    border: 1px solid var(--border);
    overflow: hidden;
  }

  .mode-selector button {
    padding: 0.65rem 1.25rem !important;
    background: transparent !important;
    border: none !important;
    border-radius: 0 !important;
    color: var(--text-secondary) !important;
    font-weight: 500;
  }

  .mode-selector button.active {
    background: var(--primary) !important;
    color: white !important;
  }

  .mode-selector button:hover:not(.active) {
    background: var(--bg-hover) !important;
    box-shadow: none !important;
    transform: none !important;
  }

  .encode-type {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .radio-label {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    cursor: pointer;
    font-size: 0.9rem;
  }

  .radio-label input {
    accent-color: var(--primary);
    width: 16px;
    height: 16px;
  }

  .radio-label .hint {
    font-size: 0.8rem;
    color: var(--text-muted);
  }

  .action-buttons {
    display: flex;
    gap: 0.5rem;
    margin-left: auto;
  }

  .secondary-btn {
    padding: 0.5rem 1rem !important;
    background: var(--bg-hover) !important;
    border: 1px solid var(--border) !important;
    font-size: 0.85rem !important;
  }

  .converter-area {
    display: grid;
    grid-template-columns: 1fr auto 1fr;
    gap: 1rem;
    align-items: stretch;
  }

  .input-panel, .output-panel {
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }

  .panel-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.75rem 1rem;
    background: var(--bg-dark);
    border-bottom: 1px solid var(--border);
    font-size: 0.85rem;
    color: var(--text-secondary);
  }

  .output-actions {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .char-count {
    font-size: 0.8rem;
    color: var(--text-muted);
  }

  .copy-inline-btn {
    padding: 0.3rem 0.75rem !important;
    font-size: 0.8rem !important;
    background: var(--bg-hover) !important;
    border: 1px solid var(--border) !important;
  }

  .input-panel textarea,
  .output-panel textarea {
    flex: 1;
    border: none !important;
    border-radius: 0 !important;
    resize: none;
    font-family: 'JetBrains Mono', monospace;
    min-height: 200px;
  }

  .output-panel textarea {
    background: var(--bg-dark) !important;
    color: var(--accent);
  }

  .swap-btn {
    align-self: center;
    width: 44px !important;
    height: 44px !important;
    padding: 0 !important;
    border-radius: 50% !important;
    background: var(--bg-dark) !important;
    border: 1px solid var(--border) !important;
    font-size: 1.25rem;
  }

  .swap-btn:hover {
    background: var(--bg-hover) !important;
    border-color: var(--primary) !important;
  }

  .error-message {
    padding: 0.75rem 1rem;
    background: rgba(239, 68, 68, 0.1);
    border: 1px solid rgba(239, 68, 68, 0.3);
    border-radius: var(--radius-sm);
    color: #f87171;
    font-size: 0.9rem;
  }

  .info-section {
    margin-top: 1rem;
  }

  .info-section h3 {
    font-size: 1rem;
    margin-bottom: 1rem;
  }

  .info-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
    gap: 1rem;
    margin-bottom: 1.5rem;
  }

  .info-card {
    padding: 1rem;
    background: var(--bg-dark);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
  }

  .info-card h4 {
    font-size: 0.9rem;
    color: var(--accent);
    margin-bottom: 0.5rem;
  }

  .info-card p {
    font-size: 0.85rem;
    line-height: 1.5;
    margin-bottom: 0.75rem;
  }

  .info-card code {
    font-size: 0.8rem;
    background: var(--bg-card) !important;
  }

  .example {
    font-size: 0.85rem;
    padding-top: 0.5rem;
    border-top: 1px solid var(--border);
  }

  .example span {
    color: var(--text-muted);
    margin-right: 0.5rem;
  }

  .common-encodings h4 {
    font-size: 0.9rem;
    margin-bottom: 0.75rem;
    color: var(--text-secondary);
  }

  .encoding-table {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(120px, 1fr));
    gap: 0.5rem;
  }

  .encoding-row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 0.75rem;
    background: var(--bg-dark);
    border-radius: var(--radius-sm);
    font-family: 'JetBrains Mono', monospace;
    font-size: 0.85rem;
  }

  .original {
    color: var(--text-primary);
    min-width: 30px;
  }

  .encoded {
    color: var(--accent);
  }

  @media (max-width: 768px) {
    .converter-area {
      grid-template-columns: 1fr;
    }

    .swap-btn {
      justify-self: center;
      transform: rotate(90deg);
    }
  }
</style>

