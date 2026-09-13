//! Where the API key lives.
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

use storycraft_llm::Secret;
use tauri::AppHandle;

use crate::error::AppError;
use crate::paths;

/// Service name for keyring entries.
const SERVICE: &str = "dev.openstorycraft.app";
/// Account name for the API key entry.
const ACCOUNT: &str = "api-key";
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
        match (self, cfg!(target_os = "android")) {
            (Self::Keyring, _) => "the system keyring",
            (Self::File, true) => "app-private storage (Android sandbox)",
            (Self::File, false) => "a local file, mode 0600 (no keyring on this machine)",
        }
    }
}

/// Reads and writes the provider API key.
#[derive(Debug, Clone)]
pub struct SecretStore {
    backend: Backend,
    path: PathBuf,
}

impl SecretStore {
    /// Open the store, picking a backend.
    ///
    /// # Errors
    ///
    /// Returns [`AppError`] when the app config directory cannot be resolved.
    pub fn open(app: &AppHandle) -> Result<Self, AppError> {
        let backend = match std::env::var(BACKEND_ENV).ok().as_deref() {
            Some("file") => Backend::File,
            Some("keyring") => Backend::Keyring,
            _ => detect_backend(),
        };
        Ok(Self {
            backend,
            path: paths::secret_file(app)?,
        })
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
            Backend::Keyring => keyring_get(),
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
            Backend::Keyring => keyring_set(value),
            Backend::File => file_set(&self.path, value),
        }
    }

    /// Forget the stored key. Missing is not an error.
    ///
    /// # Errors
    ///
    /// Returns [`AppError`] if the backend cannot delete.
    pub fn clear(&self) -> Result<(), AppError> {
        match self.backend {
            Backend::Keyring => keyring_clear(),
            Backend::File => file_clear(&self.path),
        }
    }
}

/// Keyring when it answers, file when it does not.
#[cfg(not(target_os = "android"))]
fn detect_backend() -> Backend {
    match keyring_get() {
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
fn entry() -> Result<keyring::Entry, AppError> {
    keyring::Entry::new(SERVICE, ACCOUNT)
        .map_err(|err| AppError::msg(format!("keyring unavailable: {err}")))
}

#[cfg(not(target_os = "android"))]
fn keyring_get() -> Result<Option<Secret>, AppError> {
    match entry()?.get_password() {
        Ok(secret) if secret.trim().is_empty() => Ok(None),
        Ok(secret) => Ok(Some(Secret::new(secret))),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(err) => Err(AppError::msg(format!("keyring read failed: {err}"))),
    }
}

#[cfg(not(target_os = "android"))]
fn keyring_set(value: &str) -> Result<(), AppError> {
    entry()?
        .set_password(value)
        .map_err(|err| AppError::msg(format!("keyring write failed: {err}")))
}

#[cfg(not(target_os = "android"))]
fn keyring_clear() -> Result<(), AppError> {
    match entry()?.delete_credential() {
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
fn keyring_get() -> Result<Option<Secret>, AppError> {
    Err(no_keyring())
}

#[cfg(target_os = "android")]
fn keyring_set(_value: &str) -> Result<(), AppError> {
    Err(no_keyring())
}

#[cfg(target_os = "android")]
fn keyring_clear() -> Result<(), AppError> {
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
