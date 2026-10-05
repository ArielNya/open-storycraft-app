//! Where the API key and the xAI OAuth tokens live.
//!
//! Not in the settings file. The settings hold everything else — URL, provider,
//! model, budget — and none of it is secret. The key goes to the OS secret
//! store:
//!
//! - Desktop: the system keyring (Secret Service on Linux, Keychain on macOS,
//!   Credential Manager on Windows).
//! - Desktop with no keyring reachable — a container, a headless box, a session
//!   without a Secret Service daemon: a mode-0600 file in the app config
//!   directory. The app reports which of the two it used rather than pretending.
//! - Android: app-private storage, which the OS keeps away from other apps.
//!   This crate has no Android keyring backend, so the file is the honest
//!   option; it is still inside the app sandbox.
//!
//! `STORYCRAFT_SECRET_BACKEND=keyring|file` forces a backend, which is how the
//! file path is tested on a machine that does have a keyring.

use std::fs;
use std::path::{Path, PathBuf};

use storycraft_auth::TokenSet;
use storycraft_llm::Secret;
use tauri::AppHandle;

use crate::error::AppError;
use crate::paths;

/// Service name for keyring entries.
const SERVICE: &str = "dev.openstorycraft.app";
/// Account name for the API key entry.
const API_KEY: &str = "api-key";
/// Account name for the OAuth token set (JSON).
const OAUTH: &str = "oauth-tokens";
/// Backend override: `keyring` or `file`.
const BACKEND_ENV: &str = "STORYCRAFT_SECRET_BACKEND";

/// Where a secret is kept.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Backend {
    /// OS keyring / credential manager.
    Keyring,
    /// Mode-0600 file in the app config directory.
    File,
}

impl Backend {
    /// Token reported to the UI.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Keyring => "keyring",
            Self::File => "file",
        }
    }

    /// Human description of where the key sits, for the settings screen.
    ///
    /// The wording is platform-specific: on Android the fallback is the app's
    /// private storage inside the OS sandbox, not a loose file in a home
    /// directory, and saying otherwise would mislead.
    #[must_use]
    pub const fn label(self) -> &'static str {
        if cfg!(target_os = "android") {
            match self {
                Self::Keyring => "the system keyring",
                Self::File => "app-private storage (Android sandbox)",
            }
        } else if cfg!(windows) {
            match self {
                Self::Keyring => "Windows Credential Manager",
                Self::File => "a file in your user profile (Credential Manager unavailable)",
            }
        } else {
            match self {
                Self::Keyring => "the system keyring",
                Self::File => "a local file, mode 0600 (no keyring on this machine)",
            }
        }
    }
}

/// Reads and writes one secret: the provider API key or the OAuth tokens.
#[derive(Debug, Clone)]
pub struct SecretStore {
    backend: Backend,
    account: &'static str,
    path: PathBuf,
}

impl SecretStore {
    /// Open the store, picking a backend.
    ///
    /// # Errors
    ///
    /// Returns [`AppError`] when the app config directory cannot be resolved.
    pub fn open(app: &AppHandle) -> Result<Self, AppError> {
        Ok(Self::with(API_KEY, paths::secret_file(app)?))
    }

    /// Open the store for the xAI OAuth token set.
    ///
    /// Tokens written by older builds to `oauth.json` are still read from that
    /// file, and the next save moves them into the keyring.
    ///
    /// # Errors
    ///
    /// Returns [`AppError`] when the app config directory cannot be resolved.
    pub fn open_oauth(app: &AppHandle) -> Result<Self, AppError> {
        Ok(Self::with(OAUTH, paths::oauth_file(app)?))
    }

    fn with(account: &'static str, path: PathBuf) -> Self {
        let backend = match std::env::var(BACKEND_ENV).ok().as_deref() {
            Some("file") => Backend::File,
            Some("keyring") => Backend::Keyring,
            _ => detect_backend(),
        };
        Self {
            backend,
            account,
            path,
        }
    }

    /// Which backend this store uses.
    #[must_use]
    pub const fn backend(&self) -> Backend {
        self.backend
    }

