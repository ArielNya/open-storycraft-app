---
name: kill-opinion-personification
description: Hunt the "opinion" personification tic (pain, wounds, and inanimate objects "having opinions" or "saying their piece") across a chapter range and cut or vary the repeats while keeping one or two founding uses (deletion-first, parallel focused workers, edits in place). Use when user says "kill-opinion-personification", "kill the opinion tic", "kill 'had opinions'", or flags repeated "X had an opinion about Y" / "sent up its opinion" phrasing.
metadata:
  author: Fiction Toolkit
  version: 1.0.0
  category: fiction-editing
---

# Kill the "opinion" personification tic — keep the first, vary or cut the rest

The tic: pain, wounds, body parts, and inanimate objects (pilings, walls, ribs) are given
"opinions" as a stand-in for sensory/physical description — a cute deflection move that
substitutes a labeled abstraction ("it had an opinion about X") for the actual physical
detail. It's not wrong on its own; it's a legitimate voice choice in small doses. It
becomes a tic when it recurs enough that the reader starts to predict it. Generic examples across several scenes:

- Early scene: pilings "have opinions about their own weight"
- Early scene: the wall "was now mostly opinion"
- Early scene: "everything in it had an opinion"
- Later scene: "The pain sent up its opinion, which I noted and did not act on"
- Later scene: "A wound that had opinions"
- Later scene: "my ribs said their piece"
- Final scene: "it had opinions about the walk up the hill and I was choosing not to have
  opinions back" (two uses in one sentence)

That's 7-8 uses across four chapters — this is a voice choice, not a hard rule, so the fix
is density management, not eradication.

The request arguments = the chapter file(s) or range: a path, a file pattern, or several paths.

## The pattern to detect

Any construction where a non-sentient thing (pain, a wound, ribs, a wall, pilings,
furniture, weather) is said to "have an opinion," "voice/send up its opinion," "say its
piece," "have something to say about," or an equivalent personification-via-opinion
phrasing standing in for physical sensation or description.

Search starting point (this will over-match legitimate character opinions, so triage by eye and
only count hits where the subject is non-sentient): `opinion|said (its|their) piece|had something to say`.

## What to keep vs. fix

**KEEP (do not touch):**
- The **founding use** — the first instance chronologically in the chapter range (the earliest pilings/wall/"everything in it had an opinion" cluster), since it establishes the
  voice move. Note this cluster has 3 uses in one scene already; keep at most the
  strongest one or two of that founding cluster, don't necessarily keep all three.
- At most one additional use later in the range, chosen where it's freshest/funniest and
  farthest in distance from the founding use, to keep the motif alive as a rhythm rather
  than a crutch.

**FIX (deletion-first, then vary) — every other instance:**
Default move is to **replace with a concrete physical/sensory detail** that does not use
"opinion" or a synonym stand-in ("said its piece," "had something to say"). Ground it in
what the body or object actually does: what does the pain do (spike, drag, pull), what
does the wound do (pull at the stitches, throb on the inhale), what does the wall/piling
actually do (groan, list, shed a plank). Only fall back to a different personification
device if a straight physical description genuinely loses the comic/voice effect the line
needs, and even then don't reuse "opinion" or a close synonym.

Examples of direction (not prescriptive text):
- "The pain sent up its opinion, which I noted and did not act on" → name the actual
  sensation and the character's actual response to it, cut the personification frame
  entirely if the surrounding prose already carries the voice.
- "it had opinions about the walk up the hill and I was choosing not to have opinions
  back" → this is two uses in one sentence; cut to one concrete beat about the climb
  (breath, legs, ribs) with no "opinion" language at all.

## Procedure

1. **Resolve the request arguments to files**, in chapter order. Search each for the pattern above;
   confirm each hit by eye (subject must be non-sentient/a body part/pain, not an actual
   character's opinion).
2. **Establish the chronological order of hits** across the range first — this determines
   which one or two survive.
3. **Triage:** if the total confirmed hit count is small, fix inline. If spread across
   multiple files, use one focused worker per file (or small file group)
   in parallel when the host supports workers; otherwise process them sequentially. Pass each worker the full ordered hit list
   across the whole range so it knows which instances are pre-designated KEEP vs FIX —
   don't let per-file workers independently decide what's "the founding use."
4. **Each worker gets:** its exact file(s); the full ordered hit list with KEEP/FIX
   assignments already decided; the direction rules above; the global style rules (no em
   dashes except to interrupt dialogue; complete grounded sentences; no metaphorical
   math/analytical language; POV-appropriate voice). It makes the minimal in-place edit per
   FIX hit, leaves KEEP hits untouched, and reports a per-file verdict list (KEPT-founding
   / KEPT-echo / REWRITTEN) with before→after for changes.
5. **Verify when all workers return:** search again for the pattern across the full range — should
   show at most two surviving "opinion"-personification hits total, correctly the ones
   pre-designated KEEP. Worker edits make your in-context file state stale; reread
   before any follow-up edit. Report the consolidated before/after list.
6. Offer to commit or leave staged.
