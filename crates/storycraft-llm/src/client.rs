//! HTTP client for `/v1/chat/completions` and `/v1/responses`.

use std::time::Duration;

use futures_util::StreamExt;
use reqwest::StatusCode;
use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue, RETRY_AFTER};
use serde_json::{Value, json};
use tokio_util::sync::CancellationToken;
use tracing::info;

use crate::sse::{SseBuffer, complete_text, delta_text};
use crate::{Error, Secret};

/// Which OpenAI-style endpoint to call.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApiStyle {
    /// `POST /chat/completions`
    ChatCompletions,
    /// `POST /responses` (xAI-preferred)
    Responses,
}

impl ApiStyle {
    /// Canonical CLI token.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ChatCompletions => "chat_completions",
            Self::Responses => "responses",
        }
    }
}

impl std::str::FromStr for ApiStyle {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "chat_completions" | "chat-completions" | "chat" => Ok(Self::ChatCompletions),
            "responses" | "response" => Ok(Self::Responses),
            other => Err(Error::InvalidPayload(format!("unknown api style: {other}"))),
        }
    }
}

/// Provider profile. `api_key` is omitted for local servers that do not need one.
#[derive(Clone)]
pub struct ProviderConfig {
    /// Profile name stored on the job (`openai-compat`, `xai-apikey`, `grok-oauth`).
    pub name: String,
    /// Base URL including `/v1`.
    pub base_url: String,
    /// Bearer token, if any.
    pub api_key: Option<Secret>,
    /// Endpoint dialect.
    pub api_style: ApiStyle,
    /// Model id.
    pub model: String,
    /// Per-request timeout.
    pub timeout: Duration,
    /// When false, parse a single JSON body instead of SSE.
    pub stream: bool,
}

impl std::fmt::Debug for ProviderConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProviderConfig")
            .field("name", &self.name)
            .field("base_url", &self.base_url)
            .field("api_key", &self.api_key)
            .field("api_style", &self.api_style)
            .field("model", &self.model)
            .field("timeout", &self.timeout)
            .field("stream", &self.stream)
            .finish()
    }
}

impl ProviderConfig {
    /// OpenAI-compatible defaults: 120s timeout, streaming chat completions.
    #[must_use]
    pub fn openai_compat(base_url: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            name: "openai-compat".to_owned(),
            base_url: base_url.into(),
            api_key: None,
            api_style: ApiStyle::ChatCompletions,
            model: model.into(),
            timeout: Duration::from_secs(120),
            stream: true,
        }
    }
}

/// Streaming OpenAI-compatible client.
pub struct OpenAiClient {
    http: reqwest::Client,
    config: ProviderConfig,
}

impl OpenAiClient {
    /// Build a client. Does not send a request.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Transport`] if the reqwest client cannot be built.
    pub fn new(config: ProviderConfig) -> Result<Self, Error> {
        let http = reqwest::Client::builder()
            .timeout(config.timeout)
            .build()
            .map_err(Error::Transport)?;
        Ok(Self { http, config })
    }

    /// Provider profile name.
    #[must_use]
    pub fn provider_name(&self) -> &str {
        &self.config.name
    }

    /// Model id.
    #[must_use]
    pub fn model(&self) -> &str {
        &self.config.model
    }

    /// Model ids the provider advertises.
    ///
    /// Calls `GET {base_url}/models` — the OpenAI-compatible listing endpoint.
    /// A server that only answers under `/v1/models` is retried once, so a bare
    /// host URL still works.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Transport`] on network failure, HTTP errors mapped by
    /// status (401 rejected credentials, 403 tier denied), and
    /// [`Error::NoModels`] when the body carries no recognizable model list.
    pub async fn list_models(&self) -> Result<Vec<String>, Error> {
        let base = self.config.base_url.trim_end_matches('/');
        let url = format!("{base}/models");
        match self.fetch_models(&url).await {
            Err(Error::Http { status, .. }) if status == 404 || status == 405 => {
                // Some servers keep the list under the versioned prefix only.
                let versioned = format!("{base}/v1/models");
                self.fetch_models(&versioned).await
            }
            other => other,
        }
    }

