//! Client for the server's web API (`docs/architecture/web-api.md` in Poe2ModdingProject).
//!
//! Only the endpoints the launcher uses: `GET /server`, `POST /register`, `POST /login` (kind
//! `launcher`), `POST /logout`, `POST /game-token`, `GET /me` and `POST /web-token`.

use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::modsync::Advertisement;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    /// No answer: wrong address, server down, TLS or timeout.
    #[error("cannot reach the server: {0}")]
    Unreachable(String),
    /// The server answered with an error status.
    #[error("{message}")]
    Status { status: u16, message: String },
    #[error("unexpected answer from the server: {0}")]
    Malformed(String),
}

impl ApiError {
    /// 401: the token (or password) was not accepted.
    pub fn is_unauthorized(&self) -> bool {
        matches!(self, ApiError::Status { status: 401, .. })
    }
}

/// `GET /server`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ServerInfo {
    pub name: String,
    pub version: String,
    pub cluster_id: String,
    pub online: u32,
    pub open_registration: bool,
    pub login_host: String,
    pub login_port: u16,
    /// Absent on servers older than the launcher support.
    #[serde(default)]
    pub instance_port: Option<u16>,
    #[serde(default)]
    pub gateway_region: Option<String>,
    pub uptime_secs: u64,
    /// Message of the day (optional, not sent by servers before the web API gained it): plain
    /// text with line breaks, shown on Home. Capped when read.
    #[serde(default, deserialize_with = "motd_field")]
    pub motd: Option<String>,
    /// The client mods the server requires or offers (optional; servers without mod support
    /// send nothing). See [`crate::modsync`].
    #[serde(default)]
    pub mods: Advertisement,
}

/// Longest message of the day the launcher keeps (characters).
pub const MAX_MOTD: usize = 2000;

/// One entry of `GET /leagues` (the server's active leagues). `banner_url` is the server's art
/// path (`/api/v1/art/...`); the launcher fetches it and hands the page a `data:` URL, because the
/// page may not load images from other hosts.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LeagueInfo {
    pub id: String,
    pub display_name: String,
    #[serde(default)]
    pub family: String,
    #[serde(default)]
    pub hardcore: bool,
    #[serde(default)]
    pub ssf: bool,
    #[serde(default)]
    pub permanent: bool,
    #[serde(default)]
    pub is_default: bool,
    #[serde(default)]
    pub starts_at: Option<i64>,
    #[serde(default)]
    pub ends_at: Option<i64>,
    #[serde(default, skip_serializing)]
    pub banner_url: Option<String>,
    /// The banner as a `data:image/png;base64,...` URL (set by the launcher, base leagues only).
    #[serde(default, skip_deserializing)]
    pub banner: Option<String>,
}

/// Most banner images fetched per server, and the largest one accepted.
const MAX_BANNERS: usize = 8;
const MAX_BANNER_BYTES: usize = 600 * 1024;

/// Standard base64 (RFC 4648) of `bytes`.
pub fn base64(bytes: &[u8]) -> String {
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for c in bytes.chunks(3) {
        let n = (u32::from(c[0]) << 16)
            | (u32::from(*c.get(1).unwrap_or(&0)) << 8)
            | u32::from(*c.get(2).unwrap_or(&0));
        out.push(T[(n >> 18) as usize & 63] as char);
        out.push(T[(n >> 12) as usize & 63] as char);
        out.push(if c.len() > 1 {
            T[(n >> 6) as usize & 63] as char
        } else {
            '='
        });
        out.push(if c.len() > 2 {
            T[n as usize & 63] as char
        } else {
            '='
        });
    }
    out
}

/// A missing, empty or non-text `motd` is `None`; a long one is cut; control characters other
/// than line breaks are dropped.
fn motd_field<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Option<String>, D::Error> {
    let v = Option::<serde_json::Value>::deserialize(d)?;
    let Some(serde_json::Value::String(s)) = v else {
        return Ok(None);
    };
    let s: String = s
        .replace("\r\n", "\n")
        .chars()
        .filter(|c| !c.is_control() || matches!(c, '\n' | '\t'))
        .take(MAX_MOTD)
        .collect();
    let s = s.trim().to_owned();
    Ok((!s.is_empty()).then_some(s))
}

/// `GET /server` with the time the answer took, measured by the launcher.
#[derive(Debug, Clone, Serialize)]
pub struct ServerStatus {
    pub info: ServerInfo,
    pub ping_ms: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AccountInfo {
    pub id: u32,
    pub username: String,
    pub role: String,
}

/// `POST /login` and `POST /game-token`.
#[derive(Clone, Serialize, Deserialize)]
pub struct TokenResponse {
    pub token: String,
    pub kind: String,
    pub expires_at: i64,
    pub account: AccountInfo,
}

impl std::fmt::Debug for TokenResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TokenResponse")
            .field("token", &"<redacted>")
            .field("kind", &self.kind)
            .field("expires_at", &self.expires_at)
            .field("account", &self.account)
            .finish()
    }
}

