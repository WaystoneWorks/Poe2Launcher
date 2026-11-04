//! Where the launcher reads releases and listings from: GitHub (an owner) or a static mirror
//! (a folder or web server with the same layout). One definition, used by the server and hook
//! downloads, the news, the mod index, the server list and the launcher's self-update.
//!
//! ```toml
//! [release_source]            # in settings.json as "release_source": {...}
//! kind = "github"             # "github" | "static"
//! api = "https://api.github.com"
//! owner = "OWNER"
//! base = ""                   # kind = "static": "https://host/path" or "file:///dir"
//! ```
//!
//! `github` asks `{api}/repos/{owner}/{repo}/releases`; `static` reads `{base}/{repo}/releases.json`,
//! the same JSON list. `cargo xtask release-mirror <dir>` writes such a folder.

use std::path::PathBuf;

use anyhow::{Result, bail, ensure};
use serde::{Deserialize, Serialize};

/// The owner of the project's repositories until the final organisation exists. Also read by
/// `scripts/set-owner.sh`, which rewrites this one literal.
#[macro_export]
macro_rules! default_owner {
    () => {
        "WaystoneWorks"
    };
}

pub const DEFAULT_OWNER: &str = default_owner!();
pub const GITHUB_API: &str = "https://api.github.com";
const GITHUB_RAW: &str = "https://raw.githubusercontent.com";
const GITHUB_WEB: &str = "https://github.com";

