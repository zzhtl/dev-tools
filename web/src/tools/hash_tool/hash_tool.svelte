<script lang="ts">
  import CryptoJS from "crypto-js";
  import Button from "../../lib/ui/Button.svelte";
  import CopyButton from "../../lib/ui/CopyButton.svelte";
  import DropZone from "../../lib/ui/DropZone.svelte";
  import Panel from "../../lib/ui/Panel.svelte";
  import Tabs from "../../lib/ui/Tabs.svelte";
  import TextArea from "../../lib/ui/TextArea.svelte";
  import EmptyState from "../../lib/ui/EmptyState.svelte";
  import { formatFileSize } from "../../lib/format";
  import { toast } from "../../lib/toast.svelte";

  interface Algo {
    key: string;
    name: string;
    note: string;
    create: () => { update: (data: unknown) => void; finalize: () => { toString: () => string } };
  }

  const ALGOS: Algo[] = [
    { key: "md5", name: "MD5", note: "128 位，仅适合完整性校验，勿用于安全场景", create: () => CryptoJS.algo.MD5.create() },
    { key: "sha1", name: "SHA-1", note: "160 位，已被碰撞攻破，勿用于签名", create: () => CryptoJS.algo.SHA1.create() },
    { key: "sha256", name: "SHA-256", note: "256 位，当前通用默认选择", create: () => CryptoJS.algo.SHA256.create() },
    { key: "sha384", name: "SHA-384", note: "384 位，SHA-2 系列", create: () => CryptoJS.algo.SHA384.create() },
    { key: "sha512", name: "SHA-512", note: "512 位，SHA-2 系列", create: () => CryptoJS.algo.SHA512.create() },
    { key: "sha3", name: "SHA3-256", note: "SHA-3 标准，海绵结构", create: () => CryptoJS.algo.SHA3.create({ outputLength: 256 }) },
    { key: "ripemd160", name: "RIPEMD-160", note: "160 位，比特币地址派生使用", create: () => CryptoJS.algo.RIPEMD160.create() },
  ];

  let inputType = $state<"text" | "file">("text");
  let text = $state("");
  // 默认只算两种：旧版无条件算 7 种，大文件下是 7 倍无谓开销
  let selected = $state<string[]>(["md5", "sha256"]);
  let results = $state<Record<string, string>>({});
  let computing = $state(false);
  let progress = $state(0);
  let file = $state<{ name: string; size: number } | null>(null);

  const activeAlgos = $derived(ALGOS.filter((algo) => selected.includes(algo.key)));

  function toggle(key: string) {
    selected = selected.includes(key) ? selected.filter((k) => k !== key) : [...selected, key];
  }

  // ── 文本 ──
  $effect(() => {
    if (inputType !== "text") return;
    const value = text;
    const algos = activeAlgos;
    if (!value) {
      results = {};
      return;
    }
    const next: Record<string, string> = {};
    for (const algo of algos) {
      const hasher = algo.create();
      hasher.update(CryptoJS.enc.Utf8.parse(value));
      next[algo.key] = hasher.finalize().toString();
    }
    results = next;
  });

  // ── 文件：分块增量，避免整文件进内存并保持界面响应 ──
  const CHUNK_SIZE = 4 * 1024 * 1024;
  let runToken = 0;

  async function hashFile(picked: File) {
    const token = ++runToken;
    file = { name: picked.name, size: picked.size };
    results = {};
    computing = true;
    progress = 0;

    const algos = activeAlgos;
    if (algos.length === 0) {
      computing = false;
      toast.error("请至少选择一种算法");
      return;
    }

    const hashers = algos.map((algo) => ({ key: algo.key, hasher: algo.create() }));

    try {
      for (let offset = 0; offset < picked.size; offset += CHUNK_SIZE) {
        if (token !== runToken) return; // 期间换了文件，放弃这次计算
        const buffer = await picked.slice(offset, offset + CHUNK_SIZE).arrayBuffer();
        const words = CryptoJS.lib.WordArray.create(buffer);
        for (const { hasher } of hashers) hasher.update(words);
        progress = Math.min(100, ((offset + CHUNK_SIZE) / picked.size) * 100);
        // 让出主线程，避免长任务卡住界面
        await new Promise((resolve) => setTimeout(resolve, 0));
      }
      if (token !== runToken) return;
      const next: Record<string, string> = {};
      for (const { key, hasher } of hashers) next[key] = hasher.finalize().toString();
      results = next;
      progress = 100;
    } catch (err) {
      toast.error(`读取文件失败：${err instanceof Error ? err.message : String(err)}`);
    } finally {
      if (token === runToken) computing = false;
    }
  }

  function reset() {
    runToken++;
    text = "";
    file = null;
    results = {};
    computing = false;
    progress = 0;
  }

  const hasResults = $derived(Object.keys(results).length > 0);
