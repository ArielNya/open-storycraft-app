//! Desktop settings file. API keys stay out of the book Wiki.

use std::collections::BTreeMap;
use std::fs;

use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use crate::error::AppError;
use crate::paths;

/// Provider + last-project settings for the shell.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppSettings {
    /// Last opened book directory.
    #[serde(default)]
    pub last_project: Option<String>,
    /// Override for the vendored skill pack.
    #[serde(default)]
    pub skills_dir: Option<String>,
    /// `openai-compat` | `xai-apikey` | `grok-oauth`
    #[serde(default = "default_provider")]
    pub provider: String,
    /// OpenAI-compatible base URL, including `/v1`.
    #[serde(default = "default_base_url")]
    pub base_url: String,
    /// Optional API key. Never logged.
    #[serde(default)]
    pub api_key: Option<String>,
    /// `chat_completions` | `responses`
    #[serde(default = "default_api_style")]
    pub api_style: String,
    /// Default model id.
    #[serde(default = "default_model")]
    pub model: String,
    /// Optional cheaper model for editorial / kill-pass skills.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cheap_model: Option<String>,
    /// Exact `skill → model` overrides.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub skill_models: BTreeMap<String, String>,
    /// Overlay skill ids the user turned on (`ao3-writer`, `ao3`, `*`, …).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub enabled_overlays: Vec<String>,
    /// Packer total-char budget. `0` means the core default.
    #[serde(default = "default_budget")]
    pub token_budget: u32,
}

fn default_provider() -> String {
    "openai-compat".to_owned()
}
fn default_base_url() -> String {
    "https://api.x.ai/v1".to_owned()
}
fn default_api_style() -> String {
    "chat_completions".to_owned()
}
fn default_model() -> String {
    "grok-4.6".to_owned()
}
fn default_budget() -> u32 {
    48_000
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            last_project: None,
            skills_dir: None,
            provider: default_provider(),
            base_url: default_base_url(),
            api_key: None,
            api_style: default_api_style(),
            model: default_model(),
            cheap_model: None,
            skill_models: BTreeMap::new(),
            enabled_overlays: Vec::new(),
            token_budget: default_budget(),
        }
    }
}

/// Load settings, or defaults if the file is missing.
///
/// # Errors
///
/// Returns [`AppError`] if the file exists but is not valid JSON.
pub fn load(app: &AppHandle) -> Result<AppSettings, AppError> {
    let path = paths::settings_file(app)?;
    match fs::read(&path) {
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(AppSettings::default()),
        Err(err) => Err(AppError::io(&path, &err)),
        Ok(bytes) => serde_json::from_slice(&bytes)
            .map_err(|err| AppError::msg(format!("invalid settings: {err}"))),
    }
}

/// Persist settings. Unix mode 0600 because the file may hold an API key.
///
/// # Errors
///
/// Returns [`AppError`] if the file cannot be written.
pub fn save(app: &AppHandle, settings: &AppSettings) -> Result<(), AppError> {
    let path = paths::settings_file(app)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| AppError::io(parent, &err))?;
    }
    let bytes = serde_json::to_vec_pretty(settings)
        .map_err(|err| AppError::msg(format!("serialize settings: {err}")))?;
    fs::write(&path, bytes).map_err(|err| AppError::io(&path, &err))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = fs::Permissions::from_mode(0o600);
        fs::set_permissions(&path, perms).map_err(|err| AppError::io(&path, &err))?;
    }
    Ok(())
}
