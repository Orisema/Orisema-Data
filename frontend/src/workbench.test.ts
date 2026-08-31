import { describe, expect, it } from "vitest";
import { historyWith, type QueryHistoryItem } from "./workbench";

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
