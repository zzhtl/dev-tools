import { readStore, writeStore } from "./storage";

/**
 * 自动落盘的响应式值。读取走 $state，赋值时同步写入 localStorage。
 *
 *   const theme = new Persisted("theme", "dark");
 *   theme.current = "light";   // 已持久化
 */
export class Persisted<T> {
  #key: string;
  #value = $state<T>() as T;

  constructor(key: string, initial: T) {
    this.#key = key;
    this.#value = readStore(key, initial);
  }

  get current(): T {
    return this.#value;
  }

  set current(value: T) {
    this.#value = value;
    writeStore(this.#key, value);
  }
}
