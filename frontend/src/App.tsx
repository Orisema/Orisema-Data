import { lazy, Suspense, useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useMutation, useQuery } from "@tanstack/react-query";
import { api, ApiError } from "./api";
import type { Connection, QueryResult } from "./types";
import {
  createTab, historyWith, loadWorkbench, newId, saveWorkbench,
  type EditorTab, type QueryHistoryItem, type WorkbenchData,
} from "./workbench";
import ConnectionManager from "./components/ConnectionManager";
import Explorer from "./components/Explorer";
import QueryLibrary from "./components/QueryLibrary";
import QueryResults from "./components/QueryResults";
import SecurityPanel from "./components/SecurityPanel";
import PolicyPanel from "./components/PolicyPanel";
import type { CurrentUser } from "./auth";
const SqlEditor = lazy(() => import("./components/SqlEditor"));

const SQL_WORDS = [
  "SELECT", "FROM", "WHERE", "JOIN", "LEFT JOIN", "INNER JOIN", "GROUP BY",
  "ORDER BY", "LIMIT", "OFFSET", "INSERT INTO", "UPDATE", "DELETE FROM",
  "RETURNING", "WITH", "AS", "AND", "OR", "NOT", "NULL", "TRUE", "FALSE",
  "COUNT", "SUM", "AVG", "MIN", "MAX", "TOP", "SHOW",
];

