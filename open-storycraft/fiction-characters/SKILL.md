---
name: fiction-characters
description: "Generate one Wiki/Characters/[Name].md file per character with full profiles. Personality and dialogue_style are WRITING INSTRUCTIONS, not descriptions. Use when user says 'create characters', 'generate cast', 'build character profiles', or after running fiction-synopsis."
metadata:
  author: Fiction Toolkit
  version: "3.1.0"
  category: fiction-writing
  output-format: markdown
  workflow-position: 3
  requires: fiction-synopsis
  next-skill: fiction-world
---

# Character Generator

**Output:** One Markdown file per character at `Wiki/Characters/<Character_Name>.md` (display name with spaces replaced by underscores, e.g. `Kael_Veyra.md`). The `Wiki/` folder is the single source of truth; the character files remain the source of truth.

## Step 1: Load Context

All inputs come from the project's `Wiki/` folder. Search for the files, since the project may be a subfolder (e.g. `**/Wiki/Story/synopsis.md`).

- `Wiki/Story/synopsis.md`: required. For: protagonists, mechanism, characters_in_play, act structure.
- `Wiki/Style/genre.md`: for: tropes (especially romance/power dynamic tropes).
- `Wiki/Style/audience.md`: for: protagonist_ages, content_boundaries.
- `Wiki/Style/style_guide.md`: for: style (the prose register the character voices must fit). Also accept the legacy name `Wiki/Style/Style Guide.md` when reading, but never write it.

If `Wiki/Story/synopsis.md` is missing or empty: tell the user to run `fiction-synopsis` first.

## Step 2: Gather User Input

Ask the user using the host's normal question mechanism:

**Question:** "Any specific characters you want to add, or adjust? Any cultural flavor for names? Or say 'derive from synopsis' to build the full cast from what's already there."

## Step 3: Generate the Character Files

Use one isolated worker when the host supports one. Give it the instructions below, replacing placeholders:
- `[working directory]` → current working directory
- `[wiki root]` → the resolved path to the project's `Wiki/` folder
- The full text of each context file found in Step 1
- `[user's answer from Step 2]`

If the host does not support isolated workers, or if you are already the dedicated worker for this task, follow the same instructions directly.

---

## Worker Instructions

Generate the character files under `Wiki/Characters/` for this project.

