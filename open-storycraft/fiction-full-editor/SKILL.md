---
name: fiction-full-editor
description: Comprehensive fiction editing pass combining AIism detection, prose craft
  mechanics, and line editing (voice, cliches, repetition, POV, awkward constructions)
  into a single scan. Uses focused workers for 40-line chunk scanning with all criteria
  when supported. Use when user says "full edit", "edit everything", "complete edit
  pass", or invokes fiction-full-editor with a chapter reference.
metadata:
  author: Fiction Toolkit
  version: 1.0.0
  category: fiction-editing
---

# Full Editor

Runs every micro-level check in a single pass. Each focused worker scans its 40-line chunk against the complete criteria set: AIisms + prose craft + line editing.

## Arguments

The request argument is a chapter reference (e.g., "5", "Chapter-5", "Chapter-5.md").

## Step 1: Find Paths and Measure

1. Parse the request argument to determine the chapter.
2. Search the project files for the chapter (e.g., `**/Chapter-5*`). If not found, ask.
3. Search for the project's review guide (`**/review_guide.md`) AND style guide (`**/style_guide.md`). Use whichever exist.
4. Get the chapter's line count using the host's available file or line-count operation.

## Step 2: Read All Criteria

Read ALL three reference files and combine their contents:
- [AIism criteria](../fiction-aiism-editor/references/aiism-criteria.md)
- [prose criteria](../fiction-prose-editor/references/prose-criteria.md)
- [line-edit criteria](../fiction-line-editor/references/agent-instructions.md)

Combine them into a single criteria block for the worker prompts.

## Step 3: Launch Focused Workers

Calculate 40-line chunks. For EACH chunk, use one isolated worker with the task description `Full edit lines {start}-{end}`.

Launch all chunks in parallel when the host supports workers; otherwise process the chunks sequentially. Keep the work in the foreground so all findings can be compiled in this run.

**Worker prompt for each chunk:**

> You are performing a comprehensive fiction edit. Check ONLY lines {start} through {end}.
>
> 1. Read the review guide at: {review_guide_path} (if it exists)
> 2. Read the style guide at: {style_guide_path} (if it exists)
> 3. Read the chapter using offset={start - 1} and limit=40: {chapter_path}
> 4. Check every line against ALL of the following criteria:
>
> {paste the combined contents of all three criteria files}
>
> 5. Return your findings in this EXACT format. Return NOTHING if no issues found:
>
> ```
> **Line {N}:** `exact quoted text`
> Category: [use the most specific category name from the criteria]
> Problem: [why it's a problem]
> Fix: [specific replacement]
> ```
>
> Be thorough but avoid false positives. If a construction works for character voice, skip it.

## Step 4: Compile Report

Collect all worker results. Deduplicate any issues flagged by overlapping criteria. Group into these top-level sections:

```
## Full Edit: [Chapter Title]

[2-3 sentence assessment — total count, severity, dominant pattern]

---

## AIism Issues
### Forbidden Words/Phrases
### AI Construction Patterns
### Forbidden Personification
### Vague Descriptors / Paradiastole

## Prose Craft Issues
### Narrative Fragments
### Choppy Constructions / S-V-O Chains
### Simile Overuse / Rule of Threes
### Show vs Tell / Info Dumps
### Emotion-Body Formulas

## Line Edit Issues
### Cliches and Tired Phrasing
### Repetition
### Awkward Constructions
### POV Violations
### Voice Violations
### Paired Adjectives

## Style Guide Violations
[any project-specific rule violations]

---

**Total issues: {count}**
**By severity:** {high/medium/low breakdown if clear}
```

Skip empty categories and sections.

## Step 5: Save and Report

1. Look for an `_Edits/` folder near the chapter. If none exists, create one.
2. Save as `Chapter-{N}_FullEdit.md`.
3. Tell the user: location, total issue count, top 3 categories, and whether it's clean enough to publish or needs another pass.
4. Ask if they want to apply fixes.