</script>

<div class="tool">
  <div class="toolbar">
    <Tabs
      bind:value={inputType}
      tabs={[
        { value: "text", label: "文本" },
        { value: "file", label: "文件" },
      ]}
      ariaLabel="输入来源"
      onchange={reset}
    />
    <div class="spacer"></div>
    <Button size="sm" variant="ghost" onclick={reset}>清空</Button>
  </div>

  <div class="algos">
    {#each ALGOS as algo (algo.key)}
      <label class="algo" class:on={selected.includes(algo.key)} title={algo.note}>
        <input
          type="checkbox"
          checked={selected.includes(algo.key)}
          onchange={() => toggle(algo.key)}
        />
        {algo.name}
      </label>
    {/each}
  </div>

  {#if inputType === "text"}
    <Panel title="输入文本">
      {#snippet actions()}
        <span class="count">{text.length} 字符</span>
      {/snippet}
      <TextArea bind:value={text} rows={6} placeholder="输入要计算摘要的文本…" ariaLabel="输入文本" />
    </Panel>
  {:else}
    <Panel title="选择文件">
      <DropZone
        onfiles={(files) => hashFile(files[0])}
        disabled={computing}
        hint={computing ? "计算中…" : "拖拽文件到此处，或点击选择（分块读取，支持大文件）"}
      />
      {#if file}
        <div class="fileinfo">
          <span class="fname">{file.name}</span>
          <span class="count">{formatFileSize(file.size)}</span>
        </div>
      {/if}
      {#if computing}
        <div class="progress"><div class="bar" style="width: {progress}%"></div></div>
      {/if}
    </Panel>
  {/if}

  <Panel title="摘要结果" grow>
    {#if !hasResults}
      <EmptyState
        icon="#"
        title={computing ? "正在计算…" : "还没有结果"}
        hint={inputType === "text" ? "输入文本后自动计算" : "选择文件后开始计算"}
      />
    {:else}
      <ul class="results">
        {#each activeAlgos as algo (algo.key)}
          {#if results[algo.key]}
            <li>
              <div class="head">
                <span class="name">{algo.name}</span>
                <span class="note">{algo.note}</span>
                <CopyButton value={results[algo.key]} icon title="复制 {algo.name}" />
              </div>
              <code>{results[algo.key]}</code>
            </li>
          {/if}
        {/each}
      </ul>
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

  .toolbar {
    display: flex;
    align-items: center;
    gap: var(--sp-3);
  }

  .spacer {
    flex: 1;
  }

  .algos {
    display: flex;
    flex-wrap: wrap;
    gap: var(--sp-2);
  }

  .algo {
    display: inline-flex;
    align-items: center;
    gap: var(--sp-1);
    padding: 0.25rem var(--sp-2);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-full);
    background: var(--color-surface);
    font-size: var(--fs-sm);
    color: var(--color-text-secondary);
    cursor: pointer;
    transition:
      border-color var(--motion),
      color var(--motion);
  }

  .algo.on {
    border-color: var(--color-primary-border);
    background: var(--color-primary-bg);
    color: var(--color-primary);
  }

  .algo input {
    accent-color: var(--color-primary);
  }

  .count {
    font-size: var(--fs-xs);
    color: var(--color-text-muted);
    font-family: var(--font-mono);
  }

  .fileinfo {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--sp-3);
    margin-top: var(--sp-3);
    font-size: var(--fs-sm);
  }

  .fname {
    font-family: var(--font-mono);
    color: var(--color-text);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .progress {
    margin-top: var(--sp-2);
    height: 4px;
    border-radius: var(--radius-full);
    background: var(--color-surface-3);
    overflow: hidden;
  }

  .bar {
    height: 100%;
    background: var(--color-primary);
    transition: width var(--motion);
  }

  .results {
    list-style: none;
    display: flex;
    flex-direction: column;
    gap: var(--sp-3);
  }

  .results li {
    display: flex;
    flex-direction: column;
    gap: var(--sp-1);
  }

  .head {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
  }

  .name {
    font-size: var(--fs-sm);
    font-weight: var(--fw-semibold);
    color: var(--color-text);
  }

  .note {
    flex: 1;
    font-size: var(--fs-xs);
    color: var(--color-text-muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .results code {
    padding: var(--sp-2);
    border-radius: var(--radius-sm);
    background: var(--color-code-bg);
    font-family: var(--font-mono);
    font-size: var(--fs-sm);
    color: var(--color-accent);
    word-break: break-all;
    user-select: all;
  }
</style>
