<script lang="ts">
  import CopyButton from "../../lib/ui/CopyButton.svelte";
  import Panel from "../../lib/ui/Panel.svelte";
  import TextInput from "../../lib/ui/TextInput.svelte";
  import EmptyState from "../../lib/ui/EmptyState.svelte";

  let input = $state("192.168.1.10/24");

  const EXAMPLES = ["192.168.1.10/24", "10.0.0.0/8", "172.16.5.1/20", "2001:db8::1/48", "fe80::/10"];

  interface Row {
    label: string;
    value: string;
  }

  // ───────── IPv4 ─────────
  function parseIpv4(text: string): number | null {
    const parts = text.split(".");
    if (parts.length !== 4) return null;
    let value = 0;
    for (const part of parts) {
      if (!/^\d{1,3}$/.test(part)) return null;
      const octet = Number(part);
      if (octet > 255) return null;
      value = value * 256 + octet;
    }
    return value >>> 0;
  }

  function formatIpv4(value: number): string {
    return [24, 16, 8, 0].map((shift) => (value >>> shift) & 0xff).join(".");
  }

  function ipv4Scope(value: number): string {
    const a = (value >>> 24) & 0xff;
    const b = (value >>> 16) & 0xff;
    if (a === 10) return "私有地址（RFC 1918）";
    if (a === 172 && b >= 16 && b <= 31) return "私有地址（RFC 1918）";
    if (a === 192 && b === 168) return "私有地址（RFC 1918）";
    if (a === 127) return "环回地址";
    if (a === 169 && b === 254) return "链路本地地址";
    if (a >= 224 && a <= 239) return "组播地址";
    if (a === 100 && b >= 64 && b <= 127) return "运营商级 NAT（RFC 6598）";
    return "公网地址";
  }

  function analyzeIpv4(address: string, prefix: number): Row[] {
    const value = parseIpv4(address);
    if (value === null) return [];
    // 注意 prefix=0 时 32 位移位在 JS 里是未定义行为，单独处理
    const mask = prefix === 0 ? 0 : (0xffffffff << (32 - prefix)) >>> 0;
    const network = (value & mask) >>> 0;
    const broadcast = (network | (~mask >>> 0)) >>> 0;
    const total = 2 ** (32 - prefix);
    const usable = prefix >= 31 ? (prefix === 32 ? 1 : 2) : total - 2;

    return [
      { label: "IP 地址", value: formatIpv4(value) },
      { label: "网络地址", value: `${formatIpv4(network)}/${prefix}` },
      { label: "广播地址", value: formatIpv4(broadcast) },
      { label: "子网掩码", value: formatIpv4(mask) },
      { label: "反掩码", value: formatIpv4(~mask >>> 0) },
      {
        label: "可用主机范围",
        value:
          prefix >= 31
            ? `${formatIpv4(network)} – ${formatIpv4(broadcast)}（点对点/单主机）`
            : `${formatIpv4(network + 1)} – ${formatIpv4(broadcast - 1)}`,
      },
      { label: "地址总数", value: total.toLocaleString("zh-CN") },
      { label: "可用主机数", value: usable.toLocaleString("zh-CN") },
      { label: "地址类型", value: ipv4Scope(value) },
    ];
  }

  // ───────── IPv6 ─────────
  function parseIpv6(text: string): bigint | null {
    let head = text;
    let tail = "";
    if (text.includes("::")) {
      const parts = text.split("::");
      if (parts.length !== 2) return null;
      [head, tail] = parts;
    } else if (text.split(":").length !== 8) {
      return null;
    }

    const headGroups = head ? head.split(":") : [];
    const tailGroups = tail ? tail.split(":") : [];
    if (headGroups.length + tailGroups.length > 8) return null;

    const fill = 8 - headGroups.length - tailGroups.length;
    const groups = [...headGroups, ...Array(text.includes("::") ? fill : 0).fill("0"), ...tailGroups];
    if (groups.length !== 8) return null;

    let value = 0n;
    for (const group of groups) {
      if (!/^[0-9a-fA-F]{0,4}$/.test(group)) return null;
      value = (value << 16n) | BigInt(Number.parseInt(group || "0", 16));
    }
    return value;
  }

  function formatIpv6(value: bigint): string {
    const groups: string[] = [];
    for (let i = 7n; i >= 0n; i--) {
      groups.push((((value >> (i * 16n)) & 0xffffn) || 0n).toString(16));
    }
    // 压缩最长的一段连续 0
    let bestStart = -1;
    let bestLen = 0;
    let start = -1;
    let len = 0;
    groups.forEach((group, index) => {
      if (group === "0") {
        if (start < 0) start = index;
        len++;
        if (len > bestLen) {
          bestLen = len;
          bestStart = start;
        }
      } else {
        start = -1;
        len = 0;
      }
    });
    if (bestLen < 2) return groups.join(":");
    return `${groups.slice(0, bestStart).join(":")}::${groups.slice(bestStart + bestLen).join(":")}`;
  }

  function analyzeIpv6(address: string, prefix: number): Row[] {
    const value = parseIpv6(address);
    if (value === null) return [];
    const mask = prefix === 0 ? 0n : ((1n << BigInt(prefix)) - 1n) << BigInt(128 - prefix);
    const network = value & mask;
    const last = network | (~mask & ((1n << 128n) - 1n));
    const total = 1n << BigInt(128 - prefix);

    const first16 = (value >> 112n) & 0xffffn;
    let scope = "全球单播";
    if (value === 1n) scope = "环回地址 ::1";
    else if ((first16 & 0xffc0n) === 0xfe80n) scope = "链路本地地址";
    else if ((first16 & 0xfe00n) === 0xfc00n) scope = "唯一本地地址 ULA";
    else if ((first16 & 0xff00n) === 0xff00n) scope = "组播地址";
    else if ((value >> 96n) === 0x20010db8n >> 0n) scope = "文档示例地址（RFC 3849）";

    return [
      { label: "IPv6 地址", value: formatIpv6(value) },
      { label: "网络前缀", value: `${formatIpv6(network)}/${prefix}` },
      { label: "起始地址", value: formatIpv6(network) },
      { label: "结束地址", value: formatIpv6(last) },
      { label: "地址总数", value: total.toLocaleString("zh-CN") },
      { label: "地址类型", value: scope },
    ];
  }

  const analysis = $derived.by(() => {
    const raw = input.trim();
    if (!raw) return { rows: [] as Row[], error: "" };

    const [address, prefixText] = raw.split("/");
    const isV6 = address.includes(":");
    const defaultPrefix = isV6 ? 128 : 32;
    const prefix = prefixText === undefined ? defaultPrefix : Number(prefixText);

    if (!Number.isInteger(prefix) || prefix < 0 || prefix > defaultPrefix) {
      return { rows: [], error: `前缀长度需要在 0–${defaultPrefix} 之间` };
    }

    const rows = isV6 ? analyzeIpv6(address, prefix) : analyzeIpv4(address, prefix);
    if (rows.length === 0) {
      return { rows: [], error: `无法解析为 ${isV6 ? "IPv6" : "IPv4"} 地址` };
    }
    return { rows, error: "" };
  });
