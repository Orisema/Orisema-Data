# Contributing to Orisema Data

Orisema Data is currently under active development. Keep changes small, explicit and tested.
Use Rust stable, Node.js 24 LTS, npm and PostgreSQL 17. On Windows, install the Rust
MSVC build tools and use Podman Desktop or Docker Desktop for PostgreSQL.

Follow the [README](README.md) development setup. Never commit `.env`, database dumps,
tokens, screenshots containing secrets or real connection credentials.

## Workflow

Create a branch from `main`: `feature/*`, `fix/*`, `docs/*` or `refactor/*`.
Prefer Conventional Commits: `feat:`, `fix:`, `docs:`, `refactor:`, `test:`, `chore:`.
Open a GitLab Merge Request with the purpose, approach, tests and known risks.
Include screenshots for UI changes using synthetic data only. Do not bypass failing checks.

```sh
cargo fmt --all -- --check
cargo check --locked --workspace
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
cd frontend
npm ci
npm run lint
npm test
npm run build
```

Integration tests need a **disposable** PostgreSQL database. Set
`RUSTDB_TEST_DATABASE_URL` and `RUSTDB_TEST_PASSWORD` (the target role's raw password;
empty for isolated CI trust authentication), then run from the repository root:

```sh
cargo test --locked --workspace -- --include-ignored
```

The integration test runs migrations and exercises real driver and HTTP handlers.
Without `--include-ignored`, it is explicitly skipped; do not describe that run as
a full integration check. Never point tests at production or a valuable internal database.

Frontend formatting: `npx prettier --write src vite.config.ts eslint.config.js` from
`frontend/`. Rust formatting: `cargo fmt --all`. Keep `Cargo.lock` and
`frontend/package-lock.json` committed. Future drivers implement the core contract;
do not import concrete drivers into API handlers.
