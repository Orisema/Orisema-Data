import { describe, expect, it } from "vitest";
import { queryResultCsv } from "./csv";

describe("queryResultCsv", () => {
  it("escapes commas, quotes, newlines and keeps null empty", () => {
    const csv = queryResultCsv({
      columns: [{ name: "name", type: "text" }, { name: "note", type: "text" }],
      rows: [["a,b", 'say "hi"'], [null, "line\nbreak"], ["=1+1", -2]],
      row_count: 3,
      affected_rows: 0,
      execution_time_ms: 1,
      truncated: false,
    });
    expect(csv).toBe('\uFEFFname,note\r\n"a,b","say ""hi"""\r\n,"line\nbreak"\r\n\'=1+1,-2');
  });
});
