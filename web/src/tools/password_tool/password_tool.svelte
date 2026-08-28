<script lang="ts">
  import Button from "../../lib/ui/Button.svelte";
  import CopyButton from "../../lib/ui/CopyButton.svelte";
  import Panel from "../../lib/ui/Panel.svelte";

  const LOWER = "abcdefghijklmnopqrstuvwxyz";
  const UPPER = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
  const DIGITS = "0123456789";
  const SYMBOLS = "!@#$%^&*()-_=+[]{};:,.<>?";
  /** 易混淆字符：0/O、1/l/I 等，抄写场景容易出错 */
  const AMBIGUOUS = new Set("0O1lI|`'\"{}[]()/\\");

  let length = $state(20);
  let count = $state(5);
  let useLower = $state(true);
  let useUpper = $state(true);
  let useDigits = $state(true);
  let useSymbols = $state(true);
  let excludeAmbiguous = $state(false);
  let passwords = $state<string[]>([]);

  const alphabet = $derived.by(() => {
    let chars = "";
    if (useLower) chars += LOWER;
    if (useUpper) chars += UPPER;
    if (useDigits) chars += DIGITS;
    if (useSymbols) chars += SYMBOLS;
    if (excludeAmbiguous) {
      chars = [...chars].filter((char) => !AMBIGUOUS.has(char)).join("");
    }
    return chars;
  });

  /**
   * 均匀随机取字符。
   * 直接 `random % n` 会有模偏差，这里用拒绝采样：丢弃落在不完整区间里的取值。
   */
  function randomIndex(bound: number): number {
    const limit = Math.floor(256 / bound) * bound;
    const buffer = new Uint8Array(1);
    for (;;) {
      crypto.getRandomValues(buffer);
      if (buffer[0] < limit) return buffer[0] % bound;
    }
  }

  function generateOne(): string {
    const chars = alphabet;
    return Array.from({ length }, () => chars[randomIndex(chars.length)]).join("");
  }

  function generate() {
    if (!alphabet) {
      passwords = [];
      return;
    }
    passwords = Array.from({ length: count }, generateOne);
  }

  $effect(() => {
    if (passwords.length === 0 && alphabet) generate();
  });

  /** 熵 = 长度 × log2(字符集大小)，只反映生成方式的强度，不评估是否被泄露过 */
  const entropy = $derived(alphabet ? Math.round(length * Math.log2(alphabet.length)) : 0);

  const strength = $derived.by(() => {
    if (entropy >= 128) return { label: "极强", level: 4 };
    if (entropy >= 80) return { label: "强", level: 3 };
    if (entropy >= 60) return { label: "中等", level: 2 };
    if (entropy >= 40) return { label: "偏弱", level: 1 };
    return { label: "弱", level: 0 };
  });
</script>

<div class="tool">
  <Panel title="生成设置">
    {#snippet actions()}
      <Button size="sm" variant="primary" onclick={generate} disabled={!alphabet}>⟳ 生成</Button>
    {/snippet}

    <div class="sliders">
      <label>
        <span>长度 <b>{length}</b></span>
        <input type="range" min="6" max="128" bind:value={length} />
      </label>
      <label>
        <span>数量 <b>{count}</b></span>
        <input type="range" min="1" max="30" bind:value={count} />
      </label>
    </div>

    <div class="charsets">
      <label class="chip" class:on={useLower}><input type="checkbox" bind:checked={useLower} />小写 a-z</label>
      <label class="chip" class:on={useUpper}><input type="checkbox" bind:checked={useUpper} />大写 A-Z</label>
      <label class="chip" class:on={useDigits}><input type="checkbox" bind:checked={useDigits} />数字 0-9</label>
      <label class="chip" class:on={useSymbols}><input type="checkbox" bind:checked={useSymbols} />符号</label>
      <label class="chip" class:on={excludeAmbiguous}>
        <input type="checkbox" bind:checked={excludeAmbiguous} />排除易混字符
      </label>
    </div>

    {#if !alphabet}
      <p class="error">请至少选择一种字符集</p>
    {:else}
      <div class="strength">
        <div class="bar">
          {#each [0, 1, 2, 3] as segment (segment)}
            <span class:filled={segment < strength.level} class="seg l{strength.level}"></span>
          {/each}
        </div>
        <span class="label">{strength.label}</span>
        <span class="entropy">约 {entropy} bit 熵 · 字符集 {alphabet.length} 个</span>
      </div>
    {/if}
  </Panel>

  <Panel title={`生成结果（${passwords.length}）`} grow>
    {#snippet actions()}
      <CopyButton value={() => passwords.join("\n")} label="复制全部" disabled={passwords.length === 0} />
    {/snippet}

    <ul class="list">
      {#each passwords as password, index (index)}
        <li>
          <code>{password}</code>
          <CopyButton value={password} icon title="复制这一条" />
        </li>
      {/each}
    </ul>

    <p class="hint">
      使用 <code>crypto.getRandomValues</code> 生成，并用拒绝采样消除模偏差。密码仅存在于当前页面，不会发送到服务端。
    </p>
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

  .sliders {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
    gap: var(--sp-4);
  }

  .sliders label {
    display: flex;
    flex-direction: column;
    gap: var(--sp-1);
    font-size: var(--fs-sm);
    color: var(--color-text-secondary);
  }

  .sliders b {
    font-family: var(--font-mono);
    color: var(--color-primary);
  }

  .sliders input {
    accent-color: var(--color-primary);
  }

  .charsets {
    display: flex;
    flex-wrap: wrap;
    gap: var(--sp-2);
    margin-top: var(--sp-4);
  }

  .chip {
    display: inline-flex;
    align-items: center;
    gap: var(--sp-1);
    padding: 0.25rem var(--sp-3);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-full);
    background: var(--color-surface-2);
    font-size: var(--fs-sm);
    color: var(--color-text-secondary);
    cursor: pointer;
  }

  .chip.on {
    border-color: var(--color-primary-border);
    background: var(--color-primary-bg);
    color: var(--color-primary);
  }

  .chip input {
    accent-color: var(--color-primary);
  }

  .strength {
    display: flex;
    align-items: center;
    gap: var(--sp-3);
    margin-top: var(--sp-4);
  }

  .bar {
    display: flex;
    gap: 3px;
    width: 140px;
  }

  .seg {
    flex: 1;
    height: 5px;
    border-radius: var(--radius-full);
    background: var(--color-surface-3);
  }

  .seg.filled.l0,
  .seg.filled.l1 {
    background: var(--color-danger);
  }

  .seg.filled.l2 {
    background: var(--color-warning);
  }

  .seg.filled.l3,
  .seg.filled.l4 {
    background: var(--color-success);
  }

  .label {
    font-size: var(--fs-sm);
    font-weight: var(--fw-medium);
    color: var(--color-text);
  }

  .entropy {
    font-size: var(--fs-xs);
    color: var(--color-text-muted);
    font-family: var(--font-mono);
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
    font-size: var(--fs-md);
    color: var(--color-text);
    word-break: break-all;
    user-select: all;
  }

  .error {
    margin-top: var(--sp-3);
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

  .hint code {
    font-family: var(--font-mono);
  }
</style>
