import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { api, ApiError } from "./api";

export type Role = "admin" | "editor" | "viewer";
export interface CurrentUser { id:string; email:string; display_name:string; role:Role; active:boolean }

export default function AuthGate({children}:{children:(user:CurrentUser,onLogout:()=>Promise<void>)=>React.ReactNode}){
  const me=useQuery({queryKey:["auth","me"],queryFn:()=>api<CurrentUser>("/auth/me"),retry:false});
  const bootstrap=useQuery({queryKey:["auth","bootstrap"],queryFn:()=>api<{required:boolean}>("/auth/bootstrap-status"),retry:false});
  const [email,setEmail]=useState(""); const [bootstrapToken,setBootstrapToken]=useState(""); const [name,setName]=useState(""); const [password,setPassword]=useState(""); const [error,setError]=useState(""); const [busy,setBusy]=useState(false);
  if(me.data) return <>{children(me.data,async()=>{await api("/auth/logout","POST");location.reload();})}</>;
  if(me.isPending||bootstrap.isPending)return <main className="auth-page"><p>Carregando…</p></main>;
  const first=bootstrap.data?.required===true;
  const submit=async(event:React.FormEvent)=>{event.preventDefault();setBusy(true);setError("");try{
    if(first){await api("/auth/bootstrap","POST",{bootstrap_token:bootstrapToken,email,display_name:name,password,role:"admin",active:true});}
    await api("/auth/login","POST",{email,password}); location.reload();
  }catch(e){setError(e instanceof ApiError?e.message:"Falha de autenticação");setBusy(false)}};
  return <main className="auth-page"><form className="dialog auth-card" onSubmit={submit}><header><h1>Orisema Data <span>v0.4</span></h1></header><h2>{first?"Criar administrador inicial":"Entrar"}</h2>{first&&<label>Token de configuração<input required type="password" autoComplete="off" value={bootstrapToken} onChange={e=>setBootstrapToken(e.target.value)}/></label>}{first&&<label>Nome<input required maxLength={128} value={name} onChange={e=>setName(e.target.value)}/></label>}<label>E-mail<input required type="email" autoComplete="username" value={email} onChange={e=>setEmail(e.target.value)}/></label><label>Senha<input required minLength={12} type="password" autoComplete={first?"new-password":"current-password"} value={password} onChange={e=>setPassword(e.target.value)}/></label>{error&&<p role="alert" className="error">{error}</p>}<button className="primary" disabled={busy}>{busy?"Aguarde…":first?"Criar e entrar":"Entrar"}</button>{first&&<small>O primeiro usuário recebe o papel administrador. Use uma senha exclusiva com pelo menos 12 caracteres.</small>}</form></main>;
}
