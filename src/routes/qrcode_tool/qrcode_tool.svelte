<script lang="ts">
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
        console.error("Failed to generate QR code:", err);
      }
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
    }
  
    img {
      max-width: 100%;
      height: auto;
      border: 1px solid #eee;
      border-radius: 4px;
    }
  </style>