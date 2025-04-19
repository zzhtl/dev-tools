<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  let jsonInput = $state("");
  let jsonOutput = $state("");
  let goStructOutput = $state("");
  let errorMessage = $state("");
  let isProcessing = $state(false);
  let activeMode = $state(""); // 初始为空，不选中任何模式

  async function handleModeClick(mode: string) {
    if (isProcessing) return;
    
    // 切换选中状态
    activeMode = activeMode === mode ? "" : mode;
    
    // 如果选中了模式且输入不为空，则执行转换
    if (activeMode === mode && jsonInput) {
      await processJson();
    }
  }

  async function processJson() {
    if (isProcessing || !jsonInput || !activeMode) return;
    isProcessing = true;
    try {
      if (activeMode === "parse") {
        const parsed = await processJsonInChunks(jsonInput, (chunk) => JSON.parse(chunk));
        jsonOutput = JSON.stringify(parsed, null, 2);
      } else if (activeMode === "compress") {
        const parsed = await processJsonInChunks(jsonInput, (chunk) => JSON.parse(chunk));
        jsonOutput = JSON.stringify(parsed);
      } else if (activeMode === "goStruct") {
        goStructOutput = await invoke("json_to_go_struct", { json: jsonInput });
      }
      errorMessage = "";
    } catch (e) {
      errorMessage = "Invalid JSON: " + e;
    } finally {
      isProcessing = false;
    }
  }

  async function processJsonInChunks(json: string, processFn: (chunk: string) => any) {
    const chunkSize = 500000;
    let result = {};
    for (let i = 0; i < json.length; i += chunkSize) {
      const chunk = json.slice(i, i + chunkSize);
      result = { ...result, ...processFn(chunk) };
      await new Promise((resolve) => setTimeout(resolve, 0));
    }
    return result;
  }
</script>

<main class="container">
  <!-- 修改后的顶部功能按钮 -->
  <div class="mode-selector">
    <button 
      class:active={activeMode === "parse"}
      onclick={() => handleModeClick("parse")}
      class="mode-button"
      disabled={isProcessing}
    >
      {isProcessing && activeMode === "parse" ? '格式化中...' : '格式化JSON'}
    </button>
    <button 
      class:active={activeMode === "compress"}
      onclick={() => handleModeClick("compress")}
      class="mode-button"
      disabled={isProcessing}
    >
      {isProcessing && activeMode === "compress" ? '压缩中...' : '压缩JSON'}
    </button>
    <button 
      class:active={activeMode === "goStruct"}
      onclick={() => handleModeClick("goStruct")}
      class="mode-button"
      disabled={isProcessing}
    >
      {isProcessing && activeMode === "goStruct" ? '转换中...' : '转Go结构体'}
    </button>
  </div>

  <!-- 中间输入输出区域 -->
  <div class="io-container">
    <div class="io-column">
      <h3>输入</h3>
      <textarea 
        id="json-input" 
        placeholder="输入JSON..." 
        bind:value={jsonInput} 
        class="io-textarea"
      ></textarea>
    </div>
    <div class="io-column">
      <h3>{activeMode === 'goStruct' ? 'Go结构体输出' : 'JSON输出'}</h3>
      {#if activeMode === 'goStruct'}
        <textarea 
          bind:value={goStructOutput}
          placeholder="Go结构体将在这里显示..."
          readonly
          class="io-textarea"
        ></textarea>
      {:else}
        <textarea 
          bind:value={jsonOutput}
          placeholder="处理后的JSON将在这里显示..."
          readonly
          class="io-textarea"
        ></textarea>
      {/if}
    </div>
  </div>

  {#if errorMessage}
    <p class="error">{errorMessage}</p>
  {/if}
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
    padding: 2rem;
    background: var(--bg-color);
    min-height: 100vh;
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }

  .mode-selector {
    display: flex;
    justify-content: center;
    gap: 1rem;
    margin-bottom: 1rem;
  }

  .mode-button {
    padding: 0.75rem 1.5rem;
    background: transparent;
    color: var(--text-color);
    border: 1px solid #e5e7eb;
    border-radius: var(--border-radius);
    font-weight: 600;
    cursor: pointer;
    transition: all 0.3s ease;
  }

  .mode-button.active {
    background-color: var(--primary-color);
    color: white;
    border-color: var(--primary-color);
  }

  .mode-button:hover {
    border-color: var(--primary-light);
  }

  .io-container {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 2rem;
    flex: 1;
    min-height: 500px;
  }

  .io-column {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .io-column h3 {
    margin: 0;
    font-size: 1.1rem;
    color: var(--text-color);
    font-weight: 600;
  }

  .io-textarea {
    width: 100%;
    height: 100%;
    min-height: 300px;
    padding: 1rem;
    border: 1px solid #e5e7eb;
    border-radius: var(--border-radius);
    font-family: monospace;
    font-size: 0.875rem;
    line-height: 1.5;
    resize: vertical;
    transition: all 0.3s ease;
    background: var(--card-bg);
    box-shadow: var(--shadow);
  }

  .io-textarea:focus {
    outline: none;
    border-color: var(--primary-light);
    box-shadow: 0 0 0 2px rgba(99, 102, 241, 0.2);
  }

  .action-container {
    display: flex;
    justify-content: center;
  }

  .action-button {
    padding: 0.75rem 2rem;
    background-color: var(--primary-color);
    color: white;
    border: none;
    border-radius: var(--border-radius);
    font-weight: 600;
    font-size: 1rem;
    cursor: pointer;
    transition: background-color 0.3s ease;
  }

  .action-button:hover {
    background-color: var(--primary-light);
  }

  .action-button:disabled {
    background-color: #9ca3af;
    cursor: not-allowed;
  }

  .error {
    color: var(--error-color);
    font-weight: 600;
    text-align: center;
    margin-top: 1rem;
  }

  @media (prefers-color-scheme: dark) {
    :root {
      --bg-color: #1f2937;
      --text-color: #f3f4f6;
      --card-bg: #111827;
    }

    .io-textarea {
      background-color: var(--card-bg);
      color: var(--text-color);
      border-color: #374151;
    }

    .mode-button {
      border-color: #374151;
      color: var(--text-color);
    }
  }
</style>
