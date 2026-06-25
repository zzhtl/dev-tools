<script lang="ts">
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
  let copied = $state<string | null>(null);
  let indentSize = $state(2);
  let sortKeys = $state(false);
  let showStats = $state(false);
  let jsonStats = $state<JsonStats | null>(null);
  let searchQuery = $state("");
  let searchMatches = $state<SearchMatch[]>([]);
  let activeSearchIndex = $state(-1);
  let jsonPath = $state("");
  let jsonPathResult = $state("");
  let history = $state<HistoryItem[]>([]);
  let structLanguage = $state("go");
  let viewMode = $state<"editor" | "tree">("editor");
  let collapsedPaths = $state<Set<string>>(new Set());
  let parsedJson = $state<unknown>(null);
  let isFullscreen = $state(false);
  let splitRatio = $state(50); // 左右分割比例
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

  function escapeHtml(value: string): string {
    return value
      .replaceAll("&", "&amp;")
      .replaceAll("<", "&lt;")
      .replaceAll(">", "&gt;")
      .replaceAll('"', "&quot;")
      .replaceAll("'", "&#39;");
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

  async function copyToClipboard(text: string, type: string) {
    if (!text) return;
    try {
      await navigator.clipboard.writeText(text);
      copied = type;
      setTimeout(() => copied = null, 1500);
    } catch (err) { console.error("复制失败:", err); }
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

<div class="json-tool" class:fullscreen={isFullscreen}>
  <!-- 工具栏 -->
  <div class="toolbar">
    <div class="toolbar-left">
      {#each modes as mode}
        <button class="mode-btn" class:active={activeMode === mode.id} onclick={() => handleModeClick(mode.id)}>
          <span class="mode-icon">{mode.icon}</span>
          <span class="mode-name">{mode.name}</span>
    </button>
      {/each}
    </div>
    <div class="toolbar-right">
      <button class="toolbar-btn" onclick={loadExample}>📋 Demo</button>
      <button class="toolbar-btn" onclick={clearAll}>🗑️ 清空</button>
      <button class="toolbar-btn" onclick={toggleFullscreen} title={isFullscreen ? '退出全屏' : '全屏'}>
        {isFullscreen ? '⤓' : '⤢'}
    </button>
    </div>
  </div>

  <!-- 选项栏 -->
  <div class="options-bar">
    <div class="search-box">
      <label class="search-label" for={searchInputId}>搜索:</label>
      <input
        id={searchInputId}
        type="text"
        placeholder="输入关键字后定位"
        bind:value={searchQuery}
        oninput={handleSearchInput}
        onkeydown={handleSearchKeydown}
      />
      <span class="search-status">{searchMatches.length > 0 ? `${activeSearchIndex + 1}/${searchMatches.length}` : (searchQuery.trim() ? '0/0' : '')}</span>
      <button type="button" class="search-nav-btn" onclick={() => navigateSearchMatch(-1)} disabled={searchMatches.length <= 1} aria-label="上一个匹配">↑</button>
      <button type="button" class="search-nav-btn" onclick={() => navigateSearchMatch(1)} disabled={searchMatches.length <= 1} aria-label="下一个匹配">↓</button>
    </div>
    <div class="option-group">
      <label for={indentSelectId}>缩进:</label>
      <select id={indentSelectId} bind:value={indentSize} onchange={processJson}>
        <option value={2}>2空格</option>
        <option value={4}>4空格</option>
      </select>
    </div>
    {#if activeMode === 'struct'}
      <div class="option-group">
        <label for={structLanguageSelectId}>语言:</label>
        <select id={structLanguageSelectId} bind:value={structLanguage} onchange={processJson}>
          {#each structLanguages as lang}
            <option value={lang.id}>{lang.name}</option>
          {/each}
        </select>
      </div>
    {/if}
    {#if activeMode === 'convert'}
      <div class="option-group">
        <label for="json-convert-from">从:</label>
        <select id="json-convert-from" bind:value={convertFrom} onchange={processJson}>
          {#each dataFormats as f}<option value={f.id}>{f.name}</option>{/each}
        </select>
      </div>
      <div class="option-group">
        <label for="json-convert-to">到:</label>
        <select id="json-convert-to" bind:value={convertTo} onchange={processJson}>
          {#each dataFormats as f}<option value={f.id}>{f.name}</option>{/each}
        </select>
      </div>
    {/if}
    {#if activeMode === 'schema'}
      <div class="view-toggle">
        <button class="view-btn" class:active={schemaMode === 'generate'} onclick={() => { schemaMode = 'generate'; processJson(); }}>生成 Schema</button>
        <button class="view-btn" class:active={schemaMode === 'validate'} onclick={() => { schemaMode = 'validate'; processJson(); }}>校验数据</button>
      </div>
    {/if}
    {#if activeMode === 'query'}
      <div class="view-toggle">
        <button class="view-btn" class:active={queryEngine === 'jsonpath'} onclick={() => queryEngine = 'jsonpath'}>JSONPath</button>
        <button class="view-btn" class:active={queryEngine === 'jq'} onclick={() => queryEngine = 'jq'}>jq</button>
      </div>
    {/if}
    <label class="checkbox-option">
      <input type="checkbox" bind:checked={sortKeys} onchange={processJson} />
      <span>排序键名</span>
    </label>
    <label class="checkbox-option">
      <input type="checkbox" bind:checked={showStats} onchange={processJson} />
      <span>统计</span>
    </label>
    <div class="view-toggle">
      <button class="view-btn" class:active={viewMode === 'editor'} onclick={() => viewMode = 'editor'}>📝 编辑器</button>
      <button class="view-btn" class:active={viewMode === 'tree'} onclick={() => viewMode = 'tree'} disabled={!parsedJson}>🌲 树形</button>
    </div>
    {#if viewMode === 'tree' && parsedJson}
      <div class="collapse-controls">
        <button class="collapse-btn" onclick={expandAll}>展开全部</button>
        <button class="collapse-btn" onclick={collapseAll}>折叠全部</button>
        <button class="collapse-btn" onclick={() => collapseToLevel(1)}>折叠1级</button>
        <button class="collapse-btn" onclick={() => collapseToLevel(2)}>折叠2级</button>
      </div>
    {/if}
  </div>

  <!-- 查询表达式 -->
  {#if activeMode === 'query'}
    <div class="jsonpath-bar">
      <label for={jsonPathInputId}>{queryEngine === 'jq' ? 'jq' : 'JSONPath'}:</label>
      <input
        id={jsonPathInputId}
        type="text"
        placeholder={queryEngine === 'jq' ? '.data.items | map(.id) | add' : '$.data.items[*].id'}
        bind:value={jsonPath}
        oninput={() => scheduleProcessJson(250)}
      />
    </div>
  {/if}

  <!-- 统计栏 -->
  {#if showStats && jsonStats}
    <div class="stats-bar">
      <span class="stat-chip"><b>{jsonStats.totalKeys}</b> 键</span>
      <span class="stat-chip"><b>{jsonStats.maxDepth}</b> 层</span>
      <span class="stat-chip"><b>{jsonStats.objectCount}</b> 对象</span>
      <span class="stat-chip"><b>{jsonStats.arrayCount}</b> 数组</span>
      <span class="stat-chip"><b>{formatSize(jsonStats.totalSize)}</b></span>
    </div>
  {/if}

  <!-- 主编辑区 -->
  <div class="main-editor">
    {#if viewMode === 'editor'}
      <div class="editors-container" style="--split-ratio: {splitRatio}%">
        <!-- 输入面板 -->
        <div class="editor-panel input-panel">
          <div class="editor-header">
            <span class="editor-title">JSON 输入</span>
            <span class="editor-info">{jsonInput.length.toLocaleString()} 字符 · {jsonInput.split('\n').length} 行</span>
          </div>
          <div class="editor-body">
            <div class="line-numbers" bind:this={lineNumbersEl}>
              {#each jsonInput.split('\n') as _, i}
                <div class="line-num" class:error-line={errorInfo?.line === i + 1}>{i + 1}</div>
              {/each}
            </div>
            <div class="input-editor-stack">
              <div class="highlight-layer" bind:this={highlightLayerEl} aria-hidden="true">
                <div class="highlight-content">{@html renderHighlightedInput()}</div>
              </div>
              <textarea
                bind:value={jsonInput}
                bind:this={inputTextarea}
                placeholder="在此粘贴或输入JSON数据...&#10;&#10;支持大型JSON文件，可使用 Tab 键缩进"
                class="code-textarea input-textarea"
                spellcheck="false"
                onkeydown={handleTabKey}
                onscroll={syncEditorScroll}
              ></textarea>
            </div>
          </div>
        </div>

        <!-- 分隔条 -->
        <div class="resizer" role="separator" aria-orientation="vertical"></div>

        <!-- 输出面板 / 第二输入 -->
        <div class="editor-panel output-panel">
          {#if rightPanelRole === 'inputB'}
            <div class="editor-header">
              <span class="editor-title">对比 B</span>
              <span class="editor-info">{jsonInputB.length.toLocaleString()} 字符 · {jsonInputB.split('\n').length} 行</span>
            </div>
            <div class="editor-body">
              <textarea
                bind:value={jsonInputB}
                placeholder="在此粘贴用于对比的第二份内容（JSON 会自动规范化后对比）"
                class="code-textarea input-textarea"
                spellcheck="false"
                oninput={runDiff}
              ></textarea>
            </div>
          {:else if rightPanelRole === 'schemaInput'}
            <div class="editor-header">
              <span class="editor-title">JSON Schema</span>
              <span class="editor-info">{schemaInput.length.toLocaleString()} 字符</span>
            </div>
            <div class="editor-body">
              <textarea
                bind:value={schemaInput}
                placeholder={'粘贴 JSON Schema，例如 {"type":"object","required":["name"]}'}
                class="code-textarea input-textarea"
                spellcheck="false"
                oninput={() => scheduleProcessJson(300)}
              ></textarea>
            </div>
          {:else}
            <div class="editor-header">
              <span class="editor-title">
                {activeMode === 'struct'
                  ? `${structLanguages.find(l => l.id === structLanguage)?.name || ''} 输出`
                  : activeMode === 'convert'
                    ? `${convertTo.toUpperCase()} 输出`
                    : activeMode === 'schema'
                      ? 'Schema 输出'
                      : 'JSON 输出'}
              </span>
              <div class="editor-actions">
                <span class="editor-info">{outputContent.length.toLocaleString()} 字符</span>
                {#if outputContent}
                  <button class="copy-btn" onclick={() => copyToClipboard(outputContent, 'output')}>
                    {copied === 'output' ? '✓ 已复制' : '📋 复制'}
                  </button>
                  {#if activeMode === 'struct' || activeMode === 'convert' || (activeMode === 'schema' && schemaMode === 'generate')}
                    <button class="copy-btn" onclick={downloadOutput}>⬇ 下载</button>
                  {/if}
                {/if}
              </div>
            </div>
            <div class="editor-body output-body">
              <textarea
                value={outputContent}
                placeholder={isProcessing ? '处理中...' : '结果将显示在这里...'}
                class="code-textarea output-textarea"
                class:struct-output={activeMode === 'struct'}
                class:success-output={activeMode === 'validate' && jsonOutput.includes('✓')}
                readonly
                spellcheck="false"
              ></textarea>
            </div>
          {/if}
        </div>
      </div>
    {:else if viewMode === 'tree' && parsedJson}
      <!-- 树形视图 -->
      <div class="tree-view-container">
        <div class="tree-view">
          {#snippet renderNode(value: unknown, key: string, path: string, depth: number)}
            {@const isCollapsed = collapsedPaths.has(path)}
            {@const isExpandable = value !== null && typeof value === 'object'}
            {@const childCount = Array.isArray(value) ? value.length : (typeof value === 'object' && value !== null ? Object.keys(value).length : 0)}
            
            <div class="tree-node" style="--depth: {depth}">
              <div class="node-line">
                {#if isExpandable}
                  <button class="expand-btn" onclick={() => toggleCollapse(path)}>
                    {isCollapsed ? '▶' : '▼'}
                  </button>
      {:else}
                  <span class="expand-placeholder"></span>
      {/if}
                
                <span class="node-key">{key}</span>
                <span class="node-colon">:</span>
                
                {#if isExpandable}
                  <span class="node-type">{getTypeLabel(value)}</span>
                  {#if isCollapsed}
                    <span class="collapsed-preview">
                      {Array.isArray(value) ? '[...]' : '{...}'}
                    </span>
                  {/if}
                {:else}
                  <span class={getValueClass(value)}>
                    {#if typeof value === 'string'}
                      "{value}"
                    {:else}
                      {JSON.stringify(value)}
                    {/if}
                  </span>
                {/if}
                
                <button class="copy-value-btn" onclick={() => copyToClipboard(JSON.stringify(value), `value-${path}`)} title="复制值">
                  {copied === `value-${path}` ? '✓' : '📋'}
                </button>
    </div>
              
              {#if isExpandable && !isCollapsed}
                <div class="node-children">
                  {#if Array.isArray(value)}
                    {#each value as item, index}
                      {@render renderNode(item, `[${index}]`, `${path}[${index}]`, depth + 1)}
                    {/each}
                  {:else if typeof value === 'object' && value !== null}
                    {#each Object.entries(value) as [k, v]}
                      {@render renderNode(v, k, `${path}.${k}`, depth + 1)}
                    {/each}
                  {/if}
  </div>
              {/if}
            </div>
          {/snippet}
          
          {@render renderNode(parsedJson, 'root', '$', 0)}
        </div>
      </div>
    {:else}
      <div class="empty-tree">
        <p>请先输入有效的 JSON 数据</p>
      </div>
  {/if}
  </div>

  <!-- 错误面板 -->
  {#if errorInfo}
    <div class="error-panel">
      <div class="error-header">
        <span class="error-badge">✕ JSON格式错误</span>
        <button class="jump-btn" onclick={jumpToErrorLine}>↗ 跳转到错误</button>
      </div>
      <div class="error-body">
        <div class="error-location">第 <strong>{errorInfo.line}</strong> 行，第 <strong>{errorInfo.column}</strong> 列</div>
        <div class="error-message">{errorInfo.message}</div>
        {#if errorInfo.lineContent}
          <div class="error-code-box">
            <div class="error-line-num">{errorInfo.line}</div>
            <div class="error-line-content">
              <code>{errorInfo.lineContent}</code>
              <div class="error-pointer">{getErrorPointer(errorInfo.column)}</div>
            </div>
          </div>
        {/if}
        {#if errorInfo.expected.length > 0}
          <div class="error-expect">
            <span class="expect-label">期望</span>
            {#each errorInfo.expected as exp, i}
              <code class="token-good">{exp}</code>{i < errorInfo.expected.length - 1 ? ',' : ''}
            {/each}
            <span class="found-label">实际</span>
            <code class="token-bad">{errorInfo.found}</code>
          </div>
        {/if}
      </div>
    </div>
  {/if}

  <!-- 后端处理错误（convert / schema / query） -->
  {#if backendError}
    <div class="backend-error">⚠️ {backendError}</div>
  {/if}

  <!-- Diff 结果 -->
  {#if activeMode === 'diff' && diffResult}
    <div class="diff-result">
      <div class="diff-summary">
        <span class="diff-stat add">+{diffResult.added}</span>
        <span class="diff-stat del">-{diffResult.removed}</span>
        <span class="diff-stat chg">~{diffResult.changed}</span>
        <span class="diff-hint">{diffResult.added + diffResult.removed + diffResult.changed === 0 ? '两侧内容一致' : '左为 A，右为 B'}</span>
      </div>
      <div class="diff-rows">
        {#each diffResult.rows as row}
          <div class="diff-row {row.type}">
            <span class="diff-ln">{row.leftNo ?? ''}</span>
            <span class="diff-cell left">{row.left}</span>
            <span class="diff-ln">{row.rightNo ?? ''}</span>
            <span class="diff-cell right">{row.right}</span>
          </div>
        {/each}
      </div>
    </div>
  {/if}

  <!-- Schema 校验结果 -->
  {#if activeMode === 'schema' && schemaMode === 'validate' && schemaResult}
    <div class="schema-result">
      {#if schemaResult.valid}
        <div class="schema-ok">✓ 数据符合 Schema</div>
      {:else}
        <div class="schema-fail">✕ 共 {schemaResult.errors.length} 处不符合：</div>
        <div class="schema-errors">
          {#each schemaResult.errors as err}
            <div class="schema-err-item">
              <code class="schema-err-path">{err.path}</code>
              <span class="schema-err-msg">{err.message}</span>
            </div>
          {/each}
        </div>
      {/if}
    </div>
  {/if}

  <!-- 查询结果 -->
  {#if activeMode === 'query' && jsonPathResult}
    <div class="jsonpath-result">
      <span class="result-label">查询结果:</span>
      <pre>{jsonPathResult}</pre>
      <button class="copy-btn-small" onclick={() => copyToClipboard(jsonPathResult, 'path')}>
        {copied === 'path' ? '✓' : '📋'}
      </button>
    </div>
  {/if}

  <!-- 历史记录 -->
  {#if history.length > 0}
    <div class="history-section">
      <div class="history-header">
        <span class="history-title">📜 历史记录 ({history.length}/20)</span>
        <button class="clear-history-btn" onclick={clearHistory}>清空</button>
      </div>
      <div class="history-list">
        {#each history as item}
          <div class="history-item" role="button" tabindex="0" onclick={() => loadFromHistory(item)} onkeydown={(e) => handleHistoryKeydown(e, item)}>
            <span class="history-preview">{item.preview}</span>
            <span class="history-meta">{formatTime(item.timestamp)} · {formatSize(item.size)}</span>
            <button type="button" class="history-delete" aria-label="删除历史记录" onclick={(e) => deleteHistory(item.id, e)}>×</button>
          </div>
        {/each}
      </div>
    </div>
  {/if}
</div>

<style>
  .json-tool {
    --json-editor-font-size: 0.85rem;
    --json-editor-line-height: 1.5;
    --json-editor-row-height: calc(var(--json-editor-font-size) * var(--json-editor-line-height));
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    height: 100%;
    min-height: 600px;
  }

  .json-tool.fullscreen {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    z-index: 1000;
    background: var(--bg-darker);
    padding: 1rem;
    min-height: 100vh;
  }

  /* 工具栏 */
  .toolbar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.4rem;
    background: var(--bg-dark);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    flex-wrap: wrap;
    gap: 0.5rem;
  }

  .toolbar-left, .toolbar-right {
    display: flex;
    gap: 0.25rem;
    flex-wrap: wrap;
  }

  .mode-btn {
    display: flex;
    align-items: center;
    gap: 0.3rem;
    padding: 0.4rem 0.6rem !important;
    background: transparent !important;
    border: 1px solid transparent !important;
    color: var(--text-secondary) !important;
    font-size: 0.8rem;
  }

  .mode-btn:hover {
    background: var(--bg-hover) !important;
    color: var(--text-primary) !important;
    transform: none !important;
  }

  .mode-btn.active {
    background: var(--primary) !important;
    color: white !important;
  }

  .mode-icon {
    font-family: 'JetBrains Mono', monospace;
    font-size: 0.75rem;
    font-weight: 700;
  }

  .toolbar-btn {
    padding: 0.4rem 0.6rem !important;
    background: var(--bg-card) !important;
    border: 1px solid var(--border) !important;
    font-size: 0.8rem;
  }

  .toolbar-btn:hover {
    border-color: var(--primary) !important;
    transform: none !important;
  }

  /* 选项栏 */
  .options-bar {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.4rem 0.6rem;
    background: var(--bg-dark);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    flex-wrap: wrap;
  }

  .option-group {
    display: flex;
    align-items: center;
    gap: 0.3rem;
  }

  .option-group label {
    font-size: 0.8rem;
    color: var(--text-muted);
  }

  .option-group select {
    padding: 0.25rem 1.5rem 0.25rem 0.4rem;
    background: var(--bg-card);
    border: 1px solid var(--border);
    border-radius: 4px;
    color: var(--text-primary);
    font-size: 0.8rem;
    cursor: pointer;
    appearance: none;
    background-image: url("data:image/svg+xml;charset=UTF-8,%3csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='%239ca3af' stroke-width='2'%3e%3cpolyline points='6 9 12 15 18 9'%3e%3c/polyline%3e%3c/svg%3e");
    background-repeat: no-repeat;
    background-position: right 0.2rem center;
    background-size: 0.7em;
  }

  .option-group select option {
    background: #1a1a2e;
    color: #e4e4e7;
  }

  .checkbox-option {
    display: flex;
    align-items: center;
    gap: 0.3rem;
    font-size: 0.8rem;
    color: var(--text-secondary);
    cursor: pointer;
  }

  .checkbox-option input {
    width: 14px;
    height: 14px;
  }

  .view-toggle {
    display: flex;
    gap: 0.25rem;
    margin-left: auto;
  }

  .view-btn {
    padding: 0.3rem 0.5rem !important;
    background: var(--bg-card) !important;
    border: 1px solid var(--border) !important;
    font-size: 0.75rem;
  }

  .view-btn:hover:not(:disabled) {
    border-color: var(--primary) !important;
    transform: none !important;
  }

  .view-btn.active {
    background: var(--primary) !important;
    color: white !important;
    border-color: var(--primary) !important;
  }

  .view-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .collapse-controls {
    display: flex;
    gap: 0.25rem;
  }

  .collapse-btn {
    padding: 0.2rem 0.4rem !important;
    background: var(--bg-card) !important;
    border: 1px solid var(--border) !important;
    font-size: 0.7rem;
    color: var(--text-muted) !important;
  }

  .collapse-btn:hover {
    color: var(--text-primary) !important;
    border-color: var(--accent) !important;
    transform: none !important;
  }

  .search-box {
    display: flex;
    align-items: center;
    gap: 0.35rem;
    min-width: 240px;
  }

  .search-box input {
    padding: 0.3rem 0.5rem !important;
    width: 180px;
    font-size: 0.8rem !important;
    background: var(--bg-card) !important;
  }

  .search-label {
    font-size: 0.8rem;
    color: var(--text-muted);
  }

  .search-status {
    min-width: 2.75rem;
    font-size: 0.75rem;
    color: var(--text-muted);
    text-align: center;
  }

  .search-nav-btn {
    padding: 0.2rem 0.45rem !important;
    background: var(--bg-card) !important;
    border: 1px solid var(--border) !important;
    font-size: 0.75rem;
    line-height: 1;
  }

  .search-nav-btn:hover:not(:disabled) {
    border-color: var(--primary) !important;
    transform: none !important;
  }

  .search-nav-btn:disabled {
    opacity: 0.45;
    cursor: not-allowed;
  }

  /* JSONPath栏 */
  .jsonpath-bar {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.4rem 0.6rem;
    background: var(--bg-dark);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
  }

  .jsonpath-bar label {
    font-size: 0.85rem;
    color: var(--text-secondary);
  }

  .jsonpath-bar input {
    flex: 1;
    font-family: 'JetBrains Mono', monospace;
    font-size: 0.85rem !important;
    padding: 0.35rem 0.5rem !important;
  }

  /* 统计栏 */
  .stats-bar {
    display: flex;
    flex-wrap: wrap;
    gap: 0.3rem;
    padding: 0.3rem 0.6rem;
    background: var(--bg-dark);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
  }

  .stat-chip {
    padding: 0.15rem 0.4rem;
    background: var(--bg-card);
    border-radius: 4px;
    font-size: 0.7rem;
    color: var(--text-muted);
  }

  .stat-chip b {
    color: var(--accent);
  }

  /* 主编辑区 */
  .main-editor {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
  }

  .editors-container {
    display: grid;
    grid-template-columns: var(--split-ratio, 50%) 6px calc(100% - var(--split-ratio, 50%) - 6px);
    gap: 0;
    flex: 1;
    min-height: 400px;
  }

  .resizer {
    background: var(--border);
    cursor: col-resize;
    transition: background 0.2s;
  }

  .resizer:hover {
    background: var(--primary);
  }

  .editor-panel {
    display: flex;
    flex-direction: column;
    background: var(--bg-dark);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    overflow: hidden;
    min-width: 200px;
  }

  .editor-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.4rem 0.6rem;
    background: var(--bg-card);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }

  .editor-title {
    font-size: 0.8rem;
    font-weight: 500;
    color: var(--text-secondary);
  }

  .editor-actions {
    display: flex;
    align-items: center;
    gap: 0.4rem;
  }

  .editor-info {
    font-size: 0.7rem;
    color: var(--text-muted);
  }

  .copy-btn {
    padding: 0.2rem 0.4rem !important;
    background: var(--bg-hover) !important;
    border: 1px solid var(--border) !important;
    font-size: 0.75rem;
  }

  .copy-btn:hover {
    border-color: var(--primary) !important;
    transform: none !important;
  }

  .editor-body {
    display: flex;
    flex: 1;
    overflow: hidden;
    min-height: 0;
  }

  .line-numbers {
    padding: 0.5rem 0;
    background: rgba(0, 0, 0, 0.2);
    border-right: 1px solid var(--border);
    text-align: right;
    user-select: none;
    min-width: 40px;
    overflow: hidden;
    flex-shrink: 0;
  }

  .line-num {
    box-sizing: border-box;
    display: flex;
    align-items: center;
    justify-content: flex-end;
    min-height: var(--json-editor-row-height);
    padding: 0 0.4rem;
    font-family: 'JetBrains Mono', monospace;
    font-size: var(--json-editor-font-size);
    line-height: var(--json-editor-line-height);
    color: var(--text-muted);
  }

  .line-num.error-line {
    background: rgba(239, 68, 68, 0.3);
    color: #f87171;
    font-weight: bold;
  }

  .code-textarea {
    box-sizing: border-box;
    flex: 1;
    padding: 0.5rem;
    border: none !important;
    background: transparent !important;
    color: var(--text-primary);
    font-family: 'JetBrains Mono', monospace;
    font-size: var(--json-editor-font-size);
    line-height: var(--json-editor-line-height);
    resize: none;
    min-height: 100%;
  }

  .code-textarea:focus {
    outline: none;
    box-shadow: none !important;
  }

  .input-editor-stack {
    position: relative;
    flex: 1;
    min-width: 0;
    min-height: 0;
    overflow: hidden;
  }

  .highlight-layer,
  .input-textarea {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
  }

  .highlight-layer {
    overflow: hidden;
    pointer-events: none;
    padding: 0.5rem;
    background: transparent;
  }

  .highlight-content {
    min-height: 100%;
    font-family: 'JetBrains Mono', monospace;
    font-size: var(--json-editor-font-size);
    line-height: var(--json-editor-line-height);
    color: var(--text-primary);
    white-space: pre-wrap;
    word-break: break-word;
    overflow-wrap: anywhere;
  }

  .input-textarea {
    z-index: 1;
    color: transparent;
    caret-color: var(--text-primary);
    background: transparent !important;
  }

  .input-textarea::placeholder {
    color: var(--text-muted);
  }

  .highlight-content :global(.search-highlight) {
    background: rgba(250, 204, 21, 0.35);
    border-radius: 3px;
    color: inherit;
    padding: 0;
  }

  .highlight-content :global(.search-highlight.active) {
    background: rgba(249, 115, 22, 0.55);
    box-shadow: 0 0 0 1px rgba(249, 115, 22, 0.45);
  }

  .output-body {
    background: var(--bg-card);
  }

  .output-textarea {
    color: var(--accent);
  }

  .output-textarea.struct-output {
    color: #22c55e;
  }

  .output-textarea.success-output {
    color: var(--accent-green);
    font-size: 1.1rem;
    text-align: center;
  }

  /* 树形视图 */
  .tree-view-container {
    flex: 1;
    background: var(--bg-dark);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    overflow: auto;
    min-height: 400px;
  }

  .tree-view {
    padding: 0.75rem;
    font-family: 'JetBrains Mono', monospace;
    font-size: 0.85rem;
  }

  .tree-node {
    margin-left: calc(var(--depth, 0) * 1.2rem);
  }

  .node-line {
    display: flex;
    align-items: center;
    gap: 0.3rem;
    padding: 0.15rem 0;
    border-radius: 4px;
    transition: background 0.15s;
  }

  .node-line:hover {
    background: var(--bg-hover);
  }

  .expand-btn {
    width: 1.2rem;
    height: 1.2rem;
    padding: 0 !important;
    background: transparent !important;
    border: none !important;
    color: var(--text-muted) !important;
    font-size: 0.7rem;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .expand-btn:hover {
    color: var(--primary) !important;
    transform: none !important;
  }

  .expand-placeholder {
    width: 1.2rem;
    flex-shrink: 0;
  }

  .node-key {
    color: #f472b6;
    font-weight: 500;
  }

  .node-colon {
    color: var(--text-muted);
  }

  .node-type {
    color: var(--text-muted);
    font-size: 0.75rem;
    padding: 0.1rem 0.3rem;
    background: var(--bg-card);
    border-radius: 3px;
  }

  .collapsed-preview {
    color: var(--text-muted);
    font-style: italic;
  }

  .json-null { color: #f87171; }
  .json-string { color: #a3e635; }
  .json-number { color: #60a5fa; }
  .json-boolean { color: #c084fc; }

  .copy-value-btn {
    padding: 0.1rem 0.2rem !important;
    background: transparent !important;
    border: none !important;
    font-size: 0.7rem;
    opacity: 0;
    transition: opacity 0.15s;
  }

  .node-line:hover .copy-value-btn {
    opacity: 1;
  }

  .copy-value-btn:hover {
    transform: none !important;
  }

  .node-children {
    border-left: 1px dashed var(--border);
    margin-left: 0.6rem;
    padding-left: 0.3rem;
  }

  .empty-tree {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-muted);
    background: var(--bg-dark);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    min-height: 400px;
  }

  /* 错误面板 */
  .error-panel {
    background: var(--bg-dark);
    border: 1px solid rgba(239, 68, 68, 0.4);
    border-radius: var(--radius-sm);
    overflow: hidden;
  }

  .error-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.5rem 0.6rem;
    background: rgba(239, 68, 68, 0.15);
    border-bottom: 1px solid rgba(239, 68, 68, 0.3);
  }

  .error-badge {
    font-size: 0.85rem;
    font-weight: 600;
    color: #f87171;
  }

  .jump-btn {
    padding: 0.25rem 0.5rem !important;
    background: transparent !important;
    border: 1px solid rgba(239, 68, 68, 0.4) !important;
    color: #f87171 !important;
    font-size: 0.75rem;
  }

  .jump-btn:hover {
    background: rgba(239, 68, 68, 0.1) !important;
    transform: none !important;
  }

  .error-body {
    padding: 0.6rem;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .error-location {
    font-size: 0.8rem;
    color: var(--text-secondary);
  }

  .error-location strong {
    color: #f87171;
    font-weight: 700;
  }

  .error-message {
    font-size: 0.85rem;
    color: var(--text-primary);
  }

  .error-code-box {
    display: flex;
    background: #1a1a2e;
    border-radius: 6px;
    overflow: hidden;
    border: 1px solid var(--border);
    font-family: 'JetBrains Mono', monospace;
  }

  .error-line-num {
    padding: 0.4rem 0.5rem;
    background: rgba(239, 68, 68, 0.2);
    color: #f87171;
    font-size: 0.75rem;
    font-weight: 600;
    border-right: 1px solid var(--border);
    min-width: 32px;
    text-align: center;
  }

  .error-line-content {
    flex: 1;
    padding: 0.4rem 0.6rem;
    overflow-x: auto;
  }

  .error-line-content code {
    display: block;
    font-size: 0.8rem;
    color: var(--text-primary);
    white-space: pre;
    background: transparent !important;
    padding: 0 !important;
  }

  .error-pointer {
    font-size: 0.8rem;
    color: #f87171;
    font-weight: bold;
    white-space: pre;
  }

  .error-expect {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.3rem;
    font-size: 0.8rem;
    padding: 0.4rem 0.6rem;
    background: var(--bg-card);
    border-radius: 6px;
  }

  .expect-label, .found-label {
    color: var(--text-muted);
  }

  .token-good {
    padding: 0.1rem 0.3rem;
    background: rgba(16, 185, 129, 0.15);
    border: 1px solid rgba(16, 185, 129, 0.3);
    border-radius: 4px;
    font-size: 0.7rem;
    color: #10b981;
  }

  .token-bad {
    padding: 0.1rem 0.3rem;
    background: rgba(239, 68, 68, 0.15);
    border: 1px solid rgba(239, 68, 68, 0.3);
    border-radius: 4px;
    font-size: 0.7rem;
    color: #f87171;
  }

  /* JSONPath结果 */
  .jsonpath-result {
    display: flex;
    align-items: flex-start;
    gap: 0.6rem;
    padding: 0.6rem;
    background: var(--bg-dark);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
  }

  .result-label {
    font-size: 0.75rem;
    color: var(--text-muted);
    white-space: nowrap;
  }

  .jsonpath-result pre {
    flex: 1;
    margin: 0;
    padding: 0.4rem;
    background: var(--bg-card);
    border-radius: 4px;
    font-size: 0.8rem;
    color: var(--accent-green);
    max-height: 240px;
    overflow-y: auto;
    white-space: pre-wrap;
    word-break: break-word;
  }

  .copy-btn-small {
    padding: 0.15rem 0.3rem !important;
    background: var(--bg-card) !important;
    border: 1px solid var(--border) !important;
    font-size: 0.75rem;
  }

  .copy-btn-small:hover {
    border-color: var(--primary) !important;
    transform: none !important;
  }

  /* 后端错误提示 */
  .backend-error {
    padding: 0.6rem 0.8rem;
    background: rgba(239, 68, 68, 0.1);
    border: 1px solid #ef4444;
    border-radius: var(--radius-sm);
    color: #fca5a5;
    font-size: 0.85rem;
    word-break: break-word;
  }

  /* Diff 结果 */
  .diff-result {
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    overflow: hidden;
  }

  .diff-summary {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.5rem 0.75rem;
    background: var(--bg-card);
    border-bottom: 1px solid var(--border);
    font-size: 0.8rem;
  }

  .diff-stat {
    font-family: 'JetBrains Mono', monospace;
    font-weight: 600;
  }
  .diff-stat.add { color: var(--accent-green); }
  .diff-stat.del { color: #f87171; }
  .diff-stat.chg { color: #fbbf24; }
  .diff-hint { color: var(--text-muted); margin-left: auto; }

  .diff-rows {
    max-height: 420px;
    overflow: auto;
    font-family: 'JetBrains Mono', monospace;
    font-size: 0.8rem;
    line-height: 1.5;
  }

  .diff-row {
    display: grid;
    grid-template-columns: 3rem 1fr 3rem 1fr;
    white-space: pre-wrap;
    word-break: break-word;
  }

  .diff-ln {
    text-align: right;
    padding: 0 0.5rem;
    color: var(--text-muted);
    background: var(--bg-card);
    user-select: none;
  }

  .diff-cell {
    padding: 0 0.5rem;
  }

  .diff-row.insert .diff-cell.right,
  .diff-row.replace .diff-cell.right {
    background: rgba(16, 185, 129, 0.15);
  }
  .diff-row.delete .diff-cell.left,
  .diff-row.replace .diff-cell.left {
    background: rgba(248, 113, 113, 0.15);
  }
  .diff-row.delete .diff-cell.right,
  .diff-row.insert .diff-cell.left {
    background: rgba(255, 255, 255, 0.02);
  }

  /* Schema 校验结果 */
  .schema-result {
    padding: 0.6rem 0.8rem;
    background: var(--bg-dark);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    font-size: 0.85rem;
  }
  .schema-ok { color: var(--accent-green); font-weight: 600; }
  .schema-fail { color: #f87171; font-weight: 600; margin-bottom: 0.5rem; }

  .schema-errors {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    max-height: 240px;
    overflow-y: auto;
  }

  .schema-err-item {
    display: flex;
    gap: 0.6rem;
    align-items: baseline;
  }

  .schema-err-path {
    color: var(--accent);
    background: var(--bg-card);
    padding: 0.1rem 0.4rem;
    border-radius: 4px;
    white-space: nowrap;
    font-size: 0.78rem;
  }

  .schema-err-msg {
    color: var(--text-secondary);
    word-break: break-word;
  }

  /* 历史记录 */
  .history-section {
    background: var(--bg-dark);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    overflow: hidden;
  }

  .history-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.4rem 0.6rem;
    background: var(--bg-card);
    border-bottom: 1px solid var(--border);
  }

  .history-title {
    font-size: 0.8rem;
    font-weight: 500;
    color: var(--text-secondary);
  }

  .clear-history-btn {
    padding: 0.15rem 0.4rem !important;
    background: transparent !important;
    border: 1px solid var(--border) !important;
    font-size: 0.7rem;
    color: var(--text-muted) !important;
  }

  .clear-history-btn:hover {
    border-color: #ef4444 !important;
    color: #ef4444 !important;
    transform: none !important;
  }

  .history-list {
    display: flex;
    flex-wrap: wrap;
    gap: 0.4rem;
    padding: 0.5rem;
    max-height: 100px;
    overflow-y: auto;
  }

  .history-item {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    padding: 0.3rem 0.5rem;
    background: var(--bg-card) !important;
    border: 1px solid var(--border) !important;
    border-radius: var(--radius-sm);
    cursor: pointer;
    max-width: 280px;
    text-align: left;
  }

  .history-item:hover {
    border-color: var(--primary) !important;
    background: var(--bg-hover) !important;
    transform: none !important;
  }

  .history-preview {
    font-size: 0.7rem;
    font-family: 'JetBrains Mono', monospace;
    color: var(--text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 140px;
  }

  .history-meta {
    font-size: 0.6rem;
    color: var(--text-muted);
    white-space: nowrap;
  }

  .history-delete {
    background: transparent;
    border: none;
    padding: 0;
    font-size: 0.9rem;
    color: var(--text-muted);
    opacity: 0;
    cursor: pointer;
  }

  .history-item:hover .history-delete {
    opacity: 1;
  }

  .history-delete:hover {
    color: #ef4444;
  }
</style>
