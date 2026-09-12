---
name: fiction-audience
description: "Define target reader age group, content boundaries, and structural norms. Use when user says 'define audience', 'who is this for', 'age group', 'target reader', 'content rating', or after running fiction-genre."
metadata:
  author: Fiction Toolkit
  version: "4.0.0"
  category: fiction-writing
  output-format: markdown
  workflow-position: 0.2
  requires: fiction-genre
  next-skill: fiction-theme
---

# Audience Definer

**Output:** `Wiki/Style/audience.md` at the project root. All inputs come from `Wiki/` Markdown files; all output is written back into `Wiki/`.

## Step 1: Locate the Project and Load Context

The project may live in a subfolder, so search for:

- `**/Wiki/Style/genre.md` for: genre, subgenre, tone_notes, tropes, and any content heat levels the genre file records.
- `**/Wiki/Story/synopsis.md` for: protagonist names and ages, POV structure.

If no `Wiki/Style/genre.md` exists, tell the user: "No genre profile found. Run `fiction-genre` first, or tell me the genre and I will work from that." A synopsis without a genre file is still useful; read it for genre context.

## Step 2: Gather User Input

Ask the user for one piece of information using the host's normal question mechanism:

**Question, Target Reader:**
- Children (ages 8-12)
- Middle Grade (ages 10-14)
- Young Adult (ages 14-18)
- New Adult (ages 18-25)
- Adult (general)
- Not sure, recommend based on genre

## Step 3: Generate with an Isolated Worker When Available

To keep the main conversation context clean, use an isolated worker when the host supports one. Give it: the user's age-group choice, the resolved paths to `Wiki/Style/genre.md` and `Wiki/Story/synopsis.md`, the project root, and the full instructions in Steps 4 through 6 below. The worker writes the file and returns the full profile text for review.

If the host does not support isolated workers, or if you are already running as the dedicated worker for this task, do Steps 4 through 6 directly instead of delegating again.

## Step 4: Load Audience Reference

Read the bracket-specific reference file:

- Children → [references/audience-children.md](references/audience-children.md)
- Middle Grade → [references/audience-middlegrade.md](references/audience-middlegrade.md)
- Young Adult → [references/audience-ya.md](references/audience-ya.md)
- New Adult → [references/audience-newadult.md](references/audience-newadult.md)
- Adult → [references/audience-adult.md](references/audience-adult.md)
- Not sure → [references/audience-crossref-matrix.md](references/audience-crossref-matrix.md) to match genre to bracket, then read that bracket's file

Always also read [references/audience-crossref-matrix.md](references/audience-crossref-matrix.md) to check for genre-age compatibility conflicts. Flag conflicts explicitly. For example:

- Grimdark fantasy (darkness 5) aimed at Middle Grade (darkness max 2) is a CONFLICT.
- Dark romance (heat 5) aimed at YA (heat max 3) is a CONFLICT.
- Cozy mystery (all levels 1-2) aimed at Adult is COMPATIBLE, but note the niche.

Extract from the bracket file: content boundary norms, protagonist age conventions, structural norms (word count, chapter length), thematic requirements. Use these to inform, not override, the story's specific context.

## Step 5: Generate the Audience Profile

Cover every field in [references/json-schema.json](references/json-schema.json) as a frontmatter key or a `##` section. That schema is the field checklist only; the deliverable is Markdown.

Field derivation:

- **age_group**: Use the user's choice exactly. If "recommend", derive from genre/subgenre: explicit romance and dark adult content always maps to Adult.
- **typical_reader_age**: Age range string matching the bracket.
- **protagonist_ages**: Use `heroine` and `hero` as keys. Pull ages from `Wiki/Story/synopsis.md` if available; otherwise derive from genre conventions for the age bracket. If the story has a single protagonist, use the appropriate key only.
- **tone_variant**: 2 to 4 word descriptor. Derive from the genre file's tone_notes if available.
- **content_boundaries**: Four categories: `violence`, `sexual_content`, `language`, `darkness`. Each gets a `level` (integer 1 to 5) and a `note` (20 words max describing how it functions in this specific story, not generic norms). Derive from the genre file's tropes and the story's stated tone.
- **structural_norms**: `chapter_length_words` and `total_word_count` as [min, max] integer pairs. `pov_structure` from the synopsis POV field if available. `series_potential` from story context.

Write `Wiki/Style/audience.md` in this shape (create the `Wiki/Style/` folder if missing):

```markdown
---
age_group: Young Adult
typical_reader_age: "14-18"
tone_variant: "Witty / Dark"
---

# Audience Profile

## Protagonist Ages
- **heroine:** 17
- **hero:** 18

## Content Boundaries
- **violence:**
  - **level:** 3
  - **note:** [how violence functions in this specific story]
- **sexual_content:**
  - **level:** 2
  - **note:** [...]
- **language:**
  - **level:** 2
  - **note:** [...]
- **darkness:**
  - **level:** 3
  - **note:** [...]

## Structural Norms
- **chapter_length_words:** [2000, 3500]
- **total_word_count:** [70000, 90000]
- **pov_structure:** [POV mode description]
- **series_potential:** [Standalone | Standalone with sequel hook | Series]

## Genre-Age Compatibility
- [Compatibility assessment against the crossref file]
- [Any adjustments needed]

## Compatibility Warnings
- [Only if conflicts were detected: the specific conflict and a recommendation for adjusting either the genre scope or the age target]
```

Frontmatter must be conservative YAML: block lists one item per line, never inline `[a, b]` lists in frontmatter, quote strings containing colons. The `[min, max]` pairs above live in the body bullets, not the frontmatter.

**Hard rules:**
- Content boundaries are hard limits, not suggestions. If the genre exceeds the bracket's limits, the profile must say so in Compatibility Warnings.
- Protagonist age should run 1 to 2 years older than the target reader's upper range for Children, Middle Grade, and YA.
- If the audience work surfaces a forbidden/banned-phrase list, write it to its own file at `Wiki/Style/forbidden.md` (one `- phrase` bullet per line, with an optional note after a colon). That file is the canonical forbidden-phrases list; `fiction-style` reads it when building `review_guide.md`.

## Step 6: Present and Confirm

Show the full profile to the user and ask: "What should I change?" Apply revisions by editing `Wiki/Style/audience.md` in place. For large rewrites, use the same isolated-worker mechanism when available. Continue until approved.

When approved: "Next: use `fiction-theme` to define your story's thematic engine."

