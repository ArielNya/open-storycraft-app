//! Desktop settings file. Holds no secrets — each connection profile's API key
//! lives in the OS secret store (`crate::secrets`).

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use crate::error::AppError;
use crate::paths;
use crate::secrets::SecretStore;

/// Id of the profile an older single-provider settings file becomes. Its key
/// keeps the secret-store entry those builds wrote, so nothing has to move.
pub const DEFAULT_PROFILE: &str = "default";

/// One saved connection: where to send requests and which models to use.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderProfile {
    /// Stable id: letters, digits, `-` and `_`. Names the profile's key entry.
    #[serde(default = "default_profile_id")]
    pub id: String,
    /// What the user calls it ("NVIDIA NIM", "Google AI Studio").
    #[serde(default = "default_profile_name")]
    pub name: String,
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
}

impl Default for ProviderProfile {
    fn default() -> Self {
        Self {
            id: default_profile_id(),
            name: default_profile_name(),
            provider: default_provider(),
            base_url: default_base_url(),
            api_style: default_api_style(),
            model: default_model(),
            cheap_model: None,
            skill_models: BTreeMap::new(),
        }
    }
}

/// Connection profiles + last-project settings for the shell.
///
/// Everything here is safe to read: no API key is a field. Older builds wrote
/// one, and [`import_plaintext_key`] moves it into the secret store.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppSettings {
    /// Last opened book directory.
    #[serde(default)]
    pub last_project: Option<String>,
    /// Override for the vendored skill pack.
    #[serde(default)]
    pub skills_dir: Option<String>,
    /// Saved connections. Never empty once loaded.
    #[serde(default)]
    pub profiles: Vec<ProviderProfile>,
    /// Id of the profile runs use.
    #[serde(default)]
    pub active_profile: String,
    /// Overlay skill ids the user turned on (`ao3-writer`, `ao3`, `*`, …).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub enabled_overlays: Vec<String>,
    /// Packer total-char budget. `0` means the core default.
    #[serde(default = "default_budget")]
    pub token_budget: u32,
}

impl AppSettings {
    /// The profile runs use. Falls back to the first one if the id is stale.
    #[must_use]
    pub fn active(&self) -> &ProviderProfile {
        self.profiles
            .iter()
            .find(|profile| profile.id == self.active_profile)
            .or_else(|| self.profiles.first())
            .unwrap_or(&FALLBACK_PROFILE)
    }

    /// Make the settings whole: at least one profile, unique valid ids, and an
    /// active profile that exists.
    ///
    /// # Errors
    ///
    /// Returns [`AppError`] when a profile id is empty, repeated, or has
    /// characters a secret-store entry cannot carry.
    pub fn normalize(&mut self) -> Result<(), AppError> {
        if self.profiles.is_empty() {
            self.profiles.push(ProviderProfile::default());
        }
        let mut seen = std::collections::BTreeSet::new();
        for profile in &mut self.profiles {
            if !valid_profile_id(&profile.id) {
                return Err(AppError::msg(format!(
                    "profile id {:?} must be letters, digits, '-' or '_'",
                    profile.id
                )));
            }
            if !seen.insert(profile.id.clone()) {
                return Err(AppError::msg(format!(
                    "two profiles share the id {:?}",
                    profile.id
                )));
            }
            if profile.name.trim().is_empty() {
                profile.name = profile.id.clone();
            }
        }
        if !self.profiles.iter().any(|p| p.id == self.active_profile) {
            self.active_profile = self.profiles[0].id.clone();
        }
        Ok(())
    }

    /// Read a settings file, upgrading the single-provider layout older builds
    /// wrote into one `Default` profile.
    ///
    /// # Errors
    ///
    /// Returns [`AppError`] when the JSON does not fit either layout.
    pub fn from_json(value: serde_json::Value) -> Result<Self, AppError> {
        let invalid = |err: serde_json::Error| AppError::msg(format!("invalid settings: {err}"));
        let mut settings: Self = serde_json::from_value(value.clone()).map_err(invalid)?;
        if settings.profiles.is_empty() {
            // Older builds kept provider, base_url, model … at the top level.
            let legacy: ProviderProfile = serde_json::from_value(value).map_err(invalid)?;
            settings.profiles.push(ProviderProfile {
                id: default_profile_id(),
                name: default_profile_name(),
                ..legacy
            });
        }
        settings.normalize()?;
        Ok(settings)
    }
}

static FALLBACK_PROFILE: std::sync::LazyLock<ProviderProfile> =
    std::sync::LazyLock::new(ProviderProfile::default);

fn valid_profile_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 64
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

/// What the webview sends and receives.
///
/// Keys travel one way: in. They are never serialized back, so they never
/// reach the DOM, a devtools session, or a screenshot.
#[derive(Clone, Serialize, Deserialize)]
pub struct SettingsDto {
    /// Everything that is safe to store and display.
    #[serde(flatten)]
    pub settings: AppSettings,
    /// A key to store for the active profile this round. Omitted when the
    /// field was left empty.
    #[serde(default, skip_serializing)]
    pub api_key: Option<String>,
    /// Forget the active profile's stored key instead of setting one.
    #[serde(default)]
    pub clear_api_key: bool,
    /// Ids of the profiles that have a key on file. Never the keys themselves.
    #[serde(default)]
    pub keyed_profiles: Vec<String>,
    /// `keyring` or `file` — where keys are kept.
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
            .field("keyed_profiles", &self.keyed_profiles)
            .field("secret_backend", &self.secret_backend)
            .field("secret_backend_label", &self.secret_backend_label)
            .finish()
    }
}

