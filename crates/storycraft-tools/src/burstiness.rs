//! Mechanical burstiness stats, ported from `burstiness-check/references/measure.py`.

use std::collections::HashMap;

use serde::Serialize;

/// Sentence-length / opener / dialogue report for one chapter.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BurstinessReport {
    /// Number of sentences with at least one word.
    pub sentence_count: usize,
    /// Paragraphs that were measured (headings and `---` skipped).
    pub paragraph_count: usize,
    /// Mean words per sentence.
    pub sentence_length_mean: f64,
    /// Population stdev of sentence length.
    pub sentence_length_stdev: f64,
    /// `LOW` / `MEDIUM` / `HIGH` from sentence-length CV.
    pub sentence_length_variance_bucket: &'static str,
    /// Mean sentences per paragraph.
    pub paragraph_sentence_count_mean: f64,
    /// Population stdev of paragraph sentence counts.
    pub paragraph_sentence_count_stdev: f64,
    /// `LOW` / `MEDIUM` / `HIGH` from paragraph CV.
    pub paragraph_variance_bucket: &'static str,
    /// Quoted-dialogue words / total words.
    pub dialogue_word_ratio: f64,
    /// Most common sentence openers.
    pub top_openers: Vec<OpenerStat>,
    /// Longest run of the same opener.
    pub longest_same_opener_run: usize,
    /// Repeated content words in 500-word windows.
    pub repetition_hotspots: Vec<RepetitionHotspot>,
    /// Weighted interiority score (0–4).
    pub interiority_risk_score: u8,
    /// `LOW` / `MEDIUM` / `HIGH`.
    pub interiority_risk_level: &'static str,
}

/// One opener word and how often it starts a sentence.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct OpenerStat {
    /// Lowercased opener.
    pub word: String,
    /// Absolute count.
    pub count: usize,
    /// Percent of openers.
    pub pct: f64,
}

/// Words that repeat more than twice inside a 500-word window.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RepetitionHotspot {
    /// Word index where the window starts.
    pub window_start_word: usize,
    /// Word → count inside that window.
    pub hot_words: HashMap<String, usize>,
}

