<script lang="ts">
  import Button from "../../lib/ui/Button.svelte";
  import CopyButton from "../../lib/ui/CopyButton.svelte";
  import DiffView from "../../lib/ui/DiffView.svelte";
  import Tabs from "../../lib/ui/Tabs.svelte";
  import EmptyState from "../../lib/ui/EmptyState.svelte";
  import { readStore, writeStore } from "../../lib/storage";
  import { escapeHtml } from "../../lib/format";
  import { tick, untrack } from "svelte";
  import { apiJson } from "../../lib/api";
  import { downloadBlob } from "../../lib/download";
  import { diffLines, type DiffResult } from "../../lib/diff";

  let jsonInput = $state("");
  let jsonOutput = $state("");
  let structOutput = $state("");
  let errorInfo = $state<ErrorInfo | null>(null);
  let isProcessing = $state(false);
  let activeMode = $state("format");
  let indentSize = $state(2);
  let sortKeys = $state(false);
  let showStats = $state(false);
  let jsonStats = $state<JsonStats | null>(null);
  let searchQuery = $state("");
  let searchMatches = $state<SearchMatch[]>([]);
  let activeSearchIndex = $state(-1);
  let jsonPath = $state("");
  let jsonPathResult = $state("");
  // 历史记录落盘。此前是纯内存状态，切换工具即丢失。
  interface StoredHistoryItem {
    id: string;
    content: string;
    preview: string;
    timestamp: number;
    size: number;
  }
  const HISTORY_KEY = "json.history";
  /** 单条超过这个体积就不落盘，避免撑爆 localStorage 配额 */
  const HISTORY_PERSIST_LIMIT = 200 * 1024;

  let history = $state<HistoryItem[]>(
    readStore<StoredHistoryItem[]>(HISTORY_KEY, []).map((item) => ({
      ...item,
      timestamp: new Date(item.timestamp),
    })),
  );

  $effect(() => {
    writeStore(
      HISTORY_KEY,
      history
        .filter((item) => item.size <= HISTORY_PERSIST_LIMIT)
        .slice(0, 20)
        .map((item) => ({ ...item, timestamp: item.timestamp.getTime() })),
    );
  });
  let structLanguage = $state("go");
  let viewMode = $state<"editor" | "tree">("editor");
  let collapsedPaths = $state<Set<string>>(new Set());
  let parsedJson = $state<unknown>(null);
  let isFullscreen = $state(false);
  let splitRatio = $state(50); // 左右分割比例
  let editorsContainerEl = $state<HTMLDivElement | null>(null);
  let draggingSplit = $state(false);

  // 旧版这里只渲染了一根分隔条，splitRatio 从未被修改，实际拖不动
  function startSplitDrag(event: PointerEvent) {
    draggingSplit = true;
    (event.currentTarget as HTMLElement).setPointerCapture(event.pointerId);
  }

  function onSplitDrag(event: PointerEvent) {
    if (!draggingSplit || !editorsContainerEl) return;
    const rect = editorsContainerEl.getBoundingClientRect();
    const ratio = ((event.clientX - rect.left) / rect.width) * 100;
    splitRatio = Math.min(80, Math.max(20, ratio));
  }

  function endSplitDrag(event: PointerEvent) {
    if (!draggingSplit) return;
    draggingSplit = false;
    (event.currentTarget as HTMLElement).releasePointerCapture(event.pointerId);
  }

  function onSplitKeydown(event: KeyboardEvent) {
    if (event.key === "ArrowLeft") splitRatio = Math.max(20, splitRatio - 2);
    else if (event.key === "ArrowRight") splitRatio = Math.min(80, splitRatio + 2);
    else return;
    event.preventDefault();
  }
  let inputTextarea = $state<HTMLTextAreaElement | null>(null);
  let lineNumbersEl = $state<HTMLDivElement | null>(null);
  let highlightLayerEl = $state<HTMLDivElement | null>(null);
  let processTimer: ReturnType<typeof setTimeout> | null = null;
  let lastSearchQuery = "";
  const indentSelectId = "json-indent-size";
  const structLanguageSelectId = "json-struct-language";
  const jsonPathInputId = "json-path-input";
  const searchInputId = "json-search-input";

  // 新增能力的状态
  let convertFrom = $state("json");
  let convertTo = $state("yaml");
  let schemaMode = $state<"generate" | "validate">("generate");
  let schemaInput = $state(""); // validate 模式下的 schema
  let jsonInputB = $state(""); // diff 模式的第二输入
  let queryEngine = $state<"jsonpath" | "jq">("jsonpath");
  let backendError = $state(""); // convert/schema/query 后端错误
  let schemaResult = $state<{ valid: boolean; errors: { path: string; message: string }[] } | null>(null);
  let diffResult = $state<DiffResult | null>(null);

  interface ErrorInfo {
    line: number;
    column: number;
    message: string;
    lineContent: string;
    expected: string[];
    found: string;
  }

  interface HistoryItem {
    id: string;
    content: string;
    preview: string;
    timestamp: Date;
    size: number;
  }

  interface JsonStats {
    totalKeys: number;
    totalValues: number;
    maxDepth: number;
    arrayCount: number;
    objectCount: number;
    stringCount: number;
    numberCount: number;
    booleanCount: number;
    nullCount: number;
    totalSize: number;
  }

  interface SearchMatch {
    start: number;
    end: number;
  }

  const modes = [
    { id: "format", name: "格式化", icon: "{ }" },
    { id: "compress", name: "压缩", icon: "→" },
    { id: "convert", name: "格式互转", icon: "⇄" },
    { id: "struct", name: "转结构体", icon: "</>" },
    { id: "diff", name: "对比", icon: "⇆" },
    { id: "schema", name: "Schema", icon: "✓S" },
    { id: "query", name: "查询", icon: "?" },
    { id: "escape", name: "转义", icon: "\\" },
    { id: "validate", name: "验证", icon: "✓" },
  ];

  const structLanguages = [
    { id: "go", name: "Go", ext: "go" },
    { id: "java", name: "Java", ext: "java" },
    { id: "rust", name: "Rust", ext: "rs" },
    { id: "typescript", name: "TypeScript", ext: "ts" },
    { id: "cpp", name: "C++", ext: "cpp" },
    { id: "csharp", name: "C#", ext: "cs" },
  ];

  // 格式互转支持的数据格式
  const dataFormats = [
    { id: "json", name: "JSON" },
    { id: "yaml", name: "YAML" },
    { id: "toml", name: "TOML" },
    { id: "xml", name: "XML" },
    { id: "csv", name: "CSV" },
  ];

  // 右侧面板角色：输出 / diff 第二输入 / schema 输入
  const rightPanelRole = $derived(
    activeMode === "diff"
      ? "inputB"
      : activeMode === "schema" && schemaMode === "validate"
        ? "schemaInput"
        : "output",
  );
  const outputContent = $derived(activeMode === "struct" ? structOutput : jsonOutput);

  // 解析JSON错误 - 支持多种浏览器引擎(V8/WebKit/Firefox)
  function parseJsonError(input: string, error: Error): ErrorInfo {
    const errorMsg = error.message;
    const nativeLocation = parseNativeJsonErrorLocation(input, errorMsg);
    const manualLocation = findJsonErrorPosition(input);
    let line = nativeLocation.line;
    let column = nativeLocation.column;
    let customExpected: string[] = [];
    let customFound = '';
    let customMessage = '';

    // 原生报错经常定位到“下一行首次无法继续解析的位置”。
    // 如果手动解析能给出更早的定位，则优先使用更贴近真实错误点的结果。
    if (manualLocation && (!nativeLocation.hasAccurateLocation || isEarlierLocation(manualLocation, nativeLocation))) {
      line = manualLocation.line;
      column = manualLocation.column;
      customExpected = [manualLocation.expected];
      customFound = manualLocation.found;
      if (manualLocation.context) {
        customMessage = `在 ${manualLocation.context} 后缺少 '${manualLocation.expected}' 字符`;
      } else {
        if (manualLocation.expected.includes('或')) {
          customMessage = `此处之前缺少 ${manualLocation.expected} 分隔符`;
        } else {
          customMessage = `此处缺少 '${manualLocation.expected}' 字符`;
        }
      }
    }

    const lines = input.split('\n');
    const lineContent = lines[line - 1] || '';
    
    // 如果有自定义信息，使用它
    if (customExpected.length > 0) {
      return { 
        line, 
        column, 
        message: customMessage || getErrorDescription(errorMsg), 
        lineContent, 
        expected: customExpected, 
        found: customFound 
      };
    }

    const { expected, found } = parseExpectedAndFound(errorMsg, lineContent, column);
    return { line, column, message: getErrorDescription(errorMsg), lineContent, expected, found };
  }

  function isEarlierLocation(candidate: { line: number; column: number }, baseline: { line: number; column: number }): boolean {
    return candidate.line < baseline.line || (candidate.line === baseline.line && candidate.column < baseline.column);
  }

  function parseNativeJsonErrorLocation(input: string, errorMsg: string): { line: number; column: number; hasAccurateLocation: boolean } {
    const v8LineColMatch = errorMsg.match(/\(line (\d+) column (\d+)\)/i);
    if (v8LineColMatch) {
      return {
        line: parseInt(v8LineColMatch[1], 10),
        column: parseInt(v8LineColMatch[2], 10),
        hasAccurateLocation: true
      };
    }

    const firefoxMatch = errorMsg.match(/at line (\d+) column (\d+)/i);
    if (firefoxMatch) {
      return {
        line: parseInt(firefoxMatch[1], 10),
        column: parseInt(firefoxMatch[2], 10),
        hasAccurateLocation: true
      };
    }

    const posMatch = errorMsg.match(/at position (\d+)/i);
    if (posMatch) {
      const position = parseInt(posMatch[1], 10);
      return positionToLineColumn(input, position);
    }

    return { line: 1, column: 1, hasAccurateLocation: false };
  }

  function positionToLineColumn(input: string, position: number): { line: number; column: number; hasAccurateLocation: boolean } {
    let currentPos = 0;
    const lines = input.split('\n');

    for (let i = 0; i < lines.length; i++) {
      const lineLength = lines[i].length;
      if (currentPos + lineLength >= position) {
        return {
          line: i + 1,
          column: position - currentPos + 1,
          hasAccurateLocation: true
        };
      }
      currentPos += lineLength + 1;
    }

    if (lines.length === 0) {
      return { line: 1, column: 1, hasAccurateLocation: false };
    }

    return {
      line: lines.length,
      column: lines[lines.length - 1].length + 1,
      hasAccurateLocation: true
    };
  }

  // 错误详细信息
  interface ParseErrorDetail {
    line: number;
    column: number;
    expected: string;
    found: string;
    context?: string;
  }

  // 手动查找 JSON 错误位置（用于 WebKit 等不提供位置信息的引擎）
  function findJsonErrorPosition(input: string): ParseErrorDetail | null {
    let line = 1;
    let column = 1;
    let i = 0;
    
    interface StackItem {
      char: string;
      line: number;
      column: number;
      hasKey: boolean;
      hasValue: boolean;
      keyLine?: number;
      keyColumn?: number;
      keyContent?: string;
      lastValueLine?: number;
      lastValueColumn?: number;
      lastValueContext?: string;
      lastCommaLine?: number;
      lastCommaColumn?: number;
      trailingComma?: boolean;
    }
    
    const stack: StackItem[] = [];
    type State = 'value' | 'key' | 'colon' | 'comma_or_end';
    let state: State = 'value';

    const recordContainerValue = (line: number, column: number) => {
      const top = stack[stack.length - 1];
      if (!top) return;

      top.hasValue = true;
      top.lastValueLine = line;
      top.lastValueColumn = column;
      top.lastValueContext = top.char === '{' && top.keyContent ? `"${top.keyContent}"` : undefined;
      top.trailingComma = false;
    };

    const missingDelimiterError = (expected: string, found: string, fallbackLine: number, fallbackColumn: number): ParseErrorDetail => {
      const top = stack[stack.length - 1];
      if (top?.lastValueLine !== undefined) {
        return {
          line: top.lastValueLine,
          column: top.lastValueColumn || 1,
          expected,
          found,
          context: top.lastValueContext
        };
      }
      return { line: fallbackLine, column: fallbackColumn, expected, found };
    };

    const skipWhitespace = () => {
      while (i < input.length && /\s/.test(input[i])) {
        if (input[i] === '\n') { line++; column = 1; }
        else { column++; }
        i++;
      }
    };

    const inObject = () => stack.length > 0 && stack[stack.length - 1].char === '{';
    const inArray = () => stack.length > 0 && stack[stack.length - 1].char === '[';

    // 解析字符串
    const parseString = (): { content: string; startLine: number; startColumn: number } | null => {
      const startLine = line;
      const startColumn = column;
      i++; column++;
      
      let content = '';
      let escaped = false;
      while (i < input.length) {
        const c = input[i];
        if (c === '\n') return null;
        if (escaped) {
          escaped = false;
          content += c;
        } else if (c === '\\') {
          escaped = true;
          content += c;
        } else if (c === '"') {
          i++; column++;
          return { content, startLine, startColumn };
        } else {
          content += c;
        }
        i++; column++;
      }
      return null;
    };

    while (i < input.length) {
      skipWhitespace();
      if (i >= input.length) break;

      const char = input[i];
      const currentLine = line;
      const currentColumn = column;

      // 处理字符串
      if (char === '"') {
        if (inObject()) {
          const top = stack[stack.length - 1];
          if (state === 'key' || state === 'value') {
            const result = parseString();
            if (!result) {
              return { line: currentLine, column: currentColumn, expected: '完整的字符串', found: '未结束的字符串' };
            }
            if (state === 'key') {
              top.hasKey = true;
              top.keyLine = result.startLine;
              top.keyColumn = result.startColumn;
              top.keyContent = result.content;
              top.trailingComma = false;
              state = 'colon';
            } else {
              recordContainerValue(currentLine, column);
              state = 'comma_or_end';
            }
            continue;
          } else {
            return state === 'comma_or_end'
              ? missingDelimiterError(', 或 }', '"', currentLine, currentColumn)
              : { line: currentLine, column: currentColumn, expected: state === 'colon' ? ':' : ', 或 }', found: '"' };
          }
        } else if (inArray() || state === 'value') {
          const result = parseString();
          if (!result) {
            return { line: currentLine, column: currentColumn, expected: '完整的字符串', found: '未结束的字符串' };
          }
          if (inArray()) recordContainerValue(currentLine, column);
          state = 'comma_or_end';
          continue;
        } else {
          return { line: currentLine, column: currentColumn, expected: '值', found: '"' };
        }
      }

      // 处理对象开始 {
      if (char === '{') {
        if (state !== 'value') {
          if (state === 'comma_or_end') {
            return missingDelimiterError(inObject() ? ', 或 }' : ', 或 ]', '{', currentLine, currentColumn);
          }
          return { line: currentLine, column: currentColumn, expected: state === 'colon' ? ':' : '值', found: '{' };
        }
        stack.push({ char: '{', line: currentLine, column: currentColumn, hasKey: false, hasValue: false });
        state = 'key';
        i++; column++;
        continue;
      }

      // 处理对象结束 }
      if (char === '}') {
        if (!inObject()) {
          return { line: currentLine, column: currentColumn, expected: '值或 [', found: '}' };
        }
        const top = stack[stack.length - 1];
        
        if (top.trailingComma) {
          return { line: top.lastCommaLine || currentLine, column: top.lastCommaColumn || currentColumn, expected: '属性名', found: '}' };
        }

        // 如果期望冒号（有键但没冒号），错误在键那一行
        if (state === 'colon' && top.keyLine !== undefined) {
          return { 
            line: top.keyLine, 
            column: top.keyColumn || 1, 
            expected: ':', 
            found: '}',
            context: `"${top.keyContent}"` 
          };
        }
        
        // 如果期望值（有键和冒号但没值）
        if (state === 'value' && top.hasKey && !top.hasValue) {
          return { line: currentLine, column: currentColumn, expected: '值', found: '}' };
        }
        
        if (state !== 'key' && state !== 'comma_or_end') {
          return { line: currentLine, column: currentColumn, expected: '键或值', found: '}' };
        }
        
        stack.pop();
        if (stack.length > 0) recordContainerValue(currentLine, currentColumn);
        state = stack.length > 0 ? 'comma_or_end' : 'value';
        i++; column++;
        continue;
      }

      // 处理数组开始 [
      if (char === '[') {
        if (state !== 'value') {
          if (state === 'comma_or_end') {
            return missingDelimiterError(inObject() ? ', 或 }' : ', 或 ]', '[', currentLine, currentColumn);
          }
          const top = inObject() ? stack[stack.length - 1] : null;
          if (state === 'colon' && top?.keyLine !== undefined) {
            return { line: top.keyLine, column: top.keyColumn || 1, expected: ':', found: '[', context: `"${top.keyContent}"` };
          }
          return { line: currentLine, column: currentColumn, expected: state === 'colon' ? ':' : '值', found: '[' };
        }
        stack.push({ char: '[', line: currentLine, column: currentColumn, hasKey: false, hasValue: false });
        state = 'value';
        i++; column++;
        continue;
      }

      // 处理数组结束 ]
      if (char === ']') {
        if (!inArray()) {
          return { line: currentLine, column: currentColumn, expected: inObject() ? ': 或 }' : '值', found: ']' };
        }
        const top = stack[stack.length - 1];
        if (top.trailingComma) {
          return { line: top.lastCommaLine || currentLine, column: top.lastCommaColumn || currentColumn, expected: '值', found: ']' };
        }
        if (state !== 'value' && state !== 'comma_or_end') {
          return { line: currentLine, column: currentColumn, expected: '值', found: ']' };
        }
        stack.pop();
        if (stack.length > 0) recordContainerValue(currentLine, currentColumn);
        state = stack.length > 0 ? 'comma_or_end' : 'value';
        i++; column++;
        continue;
      }

      // 处理冒号 :
      if (char === ':') {
        if (state !== 'colon' || !inObject()) {
          return { line: currentLine, column: currentColumn, expected: '键或值', found: ':' };
        }
        state = 'value';
        i++; column++;
        continue;
      }

      // 处理逗号 ,
      if (char === ',') {
        if (state !== 'comma_or_end') {
          const top = inObject() ? stack[stack.length - 1] : null;
          if (state === 'colon' && top?.keyLine !== undefined) {
            return { line: top.keyLine, column: top.keyColumn || 1, expected: ':', found: ',', context: `"${top.keyContent}"` };
          }
          return { line: currentLine, column: currentColumn, expected: '值', found: ',' };
        }
        if (inObject()) {
          const top = stack[stack.length - 1];
          top.hasKey = false;
          top.hasValue = false;
          top.keyLine = undefined;
          top.keyColumn = undefined;
          top.keyContent = undefined;
          top.lastCommaLine = currentLine;
          top.lastCommaColumn = currentColumn;
          top.trailingComma = true;
          state = 'key';
        } else {
          const top = stack[stack.length - 1];
          if (top) {
            top.lastCommaLine = currentLine;
            top.lastCommaColumn = currentColumn;
            top.trailingComma = true;
          }
          state = 'value';
        }
        i++; column++;
        continue;
      }

      // 处理 true/false/null
      const keywords = [
        { word: 'true', len: 4 },
        { word: 'false', len: 5 },
        { word: 'null', len: 4 }
      ];
      
      let matched = false;
      for (const kw of keywords) {
        if (input.substring(i, i + kw.len) === kw.word) {
          if (state !== 'value') {
            if (state === 'comma_or_end') {
              return missingDelimiterError(inObject() ? ', 或 }' : ', 或 ]', kw.word, currentLine, currentColumn);
            }
            const top = inObject() ? stack[stack.length - 1] : null;
            if (state === 'colon' && top?.keyLine !== undefined) {
              return { line: top.keyLine, column: top.keyColumn || 1, expected: ':', found: kw.word, context: `"${top.keyContent}"` };
            }
            return { line: currentLine, column: currentColumn, expected: ':', found: kw.word };
          }
          i += kw.len; column += kw.len;
          if (inObject() || inArray()) recordContainerValue(currentLine, column - 1);
          state = 'comma_or_end';
          matched = true;
          break;
        }
      }
      if (matched) continue;

      // 处理数字
      if (/[-0-9]/.test(char)) {
        if (state !== 'value') {
          if (state === 'comma_or_end') {
            return missingDelimiterError(inObject() ? ', 或 }' : ', 或 ]', '数字', currentLine, currentColumn);
          }
          const top = inObject() ? stack[stack.length - 1] : null;
          if (state === 'colon' && top?.keyLine !== undefined) {
            return { line: top.keyLine, column: top.keyColumn || 1, expected: ':', found: '数字', context: `"${top.keyContent}"` };
          }
          return { line: currentLine, column: currentColumn, expected: ':', found: '数字' };
        }
        if (char === '-') { i++; column++; }
        while (i < input.length && /[0-9]/.test(input[i])) { i++; column++; }
        if (i < input.length && input[i] === '.') {
          i++; column++;
          while (i < input.length && /[0-9]/.test(input[i])) { i++; column++; }
        }
        if (i < input.length && /[eE]/.test(input[i])) {
          i++; column++;
          if (i < input.length && /[-+]/.test(input[i])) { i++; column++; }
          while (i < input.length && /[0-9]/.test(input[i])) { i++; column++; }
        }
        if (inObject() || inArray()) recordContainerValue(currentLine, column - 1);
        state = 'comma_or_end';
        continue;
      }

      // 其他字符是错误的
      return { line: currentLine, column: currentColumn, expected: '有效的 JSON 字符', found: `'${char}'` };
    }

    // 检查是否有未闭合的括号
    if (stack.length > 0) {
      const last = stack[stack.length - 1];
      if (state === 'colon' && last.keyLine !== undefined) {
        return { line: last.keyLine, column: last.keyColumn || 1, expected: ':', found: 'EOF', context: `"${last.keyContent}"` };
      }
      if (last.char === '{') {
        return { line: last.line, column: last.column, expected: '}', found: 'EOF' };
      }
      return { line: last.line, column: last.column, expected: ']', found: 'EOF' };
    }

    return { line: 1, column: 1, expected: 'JSON', found: 'EOF' };
  }

  function parseExpectedAndFound(errorMsg: string, lineContent: string, column: number): { expected: string[], found: string } {
    const expected: string[] = [];
    let found = '';
    const charAtError = lineContent[column - 1] || 'EOF';
    
    if (errorMsg.includes('Unexpected token')) {
      const tokenMatch = errorMsg.match(/Unexpected token '?(.+?)'?(?:\s|,|$)/);
      found = tokenMatch ? tokenMatch[1] : charAtError;
    } else if (errorMsg.includes('Unexpected end')) {
      found = 'EOF';
    } else {
      found = charAtError === '' ? 'EOF' : `'${charAtError}'`;
    }

    const trimmedBefore = lineContent.substring(0, column - 1).trim();
    const lastChar = trimmedBefore[trimmedBefore.length - 1];

    if (lastChar === '{' || lastChar === ',') {
      expected.push('STRING', 'NUMBER', 'NULL', 'TRUE', 'FALSE', '{', '[');
    } else if (lastChar === '[') {
      expected.push('STRING', 'NUMBER', 'NULL', 'TRUE', 'FALSE', '{', '[', ']');
    } else if (lastChar === ':') {
      expected.push('STRING', 'NUMBER', 'NULL', 'TRUE', 'FALSE', '{', '[');
    } else {
      expected.push('STRING', 'NUMBER', 'NULL', 'TRUE', 'FALSE', '{', '[');
    }

    return { expected, found };
  }

  function getErrorDescription(errorMsg: string): string {
    if (errorMsg.includes('Unexpected token')) {
      const tokenMatch = errorMsg.match(/Unexpected token '?(.)'?/);
      if (tokenMatch) {
        const token = tokenMatch[1];
        if (token === '}') return '意外的结束花括号';
        if (token === ']') return '意外的结束方括号';
        if (token === ',') return '意外的逗号';
        if (token === ':') return '属性名必须是双引号字符串';
        if (token === "'") return '必须使用双引号';
        return `意外的字符 '${token}'`;
      }
    }
    if (errorMsg.includes('Unexpected end')) return 'JSON不完整';
    if (errorMsg.includes("Expected property name")) return '期望属性名或 }';
    if (errorMsg.includes('Bad control character')) return '字符串中有无效控制字符';
    if (errorMsg.includes('Unterminated string')) return '字符串未结束';
    return errorMsg.replace(/\s*in JSON at position.*$/, '');
  }

  // 统计信息
  function calculateStats(obj: unknown, depth = 0): JsonStats {
    const stats: JsonStats = {
      totalKeys: 0, totalValues: 0, maxDepth: depth,
      arrayCount: 0, objectCount: 0, stringCount: 0,
      numberCount: 0, booleanCount: 0, nullCount: 0, totalSize: 0
    };

    if (obj === null) { stats.nullCount = 1; stats.totalValues = 1; return stats; }
    if (Array.isArray(obj)) {
      stats.arrayCount = 1;
      for (const item of obj) mergeStats(stats, calculateStats(item, depth + 1));
    } else if (typeof obj === 'object') {
      stats.objectCount = 1;
      stats.totalKeys = Object.keys(obj).length;
      for (const value of Object.values(obj)) mergeStats(stats, calculateStats(value, depth + 1));
    } else if (typeof obj === 'string') { stats.stringCount = 1; stats.totalValues = 1; }
    else if (typeof obj === 'number') { stats.numberCount = 1; stats.totalValues = 1; }
    else if (typeof obj === 'boolean') { stats.booleanCount = 1; stats.totalValues = 1; }

    stats.maxDepth = Math.max(stats.maxDepth, depth);
    return stats;
  }

  function mergeStats(target: JsonStats, source: JsonStats) {
    target.totalKeys += source.totalKeys;
    target.totalValues += source.totalValues;
    target.maxDepth = Math.max(target.maxDepth, source.maxDepth);
    target.arrayCount += source.arrayCount;
    target.objectCount += source.objectCount;
    target.stringCount += source.stringCount;
    target.numberCount += source.numberCount;
    target.booleanCount += source.booleanCount;
    target.nullCount += source.nullCount;
  }

  function sortJsonKeys(obj: unknown): unknown {
    if (obj === null || typeof obj !== 'object') return obj;
    if (Array.isArray(obj)) return obj.map(item => sortJsonKeys(item));
    const sorted: Record<string, unknown> = {};
    for (const key of Object.keys(obj).sort()) {
      sorted[key] = sortJsonKeys((obj as Record<string, unknown>)[key]);
    }
    return sorted;
  }

  // 类型转换辅助函数
  function toPascalCase(str: string): string {
    return str.replace(/(^|[_-])([a-z])/g, (_, __, c) => c.toUpperCase()).replace(/^[a-z]/, c => c.toUpperCase());
  }
  function toCamelCase(str: string): string {
    const pascal = toPascalCase(str);
    return pascal.charAt(0).toLowerCase() + pascal.slice(1);
  }
  function toSnakeCase(str: string): string {
    return str.replace(/([A-Z])/g, '_$1').toLowerCase().replace(/^_/, '');
  }

  function inferType(value: unknown, lang: string): string {
    if (value === null) {
      const nullTypes: Record<string, string> = { go: 'interface{}', java: 'Object', rust: 'Option<serde_json::Value>', typescript: 'null', cpp: 'std::nullptr_t', csharp: 'object?' };
      return nullTypes[lang] || 'any';
    }
    if (Array.isArray(value)) {
      if (value.length === 0) {
        const emptyTypes: Record<string, string> = { go: '[]interface{}', java: 'List<Object>', rust: 'Vec<serde_json::Value>', typescript: 'unknown[]', cpp: 'std::vector<std::any>', csharp: 'List<object>' };
        return emptyTypes[lang] || 'any[]';
      }
      const itemType = inferType(value[0], lang);
      const arrayFormats: Record<string, string> = { go: `[]${itemType}`, java: `List<${itemType}>`, rust: `Vec<${itemType}>`, typescript: `${itemType}[]`, cpp: `std::vector<${itemType}>`, csharp: `List<${itemType}>` };
      return arrayFormats[lang] || `${itemType}[]`;
    }
    if (typeof value === 'object') return 'NESTED_OBJECT';
    
    const isInt = typeof value === 'number' && Number.isInteger(value);
    const typeMap: Record<string, Record<string, string>> = {
      string: { go: 'string', java: 'String', rust: 'String', typescript: 'string', cpp: 'std::string', csharp: 'string' },
      number: { go: isInt ? 'int64' : 'float64', java: isInt ? 'Long' : 'Double', rust: isInt ? 'i64' : 'f64', typescript: 'number', cpp: isInt ? 'int64_t' : 'double', csharp: isInt ? 'long' : 'double' },
      boolean: { go: 'bool', java: 'Boolean', rust: 'bool', typescript: 'boolean', cpp: 'bool', csharp: 'bool' }
    };
    return typeMap[typeof value]?.[lang] || 'any';
  }

  // 结构体生成函数
  function generateGoStruct(obj: unknown, name = 'Root'): string {
    if (typeof obj !== 'object' || obj === null || Array.isArray(obj)) return '';
    const lines: string[] = [];
    const nestedStructs: string[] = [];
    lines.push(`type ${name} struct {`);
    for (const [key, value] of Object.entries(obj)) {
      const fieldName = toPascalCase(key);
      let fieldType = inferType(value, 'go');
      if (fieldType === 'NESTED_OBJECT') {
        fieldType = fieldName;
        const nested = generateGoStruct(value, fieldName);
        if (nested) nestedStructs.push(nested);
      } else if (Array.isArray(value) && value.length > 0 && typeof value[0] === 'object' && value[0] !== null) {
        const nestedName = fieldName + 'Item';
        fieldType = `[]${nestedName}`;
        const nested = generateGoStruct(value[0], nestedName);
        if (nested) nestedStructs.push(nested);
      }
      lines.push(`    ${fieldName} ${fieldType} \`json:"${key}"\``);
    }
    lines.push(`}`);
    return [...nestedStructs, lines.join('\n')].join('\n\n');
  }

  function generateJavaClass(obj: unknown, name = 'Root'): string {
    if (typeof obj !== 'object' || obj === null || Array.isArray(obj)) return '';
    const lines: string[] = [`public class ${name} {`];
    const nestedClasses: string[] = [];
    for (const [key, value] of Object.entries(obj)) {
      const fieldName = toCamelCase(key);
      let fieldType = inferType(value, 'java');
      if (fieldType === 'NESTED_OBJECT') {
        fieldType = toPascalCase(key);
        const nested = generateJavaClass(value, fieldType);
        if (nested) nestedClasses.push(nested);
      } else if (Array.isArray(value) && value.length > 0 && typeof value[0] === 'object' && value[0] !== null) {
        const nestedName = toPascalCase(key) + 'Item';
        fieldType = `List<${nestedName}>`;
        const nested = generateJavaClass(value[0], nestedName);
        if (nested) nestedClasses.push(nested);
      }
      lines.push(`    @JsonProperty("${key}")`);
      lines.push(`    private ${fieldType} ${fieldName};`);
    }
    lines.push(`}`);
    return `import com.fasterxml.jackson.annotation.JsonProperty;\nimport java.util.List;\n\n${[...nestedClasses, lines.join('\n')].join('\n\n')}`;
  }

  function generateRustStruct(obj: unknown, name = 'Root'): string {
    if (typeof obj !== 'object' || obj === null || Array.isArray(obj)) return '';
    const lines: string[] = [`#[derive(Debug, Serialize, Deserialize)]`, `pub struct ${name} {`];
    const nestedStructs: string[] = [];
    for (const [key, value] of Object.entries(obj)) {
      const fieldName = toSnakeCase(key);
      let fieldType = inferType(value, 'rust');
      if (fieldType === 'NESTED_OBJECT') {
        fieldType = toPascalCase(key);
        const nested = generateRustStruct(value, fieldType);
        if (nested) nestedStructs.push(nested);
      } else if (Array.isArray(value) && value.length > 0 && typeof value[0] === 'object' && value[0] !== null) {
        const nestedName = toPascalCase(key) + 'Item';
        fieldType = `Vec<${nestedName}>`;
        const nested = generateRustStruct(value[0], nestedName);
        if (nested) nestedStructs.push(nested);
      }
      if (fieldName !== key) lines.push(`    #[serde(rename = "${key}")]`);
      lines.push(`    pub ${fieldName}: ${fieldType},`);
    }
    lines.push(`}`);
    return `use serde::{Deserialize, Serialize};\n\n${[...nestedStructs, lines.join('\n')].join('\n\n')}`;
  }

  function generateTypeScriptInterface(obj: unknown, name = 'Root'): string {
    if (typeof obj !== 'object' || obj === null || Array.isArray(obj)) return '';
    const lines: string[] = [`interface ${name} {`];
    const nestedInterfaces: string[] = [];
    for (const [key, value] of Object.entries(obj)) {
      let fieldType = inferType(value, 'typescript');
      if (fieldType === 'NESTED_OBJECT') {
        fieldType = toPascalCase(key);
        const nested = generateTypeScriptInterface(value, fieldType);
        if (nested) nestedInterfaces.push(nested);
      } else if (Array.isArray(value) && value.length > 0 && typeof value[0] === 'object' && value[0] !== null) {
        const nestedName = toPascalCase(key) + 'Item';
        fieldType = `${nestedName}[]`;
        const nested = generateTypeScriptInterface(value[0], nestedName);
        if (nested) nestedInterfaces.push(nested);
      }
      const needsQuotes = !/^[a-zA-Z_][a-zA-Z0-9_]*$/.test(key);
      lines.push(`  ${needsQuotes ? `"${key}"` : key}: ${fieldType};`);
    }
    lines.push(`}`);
    return [...nestedInterfaces, lines.join('\n')].join('\n\n');
  }

  function generateCppStruct(obj: unknown, name = 'Root'): string {
    if (typeof obj !== 'object' || obj === null || Array.isArray(obj)) return '';
    const lines: string[] = [`struct ${name} {`];
    const nestedStructs: string[] = [];
    for (const [key, value] of Object.entries(obj)) {
      const fieldName = toSnakeCase(key);
      let fieldType = inferType(value, 'cpp');
      if (fieldType === 'NESTED_OBJECT') {
        fieldType = toPascalCase(key);
        const nested = generateCppStruct(value, fieldType);
        if (nested) nestedStructs.push(nested);
      } else if (Array.isArray(value) && value.length > 0 && typeof value[0] === 'object' && value[0] !== null) {
        const nestedName = toPascalCase(key) + 'Item';
        fieldType = `std::vector<${nestedName}>`;
        const nested = generateCppStruct(value[0], nestedName);
        if (nested) nestedStructs.push(nested);
      }
      lines.push(`    ${fieldType} ${fieldName}; // "${key}"`);
    }
    lines.push(`};`);
    return `#include <string>\n#include <vector>\n#include <cstdint>\n\n${[...nestedStructs, lines.join('\n')].join('\n\n')}`;
  }

  function generateCSharpClass(obj: unknown, name = 'Root'): string {
    if (typeof obj !== 'object' || obj === null || Array.isArray(obj)) return '';
    const lines: string[] = [`public class ${name}`, `{`];
    const nestedClasses: string[] = [];
    for (const [key, value] of Object.entries(obj)) {
      const fieldName = toPascalCase(key);
      let fieldType = inferType(value, 'csharp');
      if (fieldType === 'NESTED_OBJECT') {
        fieldType = toPascalCase(key);
        const nested = generateCSharpClass(value, fieldType);
        if (nested) nestedClasses.push(nested);
      } else if (Array.isArray(value) && value.length > 0 && typeof value[0] === 'object' && value[0] !== null) {
        const nestedName = toPascalCase(key) + 'Item';
        fieldType = `List<${nestedName}>`;
        const nested = generateCSharpClass(value[0], nestedName);
        if (nested) nestedClasses.push(nested);
      }
      lines.push(`    [JsonProperty("${key}")]`);
      lines.push(`    public ${fieldType} ${fieldName} { get; set; }`);
    }
    lines.push(`}`);
    return `using Newtonsoft.Json;\nusing System.Collections.Generic;\n\n${[...nestedClasses, lines.join('\n')].join('\n\n')}`;
  }

  function generateStruct(obj: unknown, lang: string): string {
    const generators: Record<string, (obj: unknown) => string> = {
      go: generateGoStruct, java: generateJavaClass, rust: generateRustStruct,
      typescript: generateTypeScriptInterface, cpp: generateCppStruct, csharp: generateCSharpClass
    };
    return generators[lang]?.(obj) || '';
  }

  // JSONPath 查询
  function buildSearchMatches(input: string, query: string): SearchMatch[] {
    const normalizedQuery = query.trim().toLowerCase();
    if (!normalizedQuery) return [];

    const normalizedInput = input.toLowerCase();
    const matches: SearchMatch[] = [];
    let fromIndex = 0;

    while (fromIndex < normalizedInput.length) {
      const foundIndex = normalizedInput.indexOf(normalizedQuery, fromIndex);
      if (foundIndex === -1) break;

      matches.push({
        start: foundIndex,
        end: foundIndex + normalizedQuery.length
      });
      fromIndex = foundIndex + Math.max(normalizedQuery.length, 1);
    }

    return matches;
  }

  function renderHighlightedInput(): string {
    if (!jsonInput) return "";
    if (!searchMatches.length) return escapeHtml(jsonInput);

    let lastIndex = 0;
    let highlighted = "";

    searchMatches.forEach((match, index) => {
      highlighted += escapeHtml(jsonInput.slice(lastIndex, match.start));
      const matchText = escapeHtml(jsonInput.slice(match.start, match.end));
      const className = index === activeSearchIndex ? "search-highlight active" : "search-highlight";
      highlighted += `<mark class="${className}">${matchText}</mark>`;
      lastIndex = match.end;
    });

    highlighted += escapeHtml(jsonInput.slice(lastIndex));
    return highlighted;
  }

  async function jumpToSearchMatch(index: number) {
    const textarea = inputTextarea;
    const match = searchMatches[index];
    if (!textarea || !match) return;

    await tick();
    textarea.focus();
    textarea.setSelectionRange(match.start, match.end);
    textarea.scrollTop = getScrollTopForPosition(match.start);
    syncEditorScroll();
  }

  function navigateSearchMatch(direction: 1 | -1) {
    if (!searchMatches.length) return;
    const nextIndex = (activeSearchIndex + direction + searchMatches.length) % searchMatches.length;
    activeSearchIndex = nextIndex;
    void jumpToSearchMatch(nextIndex);
  }

  function handleSearchInput() {
    const matches = buildSearchMatches(jsonInput, searchQuery);
    searchMatches = matches;
    lastSearchQuery = searchQuery;

    if (!matches.length) {
      activeSearchIndex = -1;
      return;
    }

    activeSearchIndex = 0;
    void jumpToSearchMatch(0);
  }

  function handleSearchKeydown(e: KeyboardEvent) {
    if (e.key === "Enter") {
      e.preventDefault();
      navigateSearchMatch(e.shiftKey ? -1 : 1);
      return;
    }

    if (e.key === "ArrowDown") {
      e.preventDefault();
      navigateSearchMatch(1);
      return;
    }

    if (e.key === "ArrowUp") {
      e.preventDefault();
      navigateSearchMatch(-1);
    }
  }

  // 历史记录
  function addToHistory() {
    if (!jsonInput || jsonInput.length < 10) return;
    const preview = jsonInput.substring(0, 60).replace(/\n/g, ' ').trim();
    const item: HistoryItem = {
      id: Date.now().toString(),
      content: jsonInput,
      preview: preview + (jsonInput.length > 60 ? '...' : ''),
      timestamp: new Date(),
      size: new Blob([jsonInput]).size
    };
    if (history.find(h => h.content === jsonInput)) return;
    history = [item, ...history.slice(0, 19)];
  }

  function loadFromHistory(item: HistoryItem) {
    jsonInput = item.content;
    processJson();
  }

  function deleteHistory(id: string, e: Event) {
    e.stopPropagation();
    history = history.filter(h => h.id !== id);
  }

  function handleHistoryKeydown(e: KeyboardEvent, item: HistoryItem) {
    if (e.key !== 'Enter' && e.key !== ' ') return;
    e.preventDefault();
    loadFromHistory(item);
  }

  function clearHistory() { history = []; }

  // 折叠功能
  function toggleCollapse(path: string) {
    const newSet = new Set(collapsedPaths);
    if (newSet.has(path)) newSet.delete(path);
    else newSet.add(path);
    collapsedPaths = newSet;
  }

  function expandAll() { collapsedPaths = new Set(); }
  
  function collapseAll() {
    const paths = new Set<string>();
    function findPaths(obj: unknown, path: string) {
      if (obj === null || typeof obj !== 'object') return;
      paths.add(path);
      if (Array.isArray(obj)) {
        obj.forEach((item, i) => findPaths(item, `${path}[${i}]`));
      } else {
        for (const [k, v] of Object.entries(obj)) {
          findPaths(v, `${path}.${k}`);
        }
      }
    }
    if (parsedJson) findPaths(parsedJson, '$');
    collapsedPaths = paths;
  }

  function collapseToLevel(level: number) {
    const paths = new Set<string>();
    function findPaths(obj: unknown, path: string, currentLevel: number) {
      if (obj === null || typeof obj !== 'object') return;
      if (currentLevel >= level) paths.add(path);
      if (Array.isArray(obj)) {
        obj.forEach((item, i) => findPaths(item, `${path}[${i}]`, currentLevel + 1));
      } else {
        for (const [k, v] of Object.entries(obj)) {
          findPaths(v, `${path}.${k}`, currentLevel + 1);
        }
      }
    }
    if (parsedJson) findPaths(parsedJson, '$', 0);
    collapsedPaths = paths;
  }

  // 主要处理函数
  function cancelScheduledProcess() {
    if (!processTimer) return;
    clearTimeout(processTimer);
    processTimer = null;
  }

  function scheduleProcessJson(delay = 300) {
    cancelScheduledProcess();
    processTimer = setTimeout(() => {
      processTimer = null;
      processJson();
    }, delay);
  }

  function handleModeClick(mode: string) {
    activeMode = mode;
    cancelScheduledProcess();
    processJson();
  }

  function processJson() {
    cancelScheduledProcess();
    errorInfo = null;
    backendError = "";

    // diff：两侧文本对比，容错，无需 JSON.parse
    if (activeMode === "diff") {
      runDiff();
      isProcessing = false;
      return;
    }

    if (!jsonInput) {
      isProcessing = false;
      jsonOutput = "";
      structOutput = "";
      jsonStats = null;
      parsedJson = null;
      jsonPathResult = "";
      schemaResult = null;
      return;
    }

    // convert：源格式可能非 JSON，直接交后端
    if (activeMode === "convert") {
      void runConvert();
      return;
    }

    // 其余模式输入应为合法 JSON
    isProcessing = true;
    jsonOutput = "";
    structOutput = "";
    jsonStats = null;
    parsedJson = null;
    schemaResult = null;
    if (activeMode !== "query") {
      jsonPathResult = "";
    }

    let parsed: unknown;
    try {
      parsed = JSON.parse(jsonInput);
      parsedJson = parsed;
      addToHistory();
    } catch (e) {
      errorInfo = parseJsonError(jsonInput, e as Error);
      isProcessing = false;
      return;
    }

    if (showStats) {
      jsonStats = calculateStats(parsed);
      jsonStats.totalSize = new Blob([jsonInput]).size;
    }

    // 后端模式：schema / query（输入已确认为合法 JSON）
    if (activeMode === "schema") {
      void runSchema();
      return;
    }
    if (activeMode === "query") {
      void runQuery(parsed);
      return;
    }

    // 纯前端同步模式
    try {
      const indent = Number(indentSize) || 2;
      if (sortKeys && (activeMode === "format" || activeMode === "compress")) {
        parsed = sortJsonKeys(parsed);
      }
      if (activeMode === "format") {
        jsonOutput = JSON.stringify(parsed, null, indent);
      } else if (activeMode === "compress") {
        jsonOutput = JSON.stringify(parsed);
      } else if (activeMode === "struct") {
        structOutput = generateStruct(parsed, structLanguage);
      } else if (activeMode === "escape") {
        if (jsonInput.startsWith('"') && jsonInput.endsWith('"')) {
          try {
            const unescaped = JSON.parse(jsonInput);
            jsonOutput = typeof unescaped === "string" ? unescaped : JSON.stringify(unescaped, null, indent);
          } catch { jsonOutput = JSON.stringify(parsed, null, indent); }
        } else {
          jsonOutput = JSON.stringify(jsonInput);
        }
      } else if (activeMode === "validate") {
        jsonOutput = "✓ JSON 格式正确";
      }
    } catch (e: unknown) {
      errorInfo = { line: 0, column: 0, message: `处理错误: ${(e as Error).message || e}`, lineContent: '', expected: [], found: '' };
    } finally {
      isProcessing = false;
    }
  }

  async function runConvert() {
    isProcessing = true;
    backendError = "";
    jsonOutput = "";
    structOutput = "";
    parsedJson = null;
    try {
      const res = await apiJson<{ output: string }>("/api/json/convert", {
        input: jsonInput,
        from: convertFrom,
        to: convertTo,
        indent: Number(indentSize) || 2,
      });
      jsonOutput = res.output;
      if (convertFrom === "json") {
        try { parsedJson = JSON.parse(jsonInput); } catch { parsedJson = null; }
      }
    } catch (e) {
      backendError = (e as Error).message;
    } finally {
      isProcessing = false;
    }
  }

  async function runSchema() {
    backendError = "";
    schemaResult = null;
    jsonOutput = "";
    try {
      if (schemaMode === "generate") {
        const res = await apiJson<{ schema: string }>("/api/json/schema", {
          json: jsonInput,
          mode: "generate",
        });
        jsonOutput = res.schema ?? "";
      } else {
        const res = await apiJson<{ valid: boolean; errors: { path: string; message: string }[] }>(
          "/api/json/schema",
          { json: jsonInput, mode: "validate", schema: schemaInput },
        );
        schemaResult = { valid: res.valid, errors: res.errors ?? [] };
      }
    } catch (e) {
      backendError = (e as Error).message;
    } finally {
      isProcessing = false;
    }
  }

  async function runQuery(parsed: unknown) {
    backendError = "";
    const indent = Number(indentSize) || 2;
    jsonOutput = JSON.stringify(parsed, null, indent);
    if (!jsonPath.trim()) {
      jsonPathResult = "";
      isProcessing = false;
      return;
    }
    try {
      const res = await apiJson<{ result: string }>("/api/json/query", {
        json: jsonInput,
        engine: queryEngine,
        expr: jsonPath,
      });
      jsonPathResult = res.result;
    } catch (e) {
      backendError = (e as Error).message;
      jsonPathResult = "";
    } finally {
      isProcessing = false;
    }
  }

  function runDiff() {
    backendError = "";
    const norm = (s: string) => {
      if (!s.trim()) return "";
      try { return JSON.stringify(JSON.parse(s), null, Number(indentSize) || 2); }
      catch { return s; }
    };
    diffResult = diffLines(norm(jsonInput), norm(jsonInputB));
  }

  function downloadOutput() {
    const text = activeMode === "struct" ? structOutput : jsonOutput;
    if (!text) return;
    const ext =
      activeMode === "struct"
        ? structLanguages.find((l) => l.id === structLanguage)?.ext || "txt"
        : activeMode === "convert"
          ? convertTo
          : "json";
    downloadBlob(new Blob([text], { type: "text/plain;charset=utf-8" }), `output.${ext}`);
  }


  function clearAll() {
    cancelScheduledProcess();
    jsonInput = ""; jsonOutput = ""; structOutput = ""; errorInfo = null;
    jsonStats = null; searchQuery = ""; searchMatches = []; activeSearchIndex = -1; jsonPath = "";
    jsonPathResult = ""; parsedJson = null; collapsedPaths = new Set();
    jsonInputB = ""; schemaInput = ""; schemaResult = null; diffResult = null; backendError = "";
  }

  function loadExample() {
    jsonInput = `{
  "name": "开发工具箱",
  "version": "1.0.0",
  "description": "一款实用的开发者工具集合",
  "author": {
    "name": "Developer",
    "email": "dev@example.com",
    "url": "https://example.com"
  },
  "features": [
    "JSON格式化",
    "代码转换", 
    "加密解密",
    "时间转换"
  ],
  "settings": {
    "theme": "dark",
    "autoSave": true,
    "maxHistory": 20,
    "shortcuts": {
      "format": "Ctrl+Shift+F",
      "compress": "Ctrl+Shift+M"
    }
  },
  "stats": {
    "downloads": 10000,
    "stars": 500,
    "contributors": 25
  }
}`;
    processJson();
  }

  function handleTabKey(e: KeyboardEvent) {
    if (e.key === 'Tab') {
      e.preventDefault();
      const textarea = e.target as HTMLTextAreaElement;
      const start = textarea.selectionStart;
      const end = textarea.selectionEnd;
      const spaces = ' '.repeat(indentSize);
      jsonInput = jsonInput.substring(0, start) + spaces + jsonInput.substring(end);
      requestAnimationFrame(() => {
        textarea.selectionStart = textarea.selectionEnd = start + spaces.length;
      });
    }
  }

  function getScrollTopForPosition(position: number): number {
    const textarea = inputTextarea;
    if (!textarea) return 0;
    const contentBefore = jsonInput.slice(0, position);
    const lineIndex = contentBefore.split('\n').length - 1;
    const lineHeight = parseFloat(getComputedStyle(textarea).lineHeight) || 20;
    return Math.max(0, lineIndex * lineHeight - textarea.clientHeight / 2 + lineHeight / 2);
  }

  function syncEditorScroll() {
    if (!inputTextarea) return;
    if (lineNumbersEl) lineNumbersEl.scrollTop = inputTextarea.scrollTop;
    if (highlightLayerEl) {
      highlightLayerEl.scrollTop = inputTextarea.scrollTop;
      highlightLayerEl.scrollLeft = inputTextarea.scrollLeft;
    }
  }

  function jumpToErrorLine() {
    if (!errorInfo?.line) return;
    const textarea = inputTextarea;
    if (!textarea) return;
    const lines = jsonInput.split('\n');
    let position = 0;
    for (let i = 0; i < errorInfo.line - 1 && i < lines.length; i++) {
      position += lines[i].length + 1;
    }
    position += (errorInfo.column || 1) - 1;
    textarea.focus();
    textarea.setSelectionRange(position, position);
    textarea.scrollTop = getScrollTopForPosition(position);
    syncEditorScroll();
  }

  function formatTime(date: Date): string {
    const diff = Date.now() - date.getTime();
    if (diff < 60000) return '刚刚';
    if (diff < 3600000) return `${Math.floor(diff / 60000)}分钟前`;
    if (diff < 86400000) return `${Math.floor(diff / 3600000)}小时前`;
    return date.toLocaleDateString();
  }

  function formatSize(bytes: number): string {
    if (bytes < 1024) return bytes + ' B';
    if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB';
    return (bytes / (1024 * 1024)).toFixed(2) + ' MB';
  }

  function getErrorPointer(column: number): string {
    return '-'.repeat(Math.max(0, column - 1)) + '^';
  }

  function toggleFullscreen() {
    isFullscreen = !isFullscreen;
  }

  // 获取值的类型标签
  function getTypeLabel(value: unknown): string {
    if (value === null) return 'null';
    if (Array.isArray(value)) return `array[${value.length}]`;
    if (typeof value === 'object') return `object{${Object.keys(value).length}}`;
    return typeof value;
  }

  // 获取值的显示样式
  function getValueClass(value: unknown): string {
    if (value === null) return 'json-null';
    if (typeof value === 'string') return 'json-string';
    if (typeof value === 'number') return 'json-number';
    if (typeof value === 'boolean') return 'json-boolean';
    return '';
  }

  $effect(() => {
    if (jsonInput) {
      scheduleProcessJson();
      return () => cancelScheduledProcess();
    }
    processJson();
  });

  $effect(() => {
    // query 模式：表达式 / 引擎变化时重新查询（jsonInput 变化已由主 effect 处理）
    jsonPath;
    queryEngine;
    if (activeMode === "query") {
      untrack(() => scheduleProcessJson(250));
    }
  });

  $effect(() => {
    jsonInput;
    requestAnimationFrame(() => syncEditorScroll());
  });

  $effect(() => {
    const currentQuery = searchQuery;
    const matches = buildSearchMatches(jsonInput, currentQuery);
    searchMatches = matches;

    if (!matches.length) {
      activeSearchIndex = -1;
      lastSearchQuery = currentQuery;
      return;
    }

    const currentIndex = untrack(() => activeSearchIndex);
    const nextIndex = currentQuery !== lastSearchQuery
      ? 0
      : Math.min(Math.max(currentIndex, 0), matches.length - 1);
    const shouldJump = Boolean(currentQuery) && (currentQuery !== lastSearchQuery || currentIndex !== nextIndex);

    activeSearchIndex = nextIndex;

    if (shouldJump) {
      requestAnimationFrame(() => {
        void jumpToSearchMatch(nextIndex);
      });
    }

    lastSearchQuery = currentQuery;
  });
