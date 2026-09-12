//! Budgeted context pack. Skill + linked refs, with a hard char budget.

use std::fmt::Write;
use std::path::PathBuf;

use sha2::{Digest, Sha256};

use crate::Error;
use crate::catalog::SkillManifest;
use crate::wiki::{last_n_words, prev_chapter_tail_words};

/// What kind of file landed in a pack.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PackKind {
    /// The skill's `SKILL.md`.
    Skill,
    /// A file the skill text actually links (`references/`, `assets/`, `data/`).
    Reference,
    /// A Wiki or chapter file selected for this skill.
    Wiki,
    /// Previous-chapter body, truncated to the last 1.5k words.
    WikiTail,
}

/// One file included in a pack.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackFile {
    /// Why this file was included.
    pub kind: PackKind,
    /// Absolute path on disk.
    pub path: PathBuf,
    /// Path relative to the skill directory (or `SKILL.md`).
    pub relative: String,
}

/// Files that would be sent for one skill. Wiki files are not dumped here.
#[derive(Debug, Clone, PartialEq, Eq)]
#[must_use]
pub struct ContextPack {
    /// Skill id.
    pub skill: String,
    /// Included files, skill first.
    pub files: Vec<PackFile>,
}

/// Pack `SKILL.md` plus linked refs that exist on disk.
///
/// Linked files that are missing are skipped. The whole `references/`
/// directory is never included — only paths the skill text links.
pub fn pack_skill(manifest: &SkillManifest) -> ContextPack {
    let mut files = Vec::with_capacity(manifest.reference_files.len() + 1);
    files.push(PackFile {
        kind: PackKind::Skill,
        path: manifest.skill_md(),
        relative: "SKILL.md".to_owned(),
    });
    for relative in &manifest.reference_files {
        let path = manifest.skill_dir.join(relative);
        if path.is_file() {
            files.push(PackFile {
                kind: PackKind::Reference,
                path,
                relative: relative.clone(),
            });
        }
    }
    ContextPack {
        skill: manifest.name.clone(),
        files,
    }
}

/// Default per-file cap when materializing a pack.
pub const DEFAULT_PER_FILE_CHARS: usize = 12_000;
/// Default total cap (skill + refs) when materializing a pack.
pub const DEFAULT_TOTAL_CHARS: usize = 48_000;

/// File contents that will go into the LLM prompt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackedText {
    /// Why this file was included.
    pub kind: PackKind,
    /// Path relative to the skill directory.
    pub relative: String,
    /// Truncated body.
    pub text: String,
    /// Whether the body was cut to fit the budget.
    pub truncated: bool,
}

/// Materialized pack plus a stable hash for job retries.
#[derive(Debug, Clone, PartialEq, Eq)]
#[must_use]
pub struct PackedContent {
    /// Skill id.
    pub skill: String,
    /// Files in pack order.
    pub files: Vec<PackedText>,
    /// SHA-256 hex of relative paths + contents.
    pub hash: String,
}

impl ContextPack {
    /// Count of linked reference files (excludes `SKILL.md`).
    #[must_use]
    pub fn reference_count(&self) -> usize {
        self.files
            .iter()
            .filter(|file| file.kind == PackKind::Reference)
            .count()
    }

    /// Read packed files from disk, applying per-file and total char caps.
    ///
    /// Drop trailing reference files first if the total budget is exceeded.
    /// `SKILL.md` is kept even when truncated.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Io`] if a listed file cannot be read.
    pub fn materialize(
        &self,
        per_file_chars: usize,
        total_chars: usize,
    ) -> Result<PackedContent, Error> {
        let mut files = Vec::with_capacity(self.files.len());
        let mut used = 0usize;
        for file in &self.files {
            let raw =
                std::fs::read_to_string(&file.path).map_err(|err| Error::io(&file.path, err))?;
            let raw = if file.kind == PackKind::WikiTail {
                last_n_words(&raw, prev_chapter_tail_words())
            } else {
                raw
            };
            let (text, truncated) = truncate_chars(&raw, per_file_chars);
            if file.kind != PackKind::Skill && used.saturating_add(text.len()) > total_chars {
                break;
            }
            used = used.saturating_add(text.len());
            files.push(PackedText {
                kind: file.kind,
                relative: file.relative.clone(),
                text,
                truncated,
            });
        }
        if used > total_chars {
            shrink_to_total(&mut files, total_chars);
        }
        let hash = hash_packed(&self.skill, &files);
        Ok(PackedContent {
            skill: self.skill.clone(),
            files,
            hash,
        })
    }
}

fn truncate_chars(text: &str, max: usize) -> (String, bool) {
    if text.len() <= max {
        return (text.to_owned(), false);
    }
    let mut end = max.min(text.len());
    while end > 0 && !text.is_char_boundary(end) {
        end -= 1;
    }
    let mut cut = text[..end].to_owned();
    cut.push_str("\n[truncated]\n");
    (cut, true)
}

fn shrink_to_total(files: &mut Vec<PackedText>, total_chars: usize) {
    while files.len() > 1 {
        let used: usize = files.iter().map(|file| file.text.len()).sum();
        if used <= total_chars {
            return;
        }
        files.pop();
    }
    if let Some(first) = files.first_mut() {
        let (text, truncated) = truncate_chars(&first.text, total_chars);
        first.text = text;
        first.truncated = first.truncated || truncated;
    }
}

fn hash_packed(skill: &str, files: &[PackedText]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(skill.as_bytes());
    for file in files {
        hasher.update(file.relative.as_bytes());
        hasher.update(file.text.as_bytes());
    }
    let digest = hasher.finalize();
    let mut hex = String::with_capacity(digest.len() * 2);
    for byte in digest {
        let _ = write!(hex, "{byte:02x}");
    }
    hex
}
