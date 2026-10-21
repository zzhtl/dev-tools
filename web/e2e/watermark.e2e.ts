import { test, expect, type Download, type Page } from "@playwright/test";
import { PDFDocument, PDFName, PDFNumber, degrees } from "pdf-lib";

async function downloadBytes(download: Download) {
  const stream = await download.createReadStream();
  const chunks: Buffer[] = [];
  for await (const chunk of stream!) chunks.push(Buffer.from(chunk));
  return Buffer.concat(chunks);
}

async function setRange(page: Page, label: string, value: number) {
  await page.getByRole("slider", { name: label, exact: true }).fill(String(value));
}

async function imageFixture(page: Page) {
  return Buffer.from(await page.evaluate(() => {
    const canvas = document.createElement("canvas");
    canvas.width = 800;
    canvas.height = 600;
    const ctx = canvas.getContext("2d")!;
    ctx.fillStyle = "#2244ff";
    ctx.fillRect(0, 500, 100, 100);
    return canvas.toDataURL().split(",")[1];
  }), "base64");
}

async function pixelStats(page: Page, bytes: Buffer) {
  return page.evaluate(async (base64) => {
    const image = await createImageBitmap(await (await fetch(`data:image/png;base64,${base64}`)).blob());
    const canvas = document.createElement("canvas");
    canvas.width = image.width;
    canvas.height = image.height;
    const ctx = canvas.getContext("2d")!;
    ctx.drawImage(image, 0, 0);
    const pixels = ctx.getImageData(0, 0, canvas.width, canvas.height).data;
    let red = 0, x = 0, y = 0;
    for (let i = 0; i < pixels.length; i += 4) {
      if (pixels[i] > 180 && pixels[i + 1] < 100 && pixels[i + 2] < 100 && pixels[i + 3] > 100) {
        red++;
        x += (i / 4) % canvas.width;
        y += Math.floor(i / 4 / canvas.width);
      }
    }
    image.close();
    return { width: canvas.width, height: canvas.height, corner: [...pixels.slice(0, 4)],
      red, x: x / red / canvas.width, y: y / red / canvas.height };
  }, bytes.toString("base64"));
}

async function configureWatermark(page: Page) {
  await page.getByRole("textbox", { name: "水印文字", exact: true }).fill("自定义水印\nWATERMARK");
  await page.getByLabel("文字颜色", { exact: true }).fill("#ff0000");
  await setRange(page, "水印不透明度", 100);
  await setRange(page, "水印字号", 8);
  await setRange(page, "水印倾斜角度", -35);
}

test.beforeEach(async ({ page }) => {
  await page.goto("/#/watermark");
  await expect(page.getByRole("textbox", { name: "水印文字", exact: true })).toBeVisible();
});

test("拖入透明图片，拖动与键盘定位，导出 PNG/JPEG 并恢复已保存设置", async ({ page }) => {
  const errors: string[] = [];
  page.on("pageerror", (error) => errors.push(error.message));
  const fixture = await imageFixture(page);
  await page.locator(".zone").evaluate((zone, base64) => {
    const transfer = new DataTransfer();
    transfer.items.add(new File([Uint8Array.from(atob(base64), (c) => c.charCodeAt(0))], "透明图片.png", { type: "image/png" }));
    zone.dispatchEvent(new DragEvent("drop", { bubbles: true, cancelable: true, dataTransfer: transfer }));
  }, fixture.toString("base64"));
  await expect(page.getByRole("button", { name: "导出水印图片", exact: true })).toBeEnabled();
  await configureWatermark(page);
  const preview = page.getByRole("button", { name: "水印预览，点击或拖动定位，方向键微调", exact: true });
  const box = (await preview.boundingBox())!;
  await page.mouse.move(box.x + box.width * 0.7, box.y + box.height * 0.65);
  await page.mouse.down();
  await page.mouse.move(box.x + box.width * 0.3, box.y + box.height * 0.4, { steps: 5 });
  await page.mouse.up();
  await preview.press("ArrowRight");
  await expect(page.getByRole("slider", { name: "水印水平位置", exact: true })).toHaveValue("31");
  await expect(page.getByRole("slider", { name: "水印垂直位置", exact: true })).toHaveValue("40");
  await page.screenshot({ path: test.info().outputPath("watermark-image.png") });
  let pending = page.waitForEvent("download");
  await page.getByRole("button", { name: "导出水印图片", exact: true }).click();
  let download = await pending;
  expect(download.suggestedFilename()).toBe("透明图片-watermarked.png");
  const png = await pixelStats(page, await downloadBytes(download));
  expect([png.width, png.height]).toEqual([800, 600]);
  expect(png.corner[3]).toBe(0);
  expect(png.red).toBeGreaterThan(1000);
  expect(png.x).toBeCloseTo(0.31, 1);
  expect(png.y).toBeCloseTo(0.4, 1);
  await page.getByLabel("导出图片格式", { exact: true }).selectOption("jpeg");
  pending = page.waitForEvent("download");
  await page.getByRole("button", { name: "导出水印图片", exact: true }).click();
  download = await pending;
  expect(download.suggestedFilename()).toBe("透明图片-watermarked.jpg");
  const jpeg = await pixelStats(page, await downloadBytes(download));
  expect(jpeg.corner).toEqual([255, 255, 255, 255]);
  expect(jpeg.red).toBeGreaterThan(1000);
  await page.getByRole("button", { name: "保存水印设置", exact: true }).click();
  await page.reload();
  await expect(page.getByRole("textbox", { name: "水印文字", exact: true })).toHaveValue("自定义水印\nWATERMARK");
  await expect(page.getByRole("slider", { name: "水印水平位置", exact: true })).toHaveValue("31");
  await expect(page.getByRole("button", { name: "导出水印图片", exact: true })).toBeDisabled();
  expect(errors).toEqual([]);
});

