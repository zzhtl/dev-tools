export const IS_MAC =
  typeof navigator !== "undefined" && /mac|iphone|ipad/i.test(navigator.platform || navigator.userAgent);

/** 修饰键在 mac 上显示 ⌘，其他平台显示 Ctrl。 */
export const MOD_LABEL = IS_MAC ? "⌘" : "Ctrl";

export interface ShortcutSpec {
  key: string;
  /** ⌘（mac）/ Ctrl（其他） */
  mod?: boolean;
  shift?: boolean;
}

export function matchesShortcut(event: KeyboardEvent, spec: ShortcutSpec): boolean {
  const mod = IS_MAC ? event.metaKey : event.ctrlKey;
  if (Boolean(spec.mod) !== mod) return false;
  if (Boolean(spec.shift) !== event.shiftKey) return false;
  return event.key.toLowerCase() === spec.key.toLowerCase();
}

/** 焦点在输入控件里时，除带修饰键的快捷键外都应该让位给输入。 */
export function isEditableTarget(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) return false;
  const tag = target.tagName;
  return tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT" || target.isContentEditable;
}

export interface ShortcutHelp {
  keys: string;
  desc: string;
}

export const GLOBAL_SHORTCUTS: ShortcutHelp[] = [
  { keys: `${MOD_LABEL} K`, desc: "打开命令面板 / 搜索工具" },
  { keys: `${MOD_LABEL} /`, desc: "显示快捷键帮助" },
  { keys: `${MOD_LABEL} B`, desc: "折叠 / 展开侧边栏" },
  { keys: `${MOD_LABEL} ⇧ L`, desc: "切换主题" },
  { keys: "Esc", desc: "关闭面板 / 弹层" },
];
