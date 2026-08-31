import { useEffect, useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { api, segment } from "../api";
import type { ColumnInfo, TableInfo } from "../types";
const quote = (name: string) => `"${name.replaceAll('"', '""')}"`;

function Table({
  id,
  schema,
  table,
  onQuery,
  onMetadata,
}: {
  id: string;
  schema: string;
  table: TableInfo;
  onQuery: (sql: string) => void;
  onMetadata: (values: string[]) => void;
}) {
  const [open, setOpen] = useState(false);
  const columns = useQuery({
    queryKey: ["columns", id, schema, table.name],
    queryFn: () =>
      api<ColumnInfo[]>(
        `/connections/${id}/schemas/${segment(schema)}/tables/${segment(table.name)}/columns`,
      ),
    enabled: open,
  });
  useEffect(() => {
    if (columns.data) onMetadata(columns.data.map((column) => column.name));
  }, [columns.data, onMetadata]);
  return (
    <li>
      <div className="tree-line">
        <button
          className="tree-button"
          aria-expanded={open}
          onClick={() => setOpen(!open)}
        >
          {open ? "▾" : "▸"}{" "}
          <span className="table-icon">
            {table.kind === "view" ? "◈" : "▦"}
          </span>{" "}
          {table.name}
        </button>
        <button
          title="Abrir SELECT no editor"
          aria-label={`Consultar ${table.name}`}
          onClick={() =>
            onQuery(
              `SELECT *\nFROM ${quote(schema)}.${quote(table.name)}\nLIMIT 100;`,
            )
          }
        >
          ↗
        </button>
      </div>
      {open && (
        <ul>
          {columns.isPending && <li>Carregando colunas…</li>}
          {columns.error && <li className="error">{columns.error.message}</li>}
          {columns.data?.map((c) => (
            <li className="column" key={c.name}>
              <span>{c.name}</span>
              <small>
                {c.type}
                {c.nullable ? " ?" : ""}
              </small>
            </li>
          ))}
        </ul>
      )}
    </li>
  );
}
function Schema({
  id,
  name,
  onQuery,
  onMetadata,
}: {
  id: string;
  name: string;
  onQuery: (sql: string) => void;
  onMetadata: (values: string[]) => void;
}) {
  const [open, setOpen] = useState(false);
  const tables = useQuery({
    queryKey: ["tables", id, name],
    queryFn: () =>
      api<TableInfo[]>(`/connections/${id}/schemas/${segment(name)}/tables`),
    enabled: open,
  });
  useEffect(() => {
    if (tables.data) onMetadata(tables.data.map((table) => table.name));
  }, [tables.data, onMetadata]);
  return (
    <li>
      <button
        className="tree-button schema"
        aria-expanded={open}
        onClick={() => setOpen(!open)}
      >
        {open ? "▾" : "▸"} ◇ {name}
      </button>
      {open && (
        <ul>
          {tables.isPending && <li>Carregando tabelas…</li>}
          {tables.error && <li className="error">{tables.error.message}</li>}
          {["table", "view"].map((kind) => (
            <li key={kind}>
              <div className="tree-heading">
                {kind === "table" ? "Tabelas" : "Views"}
              </div>
              <ul>
                {tables.data
                  ?.filter((t) => t.kind === kind)
                  .map((t) => (
                    <Table
                      key={t.name}
                      id={id}
                      schema={name}
                      table={t}
                      onQuery={onQuery}
                      onMetadata={onMetadata}
                    />
                  ))}
              </ul>
            </li>
          ))}
        </ul>
      )}
    </li>
  );
}
export default function Explorer({
  id,
  onQuery,
  onMetadata,
}: {
  id: string;
  onQuery: (sql: string) => void;
  onMetadata: (values: string[]) => void;
}) {
  const schemas = useQuery({
    queryKey: ["schemas", id],
    queryFn: () => api<string[]>(`/connections/${id}/schemas`),
  });
  useEffect(() => {
    if (schemas.data) onMetadata(schemas.data);
  }, [schemas.data, onMetadata]);
  return (
    <>
      <div className="explorer-title">
        <h2>Explorer</h2>
        <button
          onClick={() => void schemas.refetch()}
          aria-label="Atualizar schemas"
        >
          ↻
        </button>
      </div>
      {schemas.isPending && <p>Conectando ao banco…</p>}
      {schemas.error && (
        <p role="alert" className="error">
          {schemas.error.message}
        </p>
      )}
      <ul className="tree">
        {schemas.data?.map((name) => (
          <Schema key={name} id={id} name={name} onQuery={onQuery} onMetadata={onMetadata} />
        ))}
      </ul>
      {schemas.data?.length === 0 && (
        <p>Nenhum schema visível para esta conta.</p>
      )}
    </>
  );
}
