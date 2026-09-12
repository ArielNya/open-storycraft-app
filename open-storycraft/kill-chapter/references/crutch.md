# Pass: kill generic crutch/filler words

Targets the filler words the model overuses to hedge, soften, or pad: `just`, `little`,
`really`, `quite`, `almost`, `slightly`, `a bit`, `seemed`, `somehow`, `simply`. Run this
pass once per word, or sweep the common set.

**The deletion-first rule:** in most cases the word can simply be **deleted** without
changing the meaning. Prefer deletion over swapping in another adjective or expanding the
sentence. Never replace one filler word with another.

Examples:
- "he just looked at her" → "he looked at her"
- "a little bit afraid" → "afraid"
- "it really mattered" → "it mattered"
- "she almost seemed to smile" → "she smiled" (keep the hedge only if the doubt is the point)

**KEEP:**
- Literal senses carrying real meaning ("a little house" = small; "just one left" = exactly one).
- Genuine in-voice tics in dialogue or close narration, used sparingly and on purpose.

**FIX:** the word hedging or padding, softening a verb, or repeating within a few lines.

Verify: re-grep the word and confirm only literal/in-voice uses remain;
`grep -rio "<word>" <files> | wc -l` for the before/after count.
