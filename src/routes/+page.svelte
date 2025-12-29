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
  import ImageTool from "./image_tool/image_tool.svelte";
  import UuidTool from "./uuid_tool/uuid_tool.svelte";
  import HashTool from "./hash_tool/hash_tool.svelte";
  import UrlTool from "./url_tool/url_tool.svelte";

  let activeTab = $state("jsonTool");
  let sidebarCollapsed = $state(false);

  const toolCategories = [
    {
      name: "数据处理",
      tools: [
        { id: "jsonTool", name: "JSON 工具", icon: "{ }", desc: "格式化、压缩、转Go结构体" },
        { id: "regexTool", name: "正则表达式", icon: ".*", desc: "正则测试与匹配" },
        { id: "htmlTool", name: "HTML 工具", icon: "</>", desc: "格式化、压缩、预览" },
      ]
    },
    {
      name: "编码加密",
      tools: [
        { id: "cryptoTool", name: "加密解密", icon: "🔐", desc: "AES/RSA/DES加解密" },
        { id: "base64Tool", name: "Base64", icon: "📄", desc: "图片转Base64编码" },
        { id: "hashTool", name: "Hash 计算", icon: "#", desc: "MD5/SHA哈希计算" },
        { id: "urlTool", name: "URL 编解码", icon: "%", desc: "URL编码与解码" },
      ]
    },
    {
      name: "生成工具",
      tools: [
        { id: "qrCodeTool", name: "二维码", icon: "▣", desc: "生成二维码" },
        { id: "uuidTool", name: "UUID", icon: "⊕", desc: "生成UUID标识符" },
        { id: "colorTool", name: "颜色选择器", icon: "◐", desc: "颜色转换与选择" },
      ]
    },
    {
      name: "时间日期",
      tools: [
        { id: "timeConvert", name: "时间转换", icon: "⏱", desc: "时间戳与日期转换" },
        { id: "cronTool", name: "Cron 表达式", icon: "⏰", desc: "Cron表达式解析" },
      ]
    },
    {
      name: "网络工具",
      tools: [
        { id: "dnsTool", name: "DNS 解析", icon: "◎", desc: "域名DNS查询" },
      ]
    },
    {
      name: "系统工具",
      tools: [
        { id: "cmdLinux", name: "Linux 命令", icon: "$_", desc: "Linux命令查询" },
        { id: "imageTool", name: "图片转换", icon: "◲", desc: "图片格式转换" },
      ]
    }
  ];

  function getActiveTool() {
    for (const category of toolCategories) {
      const tool = category.tools.find(t => t.id === activeTab);
      if (tool) return tool;
    }
    return null;
  }

  function toggleSidebar() {
    sidebarCollapsed = !sidebarCollapsed;
  }
</script>

