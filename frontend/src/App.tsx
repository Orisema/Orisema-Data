import { lazy, Suspense, useState } from "react";
import { useMutation, useQuery } from "@tanstack/react-query";
import { api } from "./api";
import type { Connection, QueryResult } from "./types";
import ConnectionManager from "./components/ConnectionManager";
import Explorer from "./components/Explorer";
import QueryResults from "./components/QueryResults";
const SqlEditor = lazy(() => import("./components/SqlEditor"));

function Workspace({ connection }: { connection?: Connection }) {
  const [sql, setSql] = useState(
    "SELECT\n  current_database() AS database,\n  current_user AS username,\n  version() AS version;",
  );
  const [limit, setLimit] = useState("");
  const query = useMutation({
    mutationFn: (text: string) =>
      api<QueryResult>(`/connections/${connection?.id}/query`, "POST", {
        sql: text,
        ...(limit ? { max_rows: Number(limit) } : {}),
      }),
  });
  const test = useMutation({
    mutationFn: () => api(`/connections/${connection?.id}/test`, "POST"),
  });
  return (
    <>
      <div className="connection-bar">
        <span className={`dot ${connection ? "active" : ""}`} />
        <span>
          {connection
            ? `${connection.host}:${connection.port} / ${connection.database_name}`
            : "Nenhuma conexão selecionada"}
        </span>
        {connection && (
          <>
            <span className="badge">
              {connection.read_only ? "Somente leitura" : "Leitura e escrita"}
            </span>
            <button disabled={test.isPending} onClick={() => test.mutate()}>
              {test.isPending ? "Testando…" : "Testar conexão"}
            </button>
            {test.isSuccess && <span role="status">Conexão OK</span>}
            {test.error && (
              <span role="alert" className="error">
                {test.error.message}
              </span>
            )}
          </>
        )}
      </div>
      <main>
        <aside>
          {connection ? (
            <Explorer id={connection.id} onQuery={setSql} />
          ) : (
            <>
              <h2>Explorer</h2>
              <p>
                Adicione uma conexão PostgreSQL para explorar schemas, tabelas e
                views.
              </p>
            </>
          )}
        </aside>
        <div className="workspace">
          <div className="query-settings">
            <span>Editor SQL</span>
            <label>
              Limite de linhas{" "}
              <input
                aria-label="Limite de linhas"
                type="number"
                min="1"
                max="10000"
                placeholder="Padrão do servidor"
                value={limit}
                onChange={(e) => setLimit(e.target.value)}
              />
            </label>
          </div>
          <Suspense fallback={<p>Carregando editor…</p>}>
            <SqlEditor
              sql={sql}
              onChange={setSql}
              onRun={(text) => query.mutate(text)}
              busy={query.isPending}
              disabled={!connection}
            />
          </Suspense>
          <QueryResults
            key={query.submittedAt}
            result={query.data}
            error={query.error}
            busy={query.isPending}
          />
        </div>
      </main>
    </>
  );
}
export default function App() {
  const connections = useQuery({
    queryKey: ["connections"],
    queryFn: () => api<Connection[]>("/connections"),
  });
  const [selected, setSelected] = useState("");
  const [manager, setManager] = useState<"new" | "edit" | null>(null);
  const current = connections.data?.find((c) => c.id === selected);
  return (
    <div className="app">
      <header className="topbar">
        <div className="brand">
          <span className="brand-icon">R</span>
          <h1>
            Rust<span>DB</span>
          </h1>
          <span className="version">v0.1</span>
        </div>
        <div className="connection-controls">
          <select
            aria-label="Conexão ativa"
            value={current?.id ?? ""}
            onChange={(e) => setSelected(e.target.value)}
          >
            <option value="">Selecionar conexão</option>
            {connections.data?.map((c) => (
              <option key={c.id} value={c.id}>
                {c.name}
              </option>
            ))}
          </select>
          <button disabled={!current} onClick={() => setManager("edit")}>
            Editar
          </button>
          <button onClick={() => setManager("new")}>＋ Nova conexão</button>
        </div>
      </header>
      {connections.error && (
        <p role="alert" className="error result-message">
          Não foi possível carregar conexões: {connections.error.message}{" "}
          <button onClick={() => void connections.refetch()}>
            Tentar novamente
          </button>
        </p>
      )}
      {connections.isPending && <p role="status">Carregando conexões…</p>}
      <Workspace
        key={current ? `${current.id}:${current.updated_at}` : "none"}
        connection={current}
      />
      <footer className="statusbar">
        <span>PostgreSQL web SQL client</span>
        <span>MVP · Uso local confiável · Sem autenticação</span>
      </footer>
      {manager && (
        <ConnectionManager
          connection={manager === "edit" ? current : undefined}
          onClose={() => setManager(null)}
          onSaved={setSelected}
        />
      )}
    </div>
  );
}
