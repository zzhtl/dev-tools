<script lang="ts">
    import { save } from '@tauri-apps/plugin-dialog';
    import { writeFile } from '@tauri-apps/plugin-fs';
    import QRCode from "qrcode";
    import { onMount } from "svelte";
  
    let text = $state("https://example.com");
    let qrCodeUrl = $state("");
  
    const generateQRCode = async () => {
      try {
        qrCodeUrl = await QRCode.toDataURL(text, {
          width: 200,
          margin: 2,
        });
      } catch (err) {
        console.error("生成二维码失败:", err);
      }
    };
  
    /** 纯浏览器实现下载二维码 */
    const saveQRCode = async () => {
      if (!qrCodeUrl) return;
      // 弹出保存对话框，选择保存位置
      const filePath = await save({
        defaultPath: 'qrcode.png',
        filters: [{ name: 'Image', extensions: ['png'] }]
      });
      if (!filePath) return;
      // 解析 Base64 数据
      const base64Data = qrCodeUrl.split(',')[1];
      const binary = Uint8Array.from(atob(base64Data), c => c.charCodeAt(0));
      // 使用 Tauri 插件写入二进制文件
      await writeFile(filePath, binary);
    };
  
    onMount(() => {
      generateQRCode();
    });
  </script>
  
  <div class="qr-code-tool">
    <h2>二维码生成工具</h2>
    <div class="input-group">
      <label for="qr-text">输入文本或URL：</label>
      <input
        id="qr-text"
        type="text"
        bind:value={text}
        placeholder="输入文本或URL"
      />
      <button onclick={generateQRCode}>生成二维码</button>
    </div>
    {#if qrCodeUrl}
      <div class="qr-code-container">
        <img src={qrCodeUrl} alt="生成的二维码" />
        <div class="button-container">
          <button class="download-btn" onclick={saveQRCode}>保存二维码</button>
        </div>
      </div>
    {/if}
  </div>
  
  <style>
    .qr-code-tool {
      display: flex;
      flex-direction: column;
      gap: 1rem;
      padding: 1rem;
    }
  
    .input-group {
      display: flex;
      flex-direction: column;
      gap: 0.5rem;
    }
  
    input {
      padding: 0.5rem;
      border: 1px solid #ccc;
      border-radius: 4px;
      font-size: 1rem;
    }
  
    button {
      padding: 0.5rem 1rem;
      background: var(--primary-color);
      color: white;
      border: none;
      border-radius: 4px;
      cursor: pointer;
      font-size: 1rem;
      transition: background 0.3s ease;
    }
  
    button:hover {
      background: var(--primary-light);
    }
  
    .qr-code-container {
      margin-top: 1rem;
      text-align: center;
      display: flex;
      flex-direction: column;
      align-items: center;
      gap: 0.5rem;
    }
  
    .button-container {
      width: 100%;
      display: flex;
      justify-content: center;
    }
  
    img {
      max-width: 100%;
      height: auto;
      border: 1px solid #eee;
      border-radius: 4px;
    }
  
    .download-btn {
      padding: 0.5rem 1rem;
      background: var(--primary-color);
      color: white;
      border: none;
      border-radius: 4px;
      cursor: pointer;
      transition: background 0.3s ease;
      font-size: 1rem;
      width: auto;
    }
  
    .download-btn:hover {
      background: var(--primary-light);
    }
  </style>