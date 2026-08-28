import type { ToolMeta } from "./tools";

/**
 * 命令面板的排序检索：优先名称，其次 id / 关键词 / 描述，最后退化为子序列匹配
 * （输入 "jt" 也能命中 "JSON 工具"）。中文直接走子串匹配。
 */
export function searchTools(tools: ToolMeta[], query: string): ToolMeta[] {
  const q = query.trim().toLowerCase();
  if (!q) return tools;

  const scored: { tool: ToolMeta; score: number }[] = [];
  for (const tool of tools) {
    const score = scoreTool(tool, q);
    if (score > 0) scored.push({ tool, score });
  }
  // Array.prototype.sort 是稳定排序，同分时保留注册表顺序
  scored.sort((a, b) => b.score - a.score);
  return scored.map((item) => item.tool);
}

function scoreTool(tool: ToolMeta, q: string): number {
  const name = tool.name.toLowerCase();
  if (name === q) return 1000;
  if (name.startsWith(q)) return 900;
  if (tool.id.startsWith(q)) return 860;
  if (name.includes(q)) return 800;
  if (tool.id.includes(q)) return 700;

  let best = 0;
  for (const keyword of tool.keywords) {
    const k = keyword.toLowerCase();
    if (k === q) best = Math.max(best, 620);
    else if (k.startsWith(q)) best = Math.max(best, 560);
    else if (k.includes(q)) best = Math.max(best, 500);
  }
  if (best > 0) return best;

  if (tool.desc.toLowerCase().includes(q)) return 400;
  if (isSubsequence(q, name)) return 300;
  return 0;
}

function isSubsequence(needle: string, haystack: string): boolean {
  let i = 0;
  for (const char of haystack) {
    if (char === needle[i]) i++;
    if (i === needle.length) return true;
  }
  return needle.length === 0;
}
