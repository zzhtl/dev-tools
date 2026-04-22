<script lang="ts">
    import cronParser from "cron-parser";
    import { format } from "date-fns";

    type InputMode = "manual" | "smart";

    let cronExpression = $state("* * * * *");
    let nextExecutions = $state<string[]>([]);
    let errorMessage = $state("");
    let timezone = $state("Asia/Shanghai");
    let isLoading = $state(false);
    let copied = $state<number | null>(null);
    let inputMode = $state<InputMode>("manual");
    let smartPrompt = $state("");
    let smartError = $state("");
    let smartGeneratedExpression = $state("");
    let smartExplanation = $state("");

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

    const smartExamples = [
        { prompt: "每隔5分钟执行一次", expression: "*/5 * * * *" },
        { prompt: "每隔1小时执行一次", expression: "0 * * * *" },
        { prompt: "每周五执行一次", expression: "0 0 * * 5" },
        { prompt: "每周一周三周五1点执行", expression: "0 1 * * 1,3,5" },
        { prompt: "每月最后一天执行", expression: "0 0 L * *" },
        { prompt: "每天下午6点30执行", expression: "30 18 * * *" },
    ];

    const smartRules = [
        "支持“每隔 N 分钟 / 小时 / 天”这类间隔表达。",
        "支持“每天 / 每周五 / 周一周三周五 1 点执行”这类固定周期表达。",
        "支持“每月 15 号 / 每月最后一天执行”这类按月规则。",
        "未写具体时间时默认按 00:00 处理；“每隔一周”默认按每周日 00:00 近似。",
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

    function normalizePrompt(prompt: string): string {
        return prompt
            .trim()
            .replace(/[，。；、]/g, "")
            .replace(/\s+/g, "")
            .replace(/个(?=(分钟|小时|天|月|星期|周))/g, "");
    }

    function parseChineseNumber(token: string): number | null {
        if (!token) return null;
        if (/^\d+$/.test(token)) return parseInt(token, 10);

        const digits: Record<string, number> = {
            零: 0, 一: 1, 二: 2, 两: 2, 三: 3, 四: 4, 五: 5,
            六: 6, 七: 7, 八: 8, 九: 9,
        };

        if (token === "十") return 10;
        if (token.includes("十")) {
            const [tensPart, onesPart] = token.split("十");
            const tens = tensPart ? digits[tensPart] ?? 0 : 1;
            const ones = onesPart ? digits[onesPart] ?? 0 : 0;
            return tens * 10 + ones;
        }

        if (token.length === 1) {
            return digits[token] ?? null;
        }

        let combined = "";
        for (const char of token) {
            if (!(char in digits)) return null;
            combined += String(digits[char]);
        }

        return combined ? parseInt(combined, 10) : null;
    }

    function pad(value: number): string {
        return String(value).padStart(2, "0");
    }

    function parseTime(text: string): { hour: number; minute: number; explicit: boolean } {
        const colonMatch = text.match(/(凌晨|早上|上午|中午|下午|晚上|傍晚)?(\d{1,2}):(\d{1,2})/);
        if (colonMatch) {
            let hour = parseInt(colonMatch[2], 10);
            const minute = parseInt(colonMatch[3], 10);
            const meridiem = colonMatch[1] || "";
            if (["下午", "晚上", "傍晚"].includes(meridiem) && hour < 12) hour += 12;
            if (meridiem === "中午" && hour < 11) hour += 12;
            return { hour, minute, explicit: true };
        }

        const timeMatch = text.match(/(凌晨|早上|上午|中午|下午|晚上|傍晚)?(\d{1,2}|[零一二三四五六七八九十两]+)(点|时)(半|((?:\d{1,2}|[零一二三四五六七八九十两]+))分?)?/);
        if (!timeMatch) return { hour: 0, minute: 0, explicit: false };

        const parsedHour = parseChineseNumber(timeMatch[2]);
        if (parsedHour === null) return { hour: 0, minute: 0, explicit: false };

        let hour = parsedHour;
        const meridiem = timeMatch[1] || "";
        let minute = 0;

        if (timeMatch[4] === "半") {
            minute = 30;
        } else if (timeMatch[5]) {
            minute = parseChineseNumber(timeMatch[5]) ?? 0;
        }

        if (["下午", "晚上", "傍晚"].includes(meridiem) && hour < 12) hour += 12;
        if (meridiem === "中午" && hour < 11) hour += 12;
        if ((meridiem === "凌晨" || meridiem === "早上" || meridiem === "上午") && hour === 12) hour = 0;

        return { hour, minute, explicit: true };
    }

    function parseWeekdays(text: string): number[] {
        if (text.includes("工作日")) return [1, 2, 3, 4, 5];
        if (text.includes("周末")) return [0, 6];

        const mapping: Record<string, number> = {
            "0": 0, "7": 0,
            一: 1, 二: 2, 三: 3, 四: 4, 五: 5, 六: 6, 日: 0, 天: 0,
            "1": 1, "2": 2, "3": 3, "4": 4, "5": 5, "6": 6,
        };

        const values = new Set<number>();
        const matches = text.matchAll(/(?:周|星期|礼拜)([一二三四五六日天0-7])/g);
        for (const match of matches) {
            const mapped = mapping[match[1]];
            if (mapped !== undefined) values.add(mapped);
        }
        return [...values].sort((a, b) => a - b);
    }

    function readInterval(text: string, unit: string): number | null {
        const match = text.match(new RegExp(`每隔([0-9零一二三四五六七八九十两]+)${unit}`));
        if (!match) return null;
        return parseChineseNumber(match[1]);
    }

    function buildCronFromPrompt(prompt: string): { expression: string; explanation: string } {
        const text = normalizePrompt(prompt);
        if (!text) {
            throw new Error("请输入要转换的自然语言描述");
        }

        const { hour, minute, explicit } = parseTime(text);
        const weekdays = parseWeekdays(text);

        const minuteInterval = readInterval(text, "分钟");
        if (minuteInterval !== null) {
            if (minuteInterval < 1 || minuteInterval > 59) throw new Error("分钟间隔需要在 1 到 59 之间");
            return {
                expression: minuteInterval === 1 ? "* * * * *" : `*/${minuteInterval} * * * *`,
                explanation: `已按“每隔 ${minuteInterval} 分钟”生成 Cron 表达式`,
            };
        }

        const hourInterval = readInterval(text, "小时");
        if (hourInterval !== null) {
            if (hourInterval < 1 || hourInterval > 23) throw new Error("小时间隔需要在 1 到 23 之间");
            return {
                expression: `${minute} ${hourInterval === 1 ? "*" : `*/${hourInterval}`} * * *`,
                explanation: `已按“每隔 ${hourInterval} 小时”生成 Cron 表达式${explicit ? `，并保留分钟 ${pad(minute)}` : ""}`,
            };
        }

        const dayInterval = readInterval(text, "天");
        if (dayInterval !== null) {
            if (dayInterval < 1 || dayInterval > 31) throw new Error("天数间隔需要在 1 到 31 之间");
            return {
                expression: `${minute} ${hour} ${dayInterval === 1 ? "*" : `*/${dayInterval}`} * *`,
                explanation: `已按“每隔 ${dayInterval} 天”生成 Cron 表达式${explicit ? `，执行时间为 ${pad(hour)}:${pad(minute)}` : ""}`,
            };
        }

        const weekInterval = readInterval(text, "周");
        if (weekInterval !== null) {
            if (weekInterval !== 1) {
                throw new Error("标准 5 位 Cron 无法精确表达“每隔 2 周以上”，目前仅支持每隔 1 周");
            }
            return {
                expression: `${minute} ${hour} * * 0`,
                explanation: `已将“每隔一周”近似为每周日 ${pad(hour)}:${pad(minute)} 执行`,
            };
        }

        if (/每月(最后一|最后1|末)天/.test(text)) {
            return {
                expression: `${minute} ${hour} L * *`,
                explanation: `已按“每月最后一天”生成 Cron 表达式${explicit ? `，执行时间为 ${pad(hour)}:${pad(minute)}` : ""}`,
            };
        }

        const monthDayMatch = text.match(/每月([0-9零一二三四五六七八九十两]+)(?:号|日)/);
        if (monthDayMatch) {
            const day = parseChineseNumber(monthDayMatch[1]);
            if (!day || day < 1 || day > 31) throw new Error("每月日期需要在 1 到 31 之间");
            return {
                expression: `${minute} ${hour} ${day} * *`,
                explanation: `已按“每月 ${day} 号”生成 Cron 表达式${explicit ? `，执行时间为 ${pad(hour)}:${pad(minute)}` : ""}`,
            };
        }

        if (weekdays.length > 0) {
            return {
                expression: `${minute} ${hour} * * ${weekdays.join(",")}`,
                explanation: `已按每周 ${weekdays.map(day => ["日", "一", "二", "三", "四", "五", "六"][day]).join("、")} ${pad(hour)}:${pad(minute)} 生成表达式`,
            };
        }

        if (text.includes("每天") || text.includes("每日")) {
            return {
                expression: `${minute} ${hour} * * *`,
                explanation: `已按每天 ${pad(hour)}:${pad(minute)} 生成 Cron 表达式`,
            };
        }

        if (text.includes("每周")) {
            return {
                expression: `${minute} ${hour} * * 0`,
                explanation: `已按每周日 ${pad(hour)}:${pad(minute)} 生成 Cron 表达式`,
            };
        }

        throw new Error("暂不支持这句话，请参考下面示例，尽量使用“每隔 / 每天 / 每周 / 每月”这类表达");
    }

    function applyGeneratedCron() {
        if (!smartGeneratedExpression) return;
        inputMode = "manual";
    }

    function generateFromPrompt() {
        try {
            const generated = buildCronFromPrompt(smartPrompt);
            smartGeneratedExpression = generated.expression;
            smartExplanation = generated.explanation;
            smartError = "";
            cronExpression = generated.expression;
        } catch (err) {
            smartGeneratedExpression = "";
            smartExplanation = "";
            smartError = (err as Error).message;
        }
    }

    function useSmartExample(prompt: string) {
        smartPrompt = prompt;
        generateFromPrompt();
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
            <label for="cron-timezone">时区：</label>
            <select id="cron-timezone" bind:value={timezone}>
                {#each Intl.supportedValuesOf("timeZone") as tz}
                    <option value={tz}>{tz.replace(/_/g, " ")}</option>
                {/each}
            </select>
        </div>
    </div>

    <div class="input-section">
        <div class="mode-tabs">
            <button class:active={inputMode === "manual"} onclick={() => inputMode = "manual"}>手动输入</button>
            <button class:active={inputMode === "smart"} onclick={() => inputMode = "smart"}>智能生成</button>
        </div>

        {#if inputMode === "manual"}
            <div class="input-header">
                <label for="cron-expression-input">Cron 表达式</label>
                {#if isLoading}
                    <span class="loading-badge">解析中...</span>
                {/if}
            </div>
            
            <input
                id="cron-expression-input"
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
        {:else}
            <div class="input-header smart-header">
                <label for="cron-natural-input">自然语言描述</label>
                <button class="generate-btn" onclick={generateFromPrompt}>生成表达式</button>
            </div>

            <textarea
                id="cron-natural-input"
                bind:value={smartPrompt}
                rows="3"
                placeholder="例如：每周一周三周五1点执行"
            ></textarea>

            {#if smartError}
                <div class="error-message">
                    ⚠️ {smartError}
                </div>
            {/if}

            {#if smartGeneratedExpression}
                <div class="smart-result">
                    <div class="smart-result-row">
                        <span class="smart-result-label">生成结果</span>
                        <code>{smartGeneratedExpression}</code>
                    </div>
                    <div class="smart-result-note">{smartExplanation}</div>
                    <button class="apply-btn" onclick={applyGeneratedCron}>切到手动编辑</button>
                </div>
            {/if}
        {/if}
    </div>

    {#if inputMode === "manual"}
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
    {:else}
        <div class="presets-section">
            <h3>智能生成示例</h3>
            <div class="presets-grid">
                {#each smartExamples as example}
                    <button
                        class="preset-btn"
                        class:active={smartPrompt === example.prompt}
                        onclick={() => useSmartExample(example.prompt)}
                        title={example.expression}
                    >
                        <span class="preset-label">{example.prompt}</span>
                        <code class="preset-value">{example.expression}</code>
                    </button>
                {/each}
            </div>
        </div>
    {/if}

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
        {#if inputMode === "manual"}
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
        {:else}
            <h3>智能生成输入规则</h3>
            <div class="syntax-grid smart-rules-grid">
                {#each smartRules as rule}
                    <div class="syntax-item rule-item">
                        <span>{rule}</span>
                    </div>
                {/each}
            </div>
        {/if}
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

    .mode-tabs {
        display: inline-flex;
        gap: 0.35rem;
        padding: 0.25rem;
        margin-bottom: 1rem;
        background: var(--bg-card);
        border: 1px solid var(--border);
        border-radius: var(--radius-sm);
    }

    .mode-tabs button {
        padding: 0.45rem 0.9rem !important;
        background: transparent !important;
        color: var(--text-secondary) !important;
        border: none !important;
        box-shadow: none !important;
    }

    .mode-tabs button.active {
        background: var(--primary) !important;
        color: white !important;
    }

    .mode-tabs button:hover {
        transform: none !important;
        box-shadow: none !important;
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

    .smart-header {
        margin-bottom: 0.65rem;
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

    .input-section textarea {
        width: 100%;
        resize: vertical;
        min-height: 88px;
        font-family: 'JetBrains Mono', monospace;
        line-height: 1.6;
    }

    .input-section input.error {
        border-color: #f87171 !important;
    }

    .generate-btn,
    .apply-btn {
        padding: 0.45rem 0.85rem !important;
        font-size: 0.85rem;
    }

    .generate-btn:hover,
    .apply-btn:hover {
        transform: none !important;
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

    .smart-result {
        margin-top: 0.9rem;
        padding: 0.9rem 1rem;
        background: rgba(34, 211, 238, 0.08);
        border: 1px solid rgba(34, 211, 238, 0.24);
        border-radius: var(--radius-sm);
    }

    .smart-result-row {
        display: flex;
        align-items: center;
        gap: 0.75rem;
        flex-wrap: wrap;
    }

    .smart-result-label {
        font-size: 0.85rem;
        color: var(--text-secondary);
    }

    .smart-result-row code {
        font-size: 0.95rem;
        color: var(--accent);
        background: var(--bg-card) !important;
    }

    .smart-result-note {
        margin-top: 0.65rem;
        font-size: 0.85rem;
        color: var(--text-secondary);
        line-height: 1.6;
    }

    .smart-result .apply-btn {
        margin-top: 0.85rem;
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

    .smart-rules-grid {
        grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
    }

    .rule-item {
        align-items: flex-start;
    }

    .rule-item span {
        line-height: 1.6;
    }
</style>
