# Gitea Pull Requests (OpenIDEA plugin)

Controls pull requests on a Gitea server: list, create, review, comment, merge.

## Status

0.1 — builds and tests pass. Component at `dist/plugin.wasm`. 5 unit tests
(pass) against the SDK mock host (incl. fuzz for bad config/merge/unknown command).
Web view wired to `web/index.html`. Pagination (page/limit) supported on list.
Security validation on config (empty fields, path traversal, scheme).

Toolchain used: Rust 1.100 (GNU) + target `wasm32-wasip2`, built manually
(`cargo +stable-x86_64-pc-windows-gnu build --release --target wasm32-wasip2`).
The `openidea-plugin` CLI (`check`/`link`) was not installed (build too heavy);
install it with `cargo install --path crates/ide-host --bin openidea-plugin`
inside an openidea checkout to validate/install.

## Install / configure

1. `openidea-plugin link` (or install from disk).
2. Approve capabilities: `net:ask`, `secrets`, `storage`, `project.read`, `vcs.read`.
3. Set the token secret `gitea_token` via Settings > Plugins > Gitea Pull Requests.
4. Run `Gitea PR: Configure Server` with a JSON arg:
   `{"base_url":"https://gitea.example.com/api/v1","owner":"org","repo":"repo","default_branch":"main"}`

## Commands

- Gitea PR: List Pull Requests
- Gitea PR: Create Pull Request
- Gitea PR: Review Pull Request
- Gitea PR: Merge Pull Request
- Gitea PR: Comment on Pull Request
- Gitea PR: Configure Server

## Next steps

- Add a web view panel to browse/review PRs with diff rendering (use `host::open_diff`).
- Add settings UI instead of JSON arg.
- Add pagination, filters, labels, reviewers, and status checks.
- Add tests against `MockHost`.