test("平铺导出、空文字校验、损坏文件恢复与移动端布局", async ({ page }) => {
  await page.locator("input[type=file]").setInputFiles({ name: "test.png", mimeType: "image/png", buffer: await imageFixture(page) });
  await expect(page.getByRole("button", { name: "导出水印图片", exact: true })).toBeEnabled();
  await configureWatermark(page);
  await page.getByLabel("水印布局", { exact: true }).selectOption("tile");
  await setRange(page, "水印字号", 4);
  await setRange(page, "平铺间距", 10);
  const pending = page.waitForEvent("download");
  await page.getByRole("button", { name: "导出水印图片", exact: true }).click();
  const tiled = await pixelStats(page, await downloadBytes(await pending));
  expect(tiled.red).toBeGreaterThan(4000);
  expect(tiled.x).toBeGreaterThan(0.35);
  expect(tiled.x).toBeLessThan(0.65);
  await page.getByRole("textbox", { name: "水印文字", exact: true }).fill("   ");
  await expect(page.getByRole("button", { name: "导出水印图片", exact: true })).toBeDisabled();
  await page.locator("input[type=file]").setInputFiles({ name: "broken.pdf", mimeType: "application/pdf", buffer: Buffer.from("broken") });
  await expect(page.getByRole("alert")).toContainText("文件打开失败");
  await page.locator("input[type=file]").setInputFiles({ name: "valid.png", mimeType: "image/png", buffer: await imageFixture(page) });
  await page.getByRole("button", { name: "重置", exact: true }).click();
  await expect(page.getByRole("button", { name: "导出水印图片", exact: true })).toBeEnabled();
  await page.setViewportSize({ width: 390, height: 844 });
  await page.getByRole("button", { name: "收起侧边栏", exact: true }).click();
  const overflow = await page.locator(".content").evaluate((el) => el.scrollWidth > el.clientWidth);
  expect(overflow).toBe(false);
});

