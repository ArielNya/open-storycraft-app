//! Resumable skill jobs. Logs live under `.storycraft/jobs/`, not in the Wiki.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::bible;
use crate::output::{SkillOutput, skill_output, split_character_preview};
use crate::{Error, Mode};

/// Lifecycle of one skill execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    /// Created, not yet sent to a model.
    Queued,
    /// Streaming or waiting on the provider.
    Running,
    /// Preview is on disk; waiting for save / reject.
    NeedsConfirm,
    /// Preview committed to the Wiki path (or accepted with no Wiki write).
    Saved,
    /// User rejected the preview.
    Rejected,
    /// Provider or validation failed.
    Failed,
}

impl JobStatus {
    /// Lowercase status token.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Running => "running",
            Self::NeedsConfirm => "needs_confirm",
            Self::Saved => "saved",
            Self::Rejected => "rejected",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for JobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One skill run. Fields are persisted as JSON beside the preview file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Job {
    /// Filename-safe id.
    pub id: String,
    /// Skill folder name.
    pub skill: String,
    /// Orchestrator mode used for this run.
    pub mode: Mode,
    /// Chapter used when packing chapter-scoped skills.
    pub chapter: u32,
    /// Answers to the skill's questions, in order.
    #[serde(default)]
    pub answers: Vec<String>,
    /// Hash of the packed context that was sent.
    pub packed_context_hash: String,
    /// Provider profile name (`openai-compat`, `xai-apikey`, `grok-oauth`).
    pub provider: String,
    /// Model id sent to the provider.
    pub model: String,
    /// Current status.
    pub status: JobStatus,
    /// Relative Wiki path, if this skill writes one file.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_path: Option<String>,
    /// Last error, if failed. Never contains secrets.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Fields needed to open a queued job.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewJob {
    /// Skill folder name.
    pub skill: String,
    /// Orchestrator mode.
    pub mode: Mode,
    /// Chapter used when packing chapter-scoped skills.
    pub chapter: u32,
    /// Answers to the skill's questions, in order.
    pub answers: Vec<String>,
    /// Hash of the packed context that will be sent.
    pub packed_context_hash: String,
    /// Provider profile name.
    pub provider: String,
    /// Model id.
    pub model: String,
    /// Relative destination, if this run writes a file or directory.
    pub output_path: Option<String>,
}

/// On-disk job log for one project (or a fallback root when there is no Wiki).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JobStore {
    root: PathBuf,
    dir: PathBuf,
}

impl JobStore {
    /// Open (and create) `.storycraft/jobs` under `root`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Io`] if the jobs directory cannot be created.
    pub fn open(root: &Path) -> Result<Self, Error> {
        let dir = root.join(".storycraft").join("jobs");
        fs::create_dir_all(&dir).map_err(|err| Error::io(&dir, err))?;
        Ok(Self {
            root: root.to_path_buf(),
            dir,
        })
    }

    /// Project / jobs root.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Jobs directory.
    #[must_use]
    pub fn dir(&self) -> &Path {
        &self.dir
    }

    /// Allocate an id, persist a queued job, and truncate its preview file.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Io`] if the job files cannot be written.
    pub fn create(&self, spec: NewJob) -> Result<Job, Error> {
        let id = new_job_id(&spec.skill, &self.dir);
        let job = Job {
            id,
            output_path: spec.output_path,
            skill: spec.skill,
            mode: spec.mode,
            chapter: spec.chapter,
            answers: spec.answers,
            packed_context_hash: spec.packed_context_hash,
            provider: spec.provider,
            model: spec.model,
            status: JobStatus::Queued,
            error: None,
        };
        self.persist(&job)?;
        let preview = self.preview_path(&job.id);
        fs::write(&preview, "").map_err(|err| Error::io(&preview, err))?;
        Ok(job)
    }

    /// JSON sidecar for `id`.
    #[must_use]
    pub fn json_path(&self, id: &str) -> PathBuf {
        self.dir.join(format!("{id}.json"))
    }

    /// Streaming preview markdown for `id`.
    #[must_use]
    pub fn preview_path(&self, id: &str) -> PathBuf {
        self.dir.join(format!("{id}.preview.md"))
    }

