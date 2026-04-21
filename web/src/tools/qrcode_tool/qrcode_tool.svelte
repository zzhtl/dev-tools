<script lang="ts">
    import QRCode from "qrcode";
    import { onMount } from "svelte";
    import { downloadDataUrl } from "../../lib/download";
  
    let text = $state("https://example.com");
    let qrCodeUrl = $state("");
    let qrSize = $state(256);
    let errorLevel = $state<"L" | "M" | "Q" | "H">("M");
    let darkColor = $state("#000000");
    let lightColor = $state("#ffffff");
    let copied = $state(false);

    const presets = [
        { label: "网址", value: "https://example.com", icon: "🌐" },
        { label: "邮箱", value: "mailto:example@mail.com", icon: "📧" },
        { label: "电话", value: "tel:+8612345678901", icon: "📞" },
        { label: "WiFi", value: "WIFI:S:NetworkName;T:WPA;P:password;;", icon: "📶" },
    ];
  
    const generateQRCode = async () => {
        try {
            qrCodeUrl = await QRCode.toDataURL(text, {
                width: qrSize,
                margin: 2,
                errorCorrectionLevel: errorLevel,
                color: {
                    dark: darkColor,
                    light: lightColor,
                },
            });
        } catch (err) {
            console.error("生成二维码失败:", err);
        }
    };
  
    const saveQRCode = () => {
        if (!qrCodeUrl) return;
        downloadDataUrl(qrCodeUrl, 'qrcode.png');
    };

    async function copyQRCode() {
        if (!qrCodeUrl) return;
        try {
            const base64Data = qrCodeUrl.split(',')[1];
            const binary = Uint8Array.from(atob(base64Data), c => c.charCodeAt(0));
            const blob = new Blob([binary], { type: 'image/png' });
            await navigator.clipboard.write([
                new ClipboardItem({ 'image/png': blob })
            ]);
            copied = true;
            setTimeout(() => copied = false, 1500);
        } catch (err) {
            console.error('复制失败:', err);
        }
    }

    function selectPreset(value: string) {
        text = value;
        generateQRCode();
    }
  
    onMount(() => {
        generateQRCode();
    });

    // 响应内容变化
    $effect(() => {
        if (text) {
            const timer = setTimeout(() => {
                generateQRCode();
            }, 300);
            return () => clearTimeout(timer);
        }
    });
</script>
  
