<script lang="ts">
  import Button from "../../lib/ui/Button.svelte";
  import CopyButton from "../../lib/ui/CopyButton.svelte";
  import Panel from "../../lib/ui/Panel.svelte";
  import SplitPane from "../../lib/ui/SplitPane.svelte";
  import Tabs from "../../lib/ui/Tabs.svelte";
  import TextInput from "../../lib/ui/TextInput.svelte";

  let mode = $state<"toDate" | "toTimestamp">("toDate");
  let unit = $state<"seconds" | "millis">("seconds");

  // ── 当前时间，每秒走一次 ──
  let now = $state(Date.now());
  $effect(() => {
    const timer = setInterval(() => (now = Date.now()), 1000);
    return () => clearInterval(timer);
  });

  const localZone = Intl.DateTimeFormat().resolvedOptions().timeZone;
  const offsetLabel = (() => {
    const minutes = -new Date().getTimezoneOffset();
    const sign = minutes >= 0 ? "+" : "-";
    const abs = Math.abs(minutes);
    return `UTC${sign}${String(Math.floor(abs / 60)).padStart(2, "0")}:${String(abs % 60).padStart(2, "0")}`;
  })();

  function pad(value: number, width = 2): string {
    return String(value).padStart(width, "0");
  }

  function formatLocal(date: Date): string {
    return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())} ${pad(date.getHours())}:${pad(date.getMinutes())}:${pad(date.getSeconds())}.${pad(date.getMilliseconds(), 3)}`;
  }

  function formatUtc(date: Date): string {
    return `${date.getUTCFullYear()}-${pad(date.getUTCMonth() + 1)}-${pad(date.getUTCDate())} ${pad(date.getUTCHours())}:${pad(date.getUTCMinutes())}:${pad(date.getUTCSeconds())}.${pad(date.getUTCMilliseconds(), 3)}`;
  }

  const RELATIVE_UNITS: [Intl.RelativeTimeFormatUnit, number][] = [
    ["year", 31536000000],
    ["month", 2592000000],
    ["day", 86400000],
    ["hour", 3600000],
    ["minute", 60000],
    ["second", 1000],
  ];

  function formatRelative(target: number, base: number): string {
    const diff = target - base;
    const formatter = new Intl.RelativeTimeFormat("zh-CN", { numeric: "auto" });
    for (const [unitName, ms] of RELATIVE_UNITS) {
      if (Math.abs(diff) >= ms || unitName === "second") {
        return formatter.format(Math.round(diff / ms), unitName);
      }
    }
    return "刚刚";
  }

  // ── 时间戳 → 日期 ──
  let timestampText = $state("");

  type TimestampView =
    | { ok: false; error: string }
    | { ok: true; local: string; utc: string; iso: string; relative: string; weekday: string };

  const fromTimestamp = $derived.by((): TimestampView | null => {
    const raw = timestampText.trim();
    if (!raw) return null;
    if (!/^-?\d+$/.test(raw)) return { ok: false, error: "时间戳只能是整数" };
    const value = Number(raw);
    const ms = unit === "seconds" ? value * 1000 : value;
    const date = new Date(ms);
    if (Number.isNaN(date.getTime())) return { ok: false, error: "超出可表示的时间范围" };
    return {
      ok: true,
      local: formatLocal(date),
      utc: formatUtc(date),
      iso: date.toISOString(),
      relative: formatRelative(ms, now),
      weekday: ["周日", "周一", "周二", "周三", "周四", "周五", "周六"][date.getDay()],
    };
  });

  // ── 日期 → 时间戳 ──
  // 用原生 datetime-local 取代手写日历：少 200 行代码，且自带键盘与本地化支持
  function toLocalInputValue(date: Date): string {
    return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}T${pad(date.getHours())}:${pad(date.getMinutes())}:${pad(date.getSeconds())}`;
  }

  let dateInput = $state(toLocalInputValue(new Date()));

  type DateView =
    | { ok: false; error: string }
    | { ok: true; seconds: string; millis: string; iso: string; utc: string; relative: string };

  const fromDate = $derived.by((): DateView | null => {
    if (!dateInput) return null;
    const date = new Date(dateInput);
    if (Number.isNaN(date.getTime())) return { ok: false, error: "无效的日期" };
    return {
      ok: true,
      seconds: Math.floor(date.getTime() / 1000).toString(),
      millis: date.getTime().toString(),
      iso: date.toISOString(),
      utc: formatUtc(date),
      relative: formatRelative(date.getTime(), now),
    };
  });

  function shiftDate(mutate: (date: Date) => void) {
    const date = new Date(dateInput || Date.now());
    mutate(date);
    dateInput = toLocalInputValue(date);
  }

  const QUICK_DATES: { label: string; apply: (date: Date) => void }[] = [
    { label: "此刻", apply: (d) => d.setTime(Date.now()) },
    { label: "今天 00:00", apply: (d) => d.setHours(0, 0, 0, 0) },
    { label: "今天 23:59", apply: (d) => d.setHours(23, 59, 59, 0) },
    { label: "昨天", apply: (d) => d.setDate(d.getDate() - 1) },
    { label: "明天", apply: (d) => d.setDate(d.getDate() + 1) },
    { label: "本周一", apply: (d) => d.setDate(d.getDate() - ((d.getDay() + 6) % 7)) },
    { label: "月初", apply: (d) => d.setDate(1) },
    { label: "月末", apply: (d) => d.setMonth(d.getMonth() + 1, 0) },
  ];

  const nowSeconds = $derived(Math.floor(now / 1000).toString());
</script>

