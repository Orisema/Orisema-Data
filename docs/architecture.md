# Architecture

Orisema Data v0.2 (formerly RustDB) is a single-operator PostgreSQL web SQL client, not a hosted multi-user service.

```
React / TanStack Query / Monaco
             | REST /api/v1
Axum handlers -> connection service -> DatabaseDriver
                      |                    |
              SQLx repository       PostgresDriver
                      |                    |
              Internal PostgreSQL    Target PostgreSQL
```

`orisema-data-core` owns transport-neutral operations, driver contracts and result models.
`orisema-data-server` owns HTTP, configuration, credential encryption, persistence and the
PostgreSQL implementation. No HTTP handler should construct a PostgreSQL driver.
The application composition root selects the driver. Future agent dispatch can implement
the same contract without changing the browser's response model. Agent transport and
cancellation are deliberately not implemented.

The internal database must be separate from databases queried by users. Only encrypted
passwords are persisted; public connection DTOs do not contain the encrypted field.
Migrations run automatically before the server accepts requests.

No authentication, RBAC or audit trail is implemented in v0.2. Listen on loopback and use only on a trusted
workstation. CORS is not authentication. Do not expose this service publicly.

## Query lifecycle

1. Validate one statement with the PostgreSQL dialect of `sqlparser` and an MVP allowlist.
2. Open a dedicated target connection with TLS policy and connection deadline.
3. Begin a transaction; set READ ONLY when configured. Server statement and idle-in-
   transaction timeouts complement the overall client execution deadline.
4. Prepare for column metadata (including zero-row results) and PostgreSQL's own syntax check.
5. Use a cursor plus bounded FETCH for SELECT. Stream DML results, retaining only the
   configured number of rows while consuming the command to determine affected rows.
6. Commit only after successful completion. A result-budget failure rolls back; a client
   timeout drops the dedicated connection rather than returning it to a shared pool.

The core contract uses positional JSON rows. Text protocol represents PostgreSQL types
without handwritten binary decoders. Nontrivial types stay as strings to avoid JavaScript
precision loss. The React grid does not branch on PostgreSQL type names.

Only the configuration repository uses a shared pool (five connections). All persisted
fields use bound parameters, and metadata schema/table filters are also bound. User SQL
is intentionally executed as code within the configured database permissions; it is
not an untrusted value that can be made safe merely by parameterization.

The HTTP boundary validates Host and browser Origin to reduce cross-site localhost and
DNS rebinding exposure, limits JSON bodies and concurrent handlers, and sanitizes
framework errors. Logs record method/status/duration without paths, SQL or credentials.
These controls do not restrict a local client that deliberately constructs requests.
