//! Wiremock coverage for chat completions, Responses, 429, and 403.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::time::Duration;

use storycraft_llm::{ApiStyle, OpenAiClient, ProviderConfig, Secret};
use tokio_util::sync::CancellationToken;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn cfg(server: &MockServer, style: ApiStyle) -> ProviderConfig {
    ProviderConfig {
        name: "openai-compat".into(),
        base_url: format!("{}/v1", server.uri()),
        api_key: Some(Secret::new("sk-test")),
        api_style: style,
        model: "mock-model".into(),
        timeout: Duration::from_secs(5),
        stream: true,
    }
}

#[tokio::test(flavor = "current_thread")]
async fn streams_chat_completion_deltas() {
    let server = MockServer::start().await;
    let body = "data: {\"choices\":[{\"delta\":{\"content\":\"Hello\"}}]}\n\n\
                data: {\"choices\":[{\"delta\":{\"content\":\" world\"}}]}\n\n\
                data: [DONE]\n\n";
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .and(header("authorization", "Bearer sk-test"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_string(body),
        )
        .mount(&server)
        .await;

    let client = OpenAiClient::new(cfg(&server, ApiStyle::ChatCompletions)).unwrap();
    let mut seen = String::new();
    let text = client
        .complete("sys", "user", &CancellationToken::new(), |delta| {
            seen.push_str(delta);
            Ok(())
        })
        .await
        .unwrap();
    assert_eq!(text, "Hello world");
    assert_eq!(seen, "Hello world");
}

#[tokio::test(flavor = "current_thread")]
async fn streams_responses_deltas() {
    let server = MockServer::start().await;
    let body = "event: response.output_text.delta\n\
                data: {\"type\":\"response.output_text.delta\",\"delta\":\"Night\"}\n\n\
                data: {\"type\":\"response.output_text.delta\",\"delta\":\" Market\"}\n\n";
    Mock::given(method("POST"))
        .and(path("/v1/responses"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_string(body),
        )
        .mount(&server)
        .await;

    let client = OpenAiClient::new(cfg(&server, ApiStyle::Responses)).unwrap();
    let text = client
        .complete("sys", "user", &CancellationToken::new(), |_| Ok(()))
        .await
        .unwrap();
    assert_eq!(text, "Night Market");
}

#[tokio::test(flavor = "current_thread")]
async fn retries_429_then_succeeds() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(
            ResponseTemplate::new(429)
                .insert_header("retry-after", "0")
                .set_body_string("rate limited"),
        )
        .up_to_n_times(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_string("data: {\"choices\":[{\"delta\":{\"content\":\"ok\"}}]}\n\n"),
        )
        .mount(&server)
        .await;

    let client = OpenAiClient::new(cfg(&server, ApiStyle::ChatCompletions)).unwrap();
    let text = client
        .complete("s", "u", &CancellationToken::new(), |_| Ok(()))
        .await
        .unwrap();
    assert_eq!(text, "ok");
}

#[tokio::test(flavor = "current_thread")]
async fn maps_403_to_tier_denied() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(403).set_body_string(r#"{"error":"tier denied"}"#))
        .mount(&server)
        .await;

    let client = OpenAiClient::new(cfg(&server, ApiStyle::ChatCompletions)).unwrap();
    let err = client
        .complete("s", "u", &CancellationToken::new(), |_| Ok(()))
        .await
        .unwrap_err();
    assert!(matches!(err, storycraft_llm::Error::TierDenied));
}

#[tokio::test(flavor = "current_thread")]
async fn cancel_before_send() {
    let server = MockServer::start().await;
    let token = CancellationToken::new();
    token.cancel();
    let client = OpenAiClient::new(cfg(&server, ApiStyle::ChatCompletions)).unwrap();
    let err = client
        .complete("s", "u", &token, |_| Ok(()))
        .await
        .unwrap_err();
    assert!(matches!(err, storycraft_llm::Error::Cancelled));
}