<div class="qrcode-tool">
    <div class="input-section">
        <div class="input-header">
            <label for="qrcode-text-input">输入内容</label>
            <div class="preset-buttons">
                {#each presets as preset}
                    <button 
                        class="preset-btn"
                        onclick={() => selectPreset(preset.value)}
                        title={preset.value}
                    >
                        {preset.icon} {preset.label}
                    </button>
                {/each}
            </div>
        </div>
        <textarea
            id="qrcode-text-input"
            bind:value={text}
            placeholder="输入文本、网址、或其他内容..."
            rows="4"
        ></textarea>
        <div class="char-count">{text.length} 字符</div>
    </div>

    <div class="main-content">
        <div class="options-section">
            <h3>生成选项</h3>
            
            <div class="option-group">
                <div class="option-group-title">二维码尺寸</div>
                <div class="size-options">
                    {#each [128, 256, 512, 1024] as size}
                        <button
                            class="size-btn"
                            class:active={qrSize === size}
                            onclick={() => { qrSize = size; generateQRCode(); }}
                        >
                            {size}px
                        </button>
                    {/each}
                </div>
            </div>

            <div class="option-group">
                <div class="option-group-title">容错级别</div>
                <div class="level-options">
                    {#each [
                        { value: "L", label: "L (7%)", desc: "低" },
                        { value: "M", label: "M (15%)", desc: "中" },
                        { value: "Q", label: "Q (25%)", desc: "较高" },
                        { value: "H", label: "H (30%)", desc: "高" },
                    ] as level}
                        <button
                            class="level-btn"
                            class:active={errorLevel === level.value}
                            onclick={() => { errorLevel = level.value as "L" | "M" | "Q" | "H"; generateQRCode(); }}
                            title={level.desc}
                        >
                            {level.label}
                        </button>
                    {/each}
                </div>
            </div>

            <div class="option-group">
                <div class="option-group-title">颜色设置</div>
                <div class="color-options">
                    <div class="color-input">
                        <span>前景色</span>
                        <input
                            type="color"
                            bind:value={darkColor}
                            onchange={generateQRCode}
                        />
                        <code>{darkColor}</code>
                    </div>
                    <div class="color-input">
                        <span>背景色</span>
                        <input
                            type="color"
                            bind:value={lightColor}
                            onchange={generateQRCode}
                        />
                        <code>{lightColor}</code>
                    </div>
                </div>
            </div>
        </div>

        <div class="preview-section">
            <div class="preview-header">
                <h3>预览</h3>
                <span class="size-badge">{qrSize} × {qrSize}</span>
            </div>
            
            {#if qrCodeUrl}
                <div class="qr-preview">
                    <img src={qrCodeUrl} alt="二维码" />
                </div>
                <div class="action-buttons">
                    <button class="action-btn primary" onclick={saveQRCode}>
                        💾 保存图片
                    </button>
                    <button class="action-btn secondary" onclick={copyQRCode}>
                        {copied ? '✓ 已复制' : '📋 复制图片'}
                    </button>
                </div>
            {:else}
                <div class="empty-preview">
                    <span>输入内容后生成二维码</span>
                </div>
            {/if}
        </div>
    </div>

    <div class="info-section">
        <h3>二维码内容格式</h3>
        <div class="format-grid">
            <div class="format-item">
                <span class="format-type">网址</span>
                <code>https://example.com</code>
            </div>
            <div class="format-item">
                <span class="format-type">邮箱</span>
                <code>mailto:user@example.com</code>
            </div>
            <div class="format-item">
                <span class="format-type">电话</span>
                <code>tel:+8612345678901</code>
            </div>
            <div class="format-item">
                <span class="format-type">WiFi</span>
                <code>WIFI:S:名称;T:WPA;P:密码;;</code>
            </div>
        </div>
    </div>
</div>
  
<style>
    .qrcode-tool {
        display: flex;
        flex-direction: column;
        gap: 1.5rem;
    }

    .input-section {
        padding: 1rem;
        background: var(--bg-dark);
        border: 1px solid var(--border);
        border-radius: var(--radius-sm);
    }

    .input-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 0.75rem;
        flex-wrap: wrap;
        gap: 0.5rem;
    }

    .input-header label {
        font-weight: 500;
    }

    .preset-buttons {
        display: flex;
        gap: 0.5rem;
        flex-wrap: wrap;
    }

    .preset-btn {
        padding: 0.35rem 0.75rem !important;
        font-size: 0.8rem !important;
        background: var(--bg-card) !important;
        border: 1px solid var(--border) !important;
    }

    .preset-btn:hover {
        border-color: var(--primary) !important;
        transform: none !important;
        box-shadow: none !important;
    }

    .input-section textarea {
        width: 100%;
        resize: vertical;
        font-family: 'JetBrains Mono', monospace;
    }

    .char-count {
        text-align: right;
        font-size: 0.8rem;
        color: var(--text-muted);
        margin-top: 0.5rem;
    }

    .main-content {
        display: grid;
        grid-template-columns: 1fr 1fr;
        gap: 1.5rem;
    }

    .options-section, .preview-section {
        padding: 1rem;
        background: var(--bg-dark);
        border: 1px solid var(--border);
        border-radius: var(--radius-sm);
    }

    .options-section h3, .preview-section h3 {
        font-size: 0.95rem;
        margin-bottom: 1rem;
    }

    .option-group {
        margin-bottom: 1.25rem;
    }

    .option-group:last-child {
        margin-bottom: 0;
    }

    .option-group-title {
        display: block;
        font-size: 0.85rem;
        color: var(--text-secondary);
        margin-bottom: 0.5rem;
    }

    .size-options, .level-options {
        display: flex;
        gap: 0.5rem;
    }

    .size-btn, .level-btn {
        flex: 1;
        padding: 0.5rem !important;
        font-size: 0.85rem !important;
        background: var(--bg-card) !important;
        border: 1px solid var(--border) !important;
    }

    .size-btn:hover, .level-btn:hover {
        border-color: var(--primary) !important;
        transform: none !important;
        box-shadow: none !important;
    }

    .size-btn.active, .level-btn.active {
        background: var(--primary) !important;
        border-color: var(--primary) !important;
        color: white !important;
    }

    .color-options {
        display: flex;
        flex-direction: column;
        gap: 0.75rem;
    }

    .color-input {
        display: flex;
        align-items: center;
        gap: 0.75rem;
    }

    .color-input span {
        min-width: 60px;
        font-size: 0.85rem;
        color: var(--text-secondary);
    }

    .color-input input[type="color"] {
        width: 40px;
        height: 32px;
        padding: 0 !important;
        border: 1px solid var(--border) !important;
        border-radius: 4px !important;
        cursor: pointer;
    }

    .color-input code {
        font-size: 0.85rem;
        background: transparent !important;
        padding: 0 !important;
        color: var(--text-muted);
    }

    .preview-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 1rem;
    }

    .preview-header h3 {
        margin-bottom: 0;
    }

    .size-badge {
        font-size: 0.8rem;
        padding: 0.25rem 0.75rem;
        background: var(--bg-card);
        border-radius: 100px;
        color: var(--text-muted);
    }

    .qr-preview {
        display: flex;
        justify-content: center;
        padding: 1rem;
        background: white;
        border-radius: var(--radius-sm);
        margin-bottom: 1rem;
    }

    .qr-preview img {
        max-width: 100%;
        height: auto;
    }

    .empty-preview {
        display: flex;
        align-items: center;
        justify-content: center;
        min-height: 200px;
        color: var(--text-muted);
        background: var(--bg-card);
        border-radius: var(--radius-sm);
    }

    .action-buttons {
        display: flex;
        gap: 0.75rem;
    }

    .action-btn {
        flex: 1;
        padding: 0.75rem !important;
        font-weight: 500 !important;
    }

    .action-btn.primary {
        background: linear-gradient(135deg, var(--primary) 0%, var(--primary-dark) 100%) !important;
    }

    .action-btn.secondary {
        background: var(--bg-card) !important;
        border: 1px solid var(--border) !important;
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

    .format-grid {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
        gap: 0.5rem;
    }

    .format-item {
        display: flex;
        align-items: center;
        gap: 0.75rem;
        padding: 0.5rem 0.75rem;
        background: var(--bg-card);
        border-radius: var(--radius-sm);
    }

    .format-type {
        font-size: 0.85rem;
        color: var(--accent);
        min-width: 50px;
    }

    .format-item code {
        font-size: 0.8rem;
        color: var(--text-muted);
        background: transparent !important;
        padding: 0 !important;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    @media (max-width: 768px) {
        .main-content {
            grid-template-columns: 1fr;
        }
    }
</style>
