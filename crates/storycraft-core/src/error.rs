//! Typed errors for the core library.

use std::fmt::Write;
use std::io;
use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::project::ProjectRoot;

/// Failure from project discovery, catalog loading, or packing.
#[derive(Debug, Error)]
pub enum Error {
    /// Filesystem failure.
    #[error("io error at {path}")]
    Io {
        /// Path being read or walked.
        path: PathBuf,
        /// Underlying IO error.
        #[source]
        source: io::Error,
    },
    /// `start` exists but contains no `Wiki/` folder.
    #[error("no Wiki folder found under {0}")]
    NoProject(PathBuf),
    /// More than one book was found; the host must ask which one.
    #[error("multiple Wiki folders found; pick one project")]
    MultipleProjects {
        /// Candidate project roots (the folder that contains `Wiki/`).
        projects: Vec<ProjectRoot>,
    },
    /// Unrecognized orchestrator mode string.
    #[error("unknown mode: {0}")]
    UnknownMode(String),
    /// Chapter numbers are 1-based.
    #[error("chapter number must be at least 1")]
    InvalidChapter,
    /// Named skill is not in the catalog.
    #[error("skill not found: {0}")]
    SkillNotFound(String),
    /// `SKILL.md` could not be parsed as a manifest.
    #[error("invalid skill manifest in {path}")]
    InvalidManifest {
        /// Path of the `SKILL.md` that failed.
        path: PathBuf,
        /// Parse detail.
        detail: String,
    },
    /// Skill listed `requires` that are missing on disk.
    #[error("missing required skills: {0}")]
    RequiresMissing(String),
    /// Job JSON was not found.
    #[error("job not found: {0}")]
    JobNotFound(String),
    /// Job is in the wrong status for this action.
    #[error("job {id} is {status}, expected {expected}")]
    JobInvalidState {
        /// Job id.
        id: String,
        /// Actual status.
        status: String,
        /// Required status.
        expected: String,
    },
    /// Preview failed the skill's basic shape check.
    #[error("preview failed validation: {0}")]
    InvalidPreview(String),
    /// Job JSON could not be parsed.
    #[error("invalid job file {path}")]
    InvalidJob {
        /// Path of the job JSON.
        path: PathBuf,
        /// Parse detail.
        detail: String,
    },
}

impl Error {
    pub(crate) fn io(path: impl AsRef<Path>, source: io::Error) -> Self {
        Self::Io {
            path: path.as_ref().to_path_buf(),
            source,
        }
    }
}

/// Formats discovered project paths for CLI output.
#[must_use]
pub fn format_project_list(projects: &[ProjectRoot]) -> String {
    let mut out = String::new();
    for project in projects {
        if !out.is_empty() {
            out.push('\n');
        }
        let _ = write!(out, "  {}", project.path().display());
    }
    out
}
