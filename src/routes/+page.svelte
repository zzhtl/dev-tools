<script lang="ts">
  import JsonTool from "./json_tool/json_tool.svelte";
  import CronTool from "./cron_tool/cron_tool.svelte";
  import CryptoTool from "./crypto_tool/crypto_tool.svelte";
  import CmdLinux from "./cmd_linux/cmd_linux.svelte";
  import TimeConvert from "./time_tool/time_convert.svelte";
  import QrCodeTool from "./qrcode_tool/qrcode_tool.svelte";
  import Base64Tool from "./base64_tool/base64_tool.svelte";
  import DnsTool from "./dns_tool/dns_tool.svelte";
  import RegexTool from "./regex_tool/regex_tool.svelte";
  import HtmlTool from "./html_tool/html_tool.svelte";
  import ColorTool from "./color_tool/color_tool.svelte";
  let activeTab = $state("jsonTool");
  
  const tools = [
    { id: "jsonTool", name: "JSON工具", emoji: "📋" },
    { id: "cronTool", name: "Cron表达式", emoji: "⏰" },
    { id: "cryptoTool", name: "编码加解密", emoji: "🔒" },
    { id: "cmdLinux", name: "Linux命令", emoji: "💻" },
    { id: "timeConvert", name: "时间转换", emoji: "🕒" },
    { id: "qrCodeTool", name: "二维码生成", emoji: "📱" },
    { id: "base64Tool", name: "图片转Base64", emoji: "📊" },
    { id: "dnsTool", name: "DNS解析", emoji: "🔍" },
    { id: "regexTool", name: "正则表达式", emoji: "🔠" },
    { id: "htmlTool", name: "HTML工具", emoji: "🌐" },
    { id: "colorTool", name: "颜色选择器", emoji: "🎨" }
  ];
</script>

<main class="container">
  <div class="sidebar">
    {#each tools as tool}
      <button
        class:active={activeTab === tool.id}
        onclick={() => (activeTab = tool.id)}
        class="menu-button"
      >
        <span class="emoji-icon">{tool.emoji}</span>
        <span class="name">{tool.name}</span>
      </button>
    {/each}
  </div>

  <div class="content">
    {#if activeTab === "jsonTool"}
      <JsonTool />
    {:else if activeTab === "cronTool"}
      <CronTool />
    {:else if activeTab === "cryptoTool"}
      <CryptoTool />
    {:else if activeTab === "cmdLinux"}
      <CmdLinux />
    {:else if activeTab === "timeConvert"}
      <TimeConvert />
    {:else if activeTab === "qrCodeTool"}
      <QrCodeTool />
    {:else if activeTab === "base64Tool"}
      <Base64Tool />
    {:else if activeTab === "dnsTool"}
      <DnsTool />
    {:else if activeTab === "regexTool"}
      <RegexTool />
    {:else if activeTab === "htmlTool"}
      <HtmlTool />
    {:else if activeTab === "colorTool"}
      <ColorTool />
    {/if}
  </div>
</main>

<style>
  :root {
    --primary-color: #4f46e5;
    --primary-light: #6366f1;
    --sidebar-width: 150px;
    --text-color: #111827;
    --bg-color: #f9fafb;
    --card-bg: #ffffff;
    --border-radius: 8px;
    --shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -1px rgba(0, 0, 0, 0.06);
  }

  .container {
    display: flex;
    min-height: 100vh;
    background: var(--bg-color);
  }

  .sidebar {
    width: var(--sidebar-width);
    padding: 1rem;
    background: var(--card-bg);
    border-right: 1px solid #eee;
    overflow-y: auto;
  }

  .menu-button {
    display: flex;
    align-items: center;
    width: 100%;
    padding: 0.75rem;
    margin-bottom: 0.5rem;
    background: transparent;
    border: none;
    border-radius: var(--border-radius);
    cursor: pointer;
    transition: all 0.3s ease;
    text-align: left;
    gap: 12px;
  }

  .menu-button:hover {
    background: rgba(99, 102, 241, 0.1);
  }

  .menu-button.active {
    background: rgba(79, 70, 229, 0.1);
    color: var(--primary-color);
  }

  .emoji-icon {
    font-size: 1.2rem;
    width: 24px;
    text-align: center;
  }

  .name {
    font-weight: 600;
  }

  .content {
    flex: 1;
    padding: 2rem;
    background: var(--card-bg);
    border-radius: var(--border-radius);
    box-shadow: var(--shadow);
    margin: 1rem;
    min-height: calc(100vh - 2rem);
  }

  @media (prefers-color-scheme: dark) {
    :root {
      --bg-color: #1f2937;
      --text-color: #f3f4f6;
      --card-bg: #111827;
    }
  }
</style>