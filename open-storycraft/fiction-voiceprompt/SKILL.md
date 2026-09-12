---
name: fiction-voiceprompt
description: "Generate or refresh a compact protagonist voice_prompt.md from character, style, theme, audience, genre, and synopsis files. Use when user says voice prompt, protagonist voice, refresh voice, or robotic prose prevention."
metadata:
  author: Fiction Toolkit
  version: "2.0.0"
  category: fiction-writing
  workflow-position: 2.5
  requires: fiction-synopsis, fiction-style, fiction-characters
  next-skill: fiction-writechapter
  output-format: markdown
---

# Voice Prompt Generator

Create a compact `Wiki/Style/voice_prompt.md` that a chapter-writing agent can obey directly. This is not a full style guide. It is the fast-access POV and voice firewall for the main character.

## Step 1: Identify Project Files

Find these paths in the working directory; the project may live in a subfolder:

Required:
- `**/Wiki/Story/synopsis.md`
- `**/Wiki/Style/style_guide.md` or legacy `**/Wiki/Style/Style Guide.md`
- `**/Wiki/Characters/**/*.md`

Optional but strongly preferred:
- `**/Wiki/Story/theme.md`
- `**/Wiki/Style/audience.md`
- `**/Wiki/Style/genre.md`
- `**/Wiki/Style/review_guide.md` or legacy `**/Wiki/Style/Review Guide.md`
- existing `**/Wiki/Style/voice_prompt.md` or root `**/voice_prompt.md`

If multiple projects exist, ask which one. If no protagonist is obvious from the synopsis/style guide, infer from POV or ask the user.

## Step 2: Read Inputs

Read:
1. Synopsis
2. Style guide
3. Protagonist character file first, then any key deuteragonist/antagonist files needed for relationship voice
4. Theme, audience, genre, and review guide if present
5. Existing voice prompt if present, to preserve useful project-specific rules

## Step 3: Build the Voice Prompt

Compose the voice prompt using this structure:

```markdown
# Voice Prompt: [Protagonist Name] Close-[POV] Rewrite

## Core POV Rule
[POV, tense, distance, and whose perception controls the prose.]

## Who [Protagonist] Is
[Age, background, pressure, contradictions, what they want, what they hide.]

## Interior Voice
[How their thoughts sound under calm, pressure, fear, desire, shame, anger, grief.]

## What They Notice
[Concrete perception filters from background, job, class, culture, body, trauma, desire, genre.]

## Emotional Translation Rules
[How to turn emotion into body, action, object, silence, and direct thought.]

## Motif Limits
[Allowed motifs, where they belong, where they become overuse, and what to vary them with.]

## Language Firewall
[Specific words, abstractions, registers, and sentence patterns that would make this protagonist sound robotic, analytical, therapist-like, outline-like, generic, or unlike themselves.]

## Relationship Voice
[How narration changes around key characters without violating POV.]

## Direct Thought Examples
[10-20 short thought lines the protagonist might actually think.]

## Before / After Examples
[5-10 micro-rewrites converting flat, robotic, abstract, or authorial prose into the protagonist's voice.]

## Final Standard
[A short pass/fail test for whether a paragraph belongs to this protagonist.]
```

## Step 4: Hard Requirements

The voice prompt must:
- Be specific enough that a chapter writer can draft from it without reading the full style guide.
- Preserve the protagonist's lived background without turning it into stereotype or exposition.
- Include a negative voice definition: what the prose must never sound like.
- Include motif boundaries so one strong motif does not become the syntax for every emotional beat.
- Include before/after examples with concrete replacements.
- Prefer body, object, action, silence, and blunt thought over abstract explanation.

Avoid:
- Generic advice that could apply to any protagonist.
- Status labels like "bestselling" or "literary" without actionable rules.
- Therapy language unless the protagonist would genuinely think that way.
- Analytical/engineering language unless the protagonist's actual voice supports it.
- Craft-outline terms in the generated prompt examples.

## Step 5: Save and Report

Write the full voice prompt Markdown to `Wiki/Style/voice_prompt.md` in the project (create the `Wiki/Style/` folder if it is missing). If a legacy voice prompt exists under another name or at the project root, still write the canonical `Wiki/Style/voice_prompt.md`; downstream skills read only the canonical path. Overwrite on refresh; the file is safe to regenerate.

Return only:

`Voice Prompt written to Wiki/Style/voice_prompt.md.`
