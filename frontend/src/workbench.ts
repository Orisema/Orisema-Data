export const DEFAULT_SQL =
  "SELECT\n  current_database() AS database,\n  current_user AS username,\n  version() AS version;";

export interface EditorTab {
  id: string;
  title: string;
  sql: string;
  connectionId: string;
  updatedAt: string;
}

export interface QueryHistoryItem {
  id: string;
  connectionId: string;
  connectionName: string;
  sql: string;
  executedAt: string;
  durationMs?: number;
  status: "success" | "error" | "cancelled";
  favorite: boolean;
}

export interface SavedQuery {
  id: string;
  name: string;
  sql: string;
  createdAt: string;
  updatedAt: string;
}

export interface WorkbenchData {
  tabs: EditorTab[];
  activeTabId: string;
  history: QueryHistoryItem[];
  saved: SavedQuery[];
  theme: "dark" | "light";
  editorPercent: number;
}

const STORAGE_KEY = "rustdb.workbench.v1";

export const newId = () => crypto.randomUUID();

export function createTab(connectionId = "", index = 1): EditorTab {
  return {
    id: newId(),
    title: `Consulta ${index}`,
    sql: DEFAULT_SQL,
    connectionId,
    updatedAt: new Date().toISOString(),
  };
}

export function initialWorkbench(): WorkbenchData {
  const tab = createTab();
  return {
    tabs: [tab],
    activeTabId: tab.id,
    history: [],
    saved: [],
    theme: "dark",
    editorPercent: 42,
  };
}

export function loadWorkbench(): WorkbenchData {
  const fallback = initialWorkbench();
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return fallback;
    const parsed = JSON.parse(raw) as Partial<WorkbenchData>;
    const tabs = Array.isArray(parsed.tabs) && parsed.tabs.length
      ? parsed.tabs.filter((tab) => tab && typeof tab.sql === "string")
      : fallback.tabs;
    const activeTabId = tabs.some((tab) => tab.id === parsed.activeTabId)
      ? parsed.activeTabId as string
      : tabs[0].id;
    return {
      tabs,
      activeTabId,
      history: Array.isArray(parsed.history) ? parsed.history.slice(0, 100) : [],
      saved: Array.isArray(parsed.saved) ? parsed.saved : [],
      theme: parsed.theme === "light" ? "light" : "dark",
      editorPercent:
        typeof parsed.editorPercent === "number"
          ? Math.min(70, Math.max(25, parsed.editorPercent))
          : 42,
    };
  } catch {
    return fallback;
  }
}

export function saveWorkbench(data: WorkbenchData) {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(data));
  } catch {
    // The editor remains usable when storage is unavailable or full.
  }
}

export function historyWith(
  history: QueryHistoryItem[],
  item: QueryHistoryItem,
) {
  return [item, ...history].slice(0, 100);
}
