---
name: fiction-psych
description: "Generates a deep psychological interior pass for each chapter in a Wiki-mode fiction project. For each chapter, reads the external beats from Wiki/Outline/outline.md and the POV character's full profile from Wiki/Characters/, then writes a chapter-level interior analysis: start state, beat-by-beat interior cost, end state, and show-don't-tell gaps. Output is saved as Wiki/Psych/Chapter_[NN]_Psych.md, human-reviewable and editable before compilation. Use after fiction-outline and before fiction-scenes/compilation."
metadata:
  author: Fiction Toolkit
  version: "2.0.0"
  category: fiction-writing
  output-format: markdown
---

# Fiction Psych: Interior Pass

Generates a psychological analysis for each chapter. The analysis maps what the POV character feels underneath every external beat: what they carry in their body, what they refuse to name, where the prose must render through action rather than report.

Output files are human-readable and editable. The user can revise any chapter's psych pass before the prompt compiler runs. The compiler injects the psych content into each chapter's compiled beat prompt.

## When to Use

- After the Outline step in the fiction-masteragent pipeline (called automatically)
- Manually, when you want to deepen a specific chapter's interior layer before writing
- After revising the outline, to refresh the psych pass for changed chapters

## Step 1: Identify the Project

Find the project's `Wiki/` folder by searching for `**/Wiki/Outline/outline.md`; the project may be a subfolder. If multiple projects exist, ask which one.

## Step 2: Gather Source Files

All inputs come from the project's `Wiki/` folder. Read:

- `Wiki/Outline/outline.md`: all chapter entries with external beats
- `Wiki/Characters/*.md`: full character profiles, one file per character
- `Wiki/Style/style_guide.md`: craft rules and emotional vocabulary: how each emotion maps to physical tells for this POV character (also accept the legacy name `Wiki/Style/Style Guide.md` when reading)
- `Wiki/Story/theme.md`: motifs, central question

Create the `Wiki/Psych/` folder if it doesn't exist.

## Step 3: Run Psych Pass Per Chapter

For each chapter, use one isolated worker when the host supports one. Do not require a particular model. If isolated workers are unavailable, perform the same chapter pass directly, one chapter at a time.

