<script lang="ts">
    import { tick } from "svelte";
    let activeTab = "timestampToDate";
    let timestamp = "";
    let dateString = "";
    let selectedTimezone = "Asia/Shanghai";
    let unit = "seconds";
    let result = "";

    const timezones = [
        "UTC",
        "Asia/Shanghai",
        "America/New_York",
        "Europe/London",
        "Asia/Tokyo",
    ];

    // 统一处理输入变化
    const handleInput = async (type: "timestamp" | "date") => {
        await tick(); // 等待 Svelte 更新 DOM

        if (type === "timestamp") {
            dateString = ""; // 清除日期输入
        } else {
            timestamp = ""; // 清除时间戳输入
        }

        try {
            if (activeTab === "timestampToDate" && timestamp) {
                const numericTimestamp =
                    unit === "seconds"
                        ? parseInt(timestamp) * 1000
                        : parseInt(timestamp);
                const date = new Date(numericTimestamp);

                // 格式化为 yyyy-MM-dd HH:mm:ss.SSS
                const year = date.getFullYear();
                const month = String(date.getMonth() + 1).padStart(2, "0");
                const day = String(date.getDate()).padStart(2, "0");
                const hours = String(date.getHours()).padStart(2, "0");
                const minutes = String(date.getMinutes()).padStart(2, "0");
                const seconds = String(date.getSeconds()).padStart(2, "0");
                const milliseconds = String(date.getMilliseconds()).padStart(
                    3,
                    "0",
                );

                result = `${year}-${month}-${day} ${hours}:${minutes}:${seconds}.${milliseconds}`;
            } else if (activeTab === "dateToTimestamp" && dateString) {
                const date = new Date(dateString);
                const ts =
                    unit === "seconds"
                        ? Math.floor(date.getTime() / 1000)
                        : date.getTime();
                result = ts.toString();
            }
        } catch (e) {
            result = `错误: ${(e as Error).message}`;
        }
    };
</script>