    /// Snapshot of the live destination before this job, for diffs.
    #[must_use]
    pub fn original_path(&self, id: &str) -> PathBuf {
        self.dir.join(format!("{id}.original.md"))
    }

    /// Absolute Wiki destination for a job, if it has one.
    #[must_use]
    pub fn output_abs(&self, job: &Job) -> Option<PathBuf> {
        job.output_path.as_ref().map(|rel| self.root.join(rel))
    }

    /// Load a job by id.
    ///
    /// # Errors
    ///
    /// Returns [`Error::JobNotFound`] or [`Error::InvalidJob`].
    pub fn load(&self, id: &str) -> Result<Job, Error> {
        let path = self.json_path(id);
        let bytes = fs::read(&path).map_err(|err| {
            if err.kind() == std::io::ErrorKind::NotFound {
                Error::JobNotFound(id.to_owned())
            } else {
                Error::io(&path, err)
            }
        })?;
        serde_json::from_slice(&bytes).map_err(|err| Error::InvalidJob {
            path,
            detail: err.to_string(),
        })
    }

    /// Every job in the store, newest id last.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Io`] or [`Error::InvalidJob`].
    pub fn list(&self) -> Result<Vec<Job>, Error> {
        let mut jobs = Vec::new();
        let entries = fs::read_dir(&self.dir).map_err(|err| Error::io(&self.dir, err))?;
        for entry in entries {
            let entry = entry.map_err(|err| Error::io(&self.dir, err))?;
            let path = entry.path();
            let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            if !name.ends_with(".json") {
                continue;
            }
            let bytes = fs::read(&path).map_err(|err| Error::io(&path, err))?;
            let job: Job = serde_json::from_slice(&bytes).map_err(|err| Error::InvalidJob {
                path: path.clone(),
                detail: err.to_string(),
            })?;
            jobs.push(job);
        }
        jobs.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(jobs)
    }

    /// Replace the preview file contents (crash-safe target for streaming).
    ///
    /// # Errors
    ///
    /// Returns [`Error::Io`] if the preview cannot be written.
    pub fn write_preview(&self, id: &str, text: &str) -> Result<(), Error> {
        let path = self.preview_path(id);
        fs::write(&path, text).map_err(|err| Error::io(&path, err))
    }

    /// Append a streamed token chunk to the preview file.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Io`] if the preview cannot be appended.
    pub fn append_preview(&self, id: &str, chunk: &str) -> Result<(), Error> {
        use std::io::Write;
        let path = self.preview_path(id);
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(|err| Error::io(&path, err))?;
        file.write_all(chunk.as_bytes())
            .map_err(|err| Error::io(&path, err))
    }

    /// Read the current preview text.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Io`] if the preview cannot be read.
    pub fn read_preview(&self, id: &str) -> Result<String, Error> {
        let path = self.preview_path(id);
        fs::read_to_string(&path).map_err(|err| Error::io(&path, err))
    }

    /// Store the live file as it existed before this job.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Io`] if the sidecar cannot be written.
    pub fn write_original(&self, id: &str, text: &str) -> Result<(), Error> {
        let path = self.original_path(id);
        fs::write(&path, text).map_err(|err| Error::io(&path, err))
    }

    /// Read the original sidecar. Missing file is an empty original (new dest).
    ///
    /// # Errors
    ///
    /// Returns [`Error::Io`] on read failure other than not found.
    pub fn read_original(&self, id: &str) -> Result<String, Error> {
        let path = self.original_path(id);
        match fs::read_to_string(&path) {
            Ok(text) => Ok(text),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
            Err(err) => Err(Error::io(&path, err)),
        }
    }

    /// Persist `job` as JSON.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Io`] if the sidecar cannot be written.
    pub fn persist(&self, job: &Job) -> Result<(), Error> {
        let path = self.json_path(&job.id);
        let bytes = serde_json::to_vec_pretty(job).map_err(|err| Error::InvalidJob {
            path: path.clone(),
            detail: err.to_string(),
        })?;
        fs::write(&path, bytes).map_err(|err| Error::io(&path, err))
    }

    /// Set status and persist.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Io`] on write failure.
    pub fn set_status(&self, job: &mut Job, status: JobStatus) -> Result<(), Error> {
        job.status = status;
        if status != JobStatus::Failed {
            job.error = None;
        }
        self.persist(job)
    }

