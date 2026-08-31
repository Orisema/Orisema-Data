# REST API v1

Base path: `/api/v1`. JSON requests need `Content-Type: application/json`.
Host and browser Origin must match the configured local addresses or `RUSTDB_CORS_ORIGIN`.
This is not an authentication mechanism.

| Method | Path | Response |
|---|---|---|
| GET | `/health` | `{"status":"ok"}` |
| GET | `/connections` | Array of public connections |
| POST | `/connections` | Created public connection (201) |
| GET | `/connections/{id}` | Public connection |
| PUT | `/connections/{id}` | Updated public connection |
| DELETE | `/connections/{id}` | No content (204) |
| POST | `/connections/{id}/test` | `{"status":"ok"}` |
| GET | `/connections/{id}/schemas` | Array of schema names |
| GET | `/connections/{id}/schemas/{schema}/tables` | Array of `{name,kind}` (table/view) |
| GET | `/connections/{id}/schemas/{schema}/tables/{table}/columns` | Array of `{name,type,nullable}` |
| POST | `/connections/{id}/query` | Query result |
| DELETE | `/queries/{query_id}` | Cancel an active query (`204`, or `404` if no longer active) |

URL-encode schema and table path segments. IDs are UUIDs. Metadata lists cap at
10,000 entries and are loaded on demand. Materialized views are not included yet.

## Connection input

Required fields: `name`, `database_type` (`postgresql`), `host`, `port`,
`database_name`, `username`. `password` is required on creation (an empty password
is allowed for explicitly configured trust authentication); omit it or send null
on PUT to preserve the existing password. Send an empty string to replace it with
an empty password. PUT replaces the connection configuration; defaults are
`read_only: true` and `ssl_mode: "verify_full"`. The only TLS alternative is `disable`.
Unknown fields are rejected. Responses contain `has_password`, never passwords.

## Query

```json
{"sql":"SELECT * FROM produtos LIMIT 100", "max_rows":100, "query_id":"02a9d26d-2095-448e-b1d1-9a475c3b3c22"}
```

`max_rows` is optional, defaults to the server cap, and cannot exceed it.

```json
{
  "columns":[{"name":"codigo","type":"int4"},{"name":"descricao","type":"text"}],
  "rows":[[1,"Produto A"],[2,"Produto B"]],
  "row_count":2,
  "affected_rows":0,
  "execution_time_ms":18,
  "truncated":false
}
```

`row_count` counts returned rows, not total rows in the database. Empty results
retain column metadata. `affected_rows` is the full write command count, independent
of the result cap. `truncated` indicates additional rows not returned.

NULL, booleans and 16/32-bit integers are JSON primitives. Other types, including
64-bit integers, decimal, floats, timestamps, JSON, arrays, bytea and extension types,
use PostgreSQL text representations to preserve precision and avoid driver-specific
frontend decoders. Duplicate column names retain positional values.

Exactly one SELECT/query or (when read_only=false) INSERT, UPDATE or DELETE is allowed.
SELECT uses a server cursor, which excludes modifying CTEs, SELECT INTO and some
PostgreSQL-specific constructs. Unsupported parser syntax fails closed. Session
state and transactions do not persist between requests.

Clients that need cancellation include a unique `query_id` UUID in the query request.
Cancellation aborts the dedicated target connection so it cannot be returned to a pool.
The `query_id` is optional for compatibility, but a server-generated ID cannot be used
by the client cancellation endpoint.

Errors use `{"error":{"code":"...","message":"..."}}`. Validation is 400,
origin rejection 403, missing connection 404, cancellation 409, SQL/policy rejection 422, target
connection failure 502 and deadline expiry 504. No raw database error is returned.
