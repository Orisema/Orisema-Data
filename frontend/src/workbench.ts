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

const isRecord = (value: unknown): value is Record<string, unknown> =>
  typeof value === "object" && value !== null;

const isEditorTab = (value: unknown): value is EditorTab =>
  isRecord(value) &&
  typeof value.id === "string" &&
  typeof value.title === "string" &&
  typeof value.sql === "string" &&
  typeof value.connectionId === "string" &&
  typeof value.updatedAt === "string";

const isHistoryItem = (value: unknown): value is QueryHistoryItem =>
  isRecord(value) &&
  typeof value.id === "string" &&
  typeof value.connectionId === "string" &&
  typeof value.connectionName === "string" &&
  typeof value.sql === "string" &&
  typeof value.executedAt === "string" &&
  (value.durationMs === undefined || typeof value.durationMs === "number") &&
  (value.status === "success" || value.status === "error" || value.status === "cancelled") &&
  typeof value.favorite === "boolean";

const isSavedQuery = (value: unknown): value is SavedQuery =>
  isRecord(value) &&
  typeof value.id === "string" &&
  typeof value.name === "string" &&
  typeof value.sql === "string" &&
  typeof value.createdAt === "string" &&
  typeof value.updatedAt === "string";

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
    const parsed: unknown = JSON.parse(raw);
    if (!isRecord(parsed)) return fallback;
    const tabs = Array.isArray(parsed.tabs) && parsed.tabs.length
      ? parsed.tabs.filter(isEditorTab)
      : fallback.tabs;
    if (tabs.length === 0) return fallback;
    const activeTabId = tabs.some((tab) => tab.id === parsed.activeTabId)
      ? parsed.activeTabId as string
      : tabs[0].id;
    return {
      tabs,
      activeTabId,
      history: Array.isArray(parsed.history)
        ? parsed.history.filter(isHistoryItem).slice(0, 100)
        : [],
      saved: Array.isArray(parsed.saved) ? parsed.saved.filter(isSavedQuery) : [],
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