    async fn fetch_models(&self, url: &str) -> Result<Vec<String>, Error> {
        let headers = self.auth_headers()?;
        info!(url = %url, "provider model list request");
        let response = self
            .http
            .get(url)
            .headers(headers)
            .send()
            .await
            .map_err(Error::Transport)?;
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(Error::from_status(status, &body));
        }
        let text = response.text().await.map_err(Error::Transport)?;
        let value: Value =
            serde_json::from_str(&text).map_err(|err| Error::InvalidPayload(err.to_string()))?;
        let models = model_ids(&value);
        if models.is_empty() {
            return Err(Error::NoModels);
        }
        Ok(models)
    }

    /// Bearer header for the configured key, marked sensitive so it never
    /// reaches a log.
    fn auth_headers(&self) -> Result<HeaderMap, Error> {
        let mut headers = HeaderMap::new();
        if let Some(key) = &self.config.api_key {
            let value = format!("Bearer {}", key.expose());
            let mut header = HeaderValue::from_str(&value)
                .map_err(|err| Error::InvalidPayload(err.to_string()))?;
            header.set_sensitive(true);
            headers.insert(AUTHORIZATION, header);
        }
        Ok(headers)
    }

    /// Run one completion. `on_delta` is called with each text chunk so the
    /// host can append the preview file while the request is in flight.
    ///
    /// # Errors
    ///
    /// Returns transport, HTTP, cancel, or parse errors. 403 is
    /// [`Error::TierDenied`].
    pub async fn complete(
        &self,
        system: &str,
        user: &str,
        cancel: &CancellationToken,
        mut on_delta: impl FnMut(&str) -> Result<(), Error>,
    ) -> Result<String, Error> {
        let mut attempt = 0u32;
        loop {
            if cancel.is_cancelled() {
                return Err(Error::Cancelled);
            }
            match self.once(system, user, cancel, &mut on_delta).await {
                Err(Error::Http { status: 429, .. }) if attempt < 3 => {
                    attempt = attempt.saturating_add(1);
                    let wait = Duration::from_millis(500u64.saturating_mul(1u64 << (attempt - 1)));
                    info!(
                        attempt,
                        wait_ms = wait.as_millis() as u64,
                        "retrying after 429"
                    );
                    tokio::select! {
                        () = cancel.cancelled() => return Err(Error::Cancelled),
                        () = tokio::time::sleep(wait) => {}
                    }
                }
                other => return other,
            }
        }
    }

    async fn once(
        &self,
        system: &str,
        user: &str,
        cancel: &CancellationToken,
        on_delta: &mut impl FnMut(&str) -> Result<(), Error>,
    ) -> Result<String, Error> {
        let url = endpoint(&self.config.base_url, self.config.api_style);
        let body = request_body(
            self.config.api_style,
            &self.config.model,
            system,
            user,
            self.config.stream,
        );
        let headers = self.auth_headers()?;

        info!(
            url = %url,
            model = %self.config.model,
            style = self.config.api_style.as_str(),
            stream = self.config.stream,
            "provider request"
        );

        let request = self.http.post(&url).headers(headers).json(&body);
        let response = tokio::select! {
            () = cancel.cancelled() => return Err(Error::Cancelled),
            result = request.send() => result.map_err(Error::Transport)?,
        };

        if response.status() == StatusCode::TOO_MANY_REQUESTS {
            let retry_after = parse_retry_after(response.headers());
            let body = response.text().await.unwrap_or_default();
            if let Some(wait) = retry_after.filter(|d| !d.is_zero()) {
                tokio::select! {
                    () = cancel.cancelled() => return Err(Error::Cancelled),
                    () = tokio::time::sleep(wait) => {}
                }
            }
            return Err(Error::Http {
                status: 429,
                body: body.chars().take(400).collect(),
            });
        }

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(Error::from_status(status, &body));
        }

        if self.config.stream {
            self.read_stream(response, cancel, on_delta).await
        } else {
            self.read_json(response, cancel, on_delta).await
        }
    }

    async fn read_json(
        &self,
        response: reqwest::Response,
        cancel: &CancellationToken,
        on_delta: &mut impl FnMut(&str) -> Result<(), Error>,
    ) -> Result<String, Error> {
        let text = tokio::select! {
            () = cancel.cancelled() => return Err(Error::Cancelled),
            result = response.text() => result.map_err(Error::Transport)?,
        };
        let value: Value =
            serde_json::from_str(&text).map_err(|err| Error::InvalidPayload(err.to_string()))?;
        let out = complete_text(&value).ok_or(Error::EmptyCompletion)?;
        if out.is_empty() {
            return Err(Error::EmptyCompletion);
        }
        on_delta(&out)?;
        Ok(out)
    }

    async fn read_stream(
        &self,
        response: reqwest::Response,
        cancel: &CancellationToken,
        on_delta: &mut impl FnMut(&str) -> Result<(), Error>,
    ) -> Result<String, Error> {
        let mut stream = response.bytes_stream();
        let mut sse = SseBuffer::default();
        let mut out = String::new();
        loop {
            let chunk = tokio::select! {
                () = cancel.cancelled() => return Err(Error::Cancelled),
                next = stream.next() => next,
            };
            match chunk {
                None => break,
                Some(Err(err)) => return Err(Error::Transport(err)),
                Some(Ok(bytes)) => {
                    let piece = String::from_utf8_lossy(&bytes);
                    for data in sse.push(&piece) {
                        if let Some(delta) = delta_text(&data) {
                            on_delta(&delta)?;
                            out.push_str(&delta);
                        }
                    }
                }
            }
        }
        if out.is_empty() {
            return Err(Error::EmptyCompletion);
        }
        Ok(out)
    }
}

