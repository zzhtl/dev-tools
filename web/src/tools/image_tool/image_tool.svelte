<script lang="ts">
    import { apiMultipart } from "../../lib/api";
    import { downloadBlob } from "../../lib/download";

    interface ImageConversionResult {
        success: boolean;
        message: string;
        file_name?: string;
        base64_data?: string;
        file_size?: number;
        blob?: Blob;
    }

    interface ImageInfo {
        width: number;
        height: number;
        color_type: string;
        file_size: number;
    }

    interface ImageConversionOptions {
        quality?: number;
        resize?: {
            width?: number;
            height?: number;
            keep_aspect_ratio: boolean;
        };
    }

    // 状态
    let selectedFile = $state<File | null>(null);
    let selectedFileName = $state<string | null>(null);
    let previewSrc = $state<string | null>(null);
    let outputFormat = $state<string>("PNG");
    let convertedResults = $state<Array<ImageConversionResult>>([]);
    let imageInfo = $state<ImageInfo | null>(null);
    let errorMessage = $state<string | null>(null);
    let successMessage = $state<string | null>(null);
    let converting = $state<boolean>(false);
    let loading = $state<boolean>(false);
    let isDragging = $state<boolean>(false);
    let fileInputEl: HTMLInputElement | null = $state(null);

    // 图片质量设置
    let quality = $state<number>(90);

    // 调整大小设置
    let resizeEnabled = $state<boolean>(false);
    let resizeWidth = $state<number | null>(null);
    let resizeHeight = $state<number | null>(null);
    let keepAspectRatio = $state<boolean>(true);

    // ICO格式尺寸设置
    let icoSize = $state<number>(32);
    const iconSizes = [16, 32, 48, 64, 128, 256];

    // 当前激活的设置选项卡
    let activeSettingTab = $state<'format' | 'resize' | 'output'>('format');

    // 支持的格式
    const outputFormats = [
        { value: "PNG", label: "PNG", desc: "无损压缩，支持透明", icon: "🖼️", color: "#10b981" },
        { value: "JPEG", label: "JPEG", desc: "有损压缩，体积小", icon: "📷", color: "#f59e0b" },
        { value: "WEBP", label: "WebP", desc: "现代格式，体积最小", icon: "🌐", color: "#6366f1" },
        { value: "GIF", label: "GIF", desc: "支持动画", icon: "🎬", color: "#ec4899" },
        { value: "BMP", label: "BMP", desc: "无压缩位图", icon: "🗺️", color: "#8b5cf6" },
        { value: "ICO", label: "ICO", desc: "图标格式", icon: "💠", color: "#14b8a6" }
    ];

    // 预设尺寸分组
    const presetSizeGroups = [
        {
            name: "社交媒体",
            sizes: [
                { label: "微信头像", width: 132, height: 132, icon: "💬" },
                { label: "微博头像", width: 180, height: 180, icon: "📱" },
                { label: "Twitter 头像", width: 400, height: 400, icon: "🐦" },
                { label: "Instagram 帖子", width: 1080, height: 1080, icon: "📸" },
            ]
        },
        {
            name: "常用尺寸",
            sizes: [
                { label: "缩略图", width: 150, height: 150, icon: "🔍" },
                { label: "小图", width: 320, height: 240, icon: "📄" },
                { label: "中图", width: 640, height: 480, icon: "📋" },
                { label: "大图", width: 1024, height: 768, icon: "🖥️" },
            ]
        },
        {
            name: "视频分辨率",
            sizes: [
                { label: "HD 720p", width: 1280, height: 720, icon: "📺" },
                { label: "Full HD 1080p", width: 1920, height: 1080, icon: "🎬" },
                { label: "2K QHD", width: 2560, height: 1440, icon: "🖥️" },
                { label: "4K UHD", width: 3840, height: 2160, icon: "📽️" },
            ]
        }
    ];

    // 文件大小格式化
    function formatFileSize(bytes: number): string {
        if (bytes < 1024) return bytes + " B";
        if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(2) + " KB";
        return (bytes / (1024 * 1024)).toFixed(2) + " MB";
    }

    // 获取压缩率
    function getCompressionRatio(): string {
        if (!imageInfo || convertedResults.length === 0 || !convertedResults[0].file_size) return "";
        const original = imageInfo.file_size;
        const converted = convertedResults[0].file_size;
        const ratio = ((1 - converted / original) * 100).toFixed(1);
        return ratio;
    }

    // 选择图片
    function selectImage() {
        errorMessage = null;
        successMessage = null;
        fileInputEl?.click();
    }

    // 文件选择变化
    async function onFilePicked(e: Event) {
        const target = e.target as HTMLInputElement;
        const file = target.files?.[0];
        if (!file) return;
        await loadImageFile(file);
        target.value = "";
    }

    // 加载图片文件
    async function loadImageFile(file: File) {
        loading = true;
        errorMessage = null;
        convertedResults = [];

        if (previewSrc && previewSrc.startsWith("blob:")) {
            URL.revokeObjectURL(previewSrc);
        }

        selectedFile = file;
        selectedFileName = file.name;

        if (file.size > 5 * 1024 * 1024) {
            successMessage = `正在处理大图片 (${formatFileSize(file.size)})，请稍候...`;
        }

        const objectUrl = URL.createObjectURL(file);
        previewSrc = objectUrl;

        try {
            const dims = await new Promise<{ width: number; height: number }>((resolve, reject) => {
                const img = new Image();
                img.onload = () => resolve({ width: img.naturalWidth, height: img.naturalHeight });
                img.onerror = () => reject(new Error("图片解码失败"));
                img.src = objectUrl;
            });
            imageInfo = {
                width: dims.width,
                height: dims.height,
                color_type: file.type || "unknown",
                file_size: file.size,
            };
            resizeWidth = dims.width;
            resizeHeight = dims.height;
            successMessage = null;
        } catch (err) {
            console.error("获取图片信息失败:", err);
            errorMessage = `获取图片信息失败: ${err}`;
        } finally {
            loading = false;
        }
    }

    // 更新高度
    function updateHeight() {
        if (!imageInfo || !resizeWidth || !keepAspectRatio) return;
        const ratio = imageInfo.height / imageInfo.width;
        resizeHeight = Math.round(resizeWidth * ratio);
    }

    // 更新宽度
    function updateWidth() {
        if (!imageInfo || !resizeHeight || !keepAspectRatio) return;
        const ratio = imageInfo.width / imageInfo.height;
        resizeWidth = Math.round(resizeHeight * ratio);
    }

    // 应用预设尺寸
    function applyPresetSize(width: number, height: number) {
        resizeEnabled = true;
        resizeWidth = width;
        resizeHeight = height;
        activeSettingTab = 'resize';
    }

    // 重置为原始尺寸
    function resetToOriginalSize() {
        if (imageInfo) {
            resizeWidth = imageInfo.width;
            resizeHeight = imageInfo.height;
        }
        resizeEnabled = false;
    }

    // 转换图片
    async function convertImage() {
        if (!selectedFile) {
            errorMessage = "请先选择图片文件";
            return;
        }

        try {
            converting = true;
            errorMessage = null;
            successMessage = "正在处理图片，请稍候...";

            const options: ImageConversionOptions = { quality };

            if (resizeEnabled && (resizeWidth || resizeHeight)) {
                options.resize = {
                    width: resizeWidth || undefined,
                    height: resizeHeight || undefined,
                    keep_aspect_ratio: keepAspectRatio
                };
            } else if (outputFormat === "ICO") {
                options.resize = {
                    width: icoSize,
                    height: icoSize,
                    keep_aspect_ratio: false
                };
            }

            try {
                const fd = new FormData();
                fd.append("file", selectedFile, selectedFileName ?? "upload");
                fd.append("format", outputFormat);
                fd.append("options", JSON.stringify(options));

                const res = await apiMultipart("/api/image/convert", fd);
                const blob = await res.blob();
                const fileName = res.headers.get("X-File-Name")
                    ?? `converted.${outputFormat.toLowerCase()}`;
                const fileSize = Number(res.headers.get("X-File-Size")) || blob.size;
                const previewUrl = URL.createObjectURL(blob);

                const result: ImageConversionResult = {
                    success: true,
                    message: "转换成功",
                    file_name: fileName,
                    file_size: fileSize,
                    base64_data: previewUrl,
                    blob,
                };

                // 释放旧的 blob URL
                for (const old of convertedResults) {
                    if (old.base64_data?.startsWith("blob:")) {
                        URL.revokeObjectURL(old.base64_data);
                    }
                }
                convertedResults = [result];

                let message = "✓ 图片转换成功";
                if (outputFormat === "ICO") {
                    message = "✓ 图片已转换为ICO格式";
                }
                successMessage = message;
            } catch (err) {
                console.error("转换图片失败:", err);
                errorMessage = `转换图片失败: ${err}`;
                successMessage = null;
            } finally {
                converting = false;
            }
        } catch (err) {
            console.error("转换图片失败:", err);
            errorMessage = `转换图片失败: ${err}`;
            converting = false;
        }
    }

    // 保存转换后的图片
    function saveImage(result: ImageConversionResult) {
        if (!result.blob || !result.file_name) {
            errorMessage = "没有可保存的文件";
            return;
        }
        try {
            downloadBlob(result.blob, result.file_name);
            successMessage = `✓ 已触发下载: ${result.file_name}`;
        } catch (err) {
            console.error("保存文件失败:", err);
            errorMessage = `保存文件失败: ${err}`;
        }
    }

    // 清除选择
    function clearSelection() {
        if (previewSrc && previewSrc.startsWith("blob:")) {
            URL.revokeObjectURL(previewSrc);
        }
        for (const r of convertedResults) {
            if (r.base64_data?.startsWith("blob:")) {
                URL.revokeObjectURL(r.base64_data);
            }
        }
        selectedFile = null;
        selectedFileName = null;
        previewSrc = null;
        imageInfo = null;
        convertedResults = [];
        errorMessage = null;
        successMessage = null;
    }

    // 拖拽事件
    function onDragOver(e: DragEvent) {
        e.preventDefault();
        isDragging = true;
    }

    function onDragLeave(e: DragEvent) {
        e.preventDefault();
        isDragging = false;
    }

    async function onDrop(e: DragEvent) {
        e.preventDefault();
        isDragging = false;
        const file = e.dataTransfer?.files?.[0];
        if (file && file.type.startsWith("image/")) {
            await loadImageFile(file);
        } else if (file) {
            errorMessage = "请拖入图片文件";
        }
    }

    // 计算纵横比
    function getAspectRatio(): string {
        if (!imageInfo) return "";
        const gcd = (a: number, b: number): number => b === 0 ? a : gcd(b, a % b);
        const divisor = gcd(imageInfo.width, imageInfo.height);
        return `${imageInfo.width / divisor}:${imageInfo.height / divisor}`;
    }

    // 获取图片方向
    function getOrientation(): string {
        if (!imageInfo) return "";
        if (imageInfo.width > imageInfo.height) return "横向";
        if (imageInfo.width < imageInfo.height) return "纵向";
        return "正方形";
    }
