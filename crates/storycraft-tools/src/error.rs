//! Errors from local generators.

use std::io;
use std::path::{Path, PathBuf};

use thiserror::Error;

/// Failure from a local tool (Markov, burstiness).
#[derive(Debug, Error)]
pub enum Error {
    /// Filesystem failure.
    #[error("io error at {path}")]
    Io {
        /// Path being read.
        path: PathBuf,
        /// Underlying IO error.
        #[source]
        source: io::Error,
    },
    /// Name/town list had no usable lines.
    #[error("name list is empty")]
    EmptyList,
    /// Markov order is outside 1..=3.
    #[error("markov order must be 1, 2, or 3")]
    InvalidOrder,
    /// Generator could not produce any unique names.
    #[error("could not generate any names with these parameters")]
    Exhausted,
}

impl Error {
    pub(crate) fn io(path: impl AsRef<Path>, source: io::Error) -> Self {
        Self::Io {
            path: path.as_ref().to_path_buf(),
            source,
        }
    }
}
