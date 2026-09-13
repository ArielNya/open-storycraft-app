//! API keys that print as `[redacted]`.

use zeroize::Zeroize;

/// Bearer token / API key wrapper.
///
/// Prints as `[redacted]` and wipes its bytes when dropped, so a key does not
/// outlive the client that used it.
#[derive(Clone)]
pub struct Secret(String);

impl Drop for Secret {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

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
    use super::*;

    #[test]
    fn debug_hides_value() {
        let secret = Secret::new("sk-secret");
        assert_eq!(format!("{secret:?}"), "[redacted]");
        assert_eq!(format!("{secret}"), "[redacted]");
        assert!(!format!("{secret:?}").contains("sk-"));
    }

    #[test]
    fn dropping_one_copy_does_not_wipe_the_other() {
        // `Secret` wipes itself on drop, and the client hands copies around.
        let secret = Secret::new("sk-secret");
        let copy = secret.clone();
        assert_eq!(copy.expose(), "sk-secret");
        drop(copy);
        assert_eq!(secret.expose(), "sk-secret", "the survivor keeps its bytes");
    }
}
