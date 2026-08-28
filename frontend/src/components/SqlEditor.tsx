import Editor, { loader, type OnMount } from '@monaco-editor/react';
import * as monaco from 'monaco-editor/esm/vs/editor/editor.api';
import 'monaco-editor/esm/vs/basic-languages/sql/sql.contribution';
import EditorWorker from 'monaco-editor/esm/vs/editor/editor.worker?worker';
import { useRef } from 'react';
self.MonacoEnvironment = {getWorker:()=>new EditorWorker()};
loader.config({monaco});

export default function SqlEditor({sql,onChange,onRun,busy,disabled}:{sql:string;onChange:(value:string)=>void;onRun:(value:string)=>void;busy:boolean;disabled:boolean}) {
  const editor=useRef<monaco.editor.IStandaloneCodeEditor|null>(null);
  const runRef=useRef(onRun); runRef.current=onRun;
  const blocked=useRef(false); blocked.current=busy || disabled;
  const execute=()=>{
    if(blocked.current) return;
    const instance=editor.current;
    const selection=instance?.getSelection();
    const selected=selection ? instance?.getModel()?.getValueInRange(selection) : '';
    const text=selected?.trim() ? selected : instance?.getValue() ?? '';
    if(text.trim()) runRef.current(text);
  };
  const mount:OnMount=(instance)=>{editor.current=instance;instance.addAction({id:'rustdb-run',label:'Executar SQL',keybindings:[monaco.KeyMod.CtrlCmd|monaco.KeyCode.Enter],run:execute});};
  return <section className="editor-pane"><div className="section-bar"><span><span className="sql-label">SQL</span> Consulta</span><div><span className="shortcut">Ctrl + Enter</span><button className="primary" disabled={busy || disabled} onClick={execute}>{busy?'Executando…':'▶ Executar'}</button></div></div><Editor height="100%" language="sql" theme="vs-dark" value={sql} onChange={v=>onChange(v ?? '')} onMount={mount} loading={<p>Carregando editor…</p>} options={{fontSize:14,minimap:{enabled:false},automaticLayout:true,scrollBeyondLastLine:false,padding:{top:18},wordWrap:'on',ariaLabel:'Editor SQL',tabSize:2}}/></section>;
}
