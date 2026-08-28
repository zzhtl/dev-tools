<script lang="ts">
  import { apiMultipart } from "../../lib/api";
  import { downloadBlob } from "../../lib/download";
  import { formatFileSize } from "../../lib/format";
  import Button from "../../lib/ui/Button.svelte";
  import DropZone from "../../lib/ui/DropZone.svelte";
  import Panel from "../../lib/ui/Panel.svelte";
  import Tabs from "../../lib/ui/Tabs.svelte";
  import EmptyState from "../../lib/ui/EmptyState.svelte";
  import { toast } from "../../lib/toast.svelte";

  type Status = "pending" | "converting" | "done" | "error";

  interface Item {
    id: number;
    file: File;
    previewUrl: string;
    width: number;
    height: number;
    status: Status;
    message: string;
    outName: string;
    outSize: number;
    outUrl: string;
    blob: Blob | null;
  }

  const FORMATS = [
    { value: "PNG", label: "PNG", desc: "无损，支持透明" },
    { value: "JPEG", label: "JPEG", desc: "有损，体积小" },
    { value: "WEBP", label: "WebP", desc: "本工具输出无损" },
    { value: "GIF", label: "GIF", desc: "256 色" },
    { value: "BMP", label: "BMP", desc: "无压缩位图" },
    { value: "ICO", label: "ICO", desc: "图标" },
  ];

  const PRESET_SIZES = [
    { label: "缩略图 150", width: 150, height: 150 },
    { label: "头像 400", width: 400, height: 400 },
    { label: "720p", width: 1280, height: 720 },
    { label: "1080p", width: 1920, height: 1080 },
    { label: "2K", width: 2560, height: 1440 },
    { label: "4K", width: 3840, height: 2160 },
  ];

  const ICO_SIZES = [16, 32, 48, 64, 128, 256];
  const MAX_FILE_BYTES = 100 * 1024 * 1024;
  /** 并发上限：图片解码在服务端是 CPU 密集的，放太多只会互相拖慢 */
  const CONCURRENCY = 3;

  let items = $state<Item[]>([]);
  let format = $state("PNG");
  let quality = $state(90);
  let resizeEnabled = $state(false);
  let resizeWidth = $state<number | null>(null);
  let resizeHeight = $state<number | null>(null);
  let keepAspectRatio = $state(true);
  let icoSize = $state(32);
  let converting = $state(false);

  let nextId = 0;

  function addFiles(files: File[]) {
    for (const file of files) {
      if (!file.type.startsWith("image/")) {
        toast.error(`${file.name} 不是图片文件`);
        continue;
      }
      if (file.size > MAX_FILE_BYTES) {
        toast.error(`${file.name} 超过 ${formatFileSize(MAX_FILE_BYTES)}`);
        continue;
      }
      const previewUrl = URL.createObjectURL(file);
      const item: Item = {
        id: ++nextId,
        file,
        previewUrl,
        width: 0,
        height: 0,
        status: "pending",
        message: "",
        outName: "",
        outSize: 0,
        outUrl: "",
        blob: null,
      };
      items = [...items, item];

      const probe = new Image();
      probe.onload = () => {
        item.width = probe.naturalWidth;
        item.height = probe.naturalHeight;
        if (resizeWidth === null && items.length === 1) {
          resizeWidth = probe.naturalWidth;
          resizeHeight = probe.naturalHeight;
        }
      };
      probe.src = previewUrl;
    }
  }

  function revoke(item: Item) {
    URL.revokeObjectURL(item.previewUrl);
    if (item.outUrl) URL.revokeObjectURL(item.outUrl);
  }

  function remove(id: number) {
    const item = items.find((entry) => entry.id === id);
    if (item) revoke(item);
    items = items.filter((entry) => entry.id !== id);
  }

  function clearAll() {
    items.forEach(revoke);
    items = [];
  }

  function buildOptions() {
    const options: Record<string, unknown> = { quality };
    if (resizeEnabled && (resizeWidth || resizeHeight)) {
      options.resize = {
        width: resizeWidth || undefined,
        height: resizeHeight || undefined,
        keep_aspect_ratio: keepAspectRatio,
      };
    } else if (format === "ICO") {
      options.resize = { width: icoSize, height: icoSize, keep_aspect_ratio: false };
    }
    return options;
  }

  async function convertOne(item: Item) {
    item.status = "converting";
    item.message = "";
    try {
      const form = new FormData();
      form.append("file", item.file, item.file.name);
      form.append("format", format);
      form.append("options", JSON.stringify(buildOptions()));

      const response = await apiMultipart("/api/image/convert", form);
      const blob = await response.blob();
      if (item.outUrl) URL.revokeObjectURL(item.outUrl);
      item.blob = blob;
      item.outUrl = URL.createObjectURL(blob);
      item.outName = response.headers.get("X-File-Name") ?? `converted.${format.toLowerCase()}`;
      item.outSize = Number(response.headers.get("X-File-Size")) || blob.size;
      item.status = "done";
    } catch (err) {
      item.status = "error";
      item.message = err instanceof Error ? err.message : String(err);
    }
  }

  /** 固定并发的任务队列，避免一次性把几十个请求全打出去 */
  async function runWithLimit(tasks: (() => Promise<void>)[], limit: number) {
    let cursor = 0;
    const workers = Array.from({ length: Math.min(limit, tasks.length) }, async () => {
      while (cursor < tasks.length) {
        const index = cursor++;
        await tasks[index]();
      }
    });
    await Promise.all(workers);
  }

  async function convertAll() {
    if (items.length === 0) return;
    converting = true;
    await runWithLimit(
      items.map((item) => () => convertOne(item)),
      CONCURRENCY,
    );
    converting = false;
    const failed = items.filter((item) => item.status === "error").length;
    if (failed === 0) toast.success(`已转换 ${items.length} 张图片`);
    else toast.error(`${failed} 张转换失败`);
  }

  function downloadAll() {
    for (const item of items) {
      if (item.blob && item.outName) downloadBlob(item.blob, item.outName);
    }
  }

  function syncHeight() {
    const first = items[0];
    if (!keepAspectRatio || !first?.width || !resizeWidth) return;
    resizeHeight = Math.round(resizeWidth * (first.height / first.width));
  }

  function applyPreset(width: number, height: number) {
    resizeEnabled = true;
    resizeWidth = width;
    resizeHeight = height;
  }

  const doneCount = $derived(items.filter((item) => item.status === "done").length);
  const totalIn = $derived(items.reduce((sum, item) => sum + item.file.size, 0));
  const totalOut = $derived(items.reduce((sum, item) => sum + item.outSize, 0));
