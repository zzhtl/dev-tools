<script lang="ts">
    let base64Result = $state("");
    let fileName = $state("");

    const handleFileChange = (event: Event) => {
      const fileInput = event.target as HTMLInputElement;
      if (fileInput.files?.[0]) processFile(fileInput.files[0]);
    };
  
    const processFile = (file: File) => {
      fileName = file.name;
      const reader = new FileReader();
      reader.onload = (e) => (base64Result = e.target?.result as string);
      reader.readAsDataURL(file);
    };
  
    const copyToClipboard = () => {
      if (base64Result) {
        navigator.clipboard.writeText(base64Result);
        alert("Base64 已复制到剪贴板！");
      }
    };
  </script>
  
  <div class="base64-tool">
    <h2>图片转 Base64</h2>
    <div class="upload-area">
      {#if fileName}
        <p>已选择文件: {fileName}</p>
      {:else}
        <p>点击上传图片</p>
      {/if}
      <input type="file" accept="image/*" onchange={handleFileChange} hidden />
      <button onclick={() => (document.querySelector('input[type="file"]') as HTMLInputElement)?.click()}>
        选择图片
      </button>
    </div>
    <div class="result">
      <textarea 
        readonly 
        rows="5" 
        bind:value={base64Result}
        style="height: 150px; resize: none; overflow-y: auto;"
        placeholder="Base64 结果将显示在这里"
      ></textarea>
      <button onclick={copyToClipboard}>复制 Base64</button>
    </div>
  </div>

<style>
  .base64-tool {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    padding: 1rem;
  }

  .upload-area {
    border: 2px dashed #ccc;
    border-radius: 8px;
    padding: 2rem;
    text-align: center;
    cursor: pointer;
  }

  textarea {
    width: 100%;
    padding: 0.5rem;
    border: 1px solid #ccc;
    border-radius: 4px;
    font-family: monospace;
  }

  button {
    padding: 0.5rem 1rem;
    background: var(--primary-color);
    color: white;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 1rem;
    margin-top: 0.5rem;
  }
</style>
