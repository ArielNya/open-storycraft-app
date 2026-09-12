# Pass: kill "soft"

"soft" is a crutch word the model overuses. Correct for **physical texture** and a **quiet
speaking voice**; wrong almost everywhere else, especially stretched over an abstract,
temporal, atmospheric, or emotional noun for a writerly effect.

**KEEP (do not touch):**
- Physical texture you could touch: "soft fur," "soft skin," "the soft of her belly," "marble worn soft."
- Speaking softly / a soft voice: "he said softly," "her voice went soft."
- Deliberate theme threads where "soft / softness" is a consciously-built character trait. When unsure if thematic, KEEP and flag.

**FIX (cut first; reword only if the beat needs it):**
- "soft" on an abstract / temporal / atmospheric / emotional noun: "the soft morning," "the soft dark," "a soft ache," "soft warmth / quiet / silence," "a soft sound," "a soft click," "the room going soft at the edges."
- "soft" on a thing with no softness ("a soft heap of copper," "soft bracelets of scar").
- Repetition: same noun called "soft" twice within a few lines, or twice in one sentence.

Default move is **deletion** ("a soft ache" → "an ache"). Reword only when deletion loses
meaning; use the plain word (quiet, faint, low, weak, gentle, dull), never another mood word.

Verify grep (should be empty after the pass):
`grep -rniE "soft (morning|day|night|dark|light|hour|ache|warmth|quiet|silence|sound|click)" <files>`
