//! Status board. Always derived from files on disk, never from memory.

use std::fmt;
use std::fs;
use std::path::Path;

use crate::catalog::frontmatter_scalar;
use crate::paths::{
    find_chapter_prose, find_psych_file, find_scene_files, is_ignored_wiki_file,
    wiki_markdown_files,
};
use crate::project::ProjectRoot;
use crate::spine::next_action;
use crate::{Error, Mode};

/// Spine slot on the orchestrator board.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Slot {
    /// `Wiki/Style/genre.md`
    Genre = 0,
    /// `Wiki/Style/audience.md`
    Audience,
    /// `Wiki/Story/theme.md`
    Theme,
    /// `Wiki/Story/synopsis.md`
    Synopsis,
    /// `Wiki/Style/style_guide.md`
    Style,
    /// `Wiki/Characters/*.md`
    Characters,
    /// Files under Locations / Organizations / Systems / Events
    World,
    /// `Wiki/Outline/outline.md`
    Outline,
    /// Scene / beat files for the current chapter
    Scenes,
    /// `Wiki/Style/voice_prompt.md`
    Voice,
    /// `Wiki/Psych/` file for the current chapter
    Psych,
    /// Chapter prose for the current chapter
    Chapters,
}

impl Slot {
    /// Board order from `project-layout.md`.
    pub const ALL: [Self; 12] = [
        Self::Genre,
        Self::Audience,
        Self::Theme,
        Self::Synopsis,
        Self::Style,
        Self::Characters,
        Self::World,
        Self::Outline,
        Self::Scenes,
        Self::Voice,
        Self::Psych,
        Self::Chapters,
    ];

    /// Lowercase slot name used on the board.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Genre => "genre",
            Self::Audience => "audience",
            Self::Theme => "theme",
            Self::Synopsis => "synopsis",
            Self::Style => "style",
            Self::Characters => "characters",
            Self::World => "world",
            Self::Outline => "outline",
            Self::Scenes => "scenes",
            Self::Voice => "voice",
            Self::Psych => "psych",
            Self::Chapters => "chapters",
        }
    }

    const fn index(self) -> usize {
        self as usize
    }
}

/// Presence of a slot's required file(s).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SlotState {
    /// File exists and has substance.
    Yes,
    /// Folder or file exists but is empty, template-only, or the wrong chapter.
    Partial,
    /// Missing.
    No,
}

impl SlotState {
    /// Lowercase board token.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Yes => "yes",
            Self::Partial => "partial",
            Self::No => "no",
        }
    }

    /// Whether this slot is complete enough to skip on resume.
    #[must_use]
    pub const fn is_complete(self) -> bool {
        matches!(self, Self::Yes)
    }
}

impl fmt::Display for SlotState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Next skill the host should run, plus why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NextAction {
    /// Skill folder name. `None` when the host must ask (spine done, edit, …).
    pub skill: Option<String>,
    /// One-line reason shown on the board.
    pub why: String,
    /// Slots that are not `yes` and block that skill.
    pub missing: Vec<Slot>,
}

/// Disk snapshot of a book project.
#[derive(Debug, Clone, PartialEq, Eq)]
#[must_use]
pub struct StatusBoard {
    project: Option<ProjectRoot>,
    mode: Mode,
    chapter: u32,
    slots: [SlotState; 12],
    next: NextAction,
}

impl StatusBoard {
    /// Inspect `project` (or the empty board) for `mode` and chapter `n`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidChapter`] when `chapter` is 0.
    pub fn inspect(project: Option<&ProjectRoot>, mode: Mode, chapter: u32) -> Result<Self, Error> {
        if chapter == 0 {
            return Err(Error::InvalidChapter);
        }
        let mut slots = [SlotState::No; 12];
        if let Some(project) = project {
            for slot in Slot::ALL {
                slots[slot.index()] = inspect_slot(project, slot, chapter);
            }
        }
        let next = next_action(project, mode, chapter, &slots);
        Ok(Self {
            project: project.cloned(),
            mode,
            chapter,
            slots,
            next,
        })
    }

    /// Project root, if a Wiki was found.
    #[must_use]
    pub fn project(&self) -> Option<&ProjectRoot> {
        self.project.as_ref()
    }

