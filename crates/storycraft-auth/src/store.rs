//! Token persistence. File mode 0600 on Unix. Values redacted in Debug.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::Error;

/// Access + refresh tokens. `Debug` prints `[redacted]`.
#[derive(Clone, Serialize, Deserialize)]
pub struct TokenSet {
    /// Bearer token for `api.x.ai`.
    pub access_token: String,
    /// Present when the server issued a refresh token.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    /// Unix timestamp when the access token should be treated as stale.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at_unix: Option<u64>,
    /// Usually `Bearer`.
    #[serde(default)]
    pub token_type: String,
}

impl std::fmt::Debug for TokenSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TokenSet")
            .field("access_token", &"[redacted]")
            .field(
                "refresh_token",
                &self.refresh_token.as_ref().map(|_| "[redacted]"),
            )
            .field("expires_at_unix", &self.expires_at_unix)
            .field("token_type", &self.token_type)
            .finish()
    }
}

impl TokenSet {
    pub(crate) fn from_response(
        access_token: String,
        refresh_token: Option<String>,
        expires_in: Option<u64>,
        token_type: Option<String>,
    ) -> Self {
        let expires_at_unix = expires_in.map(|secs| {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            // Refresh a couple of minutes before expiry.
            now.saturating_add(secs.saturating_sub(120))
        });
        Self {
            access_token,
            refresh_token,
            expires_at_unix,
            token_type: token_type.unwrap_or_else(|| "Bearer".to_owned()),
        }
    }

    /// Whether the access token should be refreshed.
    #[must_use]
    pub fn needs_refresh(&self) -> bool {
        let Some(expiry) = self.expires_at_unix else {
            return false;
        };
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        now >= expiry
    }

    /// Remaining time until we treat the token as stale.
    #[must_use]
    pub fn ttl(&self) -> Option<Duration> {
        let expiry = self.expires_at_unix?;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Some(Duration::from_secs(expiry.saturating_sub(now)))
    }
}

/// JSON file store. Not a keystore — phase 3/4 moves this to OS keyring.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenStore {
    path: PathBuf,
}

impl TokenStore {
    /// Use this file path.
    #[must_use]
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    /// Path of the JSON file.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Load tokens if the file exists.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Io`] or [`Error::InvalidPayload`].
    pub fn load(&self) -> Result<Option<TokenSet>, Error> {
        match fs::read(&self.path) {
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(err) => Err(Error::Io {
                path: self.path.clone(),
                source: err,
            }),
            Ok(bytes) => {
                let set = serde_json::from_slice(&bytes)
                    .map_err(|err| Error::InvalidPayload(err.to_string()))?;
                Ok(Some(set))
            }
        }
    }

    /// Write tokens, creating parent dirs. Unix mode 0600.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Io`] if the file cannot be written.
    pub fn save(&self, tokens: &TokenSet) -> Result<(), Error> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(|err| Error::Io {
                path: parent.to_path_buf(),
                source: err,
            })?;
        }
        let bytes = serde_json::to_vec_pretty(tokens)
            .map_err(|err| Error::InvalidPayload(err.to_string()))?;
        fs::write(&self.path, bytes).map_err(|err| Error::Io {
            path: self.path.clone(),
            source: err,
        })?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = fs::Permissions::from_mode(0o600);
            fs::set_permissions(&self.path, perms).map_err(|err| Error::Io {
                path: self.path.clone(),
                source: err,
            })?;
        }
        Ok(())
    }

    /// Delete the token file if it exists.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Io`] on an unexpected delete failure.
    pub fn clear(&self) -> Result<(), Error> {
        match fs::remove_file(&self.path) {
            Ok(()) => Ok(()),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(err) => Err(Error::Io {
                path: self.path.clone(),
                source: err,
            }),
        }
    }
}

/// `STORYCRAFT_AUTH_FILE`, else `~/.config/open-storycraft/oauth.json`.
#[must_use]
pub fn default_token_path() -> PathBuf {
    if let Some(custom) = std::env::var_os("STORYCRAFT_AUTH_FILE") {
        return PathBuf::from(custom);
    }
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
        .unwrap_or_else(|| PathBuf::from("."));
    base.join("open-storycraft").join("oauth.json")
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;

    #[test]
    fn debug_redacts_tokens() {
        let set = TokenSet::from_response(
            "secret-access".into(),
            Some("secret-refresh".into()),
            Some(3600),
            None,
        );
        let rendered = format!("{set:?}");
        assert!(!rendered.contains("secret-access"));
        assert!(!rendered.contains("secret-refresh"));
        assert!(rendered.contains("[redacted]"));
    }

    #[test]
    fn round_trip_file() {
        let tmp = tempfile::tempdir().unwrap();
        let store = TokenStore::new(tmp.path().join("oauth.json"));
        let set = TokenSet::from_response("tok".into(), None, Some(60), None);
        store.save(&set).unwrap();
        let loaded = store.load().unwrap().unwrap();
        assert_eq!(loaded.access_token, "tok");
    }
}
