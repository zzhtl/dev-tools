<script lang="ts">
    let activeTab = $state("timestampToDate");
    let timestamp = $state("");
    let unit = $state("seconds");
    let result = $state("");
    let error = $state("");
    let copied = $state(false);
    
    // 自定义日期选择器状态
    let selectedYear = $state(new Date().getFullYear());
    let selectedMonth = $state(new Date().getMonth());
    let selectedDay = $state(new Date().getDate());
    let selectedHour = $state(new Date().getHours());
    let selectedMinute = $state(new Date().getMinutes());
    let selectedSecond = $state(new Date().getSeconds());
    let showCalendar = $state(false);

    const weekDays = ['日', '一', '二', '三', '四', '五', '六'];
    const months = ['1月', '2月', '3月', '4月', '5月', '6月', '7月', '8月', '9月', '10月', '11月', '12月'];

    // 快捷日期选项
    const quickDates = [
        { label: "今天", getValue: () => new Date() },
        { label: "明天", getValue: () => { const d = new Date(); d.setDate(d.getDate() + 1); return d; } },
        { label: "昨天", getValue: () => { const d = new Date(); d.setDate(d.getDate() - 1); return d; } },
        { label: "本周一", getValue: () => { const d = new Date(); d.setDate(d.getDate() - d.getDay() + 1); return d; } },
        { label: "本周日", getValue: () => { const d = new Date(); d.setDate(d.getDate() - d.getDay() + 7); return d; } },
        { label: "月初", getValue: () => new Date(new Date().getFullYear(), new Date().getMonth(), 1) },
        { label: "月末", getValue: () => new Date(new Date().getFullYear(), new Date().getMonth() + 1, 0) },
        { label: "年初", getValue: () => new Date(new Date().getFullYear(), 0, 1) },
        { label: "年末", getValue: () => new Date(new Date().getFullYear(), 11, 31) },
    ];

    // 获取某月的天数
    function getDaysInMonth(year: number, month: number): number {
        return new Date(year, month + 1, 0).getDate();
    }

    // 获取某月第一天是星期几
    function getFirstDayOfMonth(year: number, month: number): number {
        return new Date(year, month, 1).getDay();
    }

    // 生成日历网格数据
    function getCalendarDays(): (number | null)[] {
        const days: (number | null)[] = [];
        const firstDay = getFirstDayOfMonth(selectedYear, selectedMonth);
        const daysInMonth = getDaysInMonth(selectedYear, selectedMonth);
        
        // 填充月初空白
        for (let i = 0; i < firstDay; i++) {
            days.push(null);
        }
        // 填充日期
        for (let i = 1; i <= daysInMonth; i++) {
            days.push(i);
        }
        return days;
    }

    function prevMonth() {
        if (selectedMonth === 0) {
            selectedMonth = 11;
            selectedYear--;
        } else {
            selectedMonth--;
        }
    }

    function nextMonth() {
        if (selectedMonth === 11) {
            selectedMonth = 0;
            selectedYear++;
        } else {
            selectedMonth++;
        }
    }

    function prevYear() {
        selectedYear--;
    }

    function nextYear() {
        selectedYear++;
    }

    function selectDay(day: number) {
        selectedDay = day;
        convertDate();
    }

    function selectQuickDate(getValue: () => Date) {
        const date = getValue();
        selectedYear = date.getFullYear();
        selectedMonth = date.getMonth();
        selectedDay = date.getDate();
        convertDate();
    }

    function setNow() {
        const now = new Date();
        selectedYear = now.getFullYear();
        selectedMonth = now.getMonth();
        selectedDay = now.getDate();
        selectedHour = now.getHours();
        selectedMinute = now.getMinutes();
        selectedSecond = now.getSeconds();
        convertDate();
    }

    function setStartOfDay() {
        selectedHour = 0;
        selectedMinute = 0;
        selectedSecond = 0;
        convertDate();
    }

    function setEndOfDay() {
        selectedHour = 23;
        selectedMinute = 59;
        selectedSecond = 59;
        convertDate();
    }

    // 时间戳转日期
    function convertTimestamp() {
        error = "";
        if (!timestamp) {
            result = "";
            return;
        }

        try {
            const numericTimestamp = unit === "seconds" 
                ? parseInt(timestamp) * 1000 
                : parseInt(timestamp);

            if (isNaN(numericTimestamp)) {
                error = "无效的时间戳";
                return;
            }

            const date = new Date(numericTimestamp);
            
            if (date.toString() === "Invalid Date") {
                error = "无效的时间戳";
                return;
            }

            const year = date.getFullYear();
            const month = String(date.getMonth() + 1).padStart(2, "0");
            const day = String(date.getDate()).padStart(2, "0");
            const hours = String(date.getHours()).padStart(2, "0");
            const minutes = String(date.getMinutes()).padStart(2, "0");
            const seconds = String(date.getSeconds()).padStart(2, "0");
            const milliseconds = String(date.getMilliseconds()).padStart(3, "0");

            result = `${year}-${month}-${day} ${hours}:${minutes}:${seconds}.${milliseconds}`;
        } catch (e: any) {
            error = e.message;
        }
    }

    // 日期转时间戳
    function convertDate() {
        error = "";
        try {
            const date = new Date(
                selectedYear, 
                selectedMonth, 
                selectedDay, 
                selectedHour, 
                selectedMinute, 
                selectedSecond
            );

            if (isNaN(date.getTime())) {
                error = "无效的日期";
                return;
            }

            result = unit === "seconds"
                ? Math.floor(date.getTime() / 1000).toString()
                : date.getTime().toString();
        } catch (e: any) {
            error = e.message;
        }
    }

    function getCurrentTimestamp() {
        const now = Date.now();
        timestamp = unit === "seconds" ? Math.floor(now / 1000).toString() : now.toString();
        convertTimestamp();
    }

    async function copyResult() {
        if (!result || error) return;
        try {
            await navigator.clipboard.writeText(result);
            copied = true;
            setTimeout(() => copied = false, 1500);
        } catch (err) {
            console.error('复制失败:', err);
        }
    }

    function isToday(day: number): boolean {
        const today = new Date();
        return day === today.getDate() && 
               selectedMonth === today.getMonth() && 
               selectedYear === today.getFullYear();
    }

    function isSelected(day: number): boolean {
        return day === selectedDay;
    }

    // 格式化显示的日期
    function formatSelectedDate(): string {
        return `${selectedYear}-${String(selectedMonth + 1).padStart(2, '0')}-${String(selectedDay).padStart(2, '0')} ${String(selectedHour).padStart(2, '0')}:${String(selectedMinute).padStart(2, '0')}:${String(selectedSecond).padStart(2, '0')}`;
    }

    // 响应式处理
    $effect(() => {
        if (activeTab === "timestampToDate" && timestamp) {
            convertTimestamp();
        }
    });

    $effect(() => {
        if (activeTab === "dateToTimestamp") {
            convertDate();
        }
    });
