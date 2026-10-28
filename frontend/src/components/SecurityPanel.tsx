import { useState } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { api } from "../api";
import type { CurrentUser, Role } from "../auth";

interface Audit {
  id: string;
  event_type: string;
  actor_user_id: string | null;
  target_type: string | null;
  success: boolean;
  created_at: string;
}

export default function SecurityPanel({ onClose, currentUser }: { onClose: () => void; currentUser: CurrentUser }) {
  const client = useQueryClient();
  const users = useQuery({ queryKey: ["users"], queryFn: () => api<CurrentUser[]>("/users") });
  const audit = useQuery({ queryKey: ["audit"], queryFn: () => api<Audit[]>("/audit") });
  const [email, setEmail] = useState("");
  const [name, setName] = useState("");
  const [password, setPassword] = useState("");
  const [role, setRole] = useState<Role>("viewer");
  const [error, setError] = useState("");
  const refresh = async () => {
    await client.invalidateQueries({ queryKey: ["users"] });
    await client.invalidateQueries({ queryKey: ["audit"] });
  };
  const updateUser = async (user: CurrentUser, nextRole: Role, active: boolean) => {
    setError("");
    try {
      await api(`/users/${user.id}`, "PUT", {
        email: user.email, display_name: user.display_name, role: nextRole, active,
      });
      await refresh();
    } catch (caught) { setError((caught as Error).message); }
  };
  const remove = async (id: string) => {
    setError("");
    try { await api(`/users/${id}`, "DELETE"); await refresh(); }
    catch (caught) { setError((caught as Error).message); }
  };
  const create = async (event: React.FormEvent) => {
    event.preventDefault(); setError("");
    try {
      await api("/users", "POST", { email, display_name: name, password, role, active: true });
      setEmail(""); setName(""); setPassword(""); await refresh();
    } catch (caught) { setError((caught as Error).message); }
  };
  return <div className="modal-backdrop"><section className="dialog security-dialog">
    <header><h2>Segurança e auditoria</h2><button aria-label="Fechar" onClick={onClose}>×</button></header>
    <form onSubmit={create}><h3>Novo usuário</h3>
      <label>Nome<input required value={name} onChange={(e) => setName(e.target.value)} /></label>
      <label>E-mail<input required type="email" value={email} onChange={(e) => setEmail(e.target.value)} /></label>
      <label>Senha<input required minLength={12} type="password" value={password} onChange={(e) => setPassword(e.target.value)} /></label>
      <label>Papel<select value={role} onChange={(e) => setRole(e.target.value as Role)}><option value="viewer">Visualizador</option><option value="editor">Editor</option><option value="admin">Administrador</option></select></label>
      {error && <p className="error">{error}</p>}<button className="primary">Criar usuário</button>
    </form>
    <section><h3>Usuários</h3>{users.data?.map((listed) => <p key={listed.id}><strong>{listed.display_name}</strong> · {listed.email} · <select aria-label={`Papel de ${listed.email}`} value={listed.role} disabled={listed.id === currentUser.id} onChange={(e) => void updateUser(listed, e.target.value as Role, listed.active)}><option value="viewer">viewer</option><option value="editor">editor</option><option value="admin">admin</option></select> <button onClick={() => void updateUser(listed, listed.role, !listed.active)} disabled={listed.id === currentUser.id}>{listed.active ? "Desativar" : "Ativar"}</button> <button className="danger" disabled={listed.id === currentUser.id} onClick={() => void remove(listed.id)}>Excluir</button></p>)}</section>
    <section><h3>Auditoria recente</h3><div className="audit-list">{audit.data?.map((item) => <p key={item.id}><time>{new Date(item.created_at).toLocaleString()}</time> · {item.event_type} · {item.success ? "sucesso" : "falha"}</p>)}</div></section>
  </section></div>;
}