Pass the worker:
- The chapter's full outline entry from `Wiki/Outline/outline.md` (chapter_number, title, pov, summary, emotional_arc, interior_beat, chapter_opening, chapter_closing)
- The FULL profile of the POV character from `Wiki/Characters/<Name>.md`
- The profiles of other characters_present, each from its `Wiki/Characters/` file (trimmed: personality, dialogue_style, relationship_dynamics only)
- The craft rules and any emotional vocabulary from `Wiki/Style/style_guide.md` (how this POV character's body expresses emotion)
- The motifs from `Wiki/Story/theme.md`
- The worker instructions below

Cap at 5 chapters running in sequence. Do not parallelize; each chapter's psych pass benefits from focused attention.

The worker writes each chapter's psych pass to `Wiki/Psych/Chapter_[NN]_Psych.md`, where `[NN]` is the zero-padded two-digit chapter number (e.g. `Wiki/Psych/Chapter_07_Psych.md`). Create parent folders if missing. These files are the canonical output; downstream skills and the prompt compiler read them from this exact path.

Update progress after each chapter.

## Step 4: Report

Tell the user:
- How many psych passes were generated
- That they are stored as `Wiki/Psych/Chapter_[NN]_Psych.md` files
- "Review and edit any chapter's psych pass before running the compiler. The writing agent will receive this analysis alongside the scene summary."

Next: run fiction-voiceprompt, or fiction-scenes if scene files do not exist yet, then fiction-writechapter.

---

## Worker Instructions

You are generating a psychological interior analysis for a single chapter of a novel. This analysis will be read by the writing agent that drafts the chapter. Your job is to map what the POV character feels underneath every external beat: not what they say or do, but what it costs them, what they refuse to name, and where the prose must render psychology through body and action rather than report it from a distance.

Read the criteria file at [references/psych-criteria.md](references/psych-criteria.md) before writing.

### What You Are Writing

A chapter-level psychological map with four sections:

---

**1. START STATE**

At the top of the chapter:
- Surface affect: what the character presents externally
- Underneath: what they are actually carrying, in specific emotional states, not generic ones
- Body: how the body is already holding it before anything happens, in the specific physical tells (from the character's style profile) that the prose should render

---

**2. BEAT-BY-BEAT INTERIOR COST**

For each significant external beat in the chapter summary, map:
- The beat (what happens externally)
- The interior cost (what it costs the POV character, the thing they feel but will not name)
- The body expression (the specific physical tell that carries the feeling, drawn from the character's emotional vocabulary in `Wiki/Style/style_guide.md`)
- What they won't say (the exact thought or word they swallow)

Do not work from generic psychology. Work from this specific character's profile: their personality, backstory, want, need, flaw, secret. The cost of each beat is specific to who this person is.

---

**3. END STATE**

At the chapter's close:
- Surface affect: what they present
- Underneath: what is actually in the room
- Body: the physical tell that should hold the chapter's emotional weight at the final beat

---

**4. SHOW-DON'T-TELL GAPS**

A table of specific moments where the prose must render interior through body/action, not report it. For each:
- The beat
- What a reporting draft does (observes, notes, states)
- What it should do instead (the specific body action, object, silence, or internal movement that carries the feeling)

---

### Output Format

Write the file to `Wiki/Psych/Chapter_[NN]_Psych.md` (zero-padded chapter number) with this structure:

```markdown
# Chapter [NN]: Psychological Interior Pass

**Chapter:** [Title]
**POV:** [Character name, age]
**Setting:** [Setting]
**External arc:** [One-line summary of what happens]
**Emotional arc:** [From outline, start state to end state]

---

## Start State

**Surface affect:** [What the character presents]

**Underneath:**
[Paragraph of specific emotional states. Not "he is tired." "The exhaustion has gone past tired into mean. He has counted the supplies eighty times. Each count comes up short and each shortfall feels like a personal insult." Draw from the character's personality and backstory to make this specific.]

**Body:**
[How the body is carrying it, in specific physical tells from the character's emotional vocabulary. "Jaw set. Breath through the nose. The specific decision not to shift the strap because shifting it would mean admitting it hurts."]

---

## Beat-by-Beat Interior Cost

### [Beat name / external action]
- **Interior cost:** [What it costs them, specific to this character's psychology]
- **Body expression:** [The physical tell, from the character's emotional vocabulary]
- **What they won't say:** [The exact swallowed thought, blunt, in their register]

[Repeat for each significant beat]

---

## End State

**Surface affect:** [What they present at chapter close]

**Underneath:**
[Paragraph: what is actually in the room. Specific. Drawn from what this chapter has done to this specific character.]

**Body:**
[The physical tell that holds the chapter's emotional weight at the final beat. This is the image or action the prose should land on and hold one beat longer than comfortable.]

---

## Show-Don't-Tell Gaps

| Beat | What a reporting draft does | What it should do instead |
|------|----------------------------|--------------------------|
| [Specific moment] | Observes / notes / states neutrally | [Specific body action, object, swallowed word, or silence] |

---

## Bottom Line

[2-3 sentences: What is this chapter really about on the inside? What must the writing agent understand about what this character is living through, not what they do, but what it costs them to do it?]
```

### Critical Rules

- Never write generic psychology ("he feels sad"). Write specific psychology drawn from this character's profile.
- Every interior state must map to a physical tell from the character's emotional vocabulary. If the character expresses fear by going still, fear looks like stillness, not "he felt afraid."
- The "what they won't say" entries should sound like the character's internal register: blunt, in their voice, not literary.
- Show-don't-tell gaps must name SPECIFIC beats from the chapter summary, not general advice.
- The bottom line is for the writing agent, not the reader. It should tell the agent what they must understand to stay inside the POV character's skin for this chapter.

