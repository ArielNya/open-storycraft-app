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

/// Suffix that turns a report skill into its "apply the saved report" run:
/// `fragment-hunter:apply`.
pub const APPLY_SUFFIX: &str = ":apply";

/// Editorial skills whose SKILL.md writes a findings report beside the chapter
/// and only edits the chapter when the user accepts it.
#[must_use]
pub fn is_report_skill(skill: &str) -> bool {
    report_suffix(skill).is_some()
}

/// `Chapter-001` + this + `.md` is where the skill's report is saved.
#[must_use]
pub fn report_suffix(skill: &str) -> Option<&'static str> {
    match skill {
        "fiction-line-editor" => Some("_LineEdit"),
        "fragment-hunter" => Some("_FragmentHunt"),
        _ => None,
    }
}

/// The report skill an `<skill>:apply` run applies, if `skill` is one.
#[must_use]
pub fn applied_report_skill(skill: &str) -> Option<&str> {
    skill
        .strip_suffix(APPLY_SUFFIX)
        .filter(|base| is_report_skill(base))
}

/// The skill folder a run uses: `fragment-hunter:apply` runs `fragment-hunter`.
#[must_use]
pub fn base_skill(skill: &str) -> &str {
    applied_report_skill(skill).unwrap_or(skill)
}

