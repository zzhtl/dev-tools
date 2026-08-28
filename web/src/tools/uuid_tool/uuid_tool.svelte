<script lang="ts">
  import Button from "../../lib/ui/Button.svelte";
  import CopyButton from "../../lib/ui/CopyButton.svelte";
  import Panel from "../../lib/ui/Panel.svelte";
  import Tabs from "../../lib/ui/Tabs.svelte";
  import TextInput from "../../lib/ui/TextInput.svelte";
  import EmptyState from "../../lib/ui/EmptyState.svelte";

  type Version = "v4" | "v7" | "v1";

  let version = $state<Version>("v4");
  let countText = $state("5");
  let uppercase = $state(false);
  let noDashes = $state(false);
  let uuids = $state<string[]>([]);

  const count = $derived(Math.min(1000, Math.max(1, Number.parseInt(countText, 10) || 1)));

  // crypto.getRandomValues 在非安全上下文也可用（不同于 crypto.randomUUID / crypto.subtle），
  // 因此局域网 IP 访问时这里依然是强随机。
  function randomBytes(length: number): Uint8Array {
    const bytes = new Uint8Array(length);
    crypto.getRandomValues(bytes);
    return bytes;
  }

  function toHex(bytes: Uint8Array): string {
    const hex = Array.from(bytes, (b) => b.toString(16).padStart(2, "0")).join("");
    return `${hex.slice(0, 8)}-${hex.slice(8, 12)}-${hex.slice(12, 16)}-${hex.slice(16, 20)}-${hex.slice(20)}`;
  }

  function uuidV4(): string {
    // randomUUID 需要安全上下文；不可用时用 getRandomValues 手工构造
    if (typeof crypto.randomUUID === "function") return crypto.randomUUID();
    const bytes = randomBytes(16);
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    return toHex(bytes);
  }

  /** RFC 9562 v7：前 48 位是 Unix 毫秒时间戳，天然按时间有序，适合做数据库主键。 */
  function uuidV7(): string {
    const bytes = randomBytes(16);
    const ms = Date.now();
    bytes[0] = Math.floor(ms / 2 ** 40) & 0xff;
    bytes[1] = Math.floor(ms / 2 ** 32) & 0xff;
    bytes[2] = Math.floor(ms / 2 ** 24) & 0xff;
    bytes[3] = Math.floor(ms / 2 ** 16) & 0xff;
    bytes[4] = Math.floor(ms / 2 ** 8) & 0xff;
    bytes[5] = ms & 0xff;
    bytes[6] = (bytes[6] & 0x0f) | 0x70;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    return toHex(bytes);
  }

  const GREGORIAN_OFFSET_MS = 12219292800000n;
  let v1LastMs = 0;
  let v1Counter = 0;

  /**
   * RFC 4122 v1：100ns 精度时间戳 + 时钟序列 + node。
   * 浏览器拿不到 MAC，按 §4.5 用随机 node 并置多播位标记，而不是伪装成真实网卡地址。
   */
  function uuidV1(): string {
    const nowMs = Date.now();
    if (nowMs === v1LastMs) {
      v1Counter = (v1Counter + 1) % 10000;
    } else {
      v1LastMs = nowMs;
      v1Counter = 0;
    }
    const ts = (BigInt(nowMs) + GREGORIAN_OFFSET_MS) * 10000n + BigInt(v1Counter);

    const bytes = randomBytes(16);
    const timeLow = Number(ts & 0xffffffffn);
    const timeMid = Number((ts >> 32n) & 0xffffn);
    const timeHiVersion = (Number((ts >> 48n) & 0x0fffn) | 0x1000) & 0xffff;

    bytes[0] = (timeLow >>> 24) & 0xff;
    bytes[1] = (timeLow >>> 16) & 0xff;
    bytes[2] = (timeLow >>> 8) & 0xff;
    bytes[3] = timeLow & 0xff;
    bytes[4] = (timeMid >>> 8) & 0xff;
    bytes[5] = timeMid & 0xff;
    bytes[6] = (timeHiVersion >>> 8) & 0xff;
    bytes[7] = timeHiVersion & 0xff;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    bytes[10] |= 0x01;
    return toHex(bytes);
  }

  function format(uuid: string): string {
    let result = noDashes ? uuid.replace(/-/g, "") : uuid;
    if (uppercase) result = result.toUpperCase();
    return result;
  }

  function generate() {
    const make = version === "v4" ? uuidV4 : version === "v7" ? uuidV7 : uuidV1;
    uuids = Array.from({ length: count }, () => format(make()));
  }

  // 首次进入先给一批，避免空白界面
  $effect(() => {
    if (uuids.length === 0) generate();
  });

  const allText = $derived(uuids.join("\n"));