    /// Mode this board was built for.
    #[must_use]
    pub fn mode(&self) -> Mode {
        self.mode
    }

    /// Chapter used for scenes / psych / chapters slots.
    #[must_use]
    pub fn chapter(&self) -> u32 {
        self.chapter
    }

    /// State of one slot.
    #[must_use]
    pub fn slot(&self, slot: Slot) -> SlotState {
        self.slots[slot.index()]
    }

    /// Recommended next skill.
    #[must_use]
    pub fn next(&self) -> &NextAction {
        &self.next
    }

    /// JSON-friendly copy for the Tauri/Android shell.
    #[must_use]
    pub fn snapshot(&self) -> StatusSnapshot {
        StatusSnapshot {
            project: self
                .project
                .as_ref()
                .map(|project| project.path().display().to_string()),
            title: self.project.as_ref().and_then(ProjectRoot::title),
            mode: self.mode.as_str().to_owned(),
            chapter: self.chapter,
            slots: Slot::ALL
                .iter()
                .map(|slot| SlotSnapshot {
                    name: slot.as_str().to_owned(),
                    state: self.slot(*slot).as_str().to_owned(),
                })
                .collect(),
            next_skill: self.next.skill.clone(),
            next_why: self.next.why.clone(),
            missing: self
                .next
                .missing
                .iter()
                .map(|slot| slot.as_str().to_owned())
                .collect(),
        }
    }
}

/// IPC view of one spine slot.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SlotSnapshot {
    /// Slot name (`genre`, `outline`, …).
    pub name: String,
    /// `yes` / `partial` / `no`.
    pub state: String,
}

/// IPC view of the status board.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct StatusSnapshot {
    /// Absolute project root, if a Wiki was found.
    pub project: Option<String>,
    /// Working title from genre/synopsis, if any.
    pub title: Option<String>,
    /// Orchestrator mode.
    pub mode: String,
    /// Chapter used for scenes / psych / chapters.
    pub chapter: u32,
    /// Spine slots in board order.
    pub slots: Vec<SlotSnapshot>,
    /// Next skill folder name.
    pub next_skill: Option<String>,
    /// Why that skill is next.
    pub next_why: String,
    /// Incomplete slots blocking the next skill.
    pub missing: Vec<String>,
}

impl fmt::Display for StatusBoard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.project {
            Some(project) => writeln!(f, "Project: {}", project.path().display())?,
            None => writeln!(f, "Project: (none)")?,
        }
        writeln!(f, "Mode: {}", self.mode)?;
        writeln!(f, "Chapter: {}", self.chapter)?;
        write!(f, "Have:")?;
        for (i, slot) in Slot::ALL.iter().enumerate() {
            if i == 0 {
                write!(f, " {} {}", slot.as_str(), self.slot(*slot))?;
            } else {
                write!(f, " / {} {}", slot.as_str(), self.slot(*slot))?;
            }
        }
        writeln!(f)?;
        if self.next.missing.is_empty() {
            writeln!(f, "Missing required for next skill: —")?;
        } else {
            write!(f, "Missing required for next skill:")?;
            for (i, slot) in self.next.missing.iter().enumerate() {
                if i == 0 {
                    write!(f, " {}", slot.as_str())?;
                } else {
                    write!(f, ", {}", slot.as_str())?;
                }
            }
            writeln!(f)?;
        }
        match &self.next.skill {
            Some(skill) => write!(f, "Next: {skill} — {}", self.next.why)?,
            None => write!(f, "Next: — — {}", self.next.why)?,
        }
        Ok(())
    }
}

fn inspect_slot(project: &ProjectRoot, slot: Slot, chapter: u32) -> SlotState {
    let wiki = project.wiki();
    match slot {
        Slot::Genre => genre_state(&wiki.join("Style/genre.md")),
        Slot::Audience => file_state(&wiki.join("Style/audience.md")),
        Slot::Theme => file_state(&wiki.join("Story/theme.md")),
        Slot::Synopsis => file_state(&wiki.join("Story/synopsis.md")),
        Slot::Style => file_state(&wiki.join("Style/style_guide.md")),
        Slot::Voice => file_state(&wiki.join("Style/voice_prompt.md")),
        Slot::Outline => file_state(&wiki.join("Outline/outline.md")),
        Slot::Characters => dir_markdown_state(&wiki.join("Characters")),
        Slot::World => world_state(&wiki),
        Slot::Scenes => files_state(&find_scene_files(project, chapter)),
        Slot::Psych => match find_psych_file(project, chapter) {
            Some(path) => file_state(&path),
            None => SlotState::No,
        },
        Slot::Chapters => match find_chapter_prose(project, chapter) {
            Some(path) => file_state(&path),
            None => SlotState::No,
        },
    }
}

