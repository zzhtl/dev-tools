<script lang="ts">
    import cronParser from "cron-parser";
    import { format } from "date-fns";

    let cronExpression = $state("* * * * *");
    let nextExecutions = $state<string[]>([]);
    let errorMessage = $state("");
    let timezone = $state("Asia/Shanghai");
    let isLoading = $state(false);
    let copied = $state<number | null>(null);

    const presets = [
        { label: "每分钟", value: "* * * * *", desc: "每分钟执行一次" },
        { label: "整点", value: "0 * * * *", desc: "每小时整点执行" },
        { label: "每天午夜", value: "0 0 * * *", desc: "每天00:00执行" },
        { label: "每天9点", value: "0 9 * * *", desc: "每天09:00执行" },
        { label: "周一至周五9点", value: "0 9 * * 1-5", desc: "工作日09:00执行" },
        { label: "每月1日中午", value: "0 12 1 * *", desc: "每月1日12:00执行" },
        { label: "每周日凌晨", value: "0 0 * * 0", desc: "每周日00:00执行" },
        { label: "每15分钟", value: "*/15 * * * *", desc: "每15分钟执行一次" },
    ];

    const parseCron = () => {
        isLoading = true;
        try {
            const interval = cronParser.parse(cronExpression, {
                tz: timezone,
            });

            nextExecutions = Array.from({ length: 10 }, () => {
                const next = interval.next();
                return format(next.toDate(), "yyyy-MM-dd HH:mm:ss");
            });

            errorMessage = "";
        } catch (err) {
            errorMessage = (err as Error).message
                .replace("Validation error for field: ", "字段验证错误: ")
                .replace("Got value:", "错误值:");
            nextExecutions = [];
        } finally {
            isLoading = false;
        }
    };

    async function copyTime(time: string, index: number) {
        try {
            await navigator.clipboard.writeText(time);
            copied = index;
            setTimeout(() => copied = null, 1500);
        } catch (err) {
            console.error('复制失败:', err);
        }
    }

    $effect(() => {
        if (cronExpression) {
            parseCron();
        }
    });
</script>

