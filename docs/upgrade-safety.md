# Upgrade safety and compatibility

Orisema Data v0.2 continues RustDB. This is a **local, single-operator** application without authentication, RBAC or an audit trail. Never expose it to the Internet.

## Current compatibility contract

| Surface | Current value | Policy |
| --- | --- | --- |
| Rust package names | `orisema-data-core`, `orisema-data-server` | Renamed |
| Server binary | `orisema-data-server` | Renamed |
| Source directories | `crates/rustdb-core`, `crates/rustdb-server` | Retained |
| Environment variables | `RUSTDB_*` | Retained and required |
| Compose application service | `rustdb` | Retained |
| Compose image | `localhost/rustdb:0.2` | Retained |
| PostgreSQL volume | `rustdb-data` | **Do not rename or delete** |
| PostgreSQL defaults | `rustdb` user/database | Retained |

## Upgrade procedure

1. **Back up the internal PostgreSQL database** using your established PostgreSQL backup procedure, and verify the backup is restorable. Preserve the `RUSTDB_ENCRYPTION_KEY` separately in a secure location: existing encrypted connection passwords require the original key.
2. Record the current Compose project name, volume name, `.env` values and image tag before changing anything.
3. Keep the existing `.env`, Compose service names, database names and volume mapping. Do not regenerate the encryption key and do not run `docker compose down -v` or `podman compose down -v`.
4. Build and test the new image in a disposable environment **before** updating an existing installation. Run `docker compose config` or `podman compose config` to inspect the effective configuration without sharing its secrets.
5. Apply the update only after validating the backup and the expected volume mapping. Check the health endpoint and confirm existing saved connections still decrypt.
6. If validation fails, stop and restore using the tested backup and the original encryption key. **Do not assume that downgrading the image will reverse database migrations.**

## Deferred migrations

Renaming source directories is an internal refactor, but must also update Cargo workspace members and local dependency paths. Changing Compose service, image, volume or environment-variable names requires a separate migration plan with explicit compatibility tests. Do not conflate code rebranding with persisted-state migration.

See [rebranding.md](rebranding.md) and [architecture.md](architecture.md).
