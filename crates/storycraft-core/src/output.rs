//! Canonical Wiki / chapter destinations for each skill.

use crate::Error;
use crate::project::ProjectRoot;

/// Where a skill's deliverable lands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkillOutput {
    /// Spark / diagnostic: preview only, never a Wiki write.
    None,
    /// Single markdown file under the project root.
    WikiFile(&'static str),
    /// Not in the v1 map; host keeps a preview and does not guess.
    Unknown,
}

/// Canonical output for a skill folder name.
#[must_use]
pub fn skill_output(skill: &str) -> SkillOutput {
    match skill {
        "fiction-story-sparks" | "burstiness-check" | "pangram" | "coldread" => SkillOutput::None,
        "fiction-genre" => SkillOutput::WikiFile("Wiki/Style/genre.md"),
        "fiction-audience" => SkillOutput::WikiFile("Wiki/Style/audience.md"),
        "fiction-theme" => SkillOutput::WikiFile("Wiki/Story/theme.md"),
        "fiction-synopsis" => SkillOutput::WikiFile("Wiki/Story/synopsis.md"),
        "fiction-style" => SkillOutput::WikiFile("Wiki/Style/style_guide.md"),
        "fiction-voiceprompt" => SkillOutput::WikiFile("Wiki/Style/voice_prompt.md"),
        "fiction-outline" => SkillOutput::WikiFile("Wiki/Outline/outline.md"),
        _ => SkillOutput::Unknown,
    }
}

/// Relative output path if this skill writes exactly one Wiki file.
#[must_use]
pub fn output_rel_path(skill: &str) -> Option<&'static str> {
    match skill_output(skill) {
        SkillOutput::WikiFile(path) => Some(path),
        SkillOutput::None | SkillOutput::Unknown => None,
    }
}

/// Whether a required skill already has a non-empty output on disk.
#[must_use]
pub fn requirement_satisfied(project: &ProjectRoot, required_skill: &str) -> bool {
    match skill_output(required_skill) {
        SkillOutput::None | SkillOutput::Unknown => true,
        SkillOutput::WikiFile(rel) => {
            let path = project.path().join(rel);
            std::fs::metadata(&path)
                .ok()
                .is_some_and(|meta| meta.is_file() && meta.len() > 40)
        }
    }
}

/// Required skills whose outputs are missing.
#[must_use]
pub fn missing_requirements(project: Option<&ProjectRoot>, requires: &[String]) -> Vec<String> {
    let Some(project) = project else {
        return requires.to_vec();
    };
    requires
        .iter()
        .filter(|skill| !requirement_satisfied(project, skill))
        .cloned()
        .collect()
}

/// Fail if the skill's `requires` are not on disk.
///
/// # Errors
///
/// Returns [`Error::RequiresMissing`] listing the unmet skill ids.
pub fn ensure_requires(project: Option<&ProjectRoot>, requires: &[String]) -> Result<(), Error> {
    let missing = missing_requirements(project, requires);
    if missing.is_empty() {
        Ok(())
    } else {
        Err(Error::RequiresMissing(missing.join(", ")))
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;

    #[test]
    fn sparks_never_write_wiki() {
        assert_eq!(skill_output("fiction-story-sparks"), SkillOutput::None);
        assert!(output_rel_path("fiction-story-sparks").is_none());
    }

    #[test]
    fn genre_writes_style_genre() {
        assert_eq!(
            output_rel_path("fiction-genre"),
            Some("Wiki/Style/genre.md")
        );
    }
}
