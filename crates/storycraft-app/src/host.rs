//! Skill run host. Same loop as the CLI, without clap.

use std::path::{Path, PathBuf};

use storycraft_auth::{DeviceAuth, OAuthConfig};
use storycraft_core::{
    CHUNK_LINES, Catalog, Job, JobStatus, JobStore, Mode, ModelRouter, NewJob, ProjectRoot,
    applied_report_skill, apply_chunk_edit, assemble_converted, base_skill,
    build_apply_chunk_prompt, build_chunk_prompt, build_report_chunk_prompt, build_section_prompt,
    chunk_edit_refused, discover, ensure_requires, find_chapter_prose, finish_model_output,
    infer_chapter, is_chunked_skill, is_overlay, is_report_skill, merge_chunks,
    merge_window_reports, overlay_allowed, prepare_skill, resolve_output_path, split_lines,
    split_source, validate_preview,
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
    // `fragment-hunter:apply` runs the `fragment-hunter` skill folder.
    let manifest = catalog
        .get(base_skill(skill))
        .ok_or_else(|| storycraft_core::Error::SkillNotFound(skill.to_owned()))?;
    if is_overlay(skill) && !overlay_allowed(skill, &settings.enabled_overlays) {
        return Err(storycraft_core::Error::OverlayDisabled(skill.to_owned()).into());
    }
    let project = resolve_project(project_path)?;
    let chapter = match (chapter, project.as_ref()) {
        (Some(n), _) => n,
        (None, Some(project)) => infer_chapter(project),
        (None, None) => 1,
    };
    ensure_requires(project.as_ref(), &manifest.requires, chapter)?;
    let mode = Mode::for_skill(skill);
    let root = jobs_root(project_path)?;
    let store = JobStore::open(&root)?;
    // The converter rewrites the bible where it was found: beside the book, in
    // its Wiki, or one folder down.
    let output_path = match skill {
        "storybible-convert" => storycraft_core::find_storybible_rel(&root),
        _ => resolve_output_path(project.as_ref(), skill, chapter),
    };

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

    let router = ModelRouter::new(
        settings.model.clone(),
        settings.cheap_model.clone(),
        settings.skill_models.clone(),
    );
    let model = router.model_for(skill).to_owned();
    let budget = usize::try_from(settings.token_budget).unwrap_or(0);
    let (packed, prompt) = prepare_skill(
        manifest,
        &answers,
        project.as_ref(),
        chapter,
        output_path.as_deref(),
        budget,
    )?;
    // Read what a windowed run needs before a job exists, so a missing chapter
    // or report fails cleanly instead of leaving a job stuck in `running`.
    let chapter_text = if is_report_skill(skill) {
        Some(read_chapter(project.as_ref(), chapter)?)
    } else {
        None
    };
    let saved_report = match applied_report_skill(skill) {
        Some(base) => Some(read_saved_report(project.as_ref(), base, chapter)?),
        None => None,
    };
    let mut job = store.create(NewJob {
        skill: skill.to_owned(),
        mode,
        chapter,
        answers,
        packed_context_hash: packed.hash,
        provider: settings.provider.clone(),
        model,
        output_path,
    })?;
    snapshot_original(&store, &job)?;
    store.set_status(&mut job, JobStatus::Running)?;
    let _job_notice = android::JobNotice::start(app, &job);

    let stored_key = crate::secrets::SecretStore::open(app)?.get()?;
    let client = build_client(app, settings, &job.model, stored_key).await?;
    let cancel = CancellationToken::new();
    let text = if let Some(chapter_text) = &chapter_text {
        run_report(app, &store, &job, &client, &prompt, chapter_text, &cancel).await
    } else if is_chunked_skill(skill) {
        run_chunked(
            app,
            &store,
            &job,
            &client,
            &prompt,
            saved_report.as_deref(),
            &cancel,
        )
        .await
    } else if skill == "storybible-convert" {
        run_convert(app, &store, &job, &client, &prompt, &cancel).await
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
    let text = &match finish_model_output(&job.skill, text) {
        Ok(text) => text,
        Err(err) => {
            // Keep the raw answer on disk so the user can see what came back.
            store.write_preview(&job.id, text)?;
            store.fail(job, err.to_string())?;
            return Err(err.into());
        }
    };
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
        let text = std::fs::read_to_string(&dest).map_err(|err| AppError::io(&dest, &err))?;
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
    report: Option<&str>,
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
    let mut refused = 0usize;
    for chunk in &chunks {
        let piece = match report {
            Some(report) => build_apply_chunk_prompt(prompt, report, chunk, total_lines),
            None => build_chunk_prompt(prompt, chunk, total_lines),
        };
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
        // Apply runs answer with numbered lines; anything else is window text.
        let numbered = report
            .is_some()
            .then(|| storycraft_core::apply_numbered_edits(chunk, &edited))
            .flatten();
        if let Some(window) = numbered {
            merged.push(window);
        } else {
            if chunk_edit_refused(&chunk.text, &edited) {
                refused = refused.saturating_add(1);
            }
            merged.push(apply_chunk_edit(&chunk.text, &edited));
        }
        let so_far = merge_chunks(&merged);
        store.write_preview(&job.id, &so_far)?;
    }
    if refused == chunks.len() {
        return Err(AppError::msg(
            "the model answered with a report instead of the edited chapter, so nothing was \
             changed; try again or pick another model",
        ));
    }
    if refused > 0 {
        tracing::warn!(
            refused,
            windows = chunks.len(),
            "kept the original text where the model answered with something other than an edit"
        );
    }
    Ok(merge_chunks(&merged))
}

/// Run a report skill window by window and merge the findings into one report.
/// The chapter is only read here; applying the report is a separate run.
async fn run_report(
    app: &AppHandle,
    store: &JobStore,
    job: &Job,
    client: &OpenAiClient,
    prompt: &storycraft_core::Prompt,
    chapter_text: &str,
    cancel: &CancellationToken,
) -> Result<String, AppError> {
    let chunks = split_lines(chapter_text, CHUNK_LINES);
    let total_lines = chapter_text.lines().count();
    let mut windows = Vec::with_capacity(chunks.len());
    for chunk in chunks {
        let piece = build_report_chunk_prompt(prompt, &chunk, total_lines);
        let handle = app.clone();
        let preview_id = job.id.clone();
        let answer = client
            .complete(&piece.system, &piece.user, cancel, |delta| {
                let _ = handle.emit(
                    "job-delta",
                    serde_json::json!({ "id": preview_id, "chunk": delta }),
                );
                Ok(())
            })
            .await?;
        windows.push((chunk, answer));
        store.write_preview(&job.id, &report_text(job, &windows))?;
    }
    Ok(report_text(job, &windows))
}

fn report_text(job: &Job, windows: &[(storycraft_core::LineChunk, String)]) -> String {
    merge_window_reports(&format!("{} — chapter {}", job.skill, job.chapter), windows)
}

/// The chapter prose a report skill reads.
fn read_chapter(project: Option<&ProjectRoot>, chapter: u32) -> Result<String, AppError> {
    let path = project
        .and_then(|project| find_chapter_prose(project, chapter))
        .ok_or_else(|| {
            AppError::msg(format!(
                "no chapter prose for chapter {chapter}; draft it before running an editorial pass"
            ))
        })?;
    std::fs::read_to_string(&path).map_err(|err| AppError::io(&path, &err))
}

/// The report `<skill>:apply` applies: the one `<skill>` saved for this chapter.
fn read_saved_report(
    project: Option<&ProjectRoot>,
    skill: &str,
    chapter: u32,
) -> Result<String, AppError> {
    let missing = || {
        AppError::msg(format!(
            "no saved {skill} report for chapter {chapter}; run {skill} and save its report first"
        ))
    };
    let project = project.ok_or_else(missing)?;
    let rel = resolve_output_path(Some(project), skill, chapter).ok_or_else(missing)?;
    std::fs::read_to_string(project.path().join(rel)).map_err(|_| missing())
}

/// Convert a free-form `storybible.md` section by section, so a bible of any
/// size is read whole and no answer has to carry the entire book.
async fn run_convert(
    app: &AppHandle,
    store: &JobStore,
    job: &Job,
    client: &OpenAiClient,
    prompt: &storycraft_core::Prompt,
    cancel: &CancellationToken,
) -> Result<String, AppError> {
    let source = store.read_original(&job.id)?;
    if source.trim().is_empty() {
        return Err(AppError::msg(
            "no storybible.md in this folder to convert; put your bible there first",
        ));
    }
    let sections = split_source(&source, CONVERT_SECTION_CHARS);
    let mut answers = Vec::with_capacity(sections.len());
    for (i, section) in sections.iter().enumerate() {
        let piece = build_section_prompt(prompt, i, sections.len(), section);
        let handle = app.clone();
        let preview_id = job.id.clone();
        let answer = client
            .complete(&piece.system, &piece.user, cancel, |delta| {
                let _ = handle.emit(
                    "job-delta",
                    serde_json::json!({ "id": preview_id, "chunk": delta }),
                );
                Ok(())
            })
            .await?;
        answers.push(answer);
    }
    assemble_converted(&answers).map_err(AppError::from)
}

/// Source characters per conversion call. Small enough that the converted
/// documents fit in one answer from a modest model.
const CONVERT_SECTION_CHARS: usize = 12_000;

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
            let body = std::fs::read_to_string(&path).map_err(|err| AppError::io(&path, &err))?;
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
        "storybible-import" => storycraft_core::storybible_import_preview(store.root())?,
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

/// Provider client for `settings`, with the stored key or OAuth token attached.
///
/// `stored_key` comes from the secret store; it is ignored for `grok-oauth`,
/// which uses the OAuth token instead.
pub(crate) async fn build_client(
    app: &AppHandle,
    settings: &AppSettings,
    model: &str,
    stored_key: Option<Secret>,
) -> Result<OpenAiClient, AppError> {
    let style: ApiStyle = settings
        .api_style
        .parse()
        .map_err(|err: storycraft_llm::Error| AppError::msg(err.to_string()))?;
    let mut config = ProviderConfig::openai_compat(&settings.base_url, model);
    config.name = settings.provider.clone();
    config.api_style = style;
    config.api_key = match settings.provider.as_str() {
        "grok-oauth" => Some(Secret::new(oauth_access_token(app).await?)),
        _ => stored_key,
    };
    OpenAiClient::new(config).map_err(AppError::from)
}

async fn oauth_access_token(app: &AppHandle) -> Result<String, AppError> {
    let mut tokens = crate::secrets::load_tokens(app)?
        .ok_or_else(|| AppError::msg("not signed in; use Settings → Sign in with xAI"))?;
    if tokens.needs_refresh() {
        let refresh = tokens
            .refresh_token
            .as_deref()
            .ok_or_else(|| AppError::msg("OAuth token expired; sign in again"))?;
        let auth = DeviceAuth::new(OAuthConfig::default())?;
        tokens = auth.refresh(refresh).await?;
        crate::secrets::save_tokens(app, &tokens)?;
    }
    Ok(tokens.access_token)
}
