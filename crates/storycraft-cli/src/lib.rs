//! Host logic for the `storycraft` CLI.

#![allow(missing_docs)]
#![allow(clippy::print_stdout)]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anyhow::{Context, anyhow};
use clap::{Args, Parser, Subcommand};
use storycraft_auth::{
    DEFAULT_AUTH_BASE, DEFAULT_CLIENT_ID, DEFAULT_SCOPES, DeviceAuth, OAuthConfig, TokenStore,
    default_token_path,
};
use storycraft_core::{
    CHUNK_LINES, Catalog, DEFAULT_TOTAL_CHARS, Error, Job, JobStatus, JobStore, Mode, ModelRouter,
    NewJob, PackKind, ProjectRoot, StatusBoard, apply_chunk_edit, discover, ensure_requires,
    export_zip, find_chapter_prose, find_skills_dir, format_project_list, infer_chapter,
    is_chunked_skill, is_overlay, merge_chunks, pack_skill, prepare_skill, resolve_output_path,
    split_lines, unified_diff, validate_preview,
};
use storycraft_llm::{ApiStyle, CancellationToken, OpenAiClient, ProviderConfig, Secret};
use storycraft_tools::{GenerateOpts, is_local_tool};

/// Open Storycraft command-line host.
#[derive(Parser)]
#[command(
    name = "storycraft",
    version,
    about = "Open Storycraft local writing studio"
)]
pub struct Cli {
    /// Skill library directory (the `open-storycraft` pack).
    #[arg(long, global = true, env = "STORYCRAFT_SKILLS")]
    pub skills_dir: Option<PathBuf>,
    #[command(subcommand)]
    pub command: Command,
}

/// Book directory: `--path` / `-C`, or a trailing positional PATH.
#[derive(Args, Clone)]
pub struct ProjectPath {
    /// Book directory (the folder that contains Wiki/, or a parent to search)
    #[arg(short = 'C', long = "path")]
    path: Option<PathBuf>,
    /// Same as `--path`
    #[arg(value_name = "PATH")]
    dir: Option<PathBuf>,
}

impl ProjectPath {
    fn resolve(&self) -> PathBuf {
        self.path
            .clone()
            .or_else(|| self.dir.clone())
            .unwrap_or_else(|| PathBuf::from("."))
    }
}

