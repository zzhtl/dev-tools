import { DEFAULT_TOOL_ID, TOOL_BY_ID } from "./tools";
import { readStore, writeStore } from "./storage";

/**
 * hash 路由：#/json、#/regex。
 * 后端 assets.rs 已有 SPA fallback，但用 hash 可以完全不依赖它，
 * 直接开 index.html 也能工作。
 */

let currentId = $state(DEFAULT_TOOL_ID);

function parseHash(): string | null {
  const matched = /^#\/([\w-]+)/.exec(window.location.hash);
  if (!matched) return null;
  return TOOL_BY_ID.has(matched[1]) ? matched[1] : null;
}

function sync(): void {
  const fromHash = parseHash();
  if (fromHash) {
    currentId = fromHash;
    writeStore("lastTool", fromHash);
    return;
  }
  // 无 hash 或 hash 非法：回到上次使用的工具，并静默补写 URL（不新增历史条目）
  const last = readStore<string>("lastTool", DEFAULT_TOOL_ID);
  const id = TOOL_BY_ID.has(last) ? last : DEFAULT_TOOL_ID;
  window.history.replaceState(null, "", `#/${id}`);
  currentId = id;
  writeStore("lastTool", id);
}

export function initRouter(): void {
  window.addEventListener("hashchange", sync);
  sync();
}

export const router = {
  get id(): string {
    return currentId;
  },
  navigate(id: string): void {
    if (!TOOL_BY_ID.has(id) || id === currentId) return;
    window.location.hash = `#/${id}`;
  },
};
