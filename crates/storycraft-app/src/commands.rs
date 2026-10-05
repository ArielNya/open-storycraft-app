//! Tauri commands. This is the IPC surface Android will reuse.

#![allow(clippy::needless_pass_by_value)]

use std::path::{Component, Path, PathBuf};

use serde::{Deserialize, Serialize};
use storycraft_auth::{DeviceAuth, OAuthConfig};
use storycraft_core::{
    Job, JobStore, Mode, StatusSnapshot, export_zip, find_chapter_prose, infer_chapter,
    unified_diff, validate_preview,
};
use storycraft_llm::CancellationToken;
use storycraft_tools::{BurstinessReport, GenerateOpts};
use tauri::AppHandle;
use tauri_plugin_dialog::DialogExt;
use walkdir::WalkDir;

use crate::error::AppError;
use crate::host::{self, jobs_root, load_catalog, resolve_project};
use crate::paths;
use crate::secrets::SecretStore;
use crate::settings::{self, SettingsDto};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub path: String,
    pub title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub rel: String,
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobDetail {
    pub job: Job,
    pub preview: String,
    pub preview_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillInfo {
    pub name: String,
    pub description: String,
    pub overlay: bool,
    /// Runs on device: no provider, no key, no tokens.
    pub local: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceCodeDto {
    pub user_code: String,
    pub verification_uri: String,
    pub verification_uri_complete: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RunArgs {
    pub project: String,
    pub skill: String,
    #[serde(default)]
    pub answers: Vec<String>,
    pub chapter: Option<u32>,
    #[serde(default)]
    pub commit: bool,
}

fn mode_from(raw: Option<String>) -> Result<Mode, AppError> {
    match raw.as_deref() {
        None => Ok(Mode::Resume),
        Some(s) => s.parse().map_err(AppError::from),
    }
}

fn reject_escape(rel: &str) -> Result<(), AppError> {
    let path = Path::new(rel);
    if path.is_absolute() || path.components().any(|c| matches!(c, Component::ParentDir)) {
        return Err(AppError::msg("path escapes the project"));
    }
    Ok(())
}

/// Settings for the UI: everything except the API key, plus where the key lives.
#[tauri::command]
pub fn get_settings(app: AppHandle) -> Result<SettingsDto, AppError> {
    let store = SecretStore::open(&app)?;
    Ok(SettingsDto::from_settings(settings::load(&app)?, &store))
}

/// Persist settings, and store or forget the API key.
///
/// A non-empty `api_key` replaces the stored one; `clear_api_key` forgets it.
/// Neither ever comes back out: the reply carries `has_api_key` only.
#[tauri::command]
pub fn save_settings(app: AppHandle, settings: SettingsDto) -> Result<SettingsDto, AppError> {
    let store = SecretStore::open(&app)?;
    if settings.clear_api_key {
        store.clear()?;
    }
    if let Some(key) = settings.incoming_key() {
        store.set(key)?;
    }
    settings::save(&app, &settings.settings)?;
    Ok(SettingsDto::from_settings(settings.settings, &store))
}

/// Model ids the provider advertises at `GET {base_url}/models`.
///
/// Takes the settings the user is looking at, not the saved file, so a freshly
/// typed URL and key can be tested before anything is stored. The key is used
/// for the request only; it is never logged and never written here.
#[tauri::command]
pub async fn list_models(app: AppHandle, settings: SettingsDto) -> Result<Vec<String>, AppError> {
    // A key typed into the form is used but not stored; otherwise the stored one.
    let store = SecretStore::open(&app)?;
    let key = match settings.incoming_key() {
        Some(key) => Some(storycraft_llm::Secret::new(key)),
        None => store.get()?,
    };
    let client =
        host::build_client(&app, &settings.settings, &settings.settings.model, key).await?;
    let models = client.list_models().await?;
    tracing::info!(
        count = models.len(),
        base_url = %settings.settings.base_url,
        provider = %settings.settings.provider,
        "listed provider models"
    );
    Ok(models)
}

#[tauri::command]
pub fn discover_projects(start: String) -> Result<Vec<ProjectInfo>, AppError> {
    let path = PathBuf::from(&start);
    if let Some(one) = resolve_project(&path)? {
        return Ok(vec![ProjectInfo {
            path: one.path().display().to_string(),
            title: one.title(),
        }]);
    }
    let found = storycraft_core::discover(&path)?;
    if !found.is_empty() {
        return Ok(found
            .into_iter()
            .map(|project| ProjectInfo {
                path: project.path().display().to_string(),
                title: project.title(),
            })
            .collect());
    }
    // No Wiki anywhere, but a storybible is a book waiting to be unpacked.
    Ok(storybible_project(&path).into_iter().collect())
}

/// The folder a `storybible.md` describes, when no `Wiki/` exists yet.
fn storybible_project(path: &Path) -> Option<ProjectInfo> {
    let bible = storycraft_core::find_storybible(path)?;
    let book = bible.parent()?;
    let title = std::fs::read_to_string(&bible)
        .ok()
        .and_then(|text| storycraft_core::storybible_title(&text));
    Some(ProjectInfo {
        path: book.display().to_string(),
        title,
    })
}

#[tauri::command]
pub fn get_status(
    project: String,
    chapter: Option<u32>,
    mode: Option<String>,
) -> Result<StatusSnapshot, AppError> {
    let path = PathBuf::from(&project);
    let root = resolve_project(&path)?;
    let chapter = match (chapter, root.as_ref()) {
        (Some(n), _) => n,
        (None, Some(project)) => infer_chapter(project),
        (None, None) => 1,
    };
    let board = storycraft_core::StatusBoard::inspect_folder(&path, mode_from(mode)?, chapter)?;
    Ok(board.snapshot())
}

#[tauri::command]
pub fn list_files(project: String) -> Result<Vec<FileEntry>, AppError> {
    let path = PathBuf::from(&project);
    let Some(root) = resolve_project(&path)? else {
        // No Wiki yet: a storybible is the only thing there is to show.
        return Ok(bible_files(&path));
    };
    let mut out = Vec::new();
    for folder in ["Wiki", "Chapters"] {
        let base = root.path().join(folder);
        if !base.exists() {
            continue;
        }
        for entry in WalkDir::new(&base).follow_links(false).max_depth(6) {
            let entry =
                entry.map_err(|err| AppError::msg(format!("walk {}: {err}", base.display())))?;
            let rel = entry
                .path()
                .strip_prefix(root.path())
                .unwrap_or(entry.path());
            let name = rel.to_string_lossy().replace('\\', "/");
            if name.is_empty() {
                continue;
            }
            let kind = if entry.file_type().is_dir() {
                "dir"
            } else {
                "file"
            };
            out.push(FileEntry {
                rel: name,
                kind: kind.to_owned(),
            });
        }
    }
    out.sort_by(|a, b| a.rel.cmp(&b.rel));
    Ok(out)
}

#[tauri::command]
pub fn read_project_file(project: String, rel: String) -> Result<String, AppError> {
    reject_escape(&rel)?;
    let path = PathBuf::from(&project);
    let base = match resolve_project(&path)? {
        Some(root) => root.path().to_path_buf(),
        None => path,
    };
    let file = base.join(&rel);
    if !file.is_file() {
        return Err(AppError::msg(format!("not a file: {rel}")));
    }
    std::fs::read_to_string(&file).map_err(|err| AppError::io(&file, &err))
}

#[tauri::command]
pub fn list_jobs(project: String) -> Result<Vec<Job>, AppError> {
    let root = jobs_root(&PathBuf::from(project))?;
    let store = JobStore::open(&root)?;
    Ok(store.list()?)
}

#[tauri::command]
pub fn get_job(project: String, id: String) -> Result<JobDetail, AppError> {
    let root = jobs_root(&PathBuf::from(project))?;
    let store = JobStore::open(&root)?;
    let job = store.load(&id)?;
    let preview = store.read_preview(&id).unwrap_or_else(|_| String::new());
    Ok(JobDetail {
        preview_path: store.preview_path(&id).display().to_string(),
        preview,
        job,
    })
}

#[tauri::command]
pub fn save_job(project: String, id: String) -> Result<Option<String>, AppError> {
    let root = jobs_root(&PathBuf::from(project))?;
    let store = JobStore::open(&root)?;
    let mut job = store.load(&id)?;
    let preview = store.read_preview(&id)?;
    validate_preview(&job.skill, &preview)?;
    let dest = store.commit(&mut job)?;
    // A bundle writes many files; name them rather than the folder they share.
    if matches!(
        storycraft_core::skill_output(&job.skill),
        storycraft_core::SkillOutput::Bundle | storycraft_core::SkillOutput::Files(_)
    ) {
        let docs = storycraft_core::parse_storybible(&preview)?;
        let names: Vec<&str> = docs.iter().map(|doc| doc.path.as_str()).collect();
        return Ok(Some(match names.as_slice() {
            [one] => (*one).to_owned(),
            [first, second] => format!("{first} and {second}"),
            [first, rest @ ..] => {
                format!("{} files ({first} and {} more)", names.len(), rest.len())
            }
            [] => "no files".to_owned(),
        }));
    }
    // Rebuilt from components: `root.join("Wiki/Style/x.md")` keeps the `/`s on Windows.
    Ok(dest.map(|path| path.components().collect::<PathBuf>().display().to_string()))
}

#[tauri::command]
pub fn job_diff(project: String, id: String) -> Result<String, AppError> {
    let root = jobs_root(&PathBuf::from(project))?;
    let store = JobStore::open(&root)?;
    let original = store.read_original(&id)?;
    let preview = store.read_preview(&id)?;
    Ok(unified_diff(&original, &preview))
}

#[tauri::command]
pub fn export_project(project: String) -> Result<String, AppError> {
    let path = PathBuf::from(&project);
    let root =
        resolve_project(&path)?.ok_or_else(|| AppError::msg("no Wiki folder in that path"))?;
    let dest = root.path().join(".storycraft").join("export.zip");
    export_zip(&root, &dest)?;
    Ok(dest.display().to_string())
}

#[tauri::command]
pub fn burstiness_report(
    project: String,
    chapter: Option<u32>,
) -> Result<BurstinessReport, AppError> {
    let path = PathBuf::from(&project);
    let root =
        resolve_project(&path)?.ok_or_else(|| AppError::msg("no Wiki folder in that path"))?;
    let n = chapter.unwrap_or_else(|| infer_chapter(&root));
    let file = find_chapter_prose(&root, n)
        .ok_or_else(|| AppError::msg(format!("no chapter prose for chapter {n}")))?;
    let text = std::fs::read_to_string(&file).map_err(|err| AppError::io(&file, &err))?;
    Ok(storycraft_tools::measure(&text))
}

#[tauri::command]
pub fn generate_names(
    app: AppHandle,
    kind: String,
    list: String,
    count: Option<u32>,
) -> Result<Vec<String>, AppError> {
    let settings = settings::load(&app)?;
    let skills = paths::skills_dir(&app, settings.skills_dir.as_deref())?;
    let rel = if kind == "town" {
        storycraft_tools::town_list_rel(&list)
    } else {
        storycraft_tools::name_list_rel(&list)
    };
    let path = skills.join(rel);
    storycraft_tools::generate_from_path(
        &path,
        &GenerateOpts {
            count: usize::try_from(count.unwrap_or(10)).unwrap_or(10),
            ..GenerateOpts::default()
        },
    )
    .map_err(|err| AppError::msg(err.to_string()))
}

#[tauri::command]
pub fn reject_job(project: String, id: String) -> Result<(), AppError> {
    let root = jobs_root(&PathBuf::from(project))?;
    let store = JobStore::open(&root)?;
    let mut job = store.load(&id)?;
    store.reject(&mut job)?;
    Ok(())
}

#[tauri::command]
pub fn list_skills(app: AppHandle) -> Result<Vec<SkillInfo>, AppError> {
    let settings = settings::load(&app)?;
    let catalog = load_catalog(&app, settings.skills_dir.as_deref())?;
    Ok(catalog
        .iter()
        .map(|skill| SkillInfo {
            overlay: storycraft_core::is_overlay(&skill.name),
            local: storycraft_tools::is_local_tool(&skill.name),
            name: skill.name.clone(),
            description: skill.description.clone(),
        })
        .collect())
}

#[tauri::command]
pub async fn run_skill(app: AppHandle, args: RunArgs) -> Result<Job, AppError> {
    let settings = settings::load(&app)?;
    host::execute_run(
        &app,
        &settings,
        Path::new(&args.project),
        &args.skill,
        args.answers,
        args.chapter,
        args.commit,
    )
    .await
}

/// Open the platform folder picker, starting at `start` when it exists.
///
/// Without a directory the GTK chooser opens on Recents, which is useless for a
/// book kept deep in a home directory. The Android SAF picker takes no starting
/// directory, so `start` is desktop-only.
#[tauri::command]
pub async fn pick_folder(
    app: AppHandle,
    #[cfg_attr(target_os = "android", allow(unused_variables))] start: Option<String>,
) -> Result<Option<String>, AppError> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    let dialog = app.dialog().file();
    #[cfg(not(target_os = "android"))]
    let dialog = match start.filter(|dir| Path::new(dir).is_dir()) {
        Some(dir) => dialog.set_directory(dir),
        None => dialog,
    };
    #[cfg(target_os = "android")]
    dialog.pick_file(move |picked| {
        let _ = tx.send(picked);
    });
    #[cfg(not(target_os = "android"))]
    dialog.pick_folder(move |picked| {
        let _ = tx.send(picked);
    });
    let picked = rx
        .await
        .map_err(|_| AppError::msg("folder picker cancelled"))?;
    Ok(picked
        .and_then(|path| path.into_path().ok())
        .map(book_root_from))
}

/// Markdown files in a folder that has no `Wiki/` yet, storybible first.
fn bible_files(path: &Path) -> Vec<FileEntry> {
    let mut out = Vec::new();
    for entry in WalkDir::new(path)
        .follow_links(false)
        .max_depth(2)
        .into_iter()
        .filter_entry(|entry| entry.file_name() != ".storycraft")
    {
        let Ok(entry) = entry else { continue };
        if !entry.file_type().is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_ascii_lowercase();
        if !name.ends_with(".md") {
            continue;
        }
        let rel = entry
            .path()
            .strip_prefix(path)
            .unwrap_or(entry.path())
            .to_string_lossy()
            .replace('\\', "/");
        out.push(FileEntry {
            rel,
            kind: "file".to_owned(),
        });
    }
    out.sort_by(|a, b| {
        let key = |rel: &str| (!rel.ends_with("storybible.md"), rel.to_owned());
        key(&a.rel).cmp(&key(&b.rel))
    });
    out
}

fn book_root_from(path: PathBuf) -> String {
    let mut cur = if path.is_file() {
        path.parent().map(Path::to_path_buf).unwrap_or(path)
    } else {
        path
    };
    let mut probe = cur.clone();
    for _ in 0..8 {
        if probe.join("Wiki").is_dir() {
            cur = probe;
            break;
        }
        match probe.parent() {
            Some(parent) => probe = parent.to_path_buf(),
            None => break,
        }
    }
    cur.display().to_string()
}

/// Start device-code login and open the verification URL.
#[tauri::command]
pub async fn auth_login(
    app: AppHandle,
    state: tauri::State<'_, crate::AppState>,
) -> Result<DeviceCodeDto, AppError> {
    let auth = DeviceAuth::new(OAuthConfig::default())?;
    let pending = auth.request_code().await?;
    let dto = DeviceCodeDto {
        user_code: pending.user_code.clone(),
        verification_uri: pending.verification_uri.clone(),
        verification_uri_complete: pending.verification_uri_complete.clone(),
    };
    let url = pending
        .verification_uri_complete
        .as_deref()
        .unwrap_or(&pending.verification_uri);
    crate::android::open_auth_url(&app, url);
    let mut slot = state
        .pending_device
        .lock()
        .map_err(|_| AppError::msg("auth lock poisoned"))?;
    *slot = Some(pending);
    Ok(dto)
}

#[tauri::command]
pub async fn auth_poll(
    app: AppHandle,
    state: tauri::State<'_, crate::AppState>,
) -> Result<(), AppError> {
    let pending = {
        let mut slot = state
            .pending_device
            .lock()
            .map_err(|_| AppError::msg("auth lock poisoned"))?;
        slot.take()
    };
    let Some(pending) = pending else {
        return Err(AppError::msg("no device login in progress"));
    };
    let auth = DeviceAuth::new(OAuthConfig::default())?;
    let tokens = auth.poll_token(&pending, &CancellationToken::new()).await?;
    crate::secrets::save_tokens(&app, &tokens)
}

#[tauri::command]
pub fn auth_status(app: AppHandle) -> Result<String, AppError> {
    match crate::secrets::load_tokens(&app)? {
        None => Ok("signed-out".into()),
        Some(tokens) if tokens.needs_refresh() => Ok("needs-refresh".into()),
        Some(_) => Ok("signed-in".into()),
    }
}

#[tauri::command]
pub fn auth_logout(app: AppHandle) -> Result<(), AppError> {
    SecretStore::open_oauth(&app)?.clear()
}
