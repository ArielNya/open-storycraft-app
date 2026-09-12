---
name: fiction-prose-editor
description: Prose craft mechanics check for fiction. Scans for narrative fragments,
  choppy constructions, S-V-O chains, simile overuse, rule-of-threes, paired adjectives,
  gerund chains, escalation patterns, show-vs-tell, info dumps, and emotion-body
  formulas. Uses focused workers for 40-line chunk scanning when supported. Use when
  user says "check prose", "prose edit", "tighten craft", "check mechanics", or invokes
  fiction-prose-editor with a chapter reference.
metadata:
  author: Fiction Toolkit
  version: 1.0.0
  category: fiction-editing
---

# Prose Editor

Prose craft mechanics scan using focused workers when supported. Each worker reads 40 lines and checks for structural prose issues — not voice, not AI tells, just craft.

## Arguments

The request argument is a chapter reference (e.g., "5", "Chapter-5", "Chapter-5.md").

## Step 1: Find Paths and Measure

1. Parse the request argument to determine the chapter.
2. Search the project files for the chapter (e.g., `**/Chapter-5*`). If not found, ask.
3. Search for the project's style guide: `**/style_guide.md`. Fall back to `**/review_guide.md`.
4. Get the chapter's line count using the host's available file or line-count operation.

## Step 2: Read Criteria

Read [references/prose-criteria.md](references/prose-criteria.md).

Store its contents — you will inject them into each worker prompt.

## Step 3: Launch Focused Workers

Calculate 40-line chunks. For EACH chunk, use one isolated worker with the task description `Prose scan lines {start}-{end}`.

Launch all chunks in parallel when the host supports workers; otherwise process the chunks sequentially. Keep the work in the foreground so all findings can be compiled in this run.

**Worker prompt for each chunk:**

> You are checking fiction prose craft mechanics. Check ONLY lines {start} through {end}.
>
> 1. Read the style guide at: {guide_path} — note POV rules, sentence structure preferences. If not found, use general fiction craft principles.
> 2. Read the chapter using offset={start - 1} and limit=40: {chapter_path}
> 3. Check every line against these criteria:
>
> {paste the full contents of prose-criteria.md here}
>
> 4. Return your findings in this EXACT format. Return NOTHING if no issues found:
>
> ```
> **Line {N}:** `exact quoted text`
> Category: [Narrative Fragments | Choppy Constructions | S-V-O Chains | Simile Overuse | Rule of Threes | Paired Adjectives | Gerund Chains | Escalation Patterns | Show vs Tell | Info Dumps | Emotion-Body Formulas]
> Problem: [what's wrong mechanically]
> Fix: [specific replacement]
> ```

## Step 4: Compile Report

Collect all worker results. Group by category, sort by line number. Format:

```
## Prose Edit: [Chapter Title]

[1-2 sentence assessment — count, severity, patterns]

---

### Narrative Fragments
[issues]

### Choppy Constructions
[issues]

### S-V-O Chains
[issues]

### Simile Overuse / Rule of Threes
[issues]

### Paired Adjectives for Mood
[issues]

### Gerund Chains / Escalation Patterns
[issues]

### Show vs Tell
[issues]

### Info Dumps
[issues]

### Emotion-Body Formulas
[issues]

---

**Total issues: {count}**
```

Skip empty categories.

## Step 5: Save and Report

1. Look for an `_Edits/` folder near the chapter. If none exists, create one.
2. Save as `Chapter-{N}_Prose.md`.
3. Tell the user: location, issue count, worst category.
4. Ask if they want to apply fixes.

