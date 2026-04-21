<script lang="ts">
    let base64Result = $state("");
    let fileName = $state("");
    let fileSize = $state(0);
    let fileType = $state("");
    let previewUrl = $state("");
    let copied = $state(false);
    let isDragging = $state(false);
    let fileInputEl = $state<HTMLInputElement | null>(null);

    function handleFileChange(event: Event) {
        const fileInput = event.target as HTMLInputElement;
        if (fileInput.files?.[0]) {
            processFile(fileInput.files[0]);
        }
    }

    function handleDrop(event: DragEvent) {
        event.preventDefault();
        isDragging = false;
        const file = event.dataTransfer?.files[0];
        if (file && file.type.startsWith('image/')) {
            processFile(file);
        }
    }

    function handleDragOver(event: DragEvent) {
        event.preventDefault();
        isDragging = true;
    }

    function handleDragLeave() {
        isDragging = false;
    }

    function processFile(file: File) {
        fileName = file.name;
        fileSize = file.size;
        fileType = file.type;

        const reader = new FileReader();
        reader.onload = (e) => {
            const result = e.target?.result as string;
            base64Result = result;
            previewUrl = result;
        };
        reader.readAsDataURL(file);
    }

    async function copyToClipboard() {
        if (base64Result) {
            try {
                await navigator.clipboard.writeText(base64Result);
                copied = true;
                setTimeout(() => copied = false, 1500);
            } catch (err) {
                console.error('复制失败:', err);
            }
        }
    }

    function clearFile() {
        base64Result = "";
        fileName = "";
        fileSize = 0;
        fileType = "";
        previewUrl = "";
    }

    function formatFileSize(bytes: number): string {
        if (bytes < 1024) return bytes + ' B';
        if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(2) + ' KB';
        return (bytes / (1024 * 1024)).toFixed(2) + ' MB';
    }

    function openFileDialog() {
        fileInputEl?.click();
    }

    function handleUploadAreaKeydown(event: KeyboardEvent) {
        if (event.key !== "Enter" && event.key !== " ") return;
        event.preventDefault();
        openFileDialog();
    }
</script>

