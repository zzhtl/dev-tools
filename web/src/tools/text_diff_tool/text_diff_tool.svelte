<script lang="ts">
  import { diffLines, type DiffResult } from "../../lib/diff";

  let textA = $state("");
  let textB = $state("");
  let result = $state<DiffResult | null>(null);

  function run() {
    result = diffLines(textA, textB);
  }

  $effect(() => {
    textA;
    textB;
    run();
  });

  function loadExample() {
    textA = `name: dev-tools
version: 1.0.0
features:
  - json
  - regex
port: 8080`;
    textB = `name: dev-tools
version: 1.2.0
features:
  - json
  - regex
  - jwt
port: 9000`;
  }

  function clearAll() {
    textA = "";
    textB = "";
  }
</script>

<div class="text-diff-tool">
  <div class="controls">
    <button class="primary-btn" onclick={loadExample}>📋 示例</button>
    <button class="secondary-btn" onclick={clearAll}>🗑️ 清空</button>
    {#if result}
      <div class="summary">
        <span class="stat add">+{result.added}</span>
        <span class="stat del">-{result.removed}</span>
        <span class="stat chg">~{result.changed}</span>
        <span class="hint"
          >{result.added + result.removed + result.changed === 0
            ? "两侧内容一致"
            : "左为 A，右为 B"}</span
        >
      </div>
    {/if}
  </div>

  <div class="inputs">
    <div class="input-col">
      <div class="col-header">文本 A · {textA.split("\n").length} 行</div>
      <textarea bind:value={textA} placeholder="在此粘贴文本 A..." spellcheck="false"></textarea>
    </div>
    <div class="input-col">
      <div class="col-header">文本 B · {textB.split("\n").length} 行</div>
      <textarea bind:value={textB} placeholder="在此粘贴文本 B..." spellcheck="false"></textarea>
    </div>
  </div>

  {#if result && (textA || textB)}
    <div class="diff-result">
      <div class="diff-rows">
        {#each result.rows as row}
          <div class="diff-row {row.type}">
            <span class="diff-ln">{row.leftNo ?? ""}</span>
            <span class="diff-cell left">{row.left}</span>
            <span class="diff-ln">{row.rightNo ?? ""}</span>
            <span class="diff-cell right">{row.right}</span>
          </div>
        {/each}
      </div>
    </div>
  {/if}

  <div class="info-section">
    <h3>关于文本对比</h3>
    <p>
      基于 LCS 行级算法逐行对齐：<span class="i-add">绿色</span> 为新增，<span class="i-del"
        >红色</span
      > 为删除，<span class="i-chg">黄色</span> 为修改。纯前端处理，内容不离开本机。
    </p>
  </div>
</div>

<style>
  .text-diff-tool {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .controls {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    flex-wrap: wrap;
  }

  .secondary-btn {
    background: var(--bg-dark) !important;
    border: 1px solid var(--border) !important;
    color: var(--text-secondary) !important;
  }

  .summary {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    margin-left: auto;
    font-family: "JetBrains Mono", monospace;
    font-size: 0.85rem;
  }
  .stat {
    font-weight: 600;
  }
  .stat.add {
    color: var(--accent-green);
  }
  .stat.del {
    color: #f87171;
  }
  .stat.chg {
    color: #fbbf24;
  }
  .hint {
    color: var(--text-muted);
    font-family: inherit;
  }

  .inputs {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 0.75rem;
  }

  .input-col {
    display: flex;
    flex-direction: column;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    overflow: hidden;
  }

  .col-header {
    padding: 0.5rem 0.75rem;
    background: var(--bg-dark);
    border-bottom: 1px solid var(--border);
    font-size: 0.8rem;
    color: var(--text-secondary);
  }

  .input-col textarea {
    border: none !important;
    border-radius: 0 !important;
    min-height: 200px;
    resize: vertical;
    font-family: "JetBrains Mono", monospace !important;
    font-size: 0.85rem;
  }

  .diff-result {
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    overflow: hidden;
  }

  .diff-rows {
    max-height: 460px;
    overflow: auto;
    font-family: "JetBrains Mono", monospace;
    font-size: 0.8rem;
    line-height: 1.5;
  }

  .diff-row {
    display: grid;
    grid-template-columns: 3rem 1fr 3rem 1fr;
    white-space: pre-wrap;
    word-break: break-word;
  }

  .diff-ln {
    text-align: right;
    padding: 0 0.5rem;
    color: var(--text-muted);
    background: var(--bg-card);
    user-select: none;
  }

  .diff-cell {
    padding: 0 0.5rem;
  }

  .diff-row.insert .diff-cell.right,
  .diff-row.replace .diff-cell.right {
    background: rgba(16, 185, 129, 0.15);
  }
  .diff-row.delete .diff-cell.left,
  .diff-row.replace .diff-cell.left {
    background: rgba(248, 113, 113, 0.15);
  }
  .diff-row.delete .diff-cell.right,
  .diff-row.insert .diff-cell.left {
    background: rgba(255, 255, 255, 0.02);
  }

  .info-section {
    padding: 1rem;
    background: var(--bg-dark);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
  }
  .info-section h3 {
    margin-bottom: 0.5rem;
  }
  .info-section p {
    font-size: 0.85rem;
    line-height: 1.7;
    color: var(--text-secondary);
  }
  .i-add {
    color: var(--accent-green);
  }
  .i-del {
    color: #f87171;
  }
  .i-chg {
    color: #fbbf24;
  }
</style>
