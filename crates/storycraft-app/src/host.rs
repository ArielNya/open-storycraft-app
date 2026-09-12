//! Skill run host. Same loop as the CLI, without clap.

use std::path::{Path, PathBuf};

use storycraft_auth::{DeviceAuth, OAuthConfig, TokenStore};
use storycraft_core::{
    CHUNK_LINES, Catalog, Job, JobStatus, JobStore, Mode, NewJob, ProjectRoot, apply_chunk_edit,
    build_chunk_prompt, discover, ensure_requires, find_chapter_prose, infer_chapter,
    is_chunked_skill, merge_chunks, prepare_skill, resolve_output_path, split_lines,
    validate_preview,
};
use storycraft_llm::{ApiStyle, CancellationToken, OpenAiClient, ProviderConfig, Secret};
use storycraft_tools::{GenerateOpts, is_local_tool};
use tauri::{AppHandle, Emitter};

use crate::android;
use crate::error::AppError;
use crate::paths;
use crate::settings::AppSettings;

/// Resolve exactly one project, or none.
pub(crate) fn resolve_project(path: &Path) -> Result<Option<ProjectRoot>, AppError> {
    if !path.exists() {
        return Ok(None);
    }
    let mut found = discover(path)?;
    match found.len() {
        0 => Ok(None),
        1 => Ok(found.pop()),
        _ => Err(storycraft_core::Error::MultipleProjects { projects: found }.into()),
    }
}

pub(crate) fn jobs_root(path: &Path) -> Result<PathBuf, AppError> {
    match resolve_project(path)? {
        Some(project) => Ok(project.path().to_path_buf()),
        None => Ok(path.to_path_buf()),
    }
}

pub(crate) fn load_catalog(
    app: &AppHandle,
    skills_override: Option<&str>,
) -> Result<Catalog, AppError> {
    let dir = paths::skills_dir(app, skills_override)?;
    Catalog::load(&dir).map_err(AppError::from)
}

/// Run one skill and stream deltas to the webview as `job-delta`.
pub(crate) async fn execute_run(
    app: &AppHandle,
    settings: &AppSettings,
    project_path: &Path,
    skill: &str,
    answers: Vec<String>,
    chapter: Option<u32>,
    commit: bool,
) -> Result<Job, AppError> {
    let catalog = load_catalog(app, settings.skills_dir.as_deref())?;
    let manifest = catalog
        .get(skill)
        .ok_or_else(|| storycraft_core::Error::SkillNotFound(skill.to_owned()))?;
    let project = resolve_project(project_path)?;
    let chapter = match (chapter, project.as_ref()) {
        (Some(n), _) => n,
        (None, Some(project)) => infer_chapter(project),
        (None, None) => 1,
    };
    ensure_requires(project.as_ref(), &manifest.requires, chapter)?;
    let output_path = resolve_output_path(project.as_ref(), skill, chapter);
    let mode = Mode::for_skill(skill);
    let root = jobs_root(project_path)?;
    let store = JobStore::open(&root)?;

    if is_local_tool(skill) {
        return run_local(
            app,
            settings,
            &store,
            project.as_ref(),
            skill,
            &answers,
            chapter,
            output_path,
            mode,
            commit,
        );
    }

    let (packed, prompt) = prepare_skill(
        manifest,
        &answers,
        project.as_ref(),
        chapter,
        output_path.as_deref(),
    )?;
    let mut job = store.create(NewJob {
        skill: skill.to_owned(),
        mode,
        chapter,
        answers,
        packed_context_hash: packed.hash,
        provider: settings.provider.clone(),
        model: settings.model.clone(),
        output_path,
    })?;
    snapshot_original(&store, &job)?;
    store.set_status(&mut job, JobStatus::Running)?;
    let _job_notice = android::JobNotice::start(app, &job);

    let client = build_client(app, settings).await?;
    let cancel = CancellationToken::new();
    let text = if is_chunked_skill(skill) {
        run_chunked(app, &store, &job, &client, &prompt, &cancel).await
    } else {
        let preview_id = job.id.clone();
        let handle = app.clone();
        client
            .complete(&prompt.system, &prompt.user, &cancel, |delta| {
                store
                    .append_preview(&preview_id, delta)
                    .map_err(|err| storycraft_llm::Error::InvalidPayload(err.to_string()))?;
                let _ = handle.emit(
                    "job-delta",
                    serde_json::json!({ "id": preview_id, "chunk": delta }),
                );
                Ok(())
            })
            .await
            .map_err(AppError::from)
    };

    match text {
        Ok(text) => finish_preview(&store, &mut job, &text, commit),
        Err(err) => {
            store.fail(&mut job, err.to_string())?;
            Err(AppError::msg(format!(
                "{err}; provider '{}' at {}",
                settings.provider, settings.base_url
            )))
        }
    }
}

fn finish_preview(
    store: &JobStore,
    job: &mut Job,
    text: &str,
    commit: bool,
) -> Result<Job, AppError> {
    store.write_preview(&job.id, text)?;
    if let Err(err) = validate_preview(&job.skill, text) {
        store.fail(job, err.to_string())?;
        return Err(err.into());
    }
    store.set_status(job, JobStatus::NeedsConfirm)?;
    if commit {
        let _ = store.commit(job)?;
    }
    Ok(job.clone())
}

fn snapshot_original(store: &JobStore, job: &Job) -> Result<(), AppError> {
    let Some(dest) = store.output_abs(job) else {
        return Ok(());
    };
    if dest.is_file() {
        let text = std::fs::read_to_string(&dest).map_err(|err| AppError::io(&dest, err))?;
        store.write_original(&job.id, &text)?;
    }
    Ok(())
}

