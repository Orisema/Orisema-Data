import { useState } from 'react';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { api } from '../api';
import type { Connection, ConnectionInput } from '../types';

export default function ConnectionManager({connection, onClose, onSaved}: {connection?:Connection; onClose:()=>void; onSaved:(id:string)=>void}) {
  const client = useQueryClient();
  const [form, setForm] = useState<ConnectionInput>(() => connection ? {name:connection.name,database_type:'postgresql',host:connection.host,port:connection.port,database_name:connection.database_name,username:connection.username,read_only:connection.read_only,ssl_mode:connection.ssl_mode} : {name:'',database_type:'postgresql',host:'localhost',port:5432,database_name:'',username:'',password:'',read_only:true,ssl_mode:'verify_full'});
  const [replacePassword, setReplacePassword] = useState(!connection);
  const save = useMutation({mutationFn:()=>api<Connection>(`/connections${connection ? `/${connection.id}` : ''}`,connection?'PUT':'POST',form),onSuccess:async c=>{await client.invalidateQueries({queryKey:['connections']});onSaved(c.id);onClose();}});
  const remove = useMutation({mutationFn:()=>api(`/connections/${connection?.id}`,'DELETE'),onSuccess:async()=>{await client.invalidateQueries({queryKey:['connections']});onSaved('');onClose();}});
  const busy=save.isPending || remove.isPending;
  return <div className="modal-backdrop"><section role="dialog" aria-modal="true" aria-labelledby="connection-title" className="dialog">
    <header><h2 id="connection-title">{connection?'Editar conexão':'Nova conexão'}</h2><button onClick={onClose} disabled={busy} aria-label="Fechar">×</button></header>
    <p>PostgreSQL · Credenciais armazenadas com criptografia.</p>
    <form onSubmit={e=>{e.preventDefault();save.mutate();}}>
      <label>Nome<input autoFocus required maxLength={128} value={form.name} onChange={e=>setForm({...form,name:e.target.value})}/></label>
      <div className="form-row"><label>Host<input required maxLength={253} value={form.host} onChange={e=>setForm({...form,host:e.target.value})}/></label><label>Porta<input type="number" required min={1} max={65535} value={form.port} onChange={e=>setForm({...form,port:Number(e.target.value)})}/></label></div>
      <label>Banco de dados<input required maxLength={128} value={form.database_name} onChange={e=>setForm({...form,database_name:e.target.value})}/></label>
      <label>Usuário<input required maxLength={128} autoComplete="off" value={form.username} onChange={e=>setForm({...form,username:e.target.value})}/></label>
      {connection && <label className="check"><input type="checkbox" checked={replacePassword} onChange={e=>{setReplacePassword(e.target.checked);setForm({...form,password:e.target.checked?'':undefined});}}/>Substituir senha armazenada</label>}
      {replacePassword && <label>Senha<input type="password" maxLength={4096} autoComplete="new-password" value={form.password ?? ''} onChange={e=>setForm({...form,password:e.target.value})}/></label>}
      <label>TLS<select value={form.ssl_mode} onChange={e=>setForm({...form,ssl_mode:e.target.value as ConnectionInput['ssl_mode']})}><option value="verify_full">Verificar certificado e hostname</option><option value="disable">Desabilitado (apenas rede local confiável)</option></select></label>
      <label className="check"><input type="checkbox" checked={form.read_only} onChange={e=>setForm({...form,read_only:e.target.checked})}/>Somente leitura</label>
      {!form.read_only && <p className="warning">INSERT, UPDATE e DELETE serão permitidos. Use uma conta com permissões mínimas.</p>}
      {(save.error || remove.error) && <p role="alert" className="error">{(save.error || remove.error)?.message}</p>}
      <footer>{connection && <button className="danger" type="button" disabled={busy} onClick={()=>{if(window.confirm(`Excluir a conexão “${connection.name}”?`)) remove.mutate();}}>Excluir</button>}<button type="button" onClick={onClose} disabled={busy}>Cancelar</button><button className="primary" disabled={busy}>{save.isPending?'Salvando…':'Salvar conexão'}</button></footer>
    </form>
  </section></div>;
}
