export type ToastKind = "success" | "error" | "info";

export interface ToastItem {
  id: number;
  kind: ToastKind;
  text: string;
}

let items = $state<ToastItem[]>([]);
let nextId = 0;

function push(kind: ToastKind, text: string, ttl: number): void {
  const id = ++nextId;
  items = [...items, { id, kind, text }];
  setTimeout(() => dismiss(id), ttl);
}

function dismiss(id: number): void {
  items = items.filter((item) => item.id !== id);
}

export const toastStore = {
  get items(): ToastItem[] {
    return items;
  },
  dismiss,
};

/** 全局唯一的反馈入口，取代此前并存的 alert / 内联文案 / 各自的 setTimeout。 */
export const toast = {
  success: (text: string) => push("success", text, 2200),
  error: (text: string) => push("error", text, 4000),
  info: (text: string) => push("info", text, 2600),
};
