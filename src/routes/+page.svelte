<script lang="ts">
  import JsonTool from "./json_tool/json_tool.svelte";
  import CronTool from "./cron_tool/cron_tool.svelte";
  import CryptoTool from "./crypto_tool/crypto_tool.svelte";
  import CmdLinux from "./cmd_linux/cmd_linux.svelte";
  import TimeConvert from "./time_tool/time_convert.svelte";

  let activeTab = $state("jsonTool");
</script>

<main class="container">
  <div class="tabs">
    <button
      class:active={activeTab === "jsonTool"}
      on:click={() => (activeTab = "jsonTool")}>JSON Tool</button
    >
    <button
      class:active={activeTab === "cronTool"}
      on:click={() => (activeTab = "cronTool")}>Cron表达式</button
    >
    <button
      class:active={activeTab === "cryptoTool"}
      on:click={() => (activeTab = "cryptoTool")}>编码加解密</button
    >
    <button
      class:active={activeTab === "cmdLinux"}
      on:click={() => (activeTab = "cmdLinux")}>linux命令搜索</button
    >
    <button
      class:active={activeTab === "timeConvert"}
      on:click={() => (activeTab = "timeConvert")}>时间转换</button
    >
  </div>

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
  {/if}
</main>

<style>
  :root {
    font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
    font-size: 16px;
    line-height: 24px;
    font-weight: 400;

    color: #0f0f0f;
    background-color: #f6f6f6;

    font-synthesis: none;
    text-rendering: optimizeLegibility;
    -webkit-font-smoothing: antialiased;
    -moz-osx-font-smoothing: grayscale;
    -webkit-text-size-adjust: 100%;
  }

  .container {
    margin: 0;
    padding-top: 10px;
    display: flex;
    flex-direction: column;
    justify-content: center;
    text-align: center;
  }

  .tabs {
    display: flex;
    justify-content: center;
    gap: 10px;
    margin-bottom: 10px;
  }

  .tabs button {
    padding: 12px 24px;
    border: none;
    border-radius: 8px;
    background-color: #fff;
    cursor: pointer;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
    transition: all 0.3s ease;
  }

  .tabs button.active {
    background-color: #396cd8;
    color: #fff;
    box-shadow: 0 4px 8px rgba(0, 0, 0, 0.2);
  }

  .tabs button:hover {
    transform: translateY(-2px);
    box-shadow: 0 4px 8px rgba(0, 0, 0, 0.2);
  }

  .tabs button:active {
    transform: translateY(0);
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
  }

  button {
    cursor: pointer;
  }

  button:hover {
    border-color: #396cd8;
  }
  button:active {
    border-color: #396cd8;
    background-color: #e8e8e8;
  }

  button {
    outline: none;
  }

  @media (prefers-color-scheme: dark) {
    :root {
      color: #f6f6f6;
      background-color: #2f2f2f;
    }

    button {
      color: #ffffff;
      background-color: #0f0f0f98;
    }
    button:active {
      background-color: #0f0f0f69;
    }
  }
</style>
