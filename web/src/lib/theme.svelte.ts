import { Persisted } from "./persisted.svelte";

export type ThemeMode = "dark" | "light" | "system";

export const THEME_MODES: ThemeMode[] = ["dark", "light", "system"];

export const THEME_LABELS: Record<ThemeMode, string> = {
  dark: "暗色",
  light: "亮色",
  system: "跟随系统",
};

export const THEME_ICONS: Record<ThemeMode, string> = {
  dark: "🌙",
  light: "☀",
  system: "◐",
};

const stored = new Persisted<ThemeMode>("theme", "dark");
let systemDark = $state(true);

function resolve(): "dark" | "light" {
  const mode = stored.current;
  if (mode === "system") return systemDark ? "dark" : "light";
  return mode;
}

function apply(): void {
  document.documentElement.dataset.theme = resolve();
}

/** 在应用挂载时调用一次，接管 <html data-theme>。 */
export function initTheme(): void {
  const mq = window.matchMedia("(prefers-color-scheme: dark)");
  systemDark = mq.matches;
  mq.addEventListener("change", (event) => {
    systemDark = event.matches;
    apply();
  });
  apply();
}

export const theme = {
  get mode(): ThemeMode {
    return stored.current;
  },
  set mode(value: ThemeMode) {
    stored.current = value;
    apply();
  },
  get resolved(): "dark" | "light" {
    return resolve();
  },
  cycle(): void {
    const next = (THEME_MODES.indexOf(stored.current) + 1) % THEME_MODES.length;
    theme.mode = THEME_MODES[next];
  },
};
