//! fiction-story-sparks and fiction-genre against a mock OpenAI-compat provider.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::print_stdout)]

use std::fs;
use std::path::PathBuf;

use storycraft_cli::{RunRequest, execute_run};
use storycraft_core::JobStatus;
use storycraft_llm::ApiStyle;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn skills_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../open-storycraft")
}

fn sse_text(text: &str) -> String {
    let escaped = text
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n");
    format!(
        "data: {{\"choices\":[{{\"delta\":{{\"content\":\"{escaped}\"}}}}]}}\n\ndata: [DONE]\n\n"
    )
}

fn run_req(server: &MockServer, skill: &str, path: PathBuf, commit: bool) -> RunRequest {
    RunRequest {
        skills_dir: Some(skills_dir()),
        skill: skill.to_owned(),
        path,
        answers: vec![
            "Fantasy".into(),
            "low fantasy".into(),
            "Night Market".into(),
        ],
        chapter: Some(1),
        provider: "openai-compat".into(),
        base_url: format!("{}/v1", server.uri()),
        api_key: Some("sk-mock".into()),
        api_style: ApiStyle::ChatCompletions,
        model: "mock".into(),
        cheap_model: None,
        routes: std::collections::BTreeMap::new(),
        budget: 48_000,
        commit,
        auth_file: None,
    }
}

#[tokio::test(flavor = "current_thread")]
async fn sparks_preview_does_not_write_wiki() {
    let server = MockServer::start().await;
    let spark = "Someone: a harbour clerk under a false tide table\n\
Somewhere: a fogged dock before dawn\n\
Something: the official ledger that no longer matches the water\n\
It happens: last night's high water mark is a foot above the posted number\n\n\
Premise: The clerk finds the town's public tide is a lie and has to decide whether to copy it.\n\
Reader promise: A closed-circle harbour mystery that pays off in a public record being corrected.\n\
Central question: Who gets to write the town's official weather?\n\
Answer and payoff: She posts the real numbers and keeps her private book.\n";
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_string(sse_text(spark)),
        )
        .mount(&server)
        .await;

    let tmp = tempfile::tempdir().unwrap();
    let job = execute_run(run_req(
        &server,
        "fiction-story-sparks",
        tmp.path().to_path_buf(),
        true,
    ))
    .await
    .unwrap();
    assert_eq!(job.status, JobStatus::Saved);
    assert!(!tmp.path().join("Wiki").exists());
}

#[tokio::test(flavor = "current_thread")]
async fn genre_commit_writes_wiki_style_genre() {
    let server = MockServer::start().await;
    let genre = "---\nworking_title: \"Night Market\"\ngenre: Fantasy\nsubgenre: \"Low Fantasy\"\nflavor: Coastal_Night_Market\n---\n\n# Genre\n\n## tone_notes\n\nWarm street-level fantasy with a mercantile edge and lantern-lit danger.\n";
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_string(sse_text(genre)),
        )
        .mount(&server)
        .await;

    let tmp = tempfile::tempdir().unwrap();
    let job = execute_run(run_req(
        &server,
        "fiction-genre",
        tmp.path().to_path_buf(),
        true,
    ))
    .await
    .unwrap();
    assert_eq!(job.status, JobStatus::Saved);
    let dest = tmp.path().join("Wiki/Style/genre.md");
    let text = fs::read_to_string(dest).unwrap();
    assert!(text.contains("genre: Fantasy"));
    assert!(text.contains("Night Market"));
}

#[tokio::test(flavor = "current_thread")]
async fn genre_without_commit_stays_preview() {
    let server = MockServer::start().await;
    let genre = "---\ngenre: Horror\nsubgenre: Folk\n---\n\n# Genre\n\n## tone_notes\n\nDamp stone and a town that pretends it cannot smell the tide at night.\n";
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_string(sse_text(genre)),
        )
        .mount(&server)
        .await;

    let tmp = tempfile::tempdir().unwrap();
    let job = execute_run(run_req(
        &server,
        "fiction-genre",
        tmp.path().to_path_buf(),
        false,
    ))
    .await
    .unwrap();
    assert_eq!(job.status, JobStatus::NeedsConfirm);
    assert!(!tmp.path().join("Wiki/Style/genre.md").exists());
}

fn copy_dir(src: &std::path::Path, dst: &std::path::Path) {
    fs::create_dir_all(dst).unwrap();
    for entry in fs::read_dir(src).unwrap() {
        let entry = entry.unwrap();
        let to = dst.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_dir(&entry.path(), &to);
        } else {
            fs::copy(entry.path(), to).unwrap();
        }
    }
}

fn numbered_chapter(n: usize) -> String {
    (1..=n)
        .map(|i| format!("He just walked line {i}.\n"))
        .collect()
}

