---
name: fiction-genre
description: "Select genre, subgenre, flavor, tone, and tropes for a new fiction project. Use when user says 'pick a genre', 'what genre', 'start a new book', 'genre selection', or before running fiction-synopsis."
metadata:
  author: Fiction Toolkit
  version: "3.2.0"
  category: fiction-writing
  output-format: markdown
  workflow-position: 0.1
  requires: null
  next-skill: fiction-audience
---

# Genre Selector

**Output:** `Wiki/Style/genre.md` in the project's `Wiki/` folder. The project may live in a subfolder; search for `**/Wiki/Style/genre.md` (or `**/Wiki/`) before assuming the project root. If no `Wiki/Style/` folder exists yet, create it.

## Step 1: Gather User Input

Ask the user for these three pieces of information using the host's normal question mechanism:

**Question 1: Genre:** "What genre? Fantasy, Sci-Fi, Romance, Thriller, Mystery, Horror, Comedy, Adventure, Literary Fiction, or 'Surprise me'."

**Question 2: Subgenre & Tropes:** "Any subgenre or tropes you want? (e.g. 'enemies to lovers', 'dark romance', 'slice of life') Leave blank for recommendations."

**Question 3: Working Title:** "Working title? Leave blank if none."

No more questions after this. If no title was provided, invent a short, evocative working title and flag it for review when presenting the result.

## Step 2: Load Genre Reference

Read the genre index file for the chosen genre:

- Fantasy → [references/genre-fantasy.md](references/genre-fantasy.md)
- Sci-Fi → [references/genre-scifi.md](references/genre-scifi.md)
- Romance → [references/genre-romance.md](references/genre-romance.md)
- Horror → [references/genre-horror.md](references/genre-horror.md)
- Comedy → [references/genre-comedy.md](references/genre-comedy.md)
- Adventure → [references/genre-adventure.md](references/genre-adventure.md)
- Thriller → [references/genre-thriller.md](references/genre-thriller.md)
- Mystery → [references/genre-mystery.md](references/genre-mystery.md)
- Literary Fiction → [references/genre-literary.md](references/genre-literary.md)

If "Surprise me": pick a genre yourself, then read that genre's index and select a subgenre from it.

From the index, identify the best-fit subgenre. If the user gave trope preferences, match them against the index's subgenres and trope lists; the user's preferences are canon and your selections are suggestions. Then read the subgenre detail file from this skill's `references/` folder (for example, [references/romance-dark.md](references/romance-dark.md)). The index lists the exact filename for each subgenre. If the detail file does not exist, use the summary in the index file instead. Extract: mandatory tropes, optional tropes, what each trope makes the reader expect, pacing conventions, tone, and the subgenre's satisfaction source.

## Step 3: Generate the Genre Content

Cover every field in [references/json-schema.json](references/json-schema.json) as a frontmatter key or a `##` section of the output file. Build the content directly; no separate worker is needed.

**genre**: Primary genre label. Use the user's choice exactly; if "Surprise me", pick one.

**subgenre**: Specific subgenre from the index. Match user preferences if given. Be specific: not just "Fantasy Romance" but "Slow-Burn Fantasy Romance with explicit erotica elements" if that fits.

**flavor**: The cultural or tonal flavor that makes this story distinctive. Use underscores (e.g. `Mediterranean_High_Life`, `Gothic_Southern_Gothic`, `Regency_London`). Derive from the title or user input.

**tone_notes**: One paragraph informed by the subgenre's tone and reader expectations. What the surface experience feels like, what is warm or witty, what darker currents run underneath, what texture of the world is load-bearing. Specific to this story, not a genre description.

**tropes**: List of snake_case strings. Start with the subgenre's mandatory tropes. Add the user's stated preferences. Add only optional tropes that materially strengthen the intended experience; do not copy the reference's entire catalog. Every selected trope is a commitment that downstream planning must either fulfill or deliberately subvert.

**trope_commitments**: In the body, give every selected trope its own heading and record:

- **role:** `genre_contract`, `structural_engine`, `supporting_texture`, or `deliberate_subversion`;
- **reader_expectation:** the recognizable experience, progression, scene type, or payoff the trope leads a reader to expect;
- **planning_obligation:** what the synopsis and outline must establish, develop, and pay off, or what a deliberate subversion must replace with equal satisfaction.

Mandatory subgenre tropes are normally `genre_contract` or `structural_engine`. Use `supporting_texture` only when recurrence matters but a discrete payoff does not. Use `deliberate_subversion` only when the user requested or approved the subversion. Keep these obligations genre-level; do not invent the story's plot before the synopsis exists.

## Step 4: Save and Confirm

Write the result to `Wiki/Style/genre.md` (create the `Wiki/Style/` folder if missing). The scalar fields go in YAML frontmatter; `tone_notes` gets its own `##` section in the body. Frontmatter must be plain, conservative YAML: block lists only (one `- item` per line, never inline `[a, b]`), and quote any string containing a colon.

File shape:

```markdown
---
working_title: "..."
genre: Fantasy
subgenre: "Slow-Burn Fantasy Romance"
flavor: Mediterranean_High_Life
tropes:
  - enemies_to_lovers
  - forced_proximity
  - hidden_royalty
---

# Genre

## tone_notes

One paragraph of tone notes, specific to this story.

## Trope Commitments

### enemies_to_lovers
- **role:** structural_engine
- **reader_expectation:** Sustained antagonism changes through earned vulnerability into chosen trust and intimacy.
- **planning_obligation:** Establish a real basis for opposition, develop costly changes in understanding, and pay it off through an earned choice rather than sudden attraction.
```

Show the user the output and ask: "What should I change?" Flag any choices you made that the user did not specify (invented title, surprise-me picks).

If changes are needed, revise and rewrite `Wiki/Style/genre.md` (overwriting is safe; this file is the single home of the genre profile).

When approved: "Next: use `fiction-audience` to define your target reader."

