---
name: fiction-outline
description: "Generate the chapter-by-chapter outline at Wiki/Outline/outline.md with what_must_happen directives. Use when user says 'create outline', 'outline my story', 'structure the plot', or after running fiction-synopsis."
metadata:
  author: Fiction Toolkit
  version: "5.1.0"
  category: fiction-writing
  output-format: markdown
  workflow-position: 5
  requires: fiction-synopsis, fiction-characters
  next-skill: fiction-scenes
---

# Outline Generator

**Output:** `Wiki/Outline/outline.md`, one chapter entry per chapter, each carrying a `what_must_happen` directive.

This skill runs its generation in an isolated worker when the host supports one, to keep the main conversation context clean. The coordinator locates the input files and passes their paths; it does not read them itself. If isolated workers are unavailable, follow the same generation instructions directly.

## Step 1: Locate Inputs

All inputs are Markdown files in the project's `Wiki/` folder. The project may live in a subfolder, so search for them:

- `**/Wiki/Story/synopsis.md`: required. Contains premise, central_question, reader_promise, story_answer, trope payoff plan, protagonists, mechanism, act_1/2/3, open_threads, locations_in_play, characters_in_play.
- `**/Wiki/Style/genre.md`: selected tropes and their genre-level commitments. Required when the project was built through `fiction-genre`; if missing, flag that trope coverage cannot be verified.
- `**/Wiki/Characters/*.md`: one file per character: names, roles, personality, dialogue_style.
- `**/Wiki/Locations/*.md`, `**/Wiki/Organizations/*.md`, `**/Wiki/Systems/*.md`, `**/Wiki/Events/*.md`: location names and organization constraints (prevents/forces/enables).
- `**/Wiki/Story/theme.md`: central_question, tone, motifs.

**Do NOT read these files in the coordinator.** Just collect their paths. The project root is the folder containing `Wiki/`.

If `Wiki/Story/synopsis.md` does not exist or is empty: tell the user to run `fiction-synopsis` first and stop.

## Step 2: Launch the Outline Worker

When the host supports isolated workers, start one with the description `Generate outline for [title]` and give it everything from the **"## Worker Instructions"** section below, replacing the placeholders:
- `{{SYNOPSIS_PATH}}` → the synopsis file path
- `{{GENRE_PATH}}` → the genre file path, or "none"
- `{{CHARACTER_PATHS}}` → the character file paths, or "none"
- `{{WORLD_PATHS}}` → the location/organization/system/event file paths, or "none"
- `{{THEME_PATH}}` → the theme file path, or "none"
- `{{PROJECT_ROOT}}` → the folder containing `Wiki/`

If isolated workers are unavailable, or if you are already the dedicated worker for this task, follow the same instructions directly.

---

## Worker Instructions

Generate the chapter-by-chapter outline for this project and save it to `{{PROJECT_ROOT}}/Wiki/Outline/outline.md`.

**Project root:** `{{PROJECT_ROOT}}`
**Synopsis:** read `{{SYNOPSIS_PATH}}` fully.
**Genre:** read `{{GENRE_PATH}}` fully (skip if "none" and report that trope coverage cannot be verified).
**Characters:** read every file in `{{CHARACTER_PATHS}}` (skip if "none").
**World:** read every file in `{{WORLD_PATHS}}` (skip if "none").
**Theme:** read `{{THEME_PATH}}` (skip if "none").

Cover every field in [references/json-schema.json](references/json-schema.json) as frontmatter or a labeled section in the output file; that schema is the field checklist, nothing more.

Before generating, read these craft knowledge files:
- [references/structure-templates.md](references/structure-templates.md): act distribution patterns, where chapter breaks naturally fall, how to pace revelations across the arc
- [references/anti-flat-rules.md](references/anti-flat-rules.md): what makes a chapter directive feel like instructions vs. a plot summary; how to ensure what_must_happen tells the writer what to show, not what to report
- [references/detailed-prompts.md](references/detailed-prompts.md): how to write chapter directives that are specific enough to generate a scene from, without locking down prose-level decisions
- [references/negative-arc.md](references/negative-arc.md): how to build chapter entries that track character cost, not just plot advancement

Extract: the quality standard for what_must_happen entries before finalizing.

### Contract gate

Before generating chapters, verify that the synopsis contains a premise, central question, reader promise, story answer, and trope payoff plan. Compare the payoff plan with every trope in `genre.md`. If any story-contract field is missing, or any selected trope has no planned treatment, stop and report that the synopsis must be refreshed with `fiction-synopsis`. Do not silently infer the missing contract inside the outline and do not generate an outline that drops it.

### Rules

**title**: From the synopsis.

**premise / central_question / reader_promise / story_answer**: Carry these four fields verbatim from the synopsis into the outline frontmatter. They are the story contract against which every chapter is planned. Do not paraphrase them into a different story.

**note**: Write one sentence explaining how POV is assigned across chapters (e.g., "Chapters are story events, not POV units; POV is assigned per scene as the story requires" or "Chapters alternate: odd = heroine, even = hero"). Add any other global convention the writer needs before opening individual entries.

**chapters**: One entry per chapter. The total chapter count and structure follow from the synopsis act structure. Derive chapter count from the act breakdown: use the act shapes and ends_with entries to determine where chapter breaks fall. Typical adult romance: 18-25 chapters.

**chapter_number**: Sequential integer starting at 1.

**title**: A concrete, evocative title that names the chapter's central event or image without spoiling the turn. Not a beat label like "The Inciting Incident"; a title like "The Shadow on the Sun".

