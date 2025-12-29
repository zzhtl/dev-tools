<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  
  let domain = $state('');
  let dnsResults = $state<{ type: string; records: string[] }[]>([]);
  let isLoading = $state(false);
  let error = $state('');

  const dnsTypes = [
    { value: 'A', label: 'A', desc: 'IPv4地址' },
    { value: 'AAAA', label: 'AAAA', desc: 'IPv6地址' },
    { value: 'MX', label: 'MX', desc: '邮件服务器' },
    { value: 'CNAME', label: 'CNAME', desc: '别名记录' },
    { value: 'TXT', label: 'TXT', desc: '文本记录' },
    { value: 'NS', label: 'NS', desc: '域名服务器' }
  ];
  let selectedTypes = $state(['A', 'MX']);

  function cleanDomain(input: string): string {
    let d = input.trim();
    d = d.replace(/^https?:\/\//i, '');
    d = d.replace(/^www\./i, '');
    d = d.replace(/@/, '');
    d = d.replace(/\/.*/, '');
    d = d.replace(/:.*/, '');
    return d;
  }

  function toggleType(type: string) {
    if (selectedTypes.includes(type)) {
      selectedTypes = selectedTypes.filter(t => t !== type);
    } else {
      selectedTypes = [...selectedTypes, type];
    }
  }

  function selectAllTypes() {
    selectedTypes = dnsTypes.map(t => t.value);
  }

  const queryDNS = async () => {
    const cleanedDomain = cleanDomain(domain);
    if (!cleanedDomain || !/^([a-zA-Z0-9-]+\.)+[a-zA-Z]{2,}$/.test(cleanedDomain)) {
      error = '请输入有效域名 (如: example.com)';
      return;
    }

    if (selectedTypes.length === 0) {
      error = '请至少选择一种查询类型';
      return;
    }

    isLoading = true;
    error = '';
    dnsResults = [];

    try {
      const result = await invoke<{ record_type: string; records: string[] }[]>(
        'resolve_dns',
        { domain: cleanedDomain, types: selectedTypes }
      );
      if (result.length === 0) {
        error = '未查询到DNS记录';
      } else {
        dnsResults = result.map(r => ({ type: r.record_type, records: r.records }));
      }
    } catch (e) {
      error = `查询失败: ${String(e)}`;
    } finally {
      isLoading = false;
    }
  };

  async function copyRecord(record: string) {
    try {
      await navigator.clipboard.writeText(record);
    } catch (err) {
      console.error('复制失败:', err);
    }
  }
</script>

<div class="dns-tool">
  <div class="search-section">
    <div class="input-row">
      <div class="domain-input-wrapper">
        <span class="input-prefix">🌐</span>
        <input
          type="text"
          bind:value={domain}
          placeholder="输入域名或URL (如: example.com 或 https://example.com)"
          onkeydown={(e) => e.key === 'Enter' && queryDNS()}
        />
      </div>
      <button class="query-btn" onclick={queryDNS} disabled={isLoading}>
        {isLoading ? '查询中...' : '🔍 解析DNS'}
      </button>
    </div>

    <div class="type-selector">
      <div class="type-header">
        <span>选择查询类型</span>
        <button class="select-all-btn" onclick={selectAllTypes}>全选</button>
      </div>
      <div class="type-grid">
        {#each dnsTypes as type}
          <button
            class="type-btn"
            class:active={selectedTypes.includes(type.value)}
            onclick={() => toggleType(type.value)}
          >
            <span class="type-value">{type.label}</span>
            <span class="type-desc">{type.desc}</span>
          </button>
        {/each}
      </div>
    </div>
  </div>

  {#if error}
    <div class="error-message">
      ⚠️ {error}
    </div>
  {/if}

  {#if dnsResults.length > 0}
    <div class="results-section">
      <div class="results-header">
        <h3>解析结果</h3>
        <span class="domain-badge">{cleanDomain(domain)}</span>
      </div>
      
      <div class="results-grid">
        {#each dnsResults as result}
          <div class="result-card">
            <div class="card-header">
              <span class="record-type">{result.type}</span>
              <span class="record-count">{result.records.length} 条记录</span>
            </div>
            <div class="record-list">
              {#each result.records as record}
                <div class="record-item">
                  <code>{record}</code>
                  <button class="copy-record-btn" onclick={() => copyRecord(record)} title="复制">
                    📋
                  </button>
                </div>
              {/each}
            </div>
          </div>
        {/each}
      </div>
    </div>
  {/if}

  <div class="info-section">
    <h3>DNS记录类型说明</h3>
    <div class="info-grid">
      <div class="info-item">
        <span class="info-type">A</span>
        <span class="info-desc">将域名映射到IPv4地址</span>
      </div>
      <div class="info-item">
        <span class="info-type">AAAA</span>
        <span class="info-desc">将域名映射到IPv6地址</span>
      </div>
      <div class="info-item">
        <span class="info-type">MX</span>
        <span class="info-desc">指定邮件服务器地址和优先级</span>
      </div>
      <div class="info-item">
        <span class="info-type">CNAME</span>
        <span class="info-desc">为域名创建别名</span>
      </div>
      <div class="info-item">
        <span class="info-type">TXT</span>
        <span class="info-desc">存储文本信息，常用于验证</span>
      </div>
      <div class="info-item">
        <span class="info-type">NS</span>
        <span class="info-desc">指定域名的权威DNS服务器</span>
      </div>
    </div>
  </div>
</div>

<style>
  .dns-tool {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }

  .search-section {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    padding: 1.25rem;
    background: var(--bg-dark);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
  }

  .input-row {
    display: flex;
    gap: 0.75rem;
  }

  .domain-input-wrapper {
    flex: 1;
    display: flex;
    align-items: center;
    background: var(--bg-card);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    padding: 0 1rem;
    transition: border-color 0.2s;
  }

  .domain-input-wrapper:focus-within {
    border-color: var(--primary);
  }

  .input-prefix {
    font-size: 1.1rem;
    margin-right: 0.75rem;
  }

  .domain-input-wrapper input {
    flex: 1;
    border: none !important;
    background: transparent !important;
    padding: 0.85rem 0 !important;
  }

  .domain-input-wrapper input:focus {
    box-shadow: none !important;
  }

  .query-btn {
    padding: 0.85rem 1.5rem !important;
    font-weight: 600 !important;
    white-space: nowrap;
  }

  .type-selector {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .type-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 0.85rem;
    color: var(--text-secondary);
  }

  .select-all-btn {
    padding: 0.3rem 0.75rem !important;
    font-size: 0.8rem !important;
    background: var(--bg-hover) !important;
    border: 1px solid var(--border) !important;
  }

  .type-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));
    gap: 0.5rem;
  }

  .type-btn {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 0.2rem;
    padding: 0.75rem !important;
    background: var(--bg-card) !important;
    border: 1px solid var(--border) !important;
    text-align: left;
    cursor: pointer;
  }

  .type-btn:hover {
    border-color: var(--primary) !important;
    transform: none !important;
    box-shadow: none !important;
  }

  .type-btn.active {
    background: rgba(34, 211, 238, 0.1) !important;
    border-color: var(--accent) !important;
  }

  .type-value {
    font-weight: 600;
    font-family: 'JetBrains Mono', monospace;
    color: var(--text-primary);
  }

  .type-desc {
    font-size: 0.75rem;
    color: var(--text-muted);
  }

  .error-message {
    padding: 0.75rem 1rem;
    background: rgba(239, 68, 68, 0.1);
    border: 1px solid rgba(239, 68, 68, 0.3);
    border-radius: var(--radius-sm);
    color: #f87171;
  }

  .results-section {
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    overflow: hidden;
  }

  .results-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1rem;
    background: var(--bg-dark);
    border-bottom: 1px solid var(--border);
  }

  .results-header h3 {
    font-size: 0.95rem;
    margin: 0;
  }

  .domain-badge {
    padding: 0.35rem 0.75rem;
    background: var(--primary);
    color: white;
    border-radius: 100px;
    font-size: 0.85rem;
    font-family: 'JetBrains Mono', monospace;
  }

  .results-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
    gap: 1rem;
    padding: 1rem;
  }

  .result-card {
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    overflow: hidden;
  }

  .card-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.75rem 1rem;
    background: var(--bg-dark);
    border-bottom: 1px solid var(--border);
  }

  .record-type {
    font-weight: 600;
    font-family: 'JetBrains Mono', monospace;
    color: var(--accent);
  }

  .record-count {
    font-size: 0.8rem;
    color: var(--text-muted);
  }

  .record-list {
    padding: 0.5rem;
  }

  .record-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.5rem 0.75rem;
    border-radius: 4px;
    transition: background 0.15s;
  }

  .record-item:hover {
    background: var(--bg-hover);
  }

  .record-item code {
    font-family: 'JetBrains Mono', monospace;
    font-size: 0.85rem;
    color: var(--text-primary);
    background: transparent !important;
    padding: 0 !important;
    word-break: break-all;
  }

  .copy-record-btn {
    padding: 0.25rem 0.5rem !important;
    font-size: 0.8rem !important;
    background: transparent !important;
    border: none !important;
    opacity: 0;
    transition: opacity 0.15s;
  }

  .record-item:hover .copy-record-btn {
    opacity: 1;
  }

  .info-section {
    padding: 1rem;
    background: var(--bg-dark);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
  }

  .info-section h3 {
    font-size: 0.9rem;
    margin-bottom: 0.75rem;
    color: var(--text-secondary);
  }

  .info-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    gap: 0.5rem;
  }

  .info-item {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.5rem;
  }

  .info-type {
    font-family: 'JetBrains Mono', monospace;
    font-weight: 600;
    color: var(--accent);
    min-width: 55px;
  }

  .info-desc {
    font-size: 0.85rem;
    color: var(--text-muted);
  }
</style>
