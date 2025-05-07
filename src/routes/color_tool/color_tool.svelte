<script lang="ts">
  // 定义响应式状态
  let hexColor = $state("#4f46e5"); // 默认为主题色
  let r = $state(79);
  let g = $state(70);
  let b = $state(229);
  let hue = $state(248); // 色相，0-360
  let saturation = $state(75); // 饱和度，0-100
  let lightness = $state(59); // 亮度，0-100

  // 初始化色相渐变背景
  const hueGradient = `linear-gradient(to right, 
    hsl(0, 100%, 50%), 
    hsl(60, 100%, 50%), 
    hsl(120, 100%, 50%), 
    hsl(180, 100%, 50%), 
    hsl(240, 100%, 50%), 
    hsl(300, 100%, 50%), 
    hsl(360, 100%, 50%))`;

  // 将 RGB 转换为 HEX
  function rgbToHex() {
    const rHex = r.toString(16).padStart(2, "0");
    const gHex = g.toString(16).padStart(2, "0");
    const bHex = b.toString(16).padStart(2, "0");
    hexColor = `#${rHex}${gHex}${bHex}`;
  }

  // 将 HEX 转换为 RGB
  function hexToRgb() {
    // 移除#前缀并确保是6位
    const hex = hexColor.replace("#", "");
    if (hex.length === 6) {
      r = parseInt(hex.substring(0, 2), 16);
      g = parseInt(hex.substring(2, 4), 16);
      b = parseInt(hex.substring(4, 6), 16);
      updateHSL();
    }
  }

  // RGB转HSL
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

  // HSL转RGB
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

  // 更新RGB时同步HEX和HSL
  function updateRgb() {
    // 验证RGB值在有效范围内
    r = Math.min(255, Math.max(0, r));
    g = Math.min(255, Math.max(0, g));
    b = Math.min(255, Math.max(0, b));
    rgbToHex();
    rgbToHsl();
  }

  // 更新HEX时同步RGB和HSL
  function updateHex() {
    // 简单验证HEX格式
    if (/^#[0-9A-Fa-f]{6}$/.test(hexColor)) {
      hexToRgb();
    }
  }

  // 更新HSL时同步RGB和HEX
  function updateHSL() {
    // 验证HSL值在有效范围内
    hue = Math.min(360, Math.max(0, hue));
    saturation = Math.min(100, Math.max(0, saturation));
    lightness = Math.min(100, Math.max(0, lightness));
    hslToRgb();
  }

  // 从色相选择器选择颜色
  function selectHue(event: MouseEvent) {
    const target = event.currentTarget as HTMLElement;
    const rect = target.getBoundingClientRect();
    const x = event.clientX - rect.left;
    hue = Math.round((x / rect.width) * 360);
    updateHSL();
  }

  // 从色彩面板选择颜色
  function selectColor(event: MouseEvent) {
    const target = event.currentTarget as HTMLElement;
    const rect = target.getBoundingClientRect();
    const x = event.clientX - rect.left;
    const y = event.clientY - rect.top;
    
    saturation = Math.round((x / rect.width) * 100);
    lightness = Math.round(100 - (y / rect.height) * 100);
    
    updateHSL();
  }

  // 复制颜色代码到剪贴板
  function copyToClipboard(text: string) {
    navigator.clipboard.writeText(text)
      .then(() => {
        alert("已复制到剪贴板");
      })
      .catch(err => {
        console.error("复制失败:", err);
      });
  }
</script>

