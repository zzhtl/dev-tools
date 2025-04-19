<script lang="ts">
  import JsonTool from "./json_tool/json_tool.svelte";
  import CronTool from "./cron_tool/cron_tool.svelte";
  import CryptoTool from "./crypto_tool/crypto_tool.svelte";
  import CmdLinux from "./cmd_linux/cmd_linux.svelte";
  import TimeConvert from "./time_tool/time_convert.svelte";
  import QrCodeTool from "./qrcode_tool/qrcode_tool.svelte";

  let activeTab = $state("jsonTool");
</script>

<main class="container">
  <div class="tabs">
    <button
      class:active={activeTab === "jsonTool"}
      onclick={() => (activeTab = "jsonTool")}
      class="tab-button"
      >JSON Tool</button
    >
    <button
      class:active={activeTab === "cronTool"}
      onclick={() => (activeTab = "cronTool")}
      class="tab-button"
      >Cron表达式</button
    >
    <button
      class:active={activeTab === "cryptoTool"}
      onclick={() => (activeTab = "cryptoTool")}
      class="tab-button"
      >编码加解密</button
    >
    <button
      class:active={activeTab === "cmdLinux"}
      onclick={() => (activeTab = "cmdLinux")}
      class="tab-button"
      >Linux命令搜索</button
    >
    <button
      class:active={activeTab === "timeConvert"}
      onclick={() => (activeTab = "timeConvert")}
      class="tab-button"
      >时间转换</button
    >
    <button
      class:active={activeTab === "qrCodeTool"}
      onclick={() => (activeTab = "qrCodeTool")}
      class="tab-button"
      >二维码生成</button
    >
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
    {/if}
  </div>
</main>

<style>
  :root {
    --primary-color: #4f46e5;
    --primary-light: #6366f1;
    --text-color: #111827;
    --bg-color: #f9fafb;
    --card-bg: #ffffff;
    --border-radius: 8px;
    --shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -1px rgba(0, 0, 0, 0.06);
  }

  .container {
    margin: 0 auto;
    max-width: 960px;
    padding: 2rem;
    background: var(--bg-color);
    min-height: 100vh;
  }

  .tabs {
    display: flex;
    justify-content: center;
    gap: 1rem;
    margin-bottom: 2rem;
    overflow-x: auto;
    padding-bottom: 0.5rem;
  }
  .tabs::-webkit-scrollbar {
    display: none;
  }

  .tab-button {
    background: transparent;
    border: none;
    font-size: 1rem;
    font-weight: 600;
    color: var(--text-color);
    padding: 0.75rem 1.5rem;
    cursor: pointer;
    transition: all 0.3s ease;
    border-radius: var(--border-radius);
  }

  .tab-button.active {
    color: var(--primary-color);
    background: rgba(79, 70, 229, 0.1);
  }

  .tab-button:hover {
    color: var(--primary-light);
    background: rgba(99, 102, 241, 0.1);
  }

  .content {
    background: var(--card-bg);
    padding: 2rem;
    border-radius: var(--border-radius);
    box-shadow: var(--shadow);
    min-height: 400px;
  }

  @media (prefers-color-scheme: dark) {
    :root {
      --bg-color: #1f2937;
      --text-color: #f3f4f6;
      --card-bg: #111827;
    }
  }
</style>
