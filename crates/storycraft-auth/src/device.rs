//! RFC 8628 device-code grant against a configurable auth host.

use std::time::Duration;

use serde::Deserialize;
use tokio_util::sync::CancellationToken;
use tracing::info;

use crate::store::TokenSet;
use crate::{Error, OAuthConfig};

/// Pending device authorization shown to the user.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceCode {
    /// Opaque code the token endpoint wants back.
    pub device_code: String,
    /// Short code the user types at the verification URI.
    pub user_code: String,
    /// Page the user should open.
    pub verification_uri: String,
    /// Prefill URL when the server sent one.
    pub verification_uri_complete: Option<String>,
    /// Polling interval.
    pub interval: Duration,
    /// How long the `user_code` is valid.
    pub expires_in: Duration,
}

/// Device-code client.
pub struct DeviceAuth {
    http: reqwest::Client,
    config: OAuthConfig,
}

impl DeviceAuth {
    /// Build a client with a 30s connect/request timeout.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Transport`] if reqwest cannot be built.
    pub fn new(config: OAuthConfig) -> Result<Self, Error> {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(Error::Transport)?;
        Ok(Self { http, config })
    }

    /// `POST /oauth2/device/code`.
    ///
    /// # Errors
    ///
    /// Returns transport or payload errors.
    pub async fn request_code(&self) -> Result<DeviceCode, Error> {
        let url = self.config.device_code_url();
        info!(url = %url, "requesting device code");
        let response = self
            .http
            .post(&url)
            .form(&[
                ("client_id", self.config.client_id.as_str()),
                ("scope", self.config.scopes.as_str()),
            ])
            .send()
            .await
            .map_err(Error::Transport)?;
        let status = response.status();
        let text = response.text().await.map_err(Error::Transport)?;
        if !status.is_success() {
            return Err(Error::Http {
                status: status.as_u16(),
                body: text.chars().take(400).collect(),
            });
        }
        let raw: DeviceCodeRaw =
            serde_json::from_str(&text).map_err(|err| Error::InvalidPayload(err.to_string()))?;
        Ok(DeviceCode {
            device_code: raw.device_code,
            user_code: raw.user_code,
            verification_uri: raw.verification_uri,
            verification_uri_complete: raw.verification_uri_complete,
            interval: Duration::from_secs(secs_or(raw.interval, 5)),
            expires_in: Duration::from_secs(secs_or(Some(raw.expires_in), 600)),
        })
    }

    /// Poll `/oauth2/token` until the user approves, denies, or time runs out.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Expired`], [`Error::AccessDenied`], [`Error::Cancelled`],
    /// or transport errors.
    pub async fn poll_token(
        &self,
        pending: &DeviceCode,
        cancel: &CancellationToken,
    ) -> Result<TokenSet, Error> {
        let url = self.config.token_url();
        let mut interval = pending.interval;
        let mut polls = 0u32;
        let max_polls = 200u32;
        loop {
            if cancel.is_cancelled() {
                return Err(Error::Cancelled);
            }
            polls = polls.saturating_add(1);
            if polls > max_polls {
                return Err(Error::Expired);
            }
            tokio::select! {
                () = cancel.cancelled() => return Err(Error::Cancelled),
                () = tokio::time::sleep(interval) => {}
            }
            info!(polls, "polling device token");
            let response = self
                .http
                .post(&url)
                .form(&[
                    ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
                    ("device_code", pending.device_code.as_str()),
                    ("client_id", self.config.client_id.as_str()),
                ])
                .send()
                .await
                .map_err(Error::Transport)?;
            let status = response.status();
            let text = response.text().await.map_err(Error::Transport)?;
            if status.is_success() {
                let raw: TokenRaw = serde_json::from_str(&text)
                    .map_err(|err| Error::InvalidPayload(err.to_string()))?;
                return Ok(TokenSet::from_response(
                    raw.access_token,
                    raw.refresh_token,
                    raw.expires_in,
                    raw.token_type,
                ));
            }
            let err_code = oauth_error_code(&text);
            match err_code.as_deref() {
                Some("authorization_pending") => {}
                Some("slow_down") => {
                    interval = interval.saturating_add(Duration::from_secs(5));
                }
                Some(code) => {
                    return Err(Error::from_oauth_error(code, status.as_u16(), &text));
                }
                None => {
                    return Err(Error::Http {
                        status: status.as_u16(),
                        body: text.chars().take(400).collect(),
                    });
                }
            }
        }
    }

    /// Exchange a refresh token for a new access token.
    ///
    /// # Errors
    ///
    /// Returns transport or payload errors.
    pub async fn refresh(&self, refresh_token: &str) -> Result<TokenSet, Error> {
        let url = self.config.token_url();
        let response = self
            .http
            .post(&url)
            .form(&[
                ("grant_type", "refresh_token"),
                ("refresh_token", refresh_token),
                ("client_id", self.config.client_id.as_str()),
            ])
            .send()
            .await
            .map_err(Error::Transport)?;
        let status = response.status();
        let text = response.text().await.map_err(Error::Transport)?;
        if !status.is_success() {
            return Err(Error::Http {
                status: status.as_u16(),
                body: text.chars().take(400).collect(),
            });
        }
        let raw: TokenRaw =
            serde_json::from_str(&text).map_err(|err| Error::InvalidPayload(err.to_string()))?;
        Ok(TokenSet::from_response(
            raw.access_token,
            raw.refresh_token.or_else(|| Some(refresh_token.to_owned())),
            raw.expires_in,
            raw.token_type,
        ))
    }
}

fn secs_or(value: Option<i64>, default: u64) -> u64 {
    match value {
        Some(secs) if secs > 0 => u64::try_from(secs).unwrap_or(default),
        _ => default,
    }
}

#[derive(Deserialize)]
struct DeviceCodeRaw {
    device_code: String,
    user_code: String,
    verification_uri: String,
    #[serde(default)]
    verification_uri_complete: Option<String>,
    #[serde(default)]
    expires_in: i64,
    #[serde(default)]
    interval: Option<i64>,
}

#[derive(Deserialize)]
struct TokenRaw {
    access_token: String,
    #[serde(default)]
    refresh_token: Option<String>,
    #[serde(default)]
    expires_in: Option<u64>,
    #[serde(default)]
    token_type: Option<String>,
}

#[derive(Deserialize)]
struct OAuthErrorBody {
    #[serde(default)]
    error: Option<String>,
}

fn oauth_error_code(body: &str) -> Option<String> {
    serde_json::from_str::<OAuthErrorBody>(body)
        .ok()
        .and_then(|body| body.error)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;

    #[test]
    fn maps_expired() {
        let err = Error::from_oauth_error("expired_token", 400, "{}");
        assert!(matches!(err, Error::Expired));
    }
}