/// Repository names under the owner.
pub const HUB_REPO: &str = "Poe2ModdingProject";
pub const LAUNCHER_REPO: &str = "Poe2Launcher";
pub const SERVER_REPO: &str = "Poe2Server";
pub const HOOK_REPO: &str = "Poe2ClientHook";
pub const MODS_REPO: &str = "Poe2Mods";
pub const SERVER_LIST_REPO: &str = "Poe2ServerList";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum SourceKind {
    #[default]
    Github,
    Static,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct ReleaseSource {
    pub kind: SourceKind,
    /// `github`: the API address (a parameter for tests and GitHub Enterprise).
    pub api: String,
    /// `github`: the account or organisation that owns the repositories.
    pub owner: String,
    /// `static`: `https://host/path`, `http://host/path` or `file:///dir`.
    pub base: String,
}

impl Default for ReleaseSource {
    fn default() -> Self {
        Self {
            kind: SourceKind::Github,
            api: GITHUB_API.into(),
            owner: DEFAULT_OWNER.into(),
            base: String::new(),
        }
    }
}

/// A bare address is a GitHub-shaped API (mock servers in tests).
impl From<&str> for ReleaseSource {
    fn from(api: &str) -> Self {
        Self {
            api: api.into(),
            ..Self::default()
        }
    }
}

impl From<&String> for ReleaseSource {
    fn from(api: &String) -> Self {
        api.as_str().into()
    }
}

impl From<String> for ReleaseSource {
    fn from(api: String) -> Self {
        api.as_str().into()
    }
}

fn valid_owner(o: &str) -> bool {
    !o.is_empty()
        && o.len() <= 100
        && o.chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
}

/// An `http(s)` or `file` address without credentials, query or fragment.
fn check_address(what: &str, url: &str, allow_file: bool) -> Result<()> {
    let u = reqwest::Url::parse(url)
        .map_err(|_| anyhow::anyhow!("the {what} address {url:?} is not a valid URL"))?;
    match u.scheme() {
        "https" | "http" => ensure!(u.host_str().is_some(), "the {what} address has no host"),
        "file" if allow_file => {}
        _ => bail!(
            "the {what} address must start with https://, http://{}",
            if allow_file { " or file:///" } else { "" }
        ),
    }
    ensure!(
        u.username().is_empty() && u.password().is_none(),
        "the {what} address must not contain a user name or password"
    );
    ensure!(
        u.query().is_none() && u.fragment().is_none(),
        "the {what} address must not have a query or fragment"
    );
    Ok(())
}

impl ReleaseSource {
    /// Trims spaces and trailing slashes; an empty field becomes the default for `github`.
    pub fn normalized(mut self) -> Self {
        let d = Self::default();
        self.api = self.api.trim().trim_end_matches('/').to_owned();
        self.owner = self.owner.trim().to_owned();
        self.base = self.base.trim().trim_end_matches('/').to_owned();
        if self.api.is_empty() {
            self.api = d.api;
        }
        if self.owner.is_empty() {
            self.owner = d.owner;
        }
        self
    }

    pub fn validate(&self) -> Result<()> {
        match self.kind {
            SourceKind::Github => {
                ensure!(
                    valid_owner(&self.owner),
                    "the GitHub owner may only contain letters, digits, - _ and ."
                );
                check_address("GitHub API", &self.api, false)
            }
            SourceKind::Static => {
                ensure!(
                    !self.base.is_empty(),
                    "a static release source needs its address"
                );
                check_address("release mirror", &self.base, true)
            }
        }
    }

    pub fn is_default(&self) -> bool {
        *self == Self::default()
    }

    pub fn is_github(&self) -> bool {
        self.kind == SourceKind::Github
    }

    /// Where the list of releases of repository `name` is read.
    pub fn releases_url(&self, name: &str, per_page: u32) -> String {
        match self.kind {
            SourceKind::Github => format!(
                "{}/repos/{}/{name}/releases?per_page={per_page}",
                self.api.trim_end_matches('/'),
                self.owner
            ),
            SourceKind::Static => format!("{}/{name}/releases.json", self.base),
        }
    }

    /// `owner/name` on GitHub; the folder name on a mirror (what a release records as its repo).
    pub fn repo_path(&self, name: &str) -> String {
        match self.kind {
            SourceKind::Github => format!("{}/{name}", self.owner),
            SourceKind::Static => name.to_owned(),
        }
    }

    /// The repository's page, for people.
    pub fn repo_url(&self, name: &str) -> String {
        match self.kind {
            SourceKind::Github => format!("{GITHUB_WEB}/{}/{name}", self.owner),
            SourceKind::Static => format!("{}/{name}", self.base),
        }
    }

    /// Short text for messages: "github.com/owner/name" or "the mirror at <base>/name".
    pub fn describe(&self, name: &str) -> String {
        match self.kind {
            SourceKind::Github => format!("github.com/{}/{name}", self.owner),
            SourceKind::Static => format!("the release mirror ({}/{name})", self.base),
        }
    }

    /// "GitHub" or "the release mirror".
    pub fn host_label(&self) -> &'static str {
        match self.kind {
            SourceKind::Github => "GitHub",
            SourceKind::Static => "the release mirror",
        }
    }

    pub fn index_url(&self) -> String {
        match self.kind {
            SourceKind::Github => {
                format!("{GITHUB_RAW}/{}/{MODS_REPO}/main/index.json", self.owner)
            }
            SourceKind::Static => format!("{}/{MODS_REPO}/index.json", self.base),
        }
    }

    pub fn server_list_url(&self) -> String {
        match self.kind {
            SourceKind::Github => format!(
                "{GITHUB_RAW}/{}/{SERVER_LIST_REPO}/main/servers.json",
                self.owner
            ),
            SourceKind::Static => format!("{}/{SERVER_LIST_REPO}/servers.json", self.base),
        }
    }

    pub fn news_url(&self) -> String {
        match self.kind {
            SourceKind::Github => format!("{GITHUB_RAW}/{}/{HUB_REPO}/main/news.json", self.owner),
            SourceKind::Static => format!("{}/{HUB_REPO}/news.json", self.base),
        }
    }

    pub fn hosting_guide_url(&self) -> String {
        match self.kind {
            SourceKind::Github => format!(
                "{GITHUB_WEB}/{}/{SERVER_REPO}/blob/main/docs/hosting.md",
                self.owner
            ),
            SourceKind::Static => format!("{}/{SERVER_REPO}/hosting.md", self.base),
        }
    }

    /// The signed `latest.json` of the launcher's own releases.
    pub fn updater_endpoint(&self) -> String {
        match self.kind {
            SourceKind::Github => format!(
                "{GITHUB_WEB}/{}/{LAUNCHER_REPO}/releases/latest/download/latest.json",
                self.owner
            ),
            SourceKind::Static => format!("{}/{LAUNCHER_REPO}/latest.json", self.base),
        }
    }

    /// Where the server list is maintained, in words for the Servers page.
    pub fn server_list_label(&self) -> String {
        match self.kind {
            SourceKind::Github => format!("{}/{SERVER_LIST_REPO} on GitHub", self.owner),
            SourceKind::Static => format!("{}/{SERVER_LIST_REPO}", self.base),
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Reading from a web server or from a folder (`file:///`)

/// The path of a `file://` address; `None` for anything else.
pub fn file_path(url: &str) -> Option<PathBuf> {
    let u = reqwest::Url::parse(url).ok()?;
    (u.scheme() == "file")
        .then(|| u.to_file_path().ok())
        .flatten()
}

#[derive(Debug)]
pub enum FetchError {
    /// The server could not be reached (no URL in the text).
    Unreachable(String),
    /// An HTTP status; a missing file is 404.
    Status(u16),
    TooLarge(usize),
    Other(String),
}

impl std::fmt::Display for FetchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unreachable(e) => write!(f, "cannot reach it: {e}"),
            Self::Status(c) => write!(f, "HTTP {c}"),
            Self::TooLarge(max) => write!(f, "the answer is larger than {} KB", max / 1024),
            Self::Other(e) => f.write_str(e),
        }
    }
}

