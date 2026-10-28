# Changelog

## Unreleased

- Renamed the Rust crates to `orisema-data-core` and `orisema-data-server`, the Rust import crate to `orisema_data_core`, the web package to `orisema-data-web`, and the executable to `orisema-data-server`.
- Updated the visible product identity, container image/service, build configuration and technical documentation to Orisema Data.
- Preserved all `RUSTDB_*` environment variables, migrations, internal schema, encrypted credentials, session cookie and the `rustdb-data` persistent volume for compatibility.
- Added a GitHub Actions workflow and a history-preserving migration plan for the future `Orisema/Orisema-Data` repository; no publication or transfer is performed.

## 0.4.0 — 2026-09-09

- Added MySQL and MariaDB target drivers with TLS policy, metadata exploration, SQL execution, row/size limits and best-effort cancellation by dropping dedicated connections.
- Added a separate SQL Server driver using Tiberius with metadata exploration, SQL execution, affected-row reporting and the same authorization, audit and connection-policy pipeline.
- Added per-dialect explorer SQL and connection defaults in the React interface.
- Preserved PostgreSQL as the only internal database and preserved encrypted target credentials.
- Added real-container integration coverage for MySQL 8.4, MariaDB 11.8 and SQL Server 2022.


## 0.3.0 — 2026-09-08

- Added token-gated first-administrator bootstrap, Argon2id password hashing, login, logout, login-attempt limiting and revocable server-side sessions.
- Added user management with backend-enforced admin, editor and viewer roles; sensitive user changes revoke existing sessions.
- Added paginated security audit events for authentication, users, connections, policies and query execution/cancellation without SQL or credentials.
- Added per-connection role, write, row-limit, timeout and prohibited-command policies, enforced before target database access.
- Restricted query cancellation to the owner or an administrator.
- Added authenticated frontend flows for login, users, audit logs and connection policies.


## 0.2.0 — 2026-08-31

- Added persistent SQL tabs and connection-specific drafts in browser storage.
- Added searchable query history, favorites and named saved queries.
- Added UTF-8 CSV result export with correct field escaping.
- Added cancellable query executions backed by dedicated PostgreSQL connections.
- Added metadata-aware Monaco suggestions, light/dark themes and adjustable editor height.
- Hardened persisted workbench loading against malformed browser-storage entries.
- Extended CSV formula-injection protection to formulas preceded by whitespace.
- Documented the privacy implications of locally persisted SQL text.

## 0.1.0 — 2026-08-28

Initial experimental MVP:

- Rust workspace with shared driver contracts and future agent operation types.
- Axum REST API, PostgreSQL configuration storage and SQLx migrations.
- Encrypted credentials, connection management and connection tests.
- Lazy schema, table, view and column exploration.
- Monaco SQL editor, selection execution, Ctrl+Enter and paginated results.
- Read-only transactions, SQL allowlist, deadlines, result limits and sanitized errors.
- React frontend served by Vite in development or Axum in a container.
- GitLab CI, Docker/Podman development setup and security documentation.

No authentication, agent, additional database drivers or production-readiness claim.
