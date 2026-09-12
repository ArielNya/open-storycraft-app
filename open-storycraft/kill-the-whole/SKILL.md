---
name: kill-the-whole
description: Hunt the intensifier phrase "the whole of [me/him/it/attention]" across a chapter and cut the hollow padding while keeping literal quantitative uses and in-voice dialogue idioms (deletion-first, parallel focused workers, edits in place). "the whole of X" is a frequently overused filler intensifier, especially during high-arousal/high-stress beats. Use when user says "kill-the-whole", "kill the whole", or "strip the whole-of crutch".
metadata:
  author: Fiction Toolkit
  version: 1.0.0
  category: fiction-editing
---

# Kill "the whole of X" — strip the hollow intensifier, keep the real ones

"the whole" is correct when it names the actual entirety of a concrete, countable, or
nameable thing (a crew, a room, a night, a plan) and is filler almost everywhere it's
stretched over a pronoun or a rising sensation to inflate a beat without adding a concrete
image. It shows up disproportionately at moments of escalation (pain, arousal, revelation)
as a stock way to signal totality instead of grounding the beat in a physical detail.

The request arguments = the chapter file(s): a path, a file pattern, or several paths. Resolve it to a
concrete file list before doing anything (skip files that don't contain "the whole").

## What to keep vs. fix

**KEEP (do not touch):**
- **Literal quantitative uses** naming an actual complete set/place/duration: "the whole
  crew," "the whole room," "the whole night," "the whole town," "the whole plan," "the
  whole story," "the whole speech" — anywhere "whole" genuinely refers to the entirety of a
  concrete, nameable thing.
- **In-voice dialogue idiom:** "that's the whole of it," "the whole business," "the whole
  production" when spoken by a character as their natural speech pattern.

**FIX (deletion-first) — everything else, especially:**
- "the whole of me" / "the whole of him" / "the whole of my attention" / "the whole of it"
  (referring to a rising sensation, feeling, or moment) used as a padding intensifier in
  narration. Prefer outright deletion: "the whole of me started to gather" → "I started to
  gather"; "seized the whole of my attention" → "seized my attention."
- Any case where "the whole" appears 3+ times within a few lines or one scene as an
  intensifier — vary or cut the repeats even if individually borderline.

Default move is **deletion**, not a swap to another intensifier. Only do a fuller rewrite
if deletion genuinely loses needed meaning, and ground it in a concrete physical detail
rather than another abstract totalizer.

## Procedure

1. **Resolve the request arguments to files.** Then use the host's available text search to keep only files that
   contain the phrase.
2. **Triage:** search for "the whole" and skim. If rare, fix inline. If spread
   across many files, split into up to 6 roughly equal groups and use one focused worker
   per group in parallel when the host supports workers; otherwise process the groups sequentially.
3. **Each worker gets:** its exact file list; the KEEP/FIX and deletion-first rules
   verbatim; instruction to read each file in 20-line chunks so it sees every hit in full
   context; the global style rules (no em dashes except to interrupt dialogue; complete
   grounded sentences; no metaphorical math/analytical language). It makes the minimal
   in-place edit, touching only the target phrase, and reports a per-file verdict list: each
   hit tagged (KEPT-literal / KEPT-voice / DELETED / REWRITTEN) with before→after for
   changes.
4. **Verify when all workers return:** search again for `"the whole of (me|him|her|it|my|his|her)"` →
   should be near-empty (some clean literal survivors are fine). Worker edits make your
   in-context file state stale; reread before any follow-up edit. Report the consolidated
   change list and before/after count from the host's available text search.
5. Offer to commit or leave staged.