<div class="cron-tool">
    <div class="header-section">
        <div class="timezone-select">
            <label>时区：</label>
            <select bind:value={timezone}>
                {#each Intl.supportedValuesOf("timeZone") as tz}
                    <option value={tz}>{tz.replace(/_/g, " ")}</option>
                {/each}
            </select>
        </div>
    </div>

    <div class="input-section">
        <div class="input-header">
            <label>Cron 表达式</label>
            {#if isLoading}
                <span class="loading-badge">解析中...</span>
            {/if}
        </div>
        
        <input
            bind:value={cronExpression}
            class:error={!!errorMessage}
            placeholder="例如: 0 9 * * 1-5"
        />

        {#if errorMessage}
            <div class="error-message">
                ⚠️ {errorMessage}
            </div>
        {/if}

        <div class="cron-structure">
            <div class="structure-item">
                <span class="field-value">{cronExpression.split(' ')[0] || '*'}</span>
                <span class="field-name">分钟</span>
                <span class="field-range">0-59</span>
            </div>
            <div class="structure-item">
                <span class="field-value">{cronExpression.split(' ')[1] || '*'}</span>
                <span class="field-name">小时</span>
                <span class="field-range">0-23</span>
            </div>
            <div class="structure-item">
                <span class="field-value">{cronExpression.split(' ')[2] || '*'}</span>
                <span class="field-name">日期</span>
                <span class="field-range">1-31</span>
            </div>
            <div class="structure-item">
                <span class="field-value">{cronExpression.split(' ')[3] || '*'}</span>
                <span class="field-name">月份</span>
                <span class="field-range">1-12</span>
            </div>
            <div class="structure-item">
                <span class="field-value">{cronExpression.split(' ')[4] || '*'}</span>
                <span class="field-name">星期</span>
                <span class="field-range">0-7</span>
            </div>
        </div>
    </div>

    <div class="presets-section">
        <h3>常用表达式</h3>
        <div class="presets-grid">
            {#each presets as preset}
                <button
                    class="preset-btn"
                    class:active={cronExpression === preset.value}
                    onclick={() => (cronExpression = preset.value)}
                    title={preset.desc}
                >
                    <span class="preset-label">{preset.label}</span>
                    <code class="preset-value">{preset.value}</code>
                </button>
            {/each}
        </div>
    </div>

    <div class="results-section">
        <div class="results-header">
            <h3>执行时间表</h3>
            <span class="timezone-badge">{timezone}</span>
        </div>

        {#if nextExecutions.length > 0}
            <div class="execution-list">
                {#each nextExecutions as time, i}
                    <div class="execution-item">
                        <span class="index">#{i + 1}</span>
                        <code class="time">{time}</code>
                        <button
                            class="copy-btn"
                            onclick={() => copyTime(time, i)}
                            title="复制"
                        >
                            {copied === i ? '✓' : '📋'}
                        </button>
                    </div>
                {/each}
            </div>
        {:else if !errorMessage}
            <div class="empty-state">
                <span class="empty-icon">📭</span>
                <span>没有可用的执行时间</span>
            </div>
        {/if}
    </div>

    <div class="info-section">
        <h3>Cron 表达式语法</h3>
        <div class="syntax-grid">
            <div class="syntax-item">
                <code>*</code>
                <span>任意值</span>
            </div>
            <div class="syntax-item">
                <code>,</code>
                <span>值列表 (1,3,5)</span>
            </div>
            <div class="syntax-item">
                <code>-</code>
                <span>范围 (1-5)</span>
            </div>
            <div class="syntax-item">
                <code>/</code>
                <span>步长 (*/15)</span>
            </div>
        </div>
    </div>
</div>

<style>
    .cron-tool {
        display: flex;
        flex-direction: column;
        gap: 1.5rem;
    }

    .header-section {
        display: flex;
        justify-content: flex-end;
    }

    .timezone-select {
        display: flex;
        align-items: center;
        gap: 0.75rem;
    }

    .timezone-select label {
        font-size: 0.9rem;
        color: var(--text-secondary);
    }

    .timezone-select select {
        min-width: 220px;
        background: var(--bg-card);
        border: 1px solid var(--border);
        border-radius: var(--radius-sm);
        color: var(--text-primary);
        padding: 0.6rem 2.5rem 0.6rem 1rem;
        font-size: 0.9rem;
        cursor: pointer;
        appearance: none;
        -webkit-appearance: none;
        background-image: url("data:image/svg+xml;charset=UTF-8,%3csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='%239ca3af' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'%3e%3cpolyline points='6 9 12 15 18 9'%3e%3c/polyline%3e%3c/svg%3e");
        background-repeat: no-repeat;
        background-position: right 0.75rem center;
        background-size: 1em;
    }

    .timezone-select select:focus {
        outline: none;
        border-color: var(--primary);
        box-shadow: 0 0 0 3px rgba(99, 102, 241, 0.2);
    }

    .timezone-select select option {
        background: #1a1a2e;
        color: #e4e4e7;
        padding: 0.5rem;
    }

    .input-section {
        padding: 1.25rem;
        background: var(--bg-dark);
        border: 1px solid var(--border);
        border-radius: var(--radius-sm);
    }

    .input-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 0.75rem;
    }

    .input-header label {
        font-weight: 500;
    }

    .loading-badge {
        font-size: 0.8rem;
        padding: 0.25rem 0.75rem;
        background: var(--primary);
        color: white;
        border-radius: 100px;
    }

    .input-section input {
        width: 100%;
        font-family: 'JetBrains Mono', monospace;
        font-size: 1.1rem;
        text-align: center;
        letter-spacing: 0.1em;
    }

    .input-section input.error {
        border-color: #f87171 !important;
    }

    .error-message {
        margin-top: 0.75rem;
        padding: 0.75rem 1rem;
        background: rgba(239, 68, 68, 0.1);
        border: 1px solid rgba(239, 68, 68, 0.3);
        border-radius: var(--radius-sm);
        color: #f87171;
        font-size: 0.9rem;
    }

    .cron-structure {
        display: flex;
        justify-content: center;
        gap: 0.5rem;
        margin-top: 1rem;
        padding-top: 1rem;
        border-top: 1px solid var(--border);
    }

    .structure-item {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 0.25rem;
        padding: 0.5rem 1rem;
        background: var(--bg-card);
        border-radius: var(--radius-sm);
        min-width: 70px;
    }

    .field-value {
        font-family: 'JetBrains Mono', monospace;
        font-weight: 600;
        color: var(--accent);
        font-size: 1.1rem;
    }

    .field-name {
        font-size: 0.75rem;
        color: var(--text-secondary);
    }

    .field-range {
        font-size: 0.7rem;
        color: var(--text-muted);
    }

    .presets-section {
        padding: 1rem;
        background: var(--bg-dark);
        border: 1px solid var(--border);
        border-radius: var(--radius-sm);
    }

    .presets-section h3 {
        font-size: 0.9rem;
        margin-bottom: 0.75rem;
        color: var(--text-secondary);
    }

    .presets-grid {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
        gap: 0.5rem;
    }

    .preset-btn {
        display: flex;
        flex-direction: column;
        gap: 0.25rem;
        padding: 0.75rem !important;
        background: var(--bg-card) !important;
        border: 1px solid var(--border) !important;
        text-align: left;
        cursor: pointer;
    }

    .preset-btn:hover {
        border-color: var(--primary) !important;
        transform: none !important;
        box-shadow: none !important;
    }

    .preset-btn.active {
        background: rgba(34, 211, 238, 0.1) !important;
        border-color: var(--accent) !important;
    }

    .preset-label {
        font-weight: 500;
        color: var(--text-primary);
    }

    .preset-value {
        font-size: 0.8rem;
        color: var(--accent);
        background: transparent !important;
        padding: 0 !important;
    }

    .results-section {
        border: 1px solid var(--border);
        border-radius: var(--radius-sm);
        overflow: hidden;
    }

    .results-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 0.75rem 1rem;
        background: var(--bg-dark);
        border-bottom: 1px solid var(--border);
    }

    .results-header h3 {
        font-size: 0.95rem;
        margin: 0;
    }

    .timezone-badge {
        font-size: 0.8rem;
        padding: 0.25rem 0.75rem;
        background: var(--bg-hover);
        border-radius: 100px;
        color: var(--text-secondary);
    }

    .execution-list {
        max-height: 400px;
        overflow-y: auto;
    }

    .execution-item {
        display: flex;
        align-items: center;
        gap: 1rem;
        padding: 0.75rem 1rem;
        border-bottom: 1px solid var(--border);
        transition: background 0.15s;
    }

    .execution-item:last-child {
        border-bottom: none;
    }

    .execution-item:hover {
        background: var(--bg-hover);
    }

    .index {
        font-size: 0.85rem;
        color: var(--text-muted);
        min-width: 30px;
    }

    .time {
        flex: 1;
        font-family: 'JetBrains Mono', monospace;
        font-size: 0.95rem;
        color: var(--accent);
        background: transparent !important;
        padding: 0 !important;
    }

    .copy-btn {
        padding: 0.35rem 0.65rem !important;
        background: transparent !important;
        border: none !important;
        opacity: 0.5;
        transition: opacity 0.15s;
    }

    .execution-item:hover .copy-btn {
        opacity: 1;
    }

    .empty-state {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        padding: 2rem;
        gap: 0.5rem;
        color: var(--text-muted);
    }

    .empty-icon {
        font-size: 2rem;
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

    .syntax-grid {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
        gap: 0.5rem;
    }

    .syntax-item {
        display: flex;
        align-items: center;
        gap: 0.75rem;
        padding: 0.5rem 0.75rem;
        background: var(--bg-card);
        border-radius: var(--radius-sm);
    }

    .syntax-item code {
        font-weight: 600;
        color: var(--accent);
        background: transparent !important;
        padding: 0 !important;
        min-width: 30px;
    }

    .syntax-item span {
        font-size: 0.85rem;
        color: var(--text-muted);
    }
</style>
