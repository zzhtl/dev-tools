<script lang="ts">
    // 响应式状态
    let hexColor = $state("#4f46e5");
    let r = $state(79);
    let g = $state(70);
    let b = $state(229);
    let hue = $state(248);
    let saturation = $state(75);
    let lightness = $state(59);
    let alpha = $state(100);
    let copied = $state<string | null>(null);
    
    // 颜色历史
    let colorHistory = $state<string[]>([]);
    const maxHistory = 12;
    
    // 渐变设置
    let gradientColor1 = $state("#4f46e5");
    let gradientColor2 = $state("#10b981");
    let gradientAngle = $state(90);
    let gradientType = $state<'linear' | 'radial'>('linear');

    // 预设调色板
    const presetPalettes = {
        material: [
            "#f44336", "#e91e63", "#9c27b0", "#673ab7", "#3f51b5", "#2196f3",
            "#03a9f4", "#00bcd4", "#009688", "#4caf50", "#8bc34a", "#cddc39",
            "#ffeb3b", "#ffc107", "#ff9800", "#ff5722", "#795548", "#9e9e9e"
        ],
        tailwind: [
            "#ef4444", "#f97316", "#f59e0b", "#eab308", "#84cc16", "#22c55e",
            "#10b981", "#14b8a6", "#06b6d4", "#0ea5e9", "#3b82f6", "#6366f1",
            "#8b5cf6", "#a855f7", "#d946ef", "#ec4899", "#f43f5e", "#64748b"
        ],
        pastel: [
            "#fecaca", "#fed7aa", "#fef08a", "#d9f99d", "#bbf7d0", "#a7f3d0",
            "#99f6e4", "#a5f3fc", "#bae6fd", "#bfdbfe", "#c7d2fe", "#ddd6fe",
            "#e9d5ff", "#f5d0fe", "#fbcfe8", "#fecdd3", "#e5e7eb", "#f3f4f6"
        ],
        neutral: [
            "#0a0a0a", "#171717", "#262626", "#404040", "#525252", "#737373",
            "#a3a3a3", "#d4d4d4", "#e5e5e5", "#f5f5f5", "#fafafa", "#ffffff"
        ]
    };

    // 色相渐变
    const hueGradient = `linear-gradient(to right, 
        hsl(0, 100%, 50%), hsl(60, 100%, 50%), hsl(120, 100%, 50%), 
        hsl(180, 100%, 50%), hsl(240, 100%, 50%), hsl(300, 100%, 50%), hsl(360, 100%, 50%))`;

    // 颜色转换函数
    function rgbToHex() {
        const rHex = r.toString(16).padStart(2, "0");
        const gHex = g.toString(16).padStart(2, "0");
        const bHex = b.toString(16).padStart(2, "0");
        hexColor = `#${rHex}${gHex}${bHex}`;
    }

    function hexToRgb() {
        const hex = hexColor.replace("#", "");
        if (hex.length === 6) {
            r = parseInt(hex.substring(0, 2), 16);
            g = parseInt(hex.substring(2, 4), 16);
            b = parseInt(hex.substring(4, 6), 16);
            rgbToHsl();
        } else if (hex.length === 3) {
            r = parseInt(hex[0] + hex[0], 16);
            g = parseInt(hex[1] + hex[1], 16);
            b = parseInt(hex[2] + hex[2], 16);
            rgbToHex();
            rgbToHsl();
        }
    }

    function rgbToHsl() {
        const r1 = r / 255;
        const g1 = g / 255;
        const b1 = b / 255;
        
        const max = Math.max(r1, g1, b1);
        const min = Math.min(r1, g1, b1);
        let h = 0, s = 0, l = (max + min) / 2;

        if (max !== min) {
            const d = max - min;
            s = l > 0.5 ? d / (2 - max - min) : d / (max + min);
            
            switch (max) {
                case r1: h = (g1 - b1) / d + (g1 < b1 ? 6 : 0); break;
                case g1: h = (b1 - r1) / d + 2; break;
                case b1: h = (r1 - g1) / d + 4; break;
            }
            h /= 6;
        }

        hue = Math.round(h * 360);
        saturation = Math.round(s * 100);
        lightness = Math.round(l * 100);
    }

    function hslToRgb() {
        const h = hue / 360;
        const s = saturation / 100;
        const l = lightness / 100;
        
        let r1, g1, b1;

        if (s === 0) {
            r1 = g1 = b1 = l;
        } else {
            const hue2rgb = (p: number, q: number, t: number) => {
                if (t < 0) t += 1;
                if (t > 1) t -= 1;
                if (t < 1/6) return p + (q - p) * 6 * t;
                if (t < 1/2) return q;
                if (t < 2/3) return p + (q - p) * (2/3 - t) * 6;
                return p;
            };

            const q = l < 0.5 ? l * (1 + s) : l + s - l * s;
            const p = 2 * l - q;
            
            r1 = hue2rgb(p, q, h + 1/3);
            g1 = hue2rgb(p, q, h);
            b1 = hue2rgb(p, q, h - 1/3);
        }

        r = Math.round(r1 * 255);
        g = Math.round(g1 * 255);
        b = Math.round(b1 * 255);
        
        rgbToHex();
    }

    // 更新函数
    function updateRgb() {
        r = Math.min(255, Math.max(0, r));
        g = Math.min(255, Math.max(0, g));
        b = Math.min(255, Math.max(0, b));
        rgbToHex();
        rgbToHsl();
    }

    function updateHex() {
        if (/^#[0-9A-Fa-f]{3,6}$/.test(hexColor)) {
            hexToRgb();
        }
    }

    function updateHSL() {
        hue = Math.min(360, Math.max(0, hue));
        saturation = Math.min(100, Math.max(0, saturation));
        lightness = Math.min(100, Math.max(0, lightness));
        hslToRgb();
    }

    // 色彩面板交互
    function selectColor(event: MouseEvent) {
        const target = event.currentTarget as HTMLElement;
        const rect = target.getBoundingClientRect();
        const x = event.clientX - rect.left;
        const y = event.clientY - rect.top;
        
        saturation = Math.round((x / rect.width) * 100);
        lightness = Math.round(100 - (y / rect.height) * 100);
        
        updateHSL();
    }

    function selectHue(event: MouseEvent) {
        const target = event.currentTarget as HTMLElement;
        const rect = target.getBoundingClientRect();
        const x = event.clientX - rect.left;
        hue = Math.round((x / rect.width) * 360);
        updateHSL();
    }

    // 选择预设颜色
    function selectPresetColor(color: string) {
        hexColor = color;
        hexToRgb();
        addToHistory(color);
    }

    // 添加到历史
    function addToHistory(color: string) {
        if (!colorHistory.includes(color)) {
            colorHistory = [color, ...colorHistory.slice(0, maxHistory - 1)];
        }
    }

    // 复制到剪贴板
    async function copyToClipboard(text: string, type: string) {
        try {
            await navigator.clipboard.writeText(text);
            copied = type;
            addToHistory(hexColor);
            setTimeout(() => copied = null, 1500);
        } catch (err) {
            console.error("复制失败:", err);
        }
    }

    // 随机颜色
    function randomColor() {
        r = Math.floor(Math.random() * 256);
        g = Math.floor(Math.random() * 256);
        b = Math.floor(Math.random() * 256);
        updateRgb();
        addToHistory(hexColor);
    }

    // 计算对比度
    function getLuminance(r: number, g: number, b: number): number {
        const [rs, gs, bs] = [r, g, b].map(c => {
            c = c / 255;
            return c <= 0.03928 ? c / 12.92 : Math.pow((c + 0.055) / 1.055, 2.4);
        });
        return 0.2126 * rs + 0.7152 * gs + 0.0722 * bs;
    }

    function getContrastRatio(color1: {r: number, g: number, b: number}, color2: {r: number, g: number, b: number}): number {
        const l1 = getLuminance(color1.r, color1.g, color1.b);
        const l2 = getLuminance(color2.r, color2.g, color2.b);
        const lighter = Math.max(l1, l2);
        const darker = Math.min(l1, l2);
        return (lighter + 0.05) / (darker + 0.05);
    }

    // 文本颜色建议
    function getTextColor(): string {
        const luminance = getLuminance(r, g, b);
        return luminance > 0.179 ? '#000000' : '#ffffff';
    }

    // 互补色
    function getComplementaryColor(): string {
        const compHue = (hue + 180) % 360;
        return `hsl(${compHue}, ${saturation}%, ${lightness}%)`;
    }

    // 类似色
    function getAnalogousColors(): string[] {
        return [
            `hsl(${(hue - 30 + 360) % 360}, ${saturation}%, ${lightness}%)`,
            hexColor,
            `hsl(${(hue + 30) % 360}, ${saturation}%, ${lightness}%)`
        ];
    }

    // 三等分色
    function getTriadicColors(): string[] {
        return [
            hexColor,
            `hsl(${(hue + 120) % 360}, ${saturation}%, ${lightness}%)`,
            `hsl(${(hue + 240) % 360}, ${saturation}%, ${lightness}%)`
        ];
    }

    // 生成渐变CSS
    function getGradientCSS(): string {
        if (gradientType === 'linear') {
            return `linear-gradient(${gradientAngle}deg, ${gradientColor1}, ${gradientColor2})`;
        }
        return `radial-gradient(circle, ${gradientColor1}, ${gradientColor2})`;
    }

    // 获取格式化的颜色值
    function getRgbaString(): string {
        return alpha < 100 ? `rgba(${r}, ${g}, ${b}, ${alpha/100})` : `rgb(${r}, ${g}, ${b})`;
    }

    function getHslaString(): string {
        return alpha < 100 ? `hsla(${hue}, ${saturation}%, ${lightness}%, ${alpha/100})` : `hsl(${hue}, ${saturation}%, ${lightness}%)`;
    }

    // 对比度检查
    const contrastWithWhite = $derived(getContrastRatio({r, g, b}, {r: 255, g: 255, b: 255}));
    const contrastWithBlack = $derived(getContrastRatio({r, g, b}, {r: 0, g: 0, b: 0}));
