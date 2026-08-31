import Editor, { loader, type OnMount } from "@monaco-editor/react";
import * as monaco from "monaco-editor/esm/vs/editor/editor.api";
import "monaco-editor/esm/vs/basic-languages/sql/sql.contribution";
import EditorWorker from "monaco-editor/esm/vs/editor/editor.worker?worker";
import { useEffect, useRef } from "react";
self.MonacoEnvironment = { getWorker: () => new EditorWorker() };
loader.config({ monaco });

export default function SqlEditor({
  sql,
  onChange,
  onRun,
  onCancel,
  onSave,
  busy,
  disabled,
  theme,
  suggestions,
}: {
  sql: string;
  onChange: (value: string) => void;
  onRun: (value: string) => void;
  onCancel: () => void;
  onSave: () => void;
  busy: boolean;
  disabled: boolean;
  theme: "dark" | "light";
  suggestions: string[];
}) {
  const editor = useRef<monaco.editor.IStandaloneCodeEditor | null>(null);
  const runRef = useRef(onRun);
  runRef.current = onRun;
  const blocked = useRef(false);
  const suggestionsRef = useRef(suggestions);
  const completion = useRef<monaco.IDisposable | null>(null);
  suggestionsRef.current = suggestions;
  blocked.current = busy || disabled;
  const execute = () => {
    if (blocked.current) return;
    const instance = editor.current;
    const selection = instance?.getSelection();
    const selected = selection
      ? instance?.getModel()?.getValueInRange(selection)
      : "";
    const text = selected?.trim() ? selected : (instance?.getValue() ?? "");
    if (text.trim()) runRef.current(text);
  };
  const mount: OnMount = (instance) => {
    editor.current = instance;
    instance.addAction({
      id: "rustdb-run",
      label: "Executar SQL",
      keybindings: [monaco.KeyMod.CtrlCmd | monaco.KeyCode.Enter],
      run: execute,
    });
    completion.current?.dispose();
    completion.current = monaco.languages.registerCompletionItemProvider("sql", {
      triggerCharacters: [".", '"'],
      provideCompletionItems: (model, position) => {
        const word = model.getWordUntilPosition(position);
        const range = {
          startLineNumber: position.lineNumber,
          endLineNumber: position.lineNumber,
          startColumn: word.startColumn,
          endColumn: word.endColumn,
        };
        return {
          suggestions: suggestionsRef.current.map((label) => ({
            label,
            insertText: label,
            range,
            kind: monaco.languages.CompletionItemKind.Field,
          })),
        };
      },
    });
  };
  useEffect(() => () => completion.current?.dispose(), []);
  return (
    <section className="editor-pane">
      <div className="section-bar">
        <span>
          <span className="sql-label">SQL</span> Consulta
        </span>
        <div>
          <span className="shortcut">Ctrl + Enter</span>
          <button disabled={disabled} onClick={onSave}>Salvar consulta</button>
          {busy ? (
            <button className="cancel" onClick={onCancel}>■ Cancelar</button>
          ) : (
            <button className="primary" disabled={disabled} onClick={execute}>
              ▶ Executar
            </button>
          )}
        </div>
      </div>
      <Editor
        height="100%"
        language="sql"
        theme={theme === "dark" ? "vs-dark" : "light"}
        value={sql}
        onChange={(v) => onChange(v ?? "")}
        onMount={mount}
        loading={<p>Carregando editor…</p>}
        options={{
          fontSize: 14,
          minimap: { enabled: false },
          automaticLayout: true,
          scrollBeyondLastLine: false,
          padding: { top: 18 },
          wordWrap: "on",
          ariaLabel: "Editor SQL",
          tabSize: 2,
        }}
      />
    </section>
  );
}
