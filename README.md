# Orisema Data

**Orisema Data v0.4 — authenticated multi-database Web SQL Client**

Open source web-based database management and SQL client written in Rust.

**Orisema Data is currently under active development.** This release adds MySQL, MariaDB and SQL Server target drivers while preserving authentication and authorization, but is not yet production-ready. Keep it behind HTTPS and network access controls, and do not use privileged production accounts. Read [SECURITY.md](SECURITY.md).

> Screenshot placeholder: the interface has a connection selector, lazy explorer,
> SQL editor and results grid. A synthetic-data screenshot can be added after review.

## Features

- Authentication with Argon2id password hashes, revocable server-side sessions and secure cookies.
- Users and backend-enforced admin, editor and viewer roles.
- Security audit trail and per-connection access, write, row, timeout and SQL-command policies.
- PostgreSQL, MySQL, MariaDB and SQL Server connections: create, update, delete and test.
- AES-256-GCM credential encryption with an environment-provided key.
- Lazy schemas, tables, views and columns.
- Monaco SQL editor with selection execution and Ctrl+Enter / Cmd+Enter.
- Persistent query tabs, searchable history, favorites and named saved queries.
- Metadata-aware SQL completion, themes and adjustable editor height.
- Dynamic results with types, NULL, duration, row counts, horizontal scroll and copying.
- Spreadsheet-safe UTF-8 CSV export and cancellation of active queries on all supported target drivers.
- Client-side pages of 100 rows keep DOM size bounded; server result cap is configurable.
- Read-only mode, single-command SQL policy, deadlines and sanitized errors.
- PostgreSQL internal configuration storage with automatic migrations.

## Architecture and layout

```text
crates/orisema-data-core/      shared operations, driver trait and result contracts
crates/orisema-data-server/    Axum API, service, repository, crypto, target drivers
frontend/               React, TypeScript, Vite, TanStack Query, Monaco
migrations/             embedded SQLx migrations
scripts/init-env.mjs    local random-secret configuration generator
docs/                   architecture, API, security notes, roadmap, publishing
Dockerfile              web + Rust build stages, unprivileged runtime
compose.yml             Orisema Data + internal PostgreSQL
.gitlab-ci.yml          check → test → clippy → frontend → build
```

Browser → REST API → connection service → `DatabaseDriver` → PostgreSQL, MySQL/MariaDB or SQL Server.
The API depends on a trait, not a concrete target driver. Core operation types
reserve future agent messages. Query cancellation is implemented locally through the
REST API; no remote Agent implementation is included. See
[architecture](docs/architecture.md) and [API](docs/api.md).

## Quick start — Docker or Podman

Prerequisites: Git, Node.js 24 (for configuration generation), and Docker Compose
or Podman with a working Compose provider. The container compiles Rust and the frontend.

```sh
git clone https://gitlab.com/CasMaster/rustdb.git
cd rustdb
node scripts/init-env.mjs
podman compose up -d --build
# Alternatively:
docker compose up -d --build
```

Open **http://localhost:8080**. Migrations run automatically before serving requests.
Reserve at least 15 GiB of free host disk space before an initial container build;
actual usage depends on existing image layers and the container engine. About 6 GiB
was insufficient on the validation machine when committing build layers. Do not
retry a failed build on a nearly full disk.
The generator refuses to overwrite an existing `.env`. You may instead copy
`.env.example` and supply your own values. Back up the key securely; do not regenerate
it for an existing configuration database.

The Compose PostgreSQL stores Orisema Data configuration; register a **different target
database** in the UI. From inside a container, `localhost` means that container.
Use the target's container DNS name on a shared network, or `host.containers.internal`
on Podman / `host.docker.internal` on supported Docker Desktop installations.
Keep the target database private. TLS verification is enabled by default; only disable
TLS intentionally for an isolated development database without TLS.

Use `podman compose logs orisema-data` to diagnose startup without printing `.env`. Compose
requires URL-safe PostgreSQL passwords in its constructed URL; the generator produces
safe random hex passwords. `compose down` preserves the named database volume; adding
`-v` destroys stored connections. Do not do that unless you intend to erase them.

## Development

Install Rust stable (MSVC toolchain + build tools on Windows), Node.js 24/npm and
PostgreSQL 17 or Podman/Docker. From the repository root:

```sh
node scripts/init-env.mjs
podman compose up -d postgres
cargo run -p orisema-data-server
```

In a second terminal:

```sh
cd frontend
npm ci
npm run dev
```

Open **http://localhost:5173**. Vite proxies `/api` to `127.0.0.1:8080`; the default
CORS origin matches `localhost:5173`. If using `127.0.0.1:5173`, set that exact
`RUSTDB_CORS_ORIGIN` in `.env` and restart the server. If Podman/WSL loopback forwarding
does not work, repair the port forwarding or configure a reachable private VM address
and a password-protected development port. Do not expose trust authentication externally.

For a single-process local run, `npm run build` in `frontend`, then `cargo run --release`
from the root and open port 8080. Axum serves `frontend/dist`. To apply migrations
separately, install sqlx-cli and run `sqlx migrate run --database-url "$RUSTDB_DATABASE_URL"`
with the variable loaded into your shell. The server normally handles this itself.

## Configuration

Backend loads `.env` from its working directory; existing process environment wins.
Never version secrets. Container environment is supplied by Compose.

