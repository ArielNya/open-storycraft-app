//! OAuth errors. Token values are never included.

use thiserror::Error;

/// Device-code or token-file failure.
#[derive(Debug, Error)]
pub enum Error {
    /// Transport or TLS failure.
    #[error("HTTP transport error")]
    Transport(#[source] reqwest::Error),
    /// Auth server returned a non-success status.
    #[error("HTTP {status}: {body}")]
    Http {
        /// Status code.
        status: u16,
        /// Truncated body.
        body: String,
    },
    /// User did not approve the device code in time.
    #[error("device code expired")]
    Expired,
    /// User denied the authorization request.
    #[error("access denied")]
    AccessDenied,
    /// Caller cancelled polling.
    #[error("request cancelled")]
    Cancelled,
    /// Unexpected JSON from the auth server.
    #[error("invalid OAuth payload: {0}")]
    InvalidPayload(String),
    /// Token file IO.
    #[error("io error at {path}")]
    Io {
        /// Path of the token file.
        path: std::path::PathBuf,
        /// Underlying error.
        #[source]
        source: std::io::Error,
    },
}

impl Error {
    pub(crate) fn from_oauth_error(code: &str, fallback_status: u16, body: &str) -> Self {
        match code {
            "expired_token" | "expired" => Self::Expired,
            "access_denied" => Self::AccessDenied,
            other => Self::Http {
                status: fallback_status,
                body: format!("{other}: {body}"),
            },
        }
    }
}
