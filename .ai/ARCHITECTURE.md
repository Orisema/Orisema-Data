# Arquitetura

## Visão atual

Orisema Data v0.4 mantém React/Vite/Monaco, Axum e SQLx. O PostgreSQL interno continua isolado e é o único banco de configuração. `MultiDriver` seleciona `PostgresDriver` (SQLx), `MySqlDriver` (SQLx, compartilhado por MySQL e MariaDB) ou `SqlServerDriver` (Tiberius) pelo `database_type` persistido.

Autenticação, RBAC, auditoria e políticas são aplicados no backend antes de abrir a conexão de destino. Credenciais continuam cifradas com AES-256-GCM. Cada consulta usa conexão dedicada, timeout, orçamento de 8 MiB, limite de linhas e abort handle do proprietário.

PostgreSQL usa cursor e timeouts de sessão; MySQL/MariaDB usam statements preparados e transação, incluindo `SET TRANSACTION READ ONLY`; SQL Server usa stream TDS e ApplicationIntent read-only. O cancelamento dos três novos alvos ocorre por aborto da task e queda da conexão, sendo best-effort no servidor.

A allowlist aceita uma única query ou DML autorizado e tenta os dialetos PostgreSQL, MySQL, SQL Server e genérico. Não é sandbox; permissões mínimas no destino continuam obrigatórias.

O contrato atual expõe schemas, tabelas/views e colunas/tipos. Chaves e índices não cabem no contrato atual. Histórico, consultas salvas, CSV, autocomplete por metadados, auditoria e políticas permanecem acima dos drivers.

Não existe Agent, acesso remoto ou sessão temporária na v0.4.