</script>

<div class="tool">
  <Panel title="输出设置">
    {#snippet actions()}
      <Button size="sm" variant="ghost" onclick={clearAll} disabled={items.length === 0}>清空</Button>
      <Button size="sm" onclick={downloadAll} disabled={doneCount === 0}>下载全部</Button>
      <Button variant="primary" size="sm" onclick={convertAll} loading={converting} disabled={items.length === 0}>
        转换{items.length > 1 ? ` ${items.length} 张` : ""}
      </Button>
    {/snippet}

    <Tabs bind:value={format} tabs={FORMATS.map((f) => ({ value: f.value, label: f.label }))} ariaLabel="输出格式" />
    <p class="desc">{FORMATS.find((f) => f.value === format)?.desc}</p>

    {#if format === "JPEG"}
      <label class="opt">
        <span>质量 {quality}</span>
        <input type="range" min="1" max="100" bind:value={quality} />
      </label>
    {:else if format === "WEBP"}
      <p class="warn">
        当前使用的 <code>image</code> crate 只支持 WebP 无损编码，质量参数对 WebP 不生效；
        需要有损 WebP 请改用 JPEG，或用 <code>cwebp</code> 等工具。
      </p>
    {:else if format === "ICO"}
      <label class="opt">
        <span>图标尺寸</span>
        <select bind:value={icoSize} aria-label="ICO 尺寸">
          {#each ICO_SIZES as size (size)}
            <option value={size}>{size} × {size}</option>
          {/each}
        </select>
      </label>
    {/if}

    <label class="opt">
      <input type="checkbox" bind:checked={resizeEnabled} />
      <span>调整尺寸</span>
    </label>

    {#if resizeEnabled}
      <div class="resize">
        <label>
          宽
          <input type="number" min="1" bind:value={resizeWidth} oninput={syncHeight} aria-label="宽度" />
        </label>
        <label>
          高
          <input type="number" min="1" bind:value={resizeHeight} aria-label="高度" />
        </label>
        <label class="ratio">
          <input type="checkbox" bind:checked={keepAspectRatio} />
          保持比例
        </label>
      </div>
      <div class="presets">
        {#each PRESET_SIZES as preset (preset.label)}
          <Button size="sm" onclick={() => applyPreset(preset.width, preset.height)}>{preset.label}</Button>
        {/each}
      </div>
    {/if}
  </Panel>

  <Panel title={items.length === 0 ? "选择图片" : `图片（${items.length}）`} grow>
    {#snippet actions()}
      {#if doneCount > 0}
        <span class="stat">
          {formatFileSize(totalIn)} → {formatFileSize(totalOut)}
          {#if totalIn > 0 && totalOut > 0}
            <b class:down={totalOut < totalIn}>
              {totalOut < totalIn ? "↓" : "↑"}{Math.abs(Math.round((1 - totalOut / totalIn) * 100))}%
            </b>
          {/if}
        </span>
      {/if}
    {/snippet}

    <DropZone
      onfiles={addFiles}
      accept="image/*"
      multiple
      hint="拖拽图片到此处，或点击选择（支持多选，单文件上限 100MB）"
    />

    {#if items.length === 0}
      <EmptyState icon="◲" title="还没有图片" hint="支持 PNG / JPEG / WebP / GIF / BMP / ICO 互转" />
    {:else}
      <ul class="list">
        {#each items as item (item.id)}
          <li class={item.status}>
            <img class="thumb" src={item.outUrl || item.previewUrl} alt={item.file.name} />
            <div class="meta">
              <span class="name">{item.file.name}</span>
              <span class="sub">
                {#if item.width}{item.width} × {item.height} · {/if}{formatFileSize(item.file.size)}
                {#if item.status === "done"}
                  → {item.outName.split(".").pop()?.toUpperCase()} {formatFileSize(item.outSize)}
                  <b class:down={item.outSize < item.file.size}>
                    {item.outSize < item.file.size ? "↓" : "↑"}{Math.abs(Math.round((1 - item.outSize / item.file.size) * 100))}%
                  </b>
                {:else if item.status === "converting"}
                  · 转换中…
                {:else if item.status === "error"}
                  · <span class="err">{item.message}</span>
                {/if}
              </span>
            </div>
            <div class="ops">
              {#if item.status === "done" && item.blob}
                <Button size="sm" onclick={() => item.blob && downloadBlob(item.blob, item.outName)}>下载</Button>
              {/if}
              <Button size="sm" variant="ghost" icon onclick={() => remove(item.id)} ariaLabel="移除">✕</Button>
            </div>
          </li>
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

  .desc {
    margin-top: var(--sp-2);
    font-size: var(--fs-xs);
    color: var(--color-text-muted);
  }

  .warn {
    margin-top: var(--sp-2);
    padding: var(--sp-2) var(--sp-3);
    border: 1px solid var(--color-warning-border);
    border-radius: var(--radius-sm);
    background: var(--color-warning-bg);
    color: var(--color-text-secondary);
    font-size: var(--fs-xs);
    line-height: var(--lh-normal);
  }

  .warn code {
    font-family: var(--font-mono);
  }

  .opt {
    display: flex;
    align-items: center;
    gap: var(--sp-3);
    margin-top: var(--sp-3);
    font-size: var(--fs-sm);
    color: var(--color-text-secondary);
  }

  .opt input[type="range"] {
    flex: 1;
    accent-color: var(--color-primary);
  }

  .opt input[type="checkbox"] {
    accent-color: var(--color-primary);
  }

  .opt select {
    background: var(--color-bg);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    color: var(--color-text);
    padding: 0.3rem var(--sp-2);
    font-size: var(--fs-sm);
  }

  .resize {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: var(--sp-3);
    margin-top: var(--sp-2);
    font-size: var(--fs-sm);
    color: var(--color-text-secondary);
  }

  .resize label {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
  }

  .resize input[type="number"] {
    width: 6rem;
    background: var(--color-bg);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    color: var(--color-text);
    padding: 0.3rem var(--sp-2);
    font-family: var(--font-mono);
    font-size: var(--fs-sm);
  }

  .ratio input {
    accent-color: var(--color-primary);
  }

  .presets {
    display: flex;
    flex-wrap: wrap;
    gap: var(--sp-2);
    margin-top: var(--sp-2);
  }

  .stat {
    font-family: var(--font-mono);
    font-size: var(--fs-xs);
    color: var(--color-text-muted);
  }

  .stat b,
  .sub b {
    font-family: var(--font-mono);
    color: var(--color-warning);
  }

  .stat b.down,
  .sub b.down {
    color: var(--color-success);
  }

  .list {
    list-style: none;
    display: flex;
    flex-direction: column;
    gap: var(--sp-2);
    margin-top: var(--sp-3);
  }

  .list li {
    display: flex;
    align-items: center;
    gap: var(--sp-3);
    padding: var(--sp-2);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    background: var(--color-surface-2);
  }

  .list li.error {
    border-color: var(--color-danger-border);
  }

  .list li.converting {
    opacity: 0.7;
  }

  .thumb {
    width: 52px;
    height: 52px;
    flex-shrink: 0;
    object-fit: contain;
    border-radius: var(--radius-sm);
    background: var(--color-bg);
    border: 1px solid var(--color-border);
  }

  .meta {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .name {
    font-size: var(--fs-sm);
    color: var(--color-text);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .sub {
    font-size: var(--fs-xs);
    color: var(--color-text-muted);
    font-family: var(--font-mono);
  }

  .err {
    color: var(--color-danger);
  }

  .ops {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    flex-shrink: 0;
  }
</style>
