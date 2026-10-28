# Internal Rust crate directory rename plan

This plan is deliberately separate from the Rust package/binary rename already merged.

## Proposed directory moves

- `crates/rustdb-core/` → `crates/orisema-data-core/`
- `crates/rustdb-server/` → `crates/orisema-data-server/`

Move **all** tracked files, including tests and modules. Do not change their contents solely to rebrand historical identifiers.

## Required coordinated updates

1. In root `Cargo.toml`, set workspace members to `crates/orisema-data-core` and `crates/orisema-data-server`.
2. In server `Cargo.toml`, set the `orisema-data-core` dependency path to `../orisema-data-core`.
3. Search all scripts, CI files, Dockerfile, docs and build instructions for references to the **old directory paths** and update them. The Dockerfile currently copies the entire `crates/` directory and should not need path changes.
4. Keep `RUSTDB_*` environment variables, Compose service/image names, PostgreSQL user/database names, SQL migrations and the `rustdb-data` volume unchanged.
5. Do not change the workspace package names, executable name, Cargo.lock dependency names or HTTP routes.

## Validation gates

- `cargo fmt --all -- --check`
- `cargo check --locked --workspace`
- `cargo test --locked --workspace -- --include-ignored`
- `cargo clippy --locked --workspace --all-targets -- -D warnings`
- `npm ci && npm run lint && npm test && npm run build` in `frontend/`
- `docker build --tag orisema-data:ci .` and packaged-asset checks
- Inspect the PR diff to verify that all tracked files moved and no files disappeared.

## Release safety

Directory renaming is a source-tree change, not a database migration. Nevertheless, it must pass CI before merge. A full running Compose + PostgreSQL smoke test and an upgrade/restore rehearsal are still required before release. See [upgrade-safety.md](upgrade-safety.md).
