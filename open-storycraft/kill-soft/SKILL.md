---
name: kill-soft
description: Hunt the word "soft" across a chapter and cut the figurative crutch uses while keeping the literal texture and speaking-voice ones (deletion-first, parallel focused workers, edits in place). Use when user says "kill-soft", "kill soft", or "strip the soft crutch".
metadata:
  author: Fiction Toolkit
  version: 1.0.0
  category: fiction-editing
---

# Kill "soft" — strip the writerly filler, keep the real ones

"soft" is a crutch word the model reaches for too often. It is correct for **physical
texture** and for a **quiet speaking voice**, and wrong almost everywhere else: stretched
over an abstract, temporal, atmospheric, or emotional noun for a romance/writerly effect.
This skill finds every use in the named files and cuts the bad ones in place,
deletion-first, in the POV character's voice.

The request arguments = the chapter file(s): a path, a file pattern, or several paths. Resolve it to a
concrete file list before doing anything (skip files that don't contain "soft").

## What to keep vs. fix

**KEEP (do not touch):**
- **Physical texture** of a body or object you could actually touch: "soft fur," "soft
  skin," "the soft of her belly," "marble worn soft." Canon and correct.
- **Speaking softly / a soft voice:** "he said softly," "her voice went soft." A quiet
  voice genuinely is soft.
- **Deliberate theme threads** where "soft / softness" names a character's tenderness as a
  named trait the book is consciously building. When in doubt whether a use is thematic,
  KEEP it and flag it rather than stripping.

**FIX (cut first; reword only if the beat needs it) — everything else, especially:**
- "soft" pinned to an **abstract / temporal / atmospheric / emotional** noun: "the soft
  morning," "the soft dark," "a soft ache," "soft warmth / quiet / silence," "soft
  complaints," "the room going soft at the edges," "a soft sound," "a soft click."
- "soft" on a thing that has **no softness** ("a soft heap of copper," "soft bracelets of
  scar").
- **Repetition:** the same noun called "soft" twice within a few lines, or "soft" twice in
  one sentence. Cut the weaker one.

Default move is **deletion** ("a soft ache" → "an ache," "the soft morning" → "the
morning"). Reword only when deletion loses needed meaning; ground it in the plain word the
narrator would use (quiet, faint, low, weak, gentle, dull), never another mood word.

## Procedure

1. **Resolve the request arguments to files.** Then use the host's available text search to keep only files that contain
   the word.
2. **Triage:** search for whole-word "soft" matches and skim. If rare, fix inline. If spread
   across many files, split into up to 6 roughly equal groups and use one focused worker
   per group in parallel when the host supports workers; otherwise process the groups sequentially.
3. **Each worker gets:** its exact file list; the KEEP/FIX and deletion-first rules
   verbatim; instruction to read each file in 20-line chunks so it sees every hit in full
   context; the global style rules (no em dashes except to interrupt dialogue; complete
   grounded sentences; no metaphorical math/analytical language). It makes the minimal
   in-place edit, touching ONLY the token "soft" and nothing else on the line, and reports a
   per-file verdict list: each hit tagged (KEPT-texture / KEPT-voice / KEPT-theme / DELETED /
   REWRITTEN) with before→after for changes.
4. **Verify when all workers return:** search again for `"soft (morning|day|night|dark|light|hour|ache|warmth|quiet|silence|sound|click)"`
   → should be empty. Worker edits make your in-context file state stale; reread before
   any follow-up edit. Report the consolidated change list and before/after count from the host's available text search.
5. Offer to commit or leave staged.
