<script lang="ts">
  let generatedUuids = $state<string[]>([]);
  let count = $state(5);
  let uuidVersion = $state<"v4" | "v1">("v4");
  let uppercase = $state(false);
  let noDashes = $state(false);
  let copiedIndex = $state<number | null>(null);

  // 生成 UUID v4
  function generateUuidV4(): string {
    return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, function(c) {
      const r = Math.random() * 16 | 0;
      const v = c === 'x' ? r : (r & 0x3 | 0x8);
      return v.toString(16);
    });
  }

  // 生成 UUID v1 (基于时间)
  function generateUuidV1(): string {
    const now = new Date().getTime();
    const uuid = 'xxxxxxxx-xxxx-1xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, function(c, i) {
      const r = (now + Math.random() * 16) % 16 | 0;
      const v = c === 'x' ? r : (r & 0x3 | 0x8);
      return v.toString(16);
    });
    return uuid;
  }

  function formatUuid(uuid: string): string {
    let result = uuid;
    if (noDashes) {
      result = result.replace(/-/g, '');
    }
    if (uppercase) {
      result = result.toUpperCase();
    }
    return result;
  }

  function generateUuids() {
    const uuids: string[] = [];
    for (let i = 0; i < count; i++) {
      const uuid = uuidVersion === "v4" ? generateUuidV4() : generateUuidV1();
      uuids.push(formatUuid(uuid));
    }
    generatedUuids = uuids;
  }

  async function copyUuid(uuid: string, index: number) {
    try {
      await navigator.clipboard.writeText(uuid);
      copiedIndex = index;
      setTimeout(() => {
        copiedIndex = null;
      }, 1500);
    } catch (err) {
      console.error('复制失败:', err);
    }
  }

  async function copyAll() {
    try {
      await navigator.clipboard.writeText(generatedUuids.join('\n'));
      copiedIndex = -1; // 表示全部复制
      setTimeout(() => {
        copiedIndex = null;
      }, 1500);
    } catch (err) {
      console.error('复制失败:', err);
    }
  }

  // 初始生成
  $effect(() => {
    if (generatedUuids.length === 0) {
      generateUuids();
    }
  });
</script>

<div class="uuid-tool">
  <div class="controls">
    <div class="control-group">
      <label>UUID 版本</label>
      <div class="radio-group">
        <label class="radio-label">
          <input type="radio" bind:group={uuidVersion} value="v4" />
          <span>v4 (随机)</span>
        </label>
        <label class="radio-label">
          <input type="radio" bind:group={uuidVersion} value="v1" />
          <span>v1 (时间戳)</span>
        </label>
      </div>
    </div>

    <div class="control-group">
      <label for="count">生成数量</label>
      <input 
        id="count"
        type="number" 
        bind:value={count}
        min="1"
        max="100"
      />
    </div>

    <div class="control-group">
      <label>格式选项</label>
      <div class="checkbox-group">
        <label class="checkbox-label">
          <input type="checkbox" bind:checked={uppercase} />
          <span>大写字母</span>
        </label>
        <label class="checkbox-label">
          <input type="checkbox" bind:checked={noDashes} />
          <span>去除横杠</span>
        </label>
      </div>
    </div>

    <div class="action-buttons">
      <button class="primary-btn" onclick={generateUuids}>
        ⟳ 生成 UUID
      </button>
      {#if generatedUuids.length > 0}
        <button class="secondary-btn" onclick={copyAll}>
          {copiedIndex === -1 ? '✓ 已复制全部' : '📋 复制全部'}
        </button>
      {/if}
    </div>
  </div>

  <div class="results">
    <div class="results-header">
      <span>生成结果 ({generatedUuids.length})</span>
    </div>
    <div class="uuid-list">
      {#each generatedUuids as uuid, index}
        <div class="uuid-item">
          <code class="uuid-value">{uuid}</code>
          <button 
            class="copy-btn"
            onclick={() => copyUuid(uuid, index)}
          >
            {copiedIndex === index ? '✓' : '📋'}
          </button>
        </div>
      {/each}
    </div>
  </div>

  <div class="info-section">
    <h3>关于 UUID</h3>
    <div class="info-grid">
      <div class="info-card">
        <h4>UUID v4</h4>
        <p>随机生成的 UUID，使用随机数作为基础。最常用的版本，碰撞概率极低。</p>
      </div>
      <div class="info-card">
        <h4>UUID v1</h4>
        <p>基于时间戳和MAC地址生成。包含时间信息，可以按生成时间排序。</p>
      </div>
    </div>
  </div>
</div>

<style>
  .uuid-tool {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }

  .controls {
    display: flex;
    flex-wrap: wrap;
    gap: 1.5rem;
    align-items: flex-end;
    padding: 1rem;
    background: var(--bg-dark);
    border-radius: var(--radius-sm);
    border: 1px solid var(--border);
  }

  .control-group {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .control-group > label {
    font-size: 0.85rem;
    font-weight: 500;
    color: var(--text-secondary);
  }

  .control-group input[type="number"] {
    width: 100px;
    text-align: center;
  }

  .radio-group, .checkbox-group {
    display: flex;
    gap: 1rem;
  }

  .radio-label, .checkbox-label {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    cursor: pointer;
    font-size: 0.9rem;
    color: var(--text-primary);
  }

  .radio-label input, .checkbox-label input {
    accent-color: var(--primary);
    width: 16px;
    height: 16px;
  }

  .action-buttons {
    display: flex;
    gap: 0.75rem;
    margin-left: auto;
  }

  .primary-btn {
    background: linear-gradient(135deg, var(--primary) 0%, var(--primary-dark) 100%) !important;
    padding: 0.75rem 1.5rem !important;
    font-weight: 600 !important;
  }

  .secondary-btn {
    background: var(--bg-hover) !important;
    border: 1px solid var(--border) !important;
    color: var(--text-primary) !important;
  }

  .secondary-btn:hover {
    background: var(--bg-active) !important;
  }

  .results {
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    overflow: hidden;
  }

  .results-header {
    padding: 0.75rem 1rem;
    background: var(--bg-dark);
    border-bottom: 1px solid var(--border);
    font-size: 0.85rem;
    color: var(--text-secondary);
  }

  .uuid-list {
    max-height: 400px;
    overflow-y: auto;
  }

  .uuid-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.75rem 1rem;
    border-bottom: 1px solid var(--border);
    transition: background 0.15s;
  }

  .uuid-item:last-child {
    border-bottom: none;
  }

  .uuid-item:hover {
    background: var(--bg-hover);
  }

  .uuid-value {
    font-family: 'JetBrains Mono', monospace;
    font-size: 0.95rem;
    color: var(--accent);
    background: transparent !important;
    padding: 0 !important;
  }

  .copy-btn {
    padding: 0.4rem 0.75rem !important;
    font-size: 0.85rem !important;
    background: var(--bg-dark) !important;
    border: 1px solid var(--border) !important;
    min-width: 48px;
  }

  .copy-btn:hover {
    border-color: var(--primary) !important;
  }

  .info-section {
    margin-top: 1rem;
  }

  .info-section h3 {
    font-size: 1rem;
    margin-bottom: 1rem;
  }

  .info-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
    gap: 1rem;
  }

  .info-card {
    padding: 1rem;
    background: var(--bg-dark);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
  }

  .info-card h4 {
    font-size: 0.9rem;
    color: var(--accent);
    margin-bottom: 0.5rem;
  }

  .info-card p {
    font-size: 0.85rem;
    line-height: 1.5;
  }
</style>

