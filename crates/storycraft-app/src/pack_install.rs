//! Putting the bundled skill pack where the app can read it.
//!
//! Desktop ships the pack as bundle resources and can also find it in the
//! working tree, so nothing needs installing. Android ships it as APK assets,
//! and APK assets are not files: `std::fs` cannot open
//! `file:///android_asset/…`, which is what Tauri's resource path resolves to
//! there. On Android the pack is therefore embedded in the library at compile
//! time and written into app storage on first use, after which every other code
//! path treats it as an ordinary directory.

// Android is the only target that installs a pack; desktop has bundle resources
// and a working tree. The module compiles everywhere so the extraction logic
// stays testable on the host.
#![cfg_attr(not(any(target_os = "android", test)), allow(dead_code))]

use std::fs;
use std::path::Path;

use crate::error::AppError;

/// Marker file recording which pack version was extracted.
const MARKER: &str = ".pack-version";

/// Write `pack` under `target`, replacing an extraction of a different version.
///
/// Returns whether anything was written. A stale extraction is removed whole,
/// so a skill deleted from the pack does not linger on the device.
///
/// # Errors
///
/// Returns [`AppError`] if `target` cannot be cleared or a file cannot be
/// written.
pub(crate) fn install(
    pack: &include_dir::Dir<'_>,
    target: &Path,
    version: &str,
) -> Result<bool, AppError> {
    let marker = target.join(MARKER);
    if fs::read_to_string(&marker).is_ok_and(|found| found == version) {
        return Ok(false);
    }
    match fs::remove_dir_all(target) {
        Ok(()) => {}
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
        Err(err) => return Err(AppError::io(target, &err)),
    }
    write_dir(pack, target)?;
    fs::create_dir_all(target).map_err(|err| AppError::io(target, &err))?;
    fs::write(&marker, version).map_err(|err| AppError::io(&marker, &err))?;
    Ok(true)
}

fn write_dir(dir: &include_dir::Dir<'_>, target: &Path) -> Result<(), AppError> {
    for entry in dir.entries() {
        let path = target.join(entry.path());
        match entry {
            include_dir::DirEntry::Dir(nested) => {
                fs::create_dir_all(&path).map_err(|err| AppError::io(&path, &err))?;
                write_dir(nested, target)?;
            }
            include_dir::DirEntry::File(file) => {
                if let Some(parent) = path.parent() {
                    fs::create_dir_all(parent).map_err(|err| AppError::io(parent, &err))?;
                }
                fs::write(&path, file.contents()).map_err(|err| AppError::io(&path, &err))?;
            }
        }
    }
    Ok(())
}

/// The `open-storycraft` pack as it was at build time.
#[cfg(target_os = "android")]
pub(crate) const BUNDLED: include_dir::Dir<'static> =
    include_dir::include_dir!("$CARGO_MANIFEST_DIR/../../open-storycraft");

/// Extract the embedded pack into app storage and return its directory.
///
/// # Errors
///
/// Returns [`AppError`] if the app data directory cannot be resolved or the
/// pack cannot be written.
#[cfg(target_os = "android")]
pub(crate) fn ensure_pack(app: &tauri::AppHandle) -> Result<std::path::PathBuf, AppError> {
    use tauri::Manager;

    let target = app
        .path()
        .app_data_dir()
        .map_err(|err| AppError::msg(format!("app data dir: {err}")))?
        .join("skills");
    install(&BUNDLED, &target, env!("CARGO_PKG_VERSION"))?;
    Ok(target)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;

    const PACK: include_dir::Dir<'_> =
        include_dir::include_dir!("$CARGO_MANIFEST_DIR/tests/fixtures/tiny-pack");

    #[test]
    fn extracts_a_pack_once_and_then_leaves_it_alone() {
        let tmp = tempfile::tempdir().unwrap();
        let target = tmp.path().join("skills");
        assert!(install(&PACK, &target, "1.0.0").unwrap());
        assert!(target.join("alpha/SKILL.md").is_file());
        assert!(target.join("beta/notes.md").is_file());
        assert_eq!(
            fs::read_to_string(target.join("beta/notes.md")).unwrap(),
            "ref\n"
        );
        assert!(
            !install(&PACK, &target, "1.0.0").unwrap(),
            "same version: nothing rewritten"
        );
    }

    #[test]
    fn a_new_version_replaces_the_whole_extraction() {
        let tmp = tempfile::tempdir().unwrap();
        let target = tmp.path().join("skills");
        install(&PACK, &target, "1.0.0").unwrap();
        let stale = target.join("removed-skill/SKILL.md");
        fs::create_dir_all(stale.parent().unwrap()).unwrap();
        fs::write(&stale, "gone in the next release").unwrap();
        assert!(install(&PACK, &target, "1.1.0").unwrap());
        assert!(target.join("alpha/SKILL.md").is_file());
        assert!(
            !stale.exists(),
            "skills dropped from the pack must not linger"
        );
    }
}
