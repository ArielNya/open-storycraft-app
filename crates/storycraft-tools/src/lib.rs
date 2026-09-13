//! Local Storycraft tools: Markov names/towns and burstiness stats.
//!
//! No network. No Python. These are the default before spending tokens.

#![deny(clippy::correctness)]

mod burstiness;
mod error;
mod markov;

pub use burstiness::{BurstinessReport, OpenerStat, RepetitionHotspot, measure};
pub use error::Error;
pub use markov::{GenerateOpts, generate, generate_from_path, load_list, parse_list};

/// Skills that must run on-device, never as an LLM job.
#[must_use]
pub fn is_local_tool(skill: &str) -> bool {
    matches!(
        skill,
        "burstiness-check" | "name-generator" | "town-generator" | "storybible-import"
    )
}

/// Relative data file for `name-generator` (`data/<culture>.txt`).
#[must_use]
pub fn name_list_rel(culture: &str) -> String {
    format!("name-generator/data/{culture}.txt")
}

/// Relative data file for `town-generator` (`data/<list>.txt`).
#[must_use]
pub fn town_list_rel(list: &str) -> String {
    format!("town-generator/data/{list}.txt")
}