#[tokio::test(flavor = "current_thread")]
async fn writechapter_preview_targets_chapter_file() {
    let server = MockServer::start().await;
    let chapter = "# Chapter 1\n\nThe clerk copied the lie into her private book before the first lantern went out on the quay.\n";
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_string(sse_text(chapter)),
        )
        .mount(&server)
        .await;

    let tmp = tempfile::tempdir().unwrap();
    let src = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../crates/storycraft-core/tests/fixtures/planning-done");
    copy_dir(&src, tmp.path());
    let mut req = run_req(
        &server,
        "fiction-writechapter",
        tmp.path().to_path_buf(),
        false,
    );
    req.cheap_model = Some("mini".into());
    let job = execute_run(req).await.unwrap();
    assert_eq!(job.model, "mock");
    assert_eq!(job.status, JobStatus::NeedsConfirm);
    assert_eq!(job.output_path.as_deref(), Some("Chapters/Chapter-001.md"));
    assert!(!tmp.path().join("Chapters/Chapter-001.md").exists());
}

#[tokio::test(flavor = "current_thread")]
async fn kill_pass_chunks_then_one_confirm() {
    let server = MockServer::start().await;
    let first: String = (1..=40).map(|i| format!("He walked line {i}.\n")).collect();
    let second: String = (41..=50)
        .map(|i| format!("He walked line {i}.\n"))
        .collect();
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_string(sse_text(&first)),
        )
        .up_to_n_times(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_string(sse_text(&second)),
        )
        .mount(&server)
        .await;

    let tmp = tempfile::tempdir().unwrap();
    fs::create_dir_all(tmp.path().join("Wiki/Style")).unwrap();
    fs::write(
        tmp.path().join("Wiki/Style/genre.md"),
        "---\ngenre: Fantasy\n---\n\n# Genre\n\nEnough text to count as real.\n",
    )
    .unwrap();
    fs::create_dir_all(tmp.path().join("Chapters")).unwrap();
    fs::write(
        tmp.path().join("Chapters/Chapter-001.md"),
        numbered_chapter(50),
    )
    .unwrap();

    let mut req = run_req(&server, "kill-crutch", tmp.path().to_path_buf(), false);
    req.cheap_model = Some("mini".into());
    let job = execute_run(req).await.unwrap();
    assert_eq!(job.model, "mini");
    assert_eq!(job.status, JobStatus::NeedsConfirm);
    assert_eq!(job.output_path.as_deref(), Some("Chapters/Chapter-001.md"));
    let preview = fs::read_to_string(
        tmp.path()
            .join(".storycraft/jobs")
            .join(format!("{}.preview.md", job.id)),
    )
    .unwrap();
    assert!(preview.contains("He walked line 1."));
    assert!(preview.contains("He walked line 50."));
    assert!(!preview.contains("just walked"));
    assert!(tmp.path().join("Chapters/Chapter-001.md").exists());
    let live = fs::read_to_string(tmp.path().join("Chapters/Chapter-001.md")).unwrap();
    assert!(live.contains("just walked"));
}

#[tokio::test(flavor = "current_thread")]
async fn burstiness_runs_locally_without_a_provider() {
    let tmp = tempfile::tempdir().unwrap();
    fs::create_dir_all(tmp.path().join("Wiki/Style")).unwrap();
    fs::write(
        tmp.path().join("Wiki/Style/genre.md"),
        "---\ngenre: Fantasy\n---\n\n# Genre\n\nEnough text to count as real.\n",
    )
    .unwrap();
    fs::create_dir_all(tmp.path().join("Chapters")).unwrap();
    fs::write(
        tmp.path().join("Chapters/Chapter-001.md"),
        "\"Did you see the mark?\" she said. \"A foot off.\"\n\nHe nodded. \"Copy it anyway.\"\n",
    )
    .unwrap();

    let job = execute_run(RunRequest {
        skills_dir: Some(skills_dir()),
        skill: "burstiness-check".into(),
        path: tmp.path().to_path_buf(),
        answers: Vec::new(),
        chapter: Some(1),
        provider: "openai-compat".into(),
        base_url: "http://127.0.0.1:1/v1".into(),
        api_key: None,
        api_style: ApiStyle::ChatCompletions,
        model: "mock".into(),
        cheap_model: None,
        routes: std::collections::BTreeMap::new(),
        budget: 48_000,
        commit: false,
        auth_file: None,
    })
    .await
    .unwrap();
    assert_eq!(job.status, JobStatus::NeedsConfirm);
    assert_eq!(job.provider, "local");
    let preview = fs::read_to_string(
        tmp.path()
            .join(".storycraft/jobs")
            .join(format!("{}.preview.md", job.id)),
    )
    .unwrap();
    assert!(preview.contains("dialogue ratio"));
}

