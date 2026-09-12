//! Tauri commands. This is the IPC surface Android will reuse.

#![allow(clippy::needless_pass_by_value)]

use std::path::{Component, Path, PathBuf};

use serde::{Deserialize, Serialize};
use storycraft_auth::{DeviceAuth, OAuthConfig, TokenStore};
use storycraft_core::{
    Job, JobStore, Mode, StatusSnapshot, export_zip, find_chapter_prose, infer_chapter,
    unified_diff, validate_preview,
};
use storycraft_llm::CancellationToken;
use storycraft_tools::{BurstinessReport, GenerateOpts};
use tauri::AppHandle;
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_opener::OpenerExt;
use walkdir::WalkDir;

use crate::error::AppError;
use crate::host::{self, jobs_root, load_catalog, resolve_project};
use crate::paths;
use crate::settings::{self, AppSettings};

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

#[tauri::command]
pub fn get_settings(app: AppHandle) -> Result<AppSettings, AppError> {
    settings::load(&app)
}

#[tauri::command]
pub fn save_settings(app: AppHandle, settings: AppSettings) -> Result<(), AppError> {
    settings::save(&app, &settings)
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
    Ok(found
        .into_iter()
        .map(|project| ProjectInfo {
            path: project.path().display().to_string(),
            title: project.title(),
        })
        .collect())
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
    let board = storycraft_core::StatusBoard::inspect(root.as_ref(), mode_from(mode)?, chapter)?;
    Ok(board.snapshot())
}

#[tauri::command]
pub fn list_files(project: String) -> Result<Vec<FileEntry>, AppError> {
    let path = PathBuf::from(&project);
    let root =
        resolve_project(&path)?.ok_or_else(|| AppError::msg("no Wiki folder in that path"))?;
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
    let root =
        resolve_project(&path)?.ok_or_else(|| AppError::msg("no Wiki folder in that path"))?;
    let file = root.path().join(&rel);
    if !file.is_file() {
        return Err(AppError::msg(format!("not a file: {rel}")));
    }
    std::fs::read_to_string(&file).map_err(|err| AppError::io(&file, err))
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
    Ok(dest.map(|path| path.display().to_string()))
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
    let text = std::fs::read_to_string(&file).map_err(|err| AppError::io(&file, err))?;
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

#[tauri::command]
pub async fn pick_folder(app: AppHandle) -> Result<Option<String>, AppError> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    #[cfg(target_os = "android")]
    app.dialog().file().pick_file(move |picked| {
        let _ = tx.send(picked);
    });
    #[cfg(not(target_os = "android"))]
    app.dialog().file().pick_folder(move |picked| {
        let _ = tx.send(picked);
    });
    let picked = rx
        .await
        .map_err(|_| AppError::msg("folder picker cancelled"))?;
    Ok(picked
        .and_then(|path| path.into_path().ok())
        .map(book_root_from))
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
    let _ = app.opener().open_url(url, None::<&str>);
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
    TokenStore::new(paths::oauth_file(&app)?).save(&tokens)?;
    Ok(())
}

#[tauri::command]
pub fn auth_status(app: AppHandle) -> Result<String, AppError> {
    let store = TokenStore::new(paths::oauth_file(&app)?);
    match store.load()? {
        None => Ok("signed-out".into()),
        Some(tokens) if tokens.needs_refresh() => Ok("needs-refresh".into()),
        Some(_) => Ok("signed-in".into()),
    }
}

#[tauri::command]
pub fn auth_logout(app: AppHandle) -> Result<(), AppError> {
    TokenStore::new(paths::oauth_file(&app)?).clear()?;
    Ok(())
}
