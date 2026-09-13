//! Wiki files packed with a skill run. Priority list, not the whole book.

use std::path::PathBuf;

use crate::chunk::is_chunked_skill;
use crate::pack::{PackFile, PackKind};
use crate::paths::{
    find_chapter_prose, find_psych_file, find_scene_files, padded2, wiki_markdown_files,
};
use crate::project::ProjectRoot;

const PREV_CHAPTER_TAIL_WORDS: usize = 1500;

/// Wiki / chapter files to pack for `skill` at `chapter`, in drop-last order.
#[must_use]
pub fn wiki_pack_files(project: &ProjectRoot, skill: &str, chapter: u32) -> Vec<PackFile> {
    let mut files = Vec::new();
    match skill {
        "fiction-audience" => push_rel(project, &mut files, "Wiki/Style/genre.md"),
        "fiction-theme" => {
            push_rel(project, &mut files, "Wiki/Style/genre.md");
            push_rel(project, &mut files, "Wiki/Style/audience.md");
        }
        "fiction-synopsis" => {
            push_rel(project, &mut files, "Wiki/Style/genre.md");
            push_rel(project, &mut files, "Wiki/Style/audience.md");
            push_rel(project, &mut files, "Wiki/Story/theme.md");
        }
        "fiction-style" => {
            push_rel(project, &mut files, "Wiki/Style/genre.md");
            push_rel(project, &mut files, "Wiki/Style/audience.md");
            push_rel(project, &mut files, "Wiki/Story/theme.md");
            push_rel(project, &mut files, "Wiki/Story/synopsis.md");
        }
        "fiction-characters" => {
            push_rel(project, &mut files, "Wiki/Story/synopsis.md");
            push_rel(project, &mut files, "Wiki/Style/genre.md");
            push_rel(project, &mut files, "Wiki/Style/audience.md");
            push_rel(project, &mut files, "Wiki/Style/style_guide.md");
        }
        "fiction-voiceprompt" => {
            push_rel(project, &mut files, "Wiki/Style/style_guide.md");
            push_character_dir(project, &mut files);
        }
        "fiction-outline" => {
            push_rel(project, &mut files, "Wiki/Story/synopsis.md");
            push_rel(project, &mut files, "Wiki/Story/theme.md");
            push_character_dir(project, &mut files);
        }
        "fiction-scenes" => {
            push_rel(project, &mut files, "Wiki/Outline/outline.md");
            push_rel(project, &mut files, "Wiki/Style/style_guide.md");
            push_character_dir(project, &mut files);
        }
        "fiction-psych" => {
            push_rel(project, &mut files, "Wiki/Outline/outline.md");
            push_scene(project, &mut files, chapter);
            push_rel(project, &mut files, "Wiki/Style/style_guide.md");
            push_rel(project, &mut files, "Wiki/Story/theme.md");
            push_character_dir(project, &mut files);
        }
        "fiction-writechapter" => push_writechapter(project, &mut files, chapter),
        // The bible is the whole book in one file, so it reads the whole canon
        // that already exists and carries it across verbatim.
        "fiction-storybible" => {
            for rel in [
                "Wiki/Style/genre.md",
                "Wiki/Style/audience.md",
                "Wiki/Story/theme.md",
                "Wiki/Story/synopsis.md",
                "Wiki/Style/style_guide.md",
                "Wiki/Style/voice_prompt.md",
                "Wiki/Outline/outline.md",
            ] {
                push_rel(project, &mut files, rel);
            }
            push_character_dir(project, &mut files);
            for dir in ["Locations", "Organizations", "Systems", "Events"] {
                for path in wiki_markdown_files(&project.wiki().join(dir)) {
                    push_abs(project, &mut files, path, PackKind::Wiki);
                }
            }
        }
        other if is_chunked_skill(other) => {
            push_rel(project, &mut files, "Wiki/Style/style_guide.md");
            push_rel(project, &mut files, "Wiki/Style/voice_prompt.md");
        }
        "burstiness-check" | "coldread" | "pangram" => {}
        _ => {}
    }
    files
}

fn push_writechapter(project: &ProjectRoot, files: &mut Vec<PackFile>, chapter: u32) {
    push_scene(project, files, chapter);
    push_rel(project, files, "Wiki/Style/voice_prompt.md");
    push_rel(project, files, "Wiki/Style/style_guide.md");
    push_rel(project, files, "Wiki/Story/synopsis.md");
    if chapter > 1
        && let Some(prev) = find_chapter_prose(project, chapter - 1)
    {
        push_abs(project, files, prev, PackKind::WikiTail);
    }
    if let Some(psych) = find_psych_file(project, chapter) {
        push_abs(project, files, psych, PackKind::Wiki);
    }
    push_mentioned_characters(project, files, chapter);
}

