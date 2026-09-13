//! Provider errors. Messages do not include credentials.

use thiserror::Error;

/// Failure talking to an OpenAI-compatible endpoint.
#[derive(Debug, Error)]
pub enum Error {
    /// Transport or TLS failure.
    #[error("HTTP transport error")]
    Transport(#[source] reqwest::Error),
    /// Provider returned a non-success status other than 401/403.
    #[error("HTTP {status}: {body}")]
    Http {
        /// Status code.
        status: u16,
        /// Truncated body, never a copied Authorization header.
        body: String,
    },
    /// 401 from the inference API.
    #[error("provider rejected credentials")]
    Unauthorized,
    /// 403 — OAuth allowlist / tier. `SuperGrok` is not a guarantee of API access.
    #[error(
        "inference returned 403 (tier denied); SuperGrok is not a guarantee of API access — use an API key"
    )]
    TierDenied,
    /// Caller cancelled the job.
    #[error("request cancelled")]
    Cancelled,
    /// Stream or JSON body could not be parsed.
    #[error("invalid provider payload: {0}")]
    InvalidPayload(String),
    /// Empty completion after the stream ended.
    #[error("provider returned no text")]
    EmptyCompletion,
    /// The provider answered, but with no model list this client understands.
    #[error("provider returned no model list; type the model id instead")]
    NoModels,
}

impl Error {
    pub(crate) fn from_status(status: reqwest::StatusCode, body: &str) -> Self {
        match status.as_u16() {
            401 => Self::Unauthorized,
            403 => Self::TierDenied,
            code => Self::Http {
                status: code,
                body: truncate(body),
            },
        }
    }
}

fn truncate(body: &str) -> String {
    const MAX: usize = 400;
    if body.len() <= MAX {
        body.to_owned()
    } else {
        let mut end = MAX;
        while end > 0 && !body.is_char_boundary(end) {
            end -= 1;
        }
        let mut cut = body[..end].to_owned();
        cut.push('…');
        cut
    }
}