/// Subcommands.
#[derive(Subcommand)]
pub enum Command {
    /// Print the status board from files on disk
    Status {
        #[command(flatten)]
        project: ProjectPath,
        /// Chapter used for scenes / psych / chapters slots
        #[arg(long)]
        chapter: Option<u32>,
        /// Orchestrator mode
        #[arg(long, default_value = "resume", value_parser = parse_mode)]
        mode: Mode,
    },
    /// Print the next skill and why
    Next {
        #[command(flatten)]
        project: ProjectPath,
        #[arg(long)]
        chapter: Option<u32>,
        #[arg(long, default_value = "resume", value_parser = parse_mode)]
        mode: Mode,
    },
    /// List vendored skills (frontmatter index only)
    Skills {
        /// Include optional overlays (`ao3-*`, `anti-slop-editor`)
        #[arg(long)]
        overlays: bool,
    },
    /// List files a skill pack would include (linked refs, not the whole folder)
    Pack {
        /// Skill folder name, e.g. fiction-genre
        skill: String,
    },
    /// Run one skill against a provider; writes a preview, does not save unless `--commit`
    Run {
        /// Skill folder name
        skill: String,
        #[command(flatten)]
        project: ProjectPath,
        /// Answer to a skill question, repeatable, in order
        #[arg(long = "answer")]
        answers: Vec<String>,
        #[arg(long)]
        chapter: Option<u32>,
        /// Provider profile name stored on the job
        #[arg(long, default_value = "openai-compat")]
        provider: String,
        /// OpenAI-compatible base URL, including `/v1`
        #[arg(long, default_value = "http://127.0.0.1:1234/v1")]
        base_url: String,
        #[arg(long, env = "STORYCRAFT_API_KEY", hide_env_values = true)]
        api_key: Option<String>,
        #[arg(long, default_value = "chat_completions", value_parser = parse_api_style)]
        api_style: ApiStyle,
        #[arg(long, default_value = "grok-4.6")]
        model: String,
        /// Cheaper model for editorial / kill-pass skills
        #[arg(long)]
        cheap_model: Option<String>,
        /// Per-skill override, repeatable (`kill-crutch=grok-3-mini`)
        #[arg(long = "route", value_name = "SKILL=MODEL", value_parser = parse_route)]
        routes: Vec<(String, String)>,
        /// Packer char budget (0 = default)
        #[arg(long, default_value_t = DEFAULT_TOTAL_CHARS)]
        budget: usize,
        /// Copy the preview to the Wiki path after validation
        #[arg(long)]
        commit: bool,
        #[arg(long, env = "STORYCRAFT_AUTH_FILE")]
        auth_file: Option<PathBuf>,
    },
    /// List jobs under a project
    Jobs {
        #[command(flatten)]
        project: ProjectPath,
    },
    /// Atomic-write a `needs_confirm` preview into the Wiki
    Save {
        job_id: String,
        #[command(flatten)]
        project: ProjectPath,
    },
    /// Reject a preview without touching the Wiki
    Reject {
        job_id: String,
        #[command(flatten)]
        project: ProjectPath,
    },
    /// xAI community OAuth
    Auth {
        #[command(subcommand)]
        command: AuthCommand,
    },
    /// Unified diff of a job's original vs preview
    Diff {
        job_id: String,
        #[command(flatten)]
        project: ProjectPath,
    },
    /// Zip Wiki/ and Chapters/ (jobs stay out)
    Export {
        #[command(flatten)]
        project: ProjectPath,
        /// Destination zip path
        #[arg(long, short = 'o')]
        out: Option<PathBuf>,
    },
    /// Markov names from a culture list (no LLM)
    Namegen {
        /// Culture file stem (`fantasy`, `anglo`, …)
        #[arg(long, default_value = "fantasy")]
        culture: String,
        #[arg(long, default_value_t = 10)]
        count: usize,
        #[arg(long)]
        seed: Option<u64>,
        /// Explicit `data/*.txt` path (overrides `--culture`)
        #[arg(long)]
        list: Option<PathBuf>,
    },
    /// Markov town names from a list (no LLM)
    Town {
        #[arg(long, default_value = "fantasy")]
        list_name: String,
        #[arg(long, default_value_t = 10)]
        count: usize,
        #[arg(long)]
        seed: Option<u64>,
        #[arg(long)]
        list: Option<PathBuf>,
    },
    /// Burstiness report for a chapter (no LLM)
    Burstiness {
        #[command(flatten)]
        project: ProjectPath,
        #[arg(long)]
        chapter: Option<u32>,
        /// Explicit chapter file (overrides `--chapter`)
        #[arg(long)]
        file: Option<PathBuf>,
    },
}

/// OAuth subcommands.
#[derive(Subcommand)]
pub enum AuthCommand {
    /// Device-code sign-in (community OAuth, not an xAI partnership)
    Login {
        #[arg(long, default_value = DEFAULT_AUTH_BASE)]
        auth_url: String,
        #[arg(long, default_value = DEFAULT_CLIENT_ID)]
        client_id: String,
        #[arg(long, default_value = DEFAULT_SCOPES)]
        scopes: String,
        #[arg(long, env = "STORYCRAFT_AUTH_FILE")]
        auth_file: Option<PathBuf>,
    },
    /// Show whether a token file exists (never prints the token)
    Status {
        #[arg(long, env = "STORYCRAFT_AUTH_FILE")]
        auth_file: Option<PathBuf>,
    },
    /// Delete the token file
    Logout {
        #[arg(long, env = "STORYCRAFT_AUTH_FILE")]
        auth_file: Option<PathBuf>,
    },
}

fn parse_mode(s: &str) -> Result<Mode, String> {
    s.parse::<Mode>().map_err(|err| err.to_string())
}

fn parse_api_style(s: &str) -> Result<ApiStyle, String> {
    s.parse::<ApiStyle>().map_err(|err| err.to_string())
}

fn parse_route(s: &str) -> Result<(String, String), String> {
    let (skill, model) = s
        .split_once('=')
        .ok_or_else(|| "expected SKILL=MODEL".to_owned())?;
    let skill = skill.trim();
    let model = model.trim();
    if skill.is_empty() || model.is_empty() {
        return Err("expected SKILL=MODEL".into());
    }
    Ok((skill.to_owned(), model.to_owned()))
}