/// `POST /web-token`: a single-use sign-in token for the web admin.
#[derive(Clone, Deserialize)]
pub struct WebTokenResponse {
    pub token: String,
    pub expires_in_secs: u64,
    /// `/#/token-login?t=<token>`: appended to the server's web address.
    pub login_path: String,
}

impl std::fmt::Debug for WebTokenResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WebTokenResponse")
            .field("token", &"<redacted>")
            .field("expires_in_secs", &self.expires_in_secs)
            .field("login_path", &"<redacted>")
            .finish()
    }
}

#[derive(Clone)]
pub struct ApiClient {
    http: reqwest::Client,
    base: String,
}

impl ApiClient {
    /// `base` is the normalised base URL (no trailing slash, no `/api/v1`).
    pub fn new(base: &str) -> Self {
        Self::with_timeouts(base, Duration::from_secs(6), Duration::from_secs(15))
    }

    /// For status checks of many servers at once: short timeouts.
    pub fn with_timeouts(base: &str, connect: Duration, total: Duration) -> Self {
        let http = reqwest::Client::builder()
            .timeout(total)
            .connect_timeout(connect)
            .user_agent(concat!("poe2-launcher/", env!("CARGO_PKG_VERSION")))
            .build()
            .expect("the HTTP client builds");
        Self {
            http,
            base: base.trim_end_matches('/').to_owned(),
        }
    }

    fn url(&self, path: &str) -> String {
        format!("{}/api/v1{path}", self.base)
    }

    pub async fn server_info(&self) -> Result<ServerInfo, ApiError> {
        decode(self.http.get(self.url("/server")).send().await).await
    }

    /// `GET /leagues` with each base league's banner fetched (see [`LeagueInfo`]). An old server
    /// without the endpoint (404) has no leagues to show: an empty list, not an error.
    pub async fn leagues(&self) -> Result<Vec<LeagueInfo>, ApiError> {
        let mut list: Vec<LeagueInfo> =
            match decode(self.http.get(self.url("/leagues")).send().await).await {
                Ok(l) => l,
                Err(ApiError::Status { status: 404, .. }) => return Ok(Vec::new()),
                Err(e) => return Err(e),
            };
        let mut fetched = 0;
        for l in list.iter_mut().filter(|l| !l.hardcore && !l.ssf) {
            let Some(path) = l.banner_url.as_deref() else {
                continue;
            };
            // Only the server's own art endpoint: never an address the server names.
            if fetched >= MAX_BANNERS || !path.starts_with("/api/v1/art/") || path.contains("..") {
                continue;
            }
            fetched += 1;
            let url = format!("{}{path}", self.base);
            if let Ok(res) = self.http.get(url).send().await
                && res.status().is_success()
                && let Ok(bytes) = res.bytes().await
                && bytes.len() <= MAX_BANNER_BYTES
                && bytes.starts_with(b"\x89PNG")
            {
                l.banner = Some(format!("data:image/png;base64,{}", base64(&bytes)));
            }
        }
        Ok(list)
    }

    /// `GET <path>` without a token, decoded as `T` (e.g. `/local/metrics`).
    pub async fn get_json<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
    ) -> Result<T, ApiError> {
        decode(self.http.get(self.url(path)).send().await).await
    }

    /// `GET /server` and how long the whole request took (connect, TLS and answer).
    pub async fn server_status(&self) -> Result<ServerStatus, ApiError> {
        let t = std::time::Instant::now();
        let info = self.server_info().await?;
        Ok(ServerStatus {
            info,
            ping_ms: t.elapsed().as_millis().min(u32::MAX as u128) as u32,
        })
    }

    pub async fn register(&self, username: &str, password: &str) -> Result<AccountInfo, ApiError> {
        let body = serde_json::json!({"username": username, "password": password});
        decode(
            self.http
                .post(self.url("/register"))
                .json(&body)
                .send()
                .await,
        )
        .await
    }

    /// A launcher token (lives `[accounts] remember_days`, valid for account endpoints only).
    pub async fn login(&self, username: &str, password: &str) -> Result<TokenResponse, ApiError> {
        let body =
            serde_json::json!({"username": username, "password": password, "kind": "launcher"});
        decode(self.http.post(self.url("/login")).json(&body).send().await).await
    }

    /// Revokes the token that is sent.
    pub async fn logout(&self, token: &str) -> Result<(), ApiError> {
        let res = self
            .http
            .post(self.url("/logout"))
            .bearer_auth(token)
            .send()
            .await;
        decode_empty(res).await
    }

    /// The account the token belongs to, with its current role.
    pub async fn me(&self, token: &str) -> Result<AccountInfo, ApiError> {
        decode(
            self.http
                .get(self.url("/me"))
                .bearer_auth(token)
                .send()
                .await,
        )
        .await
    }

    /// Exchanges the launcher token of an admin for a single-use web sign-in token.
    pub async fn web_token(&self, token: &str) -> Result<WebTokenResponse, ApiError> {
        decode(
            self.http
                .post(self.url("/web-token"))
                .bearer_auth(token)
                .send()
                .await,
        )
        .await
    }

    /// Exchanges the launcher token for a one-off `game_remember` token for the client's
    /// credentials file.
    pub async fn game_token(&self, token: &str) -> Result<TokenResponse, ApiError> {
        decode(
            self.http
                .post(self.url("/game-token"))
                .bearer_auth(token)
                .send()
                .await,
        )
        .await
    }
}

