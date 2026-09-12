//! Orchestrator modes from `open-storycraft/SKILL.md`.

use std::fmt;
use std::str::FromStr;

use crate::Error;

/// Job classification. The host picks exactly one per run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Mode {
    /// `fiction-story-sparks` only. No Wiki writes.
    Spark,
    /// New book. Spine starts at `fiction-genre`.
    NewProject,
    /// Continue. First missing required file on the spine.
    Resume,
    /// Named skill. Refuse if requires are missing.
    SingleSkill,
    /// Draft chapter N. Requires synopsis + outline.
    Draft,
    /// One editorial ladder step, never the whole ladder.
    Edit,
    /// Matching world skill + name/town generators.
    WorldPack,
    /// `skill-builder`.
    Meta,
}

impl Mode {
    /// Mode implied by a named skill run.
    #[must_use]
    pub fn for_skill(skill: &str) -> Self {
        match skill {
            "fiction-story-sparks" => Self::Spark,
            "fiction-writechapter" => Self::Draft,
            other if crate::chunk::is_chunked_skill(other) => Self::Edit,
            _ => Self::SingleSkill,
        }
    }

    /// Canonical lowercase identifier used on the status board and CLI.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Spark => "spark",
            Self::NewProject => "new-project",
            Self::Resume => "resume",
            Self::SingleSkill => "single-skill",
            Self::Draft => "draft",
            Self::Edit => "edit",
            Self::WorldPack => "world-pack",
            Self::Meta => "meta",
        }
    }
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Mode {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "spark" => Ok(Self::Spark),
            "new-project" => Ok(Self::NewProject),
            "resume" => Ok(Self::Resume),
            "single-skill" => Ok(Self::SingleSkill),
            "draft" => Ok(Self::Draft),
            "edit" => Ok(Self::Edit),
            "world-pack" => Ok(Self::WorldPack),
            "meta" => Ok(Self::Meta),
            other => Err(Error::UnknownMode(other.to_owned())),
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;

    #[test]
    fn parses_every_mode_round_trip() {
        for mode in [
            Mode::Spark,
            Mode::NewProject,
            Mode::Resume,
            Mode::SingleSkill,
            Mode::Draft,
            Mode::Edit,
            Mode::WorldPack,
            Mode::Meta,
        ] {
            let parsed: Mode = mode.as_str().parse().expect("mode token should parse");
            assert_eq!(parsed, mode);
        }
    }

    #[test]
    fn rejects_unknown_mode() {
        let err = "chat".parse::<Mode>().expect_err("unknown token");
        assert!(matches!(err, Error::UnknownMode(_)));
    }
}
