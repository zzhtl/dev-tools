<script lang="ts">
  import CopyButton from "../../lib/ui/CopyButton.svelte";
  import Panel from "../../lib/ui/Panel.svelte";
  import Tabs from "../../lib/ui/Tabs.svelte";
  import TextInput from "../../lib/ui/TextInput.svelte";

  const BASES = [
    { value: "2", label: "二进制", prefix: "0b" },
    { value: "8", label: "八进制", prefix: "0o" },
    { value: "10", label: "十进制", prefix: "" },
    { value: "16", label: "十六进制", prefix: "0x" },
  ];

  const WIDTHS = [
    { value: "8", label: "8 位" },
    { value: "16", label: "16 位" },
    { value: "32", label: "32 位" },
    { value: "64", label: "64 位" },
  ];

  let inputBase = $state("10");
  let inputText = $state("255");
  let width = $state("32");
  let signed = $state(false);

  let operand = $state("0");
  let operation = $state<"and" | "or" | "xor" | "shl" | "shr">("and");

  const bits = $derived(BigInt(width));
  const mask = $derived((1n << bits) - 1n);

  function parseValue(text: string, base: number): bigint | null {
    const cleaned = text.trim().replace(/[\s_]/g, "").replace(/^0[bBoOxX]/, "");
    if (!cleaned) return null;
    const negative = cleaned.startsWith("-");
    const digits = negative ? cleaned.slice(1) : cleaned;
    if (!digits) return null;
    const alphabet = "0123456789abcdefghijklmnopqrstuvwxyz".slice(0, base);
    let result = 0n;
    for (const char of digits.toLowerCase()) {
      const index = alphabet.indexOf(char);
      if (index < 0) return null;
      result = result * BigInt(base) + BigInt(index);
    }
    return negative ? -result : result;
  }

  /** 按当前位宽取模，负数用补码表示 */
  function toStored(value: bigint): bigint {
    return ((value % (mask + 1n)) + mask + 1n) & mask;
  }

  /** 有符号模式下把补码还原成负数 */
  function toDisplay(stored_: bigint): bigint {
    if (!signed) return stored_;
    const signBit = 1n << (bits - 1n);
    return stored_ >= signBit ? stored_ - (mask + 1n) : stored_;
  }

  const parsed = $derived(parseValue(inputText, Number(inputBase)));
  const stored = $derived(parsed === null ? null : toStored(parsed));
  const shown = $derived(stored === null ? null : toDisplay(stored));

  const overflow = $derived(parsed !== null && stored !== null && toDisplay(stored) !== parsed);

  function render(value: bigint | null, base: number): string {
    if (value === null) return "";
    return value < 0n ? `-${(-value).toString(base)}` : value.toString(base);
  }

  /** 每 4 位一组，便于人眼对齐 */
  function groupBinary(text: string): string {
    if (!text) return "";
    const padded = text.padStart(Number(width), "0");
    return padded.replace(/(.{4})(?=.)/g, "$1 ");
  }

  const operandValue = $derived(parseValue(operand, Number(inputBase)));

  const bitResult = $derived.by(() => {
    if (stored === null || operandValue === null) return null;
    const rhs = toStored(operandValue);
    switch (operation) {
      case "and":
        return stored & rhs;
      case "or":
        return stored | rhs;
      case "xor":
        return stored ^ rhs;
      case "shl":
        return (stored << rhs) & mask;
      case "shr":
        return stored >> rhs;
    }
  });

  const notResult = $derived(stored === null ? null : ~stored & mask);
</script>

