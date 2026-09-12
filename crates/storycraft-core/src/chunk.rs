//! Host-side 40-line windows for editorial skills.

/// Line window size used by `fiction-line-editor`, `kill-*`, and `fragment-hunter`.
pub const CHUNK_LINES: usize = 40;

/// One non-overlapping window of a chapter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineChunk {
    /// 0-based window index.
    pub index: usize,
    /// 1-based inclusive start line.
    pub start_line: usize,
    /// 1-based inclusive end line.
    pub end_line: usize,
    /// Window text, including original newlines.
    pub text: String,
}

/// Skills the host must chunk instead of sending the whole chapter.
#[must_use]
pub fn is_chunked_skill(skill: &str) -> bool {
    matches!(
        skill,
        "fiction-line-editor"
            | "fragment-hunter"
            | "kill-chapter"
            | "kill-crutch"
            | "kill-flat"
            | "kill-opinion-personification"
            | "kill-person-who"
            | "kill-simile"
            | "kill-soft"
            | "kill-the-whole"
    )
}

/// Split `text` into windows of `window` lines (last window may be shorter).
///
/// Empty input yields no chunks. `window` of 0 is treated as [`CHUNK_LINES`].
#[must_use]
pub fn split_lines(text: &str, window: usize) -> Vec<LineChunk> {
    let window = if window == 0 { CHUNK_LINES } else { window };
    if text.is_empty() {
        return Vec::new();
    }
    let lines: Vec<&str> = text.split_inclusive('\n').collect();
    let mut chunks = Vec::with_capacity(lines.len().div_ceil(window));
    for (index, piece) in lines.chunks(window).enumerate() {
        let start_line = index.saturating_mul(window).saturating_add(1);
        let end_line = start_line.saturating_add(piece.len()).saturating_sub(1);
        let mut body = String::with_capacity(piece.iter().map(|s| s.len()).sum());
        for line in piece {
            body.push_str(line);
        }
        chunks.push(LineChunk {
            index,
            start_line,
            end_line,
            text: body,
        });
    }
    chunks
}

/// Concatenate rewritten windows in order.
#[must_use]
pub fn merge_chunks(chunks: &[impl AsRef<str>]) -> String {
    let mut out = String::new();
    for chunk in chunks {
        out.push_str(chunk.as_ref());
    }
    out
}

/// Use `edited` as the replacement for `original`, unless it is empty.
///
/// Outer markdown fences are stripped. If the model returned nothing useful,
/// the original window is kept so a failed kill-pass cannot blank the chapter.
#[must_use]
pub fn apply_chunk_edit(original: &str, edited: &str) -> String {
    let stripped = strip_outer_fence(edited);
    if stripped.trim().is_empty() {
        return original.to_owned();
    }
    if original.ends_with('\n') && !stripped.ends_with('\n') {
        let mut out = stripped;
        out.push('\n');
        out
    } else {
        stripped
    }
}

fn strip_outer_fence(text: &str) -> String {
    let trimmed = text.trim();
    let Some(rest) = trimmed.strip_prefix("```") else {
        return text.to_owned();
    };
    let rest = rest
        .strip_prefix("markdown")
        .or_else(|| rest.strip_prefix("md"))
        .unwrap_or(rest);
    let rest = rest.trim_start_matches('\r').trim_start_matches('\n');
    let Some(inner) = rest.strip_suffix("```") else {
        return text.to_owned();
    };
    inner.trim_end_matches('\r').trim_end().to_owned()
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;

    fn numbered(n: usize) -> String {
        (1..=n).map(|i| format!("line {i} kept.\n")).collect()
    }

    #[test]
    fn ninety_lines_become_three_windows() {
        let text = numbered(90);
        let chunks = split_lines(&text, CHUNK_LINES);
        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0].start_line, 1);
        assert_eq!(chunks[0].end_line, 40);
        assert_eq!(chunks[1].start_line, 41);
        assert_eq!(chunks[1].end_line, 80);
        assert_eq!(chunks[2].start_line, 81);
        assert_eq!(chunks[2].end_line, 90);
        assert_eq!(
            merge_chunks(&chunks.iter().map(|c| c.text.as_str()).collect::<Vec<_>>()),
            text
        );
    }

    #[test]
    fn kill_pass_replaces_only_the_middle_window() {
        let original = numbered(90);
        let chunks = split_lines(&original, 40);
        let mut edited: Vec<String> = chunks.iter().map(|c| c.text.clone()).collect();
        edited[1] = apply_chunk_edit(&chunks[1].text, "line 41 gone.\nline 42 gone.\n");
        let merged = merge_chunks(&edited);
        assert!(merged.starts_with("line 1 kept.\n"));
        assert!(merged.contains("line 40 kept.\n"));
        assert!(merged.contains("line 41 gone.\n"));
        assert!(!merged.contains("line 41 kept.\n"));
        assert!(merged.contains("line 81 kept.\n"));
        let start: String = original
            .lines()
            .take(40)
            .map(|l| format!("{l}\n"))
            .collect();
        let end: String = original
            .lines()
            .skip(80)
            .map(|l| format!("{l}\n"))
            .collect();
        assert!(merged.starts_with(&start));
        assert!(merged.ends_with(&end));
    }

    #[test]
    fn empty_model_output_keeps_original_chunk() {
        let original = "just a little afraid.\n";
        assert_eq!(apply_chunk_edit(original, "   \n"), original);
        assert_eq!(apply_chunk_edit(original, "```\n```"), original);
    }

    #[test]
    fn strips_markdown_fence_around_a_patch() {
        let original = "he just looked at her.\n";
        let edited = apply_chunk_edit(original, "```markdown\nhe looked at her.\n```");
        assert_eq!(edited, "he looked at her.\n");
    }
}