fn endpoint(base: &str, style: ApiStyle) -> String {
    let base = base.trim_end_matches('/');
    match style {
        ApiStyle::ChatCompletions => format!("{base}/chat/completions"),
        ApiStyle::Responses => format!("{base}/responses"),
    }
}

fn request_body(style: ApiStyle, model: &str, system: &str, user: &str, stream: bool) -> Value {
    match style {
        ApiStyle::ChatCompletions => json!({
            "model": model,
            "stream": stream,
            "messages": [
                {"role": "system", "content": system},
                {"role": "user", "content": user}
            ]
        }),
        ApiStyle::Responses => json!({
            "model": model,
            "stream": stream,
            "instructions": system,
            "input": user
        }),
    }
}

fn parse_retry_after(headers: &HeaderMap) -> Option<Duration> {
    let value = headers.get(RETRY_AFTER)?.to_str().ok()?;
    value.parse::<u64>().ok().map(Duration::from_secs)
}

/// Model ids out of an OpenAI-shaped listing.
///
/// Accepts `{"data": [{"id": …}]}`, `{"models": […]}` (and the same with a
/// `name` key), or a bare array. Sorted and de-duplicated so the picker in the
/// settings view is stable between calls.
fn model_ids(value: &Value) -> Vec<String> {
    let mut ids = Vec::new();
    for key in ["data", "models"] {
        if let Some(list) = value.get(key).and_then(Value::as_array) {
            collect_ids(list, &mut ids);
        }
    }
    if ids.is_empty()
        && let Some(list) = value.as_array()
    {
        collect_ids(list, &mut ids);
    }
    ids.sort();
    ids.dedup();
    ids
}

fn collect_ids(list: &[Value], ids: &mut Vec<String>) {
    for item in list {
        let id = match item {
            Value::String(text) => Some(text.clone()),
            Value::Object(map) => map
                .get("id")
                .or_else(|| map.get("name"))
                .and_then(Value::as_str)
                .map(ToOwned::to_owned),
            _ => None,
        };
        if let Some(id) = id.map(|id| id.trim().to_owned())
            && !id.is_empty()
        {
            ids.push(id);
        }
    }
}
