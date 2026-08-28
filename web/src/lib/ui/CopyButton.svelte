<script lang="ts">
  import { copyText } from "../clipboard";
  import { toast } from "../toast.svelte";
  import Button from "./Button.svelte";

  interface Props {
    /** 直接给文本，或给一个惰性取值函数 */
    value: string | (() => string);
    label?: string;
    copiedLabel?: string;
    size?: "sm" | "md";
    variant?: "primary" | "secondary" | "ghost" | "danger";
    icon?: boolean;
    title?: string;
    disabled?: boolean;
    /** 复制失败时是否弹 toast（成功一律只用按钮内联反馈，避免刷屏） */
    notifyOnError?: boolean;
  }

  let {
    value,
    label = "复制",
    copiedLabel = "已复制",
    size = "sm",
    variant = "ghost",
    icon = false,
    title,
    disabled = false,
    notifyOnError = true,
  }: Props = $props();

  let copied = $state(false);
  let timer: ReturnType<typeof setTimeout> | undefined;

  async function handleCopy() {
    const text = typeof value === "function" ? value() : value;
    if (!text) return;
    const ok = await copyText(text);
    if (!ok) {
      if (notifyOnError) toast.error("复制失败，请手动选中复制");
      return;
    }
    copied = true;
    clearTimeout(timer);
    timer = setTimeout(() => (copied = false), 1500);
  }
</script>

<Button
  {size}
  {variant}
  {icon}
  {disabled}
  title={title ?? label}
  onclick={handleCopy}
>
  {#if icon}
    <span aria-hidden="true">{copied ? "✓" : "⧉"}</span>
  {:else}
    {copied ? copiedLabel : label}
  {/if}
</Button>
