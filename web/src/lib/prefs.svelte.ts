import { Persisted } from "./persisted.svelte";

const RECENT_LIMIT = 8;

const favorites = new Persisted<string[]>("favorites", []);
const recents = new Persisted<string[]>("recents", []);
const sidebarCollapsed = new Persisted<boolean>("sidebarCollapsed", false);

export const prefs = {
  get favorites(): string[] {
    return favorites.current;
  },
  isFavorite(id: string): boolean {
    return favorites.current.includes(id);
  },
  toggleFavorite(id: string): void {
    favorites.current = favorites.current.includes(id)
      ? favorites.current.filter((item) => item !== id)
      : [...favorites.current, id];
  },

  get recents(): string[] {
    return recents.current;
  },
  /** 记录一次使用，LRU 保留最近 8 个。 */
  touch(id: string): void {
    recents.current = [id, ...recents.current.filter((item) => item !== id)].slice(0, RECENT_LIMIT);
  },

  get sidebarCollapsed(): boolean {
    return sidebarCollapsed.current;
  },
  toggleSidebar(): void {
    sidebarCollapsed.current = !sidebarCollapsed.current;
  },
};
