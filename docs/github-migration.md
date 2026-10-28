# GitHub migration

The repository was mirrored to `https://github.com/Orisema/Orisema-Data` on
2026-10-28. The GitLab repository remains the historical source during the transition.
All advertised branches and tags were compared after publication; three branches matched
exactly and GitLab advertised no tags at migration time.

## Completed procedure

1. Created the empty public repository `Orisema/Orisema-Data`.
2. Made a bare mirror clone from the public GitLab repository.
3. Pushed the Git object graph and refs to GitHub over authenticated SSH.
4. Removed GitLab-only merge-request and pipeline refs from GitHub.
5. Compared advertised branch/tag refs between both hosts; they matched exactly.

## Repeatable mirror procedure

For an explicitly authorized refresh during the transition:

```sh
git clone --mirror https://gitlab.com/CasMaster/rustdb.git
cd rustdb.git
git push --mirror git@github.com:Orisema/Orisema-Data.git
```

Compare branch and tag lists on both hosts and keep the GitLab project available as a
historical reference during the transition. Do not create a second unrelated history
or squash-import the project. GitLab-only internal refs should not remain on GitHub.

GitHub branch protection, required Actions checks, vulnerability reporting, and whether
GitLab becomes read-only, a mirror or an archived project: A confirmar.

## Optional Orisema Infrastructure integration

Orisema Data remains independently deployable. A future integration should use an
explicitly versioned, authenticated API in which Infrastructure owns container
provisioning and lifecycle, while Orisema Data owns logical database connections,
metadata and queries. Trust, service identity, discovery, lifecycle callbacks and API
schemas: A confirmar. No Infrastructure dependency or remote execution path is added here.
