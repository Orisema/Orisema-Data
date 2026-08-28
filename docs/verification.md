# Verification record — 2026-08-28

## Latest status after authorized full WSL restart

- WSL and Podman were restarted successfully. Debian was started again, and the three
  previously running user containers (`templates-mariadb-staging`, `gerador-litestar-staging`,
  `gerador-bancos-staging`) returned healthy. No existing volumes were deleted.
- All **6 Rust tests**, including PostgreSQL integration, passed again in release mode.
- The native Windows server is running on `http://127.0.0.1:8080`, serving the production
  frontend. Browser connection testing, schema/table exploration and SELECT were verified
  again with four demo products, using a dedicated PostgreSQL SELECT-only role.
- `podman compose config --quiet` passed, and Compose's PostgreSQL service started healthy.
  The Windows API proxy was still using the old SSH port; restarting just that proxy
  restored normal `podman compose` operation. The temporary local SSH tunnel was stopped.
- The cached Rust 1.98 image contained an unreadable compiler library after the disk
  failure. A clean Rust 1.97 image was selected via the new `RUSTDB_BUILD_IMAGE` override.
  The actual **Linux release compilation passed** in that image.
- Disk space was exhausted again while committing the build layer, before the final
  runtime image was created. A second authorized WSL restart restored services. Builds
  are now paused pending more free disk space; the **final RustDB image and full application
  Compose deployment remain unverified**. Only the PostgreSQL Compose service was deployed.
- The Dockerfile now retains only the server binary in the build output layer and removes
  generated target/registry files before committing it. This disk-saving adjustment has
  not yet been exercised by a complete build. Reserve at least 15 GiB before retrying;
  this is a conservative recommendation, not a measured upper bound.
- Only RustDB-generated Cargo artifacts were cleaned. The Windows executable was preserved
  in `artifacts/rustdb-server.exe` and copied back to `target/release/rustdb-server.exe`.
  No global image-cache cleanup or filesystem repair was performed. Existing damaged
  image layers may still need attention; this is not a filesystem-integrity certification.

The remaining sections preserve the earlier verification history.

## Confirmed before the container infrastructure failure

- `cargo check --workspace`: passed.
- `cargo fmt --all`: applied; code compiled afterward.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `cargo test --workspace -- --include-ignored`: **6 passed, 0 failed, 0 ignored**.
  One core test and five server tests; the single integration test exercises many
  real PostgreSQL and HTTP operations rather than using a mocked driver.
- `npm run lint`: passed without warnings after formatting/helper cleanup.
- `npm test`: 1 passed (result-value formatting).
- `npm run build`: passed. Monaco's lazy chunk triggers a size advisory; it was not suppressed.
- `npm install` audit: 0 reported vulnerabilities. This is not a comprehensive security audit.
- Compose and GitLab pipeline files parse successfully as YAML; runtime/runner validation is separate.
- PostgreSQL 17 via Podman: startup and SQLx migrations passed; health returned `ok`.

The integration test verified connection creation/read/update/delete, omitted-password
preservation, encrypted storage and absence of secret fields in responses, connection
testing, metadata, row truncation, empty-result columns, bigint/decimal precision,
NULL/boolean values, duplicate column names, arrays/JSON/date text, statement timeout,
multi-statement rejection, read-only writes, modifying CTE rejection, DML commit/affected
counts, oversized-result rollback and bound metadata parameters.

Browser verification on the actual React/Vite application confirmed connection selection,
connection test, lazy schema/table/column expansion, views, table-to-editor SQL,
SELECT results with four synthetic products, and Ctrl+Enter executing only selected SQL.
The desktop layout and a 390-pixel responsive viewport were visually inspected.
Full automated UI regression, tests on physical mobile devices and clipboard-permission
behavior have not been completed.

## Container interruption

The multistage Podman build successfully ran frontend lint, tests and production build.
It then failed during the backend image stages with `server probably quit: unexpected EOF`.
Investigation found zero free bytes on C: and I/O/read-only-filesystem errors in the
Podman VM. Other user workloads share this VM, so it was not restarted without approval.
Only this workspace's Cargo-generated development artifacts were cleaned, recovering
approximately 2.3 GiB. No other project files, images or volumes were deleted.

The full container image, Compose startup and remote GitLab pipeline remain **unverified**.
The PostgreSQL integration and browser tests above passed before the infrastructure
failure; they are not evidence that the current local database is running afterward.
The browser subsequently displayed a sanitized internal error when the database became
unavailable. The interrupted UI creation attempt was not counted as a successful UI test.

After cleanup, `cargo test --locked --release --workspace -j 2` passed **5 tests**
with the PostgreSQL integration test explicitly ignored because the VM was unavailable.
`cargo build --locked --release -p rustdb-server -j 2` also passed and generated the
Windows executable at `target/release/rustdb-server.exe`. These builds used no debug
symbols to reduce disk consumption; no checks were disabled to hide failures.

Final review checks also passed: `cargo clippy --locked --release --workspace
--all-targets -- -D warnings`, `cargo fmt --all -- --check`, `git diff --check`,
frontend lint, **2 frontend tests** and production frontend build. The added cache test
covers metadata invalidation after editing a connection; workspace state now resets
on a connection update. This final edit was not re-tested against the unavailable DB
in the browser. Startup with a deliberately invalid encryption key was rejected with
exit code 1 and a sanitized log message.

## Security review scope

Manual implementation review covered secret DTO separation, authenticated encryption,
bound persistence/metadata queries, SQL allowlist and transaction boundaries, limits,
logging, TLS defaults, Host/Origin rejection, local-only publication and Docker context
exclusions. This is not a penetration test or a claim of production readiness.
Residual risks and unimplemented safeguards are documented in SECURITY.md.

## Reproduction after infrastructure recovery

### Authorized restart attempt

The user authorized a Podman restart after the initial delivery. The normal stop
command failed with a WSL root-user lookup I/O error, so only the Podman distribution
and its Podman network-helper distribution were terminated. Startup then failed with
`Wsl/Service/CreateInstance/E_UNEXPECTED` and `E_FAIL` before the guest could start.
Both Podman distributions remain stopped; Debian was left running and untouched.
No distribution, volume or disk image was deleted or recreated. Approximately 6.4 GiB
of host free space was available at the start of this attempt. Container verification
is still blocked; a broader WSL restart requires authorization because it also stops Debian.

1. Ensure adequate host/VM disk space and restore Podman without losing existing volumes.
2. Run the full PostgreSQL integration command from CONTRIBUTING.md.
3. Run `podman build -t localhost/rustdb:0.1 .` and `podman compose up -d --build`.
4. Verify health and the full browser flow at localhost:8080.
5. Publish only after an explicit GitLab authorization and review the actual remote CI result.
