<script lang="ts">
    import cronParser from "cron-parser";
    import { format } from "date-fns";
    import { fade } from "svelte/transition";

    let cronExpression = $state("* * * * *");
    let nextExecutions = $state<string[]>([]);
    let errorMessage = $state("");
    let timezone = $state("Asia/Shanghai");
    let isLoading = $state(false);

    const presets = [
        { label: "每分钟", value: "* * * * *", icon: "⏱️" },
        { label: "整点", value: "0 * * * *", icon: "🕛" },
        { label: "每天午夜", value: "0 0 * * *", icon: "🌙" },
        { label: "周一9点", value: "0 9 * * 1", icon: "📅" },
        { label: "月首中午", value: "0 12 1 * *", icon: "📆" },
    ];

    const parseCron = () => {
        isLoading = true;
        try {
            const interval = cronParser.parse(cronExpression, {
                tz: timezone,
            });

            nextExecutions = Array.from({ length: 5 }, () => {
                const next = interval.next();
                return format(next.toDate(), "yyyy-MM-dd HH:mm:ss");
            });

            errorMessage = "";
        } catch (err) {
            errorMessage = (err as Error).message
                .replace("Validation error for field: ", "")
                .replace("Got value:", "错误值:");
            nextExecutions = [];
        } finally {
            isLoading = false;
        }
    };

    $effect(() => {
        if (cronExpression) {
            parseCron();
        }
    });
</script>

<div class="cron-container">
    <header>
        <h1>
            <span role="img" aria-hidden="true">⏰</span>
            Cron表达式解析器
        </h1>

        <div class="timezone-select">
            <select bind:value={timezone}>
                {#each Intl.supportedValuesOf("timeZone") as tz}
                    <option value={tz}>{tz.replace(/_/g, " ")}</option>
                {/each}
            </select>
        </div>
    </header>

    <main>
        <div class="input-card">
            <div class="input-group">
                <div class="input-header">
                    <label>Cron表达式</label>
                    <div class="preset-tags">
                        {#each presets as preset}
                            <button
                                class:active={cronExpression === preset.value}
                                on:click={() => (cronExpression = preset.value)}
                            >
                                {preset.icon}
                                {preset.label}
                            </button>
                        {/each}
                    </div>
                </div>

                <div class="input-wrapper">
                    <input
                        bind:value={cronExpression}
                        class:error={!!errorMessage}
                        placeholder="例如: 0 9 * * 1-5"
                    />
                    {#if isLoading}
                        <div class="loader"></div>
                    {/if}
                </div>

                {#if errorMessage}
                    <div class="error-message" transition:fade>
                        ⚠️ {errorMessage}
                    </div>
                {/if}
            </div>
        </div>

        <div class="results-card">
            <h3>执行时间表 <span class="timezone">({timezone})</span></h3>

            {#if nextExecutions.length > 0}
                <ul>
                    {#each nextExecutions as time, i}
                        <li transition:fade>
                            <span class="index">#{i + 1}</span>
                            <span class="time">{time}</span>
                            <button
                                on:click={() =>
                                    navigator.clipboard.writeText(time)}
                                title="复制"
                            >
                                ⎘
                            </button>
                        </li>
                    {/each}
                </ul>
            {:else}
                <div class="empty-state">
                    <span role="img" aria-hidden="true">📭</span>
                    没有可用的执行时间
                </div>
            {/if}
        </div>
    </main>
</div>

<style>
    :global(body) {
        --primary: #6366f1;
        --background: #f8fafc;
        --surface: #ffffff;
        --text: #1e293b;
        --error: #dc2626;
        --radius: 12px;
        --shadow: 0 4px 6px -1px rgb(0 0 0 / 0.1);

        background: var(--background);
        color: var(--text);
        font-family: "Inter", system-ui;
    }

    @media (prefers-color-scheme: dark) {
        :global(body) {
            --background: #0f172a;
            --surface: #1e293b;
            --text: #f8fafc;
        }
    }

    .cron-container {
        max-width: 800px;
        margin: 2rem auto;
        padding: 0 1rem;
    }

    header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 2rem;

        h1 {
            font-size: 1.5rem;
            display: flex;
            align-items: center;
            gap: 0.5rem;
        }
    }

    .input-card,
    .results-card {
        background: var(--surface);
        border-radius: var(--radius);
        box-shadow: var(--shadow);
        padding: 1.5rem;
        margin-bottom: 1.5rem;
    }

    .input-group {
        position: relative;

        .input-header {
            display: flex;
            flex-wrap: wrap;
            gap: 1rem;
            margin-bottom: 1rem;
            align-items: center;

            label {
                font-weight: 500;
            }
        }
    }

    .preset-tags {
        display: flex;
        flex-wrap: wrap;
        gap: 0.5rem;

        button {
            padding: 0.5rem 1rem;
            border-radius: 20px;
            background: var(--background);
            border: 1px solid #e2e8f0;
            transition: all 0.2s;

            &.active {
                background: var(--primary);
                color: white;
                border-color: var(--primary);
            }

            &:hover {
                transform: translateY(-1px);
            }
        }
    }

    .input-wrapper {
        position: relative;

        input {
            width: 100%;
            padding: 1rem;
            border: 2px solid #e2e8f0;
            border-radius: var(--radius);
            background: var(--surface);
            color: var(--text);

            &.error {
                border-color: var(--error);
            }
        }

        .loader {
            position: absolute;
            right: 1rem;
            top: 50%;
            transform: translateY(-50%);
            width: 20px;
            height: 20px;
            border: 2px solid #e2e8f0;
            border-top-color: var(--primary);
            border-radius: 50%;
            animation: spin 1s linear infinite;
        }
    }

    .error-message {
        color: var(--error);
        margin-top: 0.5rem;
        padding: 0.5rem 1rem;
        background: #fee2e2;
        border-radius: 8px;
        font-size: 0.9em;
    }

    ul {
        list-style: none;
        padding: 0;
        margin: 0;

        li {
            display: flex;
            align-items: center;
            padding: 1rem;
            border-bottom: 1px solid #e2e8f0;

            &:last-child {
                border-bottom: none;
            }

            .index {
                width: 3rem;
                color: #64748b;
            }

            .time {
                flex: 1;
                font-family: monospace;
            }

            button {
                padding: 0.5rem;
                border-radius: 8px;
                background: none;

                &:hover {
                    background: #f1f5f9;
                }
            }
        }
    }

    .empty-state {
        text-align: center;
        padding: 2rem;
        color: #64748b;

        [role="img"] {
            font-size: 2rem;
            display: block;
            margin-bottom: 1rem;
        }
    }

    @keyframes spin {
        to {
            transform: rotate(360deg);
        }
    }
</style>