</script>

<div class="tool" class:fullscreen={isFullscreen}>
  <div class="toolbar">
    <Tabs
      value={activeMode}
      tabs={modes.map((mode) => ({ value: mode.id, label: mode.name, icon: mode.icon }))}
      ariaLabel="处理模式"
      onchange={handleModeClick}
    />
    <div class="spacer"></div>
    <Button size="sm" onclick={loadExample}>Demo</Button>
    <Button size="sm" variant="ghost" onclick={clearAll}>清空</Button>
    <Button size="sm" variant="ghost" icon onclick={toggleFullscreen} ariaLabel={isFullscreen ? "退出全屏" : "全屏"}>
      {isFullscreen ? "⤓" : "⤢"}
    </Button>
  </div>

  <div class="options">
    <div class="search">
      <label for={searchInputId}>搜索</label>
      <input
        id={searchInputId}
        type="text"
        placeholder="输入关键字后定位"
        bind:value={searchQuery}
        oninput={handleSearchInput}
        onkeydown={handleSearchKeydown}
      />
      <span class="hits">
        {searchMatches.length > 0 ? `${activeSearchIndex + 1}/${searchMatches.length}` : searchQuery.trim() ? "0/0" : ""}
      </span>
      <button class="nav" onclick={() => navigateSearchMatch(-1)} disabled={searchMatches.length <= 1} aria-label="上一个匹配">↑</button>
      <button class="nav" onclick={() => navigateSearchMatch(1)} disabled={searchMatches.length <= 1} aria-label="下一个匹配">↓</button>
    </div>

    <label class="opt">
      <span>缩进</span>
      <select id={indentSelectId} bind:value={indentSize} onchange={processJson}>
        <option value={2}>2 空格</option>
        <option value={4}>4 空格</option>
      </select>
    </label>

    {#if activeMode === "struct"}
      <label class="opt">
        <span>语言</span>
        <select id={structLanguageSelectId} bind:value={structLanguage} onchange={processJson}>
          {#each structLanguages as lang (lang.id)}
            <option value={lang.id}>{lang.name}</option>
          {/each}
        </select>
      </label>
    {/if}

    {#if activeMode === "convert"}
      <label class="opt">
        <span>从</span>
        <select bind:value={convertFrom} onchange={processJson}>
          {#each dataFormats as f (f.id)}<option value={f.id}>{f.name}</option>{/each}
        </select>
      </label>
      <label class="opt">
        <span>到</span>
        <select bind:value={convertTo} onchange={processJson}>
          {#each dataFormats as f (f.id)}<option value={f.id}>{f.name}</option>{/each}
        </select>
      </label>
    {/if}

    {#if activeMode === "schema"}
      <Tabs
        value={schemaMode}
        size="sm"
        tabs={[
          { value: "generate", label: "生成 Schema" },
          { value: "validate", label: "校验数据" },
        ]}
        ariaLabel="Schema 模式"
        onchange={(value) => {
          schemaMode = value as typeof schemaMode;
          processJson();
        }}
      />
    {/if}

    {#if activeMode === "query"}
      <Tabs
        bind:value={queryEngine}
        size="sm"
        tabs={[
          { value: "jsonpath", label: "JSONPath" },
          { value: "jq", label: "jq" },
        ]}
        ariaLabel="查询引擎"
      />
    {/if}

    <label class="check">
      <input type="checkbox" bind:checked={sortKeys} onchange={processJson} />
      排序键名
    </label>
    <label class="check">
      <input type="checkbox" bind:checked={showStats} onchange={processJson} />
      统计
    </label>

    <div class="spacer"></div>

    <Tabs
      bind:value={viewMode}
      size="sm"
      tabs={[
        { value: "editor", label: "编辑器" },
        { value: "tree", label: "树形" },
      ]}
      ariaLabel="视图"
    />

    {#if viewMode === "tree" && parsedJson}
      <div class="collapse">
        <Button size="sm" onclick={expandAll}>展开全部</Button>
        <Button size="sm" onclick={collapseAll}>折叠全部</Button>
        <Button size="sm" onclick={() => collapseToLevel(1)}>折 1 级</Button>
        <Button size="sm" onclick={() => collapseToLevel(2)}>折 2 级</Button>
      </div>
    {/if}
  </div>

  {#if activeMode === "query"}
    <div class="query-bar">
      <label for={jsonPathInputId}>{queryEngine === "jq" ? "jq" : "JSONPath"}</label>
      <input
        id={jsonPathInputId}
        type="text"
        placeholder={queryEngine === "jq" ? ".data.items | map(.id) | add" : "$.data.items[*].id"}
        bind:value={jsonPath}
        oninput={() => scheduleProcessJson(250)}
      />
    </div>
  {/if}

  {#if showStats && jsonStats}
    <div class="stats">
      <span><b>{jsonStats.totalKeys}</b> 键</span>
      <span><b>{jsonStats.maxDepth}</b> 层</span>
      <span><b>{jsonStats.objectCount}</b> 对象</span>
      <span><b>{jsonStats.arrayCount}</b> 数组</span>
      <span><b>{formatSize(jsonStats.totalSize)}</b></span>
    </div>
  {/if}

  <div class="main">
    {#if viewMode === "editor"}
      <div
        class="editors"
        class:dragging={draggingSplit}
        style="--split-ratio: {splitRatio}%"
        bind:this={editorsContainerEl}
      >
        <div class="pane">
          <div class="pane-head">
            <span class="pane-title">JSON 输入</span>
            <span class="pane-info">
              {jsonInput.length.toLocaleString()} 字符 · {jsonInput.split("\n").length} 行
            </span>
          </div>
          <div class="pane-body">
            <div class="lines" bind:this={lineNumbersEl}>
              {#each jsonInput.split("\n") as _, index (index)}
                <div class="line" class:err={errorInfo?.line === index + 1}>{index + 1}</div>
              {/each}
            </div>
            <div class="stack">
              <div class="highlight" bind:this={highlightLayerEl} aria-hidden="true">
                <div class="highlight-inner">{@html renderHighlightedInput()}</div>
              </div>
              <textarea
                bind:value={jsonInput}
                bind:this={inputTextarea}
                placeholder="在此粘贴或输入 JSON 数据…"
                class="code"
                spellcheck="false"
                onkeydown={handleTabKey}
                onscroll={syncEditorScroll}
              ></textarea>
            </div>
          </div>
        </div>

        <button
          class="resizer"
          type="button"
          role="slider"
          aria-label="拖动调整两栏宽度"
          aria-valuemin={20}
          aria-valuemax={80}
          aria-valuenow={Math.round(splitRatio)}
          onpointerdown={startSplitDrag}
          onpointermove={onSplitDrag}
          onpointerup={endSplitDrag}
          onpointercancel={endSplitDrag}
          onkeydown={onSplitKeydown}
        ></button>

        <div class="pane">
          {#if rightPanelRole === "inputB"}
            <div class="pane-head">
              <span class="pane-title">对比 B</span>
              <span class="pane-info">
                {jsonInputB.length.toLocaleString()} 字符 · {jsonInputB.split("\n").length} 行
              </span>
            </div>
            <div class="pane-body">
              <textarea
                bind:value={jsonInputB}
                placeholder="在此粘贴用于对比的第二份内容（JSON 会自动规范化后对比）"
                class="code plain"
                spellcheck="false"
                oninput={runDiff}
              ></textarea>
            </div>
          {:else if rightPanelRole === "schemaInput"}
            <div class="pane-head">
              <span class="pane-title">JSON Schema</span>
              <span class="pane-info">{schemaInput.length.toLocaleString()} 字符</span>
            </div>
            <div class="pane-body">
              <textarea
                bind:value={schemaInput}
                placeholder={'粘贴 JSON Schema，例如 {"type":"object","required":["name"]}'}
                class="code plain"
                spellcheck="false"
                oninput={() => scheduleProcessJson(300)}
              ></textarea>
            </div>
          {:else}
            <div class="pane-head">
              <span class="pane-title">
                {activeMode === "struct"
                  ? `${structLanguages.find((l) => l.id === structLanguage)?.name ?? ""} 输出`
                  : activeMode === "convert"
                    ? `${convertTo.toUpperCase()} 输出`
                    : activeMode === "schema"
                      ? "Schema 输出"
                      : "JSON 输出"}
              </span>
              <span class="pane-info">{outputContent.length.toLocaleString()} 字符</span>
              <CopyButton value={() => outputContent} disabled={!outputContent} />
              {#if outputContent && (activeMode === "struct" || activeMode === "convert" || (activeMode === "schema" && schemaMode === "generate"))}
                <Button size="sm" variant="ghost" onclick={downloadOutput}>下载</Button>
              {/if}
            </div>
            <div class="pane-body">
              <textarea
                value={outputContent}
                placeholder={isProcessing ? "处理中…" : "结果将显示在这里…"}
                class="code plain"
                class:ok={activeMode === "validate" && jsonOutput.includes("✓")}
                readonly
                spellcheck="false"
              ></textarea>
            </div>
          {/if}
        </div>
      </div>
    {:else if viewMode === "tree" && parsedJson}
      <div class="tree">
        {#snippet renderNode(value: unknown, key: string, path: string, depth: number)}
          {@const isCollapsed = collapsedPaths.has(path)}
          {@const isExpandable = value !== null && typeof value === "object"}

          <div class="node" style="--depth: {depth}">
            <div class="node-line">
              {#if isExpandable}
                <button class="expand" onclick={() => toggleCollapse(path)} aria-label={isCollapsed ? "展开" : "折叠"}>
                  {isCollapsed ? "▶" : "▼"}
                </button>
              {:else}
                <span class="expand-space"></span>
              {/if}

              <span class="key">{key}</span><span class="colon">:</span>

              {#if isExpandable}
                <span class="type">{getTypeLabel(value)}</span>
                {#if isCollapsed}
                  <span class="folded">{Array.isArray(value) ? "[ … ]" : "{ … }"}</span>
                {/if}
              {:else}
                <span class={getValueClass(value)}>
                  {typeof value === "string" ? `"${value}"` : JSON.stringify(value)}
                </span>
              {/if}

              <span class="node-copy"><CopyButton value={JSON.stringify(value)} icon title="复制值" /></span>
            </div>

            {#if isExpandable && !isCollapsed}
              <div class="children">
                {#if Array.isArray(value)}
                  {#each value as item, index (index)}
                    {@render renderNode(item, `[${index}]`, `${path}[${index}]`, depth + 1)}
                  {/each}
                {:else if typeof value === "object" && value !== null}
                  {#each Object.entries(value) as [childKey, childValue] (childKey)}
                    {@render renderNode(childValue, childKey, `${path}.${childKey}`, depth + 1)}
                  {/each}
                {/if}
              </div>
            {/if}
          </div>
        {/snippet}

        {@render renderNode(parsedJson, "root", "$", 0)}
      </div>
    {:else}
      <EmptyState icon={"{ }"} title="请先输入有效的 JSON 数据" hint="树形视图需要可解析的 JSON" />
    {/if}
  </div>

  {#if errorInfo}
    <div class="err-panel">
      <div class="err-head">
        <span class="err-badge">JSON 格式错误</span>
        <span class="err-loc">第 {errorInfo.line} 行，第 {errorInfo.column} 列</span>
        <Button size="sm" onclick={jumpToErrorLine}>跳转到错误</Button>
      </div>
      <p class="err-msg">{errorInfo.message}</p>
      {#if errorInfo.lineContent}
        <div class="err-code">
          <span class="err-no">{errorInfo.line}</span>
          <div>
            <code>{errorInfo.lineContent}</code>
            <div class="pointer">{getErrorPointer(errorInfo.column)}</div>
          </div>
        </div>
      {/if}
      {#if errorInfo.expected.length > 0}
        <p class="err-expect">
          期望
          {#each errorInfo.expected as exp, index (exp)}
            <code class="good">{exp}</code>{index < errorInfo.expected.length - 1 ? "、" : ""}
          {/each}
          ，实际 <code class="bad">{errorInfo.found}</code>
        </p>
      {/if}
    </div>
  {/if}

  {#if backendError}
    <p class="backend-err">{backendError}</p>
  {/if}

  {#if activeMode === "diff" && diffResult}
    <DiffView result={diffResult} leftTitle="A" rightTitle="B" maxHeight="38vh" />
  {/if}

  {#if activeMode === "schema" && schemaMode === "validate" && schemaResult}
    <div class="schema">
      {#if schemaResult.valid}
        <p class="schema-ok">✓ 数据符合 Schema</p>
      {:else}
        <p class="schema-fail">✕ 共 {schemaResult.errors.length} 处不符合</p>
        <ul>
          {#each schemaResult.errors as err, index (index)}
            <li><code>{err.path}</code><span>{err.message}</span></li>
          {/each}
        </ul>
      {/if}
    </div>
  {/if}

  {#if activeMode === "query" && jsonPathResult}
    <div class="query-result">
      <div class="qr-head">
        <span>查询结果</span>
        <CopyButton value={() => jsonPathResult} />
      </div>
      <pre>{jsonPathResult}</pre>
    </div>
  {/if}

  {#if history.length > 0}
    <details class="history">
      <summary>历史记录（{history.length}/20）</summary>
      <div class="hist-body">
        <div class="hist-actions">
          <Button size="sm" variant="ghost" onclick={clearHistory}>清空历史</Button>
        </div>
        <ul>
          {#each history as item (item.id)}
            <li>
              <button class="hist-item" onclick={() => loadFromHistory(item)}>
                <span class="hist-preview">{item.preview}</span>
                <span class="hist-meta">{formatTime(item.timestamp)} · {formatSize(item.size)}</span>
              </button>
              <button class="hist-del" onclick={(e) => deleteHistory(item.id, e)} aria-label="删除这条历史">×</button>
            </li>
          {/each}
        </ul>
      </div>
    </details>
  {/if}
</div>

<style>
  .tool {
    display: flex;
    flex-direction: column;
    gap: var(--sp-2);
    flex: 1;
    min-height: 0;
  }

  .tool.fullscreen {
    position: fixed;
    inset: 0;
    z-index: var(--z-overlay);
    padding: var(--sp-4);
    background: var(--color-bg);
  }

  .toolbar,
  .options {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: var(--sp-2);
  }

  .spacer {
    flex: 1;
  }

  .search {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    padding: 0 var(--sp-2);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    background: var(--color-bg);
  }

  .search label {
    font-size: var(--fs-xs);
    color: var(--color-text-muted);
  }

  .search input {
    width: 11rem;
    background: none;
    border: none;
    outline: none;
    color: var(--color-text);
    padding: 0.3rem 0;
    font-size: var(--fs-sm);
  }

  .hits {
    font-family: var(--font-mono);
    font-size: var(--fs-xs);
    color: var(--color-text-muted);
    min-width: 2.4rem;
    text-align: right;
  }

  .nav {
    width: 20px;
    height: 20px;
    border-radius: var(--radius-sm);
    color: var(--color-text-muted);
    font-size: var(--fs-xs);
  }

  .nav:hover:not(:disabled) {
    background: var(--color-surface-2);
    color: var(--color-text);
  }

  .nav:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .opt,
  .check {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    font-size: var(--fs-sm);
    color: var(--color-text-secondary);
    white-space: nowrap;
  }

  .check input {
    accent-color: var(--color-primary);
  }

  select {
    background: var(--color-bg);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    color: var(--color-text);
    padding: 0.25rem var(--sp-2);
    font-size: var(--fs-sm);
  }

  .collapse {
    display: flex;
    gap: var(--sp-1);
  }

  .query-bar {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    padding: var(--sp-2) var(--sp-3);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    background: var(--color-surface);
  }

  .query-bar label {
    font-family: var(--font-mono);
    font-size: var(--fs-sm);
    color: var(--color-accent);
  }

  .query-bar input {
    flex: 1;
    background: var(--color-bg);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    color: var(--color-text);
    padding: 0.3rem var(--sp-2);
    font-family: var(--font-mono);
    font-size: var(--fs-sm);
  }

  .stats {
    display: flex;
    flex-wrap: wrap;
    gap: var(--sp-2);
  }

  .stats span {
    padding: 0.15rem var(--sp-2);
    border-radius: var(--radius-full);
    background: var(--color-surface-2);
    font-size: var(--fs-xs);
    color: var(--color-text-muted);
  }

  .stats b {
    font-family: var(--font-mono);
    color: var(--color-text);
  }

  .main {
    flex: 1;
    min-height: 320px;
    display: flex;
  }

  .editors {
    flex: 1;
    display: grid;
    grid-template-columns: var(--split-ratio) auto 1fr;
    gap: var(--sp-2);
    min-height: 0;
  }

  .pane {
    display: flex;
    flex-direction: column;
    min-width: 0;
    min-height: 0;
    border: 1px solid var(--color-border);
    border-radius: var(--radius);
    background: var(--color-surface);
    overflow: hidden;
  }

  .pane-head {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    height: 36px;
    padding: 0 var(--sp-3);
    border-bottom: 1px solid var(--color-border);
    flex-shrink: 0;
  }

  .pane-title {
    font-size: var(--fs-sm);
    font-weight: var(--fw-semibold);
    color: var(--color-text);
    white-space: nowrap;
  }

  .pane-info {
    flex: 1;
    font-family: var(--font-mono);
    font-size: var(--fs-xs);
    color: var(--color-text-muted);
    text-align: right;
    white-space: nowrap;
    overflow: hidden;
  }

  .pane-body {
    flex: 1;
    display: flex;
    min-height: 0;
    background: var(--color-code-bg);
  }

  .lines {
    flex-shrink: 0;
    padding: var(--sp-2) var(--sp-1) var(--sp-2) var(--sp-2);
    border-right: 1px solid var(--color-border);
    background: var(--color-surface-2);
    font-family: var(--font-mono);
    font-size: var(--fs-sm);
    line-height: 1.6;
    color: var(--color-text-muted);
    text-align: right;
    overflow: hidden;
    user-select: none;
  }

  .line.err {
    color: var(--color-danger);
    font-weight: var(--fw-bold);
  }

  .stack {
    position: relative;
    flex: 1;
    min-width: 0;
  }

  .highlight {
    position: absolute;
    inset: 0;
    overflow: hidden;
    pointer-events: none;
  }

  .highlight-inner,
  .code {
    padding: var(--sp-2);
    font-family: var(--font-mono);
    font-size: var(--fs-sm);
    line-height: 1.6;
    white-space: pre-wrap;
    word-break: break-word;
  }

  .code {
    position: relative;
    width: 100%;
    height: 100%;
    background: transparent;
    border: none;
    outline: none;
    resize: none;
    color: var(--color-text);
    caret-color: var(--color-primary);
  }

  .code.plain {
    position: static;
  }

  .code.ok {
    color: var(--color-success);
  }

  .code::placeholder {
    color: var(--color-text-muted);
  }

  /* @html 注入的搜索高亮不受 scoped 样式影响 */
  .highlight-inner :global(mark) {
    background: var(--color-highlight-bg);
    color: transparent;
    border-radius: 2px;
  }

  .highlight-inner :global(mark.active) {
    background: var(--color-primary);
  }

  .resizer {
    width: 5px;
    border-radius: var(--radius-full);
    background: transparent;
    cursor: col-resize;
    transition: background var(--motion);
  }

  .resizer:hover,
  .editors.dragging .resizer {
    background: var(--color-primary);
  }

  .tree {
    flex: 1;
    min-height: 0;
    overflow: auto;
    padding: var(--sp-3);
    border: 1px solid var(--color-border);
    border-radius: var(--radius);
    background: var(--color-code-bg);
    font-family: var(--font-mono);
    font-size: var(--fs-sm);
  }

  .node-line {
    display: flex;
    align-items: center;
    gap: var(--sp-1);
    padding: 1px 0;
    padding-left: calc(var(--depth) * 1.1rem);
    border-radius: var(--radius-sm);
  }

  .node-line:hover {
    background: var(--color-surface-2);
  }

  .expand,
  .expand-space {
    width: 14px;
    flex-shrink: 0;
    font-size: 0.6rem;
    color: var(--color-text-muted);
  }

  .key {
    color: var(--color-accent);
  }

  .colon {
    color: var(--color-text-muted);
    margin-right: var(--sp-1);
  }

  .type {
    font-size: var(--fs-xs);
    color: var(--color-text-muted);
  }

  .folded {
    color: var(--color-text-muted);
  }

  .node-copy {
    opacity: 0;
    transition: opacity var(--motion);
  }

  .node-line:hover .node-copy {
    opacity: 1;
  }

  .err-panel {
    padding: var(--sp-3);
    border: 1px solid var(--color-danger-border);
    border-radius: var(--radius);
    background: var(--color-danger-bg);
  }

  .err-head {
    display: flex;
    align-items: center;
    gap: var(--sp-3);
  }

  .err-badge {
    font-size: var(--fs-sm);
    font-weight: var(--fw-semibold);
    color: var(--color-danger);
  }

  .err-loc {
    flex: 1;
    font-family: var(--font-mono);
    font-size: var(--fs-xs);
    color: var(--color-text-secondary);
  }

  .err-msg {
    margin-top: var(--sp-2);
    font-size: var(--fs-sm);
    color: var(--color-text-secondary);
  }

  .err-code {
    display: flex;
    gap: var(--sp-2);
    margin-top: var(--sp-2);
    padding: var(--sp-2);
    border-radius: var(--radius-sm);
    background: var(--color-code-bg);
    font-family: var(--font-mono);
    font-size: var(--fs-sm);
  }

  .err-no {
    color: var(--color-text-muted);
  }

  .err-code code {
    color: var(--color-text);
    white-space: pre-wrap;
    word-break: break-all;
  }

  .pointer {
    color: var(--color-danger);
    white-space: pre;
  }

  .err-expect {
    margin-top: var(--sp-2);
    font-size: var(--fs-xs);
    color: var(--color-text-secondary);
  }

  .err-expect code {
    font-family: var(--font-mono);
    padding: 0 0.25rem;
    border-radius: 3px;
  }

  .err-expect .good {
    color: var(--color-success);
    background: var(--color-success-bg);
  }

  .err-expect .bad {
    color: var(--color-danger);
    background: var(--color-danger-bg);
  }

  .backend-err {
    padding: var(--sp-2) var(--sp-3);
    border: 1px solid var(--color-warning-border);
    border-radius: var(--radius-sm);
    background: var(--color-warning-bg);
    color: var(--color-text-secondary);
    font-size: var(--fs-sm);
  }

  .schema {
    padding: var(--sp-3);
    border: 1px solid var(--color-border);
    border-radius: var(--radius);
    background: var(--color-surface);
  }

  .schema-ok {
    color: var(--color-success);
    font-size: var(--fs-sm);
  }

  .schema-fail {
    color: var(--color-danger);
    font-size: var(--fs-sm);
    margin-bottom: var(--sp-2);
  }

  .schema ul {
    list-style: none;
    display: flex;
    flex-direction: column;
    gap: var(--sp-1);
    max-height: 220px;
    overflow-y: auto;
  }

  .schema li {
    display: flex;
    gap: var(--sp-3);
    padding: var(--sp-1) var(--sp-2);
    border-radius: var(--radius-sm);
    background: var(--color-surface-2);
    font-size: var(--fs-sm);
  }

  .schema code {
    font-family: var(--font-mono);
    color: var(--color-accent);
    flex-shrink: 0;
  }

  .schema span {
    color: var(--color-text-secondary);
  }

  .query-result {
    border: 1px solid var(--color-border);
    border-radius: var(--radius);
    background: var(--color-surface);
    overflow: hidden;
  }

  .qr-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--sp-1) var(--sp-3);
    border-bottom: 1px solid var(--color-border);
    font-size: var(--fs-sm);
    color: var(--color-text-secondary);
  }

  .query-result pre {
    margin: 0;
    padding: var(--sp-3);
    max-height: 240px;
    overflow: auto;
    background: var(--color-code-bg);
    font-family: var(--font-mono);
    font-size: var(--fs-sm);
    color: var(--color-text-secondary);
  }

  .history {
    border: 1px solid var(--color-border);
    border-radius: var(--radius);
    background: var(--color-surface);
  }

  .history summary {
    padding: var(--sp-2) var(--sp-3);
    cursor: pointer;
    font-size: var(--fs-sm);
    color: var(--color-text-secondary);
  }

  .hist-body {
    padding: 0 var(--sp-3) var(--sp-3);
  }

  .hist-actions {
    display: flex;
    justify-content: flex-end;
    margin-bottom: var(--sp-2);
  }

  .history ul {
    list-style: none;
    display: flex;
    flex-direction: column;
    gap: var(--sp-1);
    max-height: 220px;
    overflow-y: auto;
  }

  .history li {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
  }

  .hist-item {
    flex: 1;
    min-width: 0;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--sp-3);
    padding: var(--sp-1) var(--sp-2);
    border-radius: var(--radius-sm);
    background: var(--color-surface-2);
    text-align: left;
  }

  .hist-item:hover {
    background: var(--color-surface-3);
  }

  .hist-preview {
    flex: 1;
    min-width: 0;
    font-family: var(--font-mono);
    font-size: var(--fs-xs);
    color: var(--color-text-secondary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .hist-meta {
    font-size: var(--fs-xs);
    color: var(--color-text-muted);
    white-space: nowrap;
  }

  .hist-del {
    color: var(--color-text-muted);
    padding: 0 var(--sp-1);
  }

  .hist-del:hover {
    color: var(--color-danger);
  }
</style>
