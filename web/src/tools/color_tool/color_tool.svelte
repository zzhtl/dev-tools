<script lang="ts">
    import CopyButton from "../../lib/ui/CopyButton.svelte";
    import Button from "../../lib/ui/Button.svelte";
    import Panel from "../../lib/ui/Panel.svelte";
    // 响应式状态
    let hexColor = $state("#4f46e5");
    let r = $state(79);
    let g = $state(70);
    let b = $state(229);
    let hue = $state(248);
    let saturation = $state(75);
    let lightness = $state(59);
    let alpha = $state(100);
    
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

<div class="tool">
  <div class="top">
    <Panel title="取色">
      {#snippet actions()}
        <Button size="sm" onclick={randomColor}>随机</Button>
      {/snippet}

      <div class="preview" style="background-color: {hexColor}">
        <span style="color: {getTextColor()}">{hexColor}</span>
      </div>

      <!-- 取色区：这里的颜色由用户选择决定，是本工具的数据本身，不使用主题 token -->
      <div
        class="area"
        style="background: linear-gradient(to top, #000, transparent), linear-gradient(to right, #fff, hsl({hue}, 100%, 50%))"
        onclick={selectColor}
        role="button"
        tabindex="0"
        aria-label="饱和度与明度取色区"
        onkeydown={(e) => e.key === "Enter" && selectColor(e as unknown as MouseEvent)}
      >
        <div class="pointer" style="left: {saturation}%; top: {100 - lightness}%;"></div>
      </div>

      <div
        class="hue"
        style="background: {hueGradient}"
        onclick={selectHue}
        role="slider"
        aria-label="色相"
        aria-valuemin={0}
        aria-valuemax={360}
        aria-valuenow={hue}
        tabindex="0"
        onkeydown={(e) => e.key === "Enter" && selectHue(e as unknown as MouseEvent)}
      >
        <div class="thumb" style="left: {(hue / 360) * 100}%"></div>
      </div>

      <label class="alpha">
        <span>透明度 {alpha}%</span>
        <input type="range" min="0" max="100" bind:value={alpha} />
      </label>
    </Panel>

    <Panel title="颜色值">
      <div class="values">
        <div class="row">
          <span class="tag">HEX</span>
          <input type="text" bind:value={hexColor} onchange={updateHex} aria-label="HEX" />
          <CopyButton value={() => hexColor} icon title="复制 HEX" />
        </div>

        <div class="row">
          <span class="tag">RGB</span>
          <div class="triple">
            <label><i>R</i><input type="number" min="0" max="255" bind:value={r} onchange={updateRgb} /></label>
            <label><i>G</i><input type="number" min="0" max="255" bind:value={g} onchange={updateRgb} /></label>
            <label><i>B</i><input type="number" min="0" max="255" bind:value={b} onchange={updateRgb} /></label>
          </div>
          <CopyButton value={() => getRgbaString()} icon title="复制 RGB" />
        </div>

        <div class="row">
          <span class="tag">HSL</span>
          <div class="triple">
            <label><i>H</i><input type="number" min="0" max="360" bind:value={hue} onchange={updateHSL} /></label>
            <label><i>S</i><input type="number" min="0" max="100" bind:value={saturation} onchange={updateHSL} /></label>
            <label><i>L</i><input type="number" min="0" max="100" bind:value={lightness} onchange={updateHSL} /></label>
          </div>
          <CopyButton value={() => getHslaString()} icon title="复制 HSL" />
        </div>
      </div>

      <div class="schemes">
        <div class="scheme">
          <span>互补色</span>
          <div class="swatches">
            <button class="swatch" style="background: {hexColor}" title={hexColor} aria-label="当前色"></button>
            <button
              class="swatch"
              style="background: {getComplementaryColor()}"
              title={getComplementaryColor()}
              aria-label="互补色"
              onclick={() => selectPresetColor(getComplementaryColor())}
            ></button>
          </div>
        </div>
        <div class="scheme">
          <span>类似色</span>
          <div class="swatches">
            {#each getAnalogousColors() as color (color)}
              <button class="swatch" style="background: {color}" title={color} aria-label={color} onclick={() => selectPresetColor(color)}></button>
            {/each}
          </div>
        </div>
        <div class="scheme">
          <span>三等分色</span>
          <div class="swatches">
            {#each getTriadicColors() as color (color)}
              <button class="swatch" style="background: {color}" title={color} aria-label={color} onclick={() => selectPresetColor(color)}></button>
            {/each}
          </div>
        </div>
      </div>
    </Panel>
  </div>

  <Panel title="对比度检查（WCAG）">
    <div class="contrast">
      <div class="cell">
        <div class="sample" style="background: {hexColor}; color: #fff;">白色文字</div>
        <div class="verdict">
          <code>{contrastWithWhite.toFixed(2)}:1</code>
          <span class:pass={contrastWithWhite >= 4.5} class:fail={contrastWithWhite < 4.5}>
            {contrastWithWhite >= 7 ? "AAA" : contrastWithWhite >= 4.5 ? "AA" : "不通过"}
          </span>
        </div>
      </div>
      <div class="cell">
        <div class="sample" style="background: {hexColor}; color: #000;">黑色文字</div>
        <div class="verdict">
          <code>{contrastWithBlack.toFixed(2)}:1</code>
          <span class:pass={contrastWithBlack >= 4.5} class:fail={contrastWithBlack < 4.5}>
            {contrastWithBlack >= 7 ? "AAA" : contrastWithBlack >= 4.5 ? "AA" : "不通过"}
          </span>
        </div>
      </div>
    </div>
    <p class="hint">正文需 ≥ 4.5:1（AA），≥ 7:1 为 AAA；大号文字（≥ 18.66px 粗体或 24px）门槛为 3:1。</p>
  </Panel>

  <Panel title="渐变生成">
    {#snippet actions()}
      <CopyButton value={() => getGradientCSS()} label="复制 CSS" />
    {/snippet}

    <div class="gradient" style="background: {getGradientCSS()}"></div>
    <div class="gradient-controls">
      <label class="pick"><input type="color" bind:value={gradientColor1} aria-label="渐变起始色" /><input type="text" bind:value={gradientColor1} aria-label="渐变起始色值" /></label>
      <label class="pick"><input type="color" bind:value={gradientColor2} aria-label="渐变结束色" /><input type="text" bind:value={gradientColor2} aria-label="渐变结束色值" /></label>
      <select bind:value={gradientType} aria-label="渐变类型">
        <option value="linear">线性渐变</option>
        <option value="radial">径向渐变</option>
      </select>
      {#if gradientType === "linear"}
        <label class="angle">
          <span>{gradientAngle}°</span>
          <input type="range" min="0" max="360" bind:value={gradientAngle} aria-label="渐变角度" />
        </label>
      {/if}
    </div>
    <code class="css">{getGradientCSS()}</code>
  </Panel>

  <Panel title="调色板">
    <div class="palettes">
      {#each Object.entries(presetPalettes) as [name, colors] (name)}
        <div class="palette">
          <span class="palette-name">{name}</span>
          <div class="swatches">
            {#each colors as color (color)}
              <button class="swatch" style="background: {color}" title={color} aria-label={color} onclick={() => selectPresetColor(color)}></button>
            {/each}
          </div>
        </div>
      {/each}
    </div>

    {#if colorHistory.length > 0}
      <div class="palette history">
        <span class="palette-name">最近使用</span>
        <div class="swatches">
          {#each colorHistory as color (color)}
            <button class="swatch" style="background: {color}" title={color} aria-label={color} onclick={() => selectPresetColor(color)}></button>
          {/each}
        </div>
      </div>
    {/if}
  </Panel>
</div>

<style>
  .tool {
    display: flex;
    flex-direction: column;
    gap: var(--sp-3);
    flex: 1;
    min-height: 0;
  }

  .top {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(320px, 1fr));
    gap: var(--sp-3);
  }

  .preview {
    height: 76px;
    border-radius: var(--radius-sm);
    display: flex;
    align-items: center;
    justify-content: center;
    font-family: var(--font-mono);
    font-size: var(--fs-lg);
    font-weight: var(--fw-semibold);
    border: 1px solid var(--color-border);
  }

  .area {
    position: relative;
    height: 150px;
    margin-top: var(--sp-3);
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border);
    cursor: crosshair;
  }

  .pointer {
    position: absolute;
    width: 12px;
    height: 12px;
    border: 2px solid #fff;
    border-radius: var(--radius-full);
    box-shadow: 0 0 0 1px rgba(0, 0, 0, 0.5);
    transform: translate(-50%, -50%);
    pointer-events: none;
  }

  .hue {
    position: relative;
    height: 14px;
    margin-top: var(--sp-3);
    border-radius: var(--radius-full);
    border: 1px solid var(--color-border);
    cursor: pointer;
  }

  .thumb {
    position: absolute;
    top: 50%;
    width: 14px;
    height: 14px;
    border: 2px solid #fff;
    border-radius: var(--radius-full);
    box-shadow: 0 0 0 1px rgba(0, 0, 0, 0.5);
    transform: translate(-50%, -50%);
    pointer-events: none;
  }

  .alpha {
    display: flex;
    align-items: center;
    gap: var(--sp-3);
    margin-top: var(--sp-3);
    font-size: var(--fs-sm);
    color: var(--color-text-secondary);
  }

  .alpha input {
    flex: 1;
    accent-color: var(--color-primary);
  }

  .values {
    display: flex;
    flex-direction: column;
    gap: var(--sp-2);
  }

  .row {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
  }

  .tag {
    width: 2.4rem;
    font-size: var(--fs-xs);
    font-weight: var(--fw-semibold);
    color: var(--color-text-muted);
  }

  .row > input[type="text"] {
    flex: 1;
  }

  .triple {
    flex: 1;
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: var(--sp-2);
  }

  .triple label {
    display: flex;
    align-items: center;
    gap: var(--sp-1);
  }

  .triple i {
    font-style: normal;
    font-size: var(--fs-xs);
    color: var(--color-text-muted);
  }

  input[type="text"],
  input[type="number"] {
    width: 100%;
    min-width: 0;
    background: var(--color-bg);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    color: var(--color-text);
    padding: 0.3rem var(--sp-2);
    font-family: var(--font-mono);
    font-size: var(--fs-sm);
  }

  input[type="text"]:focus,
  input[type="number"]:focus {
    outline: none;
    border-color: var(--color-primary);
  }

  .schemes {
    display: flex;
    flex-direction: column;
    gap: var(--sp-2);
    margin-top: var(--sp-4);
  }

  .scheme {
    display: flex;
    align-items: center;
    gap: var(--sp-3);
  }

  .scheme > span {
    width: 4.5rem;
    font-size: var(--fs-xs);
    color: var(--color-text-muted);
  }

  .swatches {
    display: flex;
    flex-wrap: wrap;
    gap: var(--sp-1);
  }

  .swatch {
    width: 26px;
    height: 26px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border);
    cursor: pointer;
    transition: transform var(--motion);
  }

  .swatch:hover {
    transform: scale(1.12);
  }

  .contrast {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: var(--sp-3);
  }

  .cell {
    display: flex;
    flex-direction: column;
    gap: var(--sp-2);
  }

  .sample {
    padding: var(--sp-4);
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border);
    text-align: center;
    font-size: var(--fs-md);
  }

  .verdict {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--sp-2);
    font-size: var(--fs-sm);
  }

  .verdict code {
    font-family: var(--font-mono);
    color: var(--color-text-secondary);
  }

  .verdict .pass {
    color: var(--color-success);
    font-weight: var(--fw-semibold);
  }

  .verdict .fail {
    color: var(--color-danger);
    font-weight: var(--fw-semibold);
  }

  .hint {
    margin-top: var(--sp-3);
    font-size: var(--fs-xs);
    color: var(--color-text-muted);
  }

  .gradient {
    height: 90px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border);
  }

  .gradient-controls {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: var(--sp-3);
    margin-top: var(--sp-3);
  }

  .pick {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
  }

  .pick input[type="color"] {
    width: 34px;
    height: 26px;
    padding: 0;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    background: none;
    cursor: pointer;
  }

  .pick input[type="text"] {
    width: 92px;
  }

  .gradient-controls select {
    background: var(--color-bg);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    color: var(--color-text);
    padding: 0.3rem var(--sp-2);
    font-size: var(--fs-sm);
  }

  .angle {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    font-size: var(--fs-sm);
    color: var(--color-text-secondary);
  }

  .angle input {
    accent-color: var(--color-primary);
  }

  .css {
    display: block;
    margin-top: var(--sp-3);
    padding: var(--sp-2);
    border-radius: var(--radius-sm);
    background: var(--color-code-bg);
    font-family: var(--font-mono);
    font-size: var(--fs-xs);
    color: var(--color-accent);
    word-break: break-all;
  }

  .palettes {
    display: flex;
    flex-direction: column;
    gap: var(--sp-3);
  }

  .palette {
    display: flex;
    align-items: center;
    gap: var(--sp-3);
  }

  .palette-name {
    width: 5.5rem;
    flex-shrink: 0;
    font-size: var(--fs-xs);
    color: var(--color-text-muted);
    text-transform: capitalize;
  }

  .history {
    margin-top: var(--sp-3);
    padding-top: var(--sp-3);
    border-top: 1px solid var(--color-border);
  }
</style>
