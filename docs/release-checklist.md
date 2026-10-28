# Release readiness checklist

Orisema Data is an experimental, single-operator application. **Do not publish a production-ready or multi-user release** until authentication, authorization, auditing and a security review are implemented.

## Before tagging a release

- [ ] GitHub Actions: Rust formatting, check, tests, Clippy, frontend checks and container build pass.
- [ ] Confirm the container image starts, serves `/api/v1/health` and serves the frontend.
- [ ] Validate a fresh Compose installation with generated credentials on a trusted local machine.
- [ ] Verify an upgrade from an existing RustDB installation with a tested PostgreSQL backup, unchanged `RUSTDB_ENCRYPTION_KEY` and existing volume mapping.
- [ ] Verify saved encrypted connections can still be opened after upgrade.
- [ ] Check `.env` and logs for secret exposure; ensure no credentials are committed.
- [ ] Verify documentation, license (AGPL-3.0-only), changelog and version number match the tag.
- [ ] Document the known limitations: PostgreSQL-only, local/single operator, no authentication/RBAC/audit.
- [ ] Publish release notes with a migration warning: `RUSTDB_*` variables, Compose service, image and volume names remain legacy-compatible.

## Scope of CI

CI proves the Rust/frontend checks and that the Dockerfile can build. It **does not** prove database upgrade safety, container startup, a successful backup restore, or production security. These require separate hands-on verification before release.

## Compatibility warning

Do not rename `rustdb-data` or regenerate the encryption key as part of a cosmetic rebrand. See [upgrade-safety.md](upgrade-safety.md).
