---
name: fiction-theme
description: "Generate the thematic engine: central question, tone, and motifs. Defines what the story is ABOUT underneath the plot. Use when user says 'define theme', 'what is this about', 'thematic core', or after running fiction-audience."
metadata:
  author: Fiction Toolkit
  version: "3.1.0"
  category: fiction-writing
  output-format: markdown
  workflow-position: 0.3
  requires: fiction-audience
  next-skill: fiction-synopsis
---

# Theme Engine

**Output:** `Wiki/Story/theme.md` in the project's `Wiki/` folder. All inputs are read from `Wiki/` Markdown files; all output is written as Markdown. All work stays in the project's Markdown files.

## Step 1: Locate the Project and Load Context

The project may live in a subfolder of the working directory, so search for the upstream files:

- `**/Wiki/Style/genre.md`: Required. For: genre, subgenre, tone_notes, tropes. If missing, tell the user: "No genre profile found. Run `fiction-genre` first, or tell me your genre."
- `**/Wiki/Style/audience.md`: Optional but recommended. For: age group, thematic expectations. If missing, note it and proceed.
- `**/Wiki/Story/synopsis.md`: Optional (theme usually runs before synopsis). If it exists, read it for: protagonists, premise, act structure.

Treat the folder containing `Wiki/` as the project root. Read the frontmatter and `##` sections of each file that exists.

## Step 2: Load Reference Files

Read these craft knowledge files before generating:
- [references/theme-construction.md](references/theme-construction.md): how to build a central question that holds across a full novel; what makes a theme deployable vs. abstract
- [references/motif-patterns.md](references/motif-patterns.md): motif types, how they function structurally, examples of motifs that do narrative work vs. motifs that just decorate
- [references/promise-guide.md](references/promise-guide.md): how tone and theme interact, how to match the thematic register to the genre's emotional promise

Extract: what makes a central question load-bearing, what makes a motif concrete and deployable, how to balance surface tone vs. underlying darkness.

## Step 3: Gather User Input

Ask the user for one piece of information using the host's normal question mechanism:

**Question:** "What is this story really about, underneath the plot? A sentence, a question, a feeling, a word; whatever you have. Or say 'derive from genre' and I'll build one from your genre and tropes."

This question is intentionally open-ended. The theme skill pulls the subconscious out of the writer.

## Step 4: Generate the Thematic Engine

You may generate directly or, when the host supports one, use an isolated worker with the context gathered above to keep the main conversation clean; either way, apply the rules below. If the user provided a theme seed, build outward from their words and do not override their intent. If "derive from genre", build from the dominant tropes and the natural tension they create.

Cover every field in [references/json-schema.json](references/json-schema.json) as frontmatter or a section (checklist: central_question, tone, motifs with name / type / what_it_is per motif).

**central_question**: A question, not a statement. What the story asks through its events. If the user provided a seed, reframe it as a question. If "derive from genre", build from the dominant tropes and the natural tension they create.

**tone**: One paragraph describing the emotional register: what is warm, witty, or light on the surface; what darker currents run underneath; how the two are balanced. Derive from the tone_notes in `Wiki/Style/genre.md` if available and add specificity for the characters and world.

**motifs**: 3 to 6 motifs. Each must be concrete and deployable: a specific image, object, phrase, recurring situation, or scene shape. "The Open Water" is a motif. "The Nature of Truth" is not. For each motif: name (short and concrete), type (Visual / Verbal / Sensory / Structural / Recurring situation), what_it_is (what it literally is in the story, what it tracks thematically, what it means by the end; 2-3 sentences).

## Step 5: Save and Confirm

Write the result to `Wiki/Story/theme.md` under the project root found in Step 1. Create the `Wiki/Story/` folder if it does not exist. `Wiki/Story/theme.md` is the single source of truth for theme; downstream skills (synopsis, style, voiceprompt, psych) read it from that exact path.

File shape:

```markdown
---
central_question: "The question the story asks, stated as a question. Quote it; it may contain colons."
---

## Tone

One paragraph: emotional register, what is light or warm, what dark currents run underneath, how the two are balanced.

## Motifs

### [Motif Name]
- **type:** Visual | Verbal | Sensory | Structural | Recurring situation
- **what_it_is:** What it literally is in the story, what it tracks thematically, and what it means by the end.

### [Next Motif Name]
...
```

Frontmatter must be plain, conservative YAML: scalar values only here, strings quoted when they contain colons. One `###` section per motif with the bold-label sub-bullets shown, so every field stays grep-able.

Then present the theme to the user in readable form (do not paste the raw file) and ask: "What should I change?"

If changes are needed, edit `Wiki/Story/theme.md` in place and re-present until approved.

When approved: "Next: use `fiction-synopsis` to build the full story structure."

