<script lang="ts">
  import { onDestroy } from "svelte";
  import type { PDFDocumentLoadingTask, PDFDocumentProxy } from "pdfjs-dist";
  import Button from "../../lib/ui/Button.svelte";
  import DropZone from "../../lib/ui/DropZone.svelte";
  import Panel from "../../lib/ui/Panel.svelte";
  import { downloadBlob } from "../../lib/download";
  import { formatFileSize } from "../../lib/format";
  import { readStore, writeStore } from "../../lib/storage";
  import { toast } from "../../lib/toast.svelte";
  import {
    DEFAULT_SETTINGS, assertCanvasSize, canvasBlob, composeImage, context2d, createCanvas,
    drawWatermark, normalizeSettings, outputName,
  } from "./watermark";

  type Source = { kind: "image"; file: File; bitmap: ImageBitmap }
    | { kind: "pdf"; file: File; bytes: ArrayBuffer; pdf: PDFDocumentProxy; task: PDFDocumentLoadingTask };

  const positions = [
    { label: "左上", x: 15, y: 15 }, { label: "上中", x: 50, y: 15 }, { label: "右上", x: 85, y: 15 },
    { label: "左中", x: 15, y: 50 }, { label: "居中", x: 50, y: 50 }, { label: "右中", x: 85, y: 50 },
    { label: "左下", x: 15, y: 85 }, { label: "下中", x: 50, y: 85 }, { label: "右下", x: 85, y: 85 },
  ];
  let settings = $state(normalizeSettings(readStore("watermark-settings", DEFAULT_SETTINGS)));
  const normalized = $derived(normalizeSettings(settings));
  let source = $state.raw<Source | null>(null);
  let baseCanvas = $state.raw<HTMLCanvasElement | null>(null);
  let previewCanvas = $state<HTMLCanvasElement>();
  let pageNumber = $state(1);
  let format = $state<"png" | "jpeg">("png");
  let quality = $state(92);
  let loading = $state(false);
  let rendering = $state(false);
  let exporting = $state(false);
  let progress = $state(0);
  let error = $state("");
  let dragging = $state(false);
  const busy = $derived(loading || rendering || exporting);
  const canExport = $derived(!!baseCanvas && !!normalized.text.trim() && !busy);
  let disposed = false;
  let loadingTask: PDFDocumentLoadingTask | null = null;
  let exportController: AbortController | null = null;

  function clearSource() {
    if (source?.kind === "image") source.bitmap.close();
    if (source?.kind === "pdf") void source.task.destroy().catch(() => {});
    source = null;
    if (baseCanvas) baseCanvas.width = baseCanvas.height = 0;
    baseCanvas = null;
    pageNumber = 1;
    error = "";
  }

  onDestroy(() => {
    disposed = true;
    exportController?.abort();
    if (loadingTask) void loadingTask.destroy().catch(() => {});
    clearSource();
  });

  function errorMessage(err: unknown) {
    const message = err instanceof Error ? err.message : String(err);
    if ((err instanceof Error && err.name === "PasswordException") || /encrypted/i.test(message)) {
      return "暂不支持加密 PDF，请先移除密码和编辑限制后重试";
    }
    return message;
  }

  async function openFiles(files: File[]) {
    if (busy || !files.length) return;
    const file = files[0];
    if (files.length > 1) toast.info("一次编辑一个文件，已选择第一个文件");
    const isPdf = file.type === "application/pdf" || /\.pdf$/i.test(file.name);
    if (!isPdf && !/\.(png|jpe?g|webp|bmp|gif)$/i.test(file.name)
      && !/^image\/(png|jpeg|webp|bmp|gif)$/.test(file.type)) {
      error = "请选择 PNG、JPEG、WebP、BMP、GIF 图片或 PDF 文件";
      return;
    }
    if (!file.size || file.size > 100 * 1024 * 1024) {
      error = "文件不能为空，且不能超过 100MB";
      return;
    }
    clearSource();
    loading = true;
    try {
      if (isPdf) {
        const { loadPdf } = await import("./watermark_pdf");
        const bytes = await file.arrayBuffer();
        if (disposed) return;
        loadingTask = loadPdf(bytes);
        const pdf = await loadingTask.promise;
        const task = loadingTask;
        loadingTask = null;
        if (disposed) { await task.destroy(); return; }
        source = { kind: "pdf", file, bytes, pdf, task };
      } else {
        const bitmap = await createImageBitmap(file);
        if (disposed) { bitmap.close(); return; }
        try { assertCanvasSize(bitmap.width, bitmap.height); }
        catch (err) { bitmap.close(); throw err; }
        source = { kind: "image", file, bitmap };
      }
      await renderPreview();
    } catch (err) {
      if (!disposed) error = `文件打开失败：${errorMessage(err)}`;
    } finally {
      if (loadingTask) { void loadingTask.destroy().catch(() => {}); loadingTask = null; }
      if (!disposed) loading = false;
    }
  }

  async function renderPreview() {
    if (!source) return;
    rendering = true;
    error = "";
    if (baseCanvas) baseCanvas.width = baseCanvas.height = 0;
    baseCanvas = null;
    try {
      let canvas: HTMLCanvasElement;
      if (source.kind === "image") {
        const { bitmap } = source;
        const scale = Math.min(1, 1600 / Math.max(bitmap.width, bitmap.height));
        canvas = createCanvas(bitmap.width * scale, bitmap.height * scale);
        context2d(canvas).drawImage(bitmap, 0, 0, canvas.width, canvas.height);
      } else {
        const { renderPdfPage } = await import("./watermark_pdf");
        canvas = await renderPdfPage(source.pdf, pageNumber);
      }
      if (disposed) { canvas.width = canvas.height = 0; return; }
      baseCanvas = canvas;
    } catch (err) {
      if (!disposed) error = `预览失败：${errorMessage(err)}`;
    } finally {
      if (!disposed) rendering = false;
    }
  }

  $effect(() => {
    const base = baseCanvas;
    const target = previewCanvas;
    const options = normalized;
    const outputFormat = format;
    if (!base || !target) return;
    const frame = requestAnimationFrame(() => {
      target.width = base.width;
      target.height = base.height;
      const ctx = context2d(target);
      if (outputFormat === "jpeg") {
        ctx.fillStyle = "#ffffff";
        ctx.fillRect(0, 0, target.width, target.height);
      }
      ctx.drawImage(base, 0, 0);
      drawWatermark(target, options);
    });
    return () => cancelAnimationFrame(frame);
  });

  function moveWatermark(event: PointerEvent) {
    if (!previewCanvas || busy) return;
    const rect = previewCanvas.getBoundingClientRect();
    settings.x = Math.round(Math.max(0, Math.min(100, (event.clientX - rect.left) / rect.width * 100)));
    settings.y = Math.round(Math.max(0, Math.min(100, (event.clientY - rect.top) / rect.height * 100)));
  }

  function moveWithKeys(event: KeyboardEvent) {
    if (busy) return;
    const step = event.shiftKey ? 5 : 1;
    if (!["ArrowLeft", "ArrowRight", "ArrowUp", "ArrowDown"].includes(event.key)) return;
    event.preventDefault();
    if (event.key === "ArrowLeft") settings.x = Math.max(0, settings.x - step);
    if (event.key === "ArrowRight") settings.x = Math.min(100, settings.x + step);
    if (event.key === "ArrowUp") settings.y = Math.max(0, settings.y - step);
    if (event.key === "ArrowDown") settings.y = Math.min(100, settings.y + step);
  }

  function changePage(delta: number) {
    if (busy || source?.kind !== "pdf") return;
    pageNumber = Math.max(1, Math.min(source.pdf.numPages, pageNumber + delta));
    void renderPreview();
  }

  async function exportFile(asPdf: boolean) {
    if (!canExport || !source) return;
    exporting = true;
    error = "";
    progress = 0;
    const current = source;
    const options = { ...normalized };
    const controller = new AbortController();
    exportController = controller;
    let rendered: HTMLCanvasElement | null = null;
    let output: HTMLCanvasElement | null = null;
    try {
      let blob: Blob;
      let name: string;
      if (asPdf && current.kind === "pdf") {
        const { exportWatermarkedPdf } = await import("./watermark_pdf");
        blob = await exportWatermarkedPdf(current.bytes, current.pdf, options,
          (page) => { if (!disposed) progress = page; }, controller.signal);
        name = outputName(current.file.name, "pdf");
      } else {
        let image: ImageBitmap | HTMLCanvasElement;
        if (current.kind === "pdf") {
          const { renderPdfPage } = await import("./watermark_pdf");
          rendered = await renderPdfPage(current.pdf, pageNumber, true);
          image = rendered;
        } else image = current.bitmap;
        controller.signal.throwIfAborted();
        output = composeImage(image, image.width, image.height, options, format);
        blob = await canvasBlob(output, format, quality / 100);
        name = outputName(current.file.name, format === "jpeg" ? "jpg" : "png",
          current.kind === "pdf" ? pageNumber : undefined);
      }
      if (!disposed && !controller.signal.aborted) {
        downloadBlob(blob, name);
        toast.success(`已生成 ${name}，开始下载`);
      }
    } catch (err) {
      if (!disposed && !controller.signal.aborted) error = `导出失败：${errorMessage(err)}`;
    } finally {
      if (rendered) rendered.width = rendered.height = 0;
      if (output) output.width = output.height = 0;
      exportController = null;
      if (!disposed) exporting = false;
    }
  }
