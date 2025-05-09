<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { open, save } from "@tauri-apps/plugin-dialog";
  import { exists } from "@tauri-apps/plugin-fs";
  import { convertFileSrc } from '@tauri-apps/api/core';
  
  interface ImageConversionResult {
    success: boolean;
    message: string;
    file_path?: string;
    file_name?: string;
    base64_data?: string;
  }
  
  interface ImageInfo {
    width: number;
    height: number;
    color_type: string;
    file_size: number;
    file_path: string;
  }
  
  interface ImageConversionOptions {
    quality?: number;
    resize?: {
      width?: number;
      height?: number;
      keepAspectRatio: boolean;
    };
  }
  
  let selectedFile = $state<string | null>(null);
  let selectedFileName = $state<string | null>(null);
  let previewSrc = $state<string | null>(null);
  let outputFormat = $state<string>("PNG");
  let convertedResults = $state<Array<ImageConversionResult>>([]);
  let imageInfo = $state<ImageInfo | null>(null);
  let errorMessage = $state<string | null>(null);
  let successMessage = $state<string | null>(null);
  let converting = $state<boolean>(false);
  
  // 图片质量设置 (1-100)
  let quality = $state<number>(90);
  
  // 调整大小设置
  let resizeEnabled = $state<boolean>(false);
  let resizeWidth = $state<number | null>(null);
  let resizeHeight = $state<number | null>(null);
  let keepAspectRatio = $state<boolean>(true);
  
  // 选择图片文件
  async function selectImage() {
    try {
      // 打开文件选择对话框
      const selected = await open({
        multiple: false,
        filters: [{
          name: "图片",
          extensions: ["jpg", "jpeg", "png", "gif", "webp", "bmp", "ico"]
        }]
      });
      
      if (selected === null) {
        return; // 用户取消了选择
      }
      
      // 处理选择的文件
      selectedFile = selected as string;
      const pathParts = selectedFile.split(/[/\\]/);
      selectedFileName = pathParts[pathParts.length - 1];
      
      try {
        // 为原图生成base64预览
        const base64Data = await invoke<string>("get_image_base64", { 
          filePath: selectedFile 
        });
        previewSrc = base64Data;
      } catch (err) {
        console.error("生成图片预览失败:", err);
        // 回退到convertFileSrc
        previewSrc = convertFileSrc(selectedFile);
      }
      
      // 获取图片信息
      try {
        imageInfo = await invoke<ImageInfo>("get_image_info", { 
          filePath: selectedFile 
        });
        
        // 设置默认调整尺寸值为原图尺寸
        if (imageInfo) {
          resizeWidth = imageInfo.width;
          resizeHeight = imageInfo.height;
        }
      } catch (err) {
        console.error("获取图片信息失败:", err);
        errorMessage = `获取图片信息失败: ${err}`;
        setTimeout(() => {
          errorMessage = null;
        }, 3000);
      }
      
      // 清空转换结果和错误/成功信息
      convertedResults = [];
      errorMessage = null;
      successMessage = null;
      
    } catch (err) {
      console.error("选择图片失败:", err);
      errorMessage = `选择图片失败: ${err}`;
      setTimeout(() => {
        errorMessage = null;
      }, 3000);
    }
  }
  
  // 更新高度（保持纵横比）
  function updateHeight() {
    if (!imageInfo || !resizeWidth || !keepAspectRatio) return;
    const ratio = imageInfo.height / imageInfo.width;
    resizeHeight = Math.round(resizeWidth * ratio);
  }
  
  // 更新宽度（保持纵横比）
  function updateWidth() {
    if (!imageInfo || !resizeHeight || !keepAspectRatio) return;
    const ratio = imageInfo.width / imageInfo.height;
    resizeWidth = Math.round(resizeHeight * ratio);
  }
  
  // 转换图片
  async function convertImage() {
    if (!selectedFile) {
      errorMessage = "请先选择图片文件";
      setTimeout(() => {
        errorMessage = null;
      }, 3000);
      return;
    }
    
    try {
      converting = true;
      errorMessage = null;
      successMessage = null;
      
      // 准备转换选项
      const options: ImageConversionOptions = {
        quality: quality
      };
      
      // 添加调整大小选项
      if (resizeEnabled && (resizeWidth || resizeHeight)) {
        options.resize = {
          width: resizeWidth || undefined,
          height: resizeHeight || undefined,
          keepAspectRatio: keepAspectRatio
        };
      }
      
      // 调用Rust函数转换图片
      const result = await invoke<ImageConversionResult>("convert_image", {
        sourcePath: selectedFile,
        format: outputFormat,
        options: options
      });
      
      if (result.success) {
        // 如果有base64数据，直接使用它来显示预览
        if (result.base64_data) {
          convertedResults = [{
            ...result,
            // 使用base64数据来预览，这样不需要依赖文件路径
          }];
        } else if (result.file_path) {
          // 否则，使用convertFileSrc处理文件路径
          convertedResults = [{
            ...result
          }];
        }
        
        successMessage = "图片转换成功";
        setTimeout(() => {
          successMessage = null;
        }, 3000);
      } else {
        errorMessage = `转换失败: ${result.message}`;
        setTimeout(() => {
          errorMessage = null;
        }, 3000);
      }
    } catch (err) {
      console.error("转换图片失败:", err);
      errorMessage = `转换图片失败: ${err}`;
      setTimeout(() => {
        errorMessage = null;
      }, 3000);
    } finally {
      converting = false;
    }
  }
  
  // 保存转换后的图片
  async function saveImage(path: string, fileName: string) {
    try {
      // 打开保存对话框
      const savePath = await save({
        defaultPath: fileName
      });
      
      if (savePath) {
        try {
          // 使用Rust后端命令复制文件，而不是前端读写文件
          const result = await invoke<boolean>("copy_file", {
            sourcePath: path,
            destPath: savePath
          });
          
          if (result) {
            successMessage = `已保存到 ${savePath}`;
          } else {
            errorMessage = "保存文件失败";
          }
          
          setTimeout(() => {
            successMessage = null;
            errorMessage = null;
          }, 3000);
        } catch (err) {
          console.error("保存文件失败:", err);
          errorMessage = `保存文件失败: ${err}`;
          setTimeout(() => {
            errorMessage = null;
          }, 3000);
        }
      }
    } catch (err) {
      console.error("选择保存路径失败:", err);
      errorMessage = `选择保存路径失败: ${err}`;
      setTimeout(() => {
        errorMessage = null;
      }, 3000);
    }
  }
