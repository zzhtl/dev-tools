export interface WatermarkSettings {
  text: string;
  mode: "single" | "tile";
  fontSize: number;
  color: string;
  opacity: number;
  angle: number;
  x: number;
  y: number;
  spacing: number;
  bold: boolean;
}

export const DEFAULT_SETTINGS: WatermarkSettings = {
  text: "仅供内部使用",
  mode: "single",
  fontSize: 5,
  color: "#64748b",
  opacity: 35,
  angle: -30,
  x: 50,
  y: 50,
  spacing: 15,
  bold: false,
};

export function normalizeSettings(value: unknown): WatermarkSettings {
  const input = value && typeof value === "object" ? value as Record<string, unknown> : {};
  const number = (key: keyof WatermarkSettings, min: number, max: number) => {
    const n = input[key];
    return typeof n === "number" && Number.isFinite(n)
      ? Math.min(max, Math.max(min, n)) : DEFAULT_SETTINGS[key] as number;
  };
  return {
    text: typeof input.text === "string" ? input.text.slice(0, 200) : DEFAULT_SETTINGS.text,
    mode: input.mode === "tile" ? "tile" : "single",
    fontSize: number("fontSize", 1, 20),
    color: typeof input.color === "string" && /^#[0-9a-f]{6}$/i.test(input.color)
      ? input.color : DEFAULT_SETTINGS.color,
    opacity: number("opacity", 1, 100),
    angle: number("angle", -180, 180),
    x: number("x", 0, 100),
    y: number("y", 0, 100),
    spacing: number("spacing", 5, 50),
    bold: input.bold === true,
  };
}

export function assertCanvasSize(width: number, height: number) {
  if (!Number.isFinite(width) || !Number.isFinite(height) || width < 1 || height < 1
    || width > 16384 || height > 16384 || width * height > 32_000_000) {
    throw new Error("图片尺寸过大或无效，请使用边长不超过 16384 像素、总像素不超过 3200 万的图片");
  }
}

export function createCanvas(width: number, height: number): HTMLCanvasElement {
  const w = Math.ceil(width);
  const h = Math.ceil(height);
  assertCanvasSize(w, h);
  const canvas = document.createElement("canvas");
  canvas.width = w;
  canvas.height = h;
  return canvas;
}

export function context2d(canvas: HTMLCanvasElement): CanvasRenderingContext2D {
  const context = canvas.getContext("2d");
  if (!context) throw new Error("浏览器无法创建画布，请尝试缩小图片或更换浏览器");
  return context;
}

/** 位置、字号、间距均相对于页面尺寸，预览缩放不会改变导出布局。 */
export function drawWatermark(canvas: HTMLCanvasElement, settings: WatermarkSettings) {
  if (!settings.text.trim()) return;
  const ctx = context2d(canvas);
  const { width, height } = canvas;
  const shortSide = Math.min(width, height);
  const fontSize = shortSide * settings.fontSize / 100;
  const lines = settings.text.split(/\r?\n/);
  const lineHeight = fontSize * 1.3;
  ctx.save();
  ctx.font = `${settings.bold ? "700" : "400"} ${fontSize}px "PingFang SC", "Microsoft YaHei", "Noto Sans CJK SC", sans-serif`;
  ctx.textAlign = "center";
  ctx.textBaseline = "middle";
  ctx.fillStyle = settings.color;
  ctx.globalAlpha = settings.opacity / 100;
  const angle = settings.angle * Math.PI / 180;
  const draw = (x: number, y: number) => {
    ctx.save();
    ctx.translate(x, y);
    ctx.rotate(angle);
    lines.forEach((line, index) => ctx.fillText(line, 0, (index - (lines.length - 1) / 2) * lineHeight));
    ctx.restore();
  };
  const x = width * settings.x / 100;
  const y = height * settings.y / 100;
  if (settings.mode === "single") {
    draw(x, y);
  } else {
    const textWidth = Math.max(...lines.map((line) => ctx.measureText(line).width));
    const textHeight = lines.length * lineHeight;
    const gap = shortSide * settings.spacing / 100;
    const stepX = Math.abs(Math.cos(angle)) * textWidth + Math.abs(Math.sin(angle)) * textHeight + gap;
    const stepY = Math.abs(Math.sin(angle)) * textWidth + Math.abs(Math.cos(angle)) * textHeight + gap;
    for (let row = Math.floor(-y / stepY) - 1; row <= Math.ceil((height - y) / stepY) + 1; row++) {
      for (let col = Math.floor(-x / stepX) - 1; col <= Math.ceil((width - x) / stepX) + 1; col++) {
        draw(x + col * stepX, y + row * stepY);
      }
    }
  }
  ctx.restore();
}

export function canvasBlob(canvas: HTMLCanvasElement, format: "png" | "jpeg", quality = 0.92): Promise<Blob> {
  return new Promise((resolve, reject) => {
    canvas.toBlob((blob) => {
      if (blob) resolve(blob);
      else reject(new Error("图片导出失败，请尝试缩小图片"));
    }, `image/${format}`, quality);
  });
}

export function composeImage(source: CanvasImageSource, width: number, height: number,
  settings: WatermarkSettings, format: "png" | "jpeg") {
  const canvas = createCanvas(width, height);
  const ctx = context2d(canvas);
  if (format === "jpeg") {
    ctx.fillStyle = "#ffffff";
    ctx.fillRect(0, 0, canvas.width, canvas.height);
  }
  ctx.drawImage(source, 0, 0, canvas.width, canvas.height);
  drawWatermark(canvas, settings);
  return canvas;
}

export function outputName(fileName: string, extension: string, page?: number) {
  const stem = fileName.replace(/\.[^.]+$/, "") || "watermark";
  return `${stem}-watermarked${page ? `-page-${page}` : ""}.${extension}`;
}
