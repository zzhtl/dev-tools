<script lang="ts">
  import QRCode from "qrcode";
  import { downloadDataUrl } from "../../lib/download";
  import Button from "../../lib/ui/Button.svelte";
  import Panel from "../../lib/ui/Panel.svelte";
  import SplitPane from "../../lib/ui/SplitPane.svelte";
  import TextArea from "../../lib/ui/TextArea.svelte";
  import Field from "../../lib/ui/Field.svelte";
  import { toast } from "../../lib/toast.svelte";

  const PRESETS = [
    { label: "网址", value: "https://example.com" },
    { label: "邮箱", value: "mailto:example@mail.com" },
    { label: "电话", value: "tel:+8612345678901" },
    { label: "WiFi", value: "WIFI:S:NetworkName;T:WPA;P:password;;" },
  ];

  const LEVELS = [
    { value: "L", label: "L 约 7%" },
    { value: "M", label: "M 约 15%" },
    { value: "Q", label: "Q 约 25%" },
    { value: "H", label: "H 约 30%" },
  ];

  let text = $state("https://example.com");
  let size = $state(256);
  let level = $state<"L" | "M" | "Q" | "H">("M");
  let dark = $state("#000000");
  let light = $state("#ffffff");

  let dataUrl = $state("");
  let error = $state("");

  $effect(() => {
    const content = text;
    const options = { width: size, margin: 2, errorCorrectionLevel: level, color: { dark, light } };
    if (!content) {
      dataUrl = "";
      error = "";
      return;
    }
    let cancelled = false;
    // 输入时防抖，避免每个按键都重绘一次
    const timer = setTimeout(() => {
      QRCode.toDataURL(content, options)
        .then((url: string) => {
          if (!cancelled) {
            dataUrl = url;
            error = "";
          }
        })
        .catch((err: unknown) => {
          if (!cancelled) {
            dataUrl = "";
            error = err instanceof Error ? err.message : String(err);
          }
        });
    }, 250);
    return () => {
      cancelled = true;
      clearTimeout(timer);
    };
  });

  async function copyImage() {
    if (!dataUrl) return;
    try {
      const base64 = dataUrl.split(",")[1];
      const bytes = Uint8Array.from(atob(base64), (char) => char.charCodeAt(0));
      const blob = new Blob([bytes], { type: "image/png" });
      // ClipboardItem 需要安全上下文；局域网 IP 访问时会走到 catch
      await navigator.clipboard.write([new ClipboardItem({ "image/png": blob })]);
      toast.success("二维码图片已复制");
    } catch {
      toast.error("当前环境不支持复制图片，请改用「下载 PNG」");
    }
  }
</script>

<div class="tool">
  <SplitPane ratio={0.55}>
    {#snippet left()}
      <Panel title="内容" grow>
        {#snippet actions()}
          <span class="count">{text.length} 字符</span>
        {/snippet}

        <TextArea bind:value={text} rows={6} placeholder="输入要编码的文本或链接…" ariaLabel="二维码内容" />

        <div class="presets">
          {#each PRESETS as preset (preset.label)}
            <Button size="sm" onclick={() => (text = preset.value)}>{preset.label}</Button>
          {/each}
        </div>

        <div class="options">
          <Field label="尺寸" inline>
            <input type="range" min="128" max="512" step="32" bind:value={size} />
            <span class="value">{size}px</span>
          </Field>

          <Field label="容错" inline>
            <select bind:value={level} aria-label="容错等级">
              {#each LEVELS as item (item.value)}
                <option value={item.value}>{item.label}</option>
              {/each}
            </select>
          </Field>

          <Field label="前景" inline>
            <input type="color" bind:value={dark} aria-label="前景色" />
          </Field>

          <Field label="背景" inline>
            <input type="color" bind:value={light} aria-label="背景色" />
          </Field>
        </div>

        <p class="tip">容错等级越高越抗遮挡，但同样内容需要更密的码点。加 logo 时用 H。</p>
      </Panel>
    {/snippet}

    {#snippet right()}
      <Panel title="预览" grow>
        {#snippet actions()}
          <Button size="sm" onclick={copyImage} disabled={!dataUrl}>复制图片</Button>
          <Button
            size="sm"
            variant="primary"
            disabled={!dataUrl}
            onclick={() => downloadDataUrl(dataUrl, "qrcode.png")}
          >
            下载 PNG
          </Button>
        {/snippet}

        <div class="stage">
          {#if error}
            <p class="error">生成失败：{error}</p>
          {:else if dataUrl}
            <img src={dataUrl} alt="二维码预览" style="width: {size}px; max-width: 100%;" />
          {:else}
            <p class="placeholder">输入内容后自动生成</p>
          {/if}
        </div>
      </Panel>
    {/snippet}
  </SplitPane>
</div>

<style>
  .tool {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-height: 0;
  }

  .count {
    font-size: var(--fs-xs);
    color: var(--color-text-muted);
    font-family: var(--font-mono);
  }

  .presets {
    display: flex;
    flex-wrap: wrap;
    gap: var(--sp-2);
    margin-top: var(--sp-3);
  }

  .options {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
    gap: var(--sp-3);
    margin-top: var(--sp-4);
  }

  .options select {
    background: var(--color-bg);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    color: var(--color-text);
    padding: 0.3rem var(--sp-2);
    font-size: var(--fs-sm);
  }

  .options input[type="range"] {
    accent-color: var(--color-primary);
    width: 100%;
  }

  .options input[type="color"] {
    width: 40px;
    height: 26px;
    padding: 0;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    background: none;
    cursor: pointer;
  }

  .value {
    font-family: var(--font-mono);
    font-size: var(--fs-xs);
    color: var(--color-text-muted);
  }

  .tip {
    margin-top: var(--sp-3);
    font-size: var(--fs-xs);
    color: var(--color-text-muted);
  }

  .stage {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: var(--sp-4);
  }

  .stage img {
    border-radius: var(--radius-sm);
  }

  .placeholder {
    color: var(--color-text-muted);
    font-size: var(--fs-md);
  }

  .error {
    color: var(--color-danger);
    font-size: var(--fs-sm);
    text-align: center;
  }
</style>
