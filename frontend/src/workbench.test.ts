import { afterEach, describe, expect, it, vi } from "vitest";
import {
  historyWith,
  loadWorkbench,
  saveWorkbench,
  type QueryHistoryItem,
  type WorkbenchData,
} from "./workbench";

const item = (index: number): QueryHistoryItem => ({
  id: String(index),
  connectionId: "connection",
  connectionName: "Local",
  sql: `SELECT ${index}`,
  executedAt: "2026-08-31T12:00:00.000Z",
  status: "success",
  favorite: false,
});

describe("query history", () => {
  it("puts the newest execution first and caps storage at 100 entries", () => {
    const history = Array.from({ length: 100 }, (_, index) => item(index));
    const result = historyWith(history, item(101));
    expect(result).toHaveLength(100);
    expect(result[0].id).toBe("101");
    expect(result.at(-1)?.id).toBe("98");
  });
});

describe("workbench persistence", () => {
  const values = new Map<string, string>();

  afterEach(() => vi.unstubAllGlobals());

  const installStorage = () => {
    vi.stubGlobal("localStorage", {
      getItem: (key: string) => values.get(key) ?? null,
      setItem: (key: string, value: string) => values.set(key, value),
    });
    values.clear();
  };

  it("restores valid tabs, drafts, history, saved queries and preferences", () => {
    installStorage();
    const state: WorkbenchData = {
      tabs: [{
        id: "tab-1", title: "Relatorio", sql: "SELECT 1", connectionId: "connection",
        updatedAt: "2026-08-31T12:00:00.000Z",
      }],
      activeTabId: "tab-1",
      history: [item(1)],
      saved: [{
        id: "saved-1", name: "Consulta salva", sql: "SELECT 2",
        createdAt: "2026-08-31T12:00:00.000Z", updatedAt: "2026-08-31T12:00:00.000Z",
      }],
      theme: "light",
      editorPercent: 60,
    };
    saveWorkbench(state);

    expect(loadWorkbench()).toEqual(state);
  });

  it("drops malformed entries and clamps persisted editor height", () => {
    installStorage();
    localStorage.setItem("rustdb.workbench.v1", JSON.stringify({
      tabs: [{
        id: "tab-1", title: "Valida", sql: "SELECT 1", connectionId: "",
        updatedAt: "2026-08-31T12:00:00.000Z",
      }],
      activeTabId: "missing",
      history: [{ sql: "missing required fields" }],
      saved: [{ name: "missing required fields" }],
      theme: "unexpected",
      editorPercent: 999,
    }));

    const restored = loadWorkbench();
    expect(restored.activeTabId).toBe("tab-1");
    expect(restored.history).toEqual([]);
    expect(restored.saved).toEqual([]);
    expect(restored.theme).toBe("dark");
    expect(restored.editorPercent).toBe(70);
  });
});