<div class="color-tool">
  <h2>颜色工具</h2>
  
  <div class="tool-container">
    <div class="picker-container">
      <div class="color-preview" style="background-color: {hexColor}"></div>
      
      <div class="color-selector">
        <!-- 色彩面板 -->
        <div 
          class="color-area"
          style="background: linear-gradient(to right, #fff, hsl({hue}, 100%, 50%));
                 background-image: linear-gradient(to top, #000, transparent), 
                                 linear-gradient(to right, #fff, hsl({hue}, 100%, 50%))"
          onclick={selectColor}
        >
          <div 
            class="color-pointer"
            style="left: {saturation}%; top: {100 - lightness}%;"
          ></div>
        </div>
        
        <!-- 色相选择条 -->
        <div 
          class="hue-slider"
          style="background: {hueGradient}"
          onclick={selectHue}
        >
          <div 
            class="hue-slider-thumb"
            style="left: {hue / 360 * 100}%"
          ></div>
        </div>
      </div>
    </div>
    
    <div class="values-section">
      <div class="input-group">
        <label>HEX:</label>
        <div class="input-with-copy">
          <input 
            type="text" 
            bind:value={hexColor}
            onchange={updateHex}
            pattern="^#[0-9A-Fa-f]{6}$"
          />
          <button class="copy-btn" onclick={() => copyToClipboard(hexColor)}>复制</button>
        </div>
      </div>
      
      <div class="rgb-section">
        <div class="input-group">
          <label>RGB:</label>
          <div class="input-with-copy">
            <input 
              type="text" 
              readonly
              value={`rgb(${r}, ${g}, ${b})`}
            />
            <button class="copy-btn" onclick={() => copyToClipboard(`rgb(${r}, ${g}, ${b})`)}>复制</button>
          </div>
        </div>
        
        <div class="rgb-inputs">
          <div class="rgb-input">
            <label>R:</label>
            <input 
              type="number" 
              min="0" 
              max="255" 
              bind:value={r}
              onchange={updateRgb}
            />
            <div class="color-bar" style="background-color: rgb({r}, 0, 0)"></div>
          </div>
          
          <div class="rgb-input">
            <label>G:</label>
            <input 
              type="number" 
              min="0" 
              max="255" 
              bind:value={g}
              onchange={updateRgb}
            />
            <div class="color-bar" style="background-color: rgb(0, {g}, 0)"></div>
          </div>
          
          <div class="rgb-input">
            <label>B:</label>
            <input 
              type="number" 
              min="0" 
              max="255" 
              bind:value={b}
              onchange={updateRgb}
            />
            <div class="color-bar" style="background-color: rgb(0, 0, {b})"></div>
          </div>
        </div>
      </div>
      
      <div class="hsl-section">
        <div class="input-group">
          <label>HSL:</label>
          <div class="input-with-copy">
            <input 
              type="text" 
              readonly
              value={`hsl(${hue}, ${saturation}%, ${lightness}%)`}
            />
            <button class="copy-btn" onclick={() => copyToClipboard(`hsl(${hue}, ${saturation}%, ${lightness}%)`)}>复制</button>
          </div>
        </div>
      </div>
    </div>
  </div>
</div>

<style>
  .color-tool {
    width: 100%;
    max-width: 800px;
    margin: 0 auto;
  }

  h2 {
    margin-bottom: 1.5rem;
    color: var(--text-color);
  }

  .tool-container {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
    padding: 1.5rem;
    background: var(--card-bg);
    border-radius: var(--border-radius);
    box-shadow: var(--shadow);
  }

  .picker-container {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .color-preview {
    width: 100%;
    height: 80px;
    border-radius: var(--border-radius);
    border: 1px solid #e5e7eb;
  }

  .color-selector {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .color-area {
    position: relative;
    width: 100%;
    height: 200px;
    border-radius: var(--border-radius);
    border: 1px solid #e5e7eb;
    cursor: crosshair;
  }

  .color-pointer {
    position: absolute;
    width: 12px;
    height: 12px;
    border-radius: 50%;
    border: 2px solid white;
    box-shadow: 0 0 2px rgba(0, 0, 0, 0.5);
    transform: translate(-50%, -50%);
    pointer-events: none;
  }

  .hue-slider {
    position: relative;
    width: 100%;
    height: 20px;
    border-radius: 10px;
    cursor: pointer;
  }

  .hue-slider-thumb {
    position: absolute;
    top: 50%;
    width: 12px;
    height: 20px;
    background: white;
    border-radius: 3px;
    border: 1px solid #e5e7eb;
    transform: translate(-50%, -50%);
    pointer-events: none;
    box-shadow: 0 0 2px rgba(0, 0, 0, 0.5);
  }

  .values-section {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .input-group {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .input-group label {
    width: 50px;
    font-weight: 600;
  }

  .input-with-copy {
    flex: 1;
    display: flex;
    gap: 0.5rem;
  }

  .input-with-copy input {
    flex: 1;
    padding: 0.5rem;
    border: 1px solid #e5e7eb;
    border-radius: var(--border-radius);
    background-color: var(--card-bg);
    color: var(--text-color);
  }

  .copy-btn {
    padding: 0.5rem 1rem;
    background-color: var(--primary-light);
    color: white;
    border: none;
    border-radius: var(--border-radius);
    cursor: pointer;
    transition: background-color 0.3s;
  }

  .copy-btn:hover {
    background-color: var(--primary-color);
  }

  .rgb-section, .hsl-section {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    margin-top: 0.5rem;
  }

  .rgb-inputs {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .rgb-input {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .rgb-input label {
    width: 30px;
    text-align: right;
    font-weight: 600;
  }

  .rgb-input input {
    width: 70px;
    padding: 0.5rem;
    border: 1px solid #e5e7eb;
    border-radius: var(--border-radius);
    background-color: var(--card-bg);
    color: var(--text-color);
  }

  .color-bar {
    flex: 1;
    height: 20px;
    border-radius: 4px;
  }

  @media (min-width: 768px) {
    .picker-container {
      flex-direction: row;
    }

    .color-preview {
      width: 150px;
      height: 230px;
    }
    
    .color-selector {
      flex: 1;
    }

    .rgb-inputs {
      flex-direction: row;
      gap: 1rem;
    }

    .rgb-input {
      flex: 1;
    }
  }
</style> 