</script>

<div class="tool">
  <div class="toolbar">
    <Tabs
      bind:value={version}
      tabs={[
        { value: "v4", label: "v4 随机" },
        { value: "v7", label: "v7 时间有序" },
        { value: "v1", label: "v1 时间戳" },
      ]}
      ariaLabel="UUID 版本"
    />

    <label class="count">
      数量
      <span class="count-input">
        <TextInput bind:value={countText} type="number" size="sm" ariaLabel="生成数量" />
      </span>
    </label>

    <label class="check">
      <input type="checkbox" bind:checked={uppercase} />
      大写
    </label>
    <label class="check">
      <input type="checkbox" bind:checked={noDashes} />
      去横杠
    </label>

    <div class="spacer"></div>

    <Button variant="primary" size="sm" onclick={generate}>⟳ 生成</Button>
  </div>

  <Panel title={`生成结果（${uuids.length}）`} grow>
    {#snippet actions()}
      <CopyButton value={() => allText} label="复制全部" disabled={uuids.length === 0} />
    {/snippet}

    {#if uuids.length === 0}
      <EmptyState icon="⊕" title="还没有生成 UUID" hint="点击右上角「生成」" />
    {:else}
      <ul class="list">
        {#each uuids as uuid, index (index)}
          <li>
            <code>{uuid}</code>
            <CopyButton value={uuid} icon title="复制这一条" />
          </li>
        {/each}
      </ul>
    {/if}
  </Panel>

  <details class="reference">
    <summary>版本怎么选</summary>
    <ul class="notes">
      <li><b>v4</b> — 纯随机，最通用。碰撞概率可忽略，但完全无序，做数据库主键会让 B+ 树频繁分裂。</li>
      <li>
        <b>v7</b> — 前 48 位是毫秒时间戳，天然按时间递增。<b>作主键优先选它</b>，兼具唯一性与索引局部性。
      </li>
      <li>
        <b>v1</b> — 时间戳 + node。浏览器无法读取 MAC，这里按 RFC 4122 §4.5 使用随机 node 并置多播位，
        因此不会泄露网卡地址，但也不具备跨设备唯一性保证。
      </li>
    </ul>
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

  .toolbar {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: var(--sp-3);
  }

  .spacer {
    flex: 1;
  }

  .count,
  .check {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    font-size: var(--fs-sm);
    color: var(--color-text-secondary);
    white-space: nowrap;
  }

  .count-input {
    display: inline-block;
    width: 5rem;
  }

  .check input {
    accent-color: var(--color-primary);
  }

  .list {
    list-style: none;
    display: flex;
    flex-direction: column;
    gap: var(--sp-1);
  }

  .list li {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--sp-2);
    padding: var(--sp-1) var(--sp-2);
    border-radius: var(--radius-sm);
    background: var(--color-surface-2);
  }

  .list code {
    font-family: var(--font-mono);
    font-size: var(--fs-sm);
    color: var(--color-text);
    user-select: all;
    word-break: break-all;
  }

  .reference {
    border: 1px solid var(--color-border);
    border-radius: var(--radius);
    background: var(--color-surface);
    flex-shrink: 0;
  }

  summary {
    padding: var(--sp-2) var(--sp-4);
    cursor: pointer;
    font-size: var(--fs-sm);
    font-weight: var(--fw-medium);
    color: var(--color-text-secondary);
  }

  .notes {
    display: flex;
    flex-direction: column;
    gap: var(--sp-2);
    padding: 0 var(--sp-4) var(--sp-4) calc(var(--sp-4) + 1.1rem);
    font-size: var(--fs-sm);
    color: var(--color-text-secondary);
  }

  .notes b {
    color: var(--color-text);
    font-family: var(--font-mono);
  }
</style>