    /// The stored key, if any.
    ///
    /// # Errors
    ///
    /// Returns [`AppError`] when the backend fails in a way that is not simply
    /// "nothing stored".
    pub fn get(&self) -> Result<Option<Secret>, AppError> {
        match self.backend {
            // A keyring miss still checks the file: a secret too big for the
            // keyring, or one an older build wrote there, lives in it.
            Backend::Keyring => match keyring_get(self.account)? {
                Some(secret) => Ok(Some(secret)),
                None => file_get(&self.path),
            },
            Backend::File => file_get(&self.path),
        }
    }

    /// Whether a key is stored.
    ///
    /// # Errors
    ///
    /// Same as [`SecretStore::get`].
    pub fn has_key(&self) -> Result<bool, AppError> {
        Ok(self.get()?.is_some())
    }

    /// Store (or replace) the key.
    ///
    /// # Errors
    ///
    /// Returns [`AppError`] if the backend cannot write.
    pub fn set(&self, value: &str) -> Result<(), AppError> {
        match self.backend {
            Backend::Keyring => match keyring_set(self.account, value) {
                // Drop any older file copy so it cannot shadow the new value.
                Ok(()) => file_clear(&self.path),
                // ponytail: Credential Manager caps a secret at 2560 bytes; a
                // token set past that goes to the file instead of failing sign-in.
                Err(err) => {
                    tracing::warn!(%err, account = self.account, "keyring refused the secret; using the file");
                    file_set(&self.path, value)
                }
            },
            Backend::File => file_set(&self.path, value),
        }
    }

    /// Forget the stored key. Missing is not an error.
    ///
    /// # Errors
    ///
    /// Returns [`AppError`] if the backend cannot delete.
    pub fn clear(&self) -> Result<(), AppError> {
        if self.backend == Backend::Keyring {
            keyring_clear(self.account)?;
        }
        file_clear(&self.path)
    }
}

/// The stored xAI OAuth token set, if signed in.
///
/// # Errors
///
/// Returns [`AppError`] if the store fails or holds something that is not a
/// token set.
pub fn load_tokens(app: &AppHandle) -> Result<Option<TokenSet>, AppError> {
    let Some(secret) = SecretStore::open_oauth(app)?.get()? else {
        return Ok(None);
    };
    serde_json::from_str(secret.expose())
        .map(Some)
        .map_err(|err| AppError::msg(format!("stored OAuth tokens are unreadable: {err}")))
}

/// Store the xAI OAuth token set.
///
/// # Errors
///
/// Returns [`AppError`] if the store cannot write.
pub fn save_tokens(app: &AppHandle, tokens: &TokenSet) -> Result<(), AppError> {
    let json = serde_json::to_string(tokens)
        .map_err(|err| AppError::msg(format!("serialize OAuth tokens: {err}")))?;
    SecretStore::open_oauth(app)?.set(&json)
}

/// Keyring when it answers, file when it does not.
#[cfg(not(target_os = "android"))]
fn detect_backend() -> Backend {
    match keyring_get(API_KEY) {
        Ok(_) => Backend::Keyring,
        Err(err) => {
            tracing::warn!(%err, "no system keyring; storing the API key in a 0600 file");
            Backend::File
        }
    }
}

/// No keyring on Android: app-private storage.
#[cfg(target_os = "android")]
fn detect_backend() -> Backend {
    Backend::File
}

#[cfg(not(target_os = "android"))]
fn entry(account: &str) -> Result<keyring::Entry, AppError> {
    keyring::Entry::new(SERVICE, account)
        .map_err(|err| AppError::msg(format!("keyring unavailable: {err}")))
}

#[cfg(not(target_os = "android"))]
fn keyring_get(account: &str) -> Result<Option<Secret>, AppError> {
    // Tokens go in as raw UTF-8 bytes: Windows stores a "password" as UTF-16,
    // which halves the room a token set gets.
    let read = if account == OAUTH {
        entry(account)?
            .get_secret()
            .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
    } else {
        entry(account)?.get_password()
    };
    match read {
        Ok(secret) if secret.trim().is_empty() => Ok(None),
        Ok(secret) => Ok(Some(Secret::new(secret))),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(err) => Err(AppError::msg(format!("keyring read failed: {err}"))),
    }
}

