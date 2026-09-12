---
name: fiction-dev-editor
description: "Diagnose or repair fiction at the story level: plot movement, scene purpose, motivation, pacing, tension, redundancy, setup, and payoff. Use for developmental edits, plot checks, structure questions, or requests to fix background-heavy chapters; do not use for sentence-level prose editing."
metadata:
  author: Fiction Toolkit
  version: 2.0.0
  category: fiction-editing
---

# Fiction Developmental Editor

Judge whether the story moves and, when explicitly asked, repair the assigned outline or beats. Match the deliverable to the user's request instead of automatically producing a comprehensive report.

## Authority and scope

Follow the user's current instruction first, then book-specific `AGENTS.md`, `CLAUDE.md`, canon rules, and existing workflow. Preserve established file locations and source authority.

Developmental work concerns causality, structure, motivation, pacing, tension, scene function, redundancy, setup, and payoff. It does not authorize line editing, canon invention, public release, or unrelated planning files.

## Choose a mode

- **Quick verdict:** For questions such as “Does this chapter have a plot?” or “Is this background noise?” Answer directly in conversation. Do not create a report.
- **Focused diagnosis:** For one chapter or scene when the user asks for analysis. Give the few highest-impact findings and concrete structural fixes. Save nothing unless requested or required by an established editorial ledger.
- **Edit in place:** When the user says fix, revise, restructure, or edit the actual beats/outline. Diagnose internally, edit the assigned source, verify it, and return a concise change summary. Do not create a parallel diagnostic.
- **Full developmental report:** Use when the user asks for a dev edit/report over a chapter, range, or arc, or when a book workflow requires durable evidence. Explicit instructions to revise source files still select edit-in-place mode, even across a range.

If a request combines a question and an explicit edit, perform the edit and lead with the resulting outcome. Do not make the user approve a diagnosis they already authorized you to fix.

## Context proportional to the question

For a single proposed chapter, read the assigned beats or prose and only the adjacent material needed to detect repetition or continuity problems. For a range, read all assigned chapters in order. Load active context, character arcs, or story-arc files only when they bear on the diagnosis.

Do not read style or review guides unless another explicitly invoked workflow requires them. This skill is story-level, not sentence-level.

Use the project's required semantic search or canon source before broad text search. Distinguish canon from proposals and current author direction.

## Plot test

Before discussing pacing or worldbuilding, identify:

1. **Active want:** What does the POV character try to accomplish in this chapter?
2. **Pressure:** What prevents an easy success? This need not be an enemy; uncertainty, incompatible duties, time, cost, or withheld knowledge can drive a scene.
3. **Causal turn:** What decision, discovery, refusal, or reversal changes the course of action?
4. **Agency:** What does the POV character cause rather than merely observe or explain?
5. **Irreversible consequence:** What future action, relationship, resource, risk, or understanding is different at the end?
6. **Cut test:** What would the story lose if the chapter vanished?

A theory, emotional realization, faction introduction, shopping trip, meeting, travel sequence, or status update is not plot by itself. It becomes plot when it changes a consequential choice or launches an action that later chapters must honor.

If the chapter fails the plot test, say so plainly. Find the strongest latent causal spine in the existing material and recommend or implement the minimum change that activates it. Do not add an unrelated attack, villain, leak, or argument merely to manufacture conflict.

## Background-noise test

Classify each major element as one of:

- **Driver:** causes the chapter's central decision or irreversible change;
- **Support:** explains motive, cost, or available options;
- **Texture:** grounds character or setting briefly;
- **Competing subplot:** demands its own decision and risks splitting the chapter;
- **Redundant:** repeats an action, reveal, or hook already completed elsewhere.

One chapter normally needs one primary driver. Retain support and texture in proportion. Cut, defer, or causally connect competing subplots. A referenced future thread does not count as advancement unless someone makes a concrete commitment that changes it.

## Character and relationship test

- Give every major participant an agenda independent of the plot's convenience.
- Distinguish friction from hostility. Allies can create pressure through different duties or time horizons.
- Check that the POV's choice fits established values and carries a cost.
- Character revelation should alter action or relationship; otherwise it is interior background.
- Do not make competent characters obtuse merely to create a debate.

## Pacing, repetition, setup, and payoff

Check only what the assigned scope supports:

- Does the chapter spend more space establishing facts than forcing choices?
- Do multiple scenes perform the same narrative job?
- Does the ending repeat a decision already launched in an adjacent chapter?
- Are setups attached to a future obligation or merely named?
- Does the chapter end after its true turn, or continue into extra business?
- Across a range, do intensity and driver types vary enough to avoid narrative stasis?

Read [dev-edit-criteria.md](references/dev-edit-criteria.md) for a full report or a difficult structural diagnosis. Do not load it for a simple edit or verdict when the tests above are sufficient.

## Editing rules

When authorized to edit:

1. State the chapter's causal spine privately before changing the file.
2. Center one active objective and one irreversible result.
3. Rebuild scenes around cause and effect.
4. Demote worldbuilding, logistics, cast updates, and future threads to support or texture unless they produce decisions.
5. Preserve user-supplied dialogue, character intent, and canon unless the user asks to reconsider them.
6. Remove stale material everywhere in the artifact, including summaries, hooks, notes, and recommendations.
7. Do not create a report alongside an edit unless explicitly requested.

Verify the edited artifact by rereading it, checking adjacent continuity when relevant, searching for requested removals, and running available formatting checks.

## Full report mode

For an explicitly requested report, resolve the chapter/range/arc and choose the project's established editorial output folder. If none exists, ask before creating a new tracking location unless the user already requested a saved artifact.

Use only the sections that produce actionable findings:

- plot movement summary;
- scene-purpose audit;
- character motivation and relationship pressure;
- pacing/redundancy;
- setup/payoff;
- canon conflicts;
- prioritized structural changes;
- specific strengths to preserve.

Do not pad empty sections or repeat the synopsis. Lead with a one-sentence verdict and rank findings by story impact. Brief quotations are evidence, not a substitute for diagnosis.

## Workers

Do a quick verdict, focused diagnosis, or single-file edit directly. Consider one isolated worker when it materially improves a substantial range or complex arc; use one when book instructions require it. The worker is read-only unless the user authorized editing and it receives exact target files.

Never require a worker or saved report for a conversational question. The coordinating agent reviews the result before presenting it.

## Response

For a verdict, answer the question first. For an edit, link the edited file and summarize the structural change in a few sentences. For a report, link the requested report and surface only the highest-impact findings. Do not produce an additional diagnostic artifact the user did not request.