/// Parse argv and execute.
///
/// # Errors
///
/// Returns user-facing failures (missing Wiki, provider errors, validation).
pub async fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Status {
            project,
            chapter,
            mode,
        } => {
            let board = build_board(&project.resolve(), chapter, mode)?;
            println!("{board}");
        }
        Command::Next {
            project,
            chapter,
            mode,
        } => {
            let board = build_board(&project.resolve(), chapter, mode)?;
            match &board.next().skill {
                Some(skill) => println!("{skill} — {}", board.next().why),
                None => println!("— — {}", board.next().why),
            }
        }
        Command::Skills { overlays } => {
            let catalog = load_catalog(cli.skills_dir.as_deref())?;
            let enabled = if overlays {
                vec!["*".to_owned()]
            } else {
                Vec::new()
            };
            let mut n = 0usize;
            for skill in catalog.iter_visible(&enabled) {
                if is_overlay(&skill.name) {
                    println!("{} [overlay] — {}", skill.name, skill.description);
                } else {
                    println!("{} — {}", skill.name, skill.description);
                }
                n = n.saturating_add(1);
            }
            eprintln!("{n} skills");
        }
        Command::Pack { ref skill } => {
            let catalog = load_catalog(cli.skills_dir.as_deref())?;
            let manifest = catalog
                .get(skill)
                .ok_or_else(|| Error::SkillNotFound(skill.clone()))?;
            let pack = pack_skill(manifest);
            for file in &pack.files {
                let label = match file.kind {
                    PackKind::Skill => "skill",
                    PackKind::Reference => "ref",
                    PackKind::Wiki => "wiki",
                    PackKind::WikiTail => "tail",
                };
                println!("{label}  {}", file.relative);
            }
            eprintln!(
                "{} files ({} linked refs)",
                pack.files.len(),
                pack.reference_count()
            );
        }
        Command::Run {
            skill,
            project,
            answers,
            chapter,
            provider,
            base_url,
            api_key,
            api_style,
            model,
            cheap_model,
            routes,
            budget,
            commit,
            auth_file,
        } => {
            let path = project.resolve();
            let job = execute_run(RunRequest {
                skills_dir: cli.skills_dir.clone(),
                skill,
                path: path.clone(),
                answers,
                chapter,
                provider,
                base_url,
                api_key,
                api_style,
                model,
                cheap_model,
                routes: routes.into_iter().collect(),
                budget,
                commit,
                auth_file,
            })
            .await?;
            let root = jobs_root(&path)?;
            println!("job {}  {}", job.id, job.status);
            println!(
                "preview {}",
                root.join(".storycraft/jobs")
                    .join(format!("{}.preview.md", job.id))
                    .display()
            );
            if job.status == JobStatus::NeedsConfirm {
                println!(
                    "not in the Wiki yet. keep it with: storycraft save {} --path {}",
                    job.id,
                    path.display()
                );
            }
        }
        Command::Jobs { project } => {
            let root = jobs_root(&project.resolve())?;
            let store = JobStore::open(&root)?;
            for job in store.list()? {
                println!("{}  {}  {}", job.id, job.skill, job.status);
            }
        }
        Command::Save { job_id, project } => {
            let root = jobs_root(&project.resolve())?;
            let store = JobStore::open(&root)?;
            let mut job = store.load(&job_id)?;
            let preview = store.read_preview(&job.id)?;
            validate_preview(&job.skill, &preview)?;
            match store.commit(&mut job)? {
                Some(dest) => println!("saved {}", dest.display()),
                None => println!("accepted preview; skill writes no Wiki file"),
            }
        }
        Command::Reject { job_id, project } => {
            let root = jobs_root(&project.resolve())?;
            let store = JobStore::open(&root)?;
            let mut job = store.load(&job_id)?;
            store.reject(&mut job)?;
            println!("rejected {}", job.id);
        }
        Command::Auth { command } => match command {
            AuthCommand::Login {
                auth_url,
                client_id,
                scopes,
                auth_file,
            } => auth_login(auth_url, client_id, scopes, auth_file).await?,
            AuthCommand::Status { auth_file } => {
                let store = TokenStore::new(auth_file.unwrap_or_else(default_token_path));
                match store.load()? {
                    None => println!("signed out ({})", store.path().display()),
                    Some(tokens) => {
                        let stale = if tokens.needs_refresh() {
                            "needs refresh"
                        } else {
                            "fresh"
                        };
                        println!("signed in ({stale}); tokens at {}", store.path().display());
                    }
                }
            }
            AuthCommand::Logout { auth_file } => {
                let store = TokenStore::new(auth_file.unwrap_or_else(default_token_path));
                store.clear()?;
                println!("signed out");
            }
        },
        Command::Diff { job_id, project } => {
            let root = jobs_root(&project.resolve())?;
            let store = JobStore::open(&root)?;
            let original = store.read_original(&job_id)?;
            let preview = store.read_preview(&job_id)?;
            let diff = unified_diff(&original, &preview);
            if diff.is_empty() {
                println!("no changes");
            } else {
                print!("{diff}");
            }
        }
        Command::Export { project, out } => {
            let path = project.resolve();
            let project = resolve_project(&path)?
                .ok_or_else(|| anyhow!("no Wiki folder under {}", path.display()))?;
            let dest = match out {
                Some(p) => p,
                None => PathBuf::from(format!(
                    "{}.zip",
                    project
                        .path()
                        .file_name()
                        .map(|n| n.to_string_lossy().into_owned())
                        .unwrap_or_else(|| "storycraft-export".into())
                )),
            };
            export_zip(&project, &dest)?;
            println!("exported {}", dest.display());
        }
        Command::Namegen {
            culture,
            count,
            seed,
            list,
        } => {
            let path = match list {
                Some(p) => p,
                None => resolve_skills_dir(cli.skills_dir.as_deref())?
                    .join(storycraft_tools::name_list_rel(&culture)),
            };
            let names = storycraft_tools::generate_from_path(
                &path,
                &GenerateOpts {
                    count,
                    seed,
                    ..GenerateOpts::default()
                },
            )?;
            for name in names {
                println!("{name}");
            }
        }
        Command::Town {
            list_name,
            count,
            seed,
            list,
        } => {
            let path = match list {
                Some(p) => p,
                None => resolve_skills_dir(cli.skills_dir.as_deref())?
                    .join(storycraft_tools::town_list_rel(&list_name)),
            };
            let names = storycraft_tools::generate_from_path(
                &path,
                &GenerateOpts {
                    count,
                    seed,
                    ..GenerateOpts::default()
                },
            )?;
            for name in names {
                println!("{name}");
            }
        }
        Command::Burstiness {
            project,
            chapter,
            file,
        } => {
            let text = match file {
                Some(path) => std::fs::read_to_string(&path)
                    .with_context(|| format!("reading {}", path.display()))?,
                None => {
                    let root = resolve_project(&project.resolve())?
                        .ok_or_else(|| anyhow!("no Wiki folder"))?;
                    let n = chapter.unwrap_or_else(|| infer_chapter(&root));
                    let path = find_chapter_prose(&root, n)
                        .ok_or_else(|| anyhow!("no chapter prose for chapter {n}"))?;
                    std::fs::read_to_string(&path)
                        .with_context(|| format!("reading {}", path.display()))?
                }
            };
            let report = storycraft_tools::measure(&text);
            println!("sentences {}", report.sentence_count);
            println!("paragraphs {}", report.paragraph_count);
            println!(
                "sentence variance {} (mean {} stdev {})",
                report.sentence_length_variance_bucket,
                report.sentence_length_mean,
                report.sentence_length_stdev
            );
            println!("dialogue ratio {}", report.dialogue_word_ratio);
            println!(
                "interiority {} ({})",
                report.interiority_risk_level, report.interiority_risk_score
            );
            if let Some(opener) = report.top_openers.first() {
                println!("top opener {} {}%", opener.word, opener.pct);
            }
        }
    }
    Ok(())
}

