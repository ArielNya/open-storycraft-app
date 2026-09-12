---
name: kill-simile
description: Hunt explain-it similes and meta-comparison crutches ("the way", "a way", "the kind", "a kind", "as if", "like") across a chapter and reword or cut them so the reader infers instead of being told (parallel focused workers, edits in place). Use when user says "kill-simile", "kill similes", or "cut the explain-it comparisons".
metadata:
  author: Fiction Toolkit
  version: 1.0.0
  category: fiction-editing
---

# Kill the explain-it simile — make the reader infer, don't compare them to death

A recurring model weakness is the **comparison crutch**: prose that reaches for a simile or
a "the way / the kind of" construction to explain a feeling or image the concrete detail
already carries. The comparison tells the reader how to feel instead of letting the image do
the work. This skill finds them in the named files and rewords or cuts them in place.

The request arguments = the chapter file(s): a path, a file pattern, or several paths. Resolve it to a
concrete file list first.

## What to fix vs. keep

**FIX (reword or cut):**
- **Explain-it similes** that gloss the emotion the scene already shows: "her face fell,
  like a child who'd been promised something" → trust the falling face, or replace the
  comparison with a concrete physical beat.
- **Meta-comparison scaffolding:** "the way that…", "in the kind of way…", "a kind of…",
  "as if to say…", "like someone who…". These narrate the comparison instead of rendering
  the thing. Cut the scaffold and state the thing.
- **Stacked comparisons:** two or more "like / as" figures in the same beat. Keep at most
  the strongest; cut the rest.
- **Abstract vehicles:** a simile whose comparison is itself abstract ("like a promise,"
  "like something ending") adds no image. Cut or ground it in a concrete vehicle.

**KEEP:**
- A **fresh, concrete simile** that adds an image the prose didn't already have and earns
  its place (sparingly, a couple per chapter at most).
- **In-voice idiomatic comparisons** a character would actually speak in dialogue.
- Literal uses of "like" (preference: "I like it") and "as" (temporal/causal: "as he
  turned," "as you wish").

## Procedure

1. **Resolve The request arguments to files.**
2. **Triage:** search for `like|as if|the way|a way|the kind|a kind` and
   skim. If spread across many files, split into up to 6 roughly equal groups and use one
   focused worker per group in parallel when the host supports workers; otherwise process the groups sequentially.
3. **Each worker gets:** its exact file list; the FIX/KEEP rules verbatim; instruction to
   read each file in 20-line chunks so it judges every hit in full context; the global style
   rules (no em dashes except to interrupt dialogue; complete grounded sentences; no
   metaphorical math/analytical language). It makes the minimal in-place edit (prefer cutting
   the comparison to rewriting the whole sentence) and reports a per-file verdict list: each
   hit tagged (KEPT-fresh / KEPT-voice / KEPT-literal / CUT / REWORDED) with before→after for
   changes.
4. **Verify when all workers return.** Worker edits make your in-context file state stale;
   reread before any follow-up edit. Report the consolidated change list and the before/after
   simile count.
5. Offer to commit or leave staged.

## Notes

- Restraint is the goal: the chapter should end up with a small number of strong, earned
  comparisons, not zero. Don't mechanically strip every "like."
