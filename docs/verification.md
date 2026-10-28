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

## v0.2 daily-workflow verification — 2026-08-31

- `cargo fmt --all -- --check`, `cargo check --locked --workspace`, the default
  workspace tests and Clippy with warnings denied passed.
- The ignored real-PostgreSQL lifecycle test passed through the Podman VM address.
  It now submits `SELECT pg_sleep(10)`, cancels it by UUID, receives HTTP 409 from
  the original request and verifies through `pg_stat_activity` that no matching
  backend query remains active.
- Frontend lint, four Vitest tests and the Vite production build passed. New tests
  cover history retention and spreadsheet-safe CSV quoting/formula neutralization.
- Browser verification on the compiled application confirmed persisted connection
  and SQL draft state after reload, successful query history, a named saved query,
  CSV availability, the cancellation control and response, the rename form, light
  theme rendering and restoration of the safe default draft. The previously noted
  Monaco lazy-chunk size advisory remains.
- The test PostgreSQL container was restarted because all Podman containers were
  stopped when this session resumed. No unrelated container was started or changed.

The final container image was rebuilt successfully as `localhost/rustdb:0.2` with
the pinned `rust:1.92-slim-bookworm` build image. Compose started PostgreSQL and
RustDB, and the health endpoint returned `{"status":"ok"}` both inside the
RustDB container and from the Podman machine. On this Windows host the forwarded
`127.0.0.1:8080` port still refused connections, so browser verification through
the host port remains environment-limited rather than an application failure.

## Reproduction after infrastructure recovery

### Infrastructure recovery record

The user authorized Podman and full WSL restarts after the initial delivery. A later
pre-flight found approximately 25.6 GiB free. The current Podman machine is running;
no volume, disk image, or unrelated service was deleted. A corrupt moving
`rust:slim-bookworm` layer prevented compilation (`libLLVM.so` could not be read),
so the build now pins the working `rust:1.92-slim-bookworm` base.

1. Ensure adequate host/VM disk space and restore Podman without losing existing volumes.
2. Run the full PostgreSQL integration command from CONTRIBUTING.md.
3. Run `podman build -t localhost/rustdb:0.2 .` and `podman compose up -d --build`.
4. Verify health from the Podman machine; if Windows port forwarding is healthy, also
   run the full browser flow at localhost:8080.
5. Review the actual remote CI result before publishing further changes.

## v0.2 follow-up verification — 2026-09-08

- `cargo fmt --all -- --check`, `cargo check --locked --workspace`,
  `cargo test --locked --workspace` and Clippy for all workspace targets with warnings
  denied passed under the Dockerfile's Rust 1.92 toolchain in an ephemeral container.
- Default Rust tests passed: 5 executed successfully and the real-PostgreSQL lifecycle
  test remained ignored because no disposable database or test variables were supplied.
- Frontend lint, 7 Vitest tests and the production build passed under an available Node
  22 container. The project documents Node 24, so Node 24 validation remains to be repeated.
- New tests cover complete workbench restoration, rejection of malformed persisted entries,
  editor-height clamping and CSV formula prefixes including leading whitespace.
- The known Monaco lazy-chunk size advisory remains. Manual browser verification was not
  available because no application/database runtime was started for this follow-up.

## v0.3 security verification — 2026-09-08

- Rust 1.92 container: formatting check, locked workspace check, unit tests and Clippy with warnings denied passed. Login limiting, audit pagination and session revocation were included in the final pass.
- PostgreSQL 17 disposable integration passed against the final migration. It covered token-gated bootstrap, Argon2id login/session, login throttling, allowed and denied RBAC paths, encrypted target credentials, connection-policy visibility and row cap, audit visibility/events, SQL policy, timeout and cancellation.
- Node.js 24 container: lint passed, 7 tests passed and production build passed. Vite retained the known Monaco chunk-size advisory.
- Manual browser verification was unavailable. The existing public v0.2 deployment was intentionally not replaced because a controlled bootstrap token and immediate first-administrator creation are required before exposing v0.3 through Caddy.

## v0.4 multi-driver verification — 2026-09-09

- MySQL 8.4 and MariaDB 11.8 disposable containers passed the same real-driver test: connection, schema/table/column discovery, typed SELECT, row truncation, read-only rejection, authorized UPDATE and affected-row count.
- SQL Server 2022 Developer started successfully in the environment and passed connection, schema/table/column discovery, streamed SELECT, truncation, read-only rejection, authorized UPDATE and affected-row count.
- All test credentials were generated ephemerally or used local trust in an isolated test network; values were not persisted or logged.
- Cancellation for the three new targets is implemented by task abort and dedicated-connection drop and is therefore documented as best-effort at the server.
