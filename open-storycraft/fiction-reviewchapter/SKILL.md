---
name: fiction-reviewchapter
description: "Review chapters against style guide, characters, and world for actionable edits. Use when user says 'review chapter', 'edit chapter', 'check my chapter', or after writing."
metadata:
  author: Fiction Toolkit
  version: "2.1.0"
  category: fiction-writing
  workflow-position: 8
  requires: fiction-writechapter
  next-skill: null
---

# Review Chapter — Orchestrator

This skill runs in an isolated worker when the host supports one, to keep the main conversation context clean. If isolated workers are unavailable, follow the same review instructions directly.

## Step 1: Identify Project and Chapter

Use the project or chapter named by the user. Otherwise identify the selected book from the current context and `**/Wiki/Story/active-context.md`. Read that book's `AGENTS.md`, `CLAUDE.md`, and applicable nested instructions before reviewing.

Find the requested chapter in the book's authoritative chapter source. If the user did not specify one, use the most recent chapter recorded by active context. Prefer the existing `<book-root>/Chapters/` convention, but obey book-specific authority such as `Published/`, volume folders, decimal chapters, or another established source. Never assume `*_Chapters/`.

Also locate these project files:
- **Review guide** (editing rules, forbidden words, AI-tells): `**/Wiki/Style/review_guide.md` or legacy `**/Wiki/Style/Review Guide.md`
- **Style guide** (writing voice, POV): `**/Wiki/Style/style_guide.md` or legacy `**/Wiki/Style/Style Guide.md`
- **Characters** (voices, relationships): `**/Wiki/Characters/**/*.md` (one file per character)
- **World** (setting, rules): look in `**/Wiki/Locations/**/*.md`, `**/Wiki/Systems/**/*.md`, `**/Wiki/Organizations/**/*.md` — capture the Wiki root directory path
- **Outline** (chapter purposes): `**/Wiki/Outline/outline.md`
- **Scene plan**: the book's established beat or scene file for the reviewed chapter, when present

Also find the previous chapter (for continuity checking).

Also locate these genre contract files:
- **Genre file**: search for `**/Wiki/Style/genre.md`
- **Synopsis** (for Genre Architecture): `**/Wiki/Story/synopsis.md`

**Do NOT read these files.** Just get their paths.

## Step 2: Launch the Review Worker

When the host supports isolated workers, start one with the description `Review latest chapter of [WorkingTitle]` and give it everything from the **"## Worker Instructions"** section below, replacing the placeholders:
  - `{{CHAPTER_PATH}}` → the chapter to review
  - `{{PREV_CHAPTER_PATH}}` → previous chapter path, or "not found"
  - `{{REVIEW_PATH}}` → `Wiki/Style/review_guide.md` or legacy `Wiki/Style/Review Guide.md` path, or "not found"
  - `{{STYLE_PATH}}` → `Wiki/Style/style_guide.md` or legacy `Wiki/Style/Style Guide.md` path, or "not found"
  - `{{CHARACTERS_DIR}}` → `Wiki/Characters/` directory path, or "not found"
  - `{{WIKI_DIR}}` → Wiki root directory path, or "not found"
  - `{{OUTLINE_PATH}}` → `Wiki/Outline/outline.md` path, or "not found"
  - `{{SCENE_PATH}}` → scene plan or beat source for this chapter, or "not found"
  - `{{GENRE_PATH}}` → `Wiki/Style/genre.md` path, or "not found"
  - `{{SYNOPSIS_PATH}}` → `Wiki/Story/synopsis.md` path, or "not found"
  - `{{WORKING_TITLE}}` → the working title
  - `{{WORKING_DIR}}` → the current working directory

If isolated workers are unavailable, or if you are already the dedicated worker for this task, follow the same instructions directly.

## Step 3: Save and Present Results