function Workspace({
  connection, activeTab, workbench, updateWorkbench, onSql,
}: {
  connection?: Connection;
  activeTab: EditorTab;
  workbench: WorkbenchData;
  updateWorkbench: React.Dispatch<React.SetStateAction<WorkbenchData>>;
  onSql: (sql: string) => void;
}) {
  const [limit, setLimit] = useState("");
  const [libraryOpen, setLibraryOpen] = useState(false);
  const [metadata, setMetadata] = useState<Record<string, string[]>>({});
  const [saveDraft, setSaveDraft] = useState<string | null>(null);
  const currentQueryId = useRef<string | null>(null);

  const addHistory = useCallback((
    variables: { sql: string; connectionId: string; connectionName: string },
    status: QueryHistoryItem["status"],
    durationMs?: number,
  ) => {
    updateWorkbench((state) => ({
      ...state,
      history: historyWith(state.history, {
        id: newId(), connectionId: variables.connectionId,
        connectionName: variables.connectionName, sql: variables.sql,
        executedAt: new Date().toISOString(), durationMs, status, favorite: false,
      }),
    }));
  }, [updateWorkbench]);

  const query = useMutation({
    mutationFn: (variables: {
      sql: string; queryId: string; connectionId: string; connectionName: string;
    }) => api<QueryResult>(`/connections/${variables.connectionId}/query`, "POST", {
      sql: variables.sql, query_id: variables.queryId,
      ...(limit ? { max_rows: Number(limit) } : {}),
    }),
    onSuccess: (result, variables) =>
      addHistory(variables, "success", result.execution_time_ms),
    onError: (error, variables) =>
      addHistory(
        variables,
        error instanceof ApiError && error.code === "query_cancelled" ? "cancelled" : "error",
      ),
    onSettled: () => { currentQueryId.current = null; },
  });

  const test = useMutation({
    mutationFn: () => api(`/connections/${connection?.id}/test`, "POST"),
  });

  const run = (sql: string) => {
    if (!connection || query.isPending) return;
    const queryId = newId();
    currentQueryId.current = queryId;
    query.mutate({
      sql, queryId, connectionId: connection.id, connectionName: connection.name,
    });
  };

  const cancel = async () => {
    const id = currentQueryId.current;
    if (!id) return;
    try {
      await api(`/queries/${id}`, "DELETE");
    } catch (error) {
      if (!(error instanceof ApiError && error.code === "not_found")) throw error;
    }
  };

  const saveCurrent = () => {
    setSaveDraft(activeTab.title.startsWith("Consulta") ? "" : activeTab.title);
  };

  const confirmSave = () => {
    const name = saveDraft?.trim();
    if (!name) return;
    const now = new Date().toISOString();
    updateWorkbench((state) => ({
      ...state,
      saved: [{
        id: newId(), name, sql: activeTab.sql, createdAt: now, updatedAt: now,
      }, ...state.saved],
    }));
    setSaveDraft(null);
  };

  const openSql = (sql: string, title?: string, connectionId?: string) => {
    const tab = {
      ...createTab(connectionId ?? connection?.id ?? "", workbench.tabs.length + 1),
      sql, ...(title ? { title } : {}),
    };
    updateWorkbench((state) => ({
      ...state, tabs: [...state.tabs, tab], activeTabId: tab.id,
    }));
  };

  const suggestions = useMemo(
    () => [...new Set([...SQL_WORDS, ...(metadata[connection?.id ?? ""] ?? [])])],
    [connection?.id, metadata],
  );
  const registerMetadata = useCallback((values: string[]) => {
    if (!connection) return;
    setMetadata((current) => ({
      ...current,
      [connection.id]: [...new Set([...(current[connection.id] ?? []), ...values])].slice(-5000),
    }));
  }, [connection]);

  return (
    <>
      <div className="connection-bar">
        <span className={`dot ${connection ? "active" : ""}`} />
        <span>{connection
          ? `${connection.host}:${connection.port} / ${connection.database_name}`
          : "Nenhuma conexão selecionada"}</span>
        {connection && (
          <>
            <span className="badge">
              {connection.read_only ? "Somente leitura" : "Leitura e escrita"}
            </span>
            <button disabled={test.isPending} onClick={() => test.mutate()}>
              {test.isPending ? "Testando…" : "Testar conexão"}
            </button>
            {test.isSuccess && <span role="status">Conexão OK</span>}
            {test.error && <span role="alert" className="error">{test.error.message}</span>}
          </>
        )}
      </div>
      <main className={libraryOpen ? "with-library" : ""}>
        <aside>
          {connection ? (
            <Explorer id={connection.id} databaseType={connection.database_type} onQuery={onSql} onMetadata={registerMetadata} />
          ) : (
            <><h2>Explorer</h2><p>Adicione uma conexão para explorar schemas, tabelas e views.</p></>
          )}
        </aside>
        <div className="workspace">
          <div className="query-settings">
            <div>
              <button onClick={() => setLibraryOpen(!libraryOpen)}>☰ Histórico e salvas</button>
              <label>
                Altura do editor{" "}
                <input
                  aria-label="Altura do editor" type="range" min="25" max="70"
                  value={workbench.editorPercent}
                  onChange={(event) => updateWorkbench((state) => ({
                    ...state, editorPercent: Number(event.target.value),
                  }))}
                />
              </label>
            </div>
            <label>
              Limite de linhas{" "}
              <input
                aria-label="Limite de linhas" type="number" min="1" max="10000"
                placeholder="Padrão do servidor" value={limit}
                onChange={(event) => setLimit(event.target.value)}
              />
            </label>
          </div>
          <Suspense fallback={<p>Carregando editor…</p>}>
            <div className="editor-size" style={{ height: `${workbench.editorPercent}%` }}>
              <SqlEditor
                sql={activeTab.sql} onChange={onSql} onRun={run}
                onCancel={() => void cancel()} onSave={saveCurrent}
                busy={query.isPending} disabled={!connection}
                theme={workbench.theme} suggestions={suggestions}
              />
            </div>
          </Suspense>
          <QueryResults
            key={query.submittedAt} result={query.data}
            error={query.error} busy={query.isPending}
          />
        </div>
        {libraryOpen && (
          <QueryLibrary
            history={workbench.history} saved={workbench.saved} onOpen={openSql}
            onToggleFavorite={(id) => updateWorkbench((state) => ({
              ...state,
              history: state.history.map((item) =>
                item.id === id ? { ...item, favorite: !item.favorite } : item),
            }))}
            onDeleteSaved={(id) => updateWorkbench((state) => ({
              ...state, saved: state.saved.filter((item) => item.id !== id),
            }))}
            onClearHistory={() => updateWorkbench((state) => ({
              ...state, history: state.history.filter((item) => item.favorite),
            }))}
            onClose={() => setLibraryOpen(false)}
          />
        )}
      </main>
      {saveDraft !== null && (
        <div className="modal-backdrop">
          <form
            className="dialog compact-dialog"
            aria-label="Salvar consulta"
            onSubmit={(event) => {
              event.preventDefault();
              confirmSave();
            }}
          >
            <header>
              <h2>Salvar consulta</h2>
              <button type="button" aria-label="Fechar" onClick={() => setSaveDraft(null)}>×</button>
            </header>
            <label>
              Nome
              <input
                autoFocus
                aria-label="Nome da consulta"
                value={saveDraft}
                onChange={(event) => setSaveDraft(event.target.value)}
              />
            </label>
            <footer>
              <button type="button" onClick={() => setSaveDraft(null)}>Cancelar</button>
              <button className="primary" disabled={!saveDraft.trim()} type="submit">Salvar</button>
            </footer>
          </form>
        </div>
      )}
    </>
  );
}
export default function App({ user, onLogout }: { user: CurrentUser; onLogout: () => Promise<void> }) {
  const connections = useQuery({
    queryKey: ["connections"], queryFn: () => api<Connection[]>("/connections"),
  });
  const [workbench, setWorkbench] = useState(loadWorkbench);
  const [manager, setManager] = useState<"new" | "edit" | null>(null);
  const [securityOpen, setSecurityOpen] = useState(false);
  const [policyOpen, setPolicyOpen] = useState(false);
  const [renameDraft, setRenameDraft] = useState<{ id: string; title: string } | null>(null);
  const activeTab =
    workbench.tabs.find((tab) => tab.id === workbench.activeTabId) ?? workbench.tabs[0];
  const selected = activeTab.connectionId;
  const current = connections.data?.find((connection) => connection.id === selected);

  useEffect(() => saveWorkbench(workbench), [workbench]);

  const updateTab = useCallback((sql: string) => {
    setWorkbench((state) => ({
      ...state,
      tabs: state.tabs.map((tab) =>
        tab.id === state.activeTabId
          ? { ...tab, sql, updatedAt: new Date().toISOString() } : tab),
    }));
  }, []);

  const selectConnection = (connectionId: string) => {
    setWorkbench((state) => ({
      ...state,
      tabs: state.tabs.map((tab) =>
        tab.id === state.activeTabId ? { ...tab, connectionId } : tab),
    }));
  };

  const addTab = () => {
    const tab = createTab(selected, workbench.tabs.length + 1);
    setWorkbench((state) => ({
      ...state, tabs: [...state.tabs, tab], activeTabId: tab.id,
    }));
  };

  const closeTab = (id: string) => {
    setWorkbench((state) => {
      const index = state.tabs.findIndex((tab) => tab.id === id);
      const tabs = state.tabs.filter((tab) => tab.id !== id);
      if (tabs.length === 0) {
        const tab = createTab(selected);
        return { ...state, tabs: [tab], activeTabId: tab.id };
      }
      return {
        ...state, tabs,
        activeTabId: state.activeTabId === id
          ? tabs[Math.max(0, index - 1)].id : state.activeTabId,
      };
    });
  };

  const activateTab = (tab: EditorTab) => {
    setWorkbench((state) => ({ ...state, activeTabId: tab.id }));
  };

  const renameTab = (tab: EditorTab) => {
    setRenameDraft({ id: tab.id, title: tab.title });
  };

  const confirmRename = () => {
    const draft = renameDraft;
    if (!draft) return;
    const title = draft.title.trim();
    if (!title) return;
    setWorkbench((state) => ({
      ...state,
      tabs: state.tabs.map((item) =>
        item.id === draft.id ? { ...item, title } : item),
    }));
    setRenameDraft(null);
  };

  return (
    <div className="app" data-theme={workbench.theme}>
      <header className="topbar">
        <div className="brand">
          <span className="brand-icon">O</span>
          <h1>Orisema <span>Data</span></h1>
          <span className="version">v0.4</span>
        </div>
        <div className="connection-controls">
          <span>{user.display_name} · {user.role}</span>
          {user.role === "admin" && <button onClick={() => setSecurityOpen(true)}>Usuários e auditoria</button>}
          <button onClick={() => void onLogout()}>Sair</button>
          <button
            aria-label="Alternar tema"
            onClick={() => setWorkbench((state) => ({
              ...state, theme: state.theme === "dark" ? "light" : "dark",
            }))}
          >
            {workbench.theme === "dark" ? "☀ Tema claro" : "☾ Tema escuro"}
          </button>
          <select
            aria-label="Conexão ativa" value={current?.id ?? ""}
            onChange={(event) => selectConnection(event.target.value)}
          >
            <option value="">Selecionar conexão</option>
            {connections.data?.map((connection) => (
              <option key={connection.id} value={connection.id}>{connection.name}</option>
            ))}
          </select>
          <button disabled={!current || user.role !== "admin"} onClick={() => setManager("edit")}>Editar</button>
          <button disabled={!current || user.role !== "admin"} onClick={() => setPolicyOpen(true)}>Política</button>
          {user.role === "admin" && <button onClick={() => setManager("new")}>＋ Nova conexão</button>}
        </div>
      </header>
      <nav className="editor-tabs" aria-label="Abas de consulta">
        {workbench.tabs.map((tab) => (
          <div className={tab.id === activeTab.id ? "active" : ""} key={tab.id}>
            <button
              className="tab-title" title="Clique duas vezes para renomear"
              onClick={() => activateTab(tab)} onDoubleClick={() => renameTab(tab)}
            >{tab.title}</button>
            <button aria-label={`Fechar ${tab.title}`} onClick={() => closeTab(tab.id)}>×</button>
          </div>
        ))}
        <button aria-label="Nova aba SQL" onClick={addTab}>＋</button>
      </nav>
      {connections.error && (
        <p role="alert" className="error result-message">
          Não foi possível carregar conexões: {connections.error.message}{" "}
          <button onClick={() => void connections.refetch()}>Tentar novamente</button>
        </p>
      )}
      {connections.isPending && <p role="status">Carregando conexões…</p>}
      <Workspace
        connection={current} activeTab={activeTab} workbench={workbench}
        updateWorkbench={setWorkbench} onSql={updateTab}
      />
      <footer className="statusbar">
        <span>Cliente SQL web multi-banco</span>
        <span>v0.4 · Sessão autenticada · Dados locais do editor</span>
      </footer>
      {securityOpen && <SecurityPanel currentUser={user} onClose={() => setSecurityOpen(false)} />}
      {policyOpen && current && <PolicyPanel connectionId={current.id} onClose={() => setPolicyOpen(false)} />}
      {manager && (
        <ConnectionManager
          connection={manager === "edit" ? current : undefined}
          onClose={() => setManager(null)}
          onSaved={(id) => selectConnection(id)}
        />
      )}
      {renameDraft && (
        <div className="modal-backdrop">
          <form
            className="dialog compact-dialog"
            aria-label="Renomear aba"
            onSubmit={(event) => {
              event.preventDefault();
              confirmRename();
            }}
          >
            <header>
              <h2>Renomear aba</h2>
              <button type="button" aria-label="Fechar" onClick={() => setRenameDraft(null)}>×</button>
            </header>
            <label>
              Nome
              <input
                autoFocus
                aria-label="Nome da aba"
                value={renameDraft.title}
                onChange={(event) =>
                  setRenameDraft({ ...renameDraft, title: event.target.value })}
              />
            </label>
            <footer>
              <button type="button" onClick={() => setRenameDraft(null)}>Cancelar</button>
              <button className="primary" disabled={!renameDraft.title.trim()} type="submit">
                Renomear
              </button>
            </footer>
          </form>
        </div>
      )}
    </div>
  );
}