/// Arguments for one skill run. Public so tests can skip clap.
pub struct RunRequest {
    pub skills_dir: Option<PathBuf>,
    pub skill: String,
    pub path: PathBuf,
    pub answers: Vec<String>,
    pub chapter: Option<u32>,
    pub provider: String,
    pub base_url: String,
    pub api_key: Option<String>,
    pub api_style: ApiStyle,
    pub model: String,
    pub cheap_model: Option<String>,
    pub routes: BTreeMap<String, String>,
    pub budget: usize,
    pub commit: bool,
    pub auth_file: Option<PathBuf>,
}

/// Run one skill, stream into a preview, optionally commit.
///
/// # Errors
///
/// Returns catalog, require, provider, or validation failures.
pub async fn execute_run(req: RunRequest) -> anyhow::Result<Job> {
    let catalog = load_catalog(req.skills_dir.as_deref())?;
    // `fragment-hunter:apply` runs the `fragment-hunter` skill folder.
    let manifest = catalog
        .get(storycraft_core::base_skill(&req.skill))
        .ok_or_else(|| Error::SkillNotFound(req.skill.clone()))?;
    let project = resolve_project(&req.path)?;
    let chapter = match (req.chapter, project.as_ref()) {
        (Some(n), _) => n,
        (None, Some(project)) => infer_chapter(project),
        (None, None) => 1,
    };
    ensure_requires(project.as_ref(), &manifest.requires, chapter)?;
    let mode = Mode::for_skill(&req.skill);
    let root = jobs_root(&req.path)?;
    let store = JobStore::open(&root)?;
    // The converter rewrites the bible where it was found (see the app host).
    let output_path = match req.skill.as_str() {
        "storybible-convert" => storycraft_core::find_storybible_rel(&root),
        _ => resolve_output_path(project.as_ref(), &req.skill, chapter),
    };

    if is_local_tool(&req.skill) {
        return run_local(&req, &store, project.as_ref(), chapter, output_path, mode).await;
    }

    let router = ModelRouter::new(
        req.model.clone(),
        req.cheap_model.clone(),
        req.routes.clone(),
    );
    let model = router.model_for(&req.skill).to_owned();
    let (packed, prompt) = prepare_skill(
        manifest,
        &req.answers,
        project.as_ref(),
        chapter,
        output_path.as_deref(),
        req.budget,
    )?;
    // Read what a windowed run needs before a job exists (see the app host).
    let chapter_text = if storycraft_core::is_report_skill(&req.skill) {
        let path = project
            .as_ref()
            .and_then(|project| find_chapter_prose(project, chapter))
            .ok_or_else(|| anyhow!("no chapter prose for chapter {chapter}; draft it first"))?;
        Some(std::fs::read_to_string(&path).with_context(|| path.display().to_string())?)
    } else {
        None
    };
    let saved_report = match storycraft_core::applied_report_skill(&req.skill) {
        Some(base) => {
            let rel = resolve_output_path(project.as_ref(), base, chapter);
            let saved = project
                .as_ref()
                .zip(rel)
                .and_then(|(project, rel)| std::fs::read_to_string(project.path().join(rel)).ok());
            Some(saved.ok_or_else(|| {
                anyhow!(
                    "no saved {base} report for chapter {chapter}; run {base} and save its report first"
                )
            })?)
        }
        None => None,
    };
    let mut job = store.create(NewJob {
        skill: req.skill.clone(),
        mode,
        chapter,
        answers: req.answers.clone(),
        packed_context_hash: packed.hash.clone(),
        provider: req.provider.clone(),
        model,
        output_path,
    })?;
    snapshot_original(&store, &job)?;
    store.set_status(&mut job, JobStatus::Running)?;

    let client = build_client(&req, &job.model).await?;
    let cancel = CancellationToken::new();
    let text = if let Some(chapter_text) = &chapter_text {
        run_report(&store, &job, &client, &prompt, chapter_text, &cancel).await
    } else if is_chunked_skill(&req.skill) {
        run_chunked(
            &store,
            &job,
            &client,
            &prompt,
            saved_report.as_deref(),
            &cancel,
        )
        .await
    } else if req.skill == "storybible-convert" {
        run_convert(&store, &job, &client, &prompt, &cancel).await
    } else {
        let preview_id = job.id.clone();
        client
            .complete(&prompt.system, &prompt.user, &cancel, |delta| {
                store
                    .append_preview(&preview_id, delta)
                    .map_err(|err| storycraft_llm::Error::InvalidPayload(err.to_string()))
            })
            .await
            .map_err(anyhow::Error::from)
    };

    match text {
        Ok(text) => finish_preview(&req, &store, &mut job, &text),
        Err(err) => {
            store.fail(&mut job, err.to_string())?;
            Err(err).context(format!(
                "provider '{}' at {} (is anything listening? pass --base-url and --api-key)",
                req.provider, req.base_url
            ))
        }
    }
}

