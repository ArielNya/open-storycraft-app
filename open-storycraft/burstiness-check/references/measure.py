#!/usr/bin/env python3
"""Mechanical burstiness measurement for one prose file. No dependencies.

Usage: python3 measure.py <path/to/chapter.md>
Prints a JSON object with sentence/paragraph variance stats, sentence-opener
distribution, and word-repetition hotspots. Never edits the file.
"""
import re
import sys
import json
import statistics

SENT_SPLIT = re.compile(r'(?<=[.!?])["\'’”]?\s+(?=[A-Z"“])')
STOP = set(
    "a an the and or but of to in on at for with is was were be been being "
    "it its he she they them his her their i you we not that this as by "
    "from up down out into over under so if then than".split()
)


def sentences_of(blob):
    blob = re.sub(r"\s+", " ", blob).strip()
    return [s.strip() for s in SENT_SPLIT.split(blob) if s.strip()]


def bucket(cv):
    if cv < 0.35:
        return "LOW"
    if cv < 0.55:
        return "MEDIUM"
    return "HIGH"


def measure(path):
    with open(path, encoding="utf-8") as f:
        text = f.read()

    paragraphs = [
        p.strip()
        for p in re.split(r"\n\s*\n", text)
        if p.strip()
        and not p.strip().startswith("#")
        and not p.strip().startswith("!")
        and p.strip() != "---"
    ]

    all_sentences, para_sentence_counts = [], []
    for para in paragraphs:
        sents = sentences_of(para)
        if sents:
            para_sentence_counts.append(len(sents))
            all_sentences.extend(sents)

    sent_lengths = [len(re.findall(r"[A-Za-z']+", s)) for s in all_sentences]
    sent_lengths = [n for n in sent_lengths if n > 0]

    openers = []
    for s in all_sentences:
        m = re.match(r"[A-Za-z']+", s)
        if m:
            openers.append(m.group().lower())

    opener_counts = {}
    for w in openers:
        opener_counts[w] = opener_counts.get(w, 0) + 1
    total_openers = len(openers) or 1
    top_openers = sorted(opener_counts.items(), key=lambda x: -x[1])[:8]

    longest_run, cur_run, cur_word = 1, 1, None
    for w in openers:
        if w == cur_word:
            cur_run += 1
        else:
            cur_run = 1
            cur_word = w
        longest_run = max(longest_run, cur_run)

    words = re.findall(r"[A-Za-z']+", text.lower())
    window = 500
    rep_flags = []
    for i in range(0, max(len(words), 1), window):
        chunk = words[i : i + window]
        freq = {}
        for w in chunk:
            if w in STOP or len(w) < 4:
                continue
            freq[w] = freq.get(w, 0) + 1
        hot = {w: c for w, c in freq.items() if c > 2}
        if hot:
            rep_flags.append({"window_start_word": i, "hot_words": hot})

    sent_mean = statistics.mean(sent_lengths) if sent_lengths else 0
    sent_stdev = statistics.pstdev(sent_lengths) if len(sent_lengths) > 1 else 0
    sent_cv = (sent_stdev / sent_mean) if sent_mean else 0

    para_mean = statistics.mean(para_sentence_counts) if para_sentence_counts else 0
    para_stdev = statistics.pstdev(para_sentence_counts) if len(para_sentence_counts) > 1 else 0
    para_cv = (para_stdev / para_mean) if para_mean else 0

    # Dialogue density: quoted spans per paragraph. Scenes with little/no dialogue
    # lose the "new speaker -> new paragraph" break that naturally shortens
    # paragraphs and diversifies sentence openers (he said / she said / names).
    # Solo, dialogue-light interiority is a known structural risk factor for
    # reading statistically flat, independent of actual writing quality.
    quote_spans = re.findall(r'"[^"]*"|“[^”]*”', text)
    dialogue_word_count = sum(len(re.findall(r"[A-Za-z']+", q)) for q in quote_spans)
    total_word_count = len(words) or 1
    dialogue_ratio = round(dialogue_word_count / total_word_count, 3)

    # Weighted score, not a strict AND-of-conditions. Cross-book validation (Black
    # two contrasting projects, including a same-chapter split at the exact
    # sentence a Pangram verdict flipped from 100% AI to 100% human) showed dialogue
    # ratio alone tracks the AI/human verdict even when a passage is too short or too
    # narration-light to trip paragraph-density/opener-concentration thresholds. Those
    # two remain real reinforcing signals, just secondary — weight dialogue highest.
    top_opener_pct = top_openers[0][1] / total_openers if top_openers else 0
    interiority_risk_score = 0
    if dialogue_ratio < 0.05:
        interiority_risk_score += 2
    elif dialogue_ratio < 0.10:
        interiority_risk_score += 1
    if top_opener_pct >= 0.20:
        interiority_risk_score += 1
    if para_mean >= 3.5:
        interiority_risk_score += 1
    if interiority_risk_score >= 2:
        interiority_risk_level = "HIGH"
    elif interiority_risk_score == 1:
        interiority_risk_level = "MEDIUM"
    else:
        interiority_risk_level = "LOW"

    return {
        "file": path,
        "sentence_count": len(sent_lengths),
        "paragraph_count": len(paragraphs),
        "sentence_length_mean": round(sent_mean, 2),
        "sentence_length_stdev": round(sent_stdev, 2),
        "sentence_length_variance_bucket": bucket(sent_cv),
        "paragraph_sentence_count_mean": round(para_mean, 2),
        "paragraph_sentence_count_stdev": round(para_stdev, 2),
        "paragraph_variance_bucket": bucket(para_cv),
        "dialogue_word_ratio": dialogue_ratio,
        "top_openers": [
            {"word": w, "count": c, "pct": round(100 * c / total_openers, 1)}
            for w, c in top_openers
        ],
        "longest_same_opener_run": longest_run,
        "repetition_hotspots": rep_flags[:5],
        "interiority_risk_score": interiority_risk_score,
        "interiority_risk_level": interiority_risk_level,
    }


if __name__ == "__main__":
    if len(sys.argv) != 2:
        sys.exit("usage: measure.py <path/to/chapter.md>")
    print(json.dumps(measure(sys.argv[1]), indent=2))