    /// Mark failed with a redacted error string.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Io`] on write failure.
    pub fn fail(&self, job: &mut Job, message: impl Into<String>) -> Result<(), Error> {
        job.status = JobStatus::Failed;
        job.error = Some(message.into());
        self.persist(job)
    }

    /// Atomic copy of the preview onto the Wiki path. Sparks (no output) just
    /// flip status to saved.
    ///
    /// # Errors
    ///
    /// Returns [`Error::JobInvalidState`] unless the job is `needs_confirm`,
    /// or [`Error::Io`] if the destination cannot be written.
    pub fn commit(&self, job: &mut Job) -> Result<Option<PathBuf>, Error> {
        if job.status != JobStatus::NeedsConfirm {
            return Err(Error::JobInvalidState {
                id: job.id.clone(),
                status: job.status.to_string(),
                expected: JobStatus::NeedsConfirm.to_string(),
            });
        }
        match skill_output(&job.skill) {
            SkillOutput::Bundle => return self.commit_bundle(job),
            SkillOutput::CharactersDir => return self.commit_characters(job),
            _ => {}
        }
        let dest = match self.output_abs(job) {
            Some(path) => path,
            None => {
                self.set_status(job, JobStatus::Saved)?;
                return Ok(None);
            }
        };
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent).map_err(|err| Error::io(parent, err))?;
        }
        let preview = self.read_preview(&job.id)?;
        write_atomic(&dest, preview.as_bytes())?;
        self.set_status(job, JobStatus::Saved)?;
        Ok(Some(dest))
    }

    /// Write a bundle preview: one file per document, each naming its own path.
    ///
    /// The storybible importer uses this to turn a single bible into a whole
    /// book folder in one confirmed step.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidStoryBible`] when a document names an unsafe
    /// destination, or [`Error::Io`] when one cannot be written.
    fn commit_bundle(&self, job: &mut Job) -> Result<Option<PathBuf>, Error> {
        let preview = self.read_preview(&job.id)?;
        let docs = bible::parse_storybible(&preview)?;
        for doc in &docs {
            let dest = self.root.join(bible::safe_rel(&doc.path)?);
            if let Some(parent) = dest.parent() {
                fs::create_dir_all(parent).map_err(|err| Error::io(parent, err))?;
            }
            write_atomic(&dest, doc.markdown().as_bytes())?;
        }
        self.set_status(job, JobStatus::Saved)?;
        Ok(Some(self.root.clone()))
    }

    fn commit_characters(&self, job: &mut Job) -> Result<Option<PathBuf>, Error> {
        let dir = self.root.join("Wiki/Characters");
        fs::create_dir_all(&dir).map_err(|err| Error::io(&dir, err))?;
        let preview = self.read_preview(&job.id)?;
        let files = split_character_preview(&preview);
        let mut last = dir.clone();
        for (name, body) in files {
            let dest = dir.join(name);
            write_atomic(&dest, body.as_bytes())?;
            last = dest;
        }
        self.set_status(job, JobStatus::Saved)?;
        Ok(Some(last))
    }

    /// Reject a preview without touching the Wiki.
    ///
    /// # Errors
    ///
    /// Returns [`Error::JobInvalidState`] unless the job is `needs_confirm`.
    pub fn reject(&self, job: &mut Job) -> Result<(), Error> {
        if job.status != JobStatus::NeedsConfirm {
            return Err(Error::JobInvalidState {
                id: job.id.clone(),
                status: job.status.to_string(),
                expected: JobStatus::NeedsConfirm.to_string(),
            });
        }
        self.set_status(job, JobStatus::Rejected)
    }
}

/// Write `bytes` to `dest` through a temporary file beside it.
fn write_atomic(dest: &Path, bytes: &[u8]) -> Result<(), Error> {
    let tmp = {
        let mut os = dest.as_os_str().to_owned();
        os.push(".tmp");
        PathBuf::from(os)
    };
    fs::write(&tmp, bytes).map_err(|err| Error::io(&tmp, err))?;
    fs::rename(&tmp, dest).map_err(|err| Error::io(dest, err))
}

