# Authentication and RBAC architecture

Orisema Data v0.4 already authenticates users with Argon2id password hashes and
revocable server-side sessions. It enforces admin, editor and viewer roles, connection
policies and audit events in the backend. This proposal extends that boundary; it does
not add authentication code as part of the crate rename.

## Proposed direction

- Keep authorization in application services before credentials are decrypted or a
  target connection is opened. HTTP handlers and frontend checks are not trust boundaries.
- Model permissions as explicit actions on scoped resources, initially users,
  connections, policies, metadata and query execution. Map the existing three roles to
  permission sets so installations retain their current behavior.
- Add an identity-provider adapter for OIDC while retaining a controlled local-admin
  recovery path. Store provider subject identifiers, never upstream access tokens unless
  a documented feature requires them.
- Bind sessions to server-side records with rotation, expiry and revocation; preserve
  secure, HttpOnly, SameSite cookies and deploy behind HTTPS plus network controls.
- Record authorization decisions and administrative mutations without SQL text,
  passwords, tokens or decrypted connection material.
- Apply least privilege twice: Orisema Data permissions constrain product actions, and
  target database accounts constrain what the database itself can execute.

## Open decisions

OIDC provider support, MFA, recovery workflow, permission granularity, tenant model,
session lifetime and audit retention: A confirmar. These require a separate threat model,
migrations, compatibility plan and tests before implementation.
