<script lang="ts">
    import cronParser from "cron-parser";
    import { format } from "date-fns";
    import Button from "../../lib/ui/Button.svelte";
    import CopyButton from "../../lib/ui/CopyButton.svelte";
    import Panel from "../../lib/ui/Panel.svelte";
    import Tabs from "../../lib/ui/Tabs.svelte";
    import TextInput from "../../lib/ui/TextInput.svelte";
    import TextArea from "../../lib/ui/TextArea.svelte";
    import EmptyState from "../../lib/ui/EmptyState.svelte";

    type InputMode = "manual" | "smart";

    let cronExpression = $state("* * * * *");
    let nextExecutions = $state<string[]>([]);
    let errorMessage = $state("");
    let timezone = $state("Asia/Shanghai");
    let isLoading = $state(false);
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

<div class="tool">
  <div class="toolbar">
    <Tabs
      bind:value={inputMode}
      tabs={[
        { value: "manual", label: "手动输入" },
        { value: "smart", label: "自然语言生成" },
      ]}
      ariaLabel="输入方式"
    />
    <div class="spacer"></div>
    <label class="tz">
      时区
      <select bind:value={timezone} aria-label="时区">
        {#each Intl.supportedValuesOf("timeZone") as tz (tz)}
          <option value={tz}>{tz.replace(/_/g, " ")}</option>
        {/each}
      </select>
    </label>
  </div>

  {#if inputMode === "manual"}
    <Panel title="Cron 表达式">
      {#snippet actions()}
        <CopyButton value={() => cronExpression} disabled={!cronExpression} />
      {/snippet}

      <TextInput
        bind:value={cronExpression}
        mono
        invalid={!!errorMessage}
        placeholder="例如：0 9 * * 1-5"
        ariaLabel="Cron 表达式"
      />

      {#if errorMessage}
        <p class="error">{errorMessage}</p>
      {/if}

      <div class="fields">
        {#each [["分钟", "0-59"], ["小时", "0-23"], ["日期", "1-31"], ["月份", "1-12"], ["星期", "0-7"]] as field, index (field[0])}
          <div class="field">
            <span class="value">{cronExpression.split(" ")[index] || "*"}</span>
            <span class="name">{field[0]}</span>
            <span class="range">{field[1]}</span>
          </div>
        {/each}
      </div>
    </Panel>

    <Panel title="常用表达式">
      <div class="presets">
        {#each presets as preset (preset.value)}
          <button class="preset" onclick={() => (cronExpression = preset.value)} title={preset.desc}>
            <code>{preset.value}</code>
            <span>{preset.label}</span>
          </button>
        {/each}
      </div>
    </Panel>
  {:else}
    <Panel title="自然语言描述">
      {#snippet actions()}
        <Button size="sm" variant="primary" onclick={generateFromPrompt}>生成表达式</Button>
      {/snippet}

      <TextArea
        bind:value={smartPrompt}
        rows={3}
        mono={false}
        placeholder="例如：每周一周三周五 1 点执行"
        ariaLabel="自然语言描述"
      />

      {#if smartError}
        <p class="error">{smartError}</p>
      {/if}

      {#if smartGeneratedExpression}
        <div class="smart">
          <div class="smart-row">
            <code>{smartGeneratedExpression}</code>
            <CopyButton value={smartGeneratedExpression} />
            <Button size="sm" onclick={applyGeneratedCron}>切到手动编辑</Button>
          </div>
          <p class="note">{smartExplanation}</p>
        </div>
      {/if}

      <div class="examples">
        {#each smartExamples as example (example.prompt)}
          <button class="example" onclick={() => (smartPrompt = example.prompt)}>
            <span>{example.prompt}</span>
            <code>{example.expression}</code>
          </button>
        {/each}
      </div>

      <ul class="rules">
        {#each smartRules as rule (rule)}
          <li>{rule}</li>
        {/each}
      </ul>
    </Panel>
  {/if}

  <Panel title="接下来 10 次执行" grow>
    {#snippet actions()}
      <CopyButton value={() => nextExecutions.join("\n")} label="复制全部" disabled={nextExecutions.length === 0} />
    {/snippet}

    {#if errorMessage}
      <EmptyState icon="⏰" title="表达式无效" hint="修正后会自动重新计算" />
    {:else if nextExecutions.length === 0}
      <EmptyState icon="⏰" title={isLoading ? "解析中…" : "还没有结果"} />
    {:else}
      <ol class="times">
        {#each nextExecutions as time, index (index)}
          <li>
            <span class="idx">{index + 1}</span>
            <code>{time}</code>
            <CopyButton value={time} icon title="复制时间" />
          </li>
        {/each}
      </ol>
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

  .toolbar {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: var(--sp-3);
  }

  .spacer {
    flex: 1;
  }

  .tz {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    font-size: var(--fs-sm);
    color: var(--color-text-secondary);
  }

  .tz select {
    background: var(--color-bg);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    color: var(--color-text);
    padding: 0.3rem var(--sp-2);
    font-size: var(--fs-sm);
    max-width: 220px;
  }

  .fields {
    display: grid;
    grid-template-columns: repeat(5, 1fr);
    gap: var(--sp-2);
    margin-top: var(--sp-3);
  }

  .field {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 2px;
    padding: var(--sp-2);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    background: var(--color-surface-2);
  }

  .value {
    font-family: var(--font-mono);
    font-size: var(--fs-lg);
    font-weight: var(--fw-semibold);
    color: var(--color-primary);
  }

  .name {
    font-size: var(--fs-xs);
    color: var(--color-text-secondary);
  }

  .range {
    font-size: var(--fs-xs);
    color: var(--color-text-muted);
    font-family: var(--font-mono);
  }

  .presets {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(190px, 1fr));
    gap: var(--sp-2);
  }

  .preset {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 2px;
    padding: var(--sp-2) var(--sp-3);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    background: var(--color-surface-2);
    text-align: left;
    transition:
      border-color var(--motion),
      background var(--motion);
  }

  .preset:hover {
    border-color: var(--color-primary-border);
    background: var(--color-primary-bg);
  }

  .preset code {
    font-family: var(--font-mono);
    font-size: var(--fs-sm);
    color: var(--color-accent);
  }

  .preset span {
    font-size: var(--fs-xs);
    color: var(--color-text-muted);
  }

  .smart {
    margin-top: var(--sp-3);
    padding: var(--sp-3);
    border: 1px solid var(--color-primary-border);
    border-radius: var(--radius-sm);
    background: var(--color-primary-bg);
  }

  .smart-row {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
  }

  .smart-row code {
    flex: 1;
    font-family: var(--font-mono);
    font-size: var(--fs-lg);
    color: var(--color-primary);
  }

  .note {
    margin-top: var(--sp-2);
    font-size: var(--fs-sm);
    color: var(--color-text-secondary);
  }

  .examples {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(230px, 1fr));
    gap: var(--sp-2);
    margin-top: var(--sp-3);
  }

  .example {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--sp-2);
    padding: var(--sp-1) var(--sp-2);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    background: var(--color-surface-2);
    font-size: var(--fs-sm);
    color: var(--color-text-secondary);
    text-align: left;
  }

  .example:hover {
    border-color: var(--color-border-strong);
    color: var(--color-text);
  }

  .example code {
    font-family: var(--font-mono);
    font-size: var(--fs-xs);
    color: var(--color-accent);
  }

  .rules {
    margin-top: var(--sp-3);
    padding-left: var(--sp-5);
    font-size: var(--fs-xs);
    color: var(--color-text-muted);
    line-height: var(--lh-relaxed);
  }

  .times {
    list-style: none;
    display: flex;
    flex-direction: column;
    gap: var(--sp-1);
  }

  .times li {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    padding: var(--sp-1) var(--sp-2);
    border-radius: var(--radius-sm);
    background: var(--color-surface-2);
  }

  .idx {
    width: 1.5rem;
    font-family: var(--font-mono);
    font-size: var(--fs-xs);
    color: var(--color-text-muted);
    text-align: right;
  }

  .times code {
    flex: 1;
    font-family: var(--font-mono);
    font-size: var(--fs-md);
    color: var(--color-text);
  }

  .error {
    margin-top: var(--sp-2);
    padding: var(--sp-2) var(--sp-3);
    border: 1px solid var(--color-danger-border);
    border-radius: var(--radius-sm);
    background: var(--color-danger-bg);
    color: var(--color-danger);
    font-size: var(--fs-sm);
  }
</style>
