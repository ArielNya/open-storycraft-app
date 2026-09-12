//! Build the v1 LLM messages: skill instructions + packed files + answers.

use crate::Error;
use crate::catalog::SkillManifest;
use crate::output::output_rel_path;
use crate::pack::{DEFAULT_PER_FILE_CHARS, DEFAULT_TOTAL_CHARS, PackedContent, pack_skill};

/// System + user strings for one skill run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Prompt {
    /// Skill instructions and host contract.
    pub system: String,
    /// Packed files and the user's answers.
    pub user: String,
}

/// Compile the prompt the model sees.
#[must_use]
pub fn build_prompt(
    manifest: &SkillManifest,
    skill_md: &str,
    packed: &PackedContent,
    answers: &[String],
) -> Prompt {
    let output = output_rel_path(&manifest.name).unwrap_or("(none — do not write Wiki files)");
    let mut system = String::new();
    system.push_str("You are running one Open Storycraft skill. Follow SKILL.md exactly.\n");
    system
        .push_str("Output only the deliverable file contents. No preamble, no closing question.\n");
    system.push_str(
        "The host saves a preview and asks the user \"What should I change?\" after you finish.\n",
    );
    system.push_str("Do not invent Wiki files this skill did not ask for.\n");
    system.push_str("Skill: ");
    system.push_str(&manifest.name);
    system.push('\n');
    system.push_str("Output path: ");
    system.push_str(output);
    system.push_str("\n\n");
    system.push_str(skill_md);

    let mut user = String::new();
    if answers.is_empty() {
        user.push_str("# Answers\n(none provided; choose reasonable defaults and flag them.)\n\n");
    } else {
        user.push_str("# Answers\n");
        for (i, answer) in answers.iter().enumerate() {
            user.push_str(&(i.saturating_add(1)).to_string());
            user.push_str(". ");
            user.push_str(answer);
            user.push('\n');
        }
        user.push('\n');
    }
    user.push_str("# Packed files\n");
    user.push_str("Only the files below. Do not assume any other reference exists.\n\n");
    for file in &packed.files {
        user.push_str("## ");
        user.push_str(&file.relative);
        if file.truncated {
            user.push_str(" (truncated)");
        }
        user.push_str("\n\n");
        user.push_str(&file.text);
        user.push_str("\n\n");
    }
    Prompt { system, user }
}

/// Pack the skill, hash it, and compile the prompt.
///
/// # Errors
///
/// Returns [`Error::Io`] if `SKILL.md` or a linked reference cannot be read.
pub fn prepare_skill(
    manifest: &SkillManifest,
    answers: &[String],
) -> Result<(PackedContent, Prompt), Error> {
    let pack = pack_skill(manifest);
    let packed = pack.materialize(DEFAULT_PER_FILE_CHARS, DEFAULT_TOTAL_CHARS)?;
    let skill_md = std::fs::read_to_string(manifest.skill_md())
        .map_err(|err| Error::io(manifest.skill_md(), err))?;
    let prompt = build_prompt(manifest, &skill_md, &packed, answers);
    Ok((packed, prompt))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;
    use crate::pack::{PackKind, PackedText};
    use std::path::PathBuf;

    fn manifest() -> SkillManifest {
        SkillManifest {
            name: "fiction-genre".into(),
            description: "genre".into(),
            version: None,
            category: None,
            workflow_position: None,
            requires: Vec::new(),
            next_skill: None,
            output_format: None,
            reference_files: Vec::new(),
            skill_dir: PathBuf::from("/tmp/fiction-genre"),
        }
    }

    #[test]
    fn includes_output_path_and_answers() {
        let packed = PackedContent {
            skill: "fiction-genre".into(),
            files: vec![PackedText {
                kind: PackKind::Skill,
                relative: "SKILL.md".into(),
                text: "# Genre Selector".into(),
                truncated: false,
            }],
            hash: "abc".into(),
        };
        let prompt = build_prompt(
            &manifest(),
            "# Genre Selector\n",
            &packed,
            &["Fantasy".into(), "".into(), "Night Market".into()],
        );
        assert!(prompt.system.contains("Wiki/Style/genre.md"));
        assert!(prompt.user.contains("1. Fantasy"));
        assert!(prompt.user.contains("3. Night Market"));
    }
}
