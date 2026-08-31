import { useMemo, useState } from "react";
import type { QueryHistoryItem, SavedQuery } from "../workbench";

const summary = (sql: string) => sql.replace(/\s+/g, " ").trim();

export default function QueryLibrary({
  history,
  saved,
  onOpen,
  onToggleFavorite,
  onDeleteSaved,
  onClearHistory,
  onClose,
}: {
  history: QueryHistoryItem[];
  saved: SavedQuery[];
  onOpen: (sql: string, title?: string, connectionId?: string) => void;
  onToggleFavorite: (id: string) => void;
  onDeleteSaved: (id: string) => void;
  onClearHistory: () => void;
  onClose: () => void;
}) {
  const [view, setView] = useState<"history" | "saved">("history");
  const [search, setSearch] = useState("");
  const items = useMemo(() => {
    const needle = search.trim().toLocaleLowerCase();
    const source = view === "history" ? history : saved;
    if (!needle) return source;
    return source.filter((item) =>
      `${"name" in item ? item.name : item.connectionName} ${item.sql}`
        .toLocaleLowerCase()
        .includes(needle),
    );
  }, [history, saved, search, view]);

  return (
    <section className="library" aria-label="Biblioteca SQL">
      <header>
        <strong>Biblioteca SQL</strong>
        <button aria-label="Fechar biblioteca" onClick={onClose}>×</button>
      </header>
      <div className="library-tabs" role="tablist">
        <button
          role="tab"
          aria-selected={view === "history"}
          onClick={() => setView("history")}
        >
          Histórico ({history.length})
        </button>
        <button
          role="tab"
          aria-selected={view === "saved"}
          onClick={() => setView("saved")}
        >
          Salvas ({saved.length})
        </button>
      </div>
      <input
        type="search"
        aria-label="Buscar consultas"
        placeholder="Buscar SQL ou nome…"
        value={search}
        onChange={(event) => setSearch(event.target.value)}
      />
      <div className="library-list">
        {items.map((item) => (
          <article key={item.id}>
            <div>
              <strong>{"name" in item ? item.name : item.connectionName}</strong>
              <small>
                {new Date("executedAt" in item ? item.executedAt : item.updatedAt)
                  .toLocaleString()}
              </small>
            </div>
            <button
              className="sql-preview"
              title="Abrir em nova aba"
              onClick={() => onOpen(
                item.sql,
                "name" in item ? item.name : undefined,
                "connectionId" in item ? item.connectionId : undefined,
              )}
            >
              {summary(item.sql)}
            </button>
            <footer>
              {"status" in item ? (
                <>
                  <span className={`history-status ${item.status}`}>
                    {item.status === "success" ? "Concluída" : item.status === "cancelled" ? "Cancelada" : "Erro"}
                    {item.durationMs !== undefined && ` · ${item.durationMs}ms`}
                  </span>
                  <button
                    aria-label={item.favorite ? "Remover dos favoritos" : "Adicionar aos favoritos"}
                    onClick={() => onToggleFavorite(item.id)}
                  >
                    {item.favorite ? "★" : "☆"}
                  </button>
                </>
              ) : (
                <button className="danger" onClick={() => onDeleteSaved(item.id)}>
                  Excluir
                </button>
              )}
            </footer>
          </article>
        ))}
        {items.length === 0 && <p>Nenhuma consulta encontrada.</p>}
      </div>
      {view === "history" && history.length > 0 && (
        <button className="library-clear" onClick={onClearHistory}>
          Limpar histórico
        </button>
      )}
    </section>
  );
}