fn world_state(wiki: &Path) -> SlotState {
    let dirs = ["Locations", "Organizations", "Systems", "Events"];
    let mut saw_dir = false;
    let mut states = Vec::new();
    for name in dirs {
        let dir = wiki.join(name);
        if dir.is_dir() {
            saw_dir = true;
            let files = wiki_markdown_files(&dir);
            for file in files {
                states.push(file_state(&file));
            }
        }
    }
    if states.contains(&SlotState::Yes) {
        SlotState::Yes
    } else if saw_dir || !states.is_empty() {
        SlotState::Partial
    } else {
        SlotState::No
    }
}

fn dir_markdown_state(dir: &Path) -> SlotState {
    if !dir.is_dir() {
        return SlotState::No;
    }
    let files = wiki_markdown_files(dir);
    if files.is_empty() {
        return SlotState::Partial;
    }
    if files.iter().any(|path| file_state(path) == SlotState::Yes) {
        SlotState::Yes
    } else {
        SlotState::Partial
    }
}

fn files_state(paths: &[std::path::PathBuf]) -> SlotState {
    if paths.is_empty() {
        return SlotState::No;
    }
    if paths.iter().any(|path| file_state(path) == SlotState::Yes) {
        SlotState::Yes
    } else {
        SlotState::Partial
    }
}

fn genre_state(path: &Path) -> SlotState {
    match file_state(path) {
        SlotState::Yes => {
            let text = read_prefix(path);
            if has_genre_signal(&text) {
                SlotState::Yes
            } else {
                SlotState::Partial
            }
        }
        other => other,
    }
}

fn file_state(path: &Path) -> SlotState {
    if !path.is_file() {
        return SlotState::No;
    }
    if let Some(name) = path.file_name().and_then(|n| n.to_str())
        && is_ignored_wiki_file(name)
    {
        return SlotState::No;
    }
    let text = read_prefix(path);
    if is_stub(&text) {
        SlotState::Partial
    } else {
        SlotState::Yes
    }
}

fn read_prefix(path: &Path) -> String {
    let Ok(bytes) = fs::read(path) else {
        return String::new();
    };
    let slice = if bytes.len() > 4096 {
        &bytes[..4096]
    } else {
        &bytes
    };
    String::from_utf8_lossy(slice).into_owned()
}

fn is_stub(text: &str) -> bool {
    let compact: String = text
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '-')
        .collect();
    compact.len() < 40
}

fn has_genre_signal(text: &str) -> bool {
    if let Some(value) = frontmatter_scalar(text, "genre")
        && !value.is_empty()
    {
        return true;
    }
    for line in text.lines() {
        let trimmed = line.trim();
        let lower = trimmed.to_ascii_lowercase();
        if lower.starts_with("genre:") {
            return true;
        }
        let heading = trimmed.trim_start_matches('#').trim();
        if heading.eq_ignore_ascii_case("genre") {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;

    #[test]
    fn empty_text_is_stub() {
        assert!(is_stub(""));
        assert!(is_stub("---\n# Style\n"));
        assert!(!is_stub(
            "---\nworking_title: Night Market\ngenre: Fantasy\nsubgenre: Low\n---\n\n# Genre\n"
        ));
    }

    #[test]
    fn genre_signal_from_frontmatter() {
        assert!(has_genre_signal("---\ngenre: Fantasy\n---\n"));
        assert!(has_genre_signal("# Genre\n\nSome text"));
        assert!(!has_genre_signal("# Audience\n"));
    }

    #[test]
    fn snapshot_lists_every_slot() {
        let board = StatusBoard::inspect(None, Mode::Resume, 1).unwrap();
        let snap = board.snapshot();
        assert_eq!(snap.slots.len(), 12);
        assert_eq!(snap.next_skill.as_deref(), Some("fiction-genre"));
        assert!(snap.project.is_none());
    }
}
