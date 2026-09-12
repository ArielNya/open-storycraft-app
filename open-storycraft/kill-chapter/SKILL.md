---
name: kill-chapter
description: One-pass cleanup bot that strips the recurring model-generated prose crutches from a chapter by running four kill passes consecutively — similes, generic filler words, "soft", and "flat". Each pass fans out parallel focused workers, edits in place deletion-first, and verifies. Book-agnostic. Use when user says "kill-chapter", "clean this chapter", "run the kill passes", or "strip the crutches".
metadata:
  author: Fiction Toolkit
  version: 1.0.0
  category: fiction-editing
---

# Kill-chapter — run every crutch pass over a chapter, one at a time

Language models can insert the same prose crutches into every book regardless of setting:
explain-it similes, hedging filler words, and the mood-adjectives "soft" and "flat". This
skill runs each kill pass **consecutively** over the same file(s), then reports a combined
change list.

The request arguments identify the chapter file(s): a path, a file pattern, or several paths (e.g.
`Chapters/Chapter_15_*.md`). Resolve it to a concrete file list once, up front.

## Why consecutive, not parallel

Each pass already fans out its own parallel workers that **edit files in place**. Running
two passes at once would put two agents on the same file simultaneously and they would
clobber each other's edits. So the passes run strictly one after another: pass N fully
completes and verifies before pass N+1 begins.

## The passes, in order

Run these in sequence. Each has a rule-set in `references/`; read it and hand it to that
pass's workers verbatim:

1. **simile** — `references/simile.md` (structural rewrites first, while the prose is untouched)
2. **crutch** — `references/crutch.md` (sweep the common filler set: just, little, really, almost, slightly, seemed)
3. **soft** — `references/soft.md`
4. **flat** — `references/flat.md`

## Per-pass procedure (identical each time)

1. From the resolved file list, use the host's available text search to identify files containing the pass's target token(s); skip files that
   don't contain it.
2. Triage: search for the token across those files and skim. If rare, fix inline. If spread across
   many files, split the file list into up to 6 roughly equal groups and use one focused
   worker per group in parallel when the host supports workers; otherwise process the groups sequentially.
3. Each worker gets: its exact file list; the pass's rule-set verbatim; the deletion-first
   rule; instruction to read each file in full context (20-line chunks, or a full sentence
   per hit) before judging; the global style rules (no em dashes except to interrupt
   dialogue; complete grounded sentences; no metaphorical math/analytical language). It makes
   the minimal in-place edit, touching ONLY the target token and nothing else on the line,
   and returns a per-file verdict list (each hit tagged KEEP-reason / DELETED / REWORDED,
   with before→after for changes).
4. When all workers for this pass return, run the pass's verification search. Worker edits make
   any in-context file state stale: reread before any follow-up edit. Record the change
   count, then move to the next pass.

## After all four passes

- Print a consolidated summary: per-pass change counts and the notable before→after edits.
- Sanity-check that the passes only touched their target tokens (a worker that "helpfully"
  fixed an adjacent word it wasn't asked to is out of scope — revert those).
- Offer to commit or leave staged. Do not commit unless asked.

## Notes

- The four `references/*.md` are the single source of each pass's keep/fix rules; edit those
  to tune behavior, not this file.
- Book-agnostic by design: no character names, settings, or per-book canon. If a specific
  book needs extra KEEP exceptions, note them at invocation and pass them to the workers.