| Variable | Default / purpose |
|---|---|
| `RUSTDB_DATABASE_URL` | Required internal PostgreSQL URL; percent-encode credentials |
| `RUSTDB_ENCRYPTION_KEY` | Required Base64 of exactly 32 random bytes |
| `RUSTDB_BOOTSTRAP_TOKEN` | Required secret used only to authorize creation of the first administrator |
| `RUSTDB_BIND_ADDRESS` | `127.0.0.1`; container uses `0.0.0.0` with loopback publication |
| `RUSTDB_PORT` | `8080` |
| `RUSTDB_CORS_ORIGIN` | `http://localhost:5173`, one exact origin, no wildcard |
| `RUSTDB_QUERY_TIMEOUT_SECONDS` | `30`, range 1–300 |
| `RUSTDB_CONNECT_TIMEOUT_SECONDS` | `5`, range 1–60 |
| `RUSTDB_MAX_ROWS` | `1000`, range 1–10000; request may choose a lower cap |
| `RUSTDB_STATIC_DIR` | `frontend/dist` |
| `RUSTDB_BUILD_IMAGE` | Optional container compiler image override; defaults to `rust:slim-bookworm` |
| `RUST_LOG` | `info`; avoid verbose dependency tracing |
| `POSTGRES_USER`, `POSTGRES_PASSWORD`, `POSTGRES_DB` | Compose internal database settings |

## Tests and verification

```sh
cargo fmt --all -- --check
cargo check --locked --workspace
cargo test --locked --workspace
cargo clippy --locked --workspace --all-targets -- -D warnings
cd frontend
npm ci
npm run lint
npm test
npm run build
```

Real PostgreSQL integration: set `RUSTDB_TEST_DATABASE_URL` and
`RUSTDB_TEST_PASSWORD` for a disposable database, then run
`cargo test --workspace -- --include-ignored` from the root. See
[CONTRIBUTING.md](CONTRIBUTING.md) and [verification record](docs/verification.md).
The default unit-test command explicitly skips the database-dependent test; GitLab CI
includes it using its isolated PostgreSQL service.

## Known limitations

No password recovery, MFA, identity-provider integration, distributed login-rate limiting, Agent, remote database access, support sessions, Oracle/Firebird drivers or Excel export. One SQL
statement per execution; DDL, COPY, transaction commands,
modifying CTE cursor queries and some dialect syntax are unsupported. Write-enabled
connections allow DML without WHERE. Read-only is not protection against all functions
or privileged-account side effects. Limits do not guarantee a cheap database query.
Results preserve nontrivial types as text, including decimals and bigint. Metadata
lists cap at 10,000 entries. Monaco produces a large lazy-loaded bundle; build emits
a size advisory. Tabs, SQL history and saved queries are stored unencrypted in the
current browser profile; avoid entering secrets in SQL text on shared workstations.

## Roadmap

[Full roadmap](docs/roadmap.md): v0.2 daily-workflow features are implemented;
v0.3 authentication/RBAC/audit/policies and v0.4 PostgreSQL/MySQL/MariaDB/SQL Server drivers are implemented; v0.5 Agent and remote
support sessions. Future work includes Oracle, Firebird, autocomplete, EXPLAIN and teams.

## Contributing and security

See [CONTRIBUTING.md](CONTRIBUTING.md), [SECURITY.md](SECURITY.md), the
[GitLab publishing record](docs/publishing.md), the [GitHub migration plan](docs/github-migration.md)
and the [authentication/RBAC proposal](docs/auth-rbac.md). The intended future repository
is `Orisema/Orisema-Data`; this change does not publish or transfer it.

## License

Orisema Data is licensed under the GNU Affero General Public License v3.0 (AGPL-3.0-only).
The unmodified official text is in [LICENSE](LICENSE).

### Driver support in v0.4

| Target | Metadata | SQL/results | Write count | Cancellation | Notes |
|---|---|---|---|---|---|
| PostgreSQL | schemas, tables, views, columns/types | yes | yes, including returned rows | connection drop plus PostgreSQL statement timeout | double-quoted identifiers, `LIMIT` |
| MySQL 8.4 | databases, tables, views, columns/types | yes | yes | best-effort connection drop and timeout | backtick identifiers, `LIMIT` |
| MariaDB 11.8 | databases, tables, views, columns/types | yes | yes | best-effort connection drop and timeout | shares the MySQL protocol driver |
| SQL Server 2022 | schemas, tables, views, columns/types | yes | yes for DML | best-effort connection drop and timeout | bracket identifiers, `TOP`; DML result sets from `OUTPUT` are not exposed |

Keys and indexes are not yet represented by the current `DatabaseDriver` metadata contract.
Saved queries, history, CSV export, RBAC, audit and connection policies are driver-neutral
and therefore apply to every target. `verify_full` validates the target certificate and
hostname; `disable` is only for isolated trusted development networks. The internal
Orisema Data database remains PostgreSQL.

Real driver tests use disposable instances and opt-in environment variables:
`RUSTDB_MYSQL_TEST_HOST`, `RUSTDB_MYSQL_TEST_PORT`, `RUSTDB_MYSQL_TEST_DATABASE`,
`RUSTDB_MYSQL_TEST_USERNAME`, `RUSTDB_MYSQL_TEST_PASSWORD`, `RUSTDB_MYSQL_TEST_TYPE`,
and the corresponding `RUSTDB_SQLSERVER_TEST_*` names. Never point them at valuable data.