async fn check(
    res: Result<reqwest::Response, reqwest::Error>,
) -> Result<reqwest::Response, ApiError> {
    let res = res.map_err(|e| ApiError::Unreachable(describe(e)))?;
    if res.status().is_success() {
        return Ok(res);
    }
    let status = res.status().as_u16();
    let text = res.text().await.unwrap_or_default();
    let message = serde_json::from_str::<serde_json::Value>(&text)
        .ok()
        .and_then(|v| v.get("error")?.as_str().map(str::to_owned))
        .unwrap_or_else(|| format!("the server answered HTTP {status}"));
    Err(ApiError::Status { status, message })
}

async fn decode<T: serde::de::DeserializeOwned>(
    res: Result<reqwest::Response, reqwest::Error>,
) -> Result<T, ApiError> {
    check(res)
        .await?
        .json::<T>()
        .await
        .map_err(|e| ApiError::Malformed(e.to_string()))
}

async fn decode_empty(res: Result<reqwest::Response, reqwest::Error>) -> Result<(), ApiError> {
    check(res).await.map(|_| ())
}

fn describe(e: reqwest::Error) -> String {
    if e.is_timeout() {
        "timed out".into()
    } else if e.is_connect() {
        "connection refused or host not found".into()
    } else {
        // Strip the URL: it can carry credentials the user typed.
        e.without_url().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_matches_the_rfc_vectors() {
        assert_eq!(base64(b""), "");
        assert_eq!(base64(b"f"), "Zg==");
        assert_eq!(base64(b"fo"), "Zm8=");
        assert_eq!(base64(b"foo"), "Zm9v");
        assert_eq!(base64(b"foobar"), "Zm9vYmFy");
    }

    #[test]
    fn league_list_decodes_the_public_shape() {
        let j = r#"[{"id":"Dawn","display_name":"Dawn","family":"Dawn","hardcore":false,"ssf":false,"permanent":false,"is_default":true,"banner_url":"/api/v1/art/x","starts_at":null,"ends_at":5,"flag":"X"}]"#;
        let l: Vec<LeagueInfo> = serde_json::from_str(j).unwrap();
        assert_eq!(l[0].id, "Dawn");
        assert_eq!(l[0].ends_at, Some(5));
        assert!(l[0].banner.is_none());
    }

    #[test]
    fn debug_output_hides_tokens() {
        let r = TokenResponse {
            token: "ab".repeat(32),
            kind: "launcher".into(),
            expires_at: 1,
            account: AccountInfo {
                id: 1,
                username: "alice".into(),
                role: "player".into(),
            },
        };
        assert!(!format!("{r:?}").contains(&"ab".repeat(32)));
    }

    #[test]
    fn web_tokens_are_not_printed() {
        let w = WebTokenResponse {
            token: "cd".repeat(32),
            expires_in_secs: 60,
            login_path: format!("/#/token-login?t={}", "cd".repeat(32)),
        };
        assert!(!format!("{w:?}").contains("cdcd"));
    }

    #[test]
    fn old_servers_without_the_new_fields_still_parse() {
        let info: ServerInfo = serde_json::from_str(
            r#"{"name":"n","version":"0.5.5f","cluster_id":"c","online":0,"open_registration":false,
                "login_host":"h","login_port":20481,"uptime_secs":3}"#,
        )
        .unwrap();
        assert_eq!(info.instance_port, None);
        assert_eq!(info.motd, None);
    }

    #[test]
    fn the_message_of_the_day_is_optional_and_tidied() {
        let parse = |motd: &str| -> Option<String> {
            serde_json::from_str::<ServerInfo>(&format!(
                r#"{{"name":"n","version":"0.5.5f","cluster_id":"c","online":0,"open_registration":false,
                    "login_host":"h","login_port":20481,"uptime_secs":3,"motd":{motd}}}"#
            ))
            .unwrap()
            .motd
        };
        assert_eq!(
            parse(r#""Hello\r\nworld\u0007""#),
            Some("Hello\nworld".into())
        );
        assert_eq!(parse(r#""   ""#), None);
        assert_eq!(parse("null"), None);
        assert_eq!(
            parse("42"),
            None,
            "a motd of the wrong type must not break the status"
        );
        let long = format!("\"{}\"", "x".repeat(MAX_MOTD + 10));
        assert_eq!(parse(&long).unwrap().chars().count(), MAX_MOTD);
    }
}
