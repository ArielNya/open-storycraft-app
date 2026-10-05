//! Minimal `text/event-stream` splitter. Buffer lives outside `select!`.

use serde_json::Value;

/// Accumulates bytes and yields `data:` payloads.
#[derive(Debug, Default)]
pub(crate) struct SseBuffer {
    buf: String,
    /// Bytes not decoded yet: a UTF-8 character or a `\r\n` split across
    /// network chunks waits here for the rest of it.
    pending: Vec<u8>,
}

impl SseBuffer {
    pub(crate) fn push(&mut self, chunk: &[u8]) -> Vec<String> {
        self.pending.extend_from_slice(chunk);
        let whole = match std::str::from_utf8(&self.pending) {
            Err(err) if err.error_len().is_none() => err.valid_up_to(),
            _ => self.pending.len(),
        };
        let mut text = String::from_utf8_lossy(&self.pending[..whole]).into_owned();
        self.pending.drain(..whole);
        if text.ends_with('\r') {
            text.pop();
            self.pending.insert(0, b'\r');
        }
        self.buf
            .push_str(&text.replace("\r\n", "\n").replace('\r', "\n"));
        let mut events = Vec::new();
        while let Some(idx) = self.buf.find("\n\n") {
            let block = self.buf[..idx].to_owned();
            self.buf.drain(..idx + 2);
            if let Some(data) = event_data(&block) {
                events.push(data);
            }
        }
        events
    }
}

fn event_data(block: &str) -> Option<String> {
    let mut data = String::new();
    for line in block.lines() {
        if let Some(rest) = line.strip_prefix("data:") {
            if !data.is_empty() {
                data.push('\n');
            }
            data.push_str(rest.trim_start());
        }
    }
    if data.is_empty() { None } else { Some(data) }
}

/// Extract a text delta from a chat-completions or Responses SSE payload.
#[must_use]
pub(crate) fn delta_text(data: &str) -> Option<String> {
    let data = data.trim();
    if data == "[DONE]" {
        return None;
    }
    let value: Value = serde_json::from_str(data).ok()?;
    // Google's OpenAI endpoint streams a model's reasoning as ordinary content,
    // flagged `extra_content.google.thought` and wrapped in `<thought>` tags.
    // It is not part of the answer.
    if value
        .pointer("/choices/0/delta/extra_content/google/thought")
        .and_then(Value::as_bool)
        == Some(true)
    {
        return None;
    }
    if let Some(text) = value
        .pointer("/choices/0/delta/content")
        .and_then(Value::as_str)
    {
        let text = text.strip_prefix("</thought>").unwrap_or(text);
        if !text.is_empty() {
            return Some(text.to_owned());
        }
    }
    let kind = value.get("type").and_then(Value::as_str).unwrap_or("");
    if (kind == "response.output_text.delta" || kind.ends_with("output_text.delta"))
        && let Some(text) = value.get("delta").and_then(Value::as_str)
        && !text.is_empty()
    {
        return Some(text.to_owned());
    }
    if let Some(text) = value.get("delta").and_then(Value::as_str)
        && !text.is_empty()
        && value.get("choices").is_none()
    {
        return Some(text.to_owned());
    }
    None
}

/// Pull the full text out of a non-stream JSON body.
#[must_use]
pub(crate) fn complete_text(value: &Value) -> Option<String> {
    if let Some(text) = value
        .pointer("/choices/0/message/content")
        .and_then(Value::as_str)
    {
        return Some(text.to_owned());
    }
    if let Some(text) = value.get("output_text").and_then(Value::as_str) {
        return Some(text.to_owned());
    }
    value
        .pointer("/output/0/content/0/text")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;

    #[test]
    fn splits_double_newline_events() {
        let mut buf = SseBuffer::default();
        let events = buf.push(b"data: {\"delta\":\"He\"}\n\ndata: {\"delta\":\"llo\"}\n\n");
        assert_eq!(events, vec![r#"{"delta":"He"}"#, r#"{"delta":"llo"}"#]);
    }

    #[test]
    fn extracts_chat_delta() {
        let data = r#"{"choices":[{"delta":{"content":"Hi"}}]}"#;
        assert_eq!(delta_text(data).as_deref(), Some("Hi"));
        assert!(delta_text("[DONE]").is_none());
    }

    #[test]
    fn extracts_responses_delta() {
        let data = r#"{"type":"response.output_text.delta","delta":"Yo"}"#;
        assert_eq!(delta_text(data).as_deref(), Some("Yo"));
    }

    #[test]
    fn google_thought_chunks_are_not_answer_text() {
        let thought = r#"{"choices":[{"delta":{"content":"<thought>plan","extra_content":{"google":{"thought":true}}}}]}"#;
        assert!(delta_text(thought).is_none());
        let answer = r#"{"choices":[{"delta":{"content":"</thought>---\ntitle: x"}}]}"#;
        assert_eq!(delta_text(answer).as_deref(), Some("---\ntitle: x"));
    }

    #[test]
    fn a_character_split_across_chunks_survives() {
        let event = "data: {\"delta\":\"coração\"}\n\n".as_bytes();
        // Cut inside the two-byte "ç".
        let cut = event.iter().position(|&b| b == 0xC3).unwrap() + 1;
        let mut buf = SseBuffer::default();
        assert!(buf.push(&event[..cut]).is_empty());
        assert_eq!(buf.push(&event[cut..]), vec![r#"{"delta":"coração"}"#]);
    }

    #[test]
    fn a_crlf_split_across_chunks_is_one_line_break() {
        let mut buf = SseBuffer::default();
        assert!(buf.push(b"data: {\"delta\":\"a\"}\r").is_empty());
        assert!(
            buf.push(b"\ndata: x").is_empty(),
            "one CRLF must not end the event"
        );
        assert_eq!(buf.push(b"\r\n\r\n"), vec!["{\"delta\":\"a\"}\nx"]);
    }
}
