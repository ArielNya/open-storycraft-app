---
name: kill-crutch
description: Hunt any crutch/filler word across a chapter and remove the bad uses while keeping literal and in-voice ones (deletion-first, parallel focused workers, edits in place). The generalized kill pass for filler words (just, little, really, almost, slightly, etc.). Use when user says "kill-crutch WORD", "kill the word X", or "strip filler word X".
metadata:
  author: Fiction Toolkit
  version: 1.0.0
  category: fiction-editing
---

# Kill a crutch word — strip the filler, keep the real ones

The first token of the request arguments is the **target word** (e.g. `just`, `little`, `really`,
`almost`, `slightly`, `seemed`). Anything after it is the **scope**: a chapter file path, a
file pattern, or several paths. This is the generalized version of `kill-soft` and `kill-flat`
for any filler word the model overuses.

The principle is always the same: a crutch word is correct in a few literal or in-voice
senses and lazy filler everywhere else, especially when it hedges, softens, or pads a
sentence that is stronger without it.

## The deletion-first rule (most important)

**In most cases the bad word can simply be DELETED without changing the meaning.** Prefer
outright deletion over swapping in another adjective or expanding the sentence. Only do a
fuller rewrite (grounding the beat in a concrete physical detail) if deletion genuinely
loses needed meaning. Never replace one filler word with another single filler word.

Examples:
- "he just looked at her" → "he looked at her"
- "a little bit afraid" → "afraid"
- "it really mattered" → "it mattered"
- "she almost seemed to smile" → "she smiled" (or commit to the hedge only if the doubt is the point)

## What to keep vs. fix

Judgment depends on the word, but the shape holds. Before fanning out, decide the **KEEP
set** for this word and put it in the worker instructions. General guidance:

- **KEEP literal senses** — the word carrying real meaning ("a little house" = small;
  "just one left" = exactly one; "really" inside dialogue a character would actually say).
- **KEEP genuine in-voice uses** — filler that is the POV character's authentic verbal tic
  in dialogue or close narration, used sparingly and on purpose.
- **FIX everything else** — the word hedging or padding ("just," "really," "quite," "a
  bit"), softening a verb, or appearing repeatedly within a few lines.

State your proposed keep/fix split for the requested word in one line, then proceed (the
user can correct mid-run).

## Procedure

1. **Parse the request arguments:** first token = WORD; remainder = scope. Resolve scope to files,
   then use the host's available text search to keep only files that contain the word.
2. **Triage:** search for whole-word matches across the files and skim. If rare, fix inline. If spread
   across many files, split into up to 6 roughly equal groups and use one focused worker
   per group in parallel when the host supports workers; otherwise process the groups sequentially.
3. **Each worker gets:** its exact file list; the WORD; the KEEP/FIX and deletion-first
   rules verbatim; instruction to read each file in 20-line chunks so it sees every hit in
   full context; the global style rules (no em dashes except to interrupt dialogue; complete
   grounded sentences; no metaphorical math/analytical language). It makes the minimal
   in-place edit, touching ONLY the target word, and reports a per-file verdict list: each
   hit tagged (KEPT-literal / KEPT-voice / DELETED / REWRITTEN) with before→after for changes.
4. **Verify when all workers return** and consolidate. Search again for rewrite residue (workers
   that expanded instead of deleting) and convert those to plain deletions. Worker edits
   make your in-context file state stale; reread before any follow-up edit. Report the
   consolidated change list and before/after count from the host's available text search.
5. Offer to commit or leave staged.

## Notes

- Never blanket-ban the bare word; it usually has legit literal uses. Target only the
  hedging/padding collocations.