<main class="app-container">
  <aside class="sidebar" class:collapsed={sidebarCollapsed}>
    <div class="sidebar-header">
      <div class="logo">
        {#if !sidebarCollapsed}
          <span class="logo-icon">⚡</span>
          <span class="logo-text">开发工具箱</span>
        {:else}
          <span class="logo-icon">⚡</span>
        {/if}
      </div>
      <button class="collapse-btn" onclick={toggleSidebar} title={sidebarCollapsed ? "展开菜单" : "收起菜单"}>
        {sidebarCollapsed ? "→" : "←"}
      </button>
    </div>

    <nav class="nav-menu">
      {#each toolCategories as category}
        <div class="category">
          {#if !sidebarCollapsed}
            <div class="category-title">{category.name}</div>
          {/if}
          <div class="category-tools">
            {#each category.tools as tool}
              <button
                class="nav-item"
                class:active={activeTab === tool.id}
                onclick={() => (activeTab = tool.id)}
                title={sidebarCollapsed ? `${tool.name}: ${tool.desc}` : tool.desc}
              >
                <span class="nav-icon">{tool.icon}</span>
                {#if !sidebarCollapsed}
                  <span class="nav-text">{tool.name}</span>
                {/if}
              </button>
            {/each}
          </div>
        </div>
      {/each}
    </nav>
  </aside>

  <div class="main-content">
    <header class="content-header">
      <div class="header-info">
        {#if getActiveTool()}
          <h1 class="page-title">
            <span class="title-icon">{getActiveTool()?.icon}</span>
            {getActiveTool()?.name}
          </h1>
          <p class="page-desc">{getActiveTool()?.desc}</p>
        {/if}
      </div>
    </header>

    <div class="content-body">
      <div class="tool-container">
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
        {:else if activeTab === "imageTool"}
          <ImageTool />
        {:else if activeTab === "uuidTool"}
          <UuidTool />
        {:else if activeTab === "hashTool"}
          <HashTool />
        {:else if activeTab === "urlTool"}
          <UrlTool />
        {/if}
      </div>
    </div>
  </div>
</main>

<style>
  :global(*) {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
  }

  :global(body) {
    font-family: 'Noto Sans SC', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
    background: #0f0f23;
    color: #e4e4e7;
    overflow: hidden;
  }

  :global(:root) {
    --primary: #6366f1;
    --primary-light: #818cf8;
    --primary-dark: #4f46e5;
    --accent: #22d3ee;
    --accent-green: #10b981;
    --bg-dark: #0f0f23;
    --bg-card: #18182f;
    --bg-hover: #1e1e3f;
    --bg-active: #252550;
    --border: #2d2d5a;
    --text-primary: #f4f4f5;
    --text-secondary: #a1a1aa;
    --text-muted: #71717a;
    --shadow: 0 4px 20px rgba(0, 0, 0, 0.3);
    --radius: 12px;
    --radius-sm: 8px;
    --transition: all 0.2s ease;
  }

  .app-container {
    display: flex;
    height: 100vh;
    width: 100vw;
    overflow: hidden;
  }

  /* 侧边栏 */
  .sidebar {
    width: 240px;
    background: linear-gradient(180deg, #18182f 0%, #0f0f23 100%);
    border-right: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    transition: width 0.3s ease;
    position: relative;
    z-index: 10;
  }

  .sidebar.collapsed {
    width: 72px;
  }

  .sidebar-header {
    padding: 1.25rem;
    display: flex;
    align-items: center;
    justify-content: space-between;
    border-bottom: 1px solid var(--border);
    min-height: 68px;
  }

  .logo {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    overflow: hidden;
  }

  .logo-icon {
    font-size: 1.5rem;
    min-width: 32px;
    text-align: center;
    filter: drop-shadow(0 0 8px rgba(99, 102, 241, 0.5));
  }

  .logo-text {
    font-weight: 700;
    font-size: 1.1rem;
    background: linear-gradient(135deg, var(--primary-light) 0%, var(--accent) 100%);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
    white-space: nowrap;
  }

  .collapse-btn {
    width: 28px;
    height: 28px;
    border: 1px solid var(--border);
    border-radius: 6px;
    background: var(--bg-card);
    color: var(--text-secondary);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 0.8rem;
    transition: var(--transition);
  }

  .collapse-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
    border-color: var(--primary);
  }

  .nav-menu {
    flex: 1;
    overflow-y: auto;
    padding: 1rem 0.75rem;
  }

  .nav-menu::-webkit-scrollbar {
    width: 4px;
  }

  .nav-menu::-webkit-scrollbar-track {
    background: transparent;
  }

  .nav-menu::-webkit-scrollbar-thumb {
    background: var(--border);
    border-radius: 2px;
  }

  .category {
    margin-bottom: 1.25rem;
  }

  .category-title {
    font-size: 0.7rem;
    text-transform: uppercase;
    letter-spacing: 0.1em;
    color: var(--text-muted);
    padding: 0 0.75rem;
    margin-bottom: 0.5rem;
    font-weight: 600;
  }

  .category-tools {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .nav-item {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.65rem 0.75rem;
    border: none;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--text-secondary);
    cursor: pointer;
    transition: var(--transition);
    font-size: 0.9rem;
    text-align: left;
    width: 100%;
  }

  .nav-item:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .nav-item.active {
    background: linear-gradient(135deg, rgba(99, 102, 241, 0.2) 0%, rgba(34, 211, 238, 0.1) 100%);
    color: var(--accent);
    border-left: 3px solid var(--accent);
    margin-left: -3px;
  }

  .nav-icon {
    min-width: 28px;
    height: 28px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 0.95rem;
    font-family: 'JetBrains Mono', monospace;
    font-weight: 600;
  }

  .nav-text {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  /* 主内容区 */
  .main-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    background: var(--bg-dark);
  }

  .content-header {
    padding: 1.25rem 2rem;
    background: var(--bg-card);
    border-bottom: 1px solid var(--border);
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .header-info {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .page-title {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    font-size: 1.35rem;
    font-weight: 600;
    color: var(--text-primary);
  }

  .title-icon {
    font-size: 1.1rem;
    font-family: 'JetBrains Mono', monospace;
    color: var(--accent);
  }

  .page-desc {
    font-size: 0.85rem;
    color: var(--text-muted);
  }

  .content-body {
    flex: 1;
    overflow: auto;
    padding: 1.5rem 2rem;
  }

  .content-body::-webkit-scrollbar {
    width: 8px;
  }

  .content-body::-webkit-scrollbar-track {
    background: var(--bg-dark);
  }

  .content-body::-webkit-scrollbar-thumb {
    background: var(--border);
    border-radius: 4px;
  }

  .content-body::-webkit-scrollbar-thumb:hover {
    background: var(--text-muted);
  }

  .tool-container {
    background: var(--bg-card);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 1.5rem;
    min-height: calc(100vh - 180px);
  }

  /* 全局样式覆盖 - 确保所有工具组件继承主题 */
  :global(.tool-container input),
  :global(.tool-container textarea),
  :global(.tool-container select) {
    background: var(--bg-dark);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    color: var(--text-primary);
    padding: 0.75rem 1rem;
    font-family: inherit;
    font-size: 0.9rem;
    transition: var(--transition);
  }

  :global(.tool-container input:focus),
  :global(.tool-container textarea:focus),
  :global(.tool-container select:focus) {
    outline: none;
    border-color: var(--primary);
    box-shadow: 0 0 0 3px rgba(99, 102, 241, 0.2);
  }

  :global(.tool-container input::placeholder),
  :global(.tool-container textarea::placeholder) {
    color: var(--text-muted);
  }

  :global(.tool-container button) {
    background: linear-gradient(135deg, var(--primary) 0%, var(--primary-dark) 100%);
    color: white;
    border: none;
    border-radius: var(--radius-sm);
    padding: 0.65rem 1.25rem;
    font-weight: 500;
    cursor: pointer;
    transition: var(--transition);
  }

  :global(.tool-container button:hover) {
    transform: translateY(-1px);
    box-shadow: 0 4px 12px rgba(99, 102, 241, 0.4);
  }

  :global(.tool-container button:active) {
    transform: translateY(0);
  }

  :global(.tool-container h1),
  :global(.tool-container h2),
  :global(.tool-container h3) {
    color: var(--text-primary);
  }

  :global(.tool-container p),
  :global(.tool-container label) {
    color: var(--text-secondary);
  }

  :global(.tool-container code) {
    font-family: 'JetBrains Mono', monospace;
    background: var(--bg-dark);
    padding: 0.2rem 0.5rem;
    border-radius: 4px;
    font-size: 0.85rem;
  }

  :global(.tool-container pre) {
    font-family: 'JetBrains Mono', monospace;
    background: var(--bg-dark);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    padding: 1rem;
    overflow-x: auto;
  }
</style>
