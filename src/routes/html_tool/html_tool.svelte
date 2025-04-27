<script lang="ts">
  // HTML工具组件 - 提供HTML格式化、压缩、编辑预览等功能
  
  // 响应式状态变量
  let htmlInput = $state("");
  let htmlOutput = $state("");
  let activeTab = $state("format"); // 'format', 'minify', 'preview'
  
  // 格式化HTML
  function formatHtml() {
    try {
      // 使用DOMParser解析HTML
      const parser = new DOMParser();
      const doc = parser.parseFromString(htmlInput, "text/html");
      
      // 检查解析错误
      const errors = doc.getElementsByTagName("parsererror");
      if (errors.length > 0) {
        throw new Error("HTML解析错误");
      }
      
      // 格式化输出HTML
      const serializer = new XMLSerializer();
      let formatted = "";
      
      // 格式化处理（简单实现）
      const formatNode = (node, level) => {
        const indent = "  ".repeat(level);
        
        if (node.nodeType === Node.TEXT_NODE) {
          const text = node.textContent.trim();
          if (text) {
            formatted += `${indent}${text}\n`;
          }
          return;
        }
        
        if (node.nodeType === Node.ELEMENT_NODE) {
          formatted += `${indent}<${node.tagName.toLowerCase()}`;
          
          // 添加属性
          for (let i = 0; i < node.attributes.length; i++) {
            const attr = node.attributes[i];
            formatted += ` ${attr.name}="${attr.value}"`;
          }
          
          if (node.childNodes.length === 0) {
            formatted += " />\n";
          } else {
            formatted += ">\n";
            
            // 处理子节点
            for (let i = 0; i < node.childNodes.length; i++) {
              formatNode(node.childNodes[i], level + 1);
            }
            
            formatted += `${indent}</${node.tagName.toLowerCase()}>\n`;
          }
        }
      };
      
      // 处理主体内容，从body节点开始
      if (doc.body) {
        for (let i = 0; i < doc.body.childNodes.length; i++) {
          formatNode(doc.body.childNodes[i], 0);
        }
      }
      
      htmlOutput = formatted || "格式化后的HTML将显示在这里";
    } catch (error) {
      htmlOutput = `错误: ${error.message}`;
    }
  }
  
  // 压缩HTML
  function minifyHtml() {
    try {
      // 移除空格和换行
      let minified = htmlInput
        .replace(/\s+/g, " ")              // 将多个空格替换为单个空格
        .replace(/>\s+</g, "><")           // 移除标签之间的空格
        .replace(/<!--.*?-->/gs, "")       // 移除注释
        .replace(/^\s+|\s+$/g, "");        // 移除首尾空格
        
      htmlOutput = minified || "压缩后的HTML将显示在这里";
    } catch (error) {
      htmlOutput = `错误: ${error.message}`;
    }
  }
  
  // 处理按钮点击
  function processHtml() {
    if (activeTab === "format") {
      formatHtml();
    } else if (activeTab === "minify") {
      minifyHtml();
    }
  }
  
  // 处理示例数据
  function loadExample() {
    htmlInput = `<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>HTML示例</title>
    <style>
        body {
            font-family: Arial, sans-serif;
            margin: 0;
            padding: 20px;
        }
        .container {
            max-width: 800px;
            margin: 0 auto;
        }
        .header {
            background-color: #f5f5f5;
            padding: 20px;
            border-radius: 5px;
            margin-bottom: 20px;
        }
        .content {
            line-height: 1.6;
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>HTML示例页面</h1>
            <p>这是一个简单的HTML示例，用于测试格式化和压缩功能。</p>
        </div>
        <div class="content">
            <h2>功能列表</h2>
            <ul>
                <li>HTML格式化</li>
                <li>HTML压缩</li>
                <li>HTML预览</li>
            </ul>
            <p>更多信息请访问 <a href="https://developer.mozilla.org/zh-CN/docs/Web/HTML">MDN HTML文档</a></p>
        </div>
    </div>
</body>
</html>`;
  }
  
  // 复制到剪贴板
  async function copyToClipboard() {
    try {
      await navigator.clipboard.writeText(htmlOutput);
      alert("已复制到剪贴板");
    } catch (err) {
      alert("复制失败: " + err);
    }
  }
  
  // 清空输入
  function clearInput() {
    htmlInput = "";
    htmlOutput = "";
  }
</script>

