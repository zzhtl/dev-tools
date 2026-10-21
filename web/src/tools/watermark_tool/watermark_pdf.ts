import { getDocument, GlobalWorkerOptions, type PDFDocumentProxy, type PDFPageProxy } from "pdfjs-dist";
import workerUrl from "pdfjs-dist/build/pdf.worker.min.mjs?url";
import { canvasBlob, createCanvas, drawWatermark, type WatermarkSettings } from "./watermark";

GlobalWorkerOptions.workerSrc = workerUrl;

export function loadPdf(bytes: ArrayBuffer) {
  return getDocument({
    // Worker 会转移缓冲区；保留原始数据供 PDF 导出使用。
    data: new Uint8Array(bytes.slice(0)),
    cMapUrl: new URL("pdfjs/cmaps/", document.baseURI).href,
    cMapPacked: true,
    standardFontDataUrl: new URL("pdfjs/standard_fonts/", document.baseURI).href,
    wasmUrl: new URL("pdfjs/wasm/", document.baseURI).href,
    iccUrl: new URL("pdfjs/iccs/", document.baseURI).href,
  });
}

function pageViewport(page: PDFPageProxy, maxEdge: number, scale: number) {
  const base = page.getViewport({ scale: 1 });
  return page.getViewport({ scale: Math.min(scale, maxEdge / Math.max(base.width, base.height)) });
}

export async function renderPdfPage(pdf: PDFDocumentProxy, pageNumber: number, forExport = false) {
  const page = await pdf.getPage(pageNumber);
  const viewport = pageViewport(page, forExport ? 4096 : 1600, 2);
  const canvas = createCanvas(viewport.width, viewport.height);
  // 将小数尺寸精确缩放到整数画布，保证预览和水印层使用相同坐标。
  await page.render({
    canvas,
    viewport,
    transform: [canvas.width / viewport.width, 0, 0, canvas.height / viewport.height, 0, 0],
  }).promise;
  return canvas;
}

export async function exportWatermarkedPdf(bytes: ArrayBuffer, pdf: PDFDocumentProxy,
  settings: WatermarkSettings, onProgress: (page: number) => void, signal: AbortSignal) {
  const { PDFDocument, degrees } = await import("pdf-lib");
  const output = await PDFDocument.load(bytes);
  for (let index = 0; index < pdf.numPages; index++) {
    signal.throwIfAborted();
    const sourcePage = await pdf.getPage(index + 1);
    const viewport = pageViewport(sourcePage, 4096, 2);
    const overlay = createCanvas(viewport.width, viewport.height);
    try {
      drawWatermark(overlay, settings);
      const png = await canvasBlob(overlay, "png");
      const image = await output.embedPng(await png.arrayBuffer());
      // 由显示坐标反算 PDF 坐标，兼容 CropBox、UserUnit 和 90/180/270 度旋转。
      const bottomLeft = viewport.convertToPdfPoint(0, viewport.height);
      const bottomRight = viewport.convertToPdfPoint(viewport.width, viewport.height);
      const topLeft = viewport.convertToPdfPoint(0, 0);
      output.getPage(index).drawImage(image, {
        x: bottomLeft[0],
        y: bottomLeft[1],
        width: Math.hypot(bottomRight[0] - bottomLeft[0], bottomRight[1] - bottomLeft[1]),
        height: Math.hypot(topLeft[0] - bottomLeft[0], topLeft[1] - bottomLeft[1]),
        rotate: degrees(viewport.rotation),
      });
    } finally {
      overlay.width = overlay.height = 0;
    }
    // 分页嵌入后释放 PNG 缓存，且让进度和取消操作有机会刷新。
    await output.flush();
    onProgress(index + 1);
    await new Promise((resolve) => setTimeout(resolve, 0));
  }
  signal.throwIfAborted();
  return new Blob([new Uint8Array(await output.save())], { type: "application/pdf" });
}
