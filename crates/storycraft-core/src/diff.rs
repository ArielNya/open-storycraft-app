//! Original vs preview, for the confirmation gate.

use similar::TextDiff;

/// Unified diff of the live file (or empty) against a job preview.
///
/// Returns an empty string when the texts are identical.
#[must_use]
pub fn unified_diff(original: &str, preview: &str) -> String {
    if original == preview {
        return String::new();
    }
    TextDiff::from_lines(original, preview)
        .unified_diff()
        .context_radius(3)
        .header("original", "preview")
        .to_string()
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;

    #[test]
    fn identical_texts_have_no_diff() {
        assert!(unified_diff("a\n", "a\n").is_empty());
    }

    #[test]
    fn replacement_shows_minus_and_plus() {
        let diff = unified_diff("he just looked.\n", "he looked.\n");
        assert!(diff.contains("-he just looked."));
        assert!(diff.contains("+he looked."));
    }
}