</script>

<div class="image-tool">
    <!-- 标题区域 -->
    <div class="tool-header">
        <div class="header-content">
            <h2>🖼️ 图片转换工具</h2>
            <p class="header-desc">支持多种图片格式转换、调整尺寸、压缩优化</p>
        </div>
        {#if selectedFile}
            <button class="header-btn" onclick={clearSelection}>
                <span>✕</span> 重新选择
            </button>
        {/if}
    </div>

    <input
        type="file"
        accept="image/*"
        hidden
        bind:this={fileInputEl}
        onchange={onFilePicked}
    />

    <!-- 上传区域 -->
    {#if !selectedFile}
        <div
            class="upload-area"
            class:dragging={isDragging}
            onclick={selectImage}
            ondragover={onDragOver}
            ondragleave={onDragLeave}
            ondrop={onDrop}
            role="button"
            tabindex="0"
            onkeydown={(e) => e.key === 'Enter' && selectImage()}
        >
            <div class="upload-visual">
                <div class="upload-icon">
                    <svg width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                        <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
                        <polyline points="17 8 12 3 7 8"/>
                        <line x1="12" y1="3" x2="12" y2="15"/>
                    </svg>
                </div>
                <h3>点击选择图片</h3>
                <p>或将图片拖放到此处</p>
            </div>
            <div class="supported-formats">
                <span class="format-tag">JPG</span>
                <span class="format-tag">PNG</span>
                <span class="format-tag">GIF</span>
                <span class="format-tag">WebP</span>
                <span class="format-tag">BMP</span>
                <span class="format-tag">ICO</span>
            </div>
        </div>

        <!-- 快速尺寸选择 -->
        <div class="quick-presets">
            <h3>📐 快速尺寸预设</h3>
            <div class="preset-groups">
                {#each presetSizeGroups as group}
                    <div class="preset-group">
                        <div class="group-header">{group.name}</div>
                        <div class="group-sizes">
                            {#each group.sizes as size}
                                <button 
                                    class="preset-chip"
                                    onclick={() => {
                                        applyPresetSize(size.width, size.height);
                                        selectImage();
                                    }}
                                    title="{size.width} × {size.height}"
                                >
                                    <span class="chip-icon">{size.icon}</span>
                                    <span class="chip-text">{size.label}</span>
                                </button>
                            {/each}
                        </div>
                    </div>
                {/each}
            </div>
        </div>
    {:else}
        <!-- 主编辑区域 -->
        <div class="editor-layout">
            <!-- 左侧：预览区域 -->
            <div class="preview-panel">
                <div class="panel-header">
                    <span class="panel-title">📷 原图</span>
                    {#if imageInfo}
                        <span class="panel-badge">{imageInfo.width} × {imageInfo.height}</span>
                    {/if}
                </div>
                
                <div class="preview-box">
                    {#if loading}
                        <div class="loading-state">
                            <div class="spinner"></div>
                            <span>加载中...</span>
                        </div>
                    {:else if previewSrc}
                        <img src={previewSrc} alt="原图预览" />
                    {:else}
                        <div class="empty-state">暂无预览</div>
                    {/if}
                </div>

                {#if imageInfo}
                    <div class="image-stats">
                        <div class="stat-row">
                            <div class="stat-item">
                                <span class="stat-icon">📐</span>
                                <div class="stat-content">
                                    <span class="stat-label">尺寸</span>
                                    <span class="stat-value">{imageInfo.width} × {imageInfo.height}</span>
                                </div>
                            </div>
                            <div class="stat-item">
                                <span class="stat-icon">📦</span>
                                <div class="stat-content">
                                    <span class="stat-label">大小</span>
                                    <span class="stat-value">{formatFileSize(imageInfo.file_size)}</span>
                                </div>
                            </div>
                        </div>
                        <div class="stat-row">
                            <div class="stat-item">
                                <span class="stat-icon">📊</span>
                                <div class="stat-content">
                                    <span class="stat-label">比例</span>
                                    <span class="stat-value">{getAspectRatio()}</span>
                                </div>
                            </div>
                            <div class="stat-item">
                                <span class="stat-icon">🔄</span>
                                <div class="stat-content">
                                    <span class="stat-label">方向</span>
                                    <span class="stat-value">{getOrientation()}</span>
                                </div>
                            </div>
                        </div>
                        <div class="stat-row">
                            <div class="stat-item full-width">
                                <span class="stat-icon">🎨</span>
                                <div class="stat-content">
                                    <span class="stat-label">颜色模式</span>
                                    <span class="stat-value">{imageInfo.color_type}</span>
                                </div>
                            </div>
                        </div>
                    </div>
                {/if}
            </div>

            <!-- 右侧：设置区域 -->
            <div class="settings-panel">
                <div class="settings-tabs">
                    <button 
                        class="tab-btn"
                        class:active={activeSettingTab === 'format'}
                        onclick={() => activeSettingTab = 'format'}
                    >
                        🎨 格式
                    </button>
                    <button 
                        class="tab-btn"
                        class:active={activeSettingTab === 'resize'}
                        onclick={() => activeSettingTab = 'resize'}
                    >
                        📐 尺寸
                    </button>
                    <button 
                        class="tab-btn"
                        class:active={activeSettingTab === 'output'}
                        onclick={() => activeSettingTab = 'output'}
                    >
                        ⚙️ 输出
                    </button>
                </div>

                <div class="settings-content">
                    <!-- 格式选项卡 -->
                    {#if activeSettingTab === 'format'}
                        <div class="tab-panel">
                            <div class="format-grid">
                                {#each outputFormats as format}
                                    <button 
                                        class="format-card"
                                        class:active={outputFormat === format.value}
                                        onclick={() => outputFormat = format.value}
                                        style="--format-color: {format.color}"
                                    >
                                        <span class="format-icon">{format.icon}</span>
                                        <div class="format-info">
                                            <span class="format-name">{format.label}</span>
                                            <span class="format-desc">{format.desc}</span>
                                        </div>
                                        {#if outputFormat === format.value}
                                            <span class="format-check">✓</span>
                                        {/if}
                                    </button>
                                {/each}
                            </div>

                            <!-- JPEG/WebP 质量设置 -->
                            {#if outputFormat === "JPEG" || outputFormat === "WEBP"}
                                <div class="quality-section">
                                    <div class="quality-header">
                                        <span>图片质量</span>
                                        <span class="quality-value">{quality}%</span>
                                    </div>
                                    <div class="quality-slider-container">
                                        <input 
                                            type="range" 
                                            min="1" max="100" 
                                            class="quality-slider"
                                            bind:value={quality}
                                        />
                                        <div class="quality-marks">
                                            <span>低</span>
                                            <span>中</span>
                                            <span>高</span>
                                        </div>
                                    </div>
                                    <div class="quality-presets">
                                        <button class:active={quality === 60} onclick={() => quality = 60}>压缩 60%</button>
                                        <button class:active={quality === 80} onclick={() => quality = 80}>标准 80%</button>
                                        <button class:active={quality === 90} onclick={() => quality = 90}>高质 90%</button>
                                        <button class:active={quality === 100} onclick={() => quality = 100}>无损 100%</button>
                                    </div>
                                </div>
                            {/if}

                            <!-- ICO 尺寸设置 -->
                            {#if outputFormat === "ICO"}
                                <div class="ico-section">
                                    <div class="section-label">图标尺寸</div>
                                    <div class="ico-sizes">
                                        {#each iconSizes as size}
                                            <button 
                                                class="ico-size-btn"
                                                class:active={icoSize === size}
                                                onclick={() => icoSize = size}
                                            >
                                                {size}×{size}
                                            </button>
                                        {/each}
                                    </div>
                                    <p class="hint-text">💡 ICO格式用于网站图标和桌面图标</p>
                                </div>
                            {/if}
                        </div>
                    {/if}

                    <!-- 尺寸选项卡 -->
                    {#if activeSettingTab === 'resize'}
                        <div class="tab-panel">
                            <div class="resize-toggle">
                                <label class="toggle-switch">
                                    <input type="checkbox" bind:checked={resizeEnabled} />
                                    <span class="toggle-slider"></span>
                                </label>
                                <span>启用尺寸调整</span>
                            </div>

                            {#if resizeEnabled}
                                <div class="resize-options">
                                    <!-- 自定义尺寸 -->
                                    <div class="custom-size-section">
                                        <div class="size-inputs">
                                            <div class="size-input-group">
                                                <label>宽度</label>
                                                <div class="input-wrapper">
                                                    <input 
                                                        type="number" 
                                                        bind:value={resizeWidth}
                                                        oninput={updateHeight}
                                                        min="1"
                                                    />
                                                    <span class="input-suffix">px</span>
                                                </div>
                                            </div>
                                            <div class="size-link" class:active={keepAspectRatio}>
                                                <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                                    {#if keepAspectRatio}
                                                        <path d="M9 11a3 3 0 1 0 6 0a3 3 0 0 0 -6 0"></path>
                                                        <path d="M12 3v5"></path>
                                                        <path d="M12 14v7"></path>
                                                    {:else}
                                                        <path d="M10 10l4 4m0 -4l-4 4"></path>
                                                    {/if}
                                                </svg>
                                            </div>
                                            <div class="size-input-group">
                                                <label>高度</label>
                                                <div class="input-wrapper">
                                                    <input 
                                                        type="number" 
                                                        bind:value={resizeHeight}
                                                        oninput={updateWidth}
                                                        min="1"
                                                    />
                                                    <span class="input-suffix">px</span>
                                                </div>
                                            </div>
                                        </div>

                                        <label class="aspect-ratio-toggle">
                                            <input type="checkbox" bind:checked={keepAspectRatio} />
                                            <span>🔗 保持宽高比</span>
                                        </label>

                                        <button class="reset-size-btn" onclick={resetToOriginalSize}>
                                            ↩️ 重置为原始尺寸
                                        </button>
                                    </div>

                                    <!-- 预设尺寸 -->
                                    <div class="preset-size-section">
                                        <div class="section-label">快速预设</div>
                                        {#each presetSizeGroups as group}
                                            <div class="preset-mini-group">
                                                <span class="mini-group-label">{group.name}</span>
                                                <div class="mini-preset-list">
                                                    {#each group.sizes as size}
                                                        <button 
                                                            class="mini-preset-btn"
                                                            onclick={() => applyPresetSize(size.width, size.height)}
                                                        >
                                                            {size.icon} {size.width}×{size.height}
                                                        </button>
                                                    {/each}
                                                </div>
                                            </div>
                                        {/each}
                                    </div>
                                </div>
                            {:else}
                                <div class="resize-disabled-hint">
                                    <p>启用后可调整输出图片的尺寸</p>
                                    <p>原图尺寸: {imageInfo?.width || 0} × {imageInfo?.height || 0} px</p>
                                </div>
                            {/if}
                        </div>
                    {/if}

                    <!-- 输出选项卡 -->
                    {#if activeSettingTab === 'output'}
                        <div class="tab-panel">
                            <div class="output-summary">
                                <h4>转换配置摘要</h4>
                                <div class="summary-grid">
                                    <div class="summary-item">
                                        <span class="summary-label">输出格式</span>
                                        <span class="summary-value">{outputFormat}</span>
                                    </div>
                                    {#if outputFormat === "JPEG" || outputFormat === "WEBP"}
                                        <div class="summary-item">
                                            <span class="summary-label">图片质量</span>
                                            <span class="summary-value">{quality}%</span>
                                        </div>
                                    {/if}
                                    {#if outputFormat === "ICO"}
                                        <div class="summary-item">
                                            <span class="summary-label">图标尺寸</span>
                                            <span class="summary-value">{icoSize}×{icoSize}</span>
                                        </div>
                                    {/if}
                                    <div class="summary-item">
                                        <span class="summary-label">调整尺寸</span>
                                        <span class="summary-value">{resizeEnabled ? `${resizeWidth}×${resizeHeight}` : '不调整'}</span>
                                    </div>
                                    {#if resizeEnabled}
                                        <div class="summary-item">
                                            <span class="summary-label">保持比例</span>
                                            <span class="summary-value">{keepAspectRatio ? '是' : '否'}</span>
                                        </div>
                                    {/if}
                                </div>
                            </div>
                        </div>
                    {/if}
                </div>

                <!-- 转换按钮 -->
                <button 
                    class="convert-btn"
                    onclick={convertImage}
                    disabled={converting || loading}
                >
                    {#if converting}
                        <span class="btn-spinner"></span>
                        转换中...
                    {:else}
                        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                            <polyline points="23 4 23 10 17 10"></polyline>
                            <path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10"></path>
                        </svg>
                        开始转换
                    {/if}
                </button>
            </div>
        </div>

        <!-- 消息提示 -->
        {#if errorMessage}
            <div class="message error">
                <span class="message-icon">⚠️</span>
                <span>{errorMessage}</span>
            </div>
        {/if}

        {#if successMessage && !errorMessage}
            <div class="message success">
                <span class="message-icon">✓</span>
                <span>{successMessage}</span>
            </div>
        {/if}

        <!-- 转换结果 -->
        {#if convertedResults.length > 0}
            <div class="results-section">
                <div class="results-header">
                    <h3>📤 转换结果</h3>
                    {#if getCompressionRatio()}
                        <span class="compression-badge">
                            {parseFloat(getCompressionRatio()) > 0 ? '压缩' : '增大'} {Math.abs(parseFloat(getCompressionRatio()))}%
                        </span>
                    {/if}
                </div>
                
                {#each convertedResults as result}
                    {#if result.success && result.base64_data}
                        <div class="result-card">
                            <div class="result-preview">
                                <img src={result.base64_data} alt="转换后图片" />
                            </div>
                            <div class="result-info">
                                <div class="result-meta">
                                    <div class="result-name">{result.file_name}</div>
                                    <div class="result-details">
                                        {#if result.file_size !== undefined}
                                            <span class="detail-item">
                                                <span class="detail-icon">📦</span>
                                                {formatFileSize(result.file_size)}
                                            </span>
                                        {/if}
                                        <span class="detail-item">
                                            <span class="detail-icon">🎨</span>
                                            {outputFormat}
                                        </span>
                                    </div>
                                </div>
                                <button
                                    class="save-btn"
                                    onclick={() => saveImage(result)}
                                >
                                    <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                        <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
                                        <polyline points="7 10 12 15 17 10"/>
                                        <line x1="12" y1="15" x2="12" y2="3"/>
                                    </svg>
                                    保存文件
                                </button>
                            </div>
                        </div>
                    {/if}
                {/each}
            </div>
        {/if}
    {/if}
</div>

<style>
    .image-tool {
        display: flex;
        flex-direction: column;
        gap: 1.5rem;
    }

    /* 标题区域 */
    .tool-header {
        display: flex;
        justify-content: space-between;
        align-items: flex-start;
        padding-bottom: 1rem;
        border-bottom: 1px solid var(--border);
    }

    .header-content h2 {
        font-size: 1.5rem;
        margin: 0 0 0.25rem 0;
        background: linear-gradient(135deg, var(--primary) 0%, #a855f7 100%);
        -webkit-background-clip: text;
        -webkit-text-fill-color: transparent;
        background-clip: text;
    }

    .header-desc {
        font-size: 0.9rem;
        color: var(--text-muted);
        margin: 0;
    }

    .header-btn {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        padding: 0.5rem 1rem !important;
        background: var(--bg-card) !important;
        border: 1px solid var(--border) !important;
        font-size: 0.85rem;
    }

    .header-btn:hover {
        border-color: var(--primary) !important;
    }

    /* 上传区域 */
    .upload-area {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        padding: 3rem 2rem;
        border: 2px dashed var(--border);
        border-radius: var(--radius-sm);
        background: linear-gradient(135deg, var(--bg-dark) 0%, rgba(99, 102, 241, 0.02) 100%);
        cursor: pointer;
        transition: all 0.3s ease;
    }

    .upload-area:hover, .upload-area.dragging {
        border-color: var(--primary);
        background: linear-gradient(135deg, var(--bg-dark) 0%, rgba(99, 102, 241, 0.08) 100%);
    }

    .upload-visual {
        display: flex;
        flex-direction: column;
        align-items: center;
        margin-bottom: 1.5rem;
    }

    .upload-icon {
        color: var(--primary);
        margin-bottom: 1rem;
        opacity: 0.8;
    }

    .upload-visual h3 {
        font-size: 1.25rem;
        margin: 0 0 0.5rem 0;
    }

    .upload-visual p {
        color: var(--text-muted);
        font-size: 0.9rem;
        margin: 0;
    }

    .supported-formats {
        display: flex;
        flex-wrap: wrap;
        gap: 0.5rem;
        justify-content: center;
    }

    .format-tag {
        padding: 0.25rem 0.75rem;
        background: var(--bg-card);
        border: 1px solid var(--border);
        border-radius: 999px;
        font-size: 0.75rem;
        color: var(--text-secondary);
        font-family: 'JetBrains Mono', monospace;
    }

    /* 快速预设 */
    .quick-presets {
        background: var(--bg-dark);
        border: 1px solid var(--border);
        border-radius: var(--radius-sm);
        padding: 1.25rem;
    }

    .quick-presets h3 {
        font-size: 1rem;
        margin: 0 0 1rem 0;
    }

    .preset-groups {
        display: flex;
        flex-direction: column;
        gap: 1rem;
    }

    .preset-group {
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
    }

    .group-header {
        font-size: 0.8rem;
        color: var(--text-muted);
        text-transform: uppercase;
        letter-spacing: 0.05em;
    }

    .group-sizes {
        display: flex;
        flex-wrap: wrap;
        gap: 0.5rem;
    }

    .preset-chip {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        padding: 0.5rem 0.75rem !important;
        background: var(--bg-card) !important;
        border: 1px solid var(--border) !important;
        border-radius: 999px !important;
        font-size: 0.85rem;
        transition: all 0.2s;
    }

    .preset-chip:hover {
        border-color: var(--primary) !important;
        background: rgba(99, 102, 241, 0.1) !important;
    }

    .chip-icon {
        font-size: 1rem;
    }

    /* 编辑器布局 */
    .editor-layout {
        display: grid;
        grid-template-columns: 1fr 1fr;
        gap: 1.5rem;
    }

    @media (max-width: 1000px) {
        .editor-layout {
            grid-template-columns: 1fr;
        }
    }

    /* 预览面板 */
    .preview-panel {
        background: var(--bg-dark);
        border: 1px solid var(--border);
        border-radius: var(--radius-sm);
        overflow: hidden;
    }

    .panel-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 0.75rem 1rem;
        background: var(--bg-card);
        border-bottom: 1px solid var(--border);
    }

    .panel-title {
        font-weight: 600;
        font-size: 0.9rem;
    }

    .panel-badge {
        font-size: 0.75rem;
        padding: 0.25rem 0.5rem;
        background: var(--primary);
        color: white;
        border-radius: 4px;
        font-family: 'JetBrains Mono', monospace;
    }

    .preview-box {
        display: flex;
        align-items: center;
        justify-content: center;
        min-height: 280px;
        max-height: 350px;
        background: 
            linear-gradient(45deg, var(--bg-card) 25%, transparent 25%),
            linear-gradient(-45deg, var(--bg-card) 25%, transparent 25%),
            linear-gradient(45deg, transparent 75%, var(--bg-card) 75%),
            linear-gradient(-45deg, transparent 75%, var(--bg-card) 75%);
        background-size: 20px 20px;
        background-position: 0 0, 0 10px, 10px -10px, -10px 0px;
        background-color: var(--bg-dark);
    }

    .preview-box img {
        max-width: 100%;
        max-height: 350px;
        object-fit: contain;
    }

    .loading-state {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 0.75rem;
        color: var(--text-muted);
    }

    .spinner {
        width: 40px;
        height: 40px;
        border: 3px solid var(--border);
        border-top-color: var(--primary);
        border-radius: 50%;
        animation: spin 1s linear infinite;
    }

    @keyframes spin {
        to { transform: rotate(360deg); }
    }

    .empty-state {
        color: var(--text-muted);
    }

    /* 图片统计信息 */
    .image-stats {
        padding: 1rem;
        display: flex;
        flex-direction: column;
        gap: 0.75rem;
    }

    .stat-row {
        display: flex;
        gap: 0.75rem;
    }

    .stat-item {
        flex: 1;
        display: flex;
        align-items: center;
        gap: 0.75rem;
        padding: 0.75rem;
        background: var(--bg-card);
        border-radius: var(--radius-sm);
    }

    .stat-item.full-width {
        flex: auto;
    }

    .stat-icon {
        font-size: 1.25rem;
    }

    .stat-content {
        display: flex;
        flex-direction: column;
    }

    .stat-label {
        font-size: 0.7rem;
        color: var(--text-muted);
        text-transform: uppercase;
        letter-spacing: 0.03em;
    }

    .stat-value {
        font-size: 0.9rem;
        font-weight: 500;
        font-family: 'JetBrains Mono', monospace;
    }

    /* 设置面板 */
    .settings-panel {
        display: flex;
        flex-direction: column;
        background: var(--bg-dark);
        border: 1px solid var(--border);
        border-radius: var(--radius-sm);
        overflow: hidden;
    }

    .settings-tabs {
        display: flex;
        background: var(--bg-card);
        border-bottom: 1px solid var(--border);
    }

    .tab-btn {
        flex: 1;
        padding: 0.75rem !important;
        background: transparent !important;
        border: none !important;
        border-bottom: 2px solid transparent !important;
        border-radius: 0 !important;
        font-size: 0.85rem;
        color: var(--text-secondary) !important;
        transition: all 0.2s;
    }

    .tab-btn:hover {
        color: var(--text-primary) !important;
        background: rgba(99, 102, 241, 0.05) !important;
    }

    .tab-btn.active {
        color: var(--primary) !important;
        border-bottom-color: var(--primary) !important;
    }

    .settings-content {
        flex: 1;
        overflow-y: auto;
    }

    .tab-panel {
        padding: 1rem;
    }

    /* 格式选择 */
    .format-grid {
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
    }

    .format-card {
        display: flex;
        align-items: center;
        gap: 0.75rem;
        padding: 0.75rem !important;
        background: var(--bg-card) !important;
        border: 1px solid var(--border) !important;
        text-align: left;
        transition: all 0.2s;
        position: relative;
    }

    .format-card.active {
        border-color: var(--format-color, var(--primary)) !important;
        background: rgba(99, 102, 241, 0.05) !important;
    }

    .format-card:hover:not(.active) {
        border-color: var(--text-muted) !important;
    }

    .format-icon {
        font-size: 1.5rem;
    }

    .format-info {
        display: flex;
        flex-direction: column;
        flex: 1;
    }

    .format-name {
        font-weight: 600;
        font-size: 0.9rem;
    }

    .format-desc {
        font-size: 0.75rem;
        color: var(--text-muted);
    }

    .format-check {
        position: absolute;
        right: 0.75rem;
        width: 20px;
        height: 20px;
        background: var(--format-color, var(--primary));
        color: white;
        border-radius: 50%;
        display: flex;
        align-items: center;
        justify-content: center;
        font-size: 0.7rem;
        font-weight: bold;
    }

    /* 质量设置 */
    .quality-section {
        margin-top: 1rem;
        padding: 1rem;
        background: var(--bg-card);
        border-radius: var(--radius-sm);
    }

    .quality-header {
        display: flex;
        justify-content: space-between;
        margin-bottom: 0.75rem;
        font-size: 0.9rem;
    }

    .quality-value {
        font-weight: 600;
        color: var(--primary);
        font-family: 'JetBrains Mono', monospace;
    }

    .quality-slider-container {
        margin-bottom: 0.75rem;
    }

    .quality-slider {
        width: 100%;
        height: 8px;
        -webkit-appearance: none;
        appearance: none;
        background: linear-gradient(to right, #ef4444, #eab308, #22c55e);
        border-radius: 4px;
        outline: none;
    }

    .quality-slider::-webkit-slider-thumb {
        -webkit-appearance: none;
        appearance: none;
        width: 20px;
        height: 20px;
        background: white;
        border: 2px solid var(--primary);
        border-radius: 50%;
        cursor: pointer;
        box-shadow: 0 2px 4px rgba(0,0,0,0.2);
    }

    .quality-marks {
        display: flex;
        justify-content: space-between;
        font-size: 0.7rem;
        color: var(--text-muted);
        margin-top: 0.25rem;
    }

    .quality-presets {
        display: flex;
        flex-wrap: wrap;
        gap: 0.5rem;
    }

    .quality-presets button {
        flex: 1;
        min-width: 80px;
        padding: 0.4rem 0.5rem !important;
        background: var(--bg-dark) !important;
        border: 1px solid var(--border) !important;
        font-size: 0.75rem;
    }

    .quality-presets button.active {
        background: var(--primary) !important;
        border-color: var(--primary) !important;
        color: white !important;
    }

    /* ICO 尺寸 */
    .ico-section {
        margin-top: 1rem;
        padding: 1rem;
        background: var(--bg-card);
        border-radius: var(--radius-sm);
    }

    .section-label {
        font-size: 0.85rem;
        margin-bottom: 0.75rem;
        color: var(--text-secondary);
    }

    .ico-sizes {
        display: flex;
        flex-wrap: wrap;
        gap: 0.5rem;
    }

    .ico-size-btn {
        padding: 0.5rem 0.75rem !important;
        background: var(--bg-dark) !important;
        border: 1px solid var(--border) !important;
        font-size: 0.85rem;
        font-family: 'JetBrains Mono', monospace;
    }

    .ico-size-btn.active {
        background: var(--primary) !important;
        border-color: var(--primary) !important;
        color: white !important;
    }

    .hint-text {
        font-size: 0.8rem;
        color: var(--text-muted);
        margin-top: 0.75rem;
        margin-bottom: 0;
    }

    /* 尺寸调整 */
    .resize-toggle {
        display: flex;
        align-items: center;
        gap: 0.75rem;
        margin-bottom: 1rem;
    }

    .toggle-switch {
        position: relative;
        display: inline-block;
        width: 44px;
        height: 24px;
    }

    .toggle-switch input {
        opacity: 0;
        width: 0;
        height: 0;
    }

    .toggle-slider {
        position: absolute;
        cursor: pointer;
        top: 0;
        left: 0;
        right: 0;
        bottom: 0;
        background-color: var(--bg-card);
        border: 1px solid var(--border);
        border-radius: 24px;
        transition: 0.3s;
    }

    .toggle-slider:before {
        position: absolute;
        content: "";
        height: 18px;
        width: 18px;
        left: 2px;
        bottom: 2px;
        background-color: var(--text-muted);
        border-radius: 50%;
        transition: 0.3s;
    }

    .toggle-switch input:checked + .toggle-slider {
        background-color: var(--primary);
        border-color: var(--primary);
    }

    .toggle-switch input:checked + .toggle-slider:before {
        background-color: white;
        transform: translateX(20px);
    }

    .resize-options {
        display: flex;
        flex-direction: column;
        gap: 1rem;
    }

    .custom-size-section {
        padding: 1rem;
        background: var(--bg-card);
        border-radius: var(--radius-sm);
    }

    .size-inputs {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        margin-bottom: 1rem;
    }

    .size-input-group {
        flex: 1;
    }

    .size-input-group label {
        display: block;
        font-size: 0.75rem;
        color: var(--text-muted);
        margin-bottom: 0.25rem;
    }

    .input-wrapper {
        display: flex;
        align-items: center;
        background: var(--bg-dark);
        border: 1px solid var(--border);
        border-radius: var(--radius-sm);
        overflow: hidden;
    }

    .input-wrapper input {
        flex: 1;
        border: none !important;
        background: transparent !important;
        text-align: center;
        padding: 0.5rem !important;
    }

    .input-suffix {
        padding: 0 0.5rem;
        color: var(--text-muted);
        font-size: 0.8rem;
        background: var(--bg-card);
    }

    .size-link {
        display: flex;
        align-items: center;
        justify-content: center;
        width: 36px;
        height: 36px;
        margin-top: 1rem;
        color: var(--text-muted);
        opacity: 0.5;
        transition: all 0.2s;
    }

    .size-link.active {
        color: var(--primary);
        opacity: 1;
    }

    .aspect-ratio-toggle {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        font-size: 0.85rem;
        cursor: pointer;
        margin-bottom: 0.75rem;
    }

    .aspect-ratio-toggle input {
        width: 16px;
        height: 16px;
    }

    .reset-size-btn {
        width: 100%;
        padding: 0.5rem !important;
        background: transparent !important;
        border: 1px dashed var(--border) !important;
        font-size: 0.85rem;
        color: var(--text-secondary) !important;
    }

    .reset-size-btn:hover {
        border-color: var(--primary) !important;
        color: var(--primary) !important;
    }

    /* 预设尺寸迷你版 */
    .preset-size-section {
        padding: 1rem;
        background: var(--bg-card);
        border-radius: var(--radius-sm);
    }

    .preset-mini-group {
        margin-bottom: 0.75rem;
    }

    .preset-mini-group:last-child {
        margin-bottom: 0;
    }

    .mini-group-label {
        display: block;
        font-size: 0.7rem;
        color: var(--text-muted);
        text-transform: uppercase;
        letter-spacing: 0.05em;
        margin-bottom: 0.5rem;
    }

    .mini-preset-list {
        display: flex;
        flex-wrap: wrap;
        gap: 0.375rem;
    }

    .mini-preset-btn {
        padding: 0.25rem 0.5rem !important;
        background: var(--bg-dark) !important;
        border: 1px solid var(--border) !important;
        font-size: 0.7rem;
        white-space: nowrap;
    }

    .mini-preset-btn:hover {
        border-color: var(--primary) !important;
    }

    .resize-disabled-hint {
        padding: 2rem;
        text-align: center;
        color: var(--text-muted);
    }

    .resize-disabled-hint p {
        margin: 0.5rem 0;
    }

    /* 输出摘要 */
    .output-summary h4 {
        margin: 0 0 1rem 0;
        font-size: 0.9rem;
    }

    .summary-grid {
        display: flex;
        flex-direction: column;
        gap: 0.75rem;
    }

    .summary-item {
        display: flex;
        justify-content: space-between;
        padding: 0.75rem;
        background: var(--bg-card);
        border-radius: var(--radius-sm);
    }

    .summary-label {
        color: var(--text-muted);
        font-size: 0.85rem;
    }

    .summary-value {
        font-weight: 600;
        font-family: 'JetBrains Mono', monospace;
    }

    /* 转换按钮 */
    .convert-btn {
        margin: 1rem;
        padding: 1rem !important;
        background: linear-gradient(135deg, var(--primary) 0%, #7c3aed 100%) !important;
        font-size: 1rem;
        font-weight: 600;
        display: flex;
        align-items: center;
        justify-content: center;
        gap: 0.5rem;
    }

    .convert-btn:disabled {
        opacity: 0.6;
        cursor: not-allowed;
    }

    .convert-btn svg {
        stroke: white;
    }

    .btn-spinner {
        width: 18px;
        height: 18px;
        border: 2px solid rgba(255,255,255,0.3);
        border-top-color: white;
        border-radius: 50%;
        animation: spin 1s linear infinite;
    }

    /* 消息提示 */
    .message {
        display: flex;
        align-items: center;
        gap: 0.75rem;
        padding: 0.875rem 1rem;
        border-radius: var(--radius-sm);
        font-size: 0.9rem;
    }

    .message-icon {
        font-size: 1.25rem;
    }

    .message.success {
        background: rgba(16, 185, 129, 0.1);
        border: 1px solid rgba(16, 185, 129, 0.3);
        color: var(--accent-green);
    }

    .message.error {
        background: rgba(239, 68, 68, 0.1);
        border: 1px solid rgba(239, 68, 68, 0.3);
        color: #ef4444;
    }

    /* 结果区域 */
    .results-section {
        background: var(--bg-dark);
        border: 1px solid var(--border);
        border-radius: var(--radius-sm);
        overflow: hidden;
    }

    .results-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 0.875rem 1rem;
        background: var(--bg-card);
        border-bottom: 1px solid var(--border);
    }

    .results-header h3 {
        font-size: 1rem;
        margin: 0;
    }

    .compression-badge {
        font-size: 0.75rem;
        padding: 0.25rem 0.75rem;
        background: var(--accent-green);
        color: white;
        border-radius: 999px;
        font-weight: 500;
    }

    .result-card {
        display: flex;
        gap: 1rem;
        padding: 1rem;
    }

    @media (max-width: 600px) {
        .result-card {
            flex-direction: column;
        }
    }

    .result-preview {
        flex-shrink: 0;
        display: flex;
        align-items: center;
        justify-content: center;
        width: 180px;
        height: 180px;
        background: 
            linear-gradient(45deg, var(--bg-card) 25%, transparent 25%),
            linear-gradient(-45deg, var(--bg-card) 25%, transparent 25%),
            linear-gradient(45deg, transparent 75%, var(--bg-card) 75%),
            linear-gradient(-45deg, transparent 75%, var(--bg-card) 75%);
        background-size: 16px 16px;
        background-position: 0 0, 0 8px, 8px -8px, -8px 0px;
        background-color: var(--bg-dark);
        border-radius: var(--radius-sm);
        overflow: hidden;
    }

    .result-preview img {
        max-width: 100%;
        max-height: 100%;
        object-fit: contain;
    }

    .result-info {
        flex: 1;
        display: flex;
        flex-direction: column;
        justify-content: space-between;
        gap: 1rem;
    }

    .result-meta {
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
    }

    .result-name {
        font-weight: 600;
        word-break: break-all;
        font-family: 'JetBrains Mono', monospace;
        font-size: 0.9rem;
    }

    .result-details {
        display: flex;
        gap: 1rem;
    }

    .detail-item {
        display: flex;
        align-items: center;
        gap: 0.375rem;
        font-size: 0.85rem;
        color: var(--text-secondary);
    }

    .detail-icon {
        font-size: 0.9rem;
    }

    .save-btn {
        display: flex;
        align-items: center;
        justify-content: center;
        gap: 0.5rem;
        padding: 0.75rem 1.5rem !important;
        background: var(--accent-green) !important;
        font-weight: 500;
        align-self: flex-start;
    }

    .save-btn svg {
        stroke: white;
    }
</style>