When the worker returns:
1. Save the review to `_Edits/Chapter-{N}_Review.md` (create `_Edits/` if it doesn't exist, using the same directory as the chapter files).
2. Display the full review with actionable edits.
3. Offer: "Would you like me to apply these edits, write the next chapter, or return to the outline?"
4. If the user wants edits applied, edit the chapter file to make only the approved changes.

---

## Worker Instructions

You are reviewing the most recently written chapter for revision opportunities.

**Working directory**: `{{WORKING_DIR}}`

### Step 1: Read Project Files

**Required**: Read `{{CHAPTER_PATH}}`

**Also read if they exist**:
- `{{PREV_CHAPTER_PATH}}` - for continuity checking
- `{{REVIEW_PATH}}` - forbidden phrases, AI-tells, mechanical checklist (primary editing reference)
- `{{STYLE_PATH}}` - POV, voice, character voices (for context only)
- `{{CHARACTERS_DIR}}` and `{{WIKI_DIR}}` - search the entire book Wiki for the proper nouns and canon claims actually present in the chapter or scene plan; read the supporting files, not every file by default
- `{{OUTLINE_PATH}}` - chapter purposes
- `{{SCENE_PATH}}` - required choices, consequences, emotional movement, motifs, arc movement, genre beats, and other chapter-specific obligations

**Genre contract files** (read if found):
- `{{GENRE_PATH}}` — if not "not found", read it and extract:
  - `## Core Tropes` — each mandatory trope name and description
  - `## Genre-Specific Craft Notes` — rules for this genre (what to always do, what to never do)
  - `## Reader Expectations` — especially the Resolution field (HEA/HFN requirement, tragic ending rule, etc.)
- `{{SYNOPSIS_PATH}}` — if not "not found", read it and extract from `## Genre Architecture` (if present):
  - `### Contract Validation` table — what tropes are PASS/FAIL/RISK for this story
  - `### Mandatory Beat Sequence` table — what genre beats must occur and in which scenes
  - `### Twist Map` table — what twists are planned and in which scenes

Store everything extracted from these two files as the **genre contract** for this review. If `{{GENRE_PATH}}` is "not found", genre compliance checking is skipped.

If files are missing, note which criteria cannot be fully evaluated.

### Step 2: Read References

Read:
- [references/review-criteria-detailed.md](references/review-criteria-detailed.md)
- [references/common-issues.md](references/common-issues.md)

### Step 3: Build Review Context

**From review.md (primary):**
- Forbidden phrases and words
- Crutch words, punctuation rules, triadic list ban
- Dialogue tag rules
- Protagonist voice check questions

**From style.md (context):**
- POV character and voice requirements
- Genre-specific expectations
- Pacing guidelines

**From character files (`Wiki/Characters/`):**
- Speaking patterns and verbal tics
- Relationship dynamics
- Character motivations

**From world files (`Wiki/Locations/`, `Wiki/Systems/`, `Wiki/Organizations/`):**
- Setting rules and constraints
- Location details
- Cultural rules

### Step 4: Evaluate Against 10 Criteria

1. **POV Voice & Agency** - Is voice present? Are they active?
2. **Scene-Plan Delivery & Plot Advancement** - Are the required choices, consequences, chapter-specific obligations, and ending movement present without becoming mechanical?
3. **Continuity Check** - Flows from previous? No contradictions?
4. **Character Authenticity** - Voices match? Consistent behavior?
5. **Reader Appeal & Genre Fit** - Meets expectations? Right pacing?
6. **Plot Logic** - No holes? Decisions make sense?
7. **Sensory Atmosphere** - Setting present? Physical sensation?
8. **World-Building Accuracy** - Locations correct? Rules followed?
9. **Style Guide Compliance** - No forbidden phrases? Format correct?
10. **Genre Compliance** - See detailed instructions below.

#### Criterion 10: Genre Compliance

Only evaluate this criterion if `{{GENRE_PATH}}` was found and the genre contract was successfully extracted. If the genre file was not found, skip this criterion entirely and note: "Genre file not found — genre compliance check skipped."

Check each of the following:

**A. Mandatory trope delivery**: For each mandatory trope from the genre contract, determine whether this chapter advances, establishes, or satisfies it. If a mandatory trope has no evidence in this chapter AND the outline position suggests it should have appeared by now, flag it as at risk or falling behind.

**B. Mandatory beat**: Check whether the outline assigns a mandatory genre beat to this specific chapter (from the `### Mandatory Beat Sequence` table in synopsis.md). If yes, determine whether the chapter delivers it fully, partially, or not at all. Flag missing or partial beats.

**C. Genre craft rule violations**: Check the chapter against every rule in `## Genre-Specific Craft Notes`. Flag each violation specifically — quote the offending passage and name the rule broken.

**D. Resolution trajectory**: If the genre requires HEA or HFN (from `## Reader Expectations`) and this is the final chapter, verify it delivers. If the genre forbids a tragic ending and this chapter ends tragically, flag it as a contract breach.

**E. Scheduled twist**: If the `### Twist Map` table assigns a twist to this chapter, check whether it is present and whether it lands with sufficient setup.

Output this section in the review report using the following format:

```markdown
## Genre Compliance

**Genre:** [subgenre from genre.md]
**Mandatory Beat for This Chapter:** [beat name from beat sequence table, or "none assigned"]
**Beat Delivered:** Yes / No / Partial — [one sentence explanation]

**Trope Tracking:**
| Trope | Status in This Chapter | Cumulative State |
|-------|----------------------|-----------------|
| [Trope name] | advances / absent / established / at risk | on track / falling behind / complete |

**Craft Rule Violations:** [bulleted list of violations with quoted passage and rule name, or "none found"]
**Twist Scheduled Here:** [twist name and description from Twist Map, or "none"]
**Resolution Trajectory:** [flag if contract breach, or "on track" or "not yet applicable"]
```

### Step 5: Generate Actionable Output

This is an EDITING PASS. For each issue found:

1. **Issue**: What's missing or weak (with line reference)
2. **Fix**: Specific text to revise
3. **Example**: Draft improved version

**Final output:**
- **Proposed Edits**: Numbered list with locations
- **New Lines to Add**: Draft text ready to insert
- **Lines to Cut**: What weakens the chapter

Do NOT provide ratings. Provide actionable edits.

Return the complete review report.