<div class="container">
    <!-- 选项卡切换 -->
    <div class="tabs">
        <button
            class={activeTab === "timestampToDate" ? "active" : ""}
            on:click={() => (activeTab = "timestampToDate")}
        >
            ⏱️ 时间戳转日期
        </button>
        <button
            class={activeTab === "dateToTimestamp" ? "active" : ""}
            on:click={() => (activeTab = "dateToTimestamp")}
        >
            📅 日期转时间戳
        </button>
    </div>

    <!-- 时间戳转日期 -->
    {#if activeTab === "timestampToDate"}
        <div class="converter-section">
            <div class="input-group">
                <input
                    type="text"
                    bind:value={timestamp}
                    on:input={() => handleInput("timestamp")}
                    placeholder="输入时间戳（如 1717027200）"
                />
                <select bind:value={unit}>
                    <option value="seconds">秒</option>
                    <option value="milliseconds">毫秒</option>
                </select>
            </div>
        </div>
    {:else}
        <!-- 日期转时间戳 -->
        <!-- <div class="converter-section">
            <div class="input-group">
                <input
                    type="datetime-local"
                    bind:value={dateString}
                    on:input={() => handleInput("date")}
                    class="custom-datetime"
                />
            </div>
        </div> -->
        <div class="date-picker">
            <input
                type="datetime-local"
                bind:value={dateString}
                on:input={() => handleInput("date")}
                class="modern-date-input"
                step="1"
            />
            <div class="custom-ui">
                <span class="icon">📅</span>
                <div class="datetime-display">
                    {#if dateString}
                        {dateString}
                    {:else}
                        <span class="placeholder">选择日期和时间</span>
                    {/if}
                </div>
            </div>
        </div>
    {/if}

    <!-- 公共控制项 -->
    <div class="settings">
        <div class="timezone-selector">
            <label>🌍 时区：</label>
            <select bind:value={selectedTimezone}>
                {#each timezones as tz}
                    <option value={tz}>{tz}</option>
                {/each}
            </select>
        </div>

        {#if activeTab === "dateToTimestamp"}
            <div class="unit-selector">
                <label>📏 单位：</label>
                <select bind:value={unit}>
                    <option value="seconds">秒</option>
                    <option value="milliseconds">毫秒</option>
                </select>
            </div>
        {/if}
    </div>

    <!-- 结果展示 -->
    <div class="result-card">
        <div class="result-header">
            <span>🔍 转换结果</span>
            <button
                on:click={() => navigator.clipboard.writeText(result)}
                title="复制结果"
            >
                📋
            </button>
        </div>
        <pre class={result?.startsWith("错误") ? "error" : ""}>{result ||
                "🕒 等待输入..."}</pre>
    </div>
</div>

<style>
    :global(:root) {
        --primary: #6366f1;
        --background: #f8fafc;
        --surface: #ffffff;
        --error: #dc2626;
    }

    .container {
        max-width: 640px;
        margin: 2rem auto;
        padding: 1.5rem;
        background: var(--surface);
        border-radius: 12px;
        box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1);
        font-family: "Segoe UI", system-ui;
    }

    .tabs {
        display: flex;
        gap: 0.5rem;
        margin-bottom: 1.5rem;
    }

    .tabs button {
        flex: 1;
        padding: 0.75rem;
        border: none;
        border-radius: 8px;
        background: var(--background);
        color: #64748b;
        cursor: pointer;
        transition: all 0.2s;
    }

    .tabs button.active {
        background: var(--primary);
        color: white;
        box-shadow: 0 2px 4px rgba(99, 102, 241, 0.3);
    }

    .input-group {
        display: flex;
        gap: 1rem;
        margin: 1rem 0;
    }

    input,
    select {
        padding: 0.75rem;
        border: 2px solid #e2e8f0;
        border-radius: 8px;
        width: 100%;
        font-size: 1rem;
        transition: border-color 0.2s;
    }

    input:focus,
    select:focus {
        outline: none;
        border-color: var(--primary);
        box-shadow: 0 0 0 2px rgba(99, 102, 241, 0.2);
    }

    .custom-datetime {
        /* 隐藏原生日期控件 */
        -webkit-appearance: none;
        -moz-appearance: none;
        appearance: none;
        background: url("data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='%2364668f'><path d='M19 3h-1V1h-2v2H8V1H6v2H5c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2zm0 16H5V9h14v10zM5 7V5h14v2H5z'/></svg>")
            no-repeat right 0.75rem center/20px 20px;
    }

    .settings {
        display: flex;
        gap: 1rem;
        margin: 1rem 0;
        flex-wrap: wrap;
    }

    .result-card {
        background: var(--background);
        border-radius: 8px;
        padding: 1rem;
        margin-top: 1.5rem;
    }

    .result-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 0.5rem;
        color: #475569;
    }

    pre {
        margin: 0;
        white-space: pre-wrap;
        word-break: break-all;
        padding: 1rem;
        background: var(--surface);
        border-radius: 6px;
    }

    .error {
        color: var(--error);
        background: #fee2e2;
    }

    @media (max-width: 640px) {
        .container {
            margin: 1rem;
            padding: 1rem;
        }

        .input-group {
            flex-direction: column;
        }
    }
    .date-picker {
        position: relative;
        margin: 1rem 0;
    }

    .modern-date-input {
        opacity: 0;
        position: absolute;
        width: 100%;
        height: 100%;
        cursor: pointer;
    }

    .custom-ui {
        display: flex;
        align-items: center;
        gap: 1rem;
        padding: 0.75rem;
        border: 2px solid #e2e8f0;
        border-radius: 8px;
        transition: all 0.2s;
        background: white;
    }

    .custom-ui:hover {
        border-color: var(--primary);
        box-shadow: 0 2px 8px rgba(99, 102, 241, 0.1);
    }

    .icon {
        font-size: 1.2rem;
        padding: 0.25rem;
    }

    .datetime-display {
        flex-grow: 1;
        color: #475569;
    }

    .placeholder {
        color: #94a3b8;
        font-style: italic;
    }

    /* 原生日期弹窗样式优化 */
    input[type="datetime-local"]::-webkit-calendar-picker-indicator {
        cursor: pointer;
        padding: 4px;
        border-radius: 4px;
        background-color: var(--primary);
    }

    input[type="datetime-local"]::-webkit-datetime-edit-fields-wrapper {
        padding: 0.5rem;
    }
</style>
