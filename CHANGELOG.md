# Changelog

## 0.2.0 — 2026-08-31

- Added persistent SQL tabs and connection-specific drafts in browser storage.
- Added searchable query history, favorites and named saved queries.
- Added UTF-8 CSV result export with correct field escaping.
- Added cancellable query executions backed by dedicated PostgreSQL connections.
- Added metadata-aware Monaco suggestions, light/dark themes and adjustable editor height.
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
