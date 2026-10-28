# Architecture

Orisema Data v0.4 is an authenticated multi-database web SQL client. Rust/Axum/SQLx and React/Vite remain unchanged; PostgreSQL remains the only internal configuration database.

```text
React / TanStack Query / Monaco
             | REST /api/v1
Axum handlers -> Service -> MultiDriver -> PostgresDriver (SQLx)
       |          |                    -> MySqlDriver (SQLx; MySQL/MariaDB)
       |          |                    -> SqlServerDriver (Tiberius)
       |          +-> active-query ownership / timeout / cancellation
       +-> authentication / RBAC / audit / connection policies
                         |
                  Internal PostgreSQL (SQLx repository)
```

`orisema-data-core` owns the transport-neutral `DatabaseDriver`, connection configuration and positional JSON result contract. `orisema-data-server` owns HTTP, authentication, policy enforcement, audit, encrypted credentials, persistence and concrete target drivers. Handlers never construct drivers. `MultiDriver` selects a driver from the persisted `database_type`; credentials still use AES-256-GCM and are decrypted only when building the target configuration.

The internal PostgreSQL must remain separate from every target database. Migration `0004_multidriver.sql` only widens the connection type check; it does not change ciphertext or security tables.

## Query lifecycle

1. Authenticate, authorize the role and verify the per-connection policy before target access.
2. Parse exactly one statement against supported SQL dialects and apply the SELECT/DML allowlist.
3. Open a dedicated target connection with the configured TLS and connection deadline.
4. Apply the effective write, row and timeout limits. PostgreSQL and MySQL/MariaDB use transactions; SQL Server uses the restrictive parser/policy and ApplicationIntent for read-only connections.
5. Stream rows into the common 8 MiB/row-limited result contract. PostgreSQL uses a bounded cursor, MySQL/MariaDB use SQLx prepared results, and SQL Server uses TDS metadata/row streams.
6. Commit supported writes only after successful processing. Database errors are sanitized at the API boundary.

The service stores an abort handle keyed by client query UUID. Cancellation aborts the task and drops its dedicated socket. PostgreSQL also applies server statement timeouts. For MySQL, MariaDB and SQL Server, socket-drop cancellation is best-effort: the server can take time to observe disconnect, so least privilege and server-side resource controls remain necessary.

## Dialect and result differences

- PostgreSQL: double quotes and `LIMIT`; complex values remain their PostgreSQL text representation.
- MySQL/MariaDB: backticks and `LIMIT`; exact decimal/bigint and temporal values are strings, binary values are Base64.
- SQL Server: brackets and `TOP`; bigint, decimal, float and temporal values are strings, binary values are Base64. DML `OUTPUT` result sets are not exposed by the current write path.
- Zero-row column metadata is available from PostgreSQL/MySQL prepared statements and SQL Server TDS metadata.
- The current metadata trait exposes schemas, tables/views and columns/types. Keys and indexes require a future contract extension.

The SQL allowlist is defensive, not a sandbox. Target accounts must have minimal privileges. Parser support differs by dialect and unsupported statements fail closed. Saved queries, browser history, CSV export, RBAC, audit and policies operate above the driver and apply unchanged to all targets.

## HTTP and browser boundary

Host and Origin checks, body/concurrency limits, secure session cookies and sanitized request logging remain unchanged. Logs do not include paths, SQL or credentials. The frontend selects driver-specific ports and generates quoted explorer queries, while Monaco highlighting remains generic SQL with metadata-based suggestions.

No Orisema Data Agent, remote transport or support-session path exists in v0.4.