fn push_scene(project: &ProjectRoot, files: &mut Vec<PackFile>, chapter: u32) {
    for path in find_scene_files(project, chapter) {
        push_abs(project, files, path, PackKind::Wiki);
    }
}

fn push_character_dir(project: &ProjectRoot, files: &mut Vec<PackFile>) {
    for path in wiki_markdown_files(&project.wiki().join("Characters")) {
        push_abs(project, files, path, PackKind::Wiki);
    }
}

fn push_mentioned_characters(project: &ProjectRoot, files: &mut Vec<PackFile>, chapter: u32) {
    let scene_blob: String = find_scene_files(project, chapter)
        .into_iter()
        .filter_map(|path| std::fs::read_to_string(path).ok())
        .collect();
    if scene_blob.is_empty() {
        return;
    }
    let lower = scene_blob.to_ascii_lowercase();
    for path in wiki_markdown_files(&project.wiki().join("Characters")) {
        let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        let underscored = stem.to_ascii_lowercase();
        let spaced = underscored.replace('_', " ");
        if lower.contains(&underscored) || lower.contains(&spaced) {
            push_abs(project, files, path, PackKind::Wiki);
        }
    }
}

fn push_rel(project: &ProjectRoot, files: &mut Vec<PackFile>, rel: &str) {
    let path = project.path().join(rel);
    if path.is_file() {
        files.push(PackFile {
            kind: PackKind::Wiki,
            path,
            relative: rel.to_owned(),
        });
    }
}

fn push_abs(project: &ProjectRoot, files: &mut Vec<PackFile>, path: PathBuf, kind: PackKind) {
    if !path.is_file() {
        return;
    }
    let relative = path
        .strip_prefix(project.path())
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|_| path.display().to_string());
    if files.iter().any(|file| file.relative == relative) {
        return;
    }
    files.push(PackFile {
        kind,
        path,
        relative,
    });
}

/// Last `n` whitespace-separated words of `text`, or the whole text.
#[must_use]
pub fn last_n_words(text: &str, n: usize) -> String {
    let words: Vec<&str> = text.split_whitespace().collect();
    if words.len() <= n {
        return text.to_owned();
    }
    words[words.len() - n..].join(" ")
}

/// Word budget used for the previous-chapter tail.
#[must_use]
pub const fn prev_chapter_tail_words() -> usize {
    PREV_CHAPTER_TAIL_WORDS
}

/// Scene filename default when the project has no convention yet.
#[must_use]
pub fn default_scene_rel(chapter: u32) -> String {
    format!("Wiki/Outline/Chapter_{}_Scene.md", padded2(chapter))
}

/// Psych filename default.
#[must_use]
pub fn default_psych_rel(chapter: u32) -> String {
    format!("Wiki/Psych/Chapter_{}_Psych.md", padded2(chapter))
}

/// Chapter prose default (`Chapters/Chapter-NNN.md`).
#[must_use]
pub fn default_chapter_rel(chapter: u32) -> String {
    format!("Chapters/Chapter-{:03}.md", chapter)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;
    use crate::project::discover_one;
    use std::path::PathBuf;

    fn fixture(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(name)
    }

    #[test]
    fn writechapter_pack_includes_scene_voice_style_not_the_whole_wiki() {
        let project = discover_one(&fixture("planning-done")).unwrap();
        let files = wiki_pack_files(&project, "fiction-writechapter", 1);
        let rels: Vec<&str> = files.iter().map(|f| f.relative.as_str()).collect();
        assert!(rels.iter().any(|r| r.contains("Scene")));
        assert!(rels.contains(&"Wiki/Style/voice_prompt.md"));
        assert!(rels.contains(&"Wiki/Style/style_guide.md"));
        assert!(rels.contains(&"Wiki/Story/synopsis.md"));
        assert!(rels.iter().any(|r| r.contains("Psych")));
        assert!(!rels.iter().any(|r| r.contains("genre.md")));
    }

    #[test]
    fn last_n_words_keeps_the_tail() {
        let text = "one two three four five";
        assert_eq!(last_n_words(text, 2), "four five");
        assert_eq!(last_n_words(text, 9), text);
    }
}
