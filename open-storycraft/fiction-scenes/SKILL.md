---
name: fiction-scenes
description: "Create or revise fiction chapter beats and scene plans. Use for chapter beats, scene breakdowns, scene files, or turning an outline into writable action; do not use for drafting prose or developmental diagnosis alone."
metadata:
  author: Fiction Toolkit
  version: "4.0.0"
  category: fiction-writing
  output-format: markdown
  workflow-position: 6
  requires: fiction-outline
  next-skill: fiction-psych
---

# Fiction Scenes

Turn chapter intent into a causal, writable scene plan. Preserve the user's requested scope: revising an existing beat sheet means editing that sheet, not creating a report or a second planning artifact.

## Authority and destination

Use this order of authority:

1. The user's current instructions.
2. The book's `AGENTS.md`, `CLAUDE.md`, and nested instructions.
3. The project's established source of truth and file conventions.
4. This skill's generic defaults.

Do not impose Wiki-only storage, integer numbering, or `Chapter_XX_Scene.md` when the project uses decimal or interlude chapters, or a different beats convention. An explicit output path wins. Otherwise, revise the existing assigned file. For a new file, follow neighboring chapter files; use `Wiki/Outline/Chapter_[NN]_Scene.md` only when no project convention exists.

Do not create a diagnostic, receipt, character profile, location file, or organization article unless the user asks or book instructions require it. Record unresolved entity work inside the requested artifact only when it affects drafting.

## Choose the lightest useful mode

Infer the mode from the request; do not ask when it is clear.

- **Edit in place:** The user asks to adjust, regenerate, fix, or tighten an existing beat sheet. Read the current file, apply the requested changes throughout it, and verify stale material is gone.
- **Compact beats (default for one chapter):** Create a concise chapter spine and 1–3 scene entries. Include only what the writer needs to draft without guessing.
- **Full scene document:** Use when the user asks for detailed scene files, the downstream generator requires structured fields, or several writers/tools will consume the artifact. Read the full-mode references below.
- **Batch scenes:** Use for a chapter range. Preserve cross-chapter continuity and vary drivers and endings.

Do not silently escalate compact beats into a full scene dossier.

## Context

Read only context that can change the plan:

- assigned outline, proposal, existing beats, or chapter directive;
- the POV character and named participants;
- relevant current-canon locations, factions, systems, and active context;
- adjacent prose or beats needed for continuity;
- the project's style/voice source only when the beats include drafting guardrails or the user names it.

Use the project's required retrieval method first. If the book has a preferred index or semantic-search rule, follow it. Search narrowly after semantic retrieval for exact facts. Do not load every character or world file by default.

Separate **established**, **author-directed**, **proposed**, and **unknown** facts. Never convert a possibility into canon merely to fill a field. Avoid inventing names, ages, institutions, dates, or technical mechanics unless the scene cannot work without them; mark any necessary invention as proposed.

## Plot gate: do this before expanding scenes

Write the chapter's causal spine in one or two sentences and test it:

1. **Want:** What does the POV character actively seek now?
2. **Pressure:** What makes that difficult? Friction may be uncertainty, cost, incompatible duties, time, or incomplete knowledge; it need not be hostility.
3. **Choice:** What consequential decision or action does the POV character make?
4. **Change:** What is irreversibly different by the chapter's end?
5. **Necessity:** What future event, relationship, or option would disappear if the chapter were cut?

If the spine cannot answer all five, repair the spine before adding worldbuilding. Information, atmosphere, shopping lists, status reports, and theories are not plot unless they cause a decision that changes later events.

Prefer one primary driver and at most one supporting driver. Consult [driver-types.md](references/driver-types.md) when the driver is unclear or when planning a batch. Use [anti-flat-scenes.md](references/anti-flat-scenes.md) when a chapter risks becoming travel, processing, setup, or exposition without action.

## Build scenes around turns

Use 1–3 scenes according to real changes in location, time, power, or emotional register. One sustained scene is valid when it carries the full turn; do not split it merely to satisfy a count.

Each scene should establish:

- the POV character's immediate goal;
- active pressure or resistance;
- concrete actions and selective physical anchors;
- the turn: the line, discovery, refusal, choice, or act that changes the scene;
- the consequence carried into the next scene;
- the surface response and the truer interior movement;
- a specific ending beat with forward pull.

Worldbuilding belongs inside decisions and consequences. Keep lists selective. A single load-bearing object or transaction usually carries more story than an inventory.

Do not duplicate the same material across summary, emotional beat, writable beats, continuity notes, and recommendations. Add a field only when it gives the eventual drafter new information.

## Compact beats format

Adapt to the project's neighboring files. A useful default is:

```markdown
# Chapter [number]: [title]

## Chapter spine
- **Want:** ...
- **Pressure:** ...
- **Decision:** ...
- **Irreversible change:** ...
- **Why this chapter cannot be cut:** ...

## Scene 1
- **Goal:** ...
- **Action and turn:** [specific beat-by-beat action]
- **Interior movement:** [surface / underneath / where POV lands]
- **Consequence or hook:** ...
```

Add continuity or fact-status notes only where the drafter could otherwise contradict canon. Do not append a generic recommendations section after the plan is already usable.

## Full scene documents

Only in full mode, read:

- [json-schema.json](references/json-schema.json) as the structured field checklist;
- [continuity-tracking.md](references/continuity-tracking.md) for clocks, injuries, relationships, and carried threads;
- [chapter-hooks.md](references/chapter-hooks.md) when the ending lacks forward pull;
- [anti-flat-scenes.md](references/anti-flat-scenes.md) and [driver-types.md](references/driver-types.md) as needed.

Use [scene-template.md](assets/scene-template.md) or [output-template.json](assets/output-template.json) only when their format matches the project or downstream tool. Field-length examples are guidance, not quotas. A summary should be long enough to draft the scene and no longer. Emotional fields must add interior causality rather than paraphrase events.

For each structured scene, populate the fields required by the project's generator. Resolve proper nouns exactly when machine ingestion depends on them. If a valid provisional name cannot resolve, flag it; do not silently replace it or create unrelated canon.

## Workers

Handle a single chapter or focused revision directly unless an isolated worker would clearly improve a large-context or multi-chapter task. Consider one worker for a substantial range or large continuity corpus; use one when book instructions require isolation. Do not delegate merely because the capability exists.

When using a worker, tell it the selected mode, exact target path, permitted side effects, authority order, and whether it may create reports. The coordinating agent must review the actual edited artifact before reporting completion.

## Revision verification

For edits, check every user-requested addition, removal, and reframing across the entire artifact, including continuity notes, hooks, guardrails, fact tables, and recommendations. Search exact banned terms, then read semantically for leftover concepts that use different words. Do not claim a concept is gone because one keyword no longer appears.

Before finishing, verify:

- the plot gate passes;
- background facts support the causal spine rather than compete with it;
- the POV causes the irreversible change;
- allies have independent agendas without forced hostility;
- no earlier chapter's decision or hook is merely repeated;
- proposed facts remain labeled;
- timeline and entity names follow project authority;
- the requested file, and only requested ancillary files, were changed;
- formatting checks pass when available.

Return a concise completion note with the file link and any true drafting blocker. Do not paste the artifact or produce a review report unless asked.

