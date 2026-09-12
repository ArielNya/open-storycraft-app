---
name: fiction-aiism-editor
description: Fast, focused scan for AI-generated patterns in fiction prose. Detects
  forbidden words, machine-sounding constructions, forbidden personification, vague
  descriptors, and paradiastole patterns. Uses focused workers for 40-line chunk
  scanning when supported. Use when user says "check for AIisms", "AI scan", "find
  AI patterns", "clean AI tells", or invokes fiction-aiism-editor with a chapter reference.
metadata:
  author: Fiction Toolkit
  version: 1.0.0
  category: fiction-editing
---

# AIism Editor

Targeted scan for AI fingerprints using focused workers when supported. Each worker reads 40 lines and checks only for AI-tells. Fast and focused.

## Arguments

The request argument is a chapter reference (e.g., "5", "Chapter-5", "Chapter-5.md").

## Step 1: Find Paths and Measure

1. Parse the request argument to determine the chapter.
2. Search the project files for the chapter (e.g., `**/Chapter-5*`). If not found, ask.
3. Search for the project's review guide: `**/review_guide.md`. Fall back to `**/style_guide.md`.
4. Get the chapter's line count using the host's available file or line-count operation.

## Step 2: Read Criteria

Read [references/aiism-criteria.md](references/aiism-criteria.md).

Store its contents — you will inject them into each worker prompt.

## Step 3: Launch Focused Workers

Calculate 40-line chunks. For EACH chunk, use one isolated worker with the task description `AIism scan lines {start}-{end}`.

Launch all chunks in parallel when the host supports workers; otherwise process the chunks sequentially. Keep the work in the foreground so all findings can be compiled in this run.

**Worker prompt for each chunk:**

> You are scanning fiction prose for AI-generated patterns. Check ONLY lines {start} through {end}.
>
> 1. Read the review/style guide at: {guide_path} — note any project-specific forbidden words/phrases. If not found, use the criteria below only.
> 2. Read the chapter using offset={start - 1} and limit=40: {chapter_path}
> 3. Check every line against these criteria:
>
> {paste the full contents of aiism-criteria.md here}
>
> 4. Return your findings in this EXACT format. Return NOTHING if no issues found:
>
> ```
> **Line {N}:** `exact quoted text`
> Category: [Forbidden Words | Forbidden Phrases | AI Constructions | Forbidden Personification | Vague Descriptors | Paradiastole | Over-Qualified Adverbs]
> Problem: [why it's an AI tell]
> Fix: [specific replacement]
> ```

## Step 4: Compile Report

Collect all worker results. Group by category, sort by line number. Format:

```
## AIism Edit: [Chapter Title]

[1-2 sentence assessment — count, severity]

---

### Forbidden Words
[issues]

### Forbidden Phrases
[issues]

### AI Construction Patterns
[issues]

### Forbidden Personification
[issues]

### Vague Abstract Descriptors
[issues]

### Paradiastole Patterns
[issues]

### Over-Qualified Adverbs
[issues]

---

**Total issues: {count}**
```

Skip empty categories.

## Step 5: Save and Report

1. Look for an `_Edits/` folder near the chapter. If none exists, create one.
2. Save as `Chapter-{N}_AIisms.md`.
3. Tell the user: location, issue count, worst category.
4. Ask if they want to apply fixes.

