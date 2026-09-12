//! API keys that print as `[redacted]`.

/// Bearer token / API key wrapper.
#[derive(Clone)]
pub struct Secret(String);

impl Secret {
    /// Wrap a secret value.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Reveal the secret at the HTTP boundary only.
    #[must_use]
    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Debug for Secret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("[redacted]")
    }
}

impl std::fmt::Display for Secret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("[redacted]")
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn debug_hides_value() {
        let secret = super::Secret::new("sk-secret");
        assert_eq!(format!("{secret:?}"), "[redacted]");
        assert!(!format!("{secret:?}").contains("sk-"));
    }
}
