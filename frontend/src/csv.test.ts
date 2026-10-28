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

  it("neutralizes every formula prefix, including leading whitespace", () => {
    const rows = ["=1+1", "+1", "-1", "@SUM(A1)", "\tformula", "\rformula", "  =1+1"];
    const csv = queryResultCsv({
      columns: [{ name: "value", type: "text" }],
      rows: rows.map((value) => [value]),
      row_count: rows.length,
      affected_rows: 0,
      execution_time_ms: 1,
      truncated: false,
    });

    expect(csv.split("\r\n").slice(1)).toEqual([
      "'=1+1", "'+1", "'-1", "'@SUM(A1)", "'\tformula", '"\'\rformula"', "'  =1+1",
    ]);
  });
});
