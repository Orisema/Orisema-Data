# Decisões confirmadas

Registre aqui somente decisões claras sustentadas por código ou documentação versionada. Cada entrada deve apontar sua evidência.

## D-001 — Aplicação local e de operador único

- **Decisão:** o MVP é destinado a estação local confiável, sem exposição pública.
- **Evidência:** `README.md`, `SECURITY.md`, bind padrão em `crates/orisema-data-server/src/config.rs` e portas de loopback em `compose.yml`.

## D-002 — Contrato de driver desacoplado da API

- **Decisão:** handlers dependem do serviço e da trait `DatabaseDriver`; o composition root escolhe PostgreSQL.
- **Evidência:** `crates/orisema-data-core/src/driver.rs`, `crates/orisema-data-server/src/main.rs`, `crates/orisema-data-server/src/service.rs` e `docs/architecture.md`.

## D-003 — PostgreSQL interno separado do destino

- **Decisão:** configurações ficam em um PostgreSQL interno; consultas usam conexões dedicadas ao banco cadastrado.
- **Evidência:** `README.md`, `SECURITY.md`, `repository.rs`, `postgres.rs` e migrations.

## D-004 — Credenciais criptografadas e versão de ciphertext

- **Decisão:** senhas persistidas usam AES-256-GCM, nonce aleatório, UUID como AAD e formato `v1:`.
- **Evidência:** `crates/orisema-data-server/src/crypto.rs` e teste de roundtrip autenticado.

## D-005 — Política SQL restritiva, sem alegação de sandbox

- **Decisão:** aceitar uma única query ou DML limitado quando escrita estiver habilitada; privilégios PostgreSQL continuam sendo a fronteira principal.
- **Evidência:** `policy.rs`, `postgres.rs`, `SECURITY.md` e testes.

## D-006 — Estado diário do editor no navegador

- **Decisão:** abas, rascunhos, histórico, favoritos, consultas salvas, tema e altura do editor são persistidos em `localStorage`.
- **Evidência:** `frontend/src/workbench.ts`, `App.tsx`, testes do workbench e `CHANGELOG.md` 0.2.0.

## D-008 — Autenticação e autorização server-side

- **Decisão:** senhas usam Argon2id, sessões persistem apenas o hash do token e os papéis admin/editor/viewer e políticas por conexão são aplicados no backend. O primeiro administrador exige `RUSTDB_BOOTSTRAP_TOKEN`.
- **Evidência:** migration `0003_security.sql`, `auth.rs`, `connection_policy.rs`, `api.rs` e testes.

## D-009 — Auditoria sem conteúdo sensível

- **Decisão:** eventos registram ator, tipo, alvo, sucesso e metadados mínimos; SQL, senhas, tokens e credenciais de conexão não entram no log de auditoria.
- **Evidência:** `auth.rs` e handlers em `api.rs`.

## D-007 — Contexto de IA versionado no repositório

- **Decisão:** instruções e contexto compartilhado de agentes residem em `AGENTS.md` e `.ai/*.md`; essa camada é documental e não implementa o Agent remoto do roadmap.
- **Evidência:** solicitação que introduziu estes arquivos, `AGENTS.md` e `.ai/ARCHITECTURE.md`.

## D-010 — Registro local de drivers de destino na v0.4

- **Decisão:** `MultiDriver` seleciona PostgreSQL, MySQL/MariaDB ou SQL Server pelo tipo persistido. SQLx continua responsável pelo PostgreSQL interno e pelos alvos PostgreSQL/MySQL; Tiberius implementa SQL Server. MySQL e MariaDB compartilham o protocolo e o adaptador.
- **Evidência:** `multi.rs`, `mysql.rs`, `sqlserver.rs`, `main.rs`, migration `0004_multidriver.sql` e integrações reais.

## D-011 — Contrato comum e cancelamento por conexão dedicada

- **Decisão:** RBAC, auditoria e políticas permanecem acima dos drivers; todos retornam o mesmo resultado posicional. Cancelamento de MySQL, MariaDB e SQL Server aborta a task e derruba a conexão dedicada, sem alegação de cancelamento síncrono no servidor.
- **Evidência:** `service.rs`, `api.rs`, `driver.rs`, `docs/architecture.md` e testes dos drivers.

## D-012 — Rebranding compatível para Orisema Data

- **Decisão:** crates, pacote web, binário e identidade visível usam Orisema Data; variáveis `RUSTDB_*`, migrations, schema interno, cookie de sessão e volume `rustdb-data` permanecem estáveis para não quebrar instalações existentes.
- **Evidência:** manifests Cargo/npm, `Dockerfile`, `compose.yml`, `config.rs`, `auth.rs`, `migrations/` e documentação de migração.
