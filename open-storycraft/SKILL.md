---
name: open-storycraft
description: Orchestrate the Open Storycraft fiction library from spark through genre, synopsis, world, outline, draft, and editorial passes. Use when the user says start a book, run storycraft, open storycraft, continue my project, what skill next, writing pipeline, or wants the full workflow instead of a single craft pass.
metadata:
  author: Fiction Toolkit
  version: "1.0.0"
  category: fiction-writing
  type: workflow
---

# Open Storycraft Orchestrator

Coordinate the unpacked Storycraft skills. Do not reimplement them. Detect the project, pick a mode, load the target skill's SKILL.md, and run that skill as written.

Companion catalog — [references/pipeline.md](references/pipeline.md)
Project file map — [references/project-layout.md](references/project-layout.md)

## Hard rules

- One active skill at a time. Finish it, save its file, then offer the next skill.
- Never invent canon, names, or Wiki files the current skill did not ask for.
- Never silently open a second book. If more than one Wiki/ exists, ask which project.
- fiction-story-sparks is not a project starter unless the user wants a spark. Do not treat a spark as canon.
- Proper names come from name-generator when a skill requires invented names.
- Read the target skill's SKILL.md before acting. Follow its questions, outputs, and confirmation gate.
- Do not dump every skill's references into context. Load only the active skill plus this orchestrator.
- Optional overlays (ao3-writer, ao3-narrative-voice, anti-slop-editor) stay off unless the user asks.

## Step 1: Identify the job

Classify the user request into exactly one mode.

| Mode | When | First move |
|---|---|---|
| spark | random prompt, card spread, writing exercise | fiction-story-sparks only |
| new-project | start a book, new story, blank premise | status board, then fiction-genre |
| resume | continue, what's next, work on the book | status board, then the first missing required file |
| single-skill | named skill or a job that maps to one skill | run that skill |
| draft | write chapter N | require outline + scenes; then fiction-writechapter |
| edit | review / clean / AI-scan / kill-pass a chapter | editorial ladder below |
| world-pack | town, faction, religion, polity, place only | matching world skill |
| meta | new skill, change the library | skill-builder |

If ambiguous, ask one question with at most three options. Do not interview.

## Step 2: Find or create the project

Search for `**/Wiki/` and `**/Wiki/Story/synopsis.md`.

- One Wiki found — that is the project root (the folder that contains `Wiki/`).
- Several Wikis — list working titles from genre.md or synopsis.md and ask.
- None, and mode is new-project — create `<working-title-slug>/Wiki/{Style,Story,Outline,Characters,Locations,Organizations,Systems,Events,Psych}/`. If no title exists yet, use `new-story` and rename after genre.
- None, and mode is not new-project or spark — ask whether to start a new project or point at an existing folder.

Print a compact status board before doing work (except spark):

```
Project: <root or "(none)">
Mode: <mode>
Have: genre / audience / theme / synopsis / style / characters / world / outline / scenes / voice / psych / chapters
Missing required for next skill: <list>
Next: <skill> — <why>
```

Mark each slot yes, partial, or no using [references/project-layout.md](references/project-layout.md).

## Step 3: Route

### spark

Load and run fiction-story-sparks. Do not create Wiki files. After the spread, ask whether to promote it into a new project (new-project using that premise) or stop.

### new-project

Run this spine, stopping after each skill for "What should I change?"

1. fiction-genre → Wiki/Style/genre.md
2. fiction-audience → Wiki/Style/audience.md
3. fiction-theme → Wiki/Story/theme.md
4. fiction-synopsis → Wiki/Story/synopsis.md
5. fiction-style → Wiki/Style/style_guide.md and review_guide.md
6. fiction-characters → Wiki/Characters/
7. fiction-voiceprompt → Wiki/Style/voice_prompt.md
8. Optional world pack, only if the story needs it — fiction-world, then any of fiction-places, fiction-polities, fiction-factions, fiction-religions, fantasy-settlement, town-generator
9. fiction-outline → Wiki/Outline/outline.md
10. fiction-scenes → scene files
11. fiction-psych → Wiki/Psych/
12. fiction-writechapter for the first writable chapter

Skip a step only when its output file already exists and the user does not want a refresh. Do not skip fiction-synopsis by writing an outline from a vibe.

### resume

Walk the spine in order. Run the first skill whose required output is missing or empty. If the spine is complete, ask draft vs edit and which chapter.

### single-skill

Map loose language to one skill using [references/pipeline.md](references/pipeline.md). If that skill lists requires and those files are missing, say so and offer to run the missing prerequisite first.

### draft

Require Wiki/Story/synopsis.md and Wiki/Outline/outline.md. Prefer existing scene and psych files; if they are missing, offer fiction-scenes / fiction-psych before drafting. Then load fiction-writechapter for the named chapter only.

### edit

Identify the chapter file (`**/Chapter*{N}*`, `**/Chapters/**`, or the path the user gave). Run the narrowest pass that matches the ask:

1. Story / structure / "does this chapter belong" → fiction-dev-editor or coldread (voice/POV blind read)
2. Style-guide review → fiction-reviewchapter
3. AI tells / generic phrasing → fiction-aiism-editor
4. Sentence mechanics → fiction-prose-editor
5. Line voice / cliche / POV leaks → fiction-line-editor
6. Combined micro pass → fiction-full-editor
7. Filter words → levelup
8. 1-3 word fragment narration → fragment-hunter
9. Abstract self-explanation → nominalization-hunt
10. Named crutch (soft, flat, just, similes, "the whole of", "person who", "had opinions") → matching kill-* skill, or kill-chapter for the four-pack
11. Detector / statistical flatness → burstiness-check first (diagnostic only), then pangram if they still want a risk writeup

Do not run the whole ladder unprompted. After a pass, report and ask whether to continue down.

### world-pack

Route to the matching skill. Feed existing synopsis / genre / world files as context. Use name-generator and town-generator instead of freehand names.

### meta

Load skill-builder. Do not modify Storycraft skills unless the user names the file to change.

## Step 4: Execute the active skill

1. Read `/home/workdir/.grok/skills/<skill>/SKILL.md`.
2. Follow it exactly — its questions, worker pattern, output path, and confirmation line.
3. If it tells you to read files under that skill's references/ or assets/, do that from the skill folder, not from this one.
4. After a successful write, update the status board with the new file path.
5. Quote the skill's own next-step sentence if it has one. Otherwise use the spine in Step 3.

## Step 5: Stop conditions

Stop and ask the user when any of these hit:

- the active skill's confirmation gate ("What should I change?")
- a missing prerequisite
- multiple projects or chapters could match
- the user asked for a spark, a single pass, or "just this chapter"
- an editorial pass finished — do not auto-chain kill passes into a rewrite

## Quality bar for the coordinator

- Status board is accurate against files on disk, not memory.
- Outputs land in the project's Wiki/ (or the chapter path the write/edit skill names).
- No skill runs "in spirit." If you did not read its SKILL.md this turn, you are not running it.
- Keep replies short. The craft lives in the child skill, not in this orchestrator.