impl std::error::Error for FetchError {}

/// The text behind `url` (at most `max` bytes), over HTTP or from a `file://` address.
pub async fn fetch_text(
    c: &reqwest::Client,
    url: &str,
    accept: &str,
    max: usize,
) -> Result<String, FetchError> {
    if let Some(path) = file_path(url) {
        return match tokio::fs::read(&path).await {
            Ok(b) if b.len() > max => Err(FetchError::TooLarge(max)),
            Ok(b) => Ok(String::from_utf8_lossy(&b).into_owned()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Err(FetchError::Status(404)),
            Err(e) => Err(FetchError::Other(format!("{}: {e}", path.display()))),
        };
    }
    let res = c
        .get(url)
        .header("Accept", accept)
        .send()
        .await
        .map_err(|e| FetchError::Unreachable(e.without_url().to_string()))?;
    let status = res.status();
    if !status.is_success() {
        return Err(FetchError::Status(status.as_u16()));
    }
    if res.content_length().is_some_and(|l| l as usize > max) {
        return Err(FetchError::TooLarge(max));
    }
    let bytes = res
        .bytes()
        .await
        .map_err(|e| FetchError::Unreachable(e.without_url().to_string()))?;
    if bytes.len() > max {
        return Err(FetchError::TooLarge(max));
    }
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mirror(base: &str) -> ReleaseSource {
        ReleaseSource {
            kind: SourceKind::Static,
            base: base.into(),
            ..Default::default()
        }
    }

    #[test]
    fn github_urls_follow_the_owner() {
        let s = ReleaseSource {
            owner: "Acme".into(),
            ..Default::default()
        };
        assert_eq!(
            s.releases_url("Poe2Server", 30),
            "https://api.github.com/repos/Acme/Poe2Server/releases?per_page=30"
        );
        assert_eq!(s.repo_path("Poe2Server"), "Acme/Poe2Server");
        assert_eq!(
            s.index_url(),
            "https://raw.githubusercontent.com/Acme/Poe2Mods/main/index.json"
        );
        assert_eq!(
            s.server_list_url(),
            "https://raw.githubusercontent.com/Acme/Poe2ServerList/main/servers.json"
        );
        assert_eq!(
            s.news_url(),
            "https://raw.githubusercontent.com/Acme/Poe2ModdingProject/main/news.json"
        );
        assert_eq!(
            s.updater_endpoint(),
            "https://github.com/Acme/Poe2Launcher/releases/latest/download/latest.json"
        );
        assert_eq!(
            s.hosting_guide_url(),
            "https://github.com/Acme/Poe2Server/blob/main/docs/hosting.md"
        );
    }

    #[test]
    fn static_urls_follow_the_base() {
        let s = mirror("http://127.0.0.1:8000/m");
        assert_eq!(
            s.releases_url("Poe2Server", 30),
            "http://127.0.0.1:8000/m/Poe2Server/releases.json"
        );
        assert_eq!(s.repo_path("Poe2Server"), "Poe2Server");
        assert_eq!(s.index_url(), "http://127.0.0.1:8000/m/Poe2Mods/index.json");
        assert_eq!(
            s.server_list_url(),
            "http://127.0.0.1:8000/m/Poe2ServerList/servers.json"
        );
        assert_eq!(
            s.updater_endpoint(),
            "http://127.0.0.1:8000/m/Poe2Launcher/latest.json"
        );
    }

    #[test]
    fn a_bare_address_is_a_github_shaped_api() {
        let s = ReleaseSource::from("http://127.0.0.1:5/");
        assert_eq!(s.api, "http://127.0.0.1:5/");
        assert!(
            s.releases_url("X", 1)
                .starts_with("http://127.0.0.1:5/repos/")
        );
    }

    #[test]
    fn validation_and_normalising() {
        assert!(ReleaseSource::default().validate().is_ok());
        let bad_owner = ReleaseSource {
            owner: "a/b".into(),
            ..Default::default()
        };
        assert!(bad_owner.validate().is_err());
        assert!(mirror("").validate().is_err());
        assert!(mirror("ftp://x/y").validate().is_err());
        assert!(mirror("https://u:p@x/y").validate().is_err());
        assert!(mirror("https://x/y?a=1").validate().is_err());
        assert!(mirror("file:///tmp/m").validate().is_ok());
        assert!(mirror("http://10.0.0.2:8000/mirror").validate().is_ok());
        let n = ReleaseSource {
            owner: " ".into(),
            api: "".into(),
            base: " https://x/y/ ".into(),
            kind: SourceKind::Static,
        }
        .normalized();
        assert_eq!(n.owner, DEFAULT_OWNER);
        assert_eq!(n.api, GITHUB_API);
        assert_eq!(n.base, "https://x/y");
        // Only the github API address must not be a file address.
        let file_api = ReleaseSource {
            api: "file:///x".into(),
            ..Default::default()
        };
        assert!(file_api.validate().is_err());
    }

    #[test]
    fn json_round_trip_and_defaults() {
        let d: ReleaseSource = serde_json::from_str("{}").unwrap();
        assert!(d.is_default());
        let s: ReleaseSource =
            serde_json::from_str(r#"{"kind":"static","base":"file:///tmp/m"}"#).unwrap();
        assert_eq!(s.kind, SourceKind::Static);
        assert_eq!(s.owner, DEFAULT_OWNER);
        assert!(!s.is_default());
    }

    #[tokio::test]
    async fn fetch_text_reads_files() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("a.json"), "[1]").unwrap();
        let url = reqwest::Url::from_file_path(dir.path().join("a.json"))
            .unwrap()
            .to_string();
        let c = reqwest::Client::new();
        assert_eq!(fetch_text(&c, &url, "x", 100).await.unwrap(), "[1]");
        assert!(matches!(
            fetch_text(&c, &url, "x", 1).await,
            Err(FetchError::TooLarge(1))
        ));
        let missing = reqwest::Url::from_file_path(dir.path().join("b.json"))
            .unwrap()
            .to_string();
        assert!(matches!(
            fetch_text(&c, &missing, "x", 100).await,
            Err(FetchError::Status(404))
        ));
    }
}