</script>

<div class="tool">
  <Panel title="地址 / CIDR">
    <TextInput
      bind:value={input}
      mono
      placeholder="例如 192.168.1.10/24 或 2001:db8::1/48"
      invalid={!!analysis.error}
      ariaLabel="IP 地址或 CIDR"
    />
    <div class="examples">
      {#each EXAMPLES as example (example)}
        <button class="example" onclick={() => (input = example)}>{example}</button>
      {/each}
    </div>
    {#if analysis.error}
      <p class="error">{analysis.error}</p>
    {/if}
  </Panel>

  <Panel title="计算结果" grow>
    {#snippet actions()}
      <CopyButton
        value={() => analysis.rows.map((row) => `${row.label}: ${row.value}`).join("\n")}
        label="复制全部"
        disabled={analysis.rows.length === 0}
      />
    {/snippet}

    {#if analysis.rows.length === 0}
      <EmptyState icon="⊞" title="输入地址后自动计算" hint="支持 IPv4 与 IPv6，可省略前缀" />
    {:else}
      <dl class="rows">
        {#each analysis.rows as row (row.label)}
          <div>
            <dt>{row.label}</dt>
            <dd>
              <code>{row.value}</code>
              <CopyButton value={row.value} icon title="复制" />
            </dd>
          </div>
        {/each}
      </dl>
    {/if}
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

  .examples {
    display: flex;
    flex-wrap: wrap;
    gap: var(--sp-2);
    margin-top: var(--sp-3);
  }

  .example {
    padding: 0.15rem var(--sp-2);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-full);
    background: var(--color-surface-2);
    font-family: var(--font-mono);
    font-size: var(--fs-xs);
    color: var(--color-text-muted);
  }

  .example:hover {
    border-color: var(--color-primary-border);
    color: var(--color-primary);
  }

  .rows {
    display: flex;
    flex-direction: column;
    gap: var(--sp-2);
  }

  .rows > div {
    display: flex;
    align-items: center;
    gap: var(--sp-3);
  }

  dt {
    width: 7.5rem;
    flex-shrink: 0;
    font-size: var(--fs-sm);
    color: var(--color-text-muted);
  }

  dd {
    flex: 1;
    min-width: 0;
    display: flex;
    align-items: center;
    gap: var(--sp-2);
  }

  dd code {
    flex: 1;
    padding: var(--sp-2);
    border-radius: var(--radius-sm);
    background: var(--color-code-bg);
    font-family: var(--font-mono);
    font-size: var(--fs-sm);
    color: var(--color-text);
    word-break: break-all;
    user-select: all;
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
</style>