/// Measure burstiness of `text`. Never edits the source.
#[must_use]
pub fn measure(text: &str) -> BurstinessReport {
    let paragraphs = prose_paragraphs(text);
    let mut all_sentences = Vec::new();
    let mut para_sentence_counts = Vec::new();
    for para in &paragraphs {
        let sents = sentences_of(para);
        if !sents.is_empty() {
            para_sentence_counts.push(sents.len());
            all_sentences.extend(sents);
        }
    }

    let mut sent_lengths = Vec::new();
    let mut openers = Vec::new();
    for sent in &all_sentences {
        let words = words_of(sent);
        if !words.is_empty() {
            sent_lengths.push(words.len());
            openers.push(words[0].clone());
        }
    }

    let mut opener_counts: HashMap<String, usize> = HashMap::new();
    for word in &openers {
        *opener_counts.entry(word.clone()).or_insert(0) += 1;
    }
    let total_openers = openers.len().max(1);
    let mut top_openers: Vec<(String, usize)> = opener_counts.into_iter().collect();
    top_openers.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    top_openers.truncate(8);
    let top_openers: Vec<OpenerStat> = top_openers
        .into_iter()
        .map(|(word, count)| OpenerStat {
            pct: round1(100.0 * count as f64 / total_openers as f64),
            word,
            count,
        })
        .collect();

    let mut longest_run = if openers.is_empty() { 0 } else { 1 };
    let mut cur_run = 1usize;
    let mut cur_word: Option<&str> = None;
    for word in &openers {
        if Some(word.as_str()) == cur_word {
            cur_run = cur_run.saturating_add(1);
        } else {
            cur_run = 1;
            cur_word = Some(word);
        }
        longest_run = longest_run.max(cur_run);
    }

    let words = words_of(text);
    let stop = stop_words();
    let mut rep_flags = Vec::new();
    let window = 500usize;
    let start_max = words.len().max(1);
    let mut i = 0usize;
    while i < start_max {
        let end = (i + window).min(words.len());
        let chunk = &words[i..end];
        let mut freq: HashMap<String, usize> = HashMap::new();
        for w in chunk {
            if w.len() < 4 || stop.contains(w.as_str()) {
                continue;
            }
            *freq.entry(w.clone()).or_insert(0) += 1;
        }
        let hot: HashMap<String, usize> = freq.into_iter().filter(|(_, c)| *c > 2).collect();
        if !hot.is_empty() {
            rep_flags.push(RepetitionHotspot {
                window_start_word: i,
                hot_words: hot,
            });
        }
        if i >= words.len() {
            break;
        }
        i = i.saturating_add(window);
        if rep_flags.len() >= 5 {
            break;
        }
    }

    let sent_f: Vec<f64> = sent_lengths.iter().map(|n| *n as f64).collect();
    let para_f: Vec<f64> = para_sentence_counts.iter().map(|n| *n as f64).collect();
    let sent_mean = mean(&sent_f);
    let sent_stdev = pstdev(&sent_f);
    let sent_cv = if sent_mean == 0.0 {
        0.0
    } else {
        sent_stdev / sent_mean
    };
    let para_mean = mean(&para_f);
    let para_stdev = pstdev(&para_f);
    let para_cv = if para_mean == 0.0 {
        0.0
    } else {
        para_stdev / para_mean
    };

    let dialogue_word_count: usize = quote_spans(text).iter().map(|q| words_of(q).len()).sum();
    let total_word_count = words.len().max(1);
    let dialogue_ratio = round3(dialogue_word_count as f64 / total_word_count as f64);

    let top_opener_pct = top_openers
        .first()
        .map(|o| o.count as f64 / total_openers as f64)
        .unwrap_or(0.0);
    let mut interiority_risk_score = 0u8;
    if dialogue_ratio < 0.05 {
        interiority_risk_score = interiority_risk_score.saturating_add(2);
    } else if dialogue_ratio < 0.10 {
        interiority_risk_score = interiority_risk_score.saturating_add(1);
    }
    if top_opener_pct >= 0.20 {
        interiority_risk_score = interiority_risk_score.saturating_add(1);
    }
    if para_mean >= 3.5 {
        interiority_risk_score = interiority_risk_score.saturating_add(1);
    }
    let interiority_risk_level = if interiority_risk_score >= 2 {
        "HIGH"
    } else if interiority_risk_score == 1 {
        "MEDIUM"
    } else {
        "LOW"
    };

    BurstinessReport {
        sentence_count: sent_lengths.len(),
        paragraph_count: paragraphs.len(),
        sentence_length_mean: round2(sent_mean),
        sentence_length_stdev: round2(sent_stdev),
        sentence_length_variance_bucket: bucket(sent_cv),
        paragraph_sentence_count_mean: round2(para_mean),
        paragraph_sentence_count_stdev: round2(para_stdev),
        paragraph_variance_bucket: bucket(para_cv),
        dialogue_word_ratio: dialogue_ratio,
        top_openers,
        longest_same_opener_run: longest_run,
        repetition_hotspots: rep_flags,
        interiority_risk_score,
        interiority_risk_level,
    }
}

fn words_of(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    for c in text.chars() {
        if c.is_ascii_alphabetic() || c == '\'' {
            cur.push(c);
        } else if !cur.is_empty() {
            out.push(std::mem::take(&mut cur).to_ascii_lowercase());
        }
    }
    if !cur.is_empty() {
        out.push(cur.to_ascii_lowercase());
    }
    out
}

fn quote_spans(text: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut rest = text;
    while !rest.is_empty() {
        if let Some(start) = rest.find('"') {
            let after = &rest[start + 1..];
            if let Some(end) = after.find('"') {
                out.push(&rest[start..start + 1 + end + 1]);
                rest = &after[end + 1..];
                continue;
            }
            break;
        }
        break;
    }
    let mut rest = text;
    while !rest.is_empty() {
        if let Some(start) = rest.find('“') {
            let after = &rest[start + '“'.len_utf8()..];
            if let Some(end) = after.find('”') {
                out.push(&rest[start..start + '“'.len_utf8() + end + '”'.len_utf8()]);
                rest = &after[end + '”'.len_utf8()..];
                continue;
            }
            break;
        }
        break;
    }
    out
}

fn stop_words() -> std::collections::HashSet<&'static str> {
    "a an the and or but of to in on at for with is was were be been being \
     it its he she they them his her their i you we not that this as by \
     from up down out into over under so if then than"
        .split_whitespace()
        .collect()
}

