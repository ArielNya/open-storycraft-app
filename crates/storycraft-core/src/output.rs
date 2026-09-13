//! Canonical Wiki / chapter destinations for each skill.

use crate::Error;
use crate::chunk::is_chunked_skill;
use crate::paths::{find_chapter_prose, find_psych_file, find_scene_files, wiki_markdown_files};
use crate::project::ProjectRoot;
use crate::wiki::{default_chapter_rel, default_psych_rel, default_scene_rel};

/// Where a skill's deliverable lands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkillOutput {
    /// Spark / diagnostic: preview only, never a Wiki write.
    None,
    /// Single markdown file under the project root.
    WikiFile(&'static str),
    /// `Wiki/Characters/<Name>.md` files split from the preview.
    CharactersDir,
    /// Many files at once, each document naming its own destination in
    /// frontmatter. Used by the storybible importer.
    Bundle,
    /// Chapter-scoped scene plan.
    Scene,
    /// Chapter-scoped psych pass.
    Psych,
    /// New or existing chapter prose.
    ChapterProse,
    /// Editorial rewrite of existing chapter prose.
    ChapterRewrite,
    /// Not in the v1 map; host keeps a preview and does not guess.
    Unknown,
}

/// Canonical output kind for a skill folder name.
#[must_use]
pub fn skill_output(skill: &str) -> SkillOutput {
    match skill {
        "fiction-story-sparks"
        | "burstiness-check"
        | "pangram"
        | "coldread"
        | "name-generator"
        | "town-generator" => SkillOutput::None,
        "fiction-storybible" => SkillOutput::WikiFile("storybible.md"),
        "storybible-import" => SkillOutput::Bundle,
        "fiction-genre" => SkillOutput::WikiFile("Wiki/Style/genre.md"),
        "fiction-audience" => SkillOutput::WikiFile("Wiki/Style/audience.md"),
        "fiction-theme" => SkillOutput::WikiFile("Wiki/Story/theme.md"),
        "fiction-synopsis" => SkillOutput::WikiFile("Wiki/Story/synopsis.md"),
        "fiction-style" => SkillOutput::WikiFile("Wiki/Style/style_guide.md"),
        "fiction-voiceprompt" => SkillOutput::WikiFile("Wiki/Style/voice_prompt.md"),
        "fiction-outline" => SkillOutput::WikiFile("Wiki/Outline/outline.md"),
        "fiction-characters" => SkillOutput::CharactersDir,
        "fiction-scenes" => SkillOutput::Scene,
        "fiction-psych" => SkillOutput::Psych,
        "fiction-writechapter" => SkillOutput::ChapterProse,
        other if is_chunked_skill(other) => SkillOutput::ChapterRewrite,
        _ => SkillOutput::Unknown,
    }
}

/// Relative output path if this skill writes exactly one static Wiki file.
#[must_use]
pub fn output_rel_path(skill: &str) -> Option<&'static str> {
    match skill_output(skill) {
        SkillOutput::WikiFile(path) => Some(path),
        _ => None,
    }
}

/// Resolve the destination relative to the project root for this run.
#[must_use]
pub fn resolve_output_path(
    project: Option<&ProjectRoot>,
    skill: &str,
    chapter: u32,
) -> Option<String> {
    match skill_output(skill) {
        SkillOutput::None | SkillOutput::Unknown | SkillOutput::Bundle => None,
        SkillOutput::WikiFile(path) => Some(path.to_owned()),
        SkillOutput::CharactersDir => Some("Wiki/Characters".to_owned()),
        SkillOutput::Scene => Some(match project {
            Some(project) => find_scene_files(project, chapter)
                .into_iter()
                .next()
                .and_then(|path| rel_to_project(project, &path))
                .unwrap_or_else(|| default_scene_rel(chapter)),
            None => default_scene_rel(chapter),
        }),
        SkillOutput::Psych => Some(match project {
            Some(project) => find_psych_file(project, chapter)
                .and_then(|path| rel_to_project(project, &path))
                .unwrap_or_else(|| default_psych_rel(chapter)),
            None => default_psych_rel(chapter),
        }),
        SkillOutput::ChapterProse => Some(match project {
            Some(project) => find_chapter_prose(project, chapter)
                .and_then(|path| rel_to_project(project, &path))
                .unwrap_or_else(|| default_chapter_rel(chapter)),
            None => default_chapter_rel(chapter),
        }),
        SkillOutput::ChapterRewrite => project.and_then(|project| {
            find_chapter_prose(project, chapter).and_then(|path| rel_to_project(project, &path))
        }),
    }
}

fn rel_to_project(project: &ProjectRoot, path: &std::path::Path) -> Option<String> {
    path.strip_prefix(project.path())
        .ok()
        .map(|p| p.to_string_lossy().replace('\\', "/"))
}

