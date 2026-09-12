//! OpenAI-compatible chat completions and Responses streaming.
//!
//! Secrets are never logged. 403 is a typed tier-denied error.

#![deny(clippy::correctness)]

mod client;
mod error;
mod secret;
mod sse;

pub use client::{ApiStyle, OpenAiClient, ProviderConfig};
pub use error::Error;
pub use secret::Secret;
pub use tokio_util::sync::CancellationToken;
