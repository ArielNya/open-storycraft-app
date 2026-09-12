---
name: kill-person-who
description: Hunt the construction "the [exact/elaborate/specific/particular/steady] [noun] of a [person] who [clause]" across a chapter and rewrite it into concrete action/dialogue/detail instead of a labeled abstraction (deletion-first rewrite, parallel focused workers, edits in place). This is a default go-to move for character interiority/subtext that reads as a mechanical tic on a binge read. Use when user says "kill-person-who", "kill the person-who construction", "kill the 'of a man/woman who' tic", or flags repeated "the [quality] of a [person] who [does X]" phrasing.
metadata:
  author: Fiction Toolkit
  version: 1.0.0
  category: fiction-editing
---

# Kill "the [quality] of a [person] who [clause]" — stop labeling subtext, show it

The tic: instead of dramatizing a character's attitude through action, dialogue, or a
concrete physical detail, the prose names an abstract quality, attaches it to a generic
type-cast of the person ("a man who," "a woman who," "men who"), and closes with a
defining relative clause that does the interpretive work for the reader. It's a shortcut
for subtext, and it's cheap: swap the quality/noun/clause and the sentence still works,
which is exactly why it recurs. Generic examples of the pattern:

- "the patience of a woman who is very good at waiting for things she knows are coming"
- "the exact expression of a man who has priced a thing for someone he expects to accept
  the price"
- "the pleasant uninterested expression of a man who has all morning"
- "the elaborate disinterest of a cat who is choosing not to be present for something that
  does not concern him"
- "the exact expression of a man who is not watching me"
- "the specific unhurry of men who know they've got you"
- "the steady expression of a woman deciding what to do about it"
- "the particular patience of a man who has done this before and is not excited by it"

The request arguments = the chapter file(s): a path, a file pattern, or several paths. Resolve it to a
concrete file list before doing anything.

## The pattern to detect

Template: `the [qualifier]? [abstract-noun or "expression"/"look"/"tone"] of a/the
[person/animal noun] who/deciding/having [clause]`

Qualifiers that flag it: exact, elaborate, specific, particular, steady, careful,
practiced, unhurried, deliberate, and similar precision-adjectives that dress up a vague
quality as if it were sharply observed.

Nouns that flag it: patience, expression, disinterest, unhurry, calm, ease, look, tone,
manner, air — any abstraction standing in for a facial/postural/behavioral beat instead of
naming the beat itself.

This is NOT the same as a simile (no "like"/"as if") and not the same as "the whole of X"
— it's a distinct construction, so don't conflate with kill-simile or kill-the-whole
passes. It can coexist with them in the same chapter; run separately.

Search starting point (catches most instances and will over-match, so triage by eye):
`the [a-z]+ (expression|patience|disinterest|unhurry|calm|ease|look|tone|manner|air) of (a|the) [a-z]+ (who|deciding|having)`.
Also search loosely for "who" clauses following "of a man"/"of a woman"/"of a cat"/"of men"/
"of women" to catch qualifier-less variants ("the calm of a man who...").

## What to keep vs. fix

**KEEP (do not touch):**
- A single, isolated instance in an otherwise clean chapter — this is a craft problem of
  *density and repetition*, not a banned phrase. One well-earned use is fine.
- Genuine dialogue where a character speaks this way as their own verbal tic (rare, would
  need to be established as character voice).

**FIX — everything else, especially any chapter with 2+ instances:**
Default move is **rewrite into a concrete beat**, not a synonym swap (swapping "patience"
for "calm" while keeping the same skeleton doesn't fix anything — the skeleton is the
problem). Ground the abstraction in one of:
- **A specific physical action or micro-gesture** the character actually does (what do
  their hands, eyes, weight, breath do?).
- **A line of dialogue or subtext-loaded beat** that shows the same attitude without
  naming it.
- **A sharper, non-generic comparison** only if it earns its place and isn't another "a
  [type] who [clause]" swap.

Examples of the fix (not prescriptive text, just direction):
- "the patience of a woman who is very good at waiting for things she knows are coming" →
  show her doing something specific with the waiting (recounting coin, not looking at the
  door, letting a silence stretch one beat too long) instead of narrating the trait.
- "the exact expression of a man who has priced a thing for someone he expects to accept
  the price" → give the actual expression (one concrete facial/postural detail) or a line
  that carries the same implication.

Vary the fix per instance — if multiple hits in one chapter get similar treatment
(e.g. all become "he did X with his hands"), that's a new tic. Push for structural
variety: sometimes dialogue, sometimes gesture, sometimes cut the beat entirely if the
surrounding lines already carry it.

## Procedure

1. **Resolve the request arguments to files.** Search each with the pattern above; keep only files
   with hits.
2. **Triage:** if hits are sparse (1-3 total), fix inline. If spread across multiple
   files/scenes, split into up to 6 roughly equal groups and use one focused
   worker per group in parallel when the host supports workers; otherwise process the groups sequentially.
3. **Each worker gets:** its exact file list; the pattern description, KEEP/FIX rules,
   and confirmed examples verbatim; instruction to read each file in full scene-length
   chunks (not narrow windows — needs surrounding context to invent a concrete beat); the
   global style rules (no em dashes except to interrupt dialogue; complete grounded
   sentences; no metaphorical math/analytical language; POV-appropriate voice). It makes
   the minimal in-place edit per hit and reports a per-file verdict list: each hit tagged
   (KEPT-isolated / REWRITTEN) with before→after.
4. **Verify when all workers return:** search again for the pattern across the same files — should
   be at most one surviving isolated instance per file. Worker edits make your in-context
   file state stale; reread before any follow-up edit. Report the consolidated
   before/after list and total hit count.
5. Offer to commit or leave staged.
