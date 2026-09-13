//! IPC error type. Serialized as a plain string for the webview.

/// Failure returned to the UI.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    /// Wrapped core / IO / provider error.
    #[error("{0}")]
    Msg(String),
}

impl AppError {
    pub(crate) fn msg(text: impl Into<String>) -> Self {
        Self::Msg(text.into())
    }

    pub(crate) fn io(path: &std::path::Path, err: &std::io::Error) -> Self {
        Self::Msg(format!("io error at {}: {err}", path.display()))
    }
}

impl From<storycraft_core::Error> for AppError {
    fn from(err: storycraft_core::Error) -> Self {
        Self::Msg(err.to_string())
    }
}

impl From<storycraft_llm::Error> for AppError {
    fn from(err: storycraft_llm::Error) -> Self {
        Self::Msg(err.to_string())
    }
}

impl From<storycraft_auth::Error> for AppError {
    fn from(err: storycraft_auth::Error) -> Self {
        Self::Msg(err.to_string())
    }
}

impl serde::Serialize for AppError {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}
