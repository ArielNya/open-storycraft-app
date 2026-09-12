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
