<script lang="ts">
  import CryptoJS from 'crypto-js';

  let inputText = $state("");
  let inputType = $state<"text" | "file">("text");
  let selectedFile = $state<File | null>(null);
  let fileContent = $state<ArrayBuffer | null>(null);
  let computing = $state(false);
  let copiedHash = $state<string | null>(null);

  interface HashResult {
    name: string;
    value: string;
    description: string;
  }

  let hashResults = $state<HashResult[]>([]);

  function computeHashes() {
    if (!inputText && !fileContent) {
      hashResults = [];
      return;
    }

    computing = true;

    setTimeout(() => {
      try {
        let data: string | unknown;
        
        if (inputType === "file" && fileContent) {
          // 将 ArrayBuffer 转换为 WordArray
          const wordArray = CryptoJS.lib.WordArray.create(fileContent);
          data = wordArray;
        } else {
          data = inputText;
        }

        hashResults = [
          {
            name: "MD5",
            value: CryptoJS.MD5(data).toString(),
            description: "128位摘要，常用于文件校验"
          },
          {
            name: "SHA-1",
            value: CryptoJS.SHA1(data).toString(),
            description: "160位摘要，已不推荐用于安全场景"
          },
          {
            name: "SHA-256",
            value: CryptoJS.SHA256(data).toString(),
            description: "256位摘要，广泛应用于安全场景"
          },
          {
            name: "SHA-384",
            value: CryptoJS.SHA384(data).toString(),
            description: "384位摘要，SHA-2系列"
          },
          {
            name: "SHA-512",
            value: CryptoJS.SHA512(data).toString(),
            description: "512位摘要，最高安全性"
          },
          {
            name: "SHA3-256",
            value: CryptoJS.SHA3(data, { outputLength: 256 }).toString(),
            description: "SHA-3标准，256位输出"
          },
          {
            name: "RIPEMD-160",
            value: CryptoJS.RIPEMD160(data).toString(),
            description: "160位摘要，用于比特币地址"
          }
        ];
      } catch (err) {
        console.error('计算Hash失败:', err);
        hashResults = [];
      } finally {
        computing = false;
      }
    }, 10);
  }

  function handleFileSelect(event: Event) {
    const input = event.target as HTMLInputElement;
    const file = input.files?.[0];
    
    if (file) {
      selectedFile = file;
      const reader = new FileReader();
      reader.onload = (e) => {
        fileContent = e.target?.result as ArrayBuffer;
        computeHashes();
      };
      reader.readAsArrayBuffer(file);
    }
  }

  async function copyHash(hash: string, name: string) {
    try {
      await navigator.clipboard.writeText(hash);
      copiedHash = name;
      setTimeout(() => {
        copiedHash = null;
      }, 1500);
    } catch (err) {
      console.error('复制失败:', err);
    }
  }

  function clearAll() {
    inputText = "";
    selectedFile = null;
    fileContent = null;
    hashResults = [];
  }

  // 响应输入变化
  $effect(() => {
    if (inputType === "text" && inputText) {
      computeHashes();
    }
  });
</script>

