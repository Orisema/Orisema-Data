# Publishing to GitLab

No remote project has been created automatically. Repository publication is a separate,
explicitly authorized action. The local repository uses `main` and logical phase commits.

After reviewing the code and ensuring no secrets are tracked, create a **blank public**
GitLab project named `rustdb` (do not initialize another README or license). Use:

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
