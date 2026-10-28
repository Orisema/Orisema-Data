# RustDB → Orisema Data: compatibility-first migration

The repository has moved to GitHub. Runtime identifiers are not yet fully renamed; this document tracks the remaining compatibility-first work.

## Current baseline

- Primary GitHub repository: `Orisema/Orisema-Data`; default branch `main`. Historical GitLab repository: `CasMaster/rustdb`.
- Rust workspace: `rustdb-core`, `rustdb-server`; frontend package `rustdb-web`.
- Current version: `0.2.0`; license: `AGPL-3.0-only`.
- Compose services, image, volume, environment variables, SQL migrations and configuration database currently use RustDB identifiers.
- PostgreSQL client/editor features exist; instance provisioning and Orisema integration do not.
- No authentication, RBAC or audit; do not deploy as public or multi-user service.

## Safe stages

1. **Documentation and identity (this MR):** introduce the Orisema Data name while explicitly retaining existing RustDB runtime identifiers and operational instructions.
2. **Compatibility contract:** inventory every public/operational identifier; choose aliases/deprecation strategy and test migration of an existing installation with saved encrypted connections.
3. **Code rename:** rename packages, binary, frontend metadata and visible branding only with build/test coverage; avoid breaking existing paths or state.
4. **Runtime transition:** introduce new environment variable names with fallback, document precedence; migrate Compose names/images/volumes only with tested, explicit data-preservation instructions. Never silently create a new empty configuration database in place of existing data.
5. **Repository location:** decide separately whether to keep GitLab or migrate to Orisema organization; preserve history and maintain redirect/notice. Do not move or rename the remote without approval.
6. **Security gate:** authentication, RBAC and audit before shared/remote management.
7. **Optional integration:** RFC and mock-service tests for Orisema Infrastructure ↔ Orisema Data. Infrastructure owns containers; Data owns logical database administration.

## Acceptance checklist for future code changes

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo check --locked --workspace`
- [ ] `cargo test --locked --workspace`
- [ ] `cargo clippy --locked --workspace --all-targets -- -D warnings`
- [ ] `npm ci && npm run lint && npm test && npm run build` in `frontend`
- [ ] Existing Compose installation can upgrade without losing connections or encryption key.
- [ ] Public routes and configuration names are backward compatible or have a documented migration.
- [ ] No credentials in logs, source or artifacts.
- [ ] Existing GitLab CI passes.
