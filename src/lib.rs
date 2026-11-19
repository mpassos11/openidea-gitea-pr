use openidea_plugin::{Error, HttpRequest, Invocation, NotificationLevel, Plugin, register_plugin};
use openidea_plugin::host;
use serde::{Deserialize, Serialize};

#[derive(Default)]
struct GiteaPr;

#[derive(Deserialize, Serialize, Clone, Default)]
struct Config {
    base_url: String,
    owner: String,
    repo: String,
    default_branch: Option<String>,
}

#[derive(Serialize, Deserialize, Default)]
struct PrSummary {
    number: u64,
    title: String,
    state: String,
    user: Login,
    head: RefInfo,
    base: RefInfo,
    html_url: String,
    #[serde(default)]
    labels: Vec<Label>,
    #[serde(default)]
    requested_reviewers: Vec<Login>,
}

#[derive(Serialize, Deserialize, Default, Clone)]
struct Label { name: String }

#[derive(Serialize, Deserialize, Default)]
struct Login { login: String }

#[derive(Serialize, Deserialize, Default)]
struct RefInfo { label: String }

impl GiteaPr {
    fn load_config(&self) -> Result<Config, Error> {
        let raw = host::storage::get("config")?;
        let raw = raw.ok_or_else(|| Error::Failed("not configured; run 'Gitea PR: Configure Server'".into()))?;
        let cfg: Config = serde_json::from_str(&raw).map_err(|e| Error::Failed(e.to_string()))?;
        // Security: reject empty or traversal-like fields.
        if cfg.base_url.trim().is_empty() || cfg.owner.trim().is_empty() || cfg.repo.trim().is_empty() {
            return Err(Error::Failed("invalid config: empty field".into()));
        }
        if cfg.owner.contains("..") || cfg.owner.contains('/') || cfg.repo.contains("..") || cfg.repo.contains('/') {
            return Err(Error::Failed("invalid config: owner/repo must not contain path separators".into()));
        }
        if !cfg.base_url.starts_with("http://") && !cfg.base_url.starts_with("https://") {
            return Err(Error::Failed("invalid config: base_url must be http(s)".into()));
        }
        Ok(cfg)
    }

    fn api(&self, cfg: &Config, path: &str) -> String {
        format!("{}/repos/{}/{}/{}",
            cfg.base_url.trim_end_matches('/'), cfg.owner, cfg.repo, path.trim_start_matches('/'))
    }

    fn token(&self) -> Result<String, Error> {
        if !host::secret_is_set("gitea_token") {
            return Err(Error::Failed("token not set; run 'Gitea PR: Configure Server'".into()));
        }
        Ok("{{secret:gitea_token}}".to_string())
    }

    fn get(&self, url: &str) -> Result<String, Error> {
        let tok = self.token()?;
        let req = HttpRequest {
            method: "GET".into(),
            url: url.into(),
            headers: vec![("Authorization".into(), format!("token {}", tok))],
            body: None,
            timeout_ms: Some(30000),
        };
        let resp = host::fetch(&req)?;
        let resp_body = String::from_utf8(resp.body).map_err(|e| Error::Failed(e.to_string()))?;
        if resp.status < 200 || resp.status >= 300 {
            return Err(Error::Failed(format!("HTTP {} {}", resp.status, resp_body)));
        }
        Ok(resp_body)
    }

    fn post(&self, url: &str, body: &str) -> Result<String, Error> {
        let tok = self.token()?;
        let req = HttpRequest {
            method: "POST".into(),
            url: url.into(),
            headers: vec![
                ("Authorization".into(), format!("token {}", tok)),
                ("Content-Type".into(), "application/json".into()),
            ],
            body: Some(body.into()),
            timeout_ms: Some(30000),
        };
        let resp = host::fetch(&req)?;
        let resp_body = String::from_utf8(resp.body).map_err(|e| Error::Failed(e.to_string()))?;
        if resp.status < 200 || resp.status >= 300 {
            return Err(Error::Failed(format!("HTTP {} {}", resp.status, resp_body)));
        }
        Ok(resp_body)
    }
}