<div class="tool">
  <Panel title="输入">
    {#snippet actions()}
      <label class="opt">
        <input type="checkbox" bind:checked={signed} />
        有符号
      </label>
      <select bind:value={width} aria-label="位宽">
        {#each WIDTHS as item (item.value)}
          <option value={item.value}>{item.label}</option>
        {/each}
      </select>
    {/snippet}

    <Tabs bind:value={inputBase} tabs={BASES.map((b) => ({ value: b.value, label: b.label }))} ariaLabel="输入进制" />
    <div class="input">
      <TextInput
        bind:value={inputText}
        mono
        placeholder="输入数值…"
        invalid={parsed === null && inputText.trim() !== ""}
        ariaLabel="数值"
      />
    </div>
    {#if parsed === null && inputText.trim() !== ""}
      <p class="error">不是合法的 {BASES.find((b) => b.value === inputBase)?.label}</p>
    {:else if overflow}
      <p class="warn">数值超出 {width} 位范围，已按补码截断</p>
    {/if}
  </Panel>

  <Panel title="各进制表示">
    <dl class="out">
      {#each BASES as base (base.value)}
        <div>
          <dt>{base.label}<span>{base.prefix}</span></dt>
          <dd>
            <code>
              {base.value === "2"
                ? groupBinary(render(stored, 2))
                : render(base.value === "10" ? shown : stored, Number(base.value))}
            </code>
            <CopyButton
              value={() => render(base.value === "10" ? shown : stored, Number(base.value))}
              icon
              title="复制"
            />
          </dd>
        </div>
      {/each}
    </dl>
  </Panel>

  <Panel title="位运算">
    <div class="bitops">
      <Tabs
        bind:value={operation}
        size="sm"
        tabs={[
          { value: "and", label: "AND &" },
          { value: "or", label: "OR |" },
          { value: "xor", label: "XOR ^" },
          { value: "shl", label: "左移 <<" },
          { value: "shr", label: "右移 >>" },
        ]}
        ariaLabel="位运算"
      />
      <div class="operand">
        <TextInput bind:value={operand} mono size="sm" ariaLabel="第二操作数" />
      </div>
    </div>

    <dl class="out">
      <div>
        <dt>运算结果</dt>
        <dd>
          <code>
            {bitResult === null
              ? ""
              : `${render(toDisplay(bitResult), 10)}  ·  0x${render(bitResult, 16)}  ·  ${groupBinary(render(bitResult, 2))}`}
          </code>
          <CopyButton value={() => (bitResult === null ? "" : render(toDisplay(bitResult), 10))} icon />
        </dd>
      </div>
      <div>
        <dt>按位取反 ~</dt>
        <dd>
          <code>
            {notResult === null
              ? ""
              : `${render(toDisplay(notResult), 10)}  ·  0x${render(notResult, 16)}  ·  ${groupBinary(render(notResult, 2))}`}
          </code>
          <CopyButton value={() => (notResult === null ? "" : render(toDisplay(notResult), 10))} icon />
        </dd>
      </div>
    </dl>
    <p class="hint">位运算按无符号补码在选定位宽内进行；右移为逻辑右移（高位补 0）。</p>
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

  .opt {
    display: flex;
    align-items: center;
    gap: var(--sp-1);
    font-size: var(--fs-sm);
    color: var(--color-text-secondary);
  }

  .opt input {
    accent-color: var(--color-primary);
  }

  select {
    background: var(--color-bg);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    color: var(--color-text);
    padding: 0.25rem var(--sp-2);
    font-size: var(--fs-sm);
  }

  .input {
    margin-top: var(--sp-3);
  }

  .out {
    display: flex;
    flex-direction: column;
    gap: var(--sp-2);
  }

  .out > div {
    display: flex;
    align-items: center;
    gap: var(--sp-3);
  }

  dt {
    width: 7rem;
    flex-shrink: 0;
    font-size: var(--fs-sm);
    color: var(--color-text-secondary);
  }

  dt span {
    margin-left: var(--sp-1);
    font-family: var(--font-mono);
    font-size: var(--fs-xs);
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
    min-height: 1.9rem;
  }

  .bitops {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: var(--sp-3);
    margin-bottom: var(--sp-3);
  }

  .operand {
    width: 12rem;
  }

  .error,
  .warn {
    margin-top: var(--sp-2);
    padding: var(--sp-2) var(--sp-3);
    border-radius: var(--radius-sm);
    font-size: var(--fs-sm);
  }

  .error {
    border: 1px solid var(--color-danger-border);
    background: var(--color-danger-bg);
    color: var(--color-danger);
  }

  .warn {
    border: 1px solid var(--color-warning-border);
    background: var(--color-warning-bg);
    color: var(--color-text-secondary);
  }

  .hint {
    margin-top: var(--sp-3);
    font-size: var(--fs-xs);
    color: var(--color-text-muted);
  }
</style>