fn new_job_id(skill: &str, dir: &Path) -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let slug: String = skill
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect();
    let mut id = format!("{slug}-{millis}");
    let mut n = 2u32;
    while dir.join(format!("{id}.json")).exists() {
        id = format!("{slug}-{millis}-{n}");
        n = n.saturating_add(1);
        if n > 32 {
            break;
        }
    }
    id
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;

    #[test]
    fn commit_writes_genre_atomically() {
        let tmp = tempfile::tempdir().unwrap();
        let store = JobStore::open(tmp.path()).unwrap();
        let mut job = store
            .create(NewJob {
                skill: "fiction-genre".into(),
                mode: Mode::SingleSkill,
                chapter: 1,
                answers: vec!["Fantasy".into()],
                packed_context_hash: "hash".into(),
                provider: "openai-compat".into(),
                model: "mock".into(),
                output_path: Some("Wiki/Style/genre.md".into()),
            })
            .unwrap();
        store
            .write_preview(
                &job.id,
                "---\ngenre: Fantasy\n---\n\n# Genre\n\nTone notes here.\n",
            )
            .unwrap();
        store.set_status(&mut job, JobStatus::NeedsConfirm).unwrap();
        let dest = store.commit(&mut job).unwrap().unwrap();
        assert_eq!(job.status, JobStatus::Saved);
        let text = fs::read_to_string(dest).unwrap();
        assert!(text.contains("genre: Fantasy"));
    }

    #[test]
    fn sparks_commit_does_not_create_wiki() {
        let tmp = tempfile::tempdir().unwrap();
        let store = JobStore::open(tmp.path()).unwrap();
        let mut job = store
            .create(NewJob {
                skill: "fiction-story-sparks".into(),
                mode: Mode::Spark,
                chapter: 1,
                answers: Vec::new(),
                packed_context_hash: "hash".into(),
                provider: "openai-compat".into(),
                model: "mock".into(),
                output_path: None,
            })
            .unwrap();
        store.write_preview(&job.id, "Premise: a test.\n").unwrap();
        store.set_status(&mut job, JobStatus::NeedsConfirm).unwrap();
        assert!(store.commit(&mut job).unwrap().is_none());
        assert!(!tmp.path().join("Wiki").exists());
        assert_eq!(job.status, JobStatus::Saved);
    }

    #[test]
    fn reject_leaves_wiki_untouched() {
        let tmp = tempfile::tempdir().unwrap();
        let store = JobStore::open(tmp.path()).unwrap();
        let mut job = store
            .create(NewJob {
                skill: "fiction-genre".into(),
                mode: Mode::SingleSkill,
                chapter: 1,
                answers: Vec::new(),
                packed_context_hash: "h".into(),
                provider: "openai-compat".into(),
                model: "mock".into(),
                output_path: Some("Wiki/Style/genre.md".into()),
            })
            .unwrap();
        store.set_status(&mut job, JobStatus::NeedsConfirm).unwrap();
        store.reject(&mut job).unwrap();
        assert_eq!(job.status, JobStatus::Rejected);
        assert!(!tmp.path().join("Wiki/Style/genre.md").exists());
    }

    #[test]
    fn commit_splits_character_files() {
        let tmp = tempfile::tempdir().unwrap();
        let store = JobStore::open(tmp.path()).unwrap();
        let mut job = store
            .create(NewJob {
                skill: "fiction-characters".into(),
                mode: Mode::SingleSkill,
                chapter: 1,
                answers: Vec::new(),
                packed_context_hash: "h".into(),
                provider: "openai-compat".into(),
                model: "mock".into(),
                output_path: Some("Wiki/Characters".into()),
            })
            .unwrap();
        store
            .write_preview(
                &job.id,
                "---\nname: Mira\nrole: protagonist\n---\n\n# Mira\n\nVoice of the dock.\n\n---\nname: Kael\nrole: antagonist\n---\n\n# Kael\n\nSteel and debt.\n",
            )
            .unwrap();
        store.set_status(&mut job, JobStatus::NeedsConfirm).unwrap();
        store.commit(&mut job).unwrap();
        assert!(tmp.path().join("Wiki/Characters/Mira.md").is_file());
        assert!(tmp.path().join("Wiki/Characters/Kael.md").is_file());
        assert_eq!(job.status, JobStatus::Saved);
    }
}
