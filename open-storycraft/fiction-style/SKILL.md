---
name: fiction-style
description: "Generate Wiki/Style/style_guide.md and Wiki/Style/review_guide.md for a fiction project. Use when user says 'create style guide', 'define the voice', 'establish prose rules', or after running fiction-synopsis."
metadata:
  author: Fiction Toolkit
  version: "7.0.0"
  category: fiction-writing
  output-format: markdown
  workflow-position: 2
  requires: fiction-synopsis
  next-skill: fiction-characters
---

# Style Guide Generator

**Outputs:** two Markdown files in the project's `Wiki/` folder:

- `Wiki/Style/style_guide.md`: the writing style guide (includes the POV/tense block)
- `Wiki/Style/review_guide.md`: the review guide

Create the `Wiki/Style/` folder if it does not exist. If a legacy alternate name exists (e.g. `Wiki/Style/Style Guide.md`), you may read it for context, but always write the canonical snake_case names above.

## Step 1: Identify Project and Gather Inputs

All inputs come from the project's `Wiki/` folder. The project may live in a subfolder of the working directory, so search for files such as `**/Wiki/Story/synopsis.md`. If multiple projects match, ask the user which one.

- `Wiki/Story/synopsis.md`: required. For: title, genre, tone, POV, protagonist names.
- `Wiki/Story/theme.md`: optional. For: central_question, tone, motifs.
- `Wiki/Style/audience.md`: optional. For: content boundaries, tone_variant, vocabulary and complexity calibration.
- `Wiki/Style/genre.md`: optional. For: subgenre, flavor, tropes.
- `Wiki/Characters/*.md`: optional. For: personality and dialogue_style per POV character (feeds voice_checks).
- Any project forbidden-phrases file (e.g. `Wiki/Style/forbidden.md` or a legacy `Forbidden.json`): optional. The forbidden phrases list.

If `Wiki/Story/synopsis.md` does not exist: tell the user to run `fiction-synopsis` first and stop.

### Optional: run with an isolated worker

To keep the main conversation context clean, you may delegate Steps 2 and 3 to one isolated worker when the host supports one. Pass it: the discovered file paths (do not paste file contents into the prompt), the working title, the exact output paths, and this skill's craft constraints. Have it read [references/agent-instructions.md](references/agent-instructions.md) for craft guidance, but the output format is the two Wiki Markdown files defined here, not JSON. The worker should return only confirmation that the files were written, not the guide contents.

## Step 2: Generate Wiki/Style/style_guide.md

Use [references/json-schema.json](references/json-schema.json) as the field checklist: every field it defines must appear in this file as a frontmatter key or a `##` section.

Write one paragraph for the **style** field that captures:
- POV mode and tense (from the synopsis or audience file)
- Sentence style, rhythm, and register
- What is explicit and how to handle it
- What is forbidden in register (clinical, analytical, therapeutic, etc.)
- Sensory priorities and physical grounding

Critical constraints on the style paragraph:
- No character voices. Those belong in character files.
- No scene instructions and nothing that dictates what happens in scenes.
- It is injected verbatim into every chapter prompt, so keep it tight and mechanical.

If the audience file exists, use it for calibration only: tone_variant, sentence complexity, abstraction tolerance, reference pool, and internal monologue allowance shape the register. Do not use it to generate character voices, scene content, or example monologue.

If the theme file exists, read it for motif candidates.

### POV block

Downstream prompt compilers read POV and tense from this file; without it the voice line defaults to first-person present. Derive POV from the synopsis and record it in the frontmatter.

Write the file with this shape:

```markdown
---
title: "Style Guide: [Story Title]"
pov_mode: single|dual
person: first|third
tense: past|present
pov_1: <character>
pov_2: <character, omit line if none>
switching: none|by_chapter
---

## Style

[the one style paragraph]
```

Frontmatter must be plain, conservative YAML: block lists one item per line, never inline lists; quote strings containing colons.

## Step 3: Generate Wiki/Style/review_guide.md

Use [references/review-guide-schema.json](references/review-guide-schema.json) as the field checklist: every field in that schema must appear as a frontmatter key or a `##` section in the Markdown file. Key sources:

- **forbidden_phrases**: pull all entries from the project's forbidden-phrases file if it exists, plus universal craft rules
- **voice_checks**: one entry per POV character, derived from their personality and dialogue_style in `Wiki/Characters/<Name>.md` (if those files exist) or from the synopsis protagonist descriptions
- **thematic_checklist**: derived from the theme file's motifs and central_question if available; otherwise from the synopsis
- All other fields: use the universal rules in the schema as the base; adjust for this project's genre and tone

Write the file with scalar fields (title, genre, pov, tone) in frontmatter and every structured field as a `##` section. Lists of structured objects become labeled sub-bullets, one bold field label per line, so they stay grep-able. For example:

```markdown
---
title: "Review Guide: [Story Title]"
genre: "[genre and subgenre]"
pov: "[POV mode and character names]"
tone: "[tone descriptor]"
---

## Forbidden Phrases
- [phrase]

## Forbidden Words
- **word:** [word]
  - **note:** [why forbidden]

## Crutch Words
- **word:** [word]
  - **note:** [rule for use]

## Punctuation
- **em_dash:** [rule]
- **fragments:** [rule]
- **semicolons:** [rule]
- **ellipsis:** [rule]

## Paradiastole Rule
[rule against not-X-but-Y constructions]

## Triadic List Rule
[rule against three-beat lists]

## Negation Hedging Rule
- **rule:** [rule against defining by what something isn't]
- **patterns_to_flag:**
  - [example sentence -> replacement]
- **fix:** [replace with the concrete positive]
- **exception:** [when deliberate negation is fine]

## Dialogue Tags
- **use:** [allowed tags]
- **never:** [forbidden tags]
- **placement:** [placement rule]

## Voice Personification Rule
[rule against giving abstract entities human behavior]

## Over Description Rule
[rule against ASMR / decorative sensory stacking]

## Telling Verbs
- **word:** [verb]
  - **note:** [how to fix]

## Simile Rule
- **max_per_chapter:** 3
- **character_filters:** [per-character simile lens rules]

## Repetition
- **word_level:** [rule]
- **body_part_rule:** [rule]
- **erotic_scenes:** [rule]

## Sentence Opening Rule
[rule against repeated sentence openings]

## Robotic Words
- **word:** [word]
  - **problem:** [register problem]

## Voice Checks
### [POV character name]
- [check derived from their personality and dialogue_style]

## Thematic Checklist
- [checklist item]

## Quick Checklist
- [checklist item]
```

## Step 4: Present and Confirm

1. Confirm both files were written: `Wiki/Style/style_guide.md` and `Wiki/Style/review_guide.md`.
2. Offer: "Would you like to review the guides and request changes?"
3. If changes are needed, revise the files (or use the same isolated-worker mechanism when available with revision instructions) and re-enforce the critical constraints from Step 2.
4. When approved, suggest: "Next: use `fiction-characters` to generate the cast."

