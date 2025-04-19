<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  
  let domain = '';
  let dnsResults: { type: string; records: string[] }[] = [];
  let isLoading = false;
  let error = '';

  const dnsTypes = [
    { value: 'A', label: 'A (IPv4地址)' },
    { value: 'AAAA', label: 'AAAA (IPv6地址)' },
    { value: 'MX', label: 'MX (邮件服务器)' },
    { value: 'CNAME', label: 'CNAME (别名)' },
    { value: 'TXT', label: 'TXT (文本记录)' },
    { value: 'NS', label: 'NS (域名服务器)' }
  ];
  let selectedTypes = ['A', 'MX'];

  // 域名清洗函数，支持去除协议、路径、@、www等
  function cleanDomain(input: string): string {
    let d = input.trim();
    d = d.replace(/^https?:\/\//i, ''); // 去除协议
    d = d.replace(/^www\./i, '');        // 去除www.
    d = d.replace(/@/, '');               // 去除@符号
    d = d.replace(/\/.*/, '');           // 去除路径
    d = d.replace(/:.*/, '');             // 去除端口
    return d;
  }

  const queryDNS = async () => {
    const cleanedDomain = cleanDomain(domain);
    if (!cleanedDomain || !/^([a-zA-Z0-9-]+\.)+[a-zA-Z]{2,}$/.test(cleanedDomain)) {
      error = '请输入有效域名 (如: example.com)';
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
</script>

<div class="dns-tool">
  <h2>DNS解析工具</h2>
  
  <div class="input-group">
    <input
      type="text"
      bind:value={domain}
      placeholder="输入域名或URL (如: example.com 或 https://example.com)"
      on:keydown={(e) => e.key === 'Enter' && queryDNS()}
    />
    <button on:click={queryDNS} disabled={isLoading}>
      {isLoading ? '查询中...' : '解析DNS'}
    </button>
  </div>

  <div class="type-selector">
    <h3>选择查询类型:</h3>
    {#each dnsTypes as type}
      <label>
        <input
          type="checkbox"
          bind:group={selectedTypes}
          value={type.value}
        />
        {type.label}
      </label>
    {/each}
  </div>

  {#if error}
    <div class="error">{error}</div>
  {/if}

  {#if dnsResults.length > 0}
    <div class="results">
      <h3>解析结果: {cleanDomain(domain)}</h3>
      {#each dnsResults as result}
        <div class="record-type">
          <h4>{result.type} 记录</h4>
          <ul>
            {#each result.records as record}
              <li>{record}</li>
            {/each}
          </ul>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .dns-tool {
    padding: 1rem;
    max-width: 800px;
    margin: 0 auto;
  }

  .input-group {
    display: flex;
    gap: 0.5rem;
    margin-bottom: 1rem;
  }

  input[type="text"] {
    flex: 1;
    padding: 0.5rem;
    border: 1px solid #ccc;
    border-radius: 4px;
  }

  button {
    padding: 0.5rem 1rem;
    background: var(--primary-color);
    color: white;
    border: none;
    border-radius: 4px;
    cursor: pointer;
  }

  button:disabled {
    opacity: 0.7;
    cursor: not-allowed;
  }

  .type-selector {
    margin: 1rem 0;
    padding: 1rem;
    background: #f5f5f5;
    border-radius: 4px;
  }

  .type-selector label {
    display: inline-block;
    margin-right: 1rem;
    cursor: pointer;
  }

  .error {
    color: #e53935;
    padding: 0.5rem;
    background: #ffebee;
    border-radius: 4px;
  }

  .results {
    margin-top: 1rem;
  }

  .record-type {
    margin-bottom: 1rem;
    padding: 1rem;
    background: white;
    border-radius: 4px;
    box-shadow: 0 1px 3px rgba(0,0,0,0.1);
  }

  .record-type h4 {
    margin-top: 0;
    color: var(--primary-color);
  }

  ul {
    margin: 0.5rem 0 0 1rem;
    padding: 0;
  }

  @media (prefers-color-scheme: dark) {
    .type-selector {
      background: #2d2d2d;
    }
    .record-type {
      background: #1e1e1e;
    }
  }
</style>
