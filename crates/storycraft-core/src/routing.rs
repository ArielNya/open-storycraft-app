//! Per-skill model routing. Editorial passes can use a cheaper model.

use std::collections::BTreeMap;

use crate::chunk::is_chunked_skill;

/// Default model plus optional cheap/editorial override and exact skill routes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelRouter {
    /// Model used when nothing more specific matches.
    pub default_model: String,
    /// Optional model for editorial / kill-pass skills.
    pub cheap_model: Option<String>,
    /// Exact `skill → model` overrides. Highest priority.
    pub routes: BTreeMap<String, String>,
}

impl ModelRouter {
    /// Build a router. Empty `cheap_model` / `routes` fall through to default.
    #[must_use]
    pub fn new(
        default_model: impl Into<String>,
        cheap_model: Option<String>,
        routes: BTreeMap<String, String>,
    ) -> Self {
        Self {
            default_model: default_model.into(),
            cheap_model: cheap_model.filter(|m| !m.is_empty()),
            routes,
        }
    }

    /// Model id to send for `skill`.
    #[must_use]
    pub fn model_for(&self, skill: &str) -> &str {
        if let Some(model) = self.routes.get(skill)
            && !model.is_empty()
        {
            return model;
        }
        if is_cheap_skill(skill)
            && let Some(cheap) = self.cheap_model.as_deref()
        {
            return cheap;
        }
        &self.default_model
    }
}

/// Skills that should use `cheap_model` when one is configured.
#[must_use]
pub fn is_cheap_skill(skill: &str) -> bool {
    is_chunked_skill(skill)
        || matches!(
            skill,
            "coldread"
                | "fiction-dev-editor"
                | "fiction-reviewchapter"
                | "fiction-aiism-editor"
                | "fiction-prose-editor"
                | "fiction-full-editor"
                | "levelup"
                | "nominalization-hunt"
                | "pangram"
        )
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;

    fn router() -> ModelRouter {
        let mut routes = BTreeMap::new();
        routes.insert("kill-flat".into(), "exact-model".into());
        ModelRouter::new("grok-4.6", Some("cheap-mini".into()), routes)
    }

    #[test]
    fn writechapter_keeps_the_default() {
        assert_eq!(router().model_for("fiction-writechapter"), "grok-4.6");
        assert_eq!(router().model_for("fiction-genre"), "grok-4.6");
    }

    #[test]
    fn editorial_uses_cheap_when_set() {
        assert_eq!(router().model_for("kill-crutch"), "cheap-mini");
        assert_eq!(router().model_for("fiction-line-editor"), "cheap-mini");
        assert_eq!(router().model_for("fiction-aiism-editor"), "cheap-mini");
    }

    #[test]
    fn exact_route_beats_cheap_and_default() {
        assert_eq!(router().model_for("kill-flat"), "exact-model");
    }

    #[test]
    fn missing_cheap_falls_back_to_default() {
        let router = ModelRouter::new("grok-4.6", None, BTreeMap::new());
        assert_eq!(router.model_for("kill-crutch"), "grok-4.6");
    }
}