<div class="hash-tool">
  <div class="input-section">
    <div class="input-tabs">
      <button 
        class:active={inputType === "text"}
        onclick={() => { inputType = "text"; clearAll(); }}
      >
        文本输入
      </button>
      <button 
        class:active={inputType === "file"}
        onclick={() => { inputType = "file"; clearAll(); }}
      >
        文件输入
      </button>
    </div>

    {#if inputType === "text"}
      <div class="text-input-area">
        <textarea 
          bind:value={inputText}
          placeholder="输入要计算Hash的文本..."
          rows="6"
        ></textarea>
        <div class="input-actions">
          <span class="char-count">{inputText.length} 字符</span>
          {#if inputText}
            <button class="clear-btn" onclick={clearAll}>清空</button>
          {/if}
        </div>
      </div>
    {:else}
      <div class="file-input-area">
        <div 
          class="file-drop-zone"
          class:has-file={selectedFile}
        >
          {#if selectedFile}
            <div class="file-info">
              <span class="file-icon">📄</span>
              <div class="file-details">
                <span class="file-name">{selectedFile.name}</span>
                <span class="file-size">{(selectedFile.size / 1024).toFixed(2)} KB</span>
              </div>
              <button class="remove-file-btn" onclick={clearAll}>✕</button>
            </div>
          {:else}
            <div class="drop-hint">
              <span class="drop-icon">📁</span>
              <span>点击选择文件或拖放文件到此处</span>
            </div>
          {/if}
          <input 
            type="file" 
            onchange={handleFileSelect}
            class="file-input"
          />
        </div>
      </div>
    {/if}
  </div>

  <div class="results-section">
    <div class="results-header">
      <h3>Hash 结果</h3>
      {#if computing}
        <span class="computing-badge">计算中...</span>
      {/if}
    </div>

    {#if hashResults.length > 0}
      <div class="hash-list">
        {#each hashResults as result}
          <div class="hash-item">
            <div class="hash-header">
              <span class="hash-name">{result.name}</span>
              <span class="hash-desc">{result.description}</span>
            </div>
            <div class="hash-value-row">
              <code class="hash-value">{result.value}</code>
              <button 
                class="copy-btn"
                onclick={() => copyHash(result.value, result.name)}
              >
                {copiedHash === result.name ? '✓ 已复制' : '复制'}
              </button>
            </div>
          </div>
        {/each}
      </div>
    {:else}
      <div class="empty-state">
        <span class="empty-icon">🔐</span>
        <span>输入文本或选择文件后显示Hash结果</span>
      </div>
    {/if}
  </div>
</div>

<style>
  .hash-tool {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }

  .input-section {
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    overflow: hidden;
  }

  .input-tabs {
    display: flex;
    background: var(--bg-dark);
    border-bottom: 1px solid var(--border);
  }

  .input-tabs button {
    flex: 1;
    padding: 0.75rem !important;
    background: transparent !important;
    border: none !important;
    border-bottom: 2px solid transparent !important;
    color: var(--text-secondary) !important;
    border-radius: 0 !important;
    font-weight: 500;
  }

  .input-tabs button.active {
    color: var(--accent) !important;
    border-bottom-color: var(--accent) !important;
    background: var(--bg-hover) !important;
  }

  .input-tabs button:hover {
    background: var(--bg-hover) !important;
    box-shadow: none !important;
    transform: none !important;
  }

  .text-input-area {
    padding: 1rem;
  }

  .text-input-area textarea {
    width: 100%;
    min-height: 150px;
    resize: vertical;
    font-family: 'JetBrains Mono', monospace;
  }

  .input-actions {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-top: 0.5rem;
    padding-top: 0.5rem;
    border-top: 1px solid var(--border);
  }

  .char-count {
    font-size: 0.85rem;
    color: var(--text-muted);
  }

  .clear-btn {
    padding: 0.4rem 0.75rem !important;
    font-size: 0.85rem !important;
    background: var(--bg-dark) !important;
    border: 1px solid var(--border) !important;
  }

  .file-input-area {
    padding: 1rem;
  }

  .file-drop-zone {
    position: relative;
    border: 2px dashed var(--border);
    border-radius: var(--radius-sm);
    padding: 2rem;
    text-align: center;
    transition: all 0.2s;
    cursor: pointer;
  }

  .file-drop-zone:hover {
    border-color: var(--primary);
    background: var(--bg-hover);
  }

  .file-drop-zone.has-file {
    border-style: solid;
    border-color: var(--accent-green);
  }

  .file-input {
    position: absolute;
    inset: 0;
    opacity: 0;
    cursor: pointer;
  }

  .drop-hint {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.75rem;
    color: var(--text-muted);
  }

  .drop-icon {
    font-size: 2rem;
  }

  .file-info {
    display: flex;
    align-items: center;
    gap: 1rem;
    text-align: left;
  }

  .file-icon {
    font-size: 2rem;
  }

  .file-details {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .file-name {
    font-weight: 500;
    color: var(--text-primary);
  }

  .file-size {
    font-size: 0.85rem;
    color: var(--text-muted);
  }

  .remove-file-btn {
    margin-left: auto;
    width: 32px !important;
    height: 32px !important;
    padding: 0 !important;
    background: var(--bg-dark) !important;
    border: 1px solid var(--border) !important;
    border-radius: 50% !important;
    position: relative;
    z-index: 10;
  }

  .results-section {
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    overflow: hidden;
  }

  .results-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.75rem 1rem;
    background: var(--bg-dark);
    border-bottom: 1px solid var(--border);
  }

  .results-header h3 {
    font-size: 0.95rem;
    font-weight: 500;
  }

  .computing-badge {
    font-size: 0.8rem;
    padding: 0.25rem 0.75rem;
    background: var(--primary);
    color: white;
    border-radius: 100px;
    animation: pulse 1s infinite;
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.7; }
  }

  .hash-list {
    max-height: 500px;
    overflow-y: auto;
  }

  .hash-item {
    padding: 1rem;
    border-bottom: 1px solid var(--border);
  }

  .hash-item:last-child {
    border-bottom: none;
  }

  .hash-header {
    display: flex;
    align-items: baseline;
    gap: 0.75rem;
    margin-bottom: 0.5rem;
  }

  .hash-name {
    font-weight: 600;
    color: var(--accent);
    font-size: 0.9rem;
  }

  .hash-desc {
    font-size: 0.8rem;
    color: var(--text-muted);
  }

  .hash-value-row {
    display: flex;
    gap: 0.75rem;
    align-items: center;
  }

  .hash-value {
    flex: 1;
    font-family: 'JetBrains Mono', monospace;
    font-size: 0.85rem;
    padding: 0.5rem 0.75rem !important;
    background: var(--bg-dark) !important;
    border-radius: var(--radius-sm);
    word-break: break-all;
    color: var(--text-primary);
  }

  .copy-btn {
    padding: 0.5rem 1rem !important;
    font-size: 0.85rem !important;
    background: var(--bg-hover) !important;
    border: 1px solid var(--border) !important;
    white-space: nowrap;
  }

  .copy-btn:hover {
    border-color: var(--primary) !important;
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 3rem;
    gap: 0.75rem;
    color: var(--text-muted);
  }

  .empty-icon {
    font-size: 2.5rem;
  }
</style>