impl SettingsDto {
    /// Build the outgoing view: settings plus which profiles have a key, never
    /// the keys. `store_for` opens the store for one profile id.
    #[must_use]
    pub fn from_settings(
        settings: AppSettings,
        store_for: impl Fn(&str) -> Option<SecretStore>,
    ) -> Self {
        let keyed_profiles = settings
            .profiles
            .iter()
            .filter(|profile| {
                store_for(&profile.id).is_some_and(|store| store.has_key().unwrap_or(false))
            })
            .map(|profile| profile.id.clone())
            .collect();
        let backend = store_for(DEFAULT_PROFILE).map(|store| store.backend());
        Self {
            settings,
            api_key: None,
            clear_api_key: false,
            keyed_profiles,
            secret_backend: backend.map(|b| b.as_str().to_owned()).unwrap_or_default(),
            secret_backend_label: backend.map(|b| b.label().to_owned()).unwrap_or_default(),
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

fn default_profile_id() -> String {
    DEFAULT_PROFILE.to_owned()
}
fn default_profile_name() -> String {
    "Default".to_owned()
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
            profiles: vec![ProviderProfile::default()],
            active_profile: default_profile_id(),
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
        Ok(bytes) => AppSettings::from_json(
            serde_json::from_slice(&bytes)
                .map_err(|err| AppError::msg(format!("invalid settings: {err}")))?,
        ),
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
/// upgrade lifts it into the default profile's entry in the keyring (or the
/// 0600 file) and rewrites the settings without it. A key already in the store
/// wins, so a stale file cannot clobber a newer one. Returns whether anything
/// was moved.
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
    let settings = AppSettings::from_json(value)?;
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

    fn parse(json: &str) -> AppSettings {
        AppSettings::from_json(serde_json::from_str(json).unwrap()).unwrap()
    }

    #[test]
    fn settings_never_serialize_a_key() {
        let json = serde_json::to_string(&AppSettings::default()).unwrap();
        assert!(!json.contains("api_key"), "{json}");
    }

    #[test]
    fn an_old_settings_file_becomes_the_default_profile() {
        let old = r#"{"last_project":null,"provider":"xai-apikey","base_url":"https://api.x.ai/v1",
            "api_key":"sk-legacy","api_style":"responses","model":"grok-4.6",
            "cheap_model":"mini","skill_models":{"kill-flat":"tiny"},"token_budget":32000}"#;
        let parsed = parse(old);
        assert_eq!(parsed.profiles.len(), 1);
        let profile = parsed.active();
        assert_eq!(profile.id, DEFAULT_PROFILE);
        assert_eq!(profile.name, "Default");
        assert_eq!(profile.provider, "xai-apikey");
        assert_eq!(profile.api_style, "responses");
        assert_eq!(profile.cheap_model.as_deref(), Some("mini"));
        assert_eq!(
            profile.skill_models.get("kill-flat").map(String::as_str),
            Some("tiny")
        );
        assert_eq!(parsed.token_budget, 32_000);
        let rewritten = serde_json::to_string(&parsed).unwrap();
        assert!(!rewritten.contains("sk-legacy"), "{rewritten}");
        assert!(rewritten.contains("\"profiles\""), "{rewritten}");
    }

    #[test]
    fn profiles_round_trip_and_the_active_one_is_used() {
        let json = r#"{"profiles":[
            {"id":"default","name":"NVIDIA NIM","base_url":"https://integrate.api.nvidia.com/v1","model":"z-ai/glm-5.3"},
            {"id":"google","name":"Google AI Studio","base_url":"https://generativelanguage.googleapis.com/v1beta/openai","model":"models/gemma-4-31b-it"}
        ],"active_profile":"google"}"#;
        let parsed = parse(json);
        assert_eq!(parsed.active().name, "Google AI Studio");
        let again = parse(&serde_json::to_string(&parsed).unwrap());
        assert_eq!(again, parsed);
    }

    #[test]
    fn a_stale_active_id_falls_back_and_bad_ids_are_refused() {
        let parsed = parse(r#"{"profiles":[{"id":"a"},{"id":"b"}],"active_profile":"gone"}"#);
        assert_eq!(parsed.active_profile, "a");
        let bad = serde_json::from_str(r#"{"profiles":[{"id":"no spaces"}]}"#).unwrap();
        assert!(AppSettings::from_json(bad).is_err());
        let twice = serde_json::from_str(r#"{"profiles":[{"id":"x"},{"id":"x"}]}"#).unwrap();
        assert!(AppSettings::from_json(twice).is_err());
    }

    #[test]
    fn the_dto_hides_keys_but_reports_which_profiles_have_one() {
        let (_tmp, store) = store();
        store.set("sk-test-123").unwrap();
        let dto = SettingsDto::from_settings(AppSettings::default(), |id| {
            (id == DEFAULT_PROFILE).then(|| store.clone())
        });
        let json = serde_json::to_string(&dto).unwrap();
        assert!(!json.contains("sk-test-123"), "{json}");
        assert_eq!(dto.keyed_profiles, vec![DEFAULT_PROFILE.to_owned()]);
        assert_eq!(dto.secret_backend, "file");
        assert!(!dto.secret_backend_label.is_empty());
        assert!(!format!("{dto:?}").contains("sk-test-123"));
    }

    #[test]
    fn incoming_keys_are_trimmed_and_blanks_ignored() {
        let mut dto = SettingsDto::from_settings(AppSettings::default(), |_| None);
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
        let parsed = parse(&fs::read_to_string(&path).unwrap());
        write_settings(&path, &parsed).unwrap();
        let text = fs::read_to_string(&path).unwrap();
        assert!(!text.contains("sk-legacy"), "{text}");
        assert!(!text.contains("api_key"), "{text}");
        assert!(text.contains("\"profiles\""), "{text}");
    }
}
