# Installing "Gitea Pull Requests"

Plugin id: `matheus.gitea-pr` — version `0.1.0` — signed by publisher `matheus`.

## Quick install (from disk)

1. Download `matheus.gitea-pr-0.1.0.oiplugin`.
2. Open OpenIDEA → **Settings › Plugins**.
3. Click **Install Plugin from Package File…** and select the `.oiplugin`.
4. Review the capabilities and accept:
   - Required: `net:ask`, `secrets`, `storage`, `project.read`, `vcs.read`
   - Optional: `vcs.write`, `editor.read`
5. (Optional) **Settings › Plugins › Gitea Pull Requests** → set secret `gitea_token`.
6. Run command **Gitea PR: Configure Server** with arg:
   `{"base_url":"https://your-gitea/api/v1","owner":"org","repo":"repo","default_branch":"main","token":"YOUR_TOKEN"}`

## Via marketplace (local)

Point **Settings › Plugins › Available** to:
`file:///C:/path/to/market/` (or an HTTP URL serving `index.json`).

## Commands

- `Gitea PR: List Pull Requests` — `{"page":1,"limit":30}` optional arg
- `Gitea PR: Create Pull Request` — `{"title":"...","head":"branch","base":"main"}`
- `Gitea PR: Review Pull Request`
- `Gitea PR: Merge Pull Request` — arg: PR number
- `Gitea PR: Comment on Pull Request` — arg: `123:comment text`
- `Gitea PR: Open PR Diff` — arg: PR number
- `Gitea PR: Configure Server`

## Publisher public key

`E9Gwsm6Zk6eQ3Bfw/YcPHBFV4WyVVopbE2Px5F/IMbo=`

Register this key (prefix `matheus.*`) in the community registry:
https://github.com/l0ng-ai/openidea-plugins
