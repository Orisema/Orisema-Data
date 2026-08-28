# Publishing to GitLab

The public repository was created with explicit user authorization on 2026-08-28:
https://gitlab.com/CasMaster/rustdb (project ID 85863107).
The local `main` branch was pushed successfully and is the remote default branch.
The remote `origin` uses HTTPS and Git Credential Manager; no token is embedded in its URL.
The project description and all seven topics below were configured and verified through
the public GitLab API on 2026-08-28.

For another installation, review the code and ensure no secrets are tracked, then create
a **blank public** GitLab project named `rustdb` (without another README or license). Use:

Description: Open source web-based database management and SQL client written in Rust.

Topics: `rust`, `database`, `postgresql`, `sql`, `database-client`, `web`, `open-source`.

```sh
git status
git ls-files .env
git remote add origin git@gitlab.com:YOUR_NAMESPACE/rustdb.git
git push -u origin main
```

Replace `YOUR_NAMESPACE`, use your normal SSH authentication and set `main` as the
default branch in GitLab. Do not place a token in the remote URL or commit credentials.
Confirm the project visibility and confidential vulnerability-reporting route. The
pipeline checks/builds artifacts but does not publish container images. A suitable
GitLab runner is required; local checks do not prove a remote pipeline has run.
