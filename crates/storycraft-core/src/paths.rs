//! On-disk locations for Wiki slots and chapter files.

use std::fs;
use std::path::{Path, PathBuf};

use crate::project::ProjectRoot;

/// Zero-pad a chapter number to two digits (`1` → `"01"`).
#[must_use]
pub fn padded2(n: u32) -> String {
    format!("{n:02}")
}

/// True when `file_name` is a chapter-N artifact (`Chapter_1.md` must not
/// match chapter 10).
#[must_use]
pub fn name_matches_chapter(file_name: &str, n: u32) -> bool {
    let prefixes = ["Chapter_", "Chapter-", "Chapter "];
    let numbers = [n.to_string(), padded2(n), format!("{n:03}")];
    for prefix in prefixes {
        for number in &numbers {
            let needle = format!("{prefix}{number}");
            if let Some(rest) = file_name.strip_prefix(&needle) {
                if rest.is_empty() || !rest.starts_with(|c: char| c.is_ascii_digit()) {
                    return true;
                }
            }
        }
    }
    false
}

/// True when the file looks like a template or README, not canon.
#[must_use]
pub fn is_ignored_wiki_file(file_name: &str) -> bool {
    let stem = file_name
        .rsplit_once('.')
        .map_or(file_name, |(stem, _)| stem);
    let lower = stem.to_ascii_lowercase();
    lower.contains("template") || lower == "readme" || lower == "gitkeep" || lower.starts_with('.')
}

/// Existing chapter prose for `n`, first match in orchestrator order.
#[must_use]
pub fn find_chapter_prose(project: &ProjectRoot, n: u32) -> Option<PathBuf> {
    let dirs = [project.chapters_dir(), project.path().to_path_buf()];
    for dir in dirs {
        if let Some(path) = first_matching_md(&dir, n, false) {
            return Some(path);
        }
    }
    None
}

/// Scene / beat files for chapter `n`.
#[must_use]
pub fn find_scene_files(project: &ProjectRoot, n: u32) -> Vec<PathBuf> {
    let dirs = [
        project.wiki().join("Outline"),
        project.wiki().join("Scenes"),
        project.path().to_path_buf(),
    ];
    let mut out = Vec::new();
    for dir in dirs {
        collect_matching_md(&dir, n, true, &mut out);
    }
    out.sort();
    out.dedup();
    out
}

/// `Wiki/Psych/Chapter_NN_Psych.md` if it exists.
#[must_use]
pub fn find_psych_file(project: &ProjectRoot, n: u32) -> Option<PathBuf> {
    let dir = project.wiki().join("Psych");
    let exact = dir.join(format!("Chapter_{}_Psych.md", padded2(n)));
    if exact.is_file() {
        return Some(exact);
    }
    first_matching_md(&dir, n, false).filter(|path| {
        path.file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.to_ascii_lowercase().contains("psych"))
    })
}

/// Markdown files in `dir` (non-recursive), excluding templates.
#[must_use]
pub fn wiki_markdown_files(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let Ok(entries) = fs::read_dir(dir) else {
        return out;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if path.is_file() && is_markdown(name) && !is_ignored_wiki_file(name) {
            out.push(path);
        }
    }
    out.sort();
    out
}

/// First chapter listed in `outline.md` that has no prose file. `1` if unknown.
#[must_use]
pub fn infer_chapter(project: &ProjectRoot) -> u32 {
    let outline = project.wiki().join("Outline/outline.md");
    let Ok(text) = fs::read_to_string(outline) else {
        return 1;
    };
    let mut chapters = Vec::new();
    for line in text.lines() {
        if let Some(n) = parse_chapter_heading(line) {
            chapters.push(n);
        }
    }
    chapters.sort_unstable();
    chapters.dedup();
    for n in chapters {
        if find_chapter_prose(project, n).is_none() {
            return n;
        }
    }
    1
}

fn parse_chapter_heading(line: &str) -> Option<u32> {
    let line = line.trim();
    let rest = line.strip_prefix("## ")?;
    let rest = rest
        .strip_prefix("Chapter ")
        .or_else(|| rest.strip_prefix("Chapter_"))
        .or_else(|| rest.strip_prefix("Chapter-"))?;
    let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() {
        None
    } else {
        digits.parse().ok()
    }
}

fn is_markdown(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    lower.ends_with(".md") || lower.ends_with(".markdown")
}

fn first_matching_md(dir: &Path, n: u32, scenes_only: bool) -> Option<PathBuf> {
    let mut matches = Vec::new();
    collect_matching_md(dir, n, scenes_only, &mut matches);
    matches.into_iter().next()
}

fn collect_matching_md(dir: &Path, n: u32, scenes_only: bool, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if !path.is_file() || !is_markdown(name) || is_ignored_wiki_file(name) {
            continue;
        }
        if !name_matches_chapter(name, n) {
            continue;
        }
        if scenes_only && !name.to_ascii_lowercase().contains("scene") {
            continue;
        }
        out.push(path);
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;

    #[test]
    fn chapter_1_does_not_match_chapter_10() {
        assert!(name_matches_chapter("Chapter_1.md", 1));
        assert!(!name_matches_chapter("Chapter_10.md", 1));
        assert!(name_matches_chapter("Chapter_10.md", 10));
        assert!(name_matches_chapter("Chapter-003.md", 3));
        assert!(name_matches_chapter("Chapter 2_Scene.md", 2));
    }

    #[test]
    fn parses_outline_headings() {
        assert_eq!(parse_chapter_heading("## Chapter 01: The Shadow"), Some(1));
        assert_eq!(parse_chapter_heading("## Chapter_12_Scene"), Some(12));
        assert_eq!(parse_chapter_heading("# Not a chapter"), None);
    }
}
