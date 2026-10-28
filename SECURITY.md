# Security policy

Orisema Data v0.3 adds authentication, users, backend-enforced RBAC, audit events and connection policies. It is still **not production-ready** and should remain behind HTTPS and network access controls.

## Reporting vulnerabilities

Do not publish credentials, exploit details or sensitive vulnerabilities in public
issues. After the GitLab project exists, use a **confidential GitLab issue**, verifying
the confidentiality setting before submitting. If this is unavailable, ask the
maintainer for a private security contact without posting technical details. No
security email address or response SLA has been established yet; do not assume one.
Provide affected version, impact and a minimal reproduction using synthetic data.

## Deployment boundary

- Bind to loopback. Never expose Orisema Data directly to the Internet. CORS and Host/Origin
  validation complement authentication but do not replace authorization. Any local process able to reach
  this service can operate its saved connections.
- Do not use privileged accounts or connect to production databases in this MVP.
  A user of Orisema Data can choose target hosts and ports: access to Orisema Data is access to
  the server's network. Use firewall/egress restrictions and a trusted workstation.
- Never expose a database to the Internet to make Orisema Data work. The Agent is not implemented.
- Use a dedicated target-database account with only the permissions required for read-only connections.
  Do not grant superuser, filesystem/program execution or powerful extension functions.
- TLS `verify_full` is the default for targets. TLS `disable` is an explicit local-only
  opt-out. Use trusted OS CA certificates; custom per-connection certificates are not supported.
- Keep the internal database separate from queried databases. Restrict access to both it
  and the encryption key. Back up them separately and securely. Losing the key loses
  credential access. Changing it does not rotate ciphertext; re-enter saved passwords.

## Implemented controls and limitations

Passwords use AES-256-GCM with a fresh 96-bit nonce and connection UUID as authenticated
associated data. API DTOs never return plaintext or ciphertext. Connection metadata
(host, database, username) is not encrypted. Secrets can exist in process memory and
container environment; this does not defend against a compromised host.

The SQL allowlist uses `sqlparser`, accepts one statement, rejects transaction/session
control, DDL and COPY, and only allows SELECT in read-only mode. PostgreSQL `READ ONLY`
transactions enforce ordinary write restrictions. These are **not a sandbox for SQL
functions**, extensions, external side effects, expensive queries or privileged accounts.
See [PostgreSQL's transaction semantics](https://www.postgresql.org/docs/current/sql-set-transaction.html).
Write-enabled connections allow INSERT/UPDATE/DELETE, including without WHERE. There
are no fine-grained policies or user confirmation for individual SQL writes yet.

Dedicated target connections, client deadlines, server statement timeouts, row limits,
an 8 MiB estimated result budget (including cell overhead and JSON escaping) and 16 concurrent HTTP handlers reduce accidental
resource consumption. The driver must receive an individual row before measuring it;
one huge target-database field can still allocate significant memory. Row limits do not cap
database CPU, scan cost or affected rows. Set target-database resource and role limits too.
Timeout/disconnect near COMMIT may leave a write outcome unknown; never blindly retry.

SQL text, credentials, connection URLs, driver error details and row values are omitted
from application logs. Avoid verbose third-party tracing in deployments. SQL errors
are intentionally sanitized; inspect the target database securely for detailed diagnosis.
Query tabs, execution history and saved queries are persisted unencrypted in the
browser's local storage. Do not put passwords, access tokens or sensitive production
data in SQL text on a shared workstation. Clearing site data removes this client-side
history. This convenience history is separate from the internal security audit log, which records event metadata but never SQL text or credentials.
