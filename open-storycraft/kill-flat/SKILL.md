---
name: kill-flat
description: Hunt the word "flat" across a chapter and cut the figurative mood-adjective uses while keeping the literal flatness ones (deletion-first, parallel focused workers, edits in place). Use when user says "kill-flat", "kill flat", or "strip the flat crutch".
metadata:
  author: Fiction Toolkit
  version: 1.0.0
  category: fiction-editing
---

# Kill "flat" — cut the writerly adjective, keep the literal ones

"flat" is correct and load-bearing for **physical flatness** you could see or touch: a
calm sea, a flat surface, a body laid flat, a flat-bottomed hull, pinned-back animal ears.
It goes wrong when stretched over a **voice, manner, gaze, emotion, or abstract noun** as a
mood adjective ("a flat voice," "flat certainty," "flat dead quiet"). In almost every
figurative case the adjective can simply be **removed** and the sentence loses nothing.

The request arguments = the chapter file(s): a path, a file pattern, or several paths. Resolve it to a
concrete file list first (skip files that don't contain "flat").

## What to keep vs. fix

**KEEP (do not touch) — literal flatness:**
- **Surfaces:** "the sea went flat," "flat grey water," "the wind had gone flat," "a flat
  field."
- **Posture / placement:** "knocked me flat," "stretched flat," "laid out flat," "flat on
  his back," "hand went flat," "board laid flat," "the blade flat against."
- **Body language** (e.g. animal ears pinned flat) where it names a real physical position.
- **Hulls / construction:** "flat bottom," "flat-bottomed," "flat-floored."
- **Objects with a flat plane:** "a flat note," "a flat key," "flat spikes," "the flat of
  the blade."
- **"the flat" as a noun** (a tidal flat, a plain, the flat of a tool). Literal.
- **Idioms:** "flat broke," "three days flat," "flat-out."

**FIX (cut first; reword only if the beat needs it) — figurative "flat":**
- on a **voice / how something was said:** "in a flat voice," "he said it flat," "flat and
  tired."
- on a **gaze / manner:** "looked at me, flat and unhurried," "a flat clever interest,"
  "flat patience."
- on an **abstract / emotional noun:** "flat certainty," "the slow flat truth," "flat
  resolve," "flat dead quiet," "the world: flat, total."
- **Repetition:** "flat" twice within a few lines where one is figurative; cut that one.

Default move is **deletion** ("a flat voice" → "a voice," "said it flat" → "said it").
Reword only when genuinely needed, with the plain word the narrator would use (level, even,
toneless, dull, still), never another mood word.

Be conservative on borderline cases (a "flat ashen grey" is a literal colour-plane; a "flat
hard slap" is an acoustic plane like a flat note). When genuinely ambiguous, KEEP and note.

## Procedure

1. **Resolve the request arguments to files;** use the host's available text search to keep only files containing "flat".
2. **Triage:** search for whole-word "flat" matches and skim. If rare, fix inline. If spread
   across many files, split into up to 6 roughly equal groups and use one focused worker
   per group in parallel when the host supports workers; otherwise process the groups sequentially.
3. **Each worker gets:** its exact file list; the KEEP/FIX and deletion-first rules
   verbatim; instruction to read a FULL sentence of context around every hit; the global
   style rules (no em dashes except to interrupt dialogue; complete grounded sentences; no
   metaphorical math/analytical language). It makes the minimal in-place edit, touching ONLY
   the token "flat" and nothing else on the line, and reports a per-file verdict list: each
   hit tagged (KEPT-literal / KEPT-idiom / KEPT-noun-the-flat / REMOVED / REWORDED) with
   before→after for changes.
4. **Verify when all workers return:** search again for
   `"flat (voice|certainty|truth|resolve|patience|interest|quiet|way)"` → should be empty.
   Worker edits make your in-context file state stale; reread before any follow-up edit.
   Report the consolidated change list and before/after count from the host's available text search.
5. Offer to commit or leave staged.
