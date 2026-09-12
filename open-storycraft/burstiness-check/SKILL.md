---
name: burstiness-check
description: Diagnostic-only mechanical proxy for finding chapters or scenes likely to
  read machine-generated and deciding where to focus a closer read. Measures sentence
  and paragraph variance, repeated sentence openings, word repetition, dialogue ratio,
  and dialogue-light interiority risk against the book's own baseline. Its statistics
  correlate with detector outcomes but do not claim to reproduce Pangram's learned
  transformer mechanism. Routes phrasing tells to fiction-aiism-editor before mechanics
  to fiction-prose-editor; never rewrites prose or chases unpredictability for its own
  sake. Use for burstiness checks, robotic-reading chapters, Pangram failures, or
  burstiness-check with chapter references.
metadata:
  author: Fiction Toolkit
  version: 1.3.0
  category: fiction-editing
---

# Burstiness Check

A measurement pass, not a rewrite pass. This skill finds *which* chapters or scenes are
statistically flat and *why*, in plain numbers, then routes the fix to the right existing
skill instead of inventing "sound less predictable" rewrites (that path produces purple
prose fighting the project's actual style rules — don't go there).

**Important caveat on the name — read before trusting this as ground truth.** Pangram's own
technical report (Emi & Spero, 2024, arXiv:2402.14873) states explicitly that Pangram is
*not* a perplexity/burstiness classifier — that's the older GPTZero approach, which the
paper positions itself against. Pangram is a transformer trained via hard-negative-mining
on matched human/AI "mirror" pairs to learn *learned stylistic tells* (generic phrasing,
LLM-typical constructions, the paper's own example: overusing "delve" or "it is important
to note") — not literal sentence-length variance. This skill's mechanical metrics
(sentence/paragraph stats, dialogue ratio) are a **proxy that correlated with four real
verdicts in testing**, not a measurement of Pangram's actual mechanism. The likely real
story: dialogue-light solo narration is where an LLM tends to lean on generic phrasing and
learned tells (no character-specific speech pattern to anchor it), and this skill's
dialogue-ratio signal is probably picking up a correlate of that content problem, not
causing the flat read itself via statistics. Treat every finding from this skill as "worth
a closer read for AI-tell phrasing," not as a verified diagnosis of *why* a detector flagged
something.

**This skill never edits text.** It reports. If the user wants fixes applied, tell them
which existing skill to run (`fiction-aiism-editor` for AI-tell phrasing — the more likely
real culprit per the paper above — or `fiction-prose-editor` for mechanics) and on which
specific files.

## Arguments

the request arguments — one or more chapter references (e.g., "12", "Chapter-12", "12-15", "Chapter_01
Chapter_02"). If omitted, ask which chapters to check — do not silently scan the whole book.

## Step 1: Resolve Target Files

1. Parse the request arguments into a chapter list (accept numbers, ranges, filenames).
2. Search the project files for matching files (e.g., `**/Chapter*{N}*`, `**/Chapter_{NN}*`). Some projects
   split a chapter into multiple scene files (e.g., `Chapter_01_Scene_1_*.md`,
   `Chapter_01_Scene_2_*.md`) — treat each scene file as its own unit for the report, but
   also roll them up per chapter.
3. If nothing matches, ask the user for the exact path.
4. If only ONE file total is being checked, still run the measurement, but tell the user
   up front that outlier detection needs at least 2-3 files to have a baseline to compare
   against — a single file only gets absolute numbers, not a flagged/clean verdict.

## Step 2: Run the Measurement Script

For each resolved file, run [references/measure.py](references/measure.py) with the host's available Python interpreter and the absolute path to the file as its sole argument.

It's stdlib-only, never edits the file, and prints one JSON object per run. Run it once
per resolved file and collect all the JSON results.

## Step 3: Compute Cross-File Baseline (if 2+ files)

From the collected JSON results:
- Compute the book/set's own mean and stdev for `sentence_length_stdev` and `paragraph_sentence_count_stdev` across all checked files.
- Flag any file more than ~1 stdev below the set's mean on `sentence_length_stdev` — that's the file reading statistically flatter than its siblings, which is the actual "why does this one fail Pangram and the others don't" answer.
- Flag any file whose `top_openers` has an entry over 20% or a `longest_same_opener_run` ≥ 4 — repetitive openings are the single most human-legible burstiness signal.
- Flag any file with `repetition_hotspots` non-empty — surface the specific hot words.
- Check `interiority_risk_level` on every file (HIGH/MEDIUM/LOW, driven mostly by
  `dialogue_word_ratio`, reinforced by paragraph density and opener concentration). This is
  a **structural** risk factor, not a craft failure: dialogue-light passages lose the
  paragraph-break and opener-diversity that dialogue gets for free (every speaker change
  forces a new paragraph and a new opener/name). A HIGH flag means the passage's *shape* —
  not its word choices — is what's reading flat.
  Cross-project validation (all four correctly classified): a dialogue-light chapter scored HIGH (0% dialogue); a dialogue-light chapter that read
  100% human scored LOW (23.6% dialogue); and — the sharpest test — a single chapter that shifted sharply mid-chapter, exactly at a line of dialogue, scored
  HIGH before that line (3.5% dialogue, too short a passage to trip paragraph/opener
  thresholds on its own) and LOW after (23% dialogue). Dialogue ratio alone tracked that
  flip correctly even when paragraph density and opener concentration didn't — weight
  `dialogue_word_ratio` as the primary signal, the others as reinforcing, not required.

## Step 4: Report

```
## Burstiness Check: [Chapter/file list]

### Baseline (this set)
- Sentence-length stdev: mean {X}, range {min}–{max}
- Paragraph-length stdev: mean {X}, range {min}–{max}

### Per-file results
| File | Sent. length μ/σ | Variance | Para variance | Dialogue % | Top opener | Longest same-opener run | Interiority risk |
|------|------------------|----------|----------------|------------|------------|--------------------------|-------------------|
| ... one row per file ... |

### Flagged as statistically flat (below this set's own baseline)
- {file}: {specific reason — e.g. "sentence-length stdev 2.1 vs set mean 5.4", "38% of sentences open with 'I'", "'shoulder' used 4x in one 500-word window"}

### Flagged as structurally at-risk (interiority_risk_level = HIGH)
- {file}: {dialogue %, plus whichever of paragraph density / top-opener % pushed the score
  up} — this passage's *shape*, not its word choices, is driving the flat read. If the file
  is long, consider whether the risk is localized (a dialogue-free stretch within an
  otherwise dialogue-heavy scene) rather than the whole file — re-run the script on just
  that stretch to confirm before recommending a global rewrite.

### Recommendation
For statistically-flat files (word/sentence level), name which existing skill to run:
- Repetitive openers / uniform sentence rhythm → `fiction-prose-editor` on `{file}`
- Repeated words/phrases, generic AI-favored phrasing → `fiction-aiism-editor` on `{file}`
- Both → run both, prose-editor first

For structurally-at-risk files (`interiority_risk_level: HIGH`), lead with
`fiction-aiism-editor` on `{file}` first, not the mechanics pass. Per Pangram's own technical
report (see caveat above), the classifier is trained to catch learned LLM phrasing tells,
not sentence-length statistics — and dialogue-light solo narration is exactly where those
tells (generic constructions, filler phrasing) tend to cluster, since there's no
character-specific voice to anchor against. This skill's structural metrics point at
*where* to look closely, not *what's* wrong. After the aiism pass, still recommend these
shape-level fixes in the POV character's own voice, since they're good craft regardless:
1. **Break paragraphs more often during solo beats.** A new physical action, a new sensory
   beat, or a shift in what he's focused on is a valid paragraph break even without a new
   speaker. Don't let interior narration run 4+ sentences per paragraph by default.
2. **Vary sentence openers away from the first-person pronoun even when "I" is still the
   actor.** Lead with the concrete action, object, or sensation instead: "Salt mud sucked at
   my feet" instead of "I felt the salt mud." This isn't rule-of-threes/AI-tell hunting, it's
   opener variety specific to solo first-person scenes.
3. If the scene structurally allows it, consider whether a beat of dialogue (even one line,
   even self-talk or a shouted aside) is missing and would help naturally.

Do NOT suggest rewriting toward "unpredictable" word choices for their own sake. The fix
is always: catch actual AI-tell phrasing first (`fiction-aiism-editor`), then vary sentence
length/opening naturally in the POV character's voice — not statistical padding for its own
sake.
```

## Notes for the agent running this skill

- This is diagnostic only. Never propose specific rewritten sentences yourself — that's
  the job of `fiction-prose-editor` / `fiction-aiism-editor`, which check against the
  project's actual style guide voice rules. This skill's only job is pointing at *which*
  file and *why* it's statistically flat.
- Variance buckets (LOW/MEDIUM/HIGH) are relative heuristics (coefficient of variation
  thresholds), not calibrated against real Pangram internals — treat them as a compass,
  not a verdict. The cross-file baseline comparison (Step 3) is the actually reliable
  signal, since it compares the book against itself rather than an absolute scale.
- If the user only ever passes one file, say so plainly: no baseline, only absolute
  numbers, and the LOW/MEDIUM/HIGH buckets are a rough guess, not a verdict.

