//! Config / resource locations. Android has no `$HOME/.config`.

use std::path::PathBuf;

use tauri::{AppHandle, Manager, path::BaseDirectory};

use crate::error::AppError;

/// App config directory (`~/.config/open-storycraft` on Linux, app-private on Android).
pub(crate) fn config_dir(app: &AppHandle) -> Result<PathBuf, AppError> {
    app.path()
        .app_config_dir()
        .map_err(|err| AppError::msg(format!("app config dir: {err}")))
}

/// `{config}/app.json`
pub(crate) fn settings_file(app: &AppHandle) -> Result<PathBuf, AppError> {
    if let Some(custom) = std::env::var_os("STORYCRAFT_APP_SETTINGS") {
        return Ok(PathBuf::from(custom));
    }
    Ok(config_dir(app)?.join("app.json"))
}

/// `{config}/oauth.json`
pub(crate) fn oauth_file(app: &AppHandle) -> Result<PathBuf, AppError> {
    if let Some(custom) = std::env::var_os("STORYCRAFT_AUTH_FILE") {
        return Ok(PathBuf::from(custom));
    }
    Ok(config_dir(app)?.join("oauth.json"))
}

/// Vendored skill pack: settings override, then bundled resources, then cwd walk.
pub(crate) fn skills_dir(app: &AppHandle, override_dir: Option<&str>) -> Result<PathBuf, AppError> {
    if let Some(path) = override_dir {
        return Ok(PathBuf::from(path));
    }
    if let Ok(bundled) = app.path().resolve("skills", BaseDirectory::Resource) {
        if bundled.join("fiction-genre/SKILL.md").is_file() {
            return Ok(bundled);
        }
        if bundled
            .join("open-storycraft/fiction-genre/SKILL.md")
            .is_file()
        {
            return Ok(bundled.join("open-storycraft"));
        }
    }
    let cwd = std::env::current_dir().map_err(|err| AppError::msg(err.to_string()))?;
    storycraft_core::find_skills_dir(&cwd)
        .ok_or_else(|| AppError::msg("no skill library found; set skills dir in Settings"))
}