</script>

<div class="color-tool">
    <div class="main-section">
        <!-- 颜色选择器 -->
        <div class="picker-section">
            <div class="color-preview-area">
                <div class="color-preview" style="background-color: {hexColor}">
                    <span class="preview-text" style="color: {getTextColor()}">{hexColor}</span>
                </div>
                <button class="random-btn" onclick={randomColor}>
                    🎲 随机颜色
                </button>
            </div>

            <div class="color-selector">
                <div 
                    class="color-area"
                    style="background: linear-gradient(to top, #000, transparent), 
                           linear-gradient(to right, #fff, hsl({hue}, 100%, 50%))"
                    onclick={selectColor}
                    role="button"
                    tabindex="0"
                    onkeydown={(e) => e.key === 'Enter' && selectColor(e as unknown as MouseEvent)}
                >
                    <div 
                        class="color-pointer"
                        style="left: {saturation}%; top: {100 - lightness}%;"
                    ></div>
                </div>
                
                <div 
                    class="hue-slider"
                    style="background: {hueGradient}"
                    onclick={selectHue}
                    role="slider"
                    tabindex="0"
                    aria-valuenow={hue}
                    onkeydown={(e) => e.key === 'Enter' && selectHue(e as unknown as MouseEvent)}
                >
                    <div 
                        class="hue-thumb"
                        style="left: {hue / 360 * 100}%"
                    ></div>
                </div>

                <!-- 透明度滑块 -->
                <div class="alpha-slider-container">
                    <span class="slider-label">透明度: {alpha}%</span>
                    <input 
                        type="range"
                        class="alpha-slider"
                        min="0"
                        max="100"
                        bind:value={alpha}
                    />
                </div>
            </div>
        </div>

        <!-- 颜色值输入 -->
        <div class="values-section">
            <div class="value-group">
                <label for="hex-input">HEX</label>
                <div class="input-row">
                    <input 
                        id="hex-input"
                        type="text" 
                        bind:value={hexColor}
                        onchange={updateHex}
                    />
                    <button 
                        class="copy-btn"
                        class:copied={copied === 'hex'}
                        onclick={() => copyToClipboard(hexColor, 'hex')}
                    >
                        {copied === 'hex' ? '✓' : '📋'}
                    </button>
                </div>
            </div>

            <div class="value-group">
                <label>RGB</label>
                <div class="rgb-inputs">
                    <div class="rgb-input">
                        <span>R</span>
                        <input 
                            type="number" 
                            min="0" max="255" 
                            bind:value={r}
                            onchange={updateRgb}
                        />
                    </div>
                    <div class="rgb-input">
                        <span>G</span>
                        <input 
                            type="number" 
                            min="0" max="255" 
                            bind:value={g}
                            onchange={updateRgb}
                        />
                    </div>
                    <div class="rgb-input">
                        <span>B</span>
                        <input 
                            type="number" 
                            min="0" max="255" 
                            bind:value={b}
                            onchange={updateRgb}
                        />
                    </div>
                    <button 
                        class="copy-btn"
                        class:copied={copied === 'rgb'}
                        onclick={() => copyToClipboard(getRgbaString(), 'rgb')}
                    >
                        {copied === 'rgb' ? '✓' : '📋'}
                    </button>
                </div>
            </div>

            <div class="value-group">
                <label>HSL</label>
                <div class="hsl-inputs">
                    <div class="hsl-input">
                        <span>H</span>
                        <input 
                            type="number" 
                            min="0" max="360" 
                            bind:value={hue}
                            onchange={updateHSL}
                        />
                    </div>
                    <div class="hsl-input">
                        <span>S</span>
                        <input 
                            type="number" 
                            min="0" max="100" 
                            bind:value={saturation}
                            onchange={updateHSL}
                        />
                    </div>
                    <div class="hsl-input">
                        <span>L</span>
                        <input 
                            type="number" 
                            min="0" max="100" 
                            bind:value={lightness}
                            onchange={updateHSL}
                        />
                    </div>
                    <button 
                        class="copy-btn"
                        class:copied={copied === 'hsl'}
                        onclick={() => copyToClipboard(getHslaString(), 'hsl')}
                    >
                        {copied === 'hsl' ? '✓' : '📋'}
                    </button>
                </div>
            </div>
        </div>
    </div>

    <!-- 配色方案 -->
    <div class="schemes-section">
        <h3>🎨 配色方案</h3>
        <div class="schemes-grid">
            <div class="scheme-item">
                <span class="scheme-label">互补色</span>
                <div class="scheme-colors">
                    <div class="scheme-color" style="background: {hexColor}"></div>
                    <div class="scheme-color" style="background: {getComplementaryColor()}"></div>
                </div>
            </div>
            <div class="scheme-item">
                <span class="scheme-label">类似色</span>
                <div class="scheme-colors">
                    {#each getAnalogousColors() as color}
                        <div class="scheme-color" style="background: {color}"></div>
                    {/each}
                </div>
            </div>
            <div class="scheme-item">
                <span class="scheme-label">三等分色</span>
                <div class="scheme-colors">
                    {#each getTriadicColors() as color}
                        <div class="scheme-color" style="background: {color}"></div>
                    {/each}
                </div>
            </div>
        </div>
    </div>

    <!-- 对比度检查 -->
    <div class="contrast-section">
        <h3>📊 对比度检查 (WCAG)</h3>
        <div class="contrast-grid">
            <div class="contrast-item">
                <div class="contrast-preview" style="background: {hexColor}; color: white;">
                    白色文字
                </div>
                <div class="contrast-info">
                    <span class="contrast-ratio">{contrastWithWhite.toFixed(2)}:1</span>
                    <span class="contrast-level" class:pass={contrastWithWhite >= 4.5} class:fail={contrastWithWhite < 4.5}>
                        {contrastWithWhite >= 7 ? 'AAA' : contrastWithWhite >= 4.5 ? 'AA' : '不通过'}
                    </span>
                </div>
            </div>
            <div class="contrast-item">
                <div class="contrast-preview" style="background: {hexColor}; color: black;">
                    黑色文字
                </div>
                <div class="contrast-info">
                    <span class="contrast-ratio">{contrastWithBlack.toFixed(2)}:1</span>
                    <span class="contrast-level" class:pass={contrastWithBlack >= 4.5} class:fail={contrastWithBlack < 4.5}>
                        {contrastWithBlack >= 7 ? 'AAA' : contrastWithBlack >= 4.5 ? 'AA' : '不通过'}
                    </span>
                </div>
            </div>
        </div>
    </div>

    <!-- 渐变生成器 -->
    <div class="gradient-section">
        <h3>🌈 渐变生成器</h3>
        <div class="gradient-preview" style="background: {getGradientCSS()}"></div>
        <div class="gradient-controls">
            <div class="gradient-colors">
                <div class="gradient-color-input">
                    <input type="color" bind:value={gradientColor1} />
                    <input type="text" bind:value={gradientColor1} />
                </div>
                <div class="gradient-color-input">
                    <input type="color" bind:value={gradientColor2} />
                    <input type="text" bind:value={gradientColor2} />
                </div>
            </div>
            <div class="gradient-options">
                <select bind:value={gradientType}>
                    <option value="linear">线性渐变</option>
                    <option value="radial">径向渐变</option>
                </select>
                {#if gradientType === 'linear'}
                    <div class="angle-input">
                        <label for="gradient-angle">角度: {gradientAngle}°</label>
                        <input 
                            id="gradient-angle"
                            type="range" 
                            min="0" max="360" 
                            bind:value={gradientAngle}
                        />
                    </div>
                {/if}
            </div>
            <button 
                class="copy-gradient-btn"
                class:copied={copied === 'gradient'}
                onclick={() => copyToClipboard(getGradientCSS(), 'gradient')}
            >
                {copied === 'gradient' ? '✓ 已复制' : '📋 复制CSS'}
            </button>
        </div>
    </div>

    <!-- 预设调色板 -->
    <div class="palettes-section">
        <h3>🎯 预设调色板</h3>
        <div class="palette-tabs">
            {#each Object.keys(presetPalettes) as palette}
                <button class="palette-tab" onclick={() => {}}>
                    {palette === 'material' ? 'Material' : 
                     palette === 'tailwind' ? 'Tailwind' : 
                     palette === 'pastel' ? 'Pastel' : 'Neutral'}
                </button>
            {/each}
        </div>
        <div class="palettes-grid">
            {#each Object.entries(presetPalettes) as [name, colors]}
                <div class="palette-group">
                    <span class="palette-name">{name}</span>
                    <div class="palette-colors">
                        {#each colors as color}
                            <button 
                                class="palette-color"
                                style="background: {color}"
                                onclick={() => selectPresetColor(color)}
                                title={color}
                            ></button>
                        {/each}
                    </div>
                </div>
            {/each}
        </div>
    </div>

    <!-- 颜色历史 -->
    {#if colorHistory.length > 0}
        <div class="history-section">
            <h3>📜 最近使用</h3>
            <div class="history-colors">
                {#each colorHistory as color}
                    <button 
                        class="history-color"
                        style="background: {color}"
                        onclick={() => selectPresetColor(color)}
                        title={color}
                    ></button>
                {/each}
            </div>
        </div>
    {/if}
</div>

<style>
    .color-tool {
        display: flex;
        flex-direction: column;
        gap: 1.5rem;
    }

    .main-section {
        display: grid;
        grid-template-columns: 1fr 1fr;
        gap: 1.5rem;
    }

    @media (max-width: 768px) {
        .main-section {
            grid-template-columns: 1fr;
        }
    }

    /* 颜色选择器区域 */
    .picker-section {
        display: flex;
        flex-direction: column;
        gap: 1rem;
    }

    .color-preview-area {
        display: flex;
        gap: 0.75rem;
    }

    .color-preview {
        flex: 1;
        height: 80px;
        border-radius: var(--radius-sm);
        border: 1px solid var(--border);
        display: flex;
        align-items: center;
        justify-content: center;
        transition: background-color 0.2s;
    }

    .preview-text {
        font-family: 'JetBrains Mono', monospace;
        font-size: 1.1rem;
        font-weight: 600;
    }

    .random-btn {
        padding: 0.75rem 1rem !important;
        background: var(--bg-dark) !important;
        border: 1px solid var(--border) !important;
        white-space: nowrap;
    }

    .color-selector {
        display: flex;
        flex-direction: column;
        gap: 0.75rem;
    }

    .color-area {
        position: relative;
        width: 100%;
        height: 180px;
        border-radius: var(--radius-sm);
        border: 1px solid var(--border);
        cursor: crosshair;
    }

    .color-pointer {
        position: absolute;
        width: 16px;
        height: 16px;
        border-radius: 50%;
        border: 2px solid white;
        box-shadow: 0 0 4px rgba(0, 0, 0, 0.5);
        transform: translate(-50%, -50%);
        pointer-events: none;
    }

    .hue-slider {
        position: relative;
        width: 100%;
        height: 16px;
        border-radius: 8px;
        cursor: pointer;
        border: 1px solid var(--border);
    }

    .hue-thumb {
        position: absolute;
        top: 50%;
        width: 10px;
        height: 20px;
        background: white;
        border-radius: 3px;
        border: 1px solid var(--border);
        transform: translate(-50%, -50%);
        pointer-events: none;
        box-shadow: 0 1px 3px rgba(0, 0, 0, 0.3);
    }

    .alpha-slider-container {
        display: flex;
        align-items: center;
        gap: 0.75rem;
    }

    .slider-label {
        font-size: 0.85rem;
        color: var(--text-secondary);
        min-width: 80px;
    }

    .alpha-slider {
        flex: 1;
        height: 8px;
        -webkit-appearance: none;
        appearance: none;
        background: linear-gradient(to right, transparent, var(--text-primary));
        border-radius: 4px;
        outline: none;
    }

    .alpha-slider::-webkit-slider-thumb {
        -webkit-appearance: none;
        appearance: none;
        width: 16px;
        height: 16px;
        background: white;
        border: 1px solid var(--border);
        border-radius: 50%;
        cursor: pointer;
        box-shadow: 0 1px 3px rgba(0, 0, 0, 0.3);
    }

    /* 数值输入区域 */
    .values-section {
        display: flex;
        flex-direction: column;
        gap: 1rem;
        background: var(--bg-dark);
        border: 1px solid var(--border);
        border-radius: var(--radius-sm);
        padding: 1rem;
    }

    .value-group {
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
    }

    .value-group label {
        font-size: 0.85rem;
        font-weight: 500;
        color: var(--text-secondary);
    }

    .input-row {
        display: flex;
        gap: 0.5rem;
    }

    .input-row input {
        flex: 1;
        font-family: 'JetBrains Mono', monospace;
    }

    .rgb-inputs, .hsl-inputs {
        display: flex;
        gap: 0.5rem;
    }

    .rgb-input, .hsl-input {
        display: flex;
        flex-direction: column;
        gap: 0.25rem;
        flex: 1;
    }

    .rgb-input span, .hsl-input span {
        font-size: 0.75rem;
        color: var(--text-muted);
        text-align: center;
    }

    .rgb-input input, .hsl-input input {
        width: 100%;
        text-align: center;
        font-family: 'JetBrains Mono', monospace;
        padding: 0.5rem !important;
    }

    .copy-btn {
        padding: 0.5rem 0.75rem !important;
        background: var(--bg-card) !important;
        border: 1px solid var(--border) !important;
        min-width: 40px;
    }

    .copy-btn.copied {
        background: var(--accent-green) !important;
        border-color: var(--accent-green) !important;
        color: white !important;
    }

    /* 配色方案 */
    .schemes-section, .contrast-section, .gradient-section, .palettes-section, .history-section {
        background: var(--bg-dark);
        border: 1px solid var(--border);
        border-radius: var(--radius-sm);
        padding: 1rem;
    }

    .schemes-section h3, .contrast-section h3, .gradient-section h3, .palettes-section h3, .history-section h3 {
        font-size: 0.95rem;
        margin-bottom: 0.75rem;
        color: var(--text-secondary);
    }

    .schemes-grid {
        display: grid;
        grid-template-columns: repeat(3, 1fr);
        gap: 1rem;
    }

    @media (max-width: 640px) {
        .schemes-grid {
            grid-template-columns: 1fr;
        }
    }

    .scheme-item {
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
    }

    .scheme-label {
        font-size: 0.8rem;
        color: var(--text-muted);
    }

    .scheme-colors {
        display: flex;
        border-radius: var(--radius-sm);
        overflow: hidden;
    }

    .scheme-color {
        flex: 1;
        height: 40px;
    }

    /* 对比度检查 */
    .contrast-grid {
        display: grid;
        grid-template-columns: 1fr 1fr;
        gap: 1rem;
    }

    @media (max-width: 480px) {
        .contrast-grid {
            grid-template-columns: 1fr;
        }
    }

    .contrast-item {
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
    }

    .contrast-preview {
        padding: 1rem;
        text-align: center;
        border-radius: var(--radius-sm);
        font-weight: 500;
    }

    .contrast-info {
        display: flex;
        justify-content: space-between;
        align-items: center;
    }

    .contrast-ratio {
        font-family: 'JetBrains Mono', monospace;
        font-size: 0.9rem;
    }

    .contrast-level {
        padding: 0.2rem 0.5rem;
        border-radius: 4px;
        font-size: 0.75rem;
        font-weight: 600;
    }

    .contrast-level.pass {
        background: rgba(16, 185, 129, 0.2);
        color: var(--accent-green);
    }

    .contrast-level.fail {
        background: rgba(239, 68, 68, 0.2);
        color: #ef4444;
    }

    /* 渐变生成器 */
    .gradient-preview {
        height: 80px;
        border-radius: var(--radius-sm);
        border: 1px solid var(--border);
        margin-bottom: 1rem;
    }

    .gradient-controls {
        display: flex;
        flex-direction: column;
        gap: 0.75rem;
    }

    .gradient-colors {
        display: flex;
        gap: 1rem;
    }

    .gradient-color-input {
        flex: 1;
        display: flex;
        gap: 0.5rem;
    }

    .gradient-color-input input[type="color"] {
        width: 40px;
        height: 36px;
        padding: 0;
        border: 1px solid var(--border);
        border-radius: var(--radius-sm);
        cursor: pointer;
    }

    .gradient-color-input input[type="text"] {
        flex: 1;
        font-family: 'JetBrains Mono', monospace;
    }

    .gradient-options {
        display: flex;
        gap: 1rem;
        align-items: center;
    }

    .gradient-options select {
        padding: 0.6rem 2.5rem 0.6rem 1rem;
        border: 1px solid var(--border);
        border-radius: var(--radius-sm);
        background: var(--bg-card);
        color: var(--text-primary);
        font-size: 0.9rem;
        cursor: pointer;
        appearance: none;
        -webkit-appearance: none;
        background-image: url("data:image/svg+xml;charset=UTF-8,%3csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='%239ca3af' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'%3e%3cpolyline points='6 9 12 15 18 9'%3e%3c/polyline%3e%3c/svg%3e");
        background-repeat: no-repeat;
        background-position: right 0.75rem center;
        background-size: 1em;
    }

    .gradient-options select:focus {
        outline: none;
        border-color: var(--primary);
        box-shadow: 0 0 0 3px rgba(99, 102, 241, 0.2);
    }

    .gradient-options select option {
        background: #1a1a2e;
        color: #e4e4e7;
        padding: 0.5rem;
    }

    .angle-input {
        flex: 1;
        display: flex;
        flex-direction: column;
        gap: 0.25rem;
    }

    .angle-input label {
        font-size: 0.8rem;
        color: var(--text-muted);
    }

    .angle-input input {
        width: 100%;
    }

    .copy-gradient-btn {
        padding: 0.5rem 1rem !important;
        background: var(--primary) !important;
    }

    .copy-gradient-btn.copied {
        background: var(--accent-green) !important;
    }

    /* 预设调色板 */
    .palette-tabs {
        display: flex;
        gap: 0.5rem;
        margin-bottom: 1rem;
        flex-wrap: wrap;
    }

    .palette-tab {
        padding: 0.4rem 0.75rem !important;
        background: var(--bg-card) !important;
        border: 1px solid var(--border) !important;
        font-size: 0.85rem;
    }

    .palettes-grid {
        display: flex;
        flex-direction: column;
        gap: 1rem;
    }

    .palette-group {
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
    }

    .palette-name {
        font-size: 0.8rem;
        color: var(--text-muted);
        text-transform: capitalize;
    }

    .palette-colors {
        display: flex;
        flex-wrap: wrap;
        gap: 4px;
    }

    .palette-color {
        width: 28px;
        height: 28px;
        border-radius: 4px;
        border: 1px solid var(--border) !important;
        padding: 0 !important;
        cursor: pointer;
        transition: transform 0.15s;
    }

    .palette-color:hover {
        transform: scale(1.2);
        z-index: 1;
        box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3) !important;
    }

    /* 颜色历史 */
    .history-colors {
        display: flex;
        flex-wrap: wrap;
        gap: 6px;
    }

    .history-color {
        width: 32px;
        height: 32px;
        border-radius: var(--radius-sm);
        border: 1px solid var(--border) !important;
        padding: 0 !important;
        cursor: pointer;
        transition: transform 0.15s;
    }

    .history-color:hover {
        transform: scale(1.15);
        box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3) !important;
    }
</style>
