//! Device-code OAuth for xAI. Client id and auth URLs are configurable.
//!
//! This is the community Grok-CLI grant, not an xAI partner agreement.

#![deny(clippy::correctness)]

mod device;
mod error;
mod store;

pub use device::{DeviceAuth, DeviceCode};
pub use error::Error;
pub use store::{TokenSet, TokenStore, default_token_path};
pub use tokio_util::sync::CancellationToken;

/// Default auth host. Override in config — xAI can change this.
pub const DEFAULT_AUTH_BASE: &str = "https://auth.x.ai";
/// Public Grok-CLI ecosystem client id. Not an official third-party app.
pub const DEFAULT_CLIENT_ID: &str = "b1a00492-073a-47ea-816f-4c329264a828";
/// Scopes seen in the wild. Override if xAI changes them.
pub const DEFAULT_SCOPES: &str = "openid profile email offline_access grok-cli:access api:access";
/// Inference base URL used with the access token.
pub const DEFAULT_API_BASE: &str = "https://api.x.ai/v1";

/// Configurable OAuth endpoints. Nothing here is compiled as the only option.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OAuthConfig {
    /// `https://auth.x.ai` (no path).
    pub auth_base: String,
    /// Public client id.
    pub client_id: String,
    /// Space-separated scopes.
    pub scopes: String,
}

impl Default for OAuthConfig {
    fn default() -> Self {
        Self {
            auth_base: DEFAULT_AUTH_BASE.to_owned(),
            client_id: DEFAULT_CLIENT_ID.to_owned(),
            scopes: DEFAULT_SCOPES.to_owned(),
        }
    }
}

impl OAuthConfig {
    /// `POST {auth}/oauth2/device/code`
    #[must_use]
    pub fn device_code_url(&self) -> String {
        join(&self.auth_base, "/oauth2/device/code")
    }

    /// `POST {auth}/oauth2/token`
    #[must_use]
    pub fn token_url(&self) -> String {
        join(&self.auth_base, "/oauth2/token")
    }
}

fn join(base: &str, path: &str) -> String {
    format!("{}{}", base.trim_end_matches('/'), path)
}
