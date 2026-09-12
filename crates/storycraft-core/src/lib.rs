//! Open Storycraft core: Wiki discovery, status board, skill catalog, packing, jobs.
//!
//! Heavy work stays here so the Tauri/Android shell can stay thin. This crate
//! never talks to a network.
//!
//! ```
//! use storycraft_core::Mode;
//! let mode: Mode = "resume".parse()?;
//! assert_eq!(mode, Mode::Resume);
//! # Ok::<(), storycraft_core::Error>(())
//! ```

#![deny(clippy::correctness)]

mod catalog;
mod error;
mod job;
mod mode;
mod output;
mod pack;
mod paths;
mod project;
mod prompt;
mod spine;
mod status;
mod validate;

pub use catalog::{Catalog, SkillManifest, find_skills_dir};
pub use error::{Error, format_project_list};
pub use job::{Job, JobStatus, JobStore, NewJob};
pub use mode::Mode;
pub use output::{SkillOutput, ensure_requires, output_rel_path, skill_output};
pub use pack::{
    ContextPack, DEFAULT_PER_FILE_CHARS, DEFAULT_TOTAL_CHARS, PackFile, PackKind, PackedContent,
    PackedText, pack_skill,
};
pub use paths::{
    find_chapter_prose, find_psych_file, find_scene_files, infer_chapter, name_matches_chapter,
    padded2,
};
pub use project::{ProjectRoot, discover, discover_one};
pub use prompt::{Prompt, build_prompt, prepare_skill};
pub use status::{NextAction, Slot, SlotState, StatusBoard};
pub use validate::validate_preview;
