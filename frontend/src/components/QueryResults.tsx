import { useState } from 'react';
import type { QueryResult } from '../types';
import { cellText } from '../format';

export default function QueryResults({result,error,busy}:{result?:QueryResult;error:Error|null;busy:boolean}) {
  const [page,setPage]=useState(0);
  const [notice,setNotice]=useState('');
  const size=100;
  const last=Math.max(0,Math.ceil((result?.rows.length ?? 0)/size)-1);
  const current=Math.min(page,last);
  const copy=async (value:string)=>{try {await navigator.clipboard.writeText(value);setNotice('Copiado');}catch {setNotice('O navegador bloqueou a área de transferência.');}};
  return <section className="results-pane" aria-busy={busy}><div className="section-bar"><strong>Resultados</strong>{result && <span className="metrics">Rows: {result.row_count} · Execution: {result.execution_time_ms}ms{result.affected_rows>0 && ` · Afetadas: ${result.affected_rows}`}</span>}</div>
    {error && <p role="alert" className="error result-message">{error.message}</p>}
    {busy && <p role="status" className="result-message">Executando consulta…</p>}
    {!result && !error && !busy && <div className="empty"><span>▦</span><h3>Seu próximo resultado começa aqui</h3><p>Escolha uma conexão e execute uma consulta.<br/>Selecione um trecho para executar apenas a seleção.</p></div>}
    {result && <><div className="grid-scroll"><table><thead><tr><th aria-label="Linha">#</th>{result.columns.map((c,i)=><th key={i}>{c.name}<small>{c.type}</small></th>)}</tr></thead><tbody>{result.rows.slice(current*size,(current+1)*size).map((row,r)=><tr key={r}><td><button aria-label={`Copiar linha ${current*size+r+1}`} title="Copiar linha (JSON)" onClick={()=>void copy(JSON.stringify(row))}>{current*size+r+1}</button></td>{row.map((cell,c)=><td key={c}><button className={`cell ${cell===null?'null':''}`} title="Copiar célula" onClick={()=>void copy(cellText(cell))}>{cellText(cell)}</button></td>)}</tr>)}</tbody></table>{result.row_count===0 && <p className="result-message">Consulta concluída sem linhas de resultado.</p>}</div><div className="results-footer"><span role="status">{notice || (result.truncated?'Limite atingido: há mais linhas no banco.':'Consulta concluída')}</span><div><button disabled={current===0} onClick={()=>setPage(current-1)}>Anterior</button><span> {current+1} / {last+1} </span><button disabled={current===last} onClick={()=>setPage(current+1)}>Próxima</button></div></div></>}
  </section>;
}
