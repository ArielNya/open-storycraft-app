//! Host logic for the `storycraft` CLI.

#![allow(missing_docs)]
#![allow(clippy::print_stdout)]

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anyhow::{Context, anyhow};
use clap::{Args, Parser, Subcommand};
use storycraft_auth::{
    DEFAULT_AUTH_BASE, DEFAULT_CLIENT_ID, DEFAULT_SCOPES, DeviceAuth, OAuthConfig, TokenStore,
    default_token_path,
};
use storycraft_core::{
    Catalog, Error, Job, JobStatus, JobStore, Mode, NewJob, ProjectRoot, StatusBoard, discover,
    ensure_requires, find_skills_dir, format_project_list, infer_chapter, pack_skill,
    prepare_skill, validate_preview,
};
use storycraft_llm::{ApiStyle, CancellationToken, OpenAiClient, ProviderConfig, Secret};

/// Open Storycraft command-line host.
#[derive(Parser)]
#[command(
    name = "storycraft",
    version,
    about = "Open Storycraft local writing studio"
)]
pub struct Cli {
    /// Skill library directory (42 craft skills + orchestrator).
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
    Skills,
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
        Command::Skills => {
            let catalog = load_catalog(cli.skills_dir.as_deref())?;
            for skill in catalog.iter() {
                println!("{} — {}", skill.name, skill.description);
            }
            eprintln!("{} skills", catalog.len());
        }
        Command::Pack { ref skill } => {
            let catalog = load_catalog(cli.skills_dir.as_deref())?;
            let manifest = catalog
                .get(skill)
                .ok_or_else(|| Error::SkillNotFound(skill.clone()))?;
            let pack = pack_skill(manifest);
            for file in &pack.files {
                let label = match file.kind {
                    storycraft_core::PackKind::Skill => "skill",
                    storycraft_core::PackKind::Reference => "ref",
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
    let manifest = catalog
        .get(&req.skill)
        .ok_or_else(|| Error::SkillNotFound(req.skill.clone()))?;
    let project = resolve_project(&req.path)?;
    ensure_requires(project.as_ref(), &manifest.requires)?;
    let chapter = match (req.chapter, project.as_ref()) {
        (Some(n), _) => n,
        (None, Some(project)) => infer_chapter(project),
        (None, None) => 1,
    };
    let mode = if req.skill == "fiction-story-sparks" {
        Mode::Spark
    } else {
        Mode::SingleSkill
    };
    let (packed, prompt) = prepare_skill(manifest, &req.answers)?;
    let root = jobs_root(&req.path)?;
    let store = JobStore::open(&root)?;
    let mut job = store.create(NewJob {
        skill: req.skill.clone(),
        mode,
        chapter,
        answers: req.answers.clone(),
        packed_context_hash: packed.hash.clone(),
        provider: req.provider.clone(),
        model: req.model.clone(),
    })?;
    store.set_status(&mut job, JobStatus::Running)?;

    let client = build_client(&req).await?;
    let cancel = CancellationToken::new();
    let preview_id = job.id.clone();
    let result = client
        .complete(&prompt.system, &prompt.user, &cancel, |delta| {
            store
                .append_preview(&preview_id, delta)
                .map_err(|err| storycraft_llm::Error::InvalidPayload(err.to_string()))
        })
        .await;

    match result {
        Ok(text) => {
            store.write_preview(&job.id, &text)?;
            if let Err(err) = validate_preview(&job.skill, &text) {
                store.fail(&mut job, err.to_string())?;
                return Err(err.into());
            }
            store.set_status(&mut job, JobStatus::NeedsConfirm)?;
            if req.commit {
                validate_preview(&job.skill, &text)?;
                let _ = store.commit(&mut job)?;
            }
            Ok(job)
        }
        Err(err) => {
            store.fail(&mut job, err.to_string())?;
            Err(err).with_context(|| {
                format!(
                    "provider '{}' at {} (is anything listening? pass --base-url and --api-key)",
                    req.provider, req.base_url
                )
            })
        }
    }
}

async fn build_client(req: &RunRequest) -> anyhow::Result<OpenAiClient> {
    let mut config = ProviderConfig::openai_compat(&req.base_url, &req.model);
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
    StatusBoard::inspect(project.as_ref(), mode, chapter).map_err(anyhow::Error::from)
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
}
