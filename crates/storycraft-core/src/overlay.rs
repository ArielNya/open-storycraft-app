//! Optional overlays stay off unless the user enables them.

/// Overlay skill ids named by the orchestrator. Not in the vendored pack.
pub const OVERLAY_SKILLS: &[&str] = &[
    "ao3-writer",
    "ao3-narrative-voice",
    "ao3-dialogue-engine",
    "ao3-scene-review",
    "anti-slop-editor",
];

/// Whether `skill` is an optional overlay (`ao3-*` or `anti-slop-editor`).
#[must_use]
pub fn is_overlay(skill: &str) -> bool {
    skill == "anti-slop-editor" || skill.starts_with("ao3-")
}

/// Core skills are always allowed. Overlays need an explicit enable.
///
/// `enabled` items may be a skill id, `ao3` (all `ao3-*`), or `*` (all overlays).
#[must_use]
pub fn overlay_allowed(skill: &str, enabled: &[String]) -> bool {
    if !is_overlay(skill) {
        return true;
    }
    enabled
        .iter()
        .any(|item| item == "*" || item == skill || (item == "ao3" && skill.starts_with("ao3-")))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;

    #[test]
    fn core_skills_are_never_gated() {
        assert!(!is_overlay("fiction-genre"));
        assert!(overlay_allowed("fiction-writechapter", &[]));
    }

    #[test]
    fn overlays_match_orchestrator_names() {
        assert!(is_overlay("ao3-writer"));
        assert!(is_overlay("ao3-narrative-voice"));
        assert!(is_overlay("anti-slop-editor"));
        assert!(!is_overlay("fiction-ao3-something"));
    }

    #[test]
    fn hidden_until_enabled() {
        assert!(!overlay_allowed("ao3-writer", &[]));
        assert!(overlay_allowed("ao3-writer", &["ao3-writer".into()]));
        assert!(overlay_allowed("ao3-scene-review", &["ao3".into()]));
        assert!(!overlay_allowed("anti-slop-editor", &["ao3".into()]));
        assert!(overlay_allowed("anti-slop-editor", &["*".into()]));
    }
}