<div class="html-tool">
  <h1>HTML工具</h1>
  
  <div class="tab-container">
    <div class="tabs">
      <button 
        class:active={activeTab === "format"} 
        onclick={() => activeTab = "format"}
      >
        格式化HTML
      </button>
      <button 
        class:active={activeTab === "minify"} 
        onclick={() => activeTab = "minify"}
      >
        压缩HTML
      </button>
      <button 
        class:active={activeTab === "preview"} 
        onclick={() => activeTab = "preview"}
      >
        HTML预览
      </button>
    </div>
  </div>
  
  <div class="tool-container">
    <div class="input-area">
      <div class="textarea-header">
        <h3>HTML输入</h3>
        <div class="button-group">
          <button class="action-button" onclick={loadExample}>加载示例</button>
          <button class="action-button" onclick={clearInput}>清空</button>
        </div>
      </div>
      <textarea
        bind:value={htmlInput}
        placeholder="请输入HTML代码..."
        rows="15"
      ></textarea>
    </div>
    
    <div class="action-area">
      {#if activeTab !== "preview"}
        <button class="process-button" onclick={processHtml}>
          {activeTab === "format" ? "格式化" : "压缩"}
        </button>
      {/if}
    </div>
    
    <div class="output-area">
      <div class="textarea-header">
        <h3>
          {#if activeTab === "format"}
            格式化输出
          {:else if activeTab === "minify"}
            压缩输出
          {:else}
            HTML预览
          {/if}
        </h3>
        {#if activeTab !== "preview" && htmlOutput}
          <button class="action-button" onclick={copyToClipboard}>复制</button>
        {/if}
      </div>
      
      {#if activeTab === "preview"}
        <div class="preview-container">
          {@html htmlInput}
        </div>
      {:else}
        <textarea
          value={htmlOutput}
          placeholder={activeTab === "format" ? "格式化后的HTML将显示在这里" : "压缩后的HTML将显示在这里"}
          rows="15"
          readonly
        ></textarea>
      {/if}
    </div>
  </div>
</div>

<style>
  .html-tool {
    width: 100%;
    max-width: 100%;
    margin: 0 auto;
  }
  
  h1 {
    margin-top: 0;
    margin-bottom: 1.5rem;
    color: var(--text-color);
  }
  
  .tab-container {
    margin-bottom: 1.5rem;
  }
  
  .tabs {
    display: flex;
    border-bottom: 2px solid #eee;
  }
  
  .tabs button {
    padding: 0.75rem 1.5rem;
    background: transparent;
    border: none;
    border-bottom: 2px solid transparent;
    margin-bottom: -2px;
    cursor: pointer;
    font-weight: 600;
    transition: all 0.3s ease;
  }
  
  .tabs button:hover {
    background: rgba(99, 102, 241, 0.05);
  }
  
  .tabs button.active {
    color: var(--primary-color);
    border-bottom: 2px solid var(--primary-color);
  }
  
  .tool-container {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }
  
  .input-area, .output-area {
    width: 100%;
  }
  
  .textarea-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.5rem;
  }
  
  .textarea-header h3 {
    margin: 0;
    font-size: 1rem;
  }
  
  textarea {
    width: 100%;
    padding: 1rem;
    border: 1px solid #ddd;
    border-radius: var(--border-radius);
    font-family: monospace;
    resize: none;
    background: var(--card-bg);
    color: var(--text-color);
  }
  
  .action-area {
    display: flex;
    justify-content: center;
    margin: 1rem 0;
  }
  
  .process-button {
    padding: 0.75rem 1.5rem;
    background: var(--primary-color);
    color: white;
    border: none;
    border-radius: var(--border-radius);
    cursor: pointer;
    font-weight: 600;
    transition: all 0.3s ease;
  }
  
  .process-button:hover {
    background: var(--primary-light);
  }
  
  .action-button {
    padding: 0.5rem 1rem;
    background: transparent;
    border: 1px solid #ddd;
    border-radius: var(--border-radius);
    cursor: pointer;
    font-size: 0.875rem;
    transition: all 0.3s ease;
  }
  
  .action-button:hover {
    background: rgba(99, 102, 241, 0.1);
  }
  
  .button-group {
    display: flex;
    gap: 0.5rem;
  }
  
  .preview-container {
    padding: 1rem;
    border: 1px solid #ddd;
    border-radius: var(--border-radius);
    min-height: 400px;
    overflow: auto;
    background: white;
    color: black;
  }
  
  @media (prefers-color-scheme: dark) {
    .preview-container {
      background: white;
      color: black;
    }
  }
</style> 