fn finish_preview(
    req: &RunRequest,
    store: &JobStore,
    job: &mut Job,
    text: &str,
) -> anyhow::Result<Job> {
    let text = &match storycraft_core::finish_model_output(&job.skill, text) {
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
    if req.commit {
        validate_preview(&job.skill, text)?;
        let _ = store.commit(job)?;
    }
    Ok(job.clone())
}

fn snapshot_original(store: &JobStore, job: &Job) -> anyhow::Result<()> {
    let Some(dest) = store.output_abs(job) else {
        return Ok(());
    };
    if dest.is_file() {
        let text = std::fs::read_to_string(&dest)
            .with_context(|| format!("reading {}", dest.display()))?;
        store.write_original(&job.id, &text)?;
    }
    Ok(())
}

async fn run_chunked(
    store: &JobStore,
    job: &Job,
    client: &OpenAiClient,
    prompt: &storycraft_core::Prompt,
    report: Option<&str>,
    cancel: &CancellationToken,
) -> anyhow::Result<String> {
    let original = store.read_original(&job.id)?;
    if original.is_empty() {
        return Err(anyhow!(
            "no chapter prose for chapter {} (chunked skills rewrite an existing file)",
            job.chapter
        ));
    }
    let chunks = split_lines(&original, CHUNK_LINES);
    let total_lines = original.lines().count();
    let mut merged = Vec::with_capacity(chunks.len());
    let mut refused = 0usize;
    for chunk in &chunks {
        let piece = match report {
            Some(report) => {
                storycraft_core::build_apply_chunk_prompt(prompt, report, chunk, total_lines)
            }
            None => storycraft_core::build_chunk_prompt(prompt, chunk, total_lines),
        };
        let edited = client
            .complete(&piece.system, &piece.user, cancel, |_| Ok(()))
            .await?;
        // Apply runs answer with numbered lines; anything else is window text.
        let numbered = report
            .is_some()
            .then(|| storycraft_core::apply_numbered_edits(chunk, &edited))
            .flatten();
        if let Some(window) = numbered {
            merged.push(window);
        } else {
            if storycraft_core::chunk_edit_refused(&chunk.text, &edited) {
                refused = refused.saturating_add(1);
            }
            merged.push(apply_chunk_edit(&chunk.text, &edited));
        }
        store.write_preview(&job.id, &merge_chunks(&merged))?;
    }
    if refused == chunks.len() {
        return Err(anyhow!(
            "the model answered with a report instead of the edited chapter, so nothing was \
             changed; try again or pick another model"
        ));
    }
    Ok(merge_chunks(&merged))
}

/// Run a report skill window by window (see the app host).
async fn run_report(
    store: &JobStore,
    job: &Job,
    client: &OpenAiClient,
    prompt: &storycraft_core::Prompt,
    chapter_text: &str,
    cancel: &CancellationToken,
) -> anyhow::Result<String> {
    let chunks = split_lines(chapter_text, CHUNK_LINES);
    let total_lines = chapter_text.lines().count();
    let title = format!("{} — chapter {}", job.skill, job.chapter);
    let mut windows = Vec::with_capacity(chunks.len());
    for chunk in chunks {
        let piece = storycraft_core::build_report_chunk_prompt(prompt, &chunk, total_lines);
        let answer = client
            .complete(&piece.system, &piece.user, cancel, |_| Ok(()))
            .await?;
        windows.push((chunk, answer));
        store.write_preview(
            &job.id,
            &storycraft_core::merge_window_reports(&title, &windows),
        )?;
    }
    Ok(storycraft_core::merge_window_reports(&title, &windows))
}

/// Convert a free-form `storybible.md` section by section (see the app host).
async fn run_convert(
    store: &JobStore,
    job: &Job,
    client: &OpenAiClient,
    prompt: &storycraft_core::Prompt,
    cancel: &CancellationToken,
) -> anyhow::Result<String> {
    let source = store.read_original(&job.id)?;
    if source.trim().is_empty() {
        return Err(anyhow!(
            "no storybible.md in this folder to convert; put your bible there first"
        ));
    }
    let sections = storycraft_core::split_source(&source, 12_000);
    let mut answers = Vec::with_capacity(sections.len());
    for (i, section) in sections.iter().enumerate() {
        let piece = storycraft_core::build_section_prompt(prompt, i, sections.len(), section);
        answers.push(
            client
                .complete(&piece.system, &piece.user, cancel, |_| Ok(()))
                .await?,
        );
    }
    Ok(storycraft_core::assemble_converted(&answers)?)
}

async fn run_local(
    req: &RunRequest,
    store: &JobStore,
    project: Option<&ProjectRoot>,
    chapter: u32,
    output_path: Option<String>,
    mode: Mode,
) -> anyhow::Result<Job> {
    let text = match req.skill.as_str() {
        "burstiness-check" => {
            let project = project.ok_or_else(|| anyhow!("burstiness-check needs a Wiki folder"))?;
            let path = find_chapter_prose(project, chapter)
                .ok_or_else(|| anyhow!("no chapter prose for chapter {chapter}"))?;
            let body = std::fs::read_to_string(&path)
                .with_context(|| format!("reading {}", path.display()))?;
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
            let skills = resolve_skills_dir(req.skills_dir.as_deref())?;
            let list_name = req.answers.first().map(String::as_str).unwrap_or("fantasy");
            let count = req
                .answers
                .get(1)
                .and_then(|s| s.parse().ok())
                .unwrap_or(10usize);
            let rel = if req.skill == "name-generator" {
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
            )?;
            let mut out = String::from("# Names\n\n");
            for name in names {
                out.push_str("- ");
                out.push_str(&name);
                out.push('\n');
            }
            out
        }
        "storybible-import" => storycraft_core::storybible_import_preview(store.root())?,
        other => return Err(anyhow!("not a local tool: {other}")),
    };
    let mut job = store.create(NewJob {
        skill: req.skill.clone(),
        mode,
        chapter,
        answers: req.answers.clone(),
        packed_context_hash: "local".into(),
        provider: "local".into(),
        model: "none".into(),
        output_path,
    })?;
    store.write_preview(&job.id, &text)?;
    store.set_status(&mut job, JobStatus::NeedsConfirm)?;
    if req.commit {
        let _ = store.commit(&mut job)?;
    }
    Ok(job)
}

async fn build_client(req: &RunRequest, model: &str) -> anyhow::Result<OpenAiClient> {
    let mut config = ProviderConfig::openai_compat(&req.base_url, model);
    config.name = req.provider.clone();
    config.api_style = req.api_style;
    config.api_key = match req.provider.as_str() {
        "grok-oauth" => Some(Secret::new(oauth_access_token(req).await?)),
        _ => req.api_key.as_ref().map(Secret::new),
    };
    OpenAiClient::new(config).map_err(anyhow::Error::from)
}

async fn oauth_access_token(req: &RunRequest) -> anyhow::Result<String> {
    let path = req.auth_file.clone().unwrap_or_else(default_token_path);
    let store = TokenStore::new(path);
    let mut tokens = store
        .load()?
        .ok_or_else(|| anyhow!("not signed in; run `storycraft auth login`"))?;
    if tokens.needs_refresh() {
        let refresh = tokens
            .refresh_token
            .as_deref()
            .ok_or_else(|| anyhow!("OAuth token expired; run `storycraft auth login`"))?;
        let auth = DeviceAuth::new(OAuthConfig::default())?;
        tokens = auth.refresh(refresh).await?;
        store.save(&tokens)?;
    }
    Ok(tokens.access_token)
}

async fn auth_login(
    auth_url: String,
    client_id: String,
    scopes: String,
    auth_file: Option<PathBuf>,
) -> anyhow::Result<()> {
    println!("Sign in with xAI (community OAuth). SuperGrok is not a guarantee of API access.");
    let config = OAuthConfig {
        auth_base: auth_url,
        client_id,
        scopes,
    };
    let auth = DeviceAuth::new(config)?;
    let pending = auth.request_code().await?;
    let open = pending
        .verification_uri_complete
        .as_deref()
        .unwrap_or(&pending.verification_uri);
    println!("Open: {open}");
    println!("User code: {}", pending.user_code);
    println!("Waiting for authorization…");
    let tokens = auth.poll_token(&pending, &CancellationToken::new()).await?;
    let store = TokenStore::new(auth_file.unwrap_or_else(default_token_path));
    store.save(&tokens)?;
    println!("signed in; tokens stored at {}", store.path().display());
    Ok(())
}

fn build_board(path: &Path, chapter: Option<u32>, mode: Mode) -> anyhow::Result<StatusBoard> {
    let project = resolve_project(path)?;
    let chapter = match (chapter, project.as_ref()) {
        (Some(n), _) => n,
        (None, Some(project)) => infer_chapter(project),
        (None, None) => 1,
    };
    // Folder-aware: a storybible with no Wiki routes to the importer.
    StatusBoard::inspect_folder(path, mode, chapter).map_err(anyhow::Error::from)
}

fn resolve_project(path: &Path) -> anyhow::Result<Option<ProjectRoot>> {
    if !path.exists() {
        return Ok(None);
    }
    let mut found = discover(path)?;
    match found.len() {
        0 => Ok(None),
        1 => Ok(found.pop()),
        _ => Err(Error::MultipleProjects { projects: found }.into()),
    }
}

fn jobs_root(path: &Path) -> anyhow::Result<PathBuf> {
    match resolve_project(path)? {
        Some(project) => Ok(project.path().to_path_buf()),
        None => Ok(path.to_path_buf()),
    }
}

fn load_catalog(skills_dir: Option<&Path>) -> anyhow::Result<Catalog> {
    let dir = resolve_skills_dir(skills_dir)?;
    Catalog::load(&dir).with_context(|| format!("loading skill catalog from {}", dir.display()))
}

fn resolve_skills_dir(explicit: Option<&Path>) -> anyhow::Result<PathBuf> {
    if let Some(path) = explicit {
        return Ok(path.to_path_buf());
    }
    let cwd = std::env::current_dir().context("current directory")?;
    find_skills_dir(&cwd).ok_or_else(|| {
        anyhow!("no skill library found; pass --skills-dir or set STORYCRAFT_SKILLS")
    })
}

/// Install a tracing subscriber when `RUST_LOG` is set.
pub fn init_tracing() {
    if std::env::var_os("RUST_LOG").is_none() {
        return;
    }
    let _ = tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .try_init();
}

/// Map core errors to process exit codes.
#[must_use]
pub fn exit_err(err: &anyhow::Error) -> ExitCode {
    if let Some(Error::MultipleProjects { projects }) = err.downcast_ref::<Error>() {
        eprintln!("multiple Wiki folders found; pick one:");
        eprintln!("{}", format_project_list(projects));
        return ExitCode::from(2);
    }
    eprintln!("{err:#}");
    if err.downcast_ref::<Error>().is_some_and(|core| {
        matches!(
            core,
            Error::NoProject(_) | Error::InvalidChapter | Error::JobNotFound(_)
        )
    }) {
        ExitCode::from(2)
    } else {
        ExitCode::FAILURE
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;
    use clap::Parser;

    #[test]
    fn run_accepts_path_flag() {
        let cli =
            Cli::try_parse_from(["storycraft", "run", "fiction-genre", "--path", "/tmp/book"])
                .expect("parse");
        match cli.command {
            Command::Run { project, skill, .. } => {
                assert_eq!(skill, "fiction-genre");
                assert_eq!(project.resolve(), PathBuf::from("/tmp/book"));
            }
            _ => panic!("expected run"),
        }
    }

    #[test]
    fn status_accepts_positional_path() {
        let cli = Cli::try_parse_from(["storycraft", "status", "/tmp/book"]).expect("parse");
        match cli.command {
            Command::Status { project, .. } => {
                assert_eq!(project.resolve(), PathBuf::from("/tmp/book"));
            }
            _ => panic!("expected status"),
        }
    }

    #[test]
    fn run_parses_cheap_model_and_routes() {
        let cli = Cli::try_parse_from([
            "storycraft",
            "run",
            "kill-crutch",
            "--cheap-model",
            "mini",
            "--route",
            "kill-flat=other",
            "--budget",
            "12000",
        ])
        .expect("parse");
        match cli.command {
            Command::Run {
                cheap_model,
                routes,
                budget,
                model,
                ..
            } => {
                assert_eq!(cheap_model.as_deref(), Some("mini"));
                assert_eq!(routes, vec![("kill-flat".into(), "other".into())]);
                assert_eq!(budget, 12_000);
                assert_eq!(model, "grok-4.6");
            }
            _ => panic!("expected run"),
        }
    }

    #[test]
    fn skills_overlays_flag_parses() {
        let cli = Cli::try_parse_from(["storycraft", "skills", "--overlays"]).expect("parse");
        match cli.command {
            Command::Skills { overlays } => assert!(overlays),
            _ => panic!("expected skills"),
        }
    }
}
