# Gitea Pull Requests (OpenIDEA plugin)

Controls pull requests on a Gitea server from inside OpenIDEA: list, create,
review, comment, merge, and view diffs.

## Status

0.1.3 — builds, tests (8 unit tests) and `openidea-plugin check` pass.
Component at `dist/plugin.wasm`. Web view wired to `web/index.html` with
filters, labels, reviewers, and per-PR actions (Diff/Merge/Comment).
Pagination, state filter, labels and requested reviewers supported on list.
Diff rendering uses `host::open_diff`. Security validation on config (empty
fields, path traversal, scheme).

Toolchain: Rust (stable) + target `wasm32-wasip2`, built with
`openidea-plugin build` (or `cargo build --target wasm32-wasip2 --release`).

## Install / configure

1. `openidea-plugin link` (or install from disk / marketplace).
2. Approve capabilities: `net:ask`, `secrets`, `storage`, `project.read`, `vcs.read`.
3. Set the token secret `gitea_token` via Settings > Plugins > Gitea Pull Requests.
4. Configure via the settings UI (base URL, owner, repo, default branch) or run
   `Gitea PR: Configure Server` with a JSON arg:
   `{"base_url":"https://gitea.example.com/api/v1","owner":"org","repo":"repo","default_branch":"main"}`

## Commands

- Gitea PR: List Pull Requests (supports `{"page","limit","state"}`)
- Gitea PR: Create Pull Request
- Gitea PR: Review Pull Request
- Gitea PR: Merge Pull Request
- Gitea PR: Comment on Pull Request (`<number>:<text>`)
- Gitea PR: Open PR Diff
- Gitea PR: Configure Server

## Web view

The "Gitea Pull Requests" panel (right sidebar) lists open/closed/all PRs with
labels, reviewers, and actions: Diff (opens the IDE diff viewer), Merge,
Comment.

## Tests

`cargo test` — 8 tests (config validation, fuzz, list with mocked host fetch,
merge, etc.).