</script>

<div class="tool">
  <Panel title="选择文件">
    {#snippet actions()}
      <span class="local-hint">本机处理</span>
      <Button size="sm" variant="ghost" disabled={!source || busy} onclick={clearSource}>清空文件</Button>
    {/snippet}
    <DropZone onfiles={openFiles} disabled={busy}
      accept="image/png,image/jpeg,image/webp,image/bmp,image/gif,application/pdf,.pdf,.png,.jpg,.jpeg,.webp,.bmp,.gif"
      hint="拖入图片或 PDF，或点击选择文件（单文件上限 100MB）" />
    {#if source}
      <p class="file-info">{source.file.name} · {formatFileSize(source.file.size)} ·
        {#if source.kind === "image"}{source.bitmap.width} × {source.bitmap.height} 像素
        {:else}{source.pdf.numPages} 页 PDF{/if}
      </p>
    {/if}
    {#if error}<p class="error" role="alert">{error}</p>{/if}
  </Panel>

  <div class="workspace">
    <Panel title="水印预览">
      {#snippet actions()}
        {#if source?.kind === "pdf"}
          <Button size="sm" disabled={busy || pageNumber <= 1} onclick={() => changePage(-1)}>上一页</Button>
          <span class="page-count">{pageNumber} / {source.pdf.numPages}</span>
          <Button size="sm" disabled={busy || pageNumber >= source.pdf.numPages} onclick={() => changePage(1)}>下一页</Button>
        {/if}
      {/snippet}
      <div class="preview-area">
        {#if baseCanvas}
          <button class="canvas-frame" class:dragging disabled={busy} aria-label="水印预览，点击或拖动定位，方向键微调"
            onpointerdown={(event) => {
              if (event.button !== 0) return;
              event.currentTarget.setPointerCapture(event.pointerId);
              dragging = true;
              moveWatermark(event);
            }}
            onpointermove={(event) => { if (dragging) moveWatermark(event); }}
            onpointerup={() => (dragging = false)} onpointercancel={() => (dragging = false)}
            onlostpointercapture={() => (dragging = false)} onkeydown={moveWithKeys}>
            <canvas bind:this={previewCanvas} aria-label="带水印的文件预览"></canvas>
          </button>
        {:else}
          <div class="empty">
            <span aria-hidden="true">▧</span>
            <p>{loading || rendering ? "正在加载预览…" : "选择文件后，实时预览水印效果"}</p>
          </div>
        {/if}
      </div>
      <p class="hint">点击或拖动预览可调整位置，方向键微调，Shift + 方向键快速移动。</p>
      {#if source?.kind === "image" && /\.gif$/i.test(source.file.name)}
        <p class="hint">GIF 使用首帧，导出为静态图片。</p>
      {/if}
    </Panel>

    <Panel title="自定义水印">
      {#snippet actions()}
        <Button size="sm" variant="ghost" disabled={busy} onclick={() => { settings = { ...DEFAULT_SETTINGS }; }}>重置</Button>
      {/snippet}
      <fieldset class="settings" disabled={busy}>
        <label class="control">水印文字
          <textarea rows="2" maxlength="200" bind:value={settings.text} placeholder="输入水印文字，支持换行" aria-label="水印文字"></textarea>
        </label>
        <label class="control">布局
          <select bind:value={settings.mode} aria-label="水印布局">
            <option value="single">单个水印</option><option value="tile">整页平铺</option>
          </select>
        </label>
        <div class="row">
          <label class="color-control">文字颜色 <input type="color" bind:value={settings.color} aria-label="文字颜色" /></label>
          <label class="check"><input type="checkbox" bind:checked={settings.bold} /> 粗体</label>
        </div>
        <label class="control">字号 <span>{settings.fontSize}% · 按页面短边</span>
          <input type="range" min="1" max="20" step="0.5" bind:value={settings.fontSize} aria-label="水印字号" />
        </label>
        <label class="control">不透明度 <span>{settings.opacity}%</span>
          <input type="range" min="1" max="100" bind:value={settings.opacity} aria-label="水印不透明度" />
        </label>
        <label class="control">倾斜角度 <span>{settings.angle}°</span>
          <input type="range" min="-180" max="180" bind:value={settings.angle} aria-label="水印倾斜角度" />
        </label>
        {#if settings.mode === "tile"}
          <label class="control">平铺间距 <span>{settings.spacing}%</span>
            <input type="range" min="5" max="50" bind:value={settings.spacing} aria-label="平铺间距" />
          </label>
        {/if}
        <div class="positions" aria-label="快捷位置">
          {#each positions as position (position.label)}
            <button class:active={settings.x === position.x && settings.y === position.y}
              onclick={() => { settings.x = position.x; settings.y = position.y; }}>{position.label}</button>
          {/each}
        </div>
        <label class="control">水平位置 <span>{settings.x}%</span>
          <input type="range" min="0" max="100" bind:value={settings.x} aria-label="水印水平位置" />
        </label>
        <label class="control">垂直位置 <span>{settings.y}%</span>
          <input type="range" min="0" max="100" bind:value={settings.y} aria-label="水印垂直位置" />
        </label>
        <Button size="sm" onclick={() => {
          writeStore("watermark-settings", normalized);
          if (JSON.stringify(readStore("watermark-settings", null)) === JSON.stringify(normalized)) {
            toast.success("水印设置已保存，下次打开自动恢复");
          } else toast.error("浏览器未允许保存设置，请检查本地存储权限");
        }}>保存水印设置</Button>
      </fieldset>
    </Panel>
  </div>

  <Panel title="保存与导出">
    <div class="export-row">
      <label class="format-control">图片格式
        <select bind:value={format} disabled={busy} aria-label="导出图片格式">
          <option value="png">PNG · 保留透明</option><option value="jpeg">JPEG · 白色背景</option>
        </select>
      </label>
      {#if format === "jpeg"}
        <label class="quality">质量 {quality}%
          <input type="range" min="10" max="100" bind:value={quality} disabled={busy} aria-label="JPEG 质量" />
        </label>
      {/if}
      <Button variant={source?.kind === "pdf" ? "secondary" : "primary"} disabled={!canExport} onclick={() => exportFile(false)}>
        {source?.kind === "pdf" ? "导出当前页图片" : "导出水印图片"}
      </Button>
      {#if source?.kind === "pdf"}
        <Button variant="primary" disabled={!canExport} onclick={() => exportFile(true)}>导出全部页 PDF</Button>
      {/if}
      {#if exporting}
        <span role="status">正在导出{progress && source?.kind === "pdf" ? ` ${progress} / ${source.pdf.numPages} 页` : "…"}</span>
        <Button size="sm" onclick={() => exportController?.abort()}>取消导出</Button>
      {/if}
    </div>
    <p class="hint">{source?.kind === "pdf"
      ? "PDF 全部页面使用同一组水印设置，保留原页面内容；当前页图片按 144 DPI 导出，长边最多 4096 像素。"
      : "图片按原尺寸导出为新文件，支持 PNG / JPEG / WebP / BMP / GIF 输入。"}</p>
  </Panel>
</div>

<style>
  .tool { display: flex; flex-direction: column; gap: var(--sp-4); padding-bottom: var(--sp-4); }
  .workspace { display: grid; grid-template-columns: minmax(0, 1fr) 300px; gap: var(--sp-4); }
  .preview-area { flex: 1; display: flex; align-items: center; justify-content: center; min-height: 320px; padding: var(--sp-3); background: var(--color-bg); border-radius: var(--radius-sm); overflow: hidden; }
  .canvas-frame { display: block; max-width: 100%; line-height: 0; cursor: crosshair; touch-action: none; box-shadow: var(--shadow); background-color: var(--color-surface); background-image: conic-gradient(var(--color-surface-3) 25%, transparent 0 50%, var(--color-surface-3) 0 75%, transparent 0); background-size: 16px 16px; }
  .canvas-frame.dragging { cursor: grabbing; }
  .canvas-frame:disabled { cursor: wait; }
  canvas { display: block; max-width: 100%; max-height: 680px; width: auto; height: auto; }
  .empty { display: flex; flex-direction: column; align-items: center; gap: var(--sp-3); color: var(--color-text-muted); text-align: center; }
  .empty > span { font-size: 3rem; }
  .settings { border: 0; display: flex; flex-direction: column; gap: var(--sp-3); min-width: 0; }
  .control { display: flex; flex-wrap: wrap; align-items: center; gap: var(--sp-1); font-size: var(--fs-sm); color: var(--color-text-secondary); }
  .control > span { margin-left: auto; font-size: var(--fs-xs); color: var(--color-text-muted); }
  .control > textarea, .control > select, .control > input { width: 100%; }
  textarea, select { border: 1px solid var(--color-border); border-radius: var(--radius-sm); background: var(--color-bg); padding: var(--sp-2); color: var(--color-text); }
  textarea { resize: vertical; min-height: 64px; }
  input[type="range"] { accent-color: var(--color-primary); cursor: pointer; }
  input[type="color"] { width: 36px; height: 28px; border: 1px solid var(--color-border); border-radius: var(--radius-sm); background: var(--color-bg); cursor: pointer; }
  .row, .color-control, .check, .format-control, .quality { display: flex; align-items: center; gap: var(--sp-2); }
  .row { justify-content: space-between; font-size: var(--fs-sm); }
  .positions { display: grid; grid-template-columns: repeat(3, 1fr); gap: var(--sp-1); }
  .positions button { padding: var(--sp-1); border: 1px solid var(--color-border); border-radius: var(--radius-sm); color: var(--color-text-secondary); font-size: var(--fs-xs); }
  .positions button.active { color: var(--color-primary); background: var(--color-primary-bg); border-color: var(--color-primary-border); }
  .positions button:hover:not(:disabled) { background: var(--color-surface-2); }
  .settings:disabled { opacity: 0.6; }
  .export-row { display: flex; flex-wrap: wrap; align-items: center; gap: var(--sp-3); }
  .quality input { width: 90px; }
  .hint, .file-info { margin-top: var(--sp-3); color: var(--color-text-muted); font-size: var(--fs-xs); overflow-wrap: anywhere; }
  .local-hint, .page-count { color: var(--color-text-muted); font-size: var(--fs-xs); }
  .error { margin-top: var(--sp-3); color: var(--color-danger); font-size: var(--fs-sm); overflow-wrap: anywhere; }
  @media (max-width: 1050px) {
    .workspace { grid-template-columns: minmax(0, 1fr); }
    canvas { max-height: 520px; }
  }
</style>
