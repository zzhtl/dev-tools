<script lang="ts">
  import { apiJson } from "../../lib/api";
  import Button from "../../lib/ui/Button.svelte";
  import CopyButton from "../../lib/ui/CopyButton.svelte";
  import Panel from "../../lib/ui/Panel.svelte";
  import TextInput from "../../lib/ui/TextInput.svelte";
  import EmptyState from "../../lib/ui/EmptyState.svelte";

  interface DnsRecord {
    value: string;
    ttl: number;
  }

  interface DnsResult {
    record_type: string;
    records: DnsRecord[];
    error?: string;
  }

  const TYPES = [
    { value: "A", desc: "IPv4 地址" },
    { value: "AAAA", desc: "IPv6 地址" },
    { value: "CNAME", desc: "别名" },
    { value: "MX", desc: "邮件服务器" },
    { value: "TXT", desc: "文本记录" },
    { value: "NS", desc: "域名服务器" },
    { value: "SOA", desc: "起始授权" },
    { value: "SRV", desc: "服务记录" },
    { value: "CAA", desc: "证书颁发授权" },
    { value: "PTR", desc: "反向解析" },
  ];

  const PRESET_SERVERS = [
    { label: "系统默认", value: "" },
    { label: "223.5.5.5 阿里", value: "223.5.5.5" },
    { label: "119.29.29.29 腾讯", value: "119.29.29.29" },
    { label: "1.1.1.1 Cloudflare", value: "1.1.1.1" },
    { label: "8.8.8.8 Google", value: "8.8.8.8" },
  ];

  let domain = $state("");
  let server = $state("");
  let selected = $state<string[]>(["A", "AAAA", "CNAME"]);
  let results = $state<DnsResult[]>([]);
  let loading = $state(false);
  let error = $state("");

  /** 容忍粘贴完整 URL / 带 www / 带端口的输入 */
  function cleanDomain(raw: string): string {
    return raw
      .trim()
      .replace(/^https?:\/\//i, "")
      .replace(/^www\./i, "")
      .replace(/@/, "")
      .replace(/\/.*/, "")
      .replace(/:.*/, "");
  }

  function toggle(type: string) {
    selected = selected.includes(type)
      ? selected.filter((t) => t !== type)
      : [...selected, type];
  }

  async function query() {
    const cleaned = cleanDomain(domain);
    if (!cleaned || !/^([a-zA-Z0-9-]+\.)+[a-zA-Z]{2,}$/.test(cleaned)) {
      error = "请输入有效域名（如 example.com）";
      return;
    }
    if (selected.length === 0) {
      error = "请至少选择一种记录类型";
      return;
    }

    loading = true;
    error = "";
    results = [];
    try {
      results = await apiJson<DnsResult[]>("/api/dns/resolve", {
        domain: cleaned,
        types: selected,
        server: server || undefined,
      });
    } catch (e) {
      error = `查询失败：${String(e)}`;
    } finally {
      loading = false;
    }
  }

  const hasAnyRecord = $derived(results.some((r) => r.records.length > 0));

  // ── 端口连通性探测 ──
  // 常见用法：先解析出域名指向哪台机器，再确认服务端口是否真的通。
  interface PortResult {
    open: boolean;
    latency_ms?: number;
    error?: string;
  }

  const COMMON_PORTS = [
    { port: 80, label: "HTTP" },
    { port: 443, label: "HTTPS" },
    { port: 22, label: "SSH" },
    { port: 3306, label: "MySQL" },
    { port: 5432, label: "PostgreSQL" },
    { port: 6379, label: "Redis" },
  ];

  let portHost = $state("");
  let portText = $state("443");
  let portResult = $state<PortResult | null>(null);
  let portError = $state("");
  let probing = $state(false);

  async function probePort() {
    const host = (portHost || cleanDomain(domain)).trim();
    const port = Number.parseInt(portText, 10);
    portResult = null;
    portError = "";
    if (!host) {
      portError = "请填写主机";
      return;
    }
    if (!Number.isInteger(port) || port < 1 || port > 65535) {
      portError = "端口需要在 1–65535 之间";
      return;
    }
    probing = true;
    try {
      portResult = await apiJson<PortResult>("/api/net/port", { host, port });
    } catch (e) {
      portError = `探测失败：${String(e)}`;
    } finally {
      probing = false;
    }
  }
</script>

<div class="tool">
  <div class="query">
    <div class="domain">
      <TextInput
        bind:value={domain}
        placeholder="example.com"
        ariaLabel="域名"
        onkeydown={(event) => {
          if (event.key === "Enter") query();
        }}
      />
    </div>
    <select bind:value={server} aria-label="DNS 服务器" class="server">
      {#each PRESET_SERVERS as preset (preset.label)}
        <option value={preset.value}>{preset.label}</option>
      {/each}
    </select>
    <Button variant="primary" onclick={query} loading={loading}>查询</Button>
  </div>

  <div class="types">
    {#each TYPES as type (type.value)}
      <label class="type" class:on={selected.includes(type.value)} title={type.desc}>
        <input
          type="checkbox"
          checked={selected.includes(type.value)}
          onchange={() => toggle(type.value)}
        />
        {type.value}
      </label>
    {/each}
  </div>

  {#if error}
    <p class="error">{error}</p>
  {/if}

  <Panel title="解析结果" grow>
    {#if results.length === 0}
      <EmptyState
        icon="◎"
        title={loading ? "查询中…" : "还没有查询"}
        hint="调用本机系统 DNS 或指定的上游服务器，单条查询 5 秒超时"
      />
    {:else}
      <div class="groups">
        {#each results as result (result.record_type)}
          <section class="group">
            <header>
              <span class="type-name">{result.record_type}</span>
              {#if result.error}
                <span class="tag err">{result.error}</span>
              {:else if result.records.length === 0}
                <span class="tag">无记录</span>
              {:else}
                <span class="tag">{result.records.length} 条</span>
              {/if}
            </header>
            {#if result.records.length > 0}
              <ul>
                {#each result.records as record, index (index)}
                  <li>
                    <code>{record.value}</code>
                    <span class="ttl" title="缓存生存时间">TTL {record.ttl}s</span>
                    <CopyButton value={record.value} icon title="复制记录" />
                  </li>
                {/each}
              </ul>
            {/if}
          </section>
        {/each}
      </div>
      {#if !hasAnyRecord}
        <p class="hint">所选类型均无记录，可换用其他记录类型或上游服务器再试。</p>
      {/if}
    {/if}
  </Panel>

  <Panel title="端口连通性">
    {#snippet actions()}
      <Button size="sm" variant="primary" onclick={probePort} loading={probing}>探测</Button>
    {/snippet}

    <div class="probe">
      <div class="probe-host">
        <TextInput
          bind:value={portHost}
          placeholder={cleanDomain(domain) || "主机（留空则用上方域名）"}
          ariaLabel="探测主机"
          onkeydown={(event) => {
            if (event.key === "Enter") probePort();
          }}
        />
      </div>
      <div class="probe-port">
        <TextInput bind:value={portText} type="number" mono ariaLabel="端口" />
      </div>
      {#each COMMON_PORTS as item (item.port)}
        <button class="port-chip" onclick={() => (portText = String(item.port))}>
          {item.port}
          <span>{item.label}</span>
        </button>
      {/each}
    </div>

    {#if portError}
      <p class="error">{portError}</p>
    {:else if portResult}
      <p class="probe-result" class:open={portResult.open}>
        {#if portResult.open}
          ✓ 端口开放，连接耗时 {portResult.latency_ms} ms
        {:else}
          ✕ 无法连接{portResult.error ? `：${portResult.error}` : ""}
        {/if}
      </p>
    {/if}

    <p class="hint">仅做单主机单端口的 TCP 连接测试，3 秒超时；不提供批量扫描。</p>
  </Panel>
</div>

<style>
  .tool {
    display: flex;
    flex-direction: column;
    gap: var(--sp-3);
    flex: 1;
    min-height: 0;
  }

  .query {
    display: flex;
    gap: var(--sp-2);
  }

  .domain {
    flex: 1;
    min-width: 0;
  }

  .server {
    background: var(--color-bg);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    color: var(--color-text);
    padding: 0.5rem var(--sp-3);
    font-size: var(--fs-sm);
    cursor: pointer;
  }

  .types {
    display: flex;
    flex-wrap: wrap;
    gap: var(--sp-2);
  }

  .type {
    display: inline-flex;
    align-items: center;
    gap: var(--sp-1);
    padding: 0.2rem var(--sp-2);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-full);
    background: var(--color-surface);
    font-family: var(--font-mono);
    font-size: var(--fs-xs);
    color: var(--color-text-secondary);
    cursor: pointer;
    transition:
      border-color var(--motion),
      color var(--motion);
  }

  .type.on {
    border-color: var(--color-primary-border);
    background: var(--color-primary-bg);
    color: var(--color-primary);
  }

  .type input {
    accent-color: var(--color-primary);
  }

  .groups {
    display: flex;
    flex-direction: column;
    gap: var(--sp-4);
  }

  .group header {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    margin-bottom: var(--sp-2);
  }

  .type-name {
    font-family: var(--font-mono);
    font-weight: var(--fw-semibold);
    color: var(--color-text);
  }

  .tag {
    padding: 0.1rem var(--sp-2);
    border-radius: var(--radius-full);
    background: var(--color-surface-2);
    font-size: var(--fs-xs);
    color: var(--color-text-muted);
  }

  .tag.err {
    background: var(--color-danger-bg);
    color: var(--color-danger);
  }

  .group ul {
    list-style: none;
    display: flex;
    flex-direction: column;
    gap: var(--sp-1);
  }

  .group li {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    padding: var(--sp-1) var(--sp-2);
    border-radius: var(--radius-sm);
    background: var(--color-surface-2);
  }

  .group code {
    flex: 1;
    font-family: var(--font-mono);
    font-size: var(--fs-sm);
    color: var(--color-text);
    word-break: break-all;
  }

  .ttl {
    font-size: var(--fs-xs);
    color: var(--color-text-muted);
    white-space: nowrap;
  }

  .error {
    padding: var(--sp-2) var(--sp-3);
    border: 1px solid var(--color-danger-border);
    border-radius: var(--radius-sm);
    background: var(--color-danger-bg);
    color: var(--color-danger);
    font-size: var(--fs-sm);
  }

  .hint {
    margin-top: var(--sp-3);
    font-size: var(--fs-xs);
    color: var(--color-text-muted);
  }

  .probe {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: var(--sp-2);
  }

  .probe-host {
    flex: 1;
    min-width: 12rem;
  }

  .probe-port {
    width: 7rem;
  }

  .port-chip {
    display: inline-flex;
    align-items: center;
    gap: var(--sp-1);
    padding: 0.2rem var(--sp-2);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-full);
    background: var(--color-surface-2);
    font-family: var(--font-mono);
    font-size: var(--fs-xs);
    color: var(--color-text-secondary);
  }

  .port-chip:hover {
    border-color: var(--color-primary-border);
    color: var(--color-primary);
  }

  .port-chip span {
    font-family: var(--font-sans);
    color: var(--color-text-muted);
  }

  .probe-result {
    margin-top: var(--sp-3);
    padding: var(--sp-2) var(--sp-3);
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-danger-border);
    background: var(--color-danger-bg);
    color: var(--color-danger);
    font-size: var(--fs-sm);
  }

  .probe-result.open {
    border-color: var(--color-success-border);
    background: var(--color-success-bg);
    color: var(--color-success);
  }
</style>