**Working directory:** [working directory]
**Wiki root:** [wiki root]
**Synopsis:** [full Wiki/Story/synopsis.md content]
**Genre:** [Wiki/Style/genre.md content if found, else "none"]
**Audience:** [Wiki/Style/audience.md content if found, else "none"]
**Style:** [Wiki/Style/style_guide.md content if found, else "none"]
**User additions:** [user's answer from Step 2]

**Name sourcing:** If any character names must be invented (not supplied by the user or the synopsis), use `name-generator` first and select from its output. Do not invent names from scratch.

**Field checklist:** Cover every field in [references/json-schema.json](references/json-schema.json) for the character's role as either a frontmatter key or a `##` section in that character's Markdown file. Do not drop fields; do not rename them beyond snake_case.

Also read [references/anti-flat-rules.md](references/anti-flat-rules.md): these rules govern what makes a character feel real vs. functional. Apply them to every entry.

### Determine the Cast

One character per name in the synopsis's characters_in_play. Add any others who are clearly implied by the plot.

**Required (every story):**
1. Protagonist
2. Antagonist (with LEGITIMATE motivation)

**Add as needed:**
3. Deuteragonist / Love Interest
4. Ally / Confidant (with their own agenda)
5. Mentor
6. Supporting cast

**Match cast size to story scope.** A 10-scene one-shot needs 2-3 characters. A 30-chapter novel needs 6-10.

### Rules

**Role types:**
- `protagonist`: use all protagonist fields including internal_monologue, chemistry_with_protagonist, romantic_obstacles, relationship_dynamics
- `antagonist`: use antagonist fields: motivations, methods, power_resources, relationship_to_protagonist
- `supporting`: use supporting fields: personality, skills_or_resources, role_in_story, relationship_to_protagonist
- Add `also_known_as` to any character known by multiple names

**Physical description**: Specific and visual. Not a catalog of features: the two or three things that a stranger would register, that the other POV character would notice first, that carry the character's status or history in their body.

**Personality**: This is a writing instruction. It tells the writer who this person IS, not how to categorize them. Include: what makes them funny or dangerous, their social intelligence, the wound underneath the armor, what they want badly enough to be stupid about. Also include who they are privately, what they notice, what they are insecure about, how their body reacts under stress, and the gap between what they show and what they feel. A one-line trait list ("controlled, precise, darkly amused") is a description and fails; the field must let the writer be this character from inside their skull.

**dialogue_style**: First person [tense from the audience or synopsis file]. This field is read directly before writing any line of their POV. It must give the writer the voice: register, wit level, the gap between what they say and what they mean, what they fall back on when something costs them. Include how their internal monologue sounds, not just how they speak out loud. "She speaks precisely" is a description and fails.

**internal_monologue**: Protagonists only. This is the most important field for POV writing. It describes how the voice sounds in the reader's head: what it notices first, what it jokes about, where it goes quiet, how sensation arrives before the intellect. It is a writing instruction, not a character description.

**chemistry_with_protagonist**: For each protagonist: what the other one does that gets through their armor. Not "they're attracted to each other"; the specific thing, the quality or behavior that works on them despite themselves.

**relationship_dynamics**: Per named relationship in the story. How this character reads each important person: what they see, what they miss, what they won't say, what they observe and file away.

**Antagonist motivations**: Never "evil for evil's sake." The antagonist must have a coherent reason that makes sense from inside their worldview. State it plainly.

**Field constraints:**
- Max 120 words per field
- Physical description can be brief (one line is fine)
- Age is a specific number
- Want / Need / Flaw / Secret are one to three sentences each

**Key rules:**
- The user's character details are canon. Your inventions are suggestions.
- Flag every AI-invented character with `ai_invented: true` in that character's frontmatter.

### Anti-Flat Checklist

Every character must pass before saving:
- [ ] Has WANT, NEED, FLAW, SECRET (protagonists at minimum)
- [ ] Antagonist has legitimate motivation and a sympathetic quality
- [ ] At least one ally has a conflicting agenda
- [ ] No two characters sound the same (untagged dialogue test)
- [ ] Personality is a writing instruction, not a description

### Save

Write one file per character to `[wiki root]/Characters/<Character_Name>.md`, replacing spaces in the display name with underscores. Create the `Wiki/Characters/` folder if it does not exist. Overwrite by filename, so the step is safe to re-run.

**File format:**
- YAML frontmatter carries the scalar fields: `name`, `role`, `age`, `pronouns`, and `also_known_as` / `ai_invented` when applicable. Frontmatter is plain, conservative YAML: block lists one item per line, never inline `[a, b]`; quote any string containing a colon.
- The Markdown body carries the prose fields, one `##` section each, snake_case headings matching the schema field names: `## physical_description`, `## personality`, `## dialogue_style`, `## backstory`, `## want`, `## need`, `## flaw`, `## secret`, `## chemistry_with_protagonist`, `## romantic_obstacles`, `## shared_history`, `## internal_monologue`, `## relationship_dynamics`, and for antagonists `## motivations`, `## methods`, `## power_resources`, `## relationship_to_protagonist`, and for supporting cast `## skills_or_resources`, `## role_in_story`, `## relationship_to_protagonist`.
- Under `## relationship_dynamics`, use one `### <Other Character Name>` sub-heading per named relationship so the entries stay grep-able.

Return: "Character files written to Wiki/Characters/" plus a one-line summary of the cast (roles and names).

### Seed relationship checkpoints (the starting state of each bond)

Relationships are **temporal**: they evolve chapter by chapter, and the profile prose in `relationship_dynamics` is not enough on its own. The profile is the standing portrait; the checkpoint is the dated state a chapter prompt should inject and that later turns update as the story moves.

After the cast files are written, seed the **chapter-1** state for each meaningful pair inside BOTH characters' files. Add a `## relationship_checkpoints` section with one labeled entry per bond:

```markdown
## relationship_checkpoints

### <Other Character Name>
- **link_type:** romantic | ally | family | rival | friend | antagonist
- **chapter_start:** 1
- **chapter_end:** (open)
- **notes:** one-line, spoiler-safe state of this bond at the story's start
```

Only author pairs with real history or a defined dynamic at the opening; unlinked pairs fall back to the first-meeting directive automatically. Later turns in a relationship are appended as new dated entries under the same `###` heading during `fiction-outline` and `fiction-scenes`, not here; when adding a later checkpoint, close the prior open span by setting its `chapter_end` to the chapter before the new checkpoint. Seeding at chapter 1 with an open end is always safe.

---

## Step 4: Present and Confirm

Show the worker's summary and ask: "What should I change?"

If changes are needed, use the same isolated-worker mechanism when available with the revision instructions plus the list of files already written, so it edits the existing `Wiki/Characters/` files instead of regenerating the cast from scratch.

When approved: "Next: use `fiction-world` to build the setting."