/// Mount `answers` as consecutive SSE replies, in order.
async fn reply_in_order(server: &MockServer, answers: &[String]) {
    for (i, answer) in answers.iter().enumerate() {
        let mock = Mock::given(method("POST"))
            .and(path("/v1/chat/completions"))
            .respond_with(
                ResponseTemplate::new(200)
                    .insert_header("content-type", "text/event-stream")
                    .set_body_string(sse_text(answer)),
            );
        // The last answer keeps answering; earlier ones answer once.
        let mock = if i + 1 < answers.len() {
            mock.up_to_n_times(1)
        } else {
            mock
        };
        mock.mount(server).await;
    }
}

/// A report skill: findings saved beside the chapter, chapter untouched; then
/// `:apply` rewrites the chapter from that saved report.
#[tokio::test(flavor = "current_thread")]
async fn fragment_report_is_saved_then_applied() {
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();
    fs::create_dir_all(tmp.path().join("Wiki/Style")).unwrap();
    fs::write(
        tmp.path().join("Wiki/Style/genre.md"),
        "---\ngenre: Fantasy\n---\n\n# Genre\n\nEnough text to count as real.\n",
    )
    .unwrap();
    fs::create_dir_all(tmp.path().join("Chapters")).unwrap();
    let chapter = numbered_chapter(50);
    fs::write(tmp.path().join("Chapters/Chapter-001.md"), &chapter).unwrap();

    reply_in_order(
        &server,
        &[
            "NOTHING IN THIS WINDOW".to_owned(),
            "**Line 45:** `He just walked line 45.` — KILL. Fix: He walked line 45.".to_owned(),
        ],
    )
    .await;
    let job = execute_run(run_req(
        &server,
        "fragment-hunter",
        tmp.path().to_path_buf(),
        true,
    ))
    .await
    .unwrap();
    assert_eq!(job.status, JobStatus::Saved);
    let report =
        fs::read_to_string(tmp.path().join("Chapters/Chapter-001_FragmentHunt.md")).unwrap();
    assert!(report.contains("## Lines 41–50"), "{report}");
    assert!(report.contains("Line 45"), "{report}");
    assert_eq!(
        fs::read_to_string(tmp.path().join("Chapters/Chapter-001.md")).unwrap(),
        chapter,
        "a report never edits the chapter"
    );

    server.reset().await;
    // Apply answers name only the lines they change.
    reply_in_order(
        &server,
        &[
            "NO CHANGES".to_owned(),
            "   45| He walked line 45.".to_owned(),
        ],
    )
    .await;
    let applied = execute_run(run_req(
        &server,
        "fragment-hunter:apply",
        tmp.path().to_path_buf(),
        true,
    ))
    .await
    .unwrap();
    assert_eq!(applied.status, JobStatus::Saved);
    let live = fs::read_to_string(tmp.path().join("Chapters/Chapter-001.md")).unwrap();
    assert!(live.contains("He walked line 45.\n"));
    assert!(live.contains("He just walked line 44.\n"));
    let requests = server.received_requests().await.unwrap();
    assert!(
        requests
            .iter()
            .all(|r| String::from_utf8_lossy(&r.body).contains("Saved report")),
        "every apply window carries the saved report"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn applying_without_a_saved_report_says_what_to_run() {
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();
    fs::create_dir_all(tmp.path().join("Wiki/Style")).unwrap();
    fs::write(
        tmp.path().join("Wiki/Style/genre.md"),
        "---\ngenre: Fantasy\n---\n\n# Genre\n\nEnough text to count as real.\n",
    )
    .unwrap();
    fs::create_dir_all(tmp.path().join("Chapters")).unwrap();
    fs::write(
        tmp.path().join("Chapters/Chapter-001.md"),
        numbered_chapter(5),
    )
    .unwrap();
    let err = execute_run(run_req(
        &server,
        "fiction-line-editor:apply",
        tmp.path().to_path_buf(),
        false,
    ))
    .await
    .unwrap_err();
    assert!(
        err.to_string()
            .contains("run fiction-line-editor and save its report first"),
        "{err}"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn a_refused_apply_leaves_no_job_behind() {
    let server = MockServer::start().await;
    let tmp = tempfile::tempdir().unwrap();
    fs::create_dir_all(tmp.path().join("Chapters")).unwrap();
    fs::write(
        tmp.path().join("Chapters/Chapter-001.md"),
        numbered_chapter(5),
    )
    .unwrap();
    fs::create_dir_all(tmp.path().join("Wiki/Style")).unwrap();
    fs::write(
        tmp.path().join("Wiki/Style/genre.md"),
        "---\ngenre: Fantasy\n---\n\n# Genre\n\nEnough text to count as real.\n",
    )
    .unwrap();
    execute_run(run_req(
        &server,
        "fragment-hunter:apply",
        tmp.path().to_path_buf(),
        false,
    ))
    .await
    .unwrap_err();
    let jobs = tmp.path().join(".storycraft/jobs");
    let left = fs::read_dir(&jobs).map(|d| d.count()).unwrap_or(0);
    assert_eq!(left, 0, "no job stuck in running");
}
