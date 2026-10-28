# Orisema Data roadmap (formerly RustDB)

No promised dates. Checked items below reflect documented RustDB v0.2 capabilities, not independently rerun tests.

## Existing RustDB v0.2
- [x] PostgreSQL connection manager and tests.
- [x] Schema explorer, Monaco SQL editor and result grid.
- [x] Saved queries, tabs, history, CSV and cancellation.
- [x] Encrypted stored credentials, local Compose and CI.
- [ ] Authentication, users, RBAC and audit.

## Phase 0 — Rebranding (first)
- [x] Introduce Orisema Data project identity and migration plan in documentation.
- [ ] Inventory and rename internal identifiers with backward compatibility and tests.
- [ ] Update frontend branding and publishing metadata.
- [ ] Decide repository hosting and migration separately.

## Phase 1 — Security for shared use
- [ ] Authentication and session management.
- [ ] User roles, connection-scoped permissions and audit trail.
- [ ] Security tests and threat model; prohibit unauthenticated public exposure.

## Phase 2 — Database instance management
- [ ] Discover/register PostgreSQL instances and health status.
- [ ] Manage connection inventory without exposing credentials.
- [ ] Define safe, least-privilege instance administration.

## Phase 3 — Optional Orisema Infrastructure integration
- [ ] RFC for versioned API, authentication, error contracts and lifecycle ownership.
- [ ] Mock integration and failure-isolation tests.
- [ ] Optional PostgreSQL instance discovery/registration and connection diagnostics.
- [ ] Infrastructure manages container lifecycle; Data manages logical database administration.
- [ ] Neither project requires the other to run.

## Phase 4 — Expand
- [ ] MySQL/MariaDB drivers.
- [ ] Tested backup/restore and additional administration.
- [ ] Remote access only with authenticated, audited security design.
- [ ] Future drivers: SQL Server, Firebird, Oracle as separately prioritized.

**Out of scope:** Gera Base integration, automatic public exposure, mandatory coupling, or an unreviewed license change.
