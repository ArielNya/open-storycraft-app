//! Wiki discovery. A project root is the directory that contains `Wiki/`.

use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use crate::Error;

/// Directory that contains `Wiki/`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProjectRoot {
    path: PathBuf,
}

impl ProjectRoot {
    /// Treat `path` as a project root without checking that `Wiki/` exists.
    #[must_use]
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    /// Folder that contains `Wiki/`.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// `Wiki/` directory.
    #[must_use]
    pub fn wiki(&self) -> PathBuf {
        self.path.join("Wiki")
    }

    /// `Chapters/` directory.
    #[must_use]
    pub fn chapters_dir(&self) -> PathBuf {
        self.path.join("Chapters")
    }

    /// `.storycraft/` directory (jobs, not canon).
    #[must_use]
    pub fn storycraft_dir(&self) -> PathBuf {
        self.path.join(".storycraft")
    }

    /// Working title from `genre.md` or `synopsis.md` frontmatter, if present.
    #[must_use]
    pub fn title(&self) -> Option<String> {
        for relative in ["Wiki/Style/genre.md", "Wiki/Story/synopsis.md"] {
            let path = self.path.join(relative);
            let Ok(text) = std::fs::read_to_string(&path) else {
                continue;
            };
            if let Some(title) = crate::catalog::frontmatter_scalar(&text, "working_title") {
                return Some(title);
            }
            if let Some(title) = crate::catalog::frontmatter_scalar(&text, "title") {
                return Some(title);
            }
        }
        self.path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
    }
}

const SKIP_DIR_NAMES: &[&str] = &[
    "target",
    ".git",
    "node_modules",
    ".idea",
    ".vscode",
    ".storycraft",
];
const MAX_WALK_DEPTH: usize = 8;

/// Find every project under `start`.
///
/// If `start` itself contains `Wiki/`, or *is* a `Wiki/` folder, that is the
/// only result. Otherwise the tree is walked (skipping build/VCS directories).
///
/// # Errors
///
/// Returns [`Error::Io`] if `start` cannot be read.
pub fn discover(start: &Path) -> Result<Vec<ProjectRoot>, Error> {
    if !start.exists() {
        return Err(Error::NoProject(start.to_path_buf()));
    }

    if is_wiki_dir(start) {
        let parent = start.parent().unwrap_or(start);
        return Ok(vec![ProjectRoot::new(parent.to_path_buf())]);
    }
    if start.join("Wiki").is_dir() {
        return Ok(vec![ProjectRoot::new(start.to_path_buf())]);
    }

    let mut found = Vec::new();
    let walker = WalkDir::new(start)
        .follow_links(false)
        .max_depth(MAX_WALK_DEPTH)
        .into_iter()
        .filter_entry(|entry| {
            let name = entry.file_name().to_string_lossy();
            !SKIP_DIR_NAMES.contains(&name.as_ref())
        });

    for entry in walker {
        let entry = entry.map_err(|err| {
            let path = err.path().unwrap_or(start).to_path_buf();
            match err.io_error() {
                Some(io) => Error::io(&path, io::clone_io(io)),
                None => Error::io(&path, std::io::Error::other(err.to_string())),
            }
        })?;
        if entry.file_type().is_dir() && is_wiki_dir(entry.path()) {
            if let Some(parent) = entry.path().parent() {
                found.push(ProjectRoot::new(parent.to_path_buf()));
            }
        }
    }

    found.sort_by(|a, b| a.path.cmp(&b.path));
    found.dedup();
    Ok(found)
}

/// Discover exactly one project.
///
/// # Errors
///
/// Returns [`Error::NoProject`] if none, [`Error::MultipleProjects`] if several,
/// or [`Error::Io`] on walk failures.
pub fn discover_one(start: &Path) -> Result<ProjectRoot, Error> {
    let mut found = discover(start)?;
    match found.len() {
        0 => Err(Error::NoProject(start.to_path_buf())),
        1 => Ok(found.remove(0)),
        _ => Err(Error::MultipleProjects { projects: found }),
    }
}

fn is_wiki_dir(path: &Path) -> bool {
    path.file_name().is_some_and(|name| name == "Wiki") && path.is_dir()
}

mod io {
    pub fn clone_io(err: &std::io::Error) -> std::io::Error {
        std::io::Error::new(err.kind(), err.to_string())
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;

    fn fixtures() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
    }

    #[test]
    fn discovers_direct_wiki_parent() {
        let root = discover_one(&fixtures().join("empty-wiki")).expect("fixture");
        assert_eq!(root.path().file_name().unwrap(), "empty-wiki");
    }

    #[test]
    fn wiki_dir_itself_resolves_to_parent() {
        let root = discover_one(&fixtures().join("empty-wiki/Wiki")).expect("fixture");
        assert_eq!(root.path().file_name().unwrap(), "empty-wiki");
    }

    #[test]
    fn multiple_wikis_are_listed_not_picked() {
        let err = discover_one(&fixtures().join("multi-root")).expect_err("two books");
        match err {
            Error::MultipleProjects { projects } => {
                assert_eq!(projects.len(), 2);
            }
            other => panic!("expected MultipleProjects, got {other}"),
        }
    }
}
