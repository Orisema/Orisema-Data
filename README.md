# Orisema Data (formerly RustDB)

**Orisema Data — open-source SQL client, evolving from RustDB v0.2**

Orisema Data is an open source, Rust-powered web database management and SQL client. This project is the continuation of **RustDB**, preserving its Git history and existing implementation.

> **Status:** experimental, single-operator local application. **No authentication, RBAC or audit trail. Do not expose to the Internet or use production credentials.** See [SECURITY.md](SECURITY.md).

## Current capabilities (implemented in RustDB v0.2)

- PostgreSQL connection management and connection tests.
- AES-256-GCM encryption for saved connection passwords (external encryption key).
- Lazy schema/table/view/column explorer.
- Monaco SQL editor, saved queries, favorites, tabs, history and SQL completion.
- Query cancellation, CSV export, row limits, deadlines and read-only mode.
- Rust Axum API, SQLx/PostgreSQL backend and React/TypeScript frontend.
- Docker/Podman Compose for a trusted local workstation.

**Migration status:** GitHub is now the primary repository. Rust crate packages and server binary use `orisema-data-*`; source directories, environment variables, Compose services, image and volume identifiers retain legacy `rustdb-*` names for compatibility. See [the rebranding plan](docs/rebranding.md).

## Relationship with Orisema Infrastructure

- **Orisema Infrastructure** plans, provisions and operates containers and their lifecycle.
- **Orisema Data** manages database connections, metadata, query workflows and eventually logical administration.
- Both tools remain independently usable. Integration is **proposed, not implemented**.
- Initial integration target: optional, authenticated discovery/registration of PostgreSQL instances and connection status.
- Gera Base is explicitly outside the scope of this integration.

## Quick start (legacy names retained)

Requires Git, Node.js 24 for configuration generation, and Docker Compose or Podman with a Compose provider.

```sh
git clone https://github.com/Orisema/Orisema-Data.git
cd Orisema-Data
node scripts/init-env.mjs
podman compose up -d --build
# Or: docker compose up -d --build
```

Open `http://localhost:8080` **on the trusted local workstation only**. The internal PostgreSQL service stores encrypted connection configuration; connect to a separate target database. Never expose the web service or the target database publicly. Keep the encryption key backed up securely.

Existing `RUSTDB_*` configuration and Compose persistence remain unchanged; see `.env.example` for required settings.

## Migration strategy

See [docs/rebranding.md](docs/rebranding.md), [docs/roadmap.md](docs/roadmap.md), and the [safe upgrade checklist](docs/upgrade-safety.md). Crate and binary renaming is complete. Future changes to environment variables, Compose services, persisted database objects or volume identifiers require compatibility planning and separate tested changes.

## License

**AGPL-3.0-only** (unchanged). See [LICENSE](LICENSE). Orisema Infrastructure uses a separate Apache-2.0 license; communicating over an API does not require combining the codebases.

## Security and contributing

Read [SECURITY.md](SECURITY.md), [CONTRIBUTING.md](CONTRIBUTING.md), and [docs/architecture.md](docs/architecture.md).


## Repository and CI

- Primary repository: [Orisema/Orisema-Data](https://github.com/Orisema/Orisema-Data).
- Historical GitLab repository: `CasMaster/rustdb`.
- GitHub Actions runs Rust formatting, check, tests and Clippy, plus frontend lint, tests and build on pull requests and pushes to `main`.