**what_must_happen**: This is the load-bearing field. Write it as a directive to the writer, not a description. Use the imperative: "Write Nico at the card table..." "Open at Le Cercle..." "Show the gap between her internal experience and her external presentation..." Include: who is present, what happens, what the reader must see and feel, specific images or exchanges that are non-negotiable, and how the chapter ends. The concrete action must execute the chapter's premise movement, central-question pressure, promise delivery, and any listed trope obligations; do not leave those jobs only in metadata. Draw from the synopsis act shapes for narrative intent. Draw from the character files' dialogue styles and internal monologue descriptions for voice guidance. Draw from the location files' sensory descriptions for atmosphere. 3-6 sentences. No passive summary.

**premise_movement**: One sentence naming how this chapter changes the unstable situation, the protagonist's options, or the cost of pursuing the premise. A chapter that does none of these is a merge or cut candidate.

**question_pressure**: One sentence naming how this chapter poses, complicates, reframes, narrows, or answers the central question through action and choice.

**promise_delivery**: One sentence naming the part of the reader experience or satisfaction this chapter actually delivers. Do not repeat the global promise without identifying an on-page event.

**trope_obligations**: Zero or more entries using the exact snake_case identifiers from the genre file and synopsis payoff plan. Format each as `trope_id: setup | development | payoff | texture | subversion - concrete chapter action`. A chapter need not carry every trope, but every selected trope must receive the complete treatment its role requires across the outline.

**ends_with**: 1-2 sentences on the exact state of play at close. What each relevant character knows, has, and wants. What is unresolved and carries forward into the next chapter.

### Distribution
Spread the act_1 shape across the first ~35% of chapters, act_2 shape across the middle ~45%, act_3 shape across the final ~20%. Place the act_2 midpoint entry at the exact chapter that carries it.

### Save

Write the outline to `{{PROJECT_ROOT}}/Wiki/Outline/outline.md` (create the `Wiki/Outline/` folder if missing). Format:

- YAML frontmatter carrying the scalar fields. Plain, conservative YAML: block lists only (one `- item` per line), never inline `[a, b]` lists; quote strings containing colons.

```
---
title: "Working Title"
note: "One sentence on POV assignment and any global convention."
chapter_count: 24
premise: "The concrete unstable situation from the synopsis."
central_question: "The dramatic question from the synopsis?"
reader_promise: "The spoiler-light experience and satisfaction owed."
story_answer: "The spoiler-containing answer demonstrated by the ending."
---
```

- Body: one `## Chapter NN: Title` section per chapter (zero-padded two-digit number), with one bold field label per line so every field stays grep-able:

```
## Chapter 01: The Shadow on the Sun

- **chapter_number:** 1
- **title:** The Shadow on the Sun
- **what_must_happen:** [directive prose]
- **premise_movement:** [how the unstable situation or cost changes]
- **question_pressure:** [how action and choice pressure the central question]
- **promise_delivery:** [the concrete reader experience delivered here]
- **trope_obligations:** enemies_to_lovers: setup - [concrete action]; forced_proximity: development - [concrete action]
- **ends_with:** [state of play at close]
```

Before saving, run a coverage pass across the complete outline:

- The opening chapters dramatize the premise and pose the central question rather than merely repeating them in frontmatter.
- Every chapter changes the premise situation or the cost of answering the central question and delivers a concrete part of the reader promise.
- The midpoint materially complicates or reframes the central question.
- The protagonist's climactic choice answers the central question, demonstrates the story answer, and the ending pays the reader promise.
- Every trope in `genre.md` appears under its exact identifier in the outline. Genre-contract and structural tropes receive setup, development, and payoff; supporting texture recurs with variation; deliberate subversions establish the expected pattern and replace it with equal satisfaction.
- Trope obligations are embodied in `what_must_happen`, not merely listed in metadata.

If the premise, promise, central question, story answer, or trope payoff plan conflict, stop and report the mismatch instead of forcing contradictory material into chapters. The synopsis is the place to repair that contract.

Return: "Outline written to Wiki/Outline/outline.md" and a one-line summary of total chapter count and act distribution.

### Relationship turns (add checkpoints where a bond changes)

Relationship states live as chapter-anchored checkpoints; `fiction-characters` seeded the chapter-1 state in each character's `Wiki/Characters/<Name>.md` file. The outline is where you know a bond *turns*: enemies to allies, friends to lovers, a betrayal. For each such turn, record a checkpoint at the chapter where it lands, so downstream skills inject the new state from that chapter on.

Checkpoints live in the character files, not in `outline.md`. For each turn, append a new dated entry to the `## relationship_checkpoints` section of BOTH characters' `Wiki/Characters/<Name>.md` files, under the existing `### <Other Character Name>` heading `fiction-characters` seeded:

```markdown
### <Other Character Name>
- **link_type:** romantic
- **chapter_start:** [chapter the turn lands]
- **chapter_end:** (open)
- **notes:** one-line state from here on
```

When adding the new checkpoint, close the prior open span by setting its `chapter_end` to the chapter before the new `chapter_start`. The new checkpoint supersedes the prior state from `chapter_start` onward; you only state the new state. Only add a checkpoint when the bond actually shifts; a stable relationship keeps its earlier checkpoint until something changes.

---

## Step 3: Present and Confirm

Show the worker's summary and ask: "What should I change?"

If changes are needed, use the same isolated-worker mechanism when available with the revision instructions plus the existing `Wiki/Outline/outline.md` path, and have it rewrite the file.

When approved, confirm the file was saved and suggest: "Next: use `fiction-scenes` to expand each chapter into detailed scene documents at `Wiki/Outline/Chapter_[NN]_Scene.md`."