<div class="tool">
  <div class="toolbar">
    <Tabs
      bind:value={mode}
      tabs={[
        { value: "toDate", label: "时间戳 → 日期" },
        { value: "toTimestamp", label: "日期 → 时间戳" },
      ]}
      ariaLabel="转换方向"
    />
    <Tabs
      bind:value={unit}
      size="sm"
      tabs={[
        { value: "seconds", label: "秒" },
        { value: "millis", label: "毫秒" },
      ]}
      ariaLabel="时间戳单位"
    />
    <div class="spacer"></div>
    <span class="zone">{localZone} · {offsetLabel}</span>
  </div>

  <SplitPane>
    {#snippet left()}
      <Panel title={mode === "toDate" ? "时间戳" : "日期时间"} grow>
        {#snippet actions()}
          {#if mode === "toDate"}
            <Button
              size="sm"
              onclick={() =>
                (timestampText = unit === "seconds" ? nowSeconds : Date.now().toString())}
            >
              取当前
            </Button>
          {/if}
        {/snippet}

        {#if mode === "toDate"}
          <TextInput
            bind:value={timestampText}
            mono
            placeholder={unit === "seconds" ? "例如 1716239022" : "例如 1716239022000"}
            ariaLabel="时间戳"
          />
          <div class="live">
            <span>当前时间戳</span>
            <code>{unit === "seconds" ? nowSeconds : now}</code>
            <CopyButton value={() => (unit === "seconds" ? nowSeconds : String(now))} icon title="复制当前时间戳" />
          </div>
        {:else}
          <input
            class="datetime"
            type="datetime-local"
            step="1"
            bind:value={dateInput}
            aria-label="日期时间"
          />
          <div class="quick">
            {#each QUICK_DATES as item (item.label)}
              <Button size="sm" onclick={() => shiftDate(item.apply)}>{item.label}</Button>
            {/each}
          </div>
        {/if}
      </Panel>
    {/snippet}

    {#snippet right()}
      <Panel title="转换结果" grow>
        {#if mode === "toDate"}
          {#if !fromTimestamp}
            <p class="placeholder">输入时间戳后自动转换</p>
          {:else if !fromTimestamp.ok}
            <p class="error">{fromTimestamp.error}</p>
          {:else}
            <dl class="out">
              <div>
                <dt>本地时间 <span class="sub">{fromTimestamp.weekday}</span></dt>
                <dd><code>{fromTimestamp.local}</code><CopyButton value={fromTimestamp.local} icon /></dd>
              </div>
              <div>
                <dt>UTC</dt>
                <dd><code>{fromTimestamp.utc}</code><CopyButton value={fromTimestamp.utc} icon /></dd>
              </div>
              <div>
                <dt>ISO 8601</dt>
                <dd><code>{fromTimestamp.iso}</code><CopyButton value={fromTimestamp.iso} icon /></dd>
              </div>
              <div>
                <dt>相对现在</dt>
                <dd><code>{fromTimestamp.relative}</code></dd>
              </div>
            </dl>
          {/if}
        {:else if !fromDate}
          <p class="placeholder">选择日期后自动转换</p>
        {:else if !fromDate.ok}
          <p class="error">{fromDate.error}</p>
        {:else}
          <dl class="out">
            <div>
              <dt>秒</dt>
              <dd><code>{fromDate.seconds}</code><CopyButton value={fromDate.seconds} icon /></dd>
            </div>
            <div>
              <dt>毫秒</dt>
              <dd><code>{fromDate.millis}</code><CopyButton value={fromDate.millis} icon /></dd>
            </div>
            <div>
              <dt>ISO 8601</dt>
              <dd><code>{fromDate.iso}</code><CopyButton value={fromDate.iso} icon /></dd>
            </div>
            <div>
              <dt>UTC</dt>
              <dd><code>{fromDate.utc}</code><CopyButton value={fromDate.utc} icon /></dd>
            </div>
            <div>
              <dt>相对现在</dt>
              <dd><code>{fromDate.relative}</code></dd>
            </div>
          </dl>
        {/if}
      </Panel>
    {/snippet}
  </SplitPane>
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

  .zone {
    font-size: var(--fs-xs);
    color: var(--color-text-muted);
    font-family: var(--font-mono);
  }

  .datetime {
    width: 100%;
    background: var(--color-bg);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    color: var(--color-text);
    padding: 0.5rem var(--sp-3);
    font-family: var(--font-mono);
    font-size: var(--fs-md);
  }

  .datetime:focus {
    outline: none;
    border-color: var(--color-primary);
    box-shadow: 0 0 0 3px var(--color-focus-ring);
  }

  .live {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    margin-top: var(--sp-3);
    padding: var(--sp-2) var(--sp-3);
    border-radius: var(--radius-sm);
    background: var(--color-surface-2);
    font-size: var(--fs-sm);
    color: var(--color-text-muted);
  }

  .live code {
    flex: 1;
    font-family: var(--font-mono);
    color: var(--color-accent);
  }

  .quick {
    display: flex;
    flex-wrap: wrap;
    gap: var(--sp-2);
    margin-top: var(--sp-3);
  }

  .out {
    display: flex;
    flex-direction: column;
    gap: var(--sp-3);
  }

  dt {
    font-size: var(--fs-xs);
    color: var(--color-text-muted);
    margin-bottom: var(--sp-1);
  }

  .sub {
    color: var(--color-text-secondary);
  }

  dd {
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
    font-size: var(--fs-md);
    color: var(--color-text);
    user-select: all;
    word-break: break-all;
  }

  .placeholder {
    color: var(--color-text-muted);
    font-size: var(--fs-md);
  }

  .error {
    color: var(--color-danger);
    font-size: var(--fs-sm);
  }
</style>