test("多页 PDF 离线预览和导出保留正文，裁剪与旋转页面水印位置一致", async ({ page }) => {
  const errors: string[] = [];
  const external: string[] = [];
  const failedResources: string[] = [];
  page.on("pageerror", (error) => errors.push(error.message));
  page.on("response", (response) => { if (response.status() >= 400) failedResources.push(response.url()); });
  await page.route("**/*", (route) => {
    const url = route.request().url();
    if (url.startsWith("http://127.0.0.1:5173/")) return route.continue();
    external.push(url);
    return route.abort();
  });
  const fixture = await PDFDocument.create();
  for (const [index, angle] of [0, 90, 180, 270, 90].entries()) {
    const pdfPage = fixture.addPage([420 + index * 10, 640]);
    pdfPage.setRotation(degrees(angle));
    pdfPage.drawText(`Original page ${index + 1}`, { x: 100, y: 200, size: 16 });
    if (index === 4) {
      pdfPage.setCropBox(40, 70, 350, 500);
      pdfPage.node.set(PDFName.of("UserUnit"), PDFNumber.of(2));
    }
  }
  await page.locator("input[type=file]").setInputFiles({ name: "mixed.pdf", mimeType: "application/pdf", buffer: Buffer.from(await fixture.save()) });
  await expect(page.getByRole("button", { name: "导出全部页 PDF", exact: true })).toBeEnabled();
  await configureWatermark(page);
  await setRange(page, "水印水平位置", 65);
  await setRange(page, "水印垂直位置", 35);
  const previewStats = [];
  for (let index = 0; index < 5; index++) {
    if (index) await page.getByRole("button", { name: "下一页", exact: true }).click();
    await expect(page.getByRole("button", { name: "导出当前页图片", exact: true })).toBeEnabled();
    const pending = page.waitForEvent("download");
    await page.getByRole("button", { name: "导出当前页图片", exact: true }).click();
    const download = await pending;
    expect(download.suggestedFilename()).toBe(`mixed-watermarked-page-${index + 1}.png`);
    previewStats.push(await pixelStats(page, await downloadBytes(download)));
  }
  const pending = page.waitForEvent("download");
  await page.getByRole("button", { name: "导出全部页 PDF", exact: true }).click();
  const download = await pending;
  expect(download.suggestedFilename()).toBe("mixed-watermarked.pdf");
  const bytes = await downloadBytes(download);
  const exported = await PDFDocument.load(bytes);
  expect(exported.getPageCount()).toBe(5);
  expect(exported.getPages().map((p) => p.getRotation().angle)).toEqual([0, 90, 180, 270, 90]);
  expect(exported.getPage(4).getCropBox()).toEqual({ x: 40, y: 70, width: 350, height: 500 });
  const renderedPages = await page.evaluate(async (base64) => {
    const { loadPdf, renderPdfPage } = await import("/src/tools/watermark_tool/watermark_pdf.ts");
    const task = loadPdf(Uint8Array.from(atob(base64), (c) => c.charCodeAt(0)).buffer);
    const pdf = await task.promise;
    const result = [];
    for (let n = 1; n <= pdf.numPages; n++) {
      const canvas = await renderPdfPage(pdf, n, true);
      const text = await (await pdf.getPage(n)).getTextContent();
      result.push({ png: canvas.toDataURL().split(",")[1], text: text.items.map((item: { str?: string }) => item.str ?? "").join(" ") });
      canvas.width = canvas.height = 0;
    }
    await task.destroy();
    return result;
  }, bytes.toString("base64"));
  for (let index = 0; index < renderedPages.length; index++) {
    expect(renderedPages[index].text).toContain(`Original page ${index + 1}`);
    const actual = await pixelStats(page, Buffer.from(renderedPages[index].png, "base64"));
    expect(actual.red).toBeGreaterThan(1000);
    expect(actual.width).toBe(previewStats[index].width);
    expect(actual.height).toBe(previewStats[index].height);
    expect(Math.abs(actual.x - previewStats[index].x)).toBeLessThan(0.005);
    expect(Math.abs(actual.y - previewStats[index].y)).toBeLessThan(0.005);
  }
  expect(errors).toEqual([]);
  expect(external).toEqual([]);
  expect(failedResources).toEqual([]);
});

test("取消 PDF 导出后可继续编辑，切换工具会释放正在处理的文件", async ({ page }) => {
  const errors: string[] = [];
  const downloads: string[] = [];
  page.on("pageerror", (error) => errors.push(error.message));
  page.on("download", (download) => downloads.push(download.suggestedFilename()));
  const fixture = await PDFDocument.create();
  for (let index = 0; index < 30; index++) fixture.addPage([600, 800]);
  await page.locator("input[type=file]").setInputFiles({ name: "many.pdf", mimeType: "application/pdf", buffer: Buffer.from(await fixture.save()) });
  await expect(page.getByRole("button", { name: "导出全部页 PDF", exact: true })).toBeEnabled();
  await page.getByRole("button", { name: "导出全部页 PDF", exact: true }).click();
  await page.getByRole("button", { name: "取消导出", exact: true }).click();
  await expect(page.getByRole("button", { name: "导出全部页 PDF", exact: true })).toBeEnabled();
  expect(downloads).toEqual([]);
  await page.getByRole("button", { name: "导出全部页 PDF", exact: true }).click();
  await page.evaluate(() => { location.hash = "/json"; });
  await expect(page.getByRole("textbox", { name: "水印文字", exact: true })).toHaveCount(0);
  await page.evaluate(() => { location.hash = "/watermark"; });
  await page.locator("input[type=file]").setInputFiles({ name: "new.png", mimeType: "image/png", buffer: await imageFixture(page) });
  await expect(page.getByRole("button", { name: "导出水印图片", exact: true })).toBeEnabled();
  expect(downloads).toEqual([]);
  expect(errors).toEqual([]);
});
