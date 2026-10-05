//! Build the v1 LLM messages: skill instructions + packed files + answers.

use crate::Error;
use crate::catalog::SkillManifest;
use crate::chunk::LineChunk;
use crate::pack::{DEFAULT_PER_FILE_CHARS, DEFAULT_TOTAL_CHARS, PackedContent, pack_skill};
use crate::project::ProjectRoot;
use crate::wiki::wiki_pack_files;

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
    output_path: Option<&str>,
) -> Prompt {
    let output = output_path.unwrap_or("(none — do not write Wiki files)");
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
    if let crate::output::SkillOutput::Files(paths) = crate::output::skill_output(&manifest.name) {
        system.push_str("Output files: ");
        system.push_str(&paths.join(", "));
        system.push_str(
            "\nWrite each file as one document: a `---` line, then `path: <file>`, then that \
file's own frontmatter keys, then `---`, then its body. No code fences around documents.\n\n",
        );
    } else {
        system.push_str("Output path: ");
        system.push_str(output);
        system.push_str("\n\n");
    }
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

/// Pack the skill (plus Wiki files when a project is open) and compile the prompt.
///
/// `total_chars` is the pack budget; `0` means [`DEFAULT_TOTAL_CHARS`].
///
/// # Errors
///
/// Returns [`Error::Io`] if `SKILL.md` or a linked reference cannot be read.
pub fn prepare_skill(
    manifest: &SkillManifest,
    answers: &[String],
    project: Option<&ProjectRoot>,
    chapter: u32,
    output_path: Option<&str>,
    total_chars: usize,
) -> Result<(PackedContent, Prompt), Error> {
    let total_chars = if total_chars == 0 {
        DEFAULT_TOTAL_CHARS
    } else {
        total_chars
    };
    let mut pack = pack_skill(manifest);
    if let Some(project) = project {
        pack.files
            .extend(wiki_pack_files(project, &manifest.name, chapter));
    }
    let packed = pack.materialize(DEFAULT_PER_FILE_CHARS, total_chars)?;
    let skill_md = std::fs::read_to_string(manifest.skill_md())
        .map_err(|err| Error::io(manifest.skill_md(), err))?;
    let prompt = build_prompt(manifest, &skill_md, &packed, answers, output_path);
    Ok((packed, prompt))
}

/// Prompt for one 40-line editorial window. Return only the rewritten chunk.
#[must_use]
pub fn build_chunk_prompt(base: &Prompt, chunk: &LineChunk, total_lines: usize) -> Prompt {
    let mut system = String::with_capacity(base.system.len() + 160);
    system.push_str(&base.system);
    system.push_str(
        "\n\nHOST OUTPUT CONTRACT — this overrides any output format SKILL.md describes. \
This call is one 40-line window of the chapter. Return ONLY the rewritten lines for this window: \
the chapter text itself with your edits applied. No report, no list of findings, no score card, \
no markdown fences, no preamble. Copy unchanged lines verbatim. If nothing needs changing, \
return the window unchanged.\n",
    );
    let mut user = String::with_capacity(base.user.len() + chunk.text.len() + 80);
    user.push_str(&base.user);
    push_window_header(&mut user, chunk, total_lines);
    user.push_str(&chunk.text);
    Prompt { system, user }
}

/// Prompt for one window of a report skill (`fragment-hunter`,
/// `fiction-line-editor`): findings for this window, in the skill's format.
#[must_use]
pub fn build_report_chunk_prompt(base: &Prompt, chunk: &LineChunk, total_lines: usize) -> Prompt {
    let mut system = String::with_capacity(base.system.len() + 500);
    system.push_str(&base.system);
    system.push_str(
        "\n\nHOST OUTPUT CONTRACT — report mode. This call shows ONE window of the chapter, \
each line prefixed with its chapter line number (`  12| text`). Write this window's findings in \
the report format SKILL.md gives, citing those line numbers. Do not rewrite the chapter and do \
not apply anything: the user reads the report and applies it in a separate step. The host merges \
the windows, so leave out whole-chapter totals, score cards, and closing offers. If nothing in \
this window needs a finding, answer exactly: ",
    );
    system.push_str(crate::chunk::NOTHING_IN_WINDOW);
    system.push('\n');
    let mut user = String::with_capacity(base.user.len() + chunk.text.len() * 2);
    user.push_str(&base.user);
    push_window_header(&mut user, chunk, total_lines);
    user.push_str(&crate::chunk::numbered_window(chunk));
    Prompt { system, user }
}

/// Prompt for one window of `<report skill>:apply`: the chapter text with the
/// saved report's fixes for this window applied, nothing else.
#[must_use]
pub fn build_apply_chunk_prompt(
    base: &Prompt,
    report: &str,
    chunk: &LineChunk,
    total_lines: usize,
) -> Prompt {
    let mut system = String::with_capacity(base.system.len() + 500);
    system.push_str(&base.system);
    system.push_str(
        "\n\nHOST OUTPUT CONTRACT — apply mode. This overrides any output format SKILL.md \
describes. The user saved the report below and asked to apply it. The window is shown with each \
line prefixed by its line number (`  12| text`). Apply only the fixes the report accepts (kills, \
suggested rewrites) whose line falls inside this window; never touch a line the report spares or \
does not mention. Answer with ONLY the lines you change, each as `<line number>| <the whole new \
line>`, one per line — no report, no unchanged lines, no markdown fences, no preamble. The host \
keeps every line you do not return exactly as it is. If no fix falls in this window, answer \
exactly: ",
    );
    system.push_str(crate::chunk::NO_CHANGES);
    system.push('\n');
    let mut user = String::with_capacity(base.user.len() + report.len() + chunk.text.len() * 2);
    user.push_str(&base.user);
    user.push_str("# Saved report\n\n");
    user.push_str(report);
    user.push_str("\n\n");
    push_window_header(&mut user, chunk, total_lines);
    user.push_str(&crate::chunk::numbered_window(chunk));
    Prompt { system, user }
}

fn push_window_header(user: &mut String, chunk: &LineChunk, total_lines: usize) {
    user.push_str("# Chunk (lines ");
    user.push_str(&chunk.start_line.to_string());
    user.push('-');
    user.push_str(&chunk.end_line.to_string());
    user.push_str(" of ");
    user.push_str(&total_lines.to_string());
    user.push_str(")\n\n");
}

/// Prompt for converting one section of a free-form story bible into
/// storybible documents.
#[must_use]
pub fn build_section_prompt(base: &Prompt, index: usize, total: usize, section: &str) -> Prompt {
    let mut system = String::with_capacity(base.system.len() + 400);
    system.push_str(&base.system);
    system.push_str(
        "\n\nHOST OUTPUT CONTRACT — this call converts ONE section of the source bible. \
Return only storybible documents (`---`, `path:` or `slot:`, the file's frontmatter, `---`, body) \
for the material in this section. Other sections are converted in other calls and merged by the \
host, so do not repeat or summarise material that is not in this section, and do not add a title, \
table of contents, or notes outside documents. No code fences. If this section holds nothing \
worth keeping, return exactly: NO DOCUMENTS\n",
    );
    let mut user = String::with_capacity(base.user.len() + section.len() + 80);
    user.push_str(&base.user);
    user.push_str("# Source bible, section ");
    user.push_str(&index.saturating_add(1).to_string());
    user.push_str(" of ");
    user.push_str(&total.to_string());
    user.push_str("\n\n");
    user.push_str(section);
    Prompt { system, user }
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
            Some("Wiki/Style/genre.md"),
        );
        assert!(prompt.system.contains("Wiki/Style/genre.md"));
        assert!(prompt.user.contains("1. Fantasy"));
        assert!(prompt.user.contains("3. Night Market"));
    }
}
