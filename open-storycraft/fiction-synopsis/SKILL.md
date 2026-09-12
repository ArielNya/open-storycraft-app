---
name: fiction-synopsis
description: "Transform a new-story braindump into Wiki/Story/synopsis.md with premise, central question, reader promise, story answer and payoff, protagonists, mechanism, act structure, and open threads. Use when user says 'create synopsis', 'start my story', 'I have a story idea', or 'turn this into a synopsis'."
metadata:
  author: Fiction Toolkit
  version: "5.2.0"
  category: fiction-writing
  output-format: markdown
  workflow-position: 0.4
  requires: fiction-theme
  next-skill: fiction-outline
---

# Synopsis Generator

**Output:** `Wiki/Story/synopsis.md` in the project's `Wiki/` folder. This Markdown file is the source of truth for the synopsis; the Markdown file remains the source of truth.

## Step 1: Load Context

All inputs come from the project's `Wiki/` folder. The project may live in a subfolder, so search for each file before reading:

- `**/Wiki/Style/genre.md`, for: genre, subgenre, flavor, tone_notes, tropes, trope commitments
- `**/Wiki/Story/theme.md`, for: central_question, tone, motifs
- `**/Wiki/Style/audience.md`, for: protagonist_ages, structural_norms

Each of these is optional. The synopsis works without them but gains depth from their context. Read whichever files exist and carry their content into the worker instructions below.

## Step 2: Gather User Input

Ask the user for these two pieces of information using the host's normal question mechanism:

**Question 1 (Working Title):** "Do you have a working title? Leave blank if not."

**Question 2 (Braindump):** "Give me your braindump: fragments, characters, vibes, scenes, whatever's in your head. The messier the better."

No more questions after this.

## Step 3: Generate the Synopsis

Run this step in an isolated worker when the host supports one, to keep the main conversation context clean. Give the worker the instructions below with the bracketed placeholders filled in. If the host does not support isolated workers, or if you are already the dedicated worker for this task, follow the same instructions directly.

---

## Worker Instructions

Generate `Wiki/Story/synopsis.md` for this project.

