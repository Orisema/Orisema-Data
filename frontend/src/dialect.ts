import type { Connection } from "./types";

type DatabaseType = Connection["database_type"];

export const quoteIdentifier = (name: string, databaseType: DatabaseType) => {
  if (databaseType === "mysql" || databaseType === "mariadb")
    return `\`${name.replaceAll("`", "``")}\``;
  if (databaseType === "sqlserver")
    return "[" + name.replaceAll("]", "]]") + "]";
  return `"${name.replaceAll('"', '""')}"`;
};

export const explorerSelect = (
  schema: string,
  table: string,
  databaseType: DatabaseType,
) => {
  const target = `${quoteIdentifier(schema, databaseType)}.${quoteIdentifier(table, databaseType)}`;
  return databaseType === "sqlserver"
    ? `SELECT TOP (100) *\nFROM ${target};`
    : `SELECT *\nFROM ${target}\nLIMIT 100;`;
};
