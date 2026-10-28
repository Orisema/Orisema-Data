import { describe, expect, it } from "vitest";
import { explorerSelect, quoteIdentifier } from "./dialect";

describe("database dialect helpers", () => {
  it("quotes identifiers without allowing delimiter injection", () => {
    expect(quoteIdentifier('a"b', "postgresql")).toBe('"a""b"');
    expect(quoteIdentifier("a`b", "mysql")).toBe("`a``b`");
    expect(quoteIdentifier("a`b", "mariadb")).toBe("`a``b`");
    expect(quoteIdentifier("a]b", "sqlserver")).toBe("[a]]b]");
  });

  it("uses LIMIT for PostgreSQL/MySQL and TOP for SQL Server", () => {
    expect(explorerSelect("public", "items", "postgresql")).toContain("LIMIT 100");
    expect(explorerSelect("shop", "items", "mysql")).toContain("`shop`.`items`");
    expect(explorerSelect("dbo", "items", "sqlserver")).toBe(
      "SELECT TOP (100) *\nFROM [dbo].[items];",
    );
  });
});
