import { describe, expect, test } from "bun:test";
import { DEFAULT_SETTINGS, assertCanvasSize, normalizeSettings, outputName } from "./watermark";

describe("水印输入与导出边界", () => {
  test("损坏或旧版本本地设置回退到可用值", () => {
    expect(normalizeSettings(null)).toEqual(DEFAULT_SETTINGS);
    expect(normalizeSettings("invalid")).toEqual(DEFAULT_SETTINGS);
    const settings = normalizeSettings({ text: "a".repeat(300), fontSize: NaN, opacity: 0,
      angle: 999, x: -30, y: Infinity, spacing: 0, color: "url(external)", mode: "unknown" });
    expect(settings.text.length).toBe(200);
    expect(settings.fontSize).toBe(DEFAULT_SETTINGS.fontSize);
    expect(settings.opacity).toBe(1);
    expect(settings.angle).toBe(180);
    expect(settings.x).toBe(0);
    expect(settings.y).toBe(50);
    expect(settings.spacing).toBe(5);
    expect(settings.color).toBe(DEFAULT_SETTINGS.color);
    expect(settings.mode).toBe("single");
  });

  test("允许透明图的常见尺寸，拒绝浏览器无法可靠导出的画布", () => {
    expect(() => assertCanvasSize(6000, 4000)).not.toThrow();
    for (const [width, height] of [[0, 100], [NaN, 100], [16385, 1], [8000, 8000]]) {
      expect(() => assertCanvasSize(width, height)).toThrow();
    }
  });

  test("导出文件名保留原始名称，PDF 当前页图片包含页号", () => {
    expect(outputName("合同.v2.pdf", "pdf")).toBe("合同.v2-watermarked.pdf");
    expect(outputName("合同.v2.pdf", "png", 3)).toBe("合同.v2-watermarked-page-3.png");
    expect(outputName("image", "jpg")).toBe("image-watermarked.jpg");
  });
});
