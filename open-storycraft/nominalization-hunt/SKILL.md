---
name: nominalization-hunt
description: Find nominalized self-explanation in fiction chapters and propose concrete rewrites that restore observation, bodily response, and action or blunt thought. Use when the user says "nominalization hunt", asks to find abstract self-explanation, or when fiction-masteragent invokes the nominalization pass.
---

# Nominalization Hunt

Read the chapter supplied by the user in chunks of 40 lines. Flag every instance of nominalized self-explanation, then propose a concrete rewrite for each.

## The Pattern

This is NOT about word choice. It is a structural flaw where the prose turns an observation or feeling into a noun phrase, then comments on that noun phrase instead of staying in the scene.

The four sub-patterns to catch:

**1. Observation-being-quality-was-meaning**
`The [observation] being [quality] was [meaning].`
> "Being ready for it was a kind of old he did not have a name for."
> "The voice being level was a bad sign."

**2. Negation nominalized**
`The not-[action] / not-[feeling] did [consequence].`
> "The not-knowing sat in his chest."
> "The not-looking changed nothing."

**3. X was the answer / The answer was**
The scene has already shown the answer. The narration then explains what the answer means.
> "The argument was complete."
> "The other thing was permission."
> "The answer was the same."

**4. AI-style logical chain**
`X was Y, and he did not like Y, and not liking Y changed nothing.`
The prose chains abstractions into a recursive loop instead of landing in the body.
> "The wanting had gone into his chest and sat there, and he had buried it before he knew he was burying it. He had buried the wanting fast, before he knew he was burying it."

## Related tells (flag but lower severity)

- `"the X of it"` used to name a quality rather than render it: "the neatness of it," "the shape of it," "the fact of it"
- `"something in him was [state]"` — abstract resident object doing the feeling
- `"a [noun] he could not [verb]"` — nominalization followed by authorial comment: "a generosity to himself he could not quite stand"
- `"[gerund] was a [noun]"` — action turned into aphorism: "Wanting was a handle people grabbed you by"
- Repeated avoidance phrase used more than once: "He did not look at that part" — one use is technique, three is a tic

## The fix rule

Every rewrite must restore the chain: **Observation → body response → action or blunt thought.**

Never explain what the observation means. Never name the feeling as a noun and then handle it. Put it in hands, breath, jaw, throat, heat, hunger, silence, or movement.

| Instead of | Use |
|---|---|
| "The wanting sat in his chest" | "His hand went into his pocket. He made a fist around nothing." |
| "The argument was complete" | "He had run it every way he knew how." |
| "The neatness of it" | "His jaw went tight." |
| "Something in him was already done" | "His chest had gone quiet. That was how he knew." |

## Output format

For each flag:

1. Line number
2. Exact quoted sentence(s)
3. Which sub-pattern it is
4. Proposed rewrite

End with a severity table: High / Medium / Low for each flag.