</script>

<div class="container p-4">
  <h2 class="text-2xl font-bold mb-4">图片转换工具</h2>
  
  <div class="mb-4">
    <button class="bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded mr-2" onclick={selectImage}>
      选择图片
    </button>
    
    {#if selectedFile && selectedFileName}
      <span class="ml-2">已选择: {selectedFileName}</span>
    {/if}
  </div>
  
  {#if selectedFile}
    <div class="mb-4 flex flex-wrap">
      <div class="w-full md:w-1/2 pr-2 mb-4">
        <div class="border rounded p-2">
          <h3 class="font-bold mb-2">原图预览</h3>
          {#if previewSrc}
            <img src={previewSrc} alt="原图预览" class="max-w-full h-auto mb-2 border" style="max-height: 300px;"/>
          {/if}
          
          {#if imageInfo}
            <div class="text-sm">
              <p>宽度: {imageInfo.width}px</p>
              <p>高度: {imageInfo.height}px</p>
              <p>颜色类型: {imageInfo.color_type}</p>
              <p>文件大小: {(imageInfo.file_size / 1024).toFixed(2)} KB</p>
            </div>
          {/if}
        </div>
      </div>
      
      <div class="w-full md:w-1/2 pl-2 mb-4">
        <div class="border rounded p-2">
          <h3 class="font-bold mb-2">转换选项</h3>
          
          <div class="mb-4">
            <label class="block text-gray-700 text-sm font-bold mb-2" for="outputFormat">
              输出格式:
            </label>
            <select 
              id="outputFormat" 
              class="shadow appearance-none border rounded w-full py-2 px-3 text-gray-700 leading-tight focus:outline-none focus:shadow-outline"
              bind:value={outputFormat}
            >
              <option value="PNG">PNG</option>
              <option value="JPEG">JPEG</option>
              <option value="GIF">GIF</option>
              <option value="WEBP">WEBP</option>
              <option value="BMP">BMP</option>
            </select>
          </div>
          
          {#if outputFormat === "JPEG" || outputFormat === "WEBP"}
            <div class="mb-4">
              <label class="block text-gray-700 text-sm font-bold mb-2">
                图片质量: {quality}%
              </label>
              <input 
                type="range" 
                min="1" 
                max="100" 
                class="w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer" 
                bind:value={quality}
              />
            </div>
          {/if}
          
          <div class="mb-4">
            <div class="flex items-center mb-2">
              <input 
                type="checkbox" 
                id="resizeEnabled" 
                class="mr-2"
                bind:checked={resizeEnabled}
              />
              <label class="text-gray-700 text-sm font-bold" for="resizeEnabled">
                调整图片尺寸
              </label>
            </div>
            
            {#if resizeEnabled}
              <div class="pl-4 border-l-2 border-gray-200">
                <div class="mb-2">
                  <label class="block text-gray-700 text-sm mb-1">
                    宽度 (像素):
                  </label>
                  <input 
                    type="number" 
                    class="shadow appearance-none border rounded w-full py-2 px-3 text-gray-700 leading-tight focus:outline-none focus:shadow-outline" 
                    bind:value={resizeWidth}
                    oninput={updateHeight}
                    min="1"
                  />
                </div>
                
                <div class="mb-2">
                  <label class="block text-gray-700 text-sm mb-1">
                    高度 (像素):
                  </label>
                  <input 
                    type="number" 
                    class="shadow appearance-none border rounded w-full py-2 px-3 text-gray-700 leading-tight focus:outline-none focus:shadow-outline" 
                    bind:value={resizeHeight}
                    oninput={updateWidth}
                    min="1"
                  />
                </div>
                
                <div class="flex items-center">
                  <input 
                    type="checkbox" 
                    id="keepAspectRatio" 
                    class="mr-2"
                    bind:checked={keepAspectRatio}
                  />
                  <label class="text-gray-700 text-xs" for="keepAspectRatio">
                    保持宽高比
                  </label>
                </div>
              </div>
            {/if}
          </div>
          
          <button 
            class="bg-green-500 hover:bg-green-700 text-white font-bold py-2 px-4 rounded w-full"
            onclick={convertImage}
            disabled={converting}
          >
            {converting ? '转换中...' : '转换图片'}
          </button>
        </div>
      </div>
    </div>
  {/if}
  
  {#if errorMessage}
    <div class="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded mb-4">
      {errorMessage}
    </div>
  {/if}
  
  {#if successMessage}
    <div class="bg-green-100 border border-green-400 text-green-700 px-4 py-3 rounded mb-4">
      {successMessage}
    </div>
  {/if}
  
  {#if convertedResults.length > 0}
    <div class="mb-4 border rounded p-4">
      <h3 class="font-bold mb-2">转换结果</h3>
      
      {#each convertedResults as result}
        {#if result.success && (result.file_path || result.base64_data)}
          <div class="mb-2">
            {#if result.base64_data}
              <img src={result.base64_data} alt="转换后图片" class="max-w-full h-auto mb-2 border" style="max-height: 300px;"/>
            {:else if result.file_path}
              <img src={convertFileSrc(result.file_path)} alt="转换后图片" class="max-w-full h-auto mb-2 border" style="max-height: 300px;"/>
            {/if}
            <p>文件名: {result.file_name}</p>
            <button 
              class="bg-blue-500 hover:bg-blue-700 text-white font-bold py-1 px-3 rounded text-sm"
              onclick={() => saveImage(result.file_path!, result.file_name!)}
            >
              保存
            </button>
          </div>
        {/if}
      {/each}
    </div>
  {/if}
</div> 