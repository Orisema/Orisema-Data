import type { QueryResult } from "./types";

const csvCell = (value: string | number | boolean | null) => {
  const raw = value === null ? "" : String(value);
  // Prevent spreadsheet applications from interpreting exported text as a formula.
  const text = typeof value === "string" && /^[=+\-@\t\r]/.test(value)
    ? `'${raw}`
    : raw;
  return /[",\r\n]/.test(text) ? `"${text.replaceAll('"', '""')}"` : text;
};

export function queryResultCsv(result: QueryResult) {
  const header = result.columns.map((column) => csvCell(column.name)).join(",");
  const rows = result.rows.map((row) => row.map(csvCell).join(","));
  return `\uFEFF${[header, ...rows].join("\r\n")}`;
}

export function downloadCsv(result: QueryResult) {
  const blob = new Blob([queryResultCsv(result)], {
    type: "text/csv;charset=utf-8",
  });
  const url = URL.createObjectURL(blob);
  const link = document.createElement("a");
  link.href = url;
  link.download = `rustdb-${new Date().toISOString().replaceAll(":", "-")}.csv`;
  link.click();
  URL.revokeObjectURL(url);
}
