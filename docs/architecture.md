# Architecture

RustDB v0.1 is a single-operator PostgreSQL web SQL client, not a hosted multi-user service.

```
React / TanStack Query / Monaco
             | REST /api/v1
Axum handlers -> connection service -> DatabaseDriver
                      |                    |
              SQLx repository       PostgresDriver
                      |                    |
              Internal PostgreSQL    Target PostgreSQL
```

`rustdb-core` owns transport-neutral operations, driver contracts and result models.
`rustdb-server` owns HTTP, configuration, credential encryption, persistence and the
PostgreSQL implementation. No HTTP handler should construct a PostgreSQL driver.
The application composition root selects the driver. Future agent dispatch can implement
the same contract without changing the browser's response model. Agent transport and
cancellation are deliberately not implemented.

The internal database must be separate from databases queried by users. Only encrypted
passwords are persisted; public connection DTOs do not contain the encrypted field.
Migrations run automatically before the server accepts requests.

No authentication is implemented in v0.1. Listen on loopback and use only on a trusted
workstation. CORS is not authentication. Do not expose this service publicly.