/// Whether a required skill already has a non-empty output on disk.
#[must_use]
pub fn requirement_satisfied(project: &ProjectRoot, required_skill: &str, chapter: u32) -> bool {
    match skill_output(required_skill) {
        SkillOutput::None | SkillOutput::Unknown | SkillOutput::Bundle => true,
        SkillOutput::WikiFile(rel) => file_has_substance(&project.path().join(rel)),
        SkillOutput::CharactersDir => wiki_markdown_files(&project.wiki().join("Characters"))
            .iter()
            .any(|path| file_has_substance(path)),
        SkillOutput::Scene => find_scene_files(project, chapter)
            .iter()
            .any(|path| file_has_substance(path)),
        SkillOutput::Psych => {
            find_psych_file(project, chapter).is_some_and(|path| file_has_substance(&path))
        }
        SkillOutput::ChapterProse | SkillOutput::ChapterRewrite => {
            find_chapter_prose(project, chapter).is_some_and(|path| file_has_substance(&path))
        }
    }
}

fn file_has_substance(path: &std::path::Path) -> bool {
    std::fs::metadata(path)
        .ok()
        .is_some_and(|meta| meta.is_file() && meta.len() > 40)
}

/// Required skills whose outputs are missing.
#[must_use]
pub fn missing_requirements(
    project: Option<&ProjectRoot>,
    requires: &[String],
    chapter: u32,
) -> Vec<String> {
    let Some(project) = project else {
        return requires.to_vec();
    };
    requires
        .iter()
        .filter(|skill| !requirement_satisfied(project, skill, chapter))
        .cloned()
        .collect()
}

/// Fail if the skill's `requires` are not on disk.
///
/// # Errors
///
/// Returns [`Error::RequiresMissing`] listing the unmet skill ids.
pub fn ensure_requires(
    project: Option<&ProjectRoot>,
    requires: &[String],
    chapter: u32,
) -> Result<(), Error> {
    let missing = missing_requirements(project, requires, chapter);
    if missing.is_empty() {
        Ok(())
    } else {
        Err(Error::RequiresMissing(missing.join(", ")))
    }
}

/// Split a characters preview into `(filename, markdown)` pairs.
#[must_use]
pub fn split_character_preview(text: &str) -> Vec<(String, String)> {
    let docs = split_frontmatter_docs(text);
    let mut out = Vec::with_capacity(docs.len().max(1));
    for (fm, body) in &docs {
        let Some(name) = frontmatter_name(fm) else {
            continue;
        };
        let file = character_filename(&name);
        let mut markdown = String::from("---\n");
        markdown.push_str(fm.trim());
        markdown.push_str("\n---\n");
        if !body.trim().is_empty() {
            markdown.push('\n');
            markdown.push_str(body.trim_start());
        }
        out.push((file, markdown));
    }
    if out.is_empty() {
        vec![("cast.md".to_owned(), text.to_owned())]
    } else {
        out
    }
}

fn split_frontmatter_docs(text: &str) -> Vec<(String, String)> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }
    let parts: Vec<&str> = trimmed.split("\n---\n").collect();
    let mut docs = Vec::new();
    if trimmed.starts_with("---") {
        let rest = trimmed.trim_start_matches('-').trim_start_matches('\n');
        let parts: Vec<&str> = rest.split("\n---\n").collect();
        let mut i = 0;
        while i + 1 < parts.len() {
            docs.push((parts[i].to_owned(), parts[i + 1].to_owned()));
            i += 2;
        }
        if i < parts.len() && !parts[i].trim().is_empty() && docs.is_empty() {
            docs.push((parts[i].to_owned(), String::new()));
        }
        if !docs.is_empty() {
            return docs;
        }
    }
    if parts.len() >= 2 {
        let mut i = 0;
        while i + 1 < parts.len() {
            docs.push((parts[i].to_owned(), parts[i + 1].to_owned()));
            i += 2;
        }
    }
    docs
}

fn frontmatter_name(fm: &str) -> Option<String> {
    for line in fm.lines() {
        let line = line.trim();
        let Some(value) = line.strip_prefix("name:") else {
            continue;
        };
        let value = value.trim().trim_matches('"').trim_matches('\'').trim();
        if !value.is_empty() {
            return Some(value.to_owned());
        }
    }
    None
}

fn character_filename(name: &str) -> String {
    format!("{}.md", slug_filename(name))
}

/// File-name-safe slug for a display name (`Kael Veyra` → `Kael_Veyra`).
pub(crate) fn slug_filename(name: &str) -> String {
    let mut s = String::new();
    for c in name.trim().chars() {
        if c.is_whitespace() {
            s.push('_');
        } else if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
            s.push(c);
        }
    }
    if s.is_empty() {
        "untitled".to_owned()
    } else {
        s
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

    #[test]
    fn writechapter_defaults_to_padded_chapter_file() {
        let path = resolve_output_path(None, "fiction-writechapter", 3);
        assert_eq!(path.as_deref(), Some("Chapters/Chapter-003.md"));
    }

    #[test]
    fn split_two_character_docs() {
        let preview = "---\nname: Mira\nrole: protagonist\n---\n\n# Mira\n\nVoice.\n\n---\nname: Kael Veyra\nrole: antagonist\n---\n\n# Kael\n\nSteel.\n";
        let files = split_character_preview(preview);
        assert_eq!(files.len(), 2);
        assert_eq!(files[0].0, "Mira.md");
        assert!(files[0].1.contains("name: Mira"));
        assert_eq!(files[1].0, "Kael_Veyra.md");
    }
}