</script>

<div class="time-tool">
    <div class="tab-selector">
        <button
            class:active={activeTab === "timestampToDate"}
            onclick={() => { activeTab = "timestampToDate"; result = ""; error = ""; }}
        >
            ⏱️ 时间戳 → 日期
        </button>
        <button
            class:active={activeTab === "dateToTimestamp"}
            onclick={() => { activeTab = "dateToTimestamp"; result = ""; error = ""; convertDate(); }}
        >
            📅 日期 → 时间戳
        </button>
    </div>

    <div class="converter-section">
        {#if activeTab === "timestampToDate"}
            <div class="input-area">
                <div class="input-header">
                    <span class="input-label">时间戳输入</span>
                    <button class="now-btn" onclick={getCurrentTimestamp}>
                        ⚡ 获取当前时间戳
                    </button>
                </div>
                <div class="input-row">
                    <input
                        type="text"
                        bind:value={timestamp}
                        placeholder="输入时间戳 (如: 1717027200)"
                    />
                    <select bind:value={unit} onchange={convertTimestamp}>
                        <option value="seconds">秒 (s)</option>
                        <option value="milliseconds">毫秒 (ms)</option>
                    </select>
                </div>
            </div>
        {:else}
            <div class="datetime-picker-container">
                <div class="picker-header">
                    <span class="input-label">选择日期时间</span>
                    <div class="quick-actions">
                        <button class="quick-btn" onclick={setNow}>现在</button>
                        <button class="quick-btn" onclick={setStartOfDay}>0:00:00</button>
                        <button class="quick-btn" onclick={setEndOfDay}>23:59:59</button>
                    </div>
                </div>

                <!-- 快捷日期选择 -->
                <div class="quick-dates">
                    {#each quickDates as quick}
                        <button class="quick-date-btn" onclick={() => selectQuickDate(quick.getValue)}>
                            {quick.label}
                        </button>
                    {/each}
                </div>

                <div class="picker-body">
                    <!-- 日历 -->
                    <div class="calendar">
                        <div class="calendar-header">
                            <button class="nav-btn" onclick={prevYear}>«</button>
                            <button class="nav-btn" onclick={prevMonth}>‹</button>
                            <span class="current-month">{selectedYear}年 {months[selectedMonth]}</span>
                            <button class="nav-btn" onclick={nextMonth}>›</button>
                            <button class="nav-btn" onclick={nextYear}>»</button>
                        </div>
                        
                        <div class="calendar-weekdays">
                            {#each weekDays as day}
                                <span class="weekday">{day}</span>
                            {/each}
                        </div>

                        <div class="calendar-days">
                            {#each getCalendarDays() as day}
                                {#if day}
                                    <button 
                                        class="day-btn"
                                        class:today={isToday(day)}
                                        class:selected={isSelected(day)}
                                        onclick={() => selectDay(day)}
                                    >
                                        {day}
                                    </button>
                                {:else}
                                    <span class="day-empty"></span>
                                {/if}
                            {/each}
                        </div>
                    </div>

                    <!-- 时间选择 -->
                    <div class="time-picker">
                        <div class="time-label">时间设置</div>
                        <div class="time-inputs">
                            <div class="time-input-group">
                                <label for="hour-input">时</label>
                                <input 
                                    id="hour-input"
                                    type="number" 
                                    min="0" 
                                    max="23" 
                                    bind:value={selectedHour}
                                    onchange={convertDate}
                                />
                            </div>
                            <span class="time-separator">:</span>
                            <div class="time-input-group">
                                <label for="minute-input">分</label>
                                <input 
                                    id="minute-input"
                                    type="number" 
                                    min="0" 
                                    max="59" 
                                    bind:value={selectedMinute}
                                    onchange={convertDate}
                                />
                            </div>
                            <span class="time-separator">:</span>
                            <div class="time-input-group">
                                <label for="second-input">秒</label>
                                <input 
                                    id="second-input"
                                    type="number" 
                                    min="0" 
                                    max="59" 
                                    bind:value={selectedSecond}
                                    onchange={convertDate}
                                />
                            </div>
                        </div>

                        <div class="selected-datetime">
                            <span class="datetime-label">选中的日期时间</span>
                            <code class="datetime-value">{formatSelectedDate()}</code>
                        </div>

                        <div class="unit-selector">
                            <span class="unit-label">输出单位：</span>
                            <div class="unit-options">
                                <button 
                                    class="unit-btn"
                                    class:active={unit === "seconds"}
                                    onclick={() => { unit = "seconds"; convertDate(); }}
                                >
                                    秒
                                </button>
                                <button 
                                    class="unit-btn"
                                    class:active={unit === "milliseconds"}
                                    onclick={() => { unit = "milliseconds"; convertDate(); }}
                                >
                                    毫秒
                                </button>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        {/if}

        <!-- 结果显示 -->
        <div class="result-area">
            <div class="result-header">
                <span>🔍 转换结果</span>
                {#if result && !error}
                    <button class="copy-btn" onclick={copyResult}>
                        {copied ? '✓ 已复制' : '📋 复制'}
                    </button>
                {/if}
            </div>
            <div class="result-display" class:error={!!error}>
                {#if error}
                    ⚠️ {error}
                {:else if result}
                    <code>{result}</code>
                {:else}
                    <span class="placeholder">🕒 等待输入...</span>
                {/if}
            </div>
        </div>
    </div>

    <!-- 快速参考 -->
    <div class="quick-reference">
        <h3>⚡ 快速参考</h3>
        <div class="reference-grid">
            <div class="reference-item">
                <span class="ref-label">当前时间戳 (秒)</span>
                <code>{Math.floor(Date.now() / 1000)}</code>
            </div>
            <div class="reference-item">
                <span class="ref-label">当前时间戳 (毫秒)</span>
                <code>{Date.now()}</code>
            </div>
            <div class="reference-item">
                <span class="ref-label">今天开始</span>
                <code>{Math.floor(new Date().setHours(0, 0, 0, 0) / 1000)}</code>
            </div>
            <div class="reference-item">
                <span class="ref-label">今天结束</span>
                <code>{Math.floor(new Date().setHours(23, 59, 59, 999) / 1000)}</code>
            </div>
        </div>
    </div>
</div>

<style>
    .time-tool {
        display: flex;
        flex-direction: column;
        gap: 1.5rem;
    }

    .tab-selector {
        display: flex;
        background: var(--bg-dark);
        border: 1px solid var(--border);
        border-radius: var(--radius-sm);
        overflow: hidden;
    }

    .tab-selector button {
        flex: 1;
        padding: 0.85rem 1.5rem !important;
        background: transparent !important;
        border: none !important;
        border-radius: 0 !important;
        color: var(--text-secondary) !important;
        font-weight: 500;
        transition: all 0.2s;
    }

    .tab-selector button:hover {
        background: var(--bg-hover) !important;
        transform: none !important;
        box-shadow: none !important;
    }

    .tab-selector button.active {
        background: var(--primary) !important;
        color: white !important;
    }

    .converter-section {
        display: flex;
        flex-direction: column;
        gap: 1.25rem;
        padding: 1.5rem;
        background: var(--bg-dark);
        border: 1px solid var(--border);
        border-radius: var(--radius-sm);
    }

    .input-area {
        display: flex;
        flex-direction: column;
        gap: 0.75rem;
    }

    .input-header, .picker-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
    }

    .input-label {
        font-weight: 500;
        color: var(--text-primary);
    }

    .now-btn {
        padding: 0.5rem 1rem !important;
        font-size: 0.85rem !important;
        background: linear-gradient(135deg, var(--accent-green) 0%, #059669 100%) !important;
    }

    .quick-actions {
        display: flex;
        gap: 0.5rem;
    }

    .quick-btn {
        padding: 0.4rem 0.75rem !important;
        font-size: 0.8rem !important;
        background: var(--bg-hover) !important;
        border: 1px solid var(--border) !important;
    }

    .input-row {
        display: flex;
        gap: 0.75rem;
    }

    .input-row input {
        flex: 1;
    }

    .input-row select {
        width: 140px;
        background: var(--bg-card);
        border: 1px solid var(--border);
        border-radius: var(--radius-sm);
        color: var(--text-primary);
        padding: 0.75rem 2.5rem 0.75rem 1rem;
        font-size: 0.9rem;
        cursor: pointer;
        appearance: none;
        -webkit-appearance: none;
        background-image: url("data:image/svg+xml;charset=UTF-8,%3csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='%239ca3af' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'%3e%3cpolyline points='6 9 12 15 18 9'%3e%3c/polyline%3e%3c/svg%3e");
        background-repeat: no-repeat;
        background-position: right 0.75rem center;
        background-size: 1em;
    }

    .input-row select:focus {
        outline: none;
        border-color: var(--primary);
    }

    .input-row select option {
        background: #1a1a2e;
        color: #e4e4e7;
        padding: 0.5rem;
    }

    /* 日期时间选择器 */
    .datetime-picker-container {
        display: flex;
        flex-direction: column;
        gap: 1rem;
    }

    .quick-dates {
        display: flex;
        flex-wrap: wrap;
        gap: 0.5rem;
    }

    .quick-date-btn {
        padding: 0.4rem 0.85rem !important;
        font-size: 0.85rem !important;
        background: var(--bg-card) !important;
        border: 1px solid var(--border) !important;
    }

    .quick-date-btn:hover {
        border-color: var(--primary) !important;
        transform: none !important;
        box-shadow: none !important;
    }

    .picker-body {
        display: grid;
        grid-template-columns: 1fr 1fr;
        gap: 1.5rem;
    }

    /* 日历样式 */
    .calendar {
        background: var(--bg-card);
        border: 1px solid var(--border);
        border-radius: var(--radius-sm);
        padding: 1rem;
    }

    .calendar-header {
        display: flex;
        align-items: center;
        justify-content: space-between;
        margin-bottom: 1rem;
    }

    .nav-btn {
        width: 32px !important;
        height: 32px !important;
        padding: 0 !important;
        background: var(--bg-dark) !important;
        border: 1px solid var(--border) !important;
        font-size: 1rem;
        display: flex;
        align-items: center;
        justify-content: center;
    }

    .nav-btn:hover {
        border-color: var(--primary) !important;
        transform: none !important;
        box-shadow: none !important;
    }

    .current-month {
        font-weight: 600;
        color: var(--text-primary);
    }

    .calendar-weekdays {
        display: grid;
        grid-template-columns: repeat(7, 1fr);
        gap: 2px;
        margin-bottom: 0.5rem;
    }

    .weekday {
        text-align: center;
        font-size: 0.8rem;
        color: var(--text-muted);
        padding: 0.25rem;
    }

    .calendar-days {
        display: grid;
        grid-template-columns: repeat(7, 1fr);
        gap: 2px;
    }

    .day-btn {
        aspect-ratio: 1;
        padding: 0 !important;
        background: transparent !important;
        border: 1px solid transparent !important;
        color: var(--text-primary) !important;
        font-size: 0.9rem;
        border-radius: 50% !important;
        transition: all 0.15s;
    }

    .day-btn:hover {
        background: var(--bg-hover) !important;
        transform: none !important;
        box-shadow: none !important;
    }

    .day-btn.today {
        border-color: var(--accent) !important;
        color: var(--accent) !important;
    }

    .day-btn.selected {
        background: var(--primary) !important;
        color: white !important;
        border-color: var(--primary) !important;
    }

    .day-empty {
        aspect-ratio: 1;
    }

    /* 时间选择器 */
    .time-picker {
        background: var(--bg-card);
        border: 1px solid var(--border);
        border-radius: var(--radius-sm);
        padding: 1rem;
        display: flex;
        flex-direction: column;
        gap: 1rem;
    }

    .time-label {
        font-weight: 500;
        color: var(--text-secondary);
        font-size: 0.9rem;
    }

    .time-inputs {
        display: flex;
        align-items: flex-end;
        gap: 0.5rem;
        justify-content: center;
    }

    .time-input-group {
        display: flex;
        flex-direction: column;
        gap: 0.25rem;
        align-items: center;
    }

    .time-input-group label {
        font-size: 0.75rem;
        color: var(--text-muted);
    }

    .time-input-group input {
        width: 60px !important;
        text-align: center;
        font-family: 'JetBrains Mono', monospace;
        font-size: 1.1rem !important;
        padding: 0.5rem !important;
    }

    .time-separator {
        font-size: 1.5rem;
        color: var(--text-muted);
        padding-bottom: 0.25rem;
    }

    .selected-datetime {
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
        padding: 0.75rem;
        background: var(--bg-dark);
        border-radius: var(--radius-sm);
    }

    .datetime-label {
        font-size: 0.8rem;
        color: var(--text-muted);
    }

    .datetime-value {
        font-size: 1rem;
        color: var(--accent) !important;
        background: transparent !important;
        padding: 0 !important;
    }

    .unit-selector {
        display: flex;
        align-items: center;
        gap: 0.75rem;
    }

    .unit-label {
        font-size: 0.85rem;
        color: var(--text-secondary);
    }

    .unit-options {
        display: flex;
        gap: 0.5rem;
    }

    .unit-btn {
        padding: 0.4rem 1rem !important;
        font-size: 0.85rem !important;
        background: var(--bg-dark) !important;
        border: 1px solid var(--border) !important;
    }

    .unit-btn.active {
        background: var(--primary) !important;
        border-color: var(--primary) !important;
        color: white !important;
    }

    .unit-btn:hover:not(.active) {
        border-color: var(--primary) !important;
        transform: none !important;
        box-shadow: none !important;
    }

    /* 结果区域 */
    .result-area {
        border: 1px solid var(--border);
        border-radius: var(--radius-sm);
        overflow: hidden;
    }

    .result-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 0.75rem 1rem;
        background: var(--bg-card);
        border-bottom: 1px solid var(--border);
        font-size: 0.9rem;
        color: var(--text-secondary);
    }

    .copy-btn {
        padding: 0.35rem 0.75rem !important;
        font-size: 0.8rem !important;
        background: var(--bg-hover) !important;
        border: 1px solid var(--border) !important;
    }

    .result-display {
        padding: 1.5rem;
        font-family: 'JetBrains Mono', monospace;
        font-size: 1.1rem;
        text-align: center;
        min-height: 60px;
        display: flex;
        align-items: center;
        justify-content: center;
    }

    .result-display code {
        background: transparent !important;
        padding: 0 !important;
        color: var(--accent);
        font-size: 1.5rem;
    }

    .result-display.error {
        color: #f87171;
        font-size: 0.95rem;
    }

    .placeholder {
        color: var(--text-muted);
    }

    /* 快速参考 */
    .quick-reference {
        padding: 1rem;
        background: var(--bg-dark);
        border: 1px solid var(--border);
        border-radius: var(--radius-sm);
    }

    .quick-reference h3 {
        font-size: 0.9rem;
        margin-bottom: 0.75rem;
        color: var(--text-secondary);
    }

    .reference-grid {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
        gap: 0.75rem;
    }

    .reference-item {
        display: flex;
        flex-direction: column;
        gap: 0.25rem;
        padding: 0.75rem;
        background: var(--bg-card);
        border-radius: var(--radius-sm);
    }

    .ref-label {
        font-size: 0.8rem;
        color: var(--text-muted);
    }

    .reference-item code {
        font-size: 0.9rem;
        color: var(--accent);
        background: transparent !important;
        padding: 0 !important;
    }

    @media (max-width: 768px) {
        .picker-body {
            grid-template-columns: 1fr;
        }
    }
</style>
