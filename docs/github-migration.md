# GitHub migration plan

The intended destination is `https://github.com/Orisema/Orisema-Data`. The current
GitLab repository remains the historical source during the transition. No repository
has been created, transferred or published by this change.

## Preconditions

1. Merge and tag the Orisema Data rename on GitLab after its pipeline passes.
2. Confirm the `Orisema` organization owners, repository visibility, branch protection,
   vulnerability-reporting route and required GitHub Actions checks.
3. Create one empty GitHub repository named `Orisema-Data`, without generated files.
4. Verify that the AGPL-3.0-only `LICENSE`, all tags and the complete Git history are present.

## Authorized migration procedure

Run only after explicit authorization, from a clean clone whose `origin` remains GitLab:

```sh
git remote add github git@github.com:Orisema/Orisema-Data.git
git push github --all
git push github --tags
```

Compare branch and tag lists on both hosts, set `main` as the GitHub default branch,
enable the workflow in `.github/workflows/ci.yml`, and require its Rust and frontend
jobs. Keep the GitLab remote and project available as a historical reference during
the transition. Do not create a second unrelated history or squash-import the project.

After validation, update the primary clone URL and contribution links in a dedicated
documentation change. Whether GitLab becomes read-only, a mirror or an archived project:
A confirmar.

## Optional Orisema Infrastructure integration

Orisema Data remains independently deployable. A future integration should use an
explicitly versioned, authenticated API in which Infrastructure owns container
provisioning and lifecycle, while Orisema Data owns logical database connections,
metadata and queries. Trust, service identity, discovery, lifecycle callbacks and API
schemas: A confirmar. No Infrastructure dependency or remote execution path is added here.