#[cfg(not(target_os = "android"))]
fn keyring_set(account: &str, value: &str) -> Result<(), AppError> {
    let entry = entry(account)?;
    if account == OAUTH {
        entry.set_secret(value.as_bytes())
    } else {
        entry.set_password(value)
    }
    .map_err(|err| AppError::msg(format!("keyring write failed: {err}")))
}

#[cfg(not(target_os = "android"))]
fn keyring_clear(account: &str) -> Result<(), AppError> {
    match entry(account)?.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(err) => Err(AppError::msg(format!("keyring delete failed: {err}"))),
    }
}

/// Android has no keyring backend in this crate. The variants still exist so the
/// store type is the same everywhere, but asking for them is a programming
/// error the user can see rather than a compile-time gap.
#[cfg(target_os = "android")]
fn no_keyring() -> AppError {
    AppError::msg(format!(
        "no keyring backend on this platform; unset {BACKEND_ENV} to use app-private storage"
    ))
}

#[cfg(target_os = "android")]
fn keyring_get(_account: &str) -> Result<Option<Secret>, AppError> {
    Err(no_keyring())
}

#[cfg(target_os = "android")]
fn keyring_set(_account: &str, _value: &str) -> Result<(), AppError> {
    Err(no_keyring())
}

#[cfg(target_os = "android")]
fn keyring_clear(_account: &str) -> Result<(), AppError> {
    Err(no_keyring())
}

fn file_get(path: &Path) -> Result<Option<Secret>, AppError> {
    match fs::read_to_string(path) {
        Ok(text) if text.trim().is_empty() => Ok(None),
        Ok(text) => Ok(Some(Secret::new(text.trim()))),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(err) => Err(AppError::io(path, &err)),
    }
}

fn file_set(path: &Path, value: &str) -> Result<(), AppError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| AppError::io(parent, &err))?;
    }
    fs::write(path, format!("{value}\n")).map_err(|err| AppError::io(path, &err))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600))
            .map_err(|err| AppError::io(path, &err))?;
    }
    Ok(())
}

fn file_clear(path: &Path) -> Result<(), AppError> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(err) => Err(AppError::io(path, &err)),
    }
}

#[cfg(test)]
impl SecretStore {
    /// A store pinned to `path` with the file backend, for tests.
    pub(crate) fn for_test(path: PathBuf) -> Self {
        Self {
            backend: Backend::File,
            account: API_KEY,
            path,
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;

    fn store_in(dir: &Path) -> SecretStore {
        SecretStore::for_test(dir.join("api-key"))
    }

    #[test]
    fn file_backend_round_trips() {
        let tmp = tempfile::tempdir().unwrap();
        let store = store_in(tmp.path());
        assert!(store.get().unwrap().is_none(), "nothing stored yet");
        store.set("sk-test-123").unwrap();
        assert_eq!(store.get().unwrap().unwrap().expose(), "sk-test-123");
        store.clear().unwrap();
        assert!(store.get().unwrap().is_none());
        store.clear().unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn file_backend_is_not_world_readable() {
        use std::os::unix::fs::PermissionsExt;
        let tmp = tempfile::tempdir().unwrap();
        let store = store_in(tmp.path());
        store.set("sk-test-123").unwrap();
        let mode = fs::metadata(tmp.path().join("api-key"))
            .unwrap()
            .permissions()
            .mode();
        assert_eq!(mode & 0o777, 0o600, "got {mode:o}");
    }

    #[test]
    fn a_debug_print_never_contains_the_key() {
        let tmp = tempfile::tempdir().unwrap();
        let store = store_in(tmp.path());
        store.set("sk-super-secret").unwrap();
        let printed = format!("{:?}", store.get().unwrap().unwrap());
        assert!(!printed.contains("sk-super-secret"), "{printed}");
    }
}
