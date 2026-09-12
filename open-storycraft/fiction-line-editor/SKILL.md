---
name: fiction-line-editor
description: Expert line editing for fiction prose at the sentence level. Finds AIisms,
  cliches, narrative fragments, POV violations, voice issues, show-vs-tell problems,
  repetition, and awkward constructions. Uses focused workers for thorough 40-line
  chunk scanning when supported. Use when user says "line edit", "check my prose",
  "tighten the writing", or invokes fiction-line-editor with a chapter reference.
metadata:
  author: Fiction Toolkit
  version: 5.0.0
  category: fiction-editing
---

# Line Editor

Comprehensive sentence-level editing using focused workers when supported. Each worker reads 40 lines, checks against all line-editing criteria, and returns issues. The coordinating agent compiles the full report.

## Arguments

The request argument is a chapter reference (e.g., "5", "Chapter-5", "Chapter-5.md", "Chapters/Chapter-5.md").

## Step 1: Find Paths and Measure

1. Parse the request argument to determine the chapter. Accept number, filename, or path.
2. Search the project files for the chapter (e.g., `**/Chapter-5*`). If not found, ask.
3. Search for the project's review guide: `**/review_guide.md`. Fall back to `**/style_guide.md`.
4. Get the chapter's line count using the host's available file or line-count operation.

## Step 2: Read Criteria

Read [references/agent-instructions.md](references/agent-instructions.md).

Store its contents — you will inject them into each worker prompt.

## Step 3: Launch Focused Workers

Calculate 40-line chunks: lines 1-40, 41-80, 81-120, etc.

For EACH chunk, use one isolated worker with the task description `Line edit lines {start}-{end}`.

Launch all chunks in parallel when the host supports workers; otherwise process the chunks sequentially. Keep the work in the foreground so all findings can be compiled in this run.

**Worker prompt for each chunk:**

> You are a line editor scanning fiction prose. Check ONLY lines {start} through {end}.
>
> 1. Read the review/style guide at: {guide_path} — note project-specific rules, forbidden words, POV requirements, character voice patterns. If the file is not found, use general fiction craft principles.
> 2. Read the chapter using offset={start - 1} and limit=40: {chapter_path}
> 3. Check every line against ALL of the following criteria:
>
> {paste the full contents of agent-instructions.md here}
>
> 4. Return your findings in this EXACT format. Return NOTHING if no issues found in your chunk:
>
> ```
> **Line {N}:** `exact quoted text`
> Category: [Narrative Fragments | Cliches | AIisms | Repetition | Telling Over Showing | Awkward Constructions | POV Violations | Voice Violations | Paired Adjectives | Style Guide Violations]
> Problem: [why it's a problem]
> Fix: [specific replacement text]
> ```

## Step 4: Compile Report

Collect all worker results. Group issues by category. Sort by line number within each category. Compile into this format:

```
## Line Edit: [Chapter Title]

[1-2 sentence overall assessment — issue count, severity, patterns]

---

### Narrative Fragments
[issues sorted by line number]

### Cliches and Tired Phrasing
[issues]

### AIisms
[issues]

### Repetition
[issues]

### Telling Over Showing
[issues]

### Awkward Constructions
[issues]

### POV Violations
[issues]

### Voice Violations
[issues]

### Paired Adjectives
[issues]

### Style Guide Violations
[issues]

---

**Total issues: {count}**
```

Skip any category with zero findings.

## Step 5: Save and Report

1. Look for an `_Edits/` folder near the chapter (search for `**/*_Edits`). If none exists, create one in the chapter's directory.
2. Save report as `Chapter-{N}_LineEdit.md`.
3. Tell the user: location, issue count, top 3 most common categories.
4. Ask if they want to apply specific fixes.