<div class="base64-tool">
    <div 
        class="upload-area"
        class:dragging={isDragging}
        class:has-file={!!fileName}
        ondrop={handleDrop}
        ondragover={handleDragOver}
        ondragleave={handleDragLeave}
        onclick={openFileDialog}
        onkeydown={handleUploadAreaKeydown}
        role="button"
        tabindex="0"
    >
        <input 
            bind:this={fileInputEl}
            type="file" 
            accept="image/*" 
            onchange={handleFileChange}
            hidden
        />
        
        {#if fileName}
            <div class="file-preview">
                {#if previewUrl}
                    <img src={previewUrl} alt="预览" class="preview-image" />
                {/if}
                <div class="file-info">
                    <span class="file-name">{fileName}</span>
                    <span class="file-meta">
                        {fileType} · {formatFileSize(fileSize)}
                    </span>
                </div>
            </div>
            <button class="clear-btn" onclick={(e) => { e.stopPropagation(); clearFile(); }}>
                ✕ 清除
            </button>
        {:else}
            <div class="upload-hint">
                <span class="upload-icon">🖼️</span>
                <span class="upload-text">点击或拖放图片到此处</span>
                <span class="upload-formats">支持 JPG, PNG, GIF, WebP, SVG 等格式</span>
            </div>
        {/if}
    </div>

    {#if base64Result}
        <div class="result-section">
            <div class="result-header">
                <div class="result-info">
                    <h3>Base64 编码结果</h3>
                    <span class="result-size">
                        {base64Result.length.toLocaleString()} 字符
                        (约 {formatFileSize(base64Result.length)})
                    </span>
                </div>
                <button class="copy-btn" onclick={copyToClipboard}>
                    {copied ? '✓ 已复制' : '📋 复制 Base64'}
                </button>
            </div>
            <textarea 
                readonly 
                value={base64Result}
                class="result-textarea"
            ></textarea>
        </div>

        <div class="usage-section">
            <h3>使用示例</h3>
            <div class="usage-grid">
                <div class="usage-card">
                    <h4>HTML</h4>
                    <code>&lt;img src="{base64Result.substring(0, 50)}..." /&gt;</code>
                </div>
                <div class="usage-card">
                    <h4>CSS</h4>
                    <code>background-image: url({base64Result.substring(0, 50)}...);</code>
                </div>
            </div>
        </div>
    {/if}

    <div class="info-section">
        <h3>关于 Base64 编码</h3>
        <ul>
            <li><strong>用途</strong>：将图片转换为文本格式，可直接嵌入 HTML/CSS</li>
            <li><strong>优点</strong>：减少 HTTP 请求，适合小图片和图标</li>
            <li><strong>缺点</strong>：编码后体积增大约 33%，不适合大图片</li>
            <li><strong>建议</strong>：图片小于 10KB 时使用 Base64 效果较好</li>
        </ul>
    </div>
</div>

<style>
    .base64-tool {
        display: flex;
        flex-direction: column;
        gap: 1.5rem;
    }

    .upload-area {
        position: relative;
        border: 2px dashed var(--border);
        border-radius: var(--radius-sm);
        padding: 2rem;
        text-align: center;
        cursor: pointer;
        transition: all 0.2s;
        min-height: 200px;
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
    }

    .upload-area:hover {
        border-color: var(--primary);
        background: var(--bg-hover);
    }

    .upload-area.dragging {
        border-color: var(--accent);
        background: rgba(34, 211, 238, 0.1);
    }

    .upload-area.has-file {
        border-style: solid;
        padding: 1rem;
    }

    .upload-hint {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 0.75rem;
    }

    .upload-icon {
        font-size: 3rem;
    }

    .upload-text {
        font-size: 1.1rem;
        color: var(--text-primary);
        font-weight: 500;
    }

    .upload-formats {
        font-size: 0.85rem;
        color: var(--text-muted);
    }

    .file-preview {
        display: flex;
        align-items: center;
        gap: 1rem;
        width: 100%;
    }

    .preview-image {
        width: 80px;
        height: 80px;
        object-fit: cover;
        border-radius: var(--radius-sm);
        border: 1px solid var(--border);
    }

    .file-info {
        display: flex;
        flex-direction: column;
        gap: 0.25rem;
        text-align: left;
    }

    .file-name {
        font-weight: 500;
        color: var(--text-primary);
    }

    .file-meta {
        font-size: 0.85rem;
        color: var(--text-muted);
    }

    .clear-btn {
        position: absolute;
        top: 1rem;
        right: 1rem;
        padding: 0.4rem 0.85rem !important;
        font-size: 0.85rem !important;
        background: var(--bg-dark) !important;
        border: 1px solid var(--border) !important;
    }

    .result-section {
        border: 1px solid var(--border);
        border-radius: var(--radius-sm);
        overflow: hidden;
    }

    .result-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 1rem;
        background: var(--bg-dark);
        border-bottom: 1px solid var(--border);
    }

    .result-info {
        display: flex;
        flex-direction: column;
        gap: 0.25rem;
    }

    .result-info h3 {
        font-size: 0.95rem;
        margin: 0;
    }

    .result-size {
        font-size: 0.8rem;
        color: var(--text-muted);
    }

    .copy-btn {
        padding: 0.65rem 1.25rem !important;
        font-weight: 500 !important;
    }

    .result-textarea {
        width: 100%;
        min-height: 200px;
        border: none !important;
        border-radius: 0 !important;
        resize: vertical;
        font-family: 'JetBrains Mono', monospace;
        font-size: 0.85rem;
        background: var(--bg-dark) !important;
        color: var(--accent);
        padding: 1rem;
    }

    .usage-section {
        padding: 1rem;
        background: var(--bg-dark);
        border: 1px solid var(--border);
        border-radius: var(--radius-sm);
    }

    .usage-section h3 {
        font-size: 0.9rem;
        margin-bottom: 0.75rem;
        color: var(--text-secondary);
    }

    .usage-grid {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
        gap: 0.75rem;
    }

    .usage-card {
        padding: 0.75rem;
        background: var(--bg-card);
        border-radius: var(--radius-sm);
    }

    .usage-card h4 {
        font-size: 0.85rem;
        margin-bottom: 0.5rem;
        color: var(--accent);
    }

    .usage-card code {
        font-size: 0.8rem;
        color: var(--text-muted);
        word-break: break-all;
        display: block;
        background: transparent !important;
        padding: 0 !important;
    }

    .info-section {
        padding: 1rem;
        background: var(--bg-dark);
        border: 1px solid var(--border);
        border-radius: var(--radius-sm);
    }

    .info-section h3 {
        font-size: 0.9rem;
        margin-bottom: 0.75rem;
        color: var(--text-secondary);
    }

    .info-section ul {
        list-style: none;
        padding: 0;
        margin: 0;
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
    }

    .info-section li {
        font-size: 0.85rem;
        color: var(--text-muted);
    }

    .info-section li strong {
        color: var(--accent);
    }
</style>
