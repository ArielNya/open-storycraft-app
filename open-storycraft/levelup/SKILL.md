---
name: levelup
description: Filter word scan for fiction prose. Reads a chapter in 40-line sections
  using focused workers when supported, identifies filter words that create reader
  distance, and offers specific replacement text for each hit. Use when user invokes
  levelup with a chapter reference or asks to remove filter words, tighten POV, or
  strengthen immersion.
metadata:
  author: Fiction Toolkit
  version: 1.0.0
  category: fiction-editing
---

# LevelUp — Filter Word Scanner

Scans a chapter for filter words that create distance between the reader and the scene. Uses focused workers when supported (40 lines each) and compiles a prioritized fix list.

## Arguments

The request argument is a chapter reference (e.g., "258", "Chapter-258", "Chapter-258.md", "Chapters/Chapter-258.md").

---

## Step 1: Find the Chapter

1. Parse the request argument. Accept number, filename, or path.
2. Search the project files for the chapter (e.g., `**/Chapter-258*`). If not found, ask the user.
3. Get the line count using the host's available file or line-count operation.

---

## Step 2: Read Criteria

Read [references/filter-word-criteria.md](references/filter-word-criteria.md).

Store its full contents. Inject them into every worker prompt.

---

## Step 3: Launch Focused Workers

Calculate 40-line chunks: 1-40, 41-80, 81-120, etc.

For EACH chunk, use one isolated worker with the task description `Filter scan lines {start}-{end}`.

Launch all chunks in parallel when the host supports workers; otherwise process them sequentially.

**Worker prompt template:**

> You are a filter word editor scanning fiction prose. Your job is to find filter words — words that place the POV character between the reader and the scene, creating unnecessary distance.
>
> Scan ONLY lines {start} through {end}.
>
> 1. Read the chapter file at {chapter_path} using offset={start - 1} and limit=40.
> 2. Apply these criteria:
>
> {paste full contents of filter-word-criteria.md here}
>
> 3. Flag every filter word or phrase. For each hit, return EXACTLY this format:
>
> ```
> **Line {N}:** `exact quoted text containing the filter`
> Filter: [the specific filter word or phrase]
> Category: [Sight | Sound | Touch | Smell | Taste | Cognition | Experience | Ability]
> Problem: [one sentence — what distance it creates]
> Fix: [the rewritten line with the filter removed]
> ```
>
> Return NOTHING for lines with no filter words. Do not flag lines where the filter is justified (perception IS the point, emotional realization beat, necessary attribution).

---

## Step 4: Compile Report

Collect all worker results. Sort all findings by line number. Group into two tiers:

**Tier 1 — Cut Immediately:** Filters where the detail can speak for itself (pure perception pass-throughs with no additional meaning).

**Tier 2 — Review Before Cutting:** Filters that may be doing legitimate work (realization beats, emotional moments, straining-to-perceive situations).

Output format:

```
## LevelUp: [Chapter Filename]

[2-3 sentence summary: total filter count, most common categories, overall immersion assessment]

---

### Tier 1 — Cut Immediately

**Line {N}:** `exact quoted text`
Filter: [word]  |  Category: [type]
Fix: [rewritten line]

[repeat for each Tier 1 finding, sorted by line number]

---

### Tier 2 — Review Before Cutting

**Line {N}:** `exact quoted text`
Filter: [word]  |  Category: [type]
Consideration: [why this might be worth keeping or reframing]
Fix option: [rewritten line if they choose to cut]

[repeat for each Tier 2 finding, sorted by line number]

---

**Total filter hits: {count}**  |  Tier 1: {n}  |  Tier 2: {n}
```

Skip any tier with zero findings.

---

## Step 5: Save and Report

1. Search for an `_Edits/` or `Reviews/` folder near the chapter. If none exists, create an `_Edits/` folder in the chapter's directory.
2. Save report as `[chapter-name]_LevelUp.md` in that folder.
3. Tell the user: save location, total hit count, top filter categories.
4. Ask if they want to apply specific fixes interactively.

