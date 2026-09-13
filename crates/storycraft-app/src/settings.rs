//! Desktop settings file. Holds no secrets — the API key lives in the OS secret
//! store (`crate::secrets`).

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use crate::error::AppError;
use crate::paths;
use crate::secrets::SecretStore;

/// Provider + last-project settings for the shell.
///
/// Everything here is safe to read: the API key is not a field. Older builds
/// wrote one, and [`import_plaintext_key`] moves it into the secret store.
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

/// What the webview sends and receives.
///
/// The key travels one way: in. It is never serialized back, so it never
/// reaches the DOM, a devtools session, or a screenshot.
#[derive(Clone, Serialize, Deserialize)]
pub struct SettingsDto {
    /// Everything that is safe to store and display.
    #[serde(flatten)]
    pub settings: AppSettings,
    /// A key to store this round. Omitted when the field was left empty.
    #[serde(default, skip_serializing)]
    pub api_key: Option<String>,
    /// Forget the stored key instead of setting one.
    #[serde(default)]
    pub clear_api_key: bool,
    /// Whether a key is on file. Never the key itself.
    #[serde(default)]
    pub has_api_key: bool,
    /// `keyring` or `file` — where the key is kept.
    #[serde(default)]
    pub secret_backend: String,
    /// That backend in words, for display.
    #[serde(default)]
    pub secret_backend_label: String,
}

impl std::fmt::Debug for SettingsDto {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SettingsDto")
            .field("settings", &self.settings)
            .field("api_key", &self.api_key.as_ref().map(|_| "[redacted]"))
            .field("clear_api_key", &self.clear_api_key)
            .field("has_api_key", &self.has_api_key)
            .field("secret_backend", &self.secret_backend)
            .field("secret_backend_label", &self.secret_backend_label)
            .finish()
    }
}

impl SettingsDto {
    /// Build the outgoing view: settings plus what the store knows, never keys.
    #[must_use]
    pub fn from_settings(settings: AppSettings, store: &SecretStore) -> Self {
        let has_api_key = store.has_key().unwrap_or(false);
        Self {
            settings,
            api_key: None,
            clear_api_key: false,
            has_api_key,
            secret_backend: store.backend().as_str().to_owned(),
            secret_backend_label: store.backend().label().to_owned(),
        }
    }

    /// The key from this request, if the user typed one.
    #[must_use]
    pub fn incoming_key(&self) -> Option<&str> {
        self.api_key
            .as_deref()
            .map(str::trim)
            .filter(|key| !key.is_empty())
    }
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

/// Persist settings. Mode 0600 on Unix.
///
/// # Errors
///
/// Returns [`AppError`] if the file cannot be written.
pub fn save(app: &AppHandle, settings: &AppSettings) -> Result<(), AppError> {
    write_settings(&paths::settings_file(app)?, settings)
}

fn write_settings(path: &Path, settings: &AppSettings) -> Result<(), AppError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| AppError::io(parent, &err))?;
    }
    let bytes = serde_json::to_vec_pretty(settings)
        .map_err(|err| AppError::msg(format!("serialize settings: {err}")))?;
    fs::write(path, bytes).map_err(|err| AppError::io(path, &err))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = fs::Permissions::from_mode(0o600);
        fs::set_permissions(path, perms).map_err(|err| AppError::io(path, &err))?;
    }
    Ok(())
}

/// Move an API key written by an older build into the secret store.
///
/// Older settings files carried `api_key` in plain text. The first run after an
/// upgrade lifts it into the keyring (or the 0600 file) and rewrites the
/// settings without it. A key already in the store wins, so a stale file cannot
/// clobber a newer one. Returns whether anything was moved.
///
/// # Errors
///
/// Returns [`AppError`] if the settings file cannot be read, or the store or
/// file cannot be written.
pub fn import_plaintext_key(app: &AppHandle, store: &SecretStore) -> Result<bool, AppError> {
    let path = paths::settings_file(app)?;
    let Ok(bytes) = fs::read(&path) else {
        return Ok(false);
    };
    let Ok(value) = serde_json::from_slice::<serde_json::Value>(&bytes) else {
        return Ok(false);
    };
    let found = value
        .get("api_key")
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|key| !key.is_empty())
        .map(ToOwned::to_owned);
    let Some(found) = found else {
        return Ok(false);
    };
    let settings: AppSettings = serde_json::from_value(value)
        .map_err(|err| AppError::msg(format!("invalid settings: {err}")))?;
    if store.has_key()? {
        tracing::info!(
            "settings carried a key but the secret store already has one; dropped the file copy"
        );
    } else {
        store.set(&found)?;
    }
    write_settings(&path, &settings)?;
    tracing::info!("moved the plaintext API key out of the settings file");
    Ok(true)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;

    fn store() -> (tempfile::TempDir, SecretStore) {
        let tmp = tempfile::tempdir().unwrap();
        let store = SecretStore::for_test(tmp.path().join("api-key"));
        (tmp, store)
    }

    #[test]
    fn settings_never_serialize_a_key() {
        let json = serde_json::to_string(&AppSettings::default()).unwrap();
        assert!(!json.contains("api_key"), "{json}");
    }

    #[test]
    fn an_old_settings_file_still_loads_without_the_key() {
        let old = r#"{"last_project":null,"provider":"xai-apikey","base_url":"https://api.x.ai/v1",
            "api_key":"sk-legacy","api_style":"responses","model":"grok-4.6","token_budget":32000}"#;
        let parsed: AppSettings = serde_json::from_str(old).unwrap();
        assert_eq!(parsed.model, "grok-4.6");
        assert_eq!(parsed.token_budget, 32_000);
        let rewritten = serde_json::to_string(&parsed).unwrap();
        assert!(!rewritten.contains("sk-legacy"), "{rewritten}");
    }

    #[test]
    fn the_dto_hides_the_key_but_reports_it() {
        let (_tmp, store) = store();
        store.set("sk-test-123").unwrap();
        let dto = SettingsDto::from_settings(AppSettings::default(), &store);
        let json = serde_json::to_string(&dto).unwrap();
        assert!(!json.contains("sk-test-123"), "{json}");
        assert!(dto.has_api_key);
        assert_eq!(dto.secret_backend, "file");
        assert!(!dto.secret_backend_label.is_empty());
        assert!(!format!("{dto:?}").contains("sk-test-123"));
    }

    #[test]
    fn incoming_keys_are_trimmed_and_blanks_ignored() {
        let (_tmp, store) = store();
        let mut dto = SettingsDto::from_settings(AppSettings::default(), &store);
        assert!(dto.incoming_key().is_none());
        dto.api_key = Some("   ".to_owned());
        assert!(dto.incoming_key().is_none());
        dto.api_key = Some(" sk-live ".to_owned());
        assert_eq!(dto.incoming_key(), Some("sk-live"));
    }

    #[test]
    fn writing_settings_drops_a_key_that_was_there() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("app.json");
        fs::write(
            &path,
            r#"{"api_key":"sk-legacy","provider":"openai-compat","base_url":"http://x/v1",
                "api_style":"chat_completions","model":"m","token_budget":1000}"#,
        )
        .unwrap();
        let parsed =
            serde_json::from_str::<AppSettings>(&fs::read_to_string(&path).unwrap()).unwrap();
        write_settings(&path, &parsed).unwrap();
        let text = fs::read_to_string(&path).unwrap();
        assert!(!text.contains("sk-legacy"), "{text}");
        assert!(!text.contains("api_key"), "{text}");
    }
}