fn prose_paragraphs(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut current = String::new();
    for line in text.lines() {
        if line.trim().is_empty() {
            push_para(&mut out, &mut current);
        } else {
            if !current.is_empty() {
                current.push('\n');
            }
            current.push_str(line);
        }
    }
    push_para(&mut out, &mut current);
    out
}

fn push_para(out: &mut Vec<String>, current: &mut String) {
    let trimmed = current.trim();
    if !trimmed.is_empty()
        && !trimmed.starts_with('#')
        && !trimmed.starts_with('!')
        && trimmed != "---"
    {
        out.push(trimmed.to_owned());
    }
    current.clear();
}

fn sentences_of(blob: &str) -> Vec<String> {
    let blob = collapse_ws(blob);
    let chars: Vec<char> = blob.chars().collect();
    let mut out = Vec::new();
    let mut start = 0usize;
    let mut i = 0usize;
    while i < chars.len() {
        if matches!(chars[i], '.' | '!' | '?') {
            let mut j = i + 1;
            while j < chars.len() && matches!(chars[j], '"' | '\'' | '’' | '”') {
                j += 1;
            }
            if j < chars.len() && chars[j].is_whitespace() {
                let mut k = j + 1;
                while k < chars.len() && chars[k].is_whitespace() {
                    k += 1;
                }
                if k < chars.len()
                    && (chars[k].is_ascii_uppercase() || matches!(chars[k], '"' | '“'))
                {
                    let sent: String = chars[start..j].iter().collect();
                    let sent = sent.trim();
                    if !sent.is_empty() {
                        out.push(sent.to_owned());
                    }
                    start = k;
                    i = k;
                    continue;
                }
            }
        }
        i += 1;
    }
    let tail: String = chars[start..].iter().collect();
    let tail = tail.trim();
    if !tail.is_empty() {
        out.push(tail.to_owned());
    }
    out
}

fn collapse_ws(blob: &str) -> String {
    let mut out = String::with_capacity(blob.len());
    let mut prev_space = false;
    for c in blob.chars() {
        if c.is_whitespace() {
            if !prev_space && !out.is_empty() {
                out.push(' ');
            }
            prev_space = true;
        } else {
            prev_space = false;
            out.push(c);
        }
    }
    out
}

fn bucket(cv: f64) -> &'static str {
    if cv < 0.35 {
        "LOW"
    } else if cv < 0.55 {
        "MEDIUM"
    } else {
        "HIGH"
    }
}

fn mean(xs: &[f64]) -> f64 {
    if xs.is_empty() {
        0.0
    } else {
        xs.iter().sum::<f64>() / xs.len() as f64
    }
}

fn pstdev(xs: &[f64]) -> f64 {
    if xs.len() < 2 {
        return 0.0;
    }
    let m = mean(xs);
    let var = xs.iter().map(|x| (x - m) * (x - m)).sum::<f64>() / xs.len() as f64;
    var.sqrt()
}

fn round1(x: f64) -> f64 {
    (x * 10.0).round() / 10.0
}

fn round2(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

fn round3(x: f64) -> f64 {
    (x * 1000.0).round() / 1000.0
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::float_cmp)]
    use super::*;

    #[test]
    fn dialogue_heavy_text_is_low_interiority() {
        let text = "\"Did you see the mark?\" she said. \"A foot off.\"\n\n\
He nodded. \"Copy it anyway.\"\n\n\
\"I will,\" she said. \"Before dawn.\"\n";
        let report = measure(text);
        assert!(report.sentence_count >= 3);
        assert!(report.dialogue_word_ratio > 0.3);
        assert_eq!(report.interiority_risk_level, "LOW");
    }

    #[test]
    fn interior_monologue_flags_high_risk() {
        let text = "She counted the supplies again. She counted them a second time. \
She counted them because the number refused to stay still. \
She told herself the wall was lying. She told herself the table was lying. \
She told herself the harbour had always been this dishonest in the dark. \
The strap bit her shoulder. The strap bit deeper. The strap was a private insult.\n";
        let report = measure(text);
        assert!(report.dialogue_word_ratio < 0.05);
        assert_eq!(report.interiority_risk_level, "HIGH");
        assert!(report.top_openers.iter().any(|o| o.word == "she"));
    }

    #[test]
    fn headings_are_skipped() {
        let text = "# Chapter 1\n\n---\n\nThe tide was wrong.\n";
        let report = measure(text);
        assert_eq!(report.paragraph_count, 1);
        assert_eq!(report.sentence_count, 1);
    }
}
