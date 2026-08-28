/**
 * 带命名空间的 localStorage 包装。
 * 所有读写都吞掉异常：隐私模式下 localStorage 访问会直接抛错，
 * 持久化只是增强，绝不能让它把功能带崩。
 */

const PREFIX = "devtools:";

export function readStore<T>(key: string, fallback: T): T {
  try {
    const raw = localStorage.getItem(PREFIX + key);
    if (raw === null) return fallback;
    return JSON.parse(raw) as T;
  } catch {
    return fallback;
  }
}

export function writeStore<T>(key: string, value: T): void {
  try {
    localStorage.setItem(PREFIX + key, JSON.stringify(value));
  } catch {
    /* 隐私模式 / 配额已满：静默忽略 */
  }
}

export function removeStore(key: string): void {
  try {
    localStorage.removeItem(PREFIX + key);
  } catch {
    /* 同上 */
  }
}