/// Skills the host must chunk instead of sending the whole chapter.
#[must_use]
pub fn is_chunked_skill(skill: &str) -> bool {
    matches!(
        base_skill(skill),
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

/// Answer a report window gives when it found nothing.
pub const NOTHING_IN_WINDOW: &str = "NOTHING IN THIS WINDOW";

/// The window with each line prefixed by its chapter line number, so findings
/// cite the same numbers the author sees in their editor.
#[must_use]
pub fn numbered_window(chunk: &LineChunk) -> String {
    let mut out = String::with_capacity(chunk.text.len() + chunk.text.len() / 8);
    for (offset, line) in chunk.text.lines().enumerate() {
        out.push_str(&format!("{:>5}| {line}\n", chunk.start_line + offset));
    }
    out
}

/// Answer an apply window gives when no fix falls inside it.
pub const NO_CHANGES: &str = "NO CHANGES";

/// Apply an answer of numbered replacement lines (`  5| new text`) to `chunk`.
///
/// Only the numbered lines inside the window change; every other line is the
/// author's, byte for byte — a model cannot drop a heading it was never asked
/// to return. Returns `None` when the answer is not in that form (the caller
/// then falls back to treating it as whole-window text).
#[must_use]
pub fn apply_numbered_edits(chunk: &LineChunk, answer: &str) -> Option<String> {
    let answer = strip_outer_fence(answer);
    let answer = answer.trim();
    if answer == NO_CHANGES {
        return Some(chunk.text.clone());
    }
    let mut edits = std::collections::BTreeMap::new();
    for line in answer.lines().filter(|line| !line.trim().is_empty()) {
        let (number, text) = line.trim_start().split_once('|')?;
        let number: usize = number.trim().parse().ok()?;
        if (chunk.start_line..=chunk.end_line).contains(&number) {
            edits.insert(number, text.strip_prefix(' ').unwrap_or(text).trim_end());
        }
    }
    if edits.is_empty() {
        return None;
    }
    let mut out = String::with_capacity(chunk.text.len());
    for (offset, line) in chunk.text.split_inclusive('\n').enumerate() {
        match edits.get(&(chunk.start_line + offset)) {
            Some(text) => {
                out.push_str(text);
                // Keep the line's own ending: `\r\n`, `\n`, or none at EOF.
                let content = line.trim_end_matches(['\r', '\n']);
                out.push_str(&line[content.len()..]);
            }
            None => out.push_str(line),
        }
    }
    Some(out)
}

/// One report out of per-window findings, each under its line range.
#[must_use]
pub fn merge_window_reports(title: &str, windows: &[(LineChunk, String)]) -> String {
    let mut out = format!("# {title}\n\n");
    let mut found = false;
    for (chunk, answer) in windows {
        let answer = unwrap_model_output(answer);
        let answer = answer.trim();
        if answer.is_empty() || answer == NOTHING_IN_WINDOW {
            continue;
        }
        found = true;
        out.push_str(&format!(
            "## Lines {}–{}\n\n{answer}\n\n",
            chunk.start_line, chunk.end_line
        ));
    }
    if !found {
        out.push_str("No findings: nothing in this chapter needed changing.\n");
    }
    out
}

/// Use `edited` as the replacement for `original`, unless it is empty or is
/// not an edit of `original` at all.
///
/// Outer markdown fences are stripped. If the model returned nothing useful,
/// the original window is kept so a failed kill-pass cannot blank the chapter.
/// The same goes for an answer that shares under half its words with the
/// window: models sometimes follow the skill's report format instead of the
/// host's "return the edited lines" contract, and saving that would replace
/// the prose with a critique of it.
#[must_use]
pub fn apply_chunk_edit(original: &str, edited: &str) -> String {
    let stripped = strip_outer_fence(edited);
    if stripped.trim().is_empty() || chunk_edit_refused(original, edited) {
        return original.to_owned();
    }
    let mut out = stripped;
    if original.ends_with('\n') && !out.ends_with('\n') {
        out.push('\n');
    }
    // Models answer with `\n`. A chapter saved on Windows keeps its `\r\n`, so
    // the diff shows the edit and not every line of the window.
    if original.contains("\r\n") && !out.contains("\r\n") {
        out = out.replace('\n', "\r\n");
    }
    out
}

/// Whether [`apply_chunk_edit`] throws `edited` away because it is not an edit
/// of `original` (a report, a summary, a different text). Hosts count these to
/// tell the user why a pass changed nothing.
#[must_use]
pub fn chunk_edit_refused(original: &str, edited: &str) -> bool {
    let stripped = strip_outer_fence(edited);
    // ponytail: word-overlap heuristic; a deliberate full rewrite of a window
    // would also be refused. Fine for line-level passes, which is all that
    // runs chunked.
    !stripped.trim().is_empty()
        && similar::TextDiff::from_words(original, stripped.as_str()).ratio() < 0.5
}

/// A whole-file model answer, minus the ```` ```markdown ```` fence some models
/// wrap it in. Text that is not one fenced block comes back unchanged.
#[must_use]
pub fn unwrap_model_output(text: &str) -> String {
    let inner = strip_outer_fence(text);
    if inner == text { inner } else { inner + "\n" }
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

    #[test]
    fn numbered_edits_replace_only_their_lines() {
        let text = "# Chapter 1\r\n\r\nCold. Wet. Late.\r\n\r\nShe was. Not tonight.\r\n";
        let chunk = split_lines(text, 40).remove(0);
        let out = apply_numbered_edits(
            &chunk,
            "    3| In the cold, wet, late air.\n  5| She was, not tonight.\n",
        )
        .unwrap();
        assert_eq!(
            out, "# Chapter 1\r\n\r\nIn the cold, wet, late air.\r\n\r\nShe was, not tonight.\r\n",
            "heading and blank lines stay, endings stay"
        );
        assert_eq!(apply_numbered_edits(&chunk, "NO CHANGES").unwrap(), text);
        assert_eq!(
            apply_numbered_edits(&chunk, "```\n3| x\n99| outside the window\n```").unwrap(),
            "# Chapter 1\r\n\r\nx\r\n\r\nShe was. Not tonight.\r\n"
        );
        assert!(apply_numbered_edits(&chunk, "# Chapter 1\n\nWhole text back.\n").is_none());
    }

    #[test]
    fn an_edit_keeps_the_windows_line_endings() {
        let original = "One line.\r\nCold. Wet.\r\nLast line.\r\n";
        let edited = "One line.\nCold and wet.\nLast line.\n";
        assert_eq!(
            apply_chunk_edit(original, edited),
            "One line.\r\nCold and wet.\r\nLast line.\r\n"
        );
        let unix = "One line.\nCold. Wet.\nLast line.\n";
        assert_eq!(apply_chunk_edit(unix, edited), edited);
    }

    #[test]
    fn report_skills_and_their_apply_runs() {
        assert!(is_report_skill("fragment-hunter"));
        assert!(!is_report_skill("kill-simile"));
        assert_eq!(
            applied_report_skill("fragment-hunter:apply"),
            Some("fragment-hunter")
        );
        assert_eq!(
            applied_report_skill("kill-simile:apply"),
            None,
            "only reports apply"
        );
        assert_eq!(
            base_skill("fiction-line-editor:apply"),
            "fiction-line-editor"
        );
        assert!(is_chunked_skill("fragment-hunter:apply"));
        assert_eq!(
            crate::output::skill_output("fragment-hunter"),
            crate::output::SkillOutput::ChapterReport("_FragmentHunt")
        );
        assert_eq!(
            crate::output::skill_output("fragment-hunter:apply"),
            crate::output::SkillOutput::ChapterRewrite
        );
        assert_eq!(
            crate::output::resolve_output_path(None, "fiction-line-editor", 3).as_deref(),
            Some("Chapters/Chapter-003_LineEdit.md")
        );
    }

    #[test]
    fn window_reports_merge_under_their_line_ranges() {
        let chunks = split_lines(&numbered(50), 40);
        assert_eq!(
            numbered_window(&chunks[1]).lines().next(),
            Some("   41| line 41 kept.")
        );
        let report = merge_window_reports(
            "Fragment hunt",
            &[
                (chunks[0].clone(), NOTHING_IN_WINDOW.to_owned()),
                (
                    chunks[1].clone(),
                    "```markdown\n**Line 44:** `Cold.` — kill\n```".to_owned(),
                ),
            ],
        );
        assert!(report.starts_with("# Fragment hunt\n"));
        assert!(!report.contains("Lines 1–40"), "empty windows are left out");
        assert!(
            report.contains("## Lines 41–50\n\n**Line 44:**"),
            "{report}"
        );
        let empty = merge_window_reports("x", &[(chunks[0].clone(), NOTHING_IN_WINDOW.to_owned())]);
        assert!(empty.contains("No findings"));
    }

    #[test]
    fn a_report_instead_of_an_edit_keeps_the_prose() {
        let original = "The tide was wrong. Mira counted it twice.\n\nShe walked the long pier. Cold. Wet. Late.\n\nShe was. She did not care. Not tonight.\n";
        let report = "## Fragment Hunter: Chapter_01.md\n\n**Kills: 4** | **Spared: 1**\n\n### Kills\n\n**Line 5:** `Cold.`\nWords: 1 | Voice Actor Test: FAIL\nContext: She walked the long pier. Cold. Wet. Late.\nFix: She walked the long pier in the cold, wet, late-night air.\n\n### Score Card\n\n| Stat | Count |\n|------|-------|\n| Total kills | 4 |\n";
        assert_eq!(apply_chunk_edit(original, report), original);
        let edit = "The tide was wrong. Mira counted it twice.\n\nShe walked the long pier in the cold, wet, late-night air.\n\nShe was. She did not care, not tonight.\n";
        assert_eq!(apply_chunk_edit(original, edit), edit);
    }

    #[test]
    fn a_fenced_whole_file_answer_is_unwrapped() {
        let fenced = "```markdown\n---\ntitle: x\n---\n\n# Style\n```\n";
        assert_eq!(
            unwrap_model_output(fenced),
            "---\ntitle: x\n---\n\n# Style\n"
        );
        let plain = "---\ntitle: x\n---\n\nUse ``` for code.\n";
        assert_eq!(unwrap_model_output(plain), plain);
    }

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
        let rewrite = chunks[1]
            .text
            .replace("line 41 kept.", "line 41 gone.")
            .replace("line 42 kept.", "line 42 gone.");
        edited[1] = apply_chunk_edit(&chunks[1].text, &rewrite);
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