impl Plugin for GiteaPr {
    fn run_command(&mut self, id: &str, ctx: &Invocation, arg: Option<&str>) -> Result<Option<String>, Error> {
        match id {
            "gitea-pr.configure" => {
                // Settings UI takes precedence; fall back to JSON arg for backward compat.
                let v: serde_json::Value = if let Some(bu) = host::setting_string("baseUrl") {
                    serde_json::json!({
                        "base_url": bu,
                        "owner": host::setting_string("owner").unwrap_or_default(),
                        "repo": host::setting_string("repo").unwrap_or_default(),
                        "default_branch": host::setting_string("defaultBranch"),
                        "token": serde_json::from_str::<serde_json::Value>(arg.unwrap_or("{}")).ok()
                            .and_then(|x| x.get("token").and_then(|t| t.as_str()).map(String::from)),
                    })
                } else {
                    serde_json::from_str(arg.unwrap_or("{}")).unwrap_or_default()
                };
                let cfg: Config = serde_json::from_value(v.clone())
                    .map_err(|e| Error::Failed(e.to_string()))?;
                if cfg.base_url.trim().is_empty() || cfg.owner.trim().is_empty() || cfg.repo.trim().is_empty() {
                    return Err(Error::Failed("invalid config: empty field".into()));
                }
                if cfg.owner.contains("..") || cfg.owner.contains('/') || cfg.repo.contains("..") || cfg.repo.contains('/') {
                    return Err(Error::Failed("invalid config: owner/repo must not contain path separators".into()));
                }
                if !cfg.base_url.starts_with("http://") && !cfg.base_url.starts_with("https://") {
                    return Err(Error::Failed("invalid config: base_url must be http(s)".into()));
                }
                host::storage::set("config", &serde_json::to_string(&cfg).map_err(|e| Error::Failed(e.to_string()))?)?;
                if let Some(t) = v.get("token").and_then(|x| x.as_str()) {
                    host::secret_set("gitea_token", t)?;
                }
                let msg = format!("Configuration saved: {}/{}", cfg.owner, cfg.repo);
                host::notify(NotificationLevel::Info, "Gitea PR", &msg);
                let _ = host::output_append("Gitea PR", &msg);
                let _ = host::output_reveal("Gitea PR");
                Ok(None)
            }
            "gitea-pr.list" => {
                let cfg = self.load_config()?;
                let v: serde_json::Value = serde_json::from_str(arg.unwrap_or("{}")).unwrap_or_default();
                let page = v.get("page").and_then(|x| x.as_u64()).unwrap_or(1);
                let limit = v.get("limit").and_then(|x| x.as_u64()).unwrap_or(30).min(100);
                let state = v.get("state").and_then(|x| x.as_str()).unwrap_or("open");
                let url = self.api(&cfg, &format!("pulls?state={}&page={}&limit={}", state, page, limit));
                let body = self.get(&url)?;
                let prs: Vec<PrSummary> = serde_json::from_str(&body)
                    .map_err(|e| Error::Failed(e.to_string()))?;
                let msg = prs.iter()
                    .map(|p| {
                        let labels = p.labels.iter().map(|l| l.name.clone()).collect::<Vec<_>>().join(",");
                        let reviewers = p.requested_reviewers.iter().map(|r| r.login.clone()).collect::<Vec<_>>().join(",");
                        format!("#{} {} ({}) {} -> {} [{}] reviewers:{}", p.number, p.title, p.state, p.head.label, p.base.label, labels, reviewers)
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                host::notify(NotificationLevel::Info, "Gitea PRs", if msg.is_empty() { "No PRs." } else { &msg });
                let _ = host::output_append("Gitea PR", if msg.is_empty() { "No PRs." } else { &msg });
                let _ = host::output_reveal("Gitea PR");
                host::storage::set("prs", &body)?;
                host::storage::set("page", &page.to_string())?;
                host::storage::set("state", state)?;
                Ok(None)
            }
            "gitea-pr.create" => {
                let cfg = self.load_config()?;
                let payload: serde_json::Value = serde_json::from_str(arg.unwrap_or("{}"))
                    .map_err(|e| Error::Failed(e.to_string()))?;
                let title = payload.get("title").and_then(|v| v.as_str()).unwrap_or("New PR");
                let head = payload.get("head").and_then(|v| v.as_str()).unwrap_or("");
                let base = payload.get("base").and_then(|v| v.as_str())
                    .map(String::from).unwrap_or_else(|| cfg.default_branch.clone().unwrap_or_else(|| "main".into()));
                let body = serde_json::json!({ "title": title, "head": head, "base": base, "body": "" }).to_string();
                let url = self.api(&cfg, "pulls");
                let resp = self.post(&url, &body)?;
                host::notify(NotificationLevel::Info, "Gitea PR", &format!("Created PR: {}", resp));
                Ok(None)
            }
            "gitea-pr.merge" => {
                let cfg = self.load_config()?;
                let number: u64 = arg.unwrap_or("").parse().map_err(|_| Error::Failed("pass PR number as arg".into()))?;
                let url = self.api(&cfg, &format!("pulls/{}/merge", number));
                let _ = self.post(&url, r#"{"Do": "merge"}"#)?;
                host::notify(NotificationLevel::Info, "Gitea PR", &format!("Merged #{}", number));
                Ok(None)
            }
            "gitea-pr.comment" => {
                let cfg = self.load_config()?;
                let (num, text) = arg.unwrap_or("").split_once(':').unwrap_or(("0", ""));
                let number: u64 = num.parse().map_err(|_| Error::Failed("arg format: <number>:<comment>".into()))?;
                let url = self.api(&cfg, &format!("issues/{}/comments", number));
                let body = serde_json::json!({ "body": text }).to_string();
                let _ = self.post(&url, &body)?;
                host::notify(NotificationLevel::Info, "Gitea PR", "Comment posted.");
                Ok(None)
            }
            "gitea-pr.review" => {
                self.run_command("gitea-pr.list", ctx, None)?;
                host::notify(NotificationLevel::Info, "Gitea PR", "Open the Gitea Pull Requests panel to review.");
                Ok(None)
            }
            "gitea-pr.diff" => {
                let cfg = self.load_config()?;
                let number: u64 = arg.unwrap_or("").parse().map_err(|_| Error::Failed("pass PR number as arg".into()))?;
                let url = format!("{}/repos/{}/{}/pulls/{}.diff",
                    cfg.base_url.trim_end_matches('/'), cfg.owner, cfg.repo, number);
                let tok = self.token()?;
                let req = HttpRequest {
                    method: "GET".into(),
                    url: url.clone(),
                    headers: vec![("Authorization".into(), format!("token {}", tok))],
                    body: None,
                    timeout_ms: Some(30000),
                };
                let resp = host::fetch(&req)?;
                let diff = String::from_utf8(resp.body).map_err(|e| Error::Failed(e.to_string()))?;
                let cd = openidea_plugin::ContentDiff {
                    left: String::new(),
                    right: diff,
                    left_label: Some(format!("base (PR #{})", number)),
                    right_label: Some(format!("head (PR #{})", number)),
                    title: Some(format!("PR #{} diff", number)),
                };
                host::open_diff(&cd, Some(&format!("pr-{}.diff", number)))?;
                Ok(None)
            }
            _ => Err(Error::NotFound(format!("command {id}"))),
        }
    }
}

const VIEW: &str = "gitea-pr.panel";

impl openidea_plugin::views::Views for GiteaPr {
    fn render(&mut self, view: &str, _viewport: Option<openidea_plugin::views::Viewport>, _ctx: &Invocation) -> Result<openidea_plugin::views::View, Error> {
        if view != VIEW {
            return Err(Error::NotFound(view.into()));
        }
        // Web body is served from the package's web/index.html (declared in plugin.toml).
        Ok(openidea_plugin::views::View::new(openidea_plugin::views::Body::Message("".into())))
    }

    fn handle(&mut self, view: &str, event: openidea_plugin::views::ViewEvent, _ctx: &Invocation) -> Result<(), Error> {
        if view != VIEW {
            return Err(Error::NotFound(view.into()));
        }
        match event {
            openidea_plugin::views::ViewEvent::Message(msg) => {
                let v: serde_json::Value = serde_json::from_str(&msg).unwrap_or_default();
                let op = v.get("op").and_then(|x| x.as_str()).unwrap_or("");
                match op {
                    "refresh" | "list" => {
                        if let Some(st) = v.get("state").and_then(|x| x.as_str()) {
                            let arg = serde_json::json!({"state": st, "page": v.get("page").cloned().unwrap_or(serde_json::json!(1))}).to_string();
                            if let Err(e) = self.run_command("gitea-pr.list", _ctx, Some(&arg)) {
                                let reply = serde_json::json!({"op":"error","message": e.to_string()}).to_string();
                                let _ = host::post_view_message(VIEW, &reply);
                            }
                        }
                        let data = host::storage::get("prs").unwrap_or(None).unwrap_or_else(|| "[]".into());
                        let reply = serde_json::json!({"op":"prs","data": data}).to_string();
                        host::post_view_message(VIEW, &reply)?;
                    }
                    "diff" => {
                        if let Some(n) = v.get("number").and_then(|x| x.as_u64()) {
                            let _ = self.run_command("gitea-pr.diff", _ctx, Some(&n.to_string()));
                        }
                    }
                    "merge" => {
                        if let Some(n) = v.get("number").and_then(|x| x.as_u64()) {
                            let _ = self.run_command("gitea-pr.merge", _ctx, Some(&n.to_string()));
                        }
                    }
                    "comment" => {
                        let n = v.get("number").and_then(|x| x.as_u64()).unwrap_or(0);
                        let text = v.get("text").and_then(|x| x.as_str()).unwrap_or("");
                        let _ = self.run_command("gitea-pr.comment", _ctx, Some(&format!("{}:{}", n, text)));
                    }
                    _ => {}
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }
}

register_plugin!(GiteaPr, views);

#[cfg(test)]
mod tests {
    use super::*;
    use openidea_plugin::mock::MockHost;

    #[test]
    fn list_without_config_fails() {
        let _host = MockHost::install().with_capabilities(&["storage", "secrets", "project.read", "vcs.read"]);
        let mut p = GiteaPr::default();
        let res = p.run_command("gitea-pr.list", &_host.invocation(), None);
        assert!(res.is_err());
    }

    #[test]
    fn configure_saves_config() {
        let host = MockHost::install().with_capabilities(&["storage", "secrets"]);
        host.declare_secret("gitea_token");
        let mut p = GiteaPr::default();
        let arg = r#"{"base_url":"https://gitea.example.com/api/v1","owner":"org","repo":"repo","token":"abc"}"#;
        p.run_command("gitea-pr.configure", &host.invocation(), Some(arg)).unwrap();
        let raw = host.storage().get("config").cloned().unwrap();
        assert!(raw.contains("\"owner\":\"org\""));
    }

    #[test]
    fn configure_from_settings() {
        let host = MockHost::install().with_capabilities(&["storage", "secrets"]);
        host.declare_secret("gitea_token");
        host.set_setting("baseUrl", "\"https://gitea.example.com/api/v1\"");
        host.set_setting("owner", "\"org\"");
        host.set_setting("repo", "\"repo\"");
        host.set_setting("defaultBranch", "\"main\"");
        let mut p = GiteaPr::default();
        p.run_command("gitea-pr.configure", &host.invocation(), None).unwrap();
        let raw = host.storage().get("config").cloned().unwrap();
        assert!(raw.contains("\"owner\":\"org\""));
        assert!(raw.contains("\"base_url\":\"https://gitea.example.com/api/v1\""));
    }

    #[test]
    fn fuzz_configure_rejects_bad_input() {
        let host = MockHost::install().with_capabilities(&["storage", "secrets"]);
        let mut p = GiteaPr::default();
        // invalid JSON
        assert!(p.run_command("gitea-pr.configure", &host.invocation(), Some("{not json")).is_err());
        // path traversal in owner
        let arg = r#"{"base_url":"https://gitea.example.com/api/v1","owner":"../etc","repo":"repo"}"#;
        assert!(p.run_command("gitea-pr.configure", &host.invocation(), Some(arg)).is_err());
        // bad base_url
        let arg = r#"{"base_url":"ftp://x","owner":"org","repo":"repo"}"#;
        assert!(p.run_command("gitea-pr.configure", &host.invocation(), Some(arg)).is_err());
    }

    #[test]
    fn fuzz_merge_requires_number() {
        let host = MockHost::install().with_capabilities(&["storage", "secrets", "project.read", "vcs.read"]);
        let mut p = GiteaPr::default();
        // no config -> error
        assert!(p.run_command("gitea-pr.merge", &host.invocation(), Some("abc")).is_err());
    }

    #[test]
    fn fuzz_unknown_command() {
        let host = MockHost::install().with_capabilities(&["storage"]);
        let mut p = GiteaPr::default();
        assert!(p.run_command("gitea-pr.nope", &host.invocation(), None).is_err());
    }

    #[test]
    fn list_with_mock_fetch() {
        let host = MockHost::install().with_capabilities(&["storage", "secrets", "net:gitea.example.com"]);
        host.declare_secret("gitea_token");
        host.set_secret("gitea_token", "tok");
        let mut p = GiteaPr::default();
        let arg = r#"{"base_url":"https://gitea.example.com/api/v1","owner":"org","repo":"repo"}"#;
        p.run_command("gitea-pr.configure", &host.invocation(), Some(arg)).unwrap();
        host.on_fetch(|_r| Ok(openidea_plugin::HttpResponse { status: 200, headers: vec![], body: b"[{\"number\":1,\"title\":\"T\",\"state\":\"open\",\"user\":{\"login\":\"u\"},\"head\":{\"label\":\"h\"},\"base\":{\"label\":\"b\"},\"html_url\":\"u\"}]".to_vec() }));
        p.run_command("gitea-pr.list", &host.invocation(), Some(r#"{"state":"open"}"#)).unwrap();
        let stored = host.storage().get("prs").cloned().unwrap();
        assert!(stored.contains("\"number\":1"));
    }

    #[test]
    fn comment_requires_config() {
        let host = MockHost::install().with_capabilities(&["storage", "secrets"]);
        let mut p = GiteaPr::default();
        assert!(p.run_command("gitea-pr.comment", &host.invocation(), Some("1:hi")).is_err());
    }

    #[test]
    fn merge_with_mock() {
        let host = MockHost::install().with_capabilities(&["storage", "secrets", "net:gitea.example.com"]);
        host.declare_secret("gitea_token");
        host.set_secret("gitea_token", "tok");
        let mut p = GiteaPr::default();
        let arg = r#"{"base_url":"https://gitea.example.com/api/v1","owner":"org","repo":"repo"}"#;
        p.run_command("gitea-pr.configure", &host.invocation(), Some(arg)).unwrap();
        host.on_fetch(|_r| Ok(openidea_plugin::HttpResponse { status: 200, headers: vec![], body: b"{}".to_vec() }));
        p.run_command("gitea-pr.merge", &host.invocation(), Some("5")).unwrap();
    }
}