**Working directory:** [working directory]
**Wiki root:** [path to the project's Wiki/ folder, e.g. [project]/Wiki]
**Working title:** [title or "none, invent one"]
**Genre context:** [content of Wiki/Style/genre.md if found, else "none"]
**Theme context:** [content of Wiki/Story/theme.md if found, else "none"]
**Audience context:** [content of Wiki/Style/audience.md if found, else "none"]
**Braindump:** [user's braindump]

Cover every field in [references/json-schema.json](references/json-schema.json) as frontmatter or a `##` section of the output file; that schema is the field checklist for this skill.

Before generating, read these craft knowledge files:
- [references/anti-flat-checklist.md](references/anti-flat-checklist.md): what makes a premise feel thin vs. load-bearing; how to ensure each protagonist has a genuine want, need, and irony built in; how to avoid the setup that collapses at act 2
- [references/dynamic-spine-guide.md](references/dynamic-spine-guide.md): how act shapes should escalate, what makes an act break feel earned, how the midpoint must reframe rather than simply advance
- [references/twist-principles.md](references/twist-principles.md): how to plant open threads that function as delayed promises rather than loose ends

Extract: the specific quality checks for each field before finalizing the output.

### Rules

**title**: Use the user's working title if provided. If none, invent one from the braindump's tone and premise.

**premise**: 2-3 sentences. Who the protagonists are, what brings them into proximity, what each wants from the arrangement, and the central irony. This is not a plot summary; it is the core collision stated plainly.

**central_question**: Pull from the theme file if available. If not, derive from the braindump's central tension. State it as a question.

**reader_promise**: 1-2 spoiler-light sentences stating what emotional and genre experience the story commits to deliver, what question or tension will sustain the read, and what kind of satisfaction the ending owes. Derive it from the premise, genre, audience, and central question.

**story_answer**: 1-2 spoiler-containing sentences stating how the protagonist's final choice and the story's outcome answer the central question. This is the editorial answer, not public copy.

**trope_payoff_plan**: Carry every selected trope from the genre file into the body under its exact snake_case identifier. For `genre_contract` and `structural_engine`, specify the story-specific **setup**, **development**, and **payoff**. For `supporting_texture`, specify the recurring expression and where it intensifies or varies. For `deliberate_subversion`, specify the expected pattern established, the turn that overturns it, and the replacement satisfaction delivered. Tie every entry to the premise, central question, or reader promise. If a selected trope does none of those jobs, remove it from the genre profile with the user's approval rather than decorating the synopsis with it.

**protagonists**: Use role-appropriate labels (heroine/hero for romance, protagonist/deuteragonist for others). Each description should sound like the character: the register of the description is the first test of whether the voice is right. Include: name, age, origin, personality, want, unstated need, and what makes them compelling or dangerous. 3-5 sentences each. The two descriptions should have visible contrast.

**mechanism**: The inciting logic. Sub-headings should describe what's actually happening in this story. For a romance: how they meet, how the deal forms, what each is hiding. For a thriller: how the protagonist is recruited, what they're told vs. what's true, what they'd refuse if they knew. Label the sub-headings to match. 2-4 entries.

**act_1 / act_2 / act_3**: Each act needs **shape** (3-5 sentences on what happens) and **ends_with** (1-2 sentences on state of play at the act break). Act 2 also needs **midpoint** (1-2 sentences on the specific event that reframes everything). Act 3 also needs **payoff** (1-2 sentences naming the concrete climax or resolution that fulfills the reader promise and demonstrates the story answer).

**open_threads**: 1-4 unresolved elements that sit at the edge of the story's resolution. Not everything must be tied off. These are the threads that make a standalone feel like it has a world beyond its edges.

**locations_in_play / characters_in_play**: Simple name lists. These seed the fiction-world and fiction-characters skills; the names must match the `Wiki/Locations/` and `Wiki/Characters/` files those skills will create.

### Preserve the braindump
Every specific element the user gave (names, scenes, relationships, tones) must appear in the output. If you invent something, it must be additive, not a replacement.

### Save

Write the synopsis to `Wiki/Story/synopsis.md` under the project root. Create the `Wiki/Story/` folder if it does not exist. Use this shape:

```markdown
---
title: "The Working Title"
central_question: "The question the story asks, stated as a question"
reader_promise: "The spoiler-light experience and satisfaction promised to the reader"
locations_in_play:
  - Location Name
characters_in_play:
  - Character Name
---

# [Title]: Synopsis

## Premise
[2-3 sentences]

## Story Answer
[1-2 spoiler-containing sentences explaining how the ending answers the central question]

## Protagonists
### [role label, e.g. heroine]
[3-5 sentence description]
### [role label, e.g. hero]
[3-5 sentence description]

## Mechanism
### [story-specific label, e.g. how_they_meet]
[entry]
### [story-specific label]
[entry]

## Trope Payoff Plan

### enemies_to_lovers
- **role:** structural_engine
- **setup:** [How genuine opposition is established on the page]
- **development:** [How choices change the relationship and increase the cost of trust]
- **payoff:** [The earned choice or action that completes the trajectory]

## Act 1
- **shape:** [3-5 sentences]
- **ends_with:** [1-2 sentences]

## Act 2
- **shape:** [3-5 sentences]
- **midpoint:** [1-2 sentences]
- **ends_with:** [1-2 sentences]

## Act 3
- **shape:** [3-5 sentences]
- **payoff:** [1-2 sentences naming the concrete fulfillment of the reader promise and story answer]
- **ends_with:** [1-2 sentences]

## Open Threads
- [thread]
```

Frontmatter rules: plain, conservative YAML only. Lists are block lists, one `- item` per line, never inline `[a, b]`. Quote any string containing a colon.

Before saving, verify the promise chain:
- The premise creates the central collision.
- The central question arises from that collision.
- The reader promise states the spoiler-light experience and satisfaction owed.
- The story answer states what the ending proves through the protagonist's choice.
- Act 1 poses the question, the midpoint complicates or reframes it, and the Act 3 payoff fulfills the promise and demonstrates the answer.
- Every selected trope appears in the trope payoff plan under the exact identifier from the genre file.
- Every genre-contract and structural trope has a setup, development, and payoff that supports the premise, pressures the central question, or fulfills the reader promise.

Return: "Synopsis written to Wiki/Story/synopsis.md", the full synopsis content so it can be shown to the user, and a 3-sentence summary of what was built.

---

## Step 4: Present and Confirm

Display the full synopsis and the worker's summary, then ask: "What should I change?"

If changes are needed, use the same isolated-worker mechanism when available with the revision instructions and the path `Wiki/Story/synopsis.md`; otherwise revise directly. It edits the file in place and returns the revised content. Repeat until approved.

When approved: "Next: use `fiction-outline` to build the chapter-by-chapter structure."

## Adding Twists

When the user asks for more twists, read [references/twist-principles.md](references/twist-principles.md) and revise the open threads and act shapes accordingly.

