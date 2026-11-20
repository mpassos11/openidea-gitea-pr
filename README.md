# Gitea Pull Requests (OpenIDEA plugin)

Controls pull requests on a Gitea server from inside OpenIDEA: list, create,
review, comment, merge, and view diffs.

## Status

0.2.1 — builds, tests (9 unit tests) and `openidea-plugin check` pass.
Component at `dist/plugin.wasm`. Web view wired to `web/index.html` with
filters, labels, reviewers, status, auto-refresh, and per-PR actions
(Diff/Merge/Close/Reopen/Edit/Review/Reviewers/Details).
Pagination, state/author/label filters, merge methods (merge/squash/rebase).

## Features

- List PRs (open/closed/all) with pagination, author and label filters.
- Create PR from the current branch (auto-detects head branch and last commit
  subject as title).
- Merge with selectable method (merge / squash / rebase).
- Close / reopen PR.
- Edit title and body.
- Request reviewers.
- Submit reviews: Approve / Request changes / Comment.
- View PR diff in the IDE diff viewer.
- Comment on PR.
- Auto-refresh (30s) toggle.
- Detail view (body, metadata).

## Install / configure

1. `openidea-plugin link` (or install from disk / marketplace).
2. Approve capabilities: `net:ask`, `secrets`, `storage`, `project.read`, `vcs.read`.
3. Set the token secret `gitea_token` via Settings > Plugins > Gitea Pull Requests.
4. Configure via settings UI (base URL, owner, repo, default branch) or run
   `Gitea PR: Configure Server` with a JSON arg:
   `{"base_url":"https://gitea.example.com/api/v1","owner":"org","repo":"repo","default_branch":"main"}`

Note: the first fetch triggers a host permission dialog (net:ask). Run
"Gitea PR: List Pull Requests" once interactively to approve the host.

## Commands

- Gitea PR: List Pull Requests (supports `{"page","limit","state"}`)
- Gitea PR: Create Pull Request
- Gitea PR: Review Pull Request
- Gitea PR: Merge Pull Request (`<number>` or `<number>:<method>`)
- Gitea PR: Comment on Pull Request (`<number>:<text>`)
- Gitea PR: Open PR Diff
- Gitea PR: Close Pull Request
- Gitea PR: Reopen Pull Request
- Gitea PR: Edit Pull Request
- Gitea PR: Configure Server

## Web view

The "Gitea Pull Requests" panel (right sidebar) lists PRs with labels,
reviewers, status, and actions: Diff, Merge (with method selector),
Close/Reopen, Edit, Review (approve/request changes/comment),
Request reviewers, Details.

## Tests

`cargo test` — 9 tests (config validation, fuzz, settings, list with mocked
host fetch, merge).