async fn run_chunked(
    app: &AppHandle,
    store: &JobStore,
    job: &Job,
    client: &OpenAiClient,
    prompt: &storycraft_core::Prompt,
    cancel: &CancellationToken,
) -> Result<String, AppError> {
    let original = store.read_original(&job.id)?;
    if original.is_empty() {
        return Err(AppError::msg(format!(
            "no chapter prose for chapter {} (chunked skills rewrite an existing file)",
            job.chapter
        )));
    }
    let chunks = split_lines(&original, CHUNK_LINES);
    let total_lines = original.lines().count();
    let mut merged = Vec::with_capacity(chunks.len());
    for chunk in &chunks {
        let piece = build_chunk_prompt(prompt, chunk, total_lines);
        let handle = app.clone();
        let preview_id = job.id.clone();
        let edited = client
            .complete(&piece.system, &piece.user, cancel, |delta| {
                let _ = handle.emit(
                    "job-delta",
                    serde_json::json!({ "id": preview_id, "chunk": delta }),
                );
                Ok(())
            })
            .await?;
        merged.push(apply_chunk_edit(&chunk.text, &edited));
        let so_far = merge_chunks(&merged);
        store.write_preview(&job.id, &so_far)?;
    }
    Ok(merge_chunks(&merged))
}

#[allow(clippy::too_many_arguments)]
fn run_local(
    app: &AppHandle,
    settings: &AppSettings,
    store: &JobStore,
    project: Option<&ProjectRoot>,
    skill: &str,
    answers: &[String],
    chapter: u32,
    output_path: Option<String>,
    mode: Mode,
    commit: bool,
) -> Result<Job, AppError> {
    let text = match skill {
        "burstiness-check" => {
            let project =
                project.ok_or_else(|| AppError::msg("burstiness-check needs a Wiki folder"))?;
            let path = find_chapter_prose(project, chapter)
                .ok_or_else(|| AppError::msg(format!("no chapter prose for chapter {chapter}")))?;
            let body = std::fs::read_to_string(&path).map_err(|err| AppError::io(&path, err))?;
            let report = storycraft_tools::measure(&body);
            format!(
                "# Burstiness\n\n- sentences: {}\n- paragraphs: {}\n- sentence variance: {} (mean {}, stdev {})\n- dialogue ratio: {}\n- interiority: {} ({})\n- longest opener run: {}\n",
                report.sentence_count,
                report.paragraph_count,
                report.sentence_length_variance_bucket,
                report.sentence_length_mean,
                report.sentence_length_stdev,
                report.dialogue_word_ratio,
                report.interiority_risk_level,
                report.interiority_risk_score,
                report.longest_same_opener_run
            )
        }
        "name-generator" | "town-generator" => {
            let skills = paths::skills_dir(app, settings.skills_dir.as_deref())?;
            let list_name = answers.first().map(String::as_str).unwrap_or("fantasy");
            let count = answers
                .get(1)
                .and_then(|s| s.parse().ok())
                .unwrap_or(10usize);
            let rel = if skill == "name-generator" {
                storycraft_tools::name_list_rel(list_name)
            } else {
                storycraft_tools::town_list_rel(list_name)
            };
            let path = skills.join(rel);
            let names = storycraft_tools::generate_from_path(
                &path,
                &GenerateOpts {
                    count,
                    ..GenerateOpts::default()
                },
            )
            .map_err(|err| AppError::msg(err.to_string()))?;
            let mut out = String::from("# Names\n\n");
            for name in names {
                out.push_str("- ");
                out.push_str(&name);
                out.push('\n');
            }
            out
        }
        other => return Err(AppError::msg(format!("not a local tool: {other}"))),
    };
    let mut job = store.create(NewJob {
        skill: skill.to_owned(),
        mode,
        chapter,
        answers: answers.to_vec(),
        packed_context_hash: "local".into(),
        provider: "local".into(),
        model: "none".into(),
        output_path,
    })?;
    store.write_preview(&job.id, &text)?;
    store.set_status(&mut job, JobStatus::NeedsConfirm)?;
    if commit {
        let _ = store.commit(&mut job)?;
    }
    Ok(job)
}

async fn build_client(app: &AppHandle, settings: &AppSettings) -> Result<OpenAiClient, AppError> {
    let style: ApiStyle = settings
        .api_style
        .parse()
        .map_err(|err: storycraft_llm::Error| AppError::msg(err.to_string()))?;
    let mut config = ProviderConfig::openai_compat(&settings.base_url, &settings.model);
    config.name = settings.provider.clone();
    config.api_style = style;
    config.api_key = match settings.provider.as_str() {
        "grok-oauth" => Some(Secret::new(oauth_access_token(app).await?)),
        _ => settings.api_key.as_ref().map(Secret::new),
    };
    OpenAiClient::new(config).map_err(AppError::from)
}

async fn oauth_access_token(app: &AppHandle) -> Result<String, AppError> {
    let store = TokenStore::new(paths::oauth_file(app)?);
    let mut tokens = store
        .load()?
        .ok_or_else(|| AppError::msg("not signed in; use Settings → Sign in with xAI"))?;
    if tokens.needs_refresh() {
        let refresh = tokens
            .refresh_token
            .as_deref()
            .ok_or_else(|| AppError::msg("OAuth token expired; sign in again"))?;
        let auth = DeviceAuth::new(OAuthConfig::default())?;
        tokens = auth.refresh(refresh).await?;
        store.save(&tokens)?;
    }
    Ok(tokens.access_token)
}
