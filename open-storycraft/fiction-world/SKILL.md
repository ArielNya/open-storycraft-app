---
name: fiction-world
description: "Build story setting: locations, organizations, systems, events. Use when user says 'build the world', 'create setting', 'world building', or after running fiction-characters."
metadata:
  author: Fiction Toolkit
  version: "5.0.0"
  category: fiction-writing
  output-format: markdown
  workflow-position: 4
  requires: fiction-synopsis, fiction-characters
  next-skill: fiction-outline
---

# World Builder

**Output:** one Markdown file per world entity in the project's `Wiki/` folder: `Wiki/Locations/<Name>.md`, `Wiki/Organizations/<Name>.md`, `Wiki/Systems/<Name>.md`, `Wiki/Events/<Name>.md`. Spaces in the entity name become underscores in the filename (`The_Amber_Coast.md`). Each world entry remains a Markdown file.

This skill runs its generation work in an isolated worker when the host supports one, to keep the main conversation context clean. The coordinator locates the input files and launches the builder; the builder reads context, generates, and writes the Wiki files; the coordinator presents results and runs the approval loop. If isolated workers are unavailable, follow the same builder instructions directly.

## Step 1: Identify Project and Locate Inputs

Search for `**/Wiki/Story/synopsis.md` from the working directory (the project may live in a subfolder). If multiple projects match, ask which one. If none: tell the user to run `fiction-synopsis` first and stop.

Also search for `**/Wiki/Characters/*.md` in the same project. Character files are optional; note whether any exist.

Do NOT read these files in the main conversation. Record their paths and the project root (the folder containing `Wiki/`).

## Step 2: Launch the World Builder

When the host supports isolated workers, start one with the description `Build world for [project]` and give it everything from the "## Worker Instructions" section below, with the placeholders filled in:
- `{{SYNOPSIS_PATH}}` = the synopsis file path
- `{{CHARACTERS_PATTERN}}` = the `Wiki/Characters/*.md` search pattern for this project, or "none found"
- `{{PROJECT_ROOT}}` = the project root folder (parent of `Wiki/`)
- `{{SKILL_DIR}}` = the directory containing this SKILL.md

If isolated workers are unavailable, or if you are already the dedicated worker for this task, follow the same instructions directly.

## Step 3: Present Results and Approval Loop

When the worker returns:
1. Show the entry list (names and tags only, grouped by type) to the user.
2. Ask: "Anything missing or to change?"
3. If changes are needed, use the same isolated-worker mechanism when available with the revision instructions plus the list of files already written; it edits or adds the affected `Wiki/` files only.
4. When approved: "Next: use `fiction-outline` to structure the story."

---

## Worker Instructions

You are a world-building expert who creates rich, functional settings.

**Project root**: `{{PROJECT_ROOT}}`

### Step 1: Load Context

- Read `{{SYNOPSIS_PATH}}` (required). Extract: premise, act structure, locations_in_play, characters_in_play, plus genre and tone if present (they set the technology or magic level and the atmosphere; infer rather than ask).
- Read every file matching `{{CHARACTERS_PATTERN}}` (optional). Extract: character names, roles, and the factions or groups they belong to.

If the synopsis file is missing or empty, report that the user must run `fiction-synopsis` first and stop.

### Step 2: Load Reference Files

Read these craft knowledge files before generating:
- `{{SKILL_DIR}}/references/locations-guide.md` for what makes a location narratively load-bearing: the sensory, social, and structural details that constrain or enable scenes
- `{{SKILL_DIR}}/references/world-building-principles.md` for what to build and what to leave out; how to build only what the story needs without over-constructing
- `{{SKILL_DIR}}/references/factions-guide.md` for how organizations create pressure on characters: what they permit, forbid, and demand; how institutional power shapes behavior

Extract: what makes a location entry useful for a writer, what Organization prevents/forces/enables fields should capture, what level of detail is enough for each entry type.

### Step 2b: Determine World Scope

Based on the synopsis:
- **Single location**: detail it deeply
- **Regional**: multiple connected locations
- **Continental or global**: sketch broadly, detail key locations

### Step 2c: Generate a Settlement Name Pool

If any location, settlement, or place names need to be invented (not supplied by the synopsis or character files), do not invent place names from scratch. First read the culture map at `../name-generator/references/culture-map.md`, identify the town list matching the setting's culture, then run the town generator for a pool of 15 authentic settlement names:

```bash
# Most cultures
python ../town-generator/scripts/generate_town.py \
  --list ../town-generator/data/<matching-list>.txt \
  --count 15 --order 2 --epithet-chance 0.3

# Spanish / Iberian settings (adds San/Santa/Monte prefixes)
python ../town-generator/scripts/generate_town.py \
  --list ../town-generator/data/spanish-cities.txt \
  --prefix-file ../town-generator/data/spanish-prefixes.txt \
  --prefix-chance 0.35 --epithet-chance 0.25 --count 15
```

Use epitheted names (e.g. "Vorheim, the Throne of the Dead") for major cities and capitals, bare names for minor settlements. If no cultural match exists, use `fantasy.txt`.

### Step 2d: Generate Settlement Profiles

For each key settlement (typically the 2 to 4 settlements most relevant to the protagonist's journey), run the settlement generator:

```bash
python ../fantasy-settlement/scripts/generate_settlement.py \
  "<settlement name>" "<type description>"
```

The `<type description>` combines biome and size (e.g. `"coastal city"`, `"underground village"`, `"sky town"`). For scifi settings, use scifi keywords (e.g. `"asteroid mining station"`, `"orbital colony"`). Use the output to populate population, economy, notable figures, and factions. Synthesize it into the entries; do not paste raw generator output verbatim.

### Step 3: Generate the World Entries

Use `{{SKILL_DIR}}/references/json-schema.json` as the field checklist: every required field for an entry type must appear in that entry's file, as a frontmatter key or a `##` section. Do not drop fields; do not rename them beyond snake_case.

**Every entry requires:** `name`, `tag`, `type`

**tag**: One line: what this entry is and why it matters to the story. Written for a writer scanning a list, not for a reader.

**Entry types and their fields:**

#### Location
Required: `name`, `tag`, `type`, `subtype`, `physical_description`, `sensory_description`
Optional: `also_known_as`, `owner`, `social_rules`
Add `prevents`, `forces`, `enables` when the location has rules that create scene-level constraints (brothels, private clubs, restricted spaces).

#### Organization
Required: `name`, `tag`, `type`, `description`
Optional: `prevents`, `forces`, `enables`, each a single present-tense sentence stating what characters can and cannot do in scenes involving this organization.

#### System
Required: `name`, `tag`, `type`, `subtype`
Fields vary by subtype:
- Social_Norm: `physical_description`, `sensory_description`, `social_rules`
- Household: `members`, `rules`, `slice_of_life_texture`
- Economy: `currency`, `key_industries`, `trade_system`
- Culture: `traditions`, `language_dialects`, `arts_entertainment`

#### Event
Required: `name`, `tag`, `type`, `when`, `what`, `implications`

### Step 4: What to Include

Cover everything referenced in the synopsis's locations_in_play and characters_in_play. For each location a character inhabits, create a Location entry. For each faction or institution with power over characters, create an Organization entry. For background systems that shape behavior (economy, culture, social norms), create System entries. For past events that drive the present, create Event entries.

Build only what matters to the story. Do not create entries for places or groups that never appear or exert pressure on characters. Every element should enable conflict, limit solutions, or force difficult choices. Leave room for mystery: not everything explained, some history debated, some rules not fully understood.

### Step 5: Write the Wiki Files

Write one Markdown file per entry, into the folder matching its type:

| type | folder |
|------|--------|
| Location | `{{PROJECT_ROOT}}/Wiki/Locations/` |
| Organization | `{{PROJECT_ROOT}}/Wiki/Organizations/` |
| System | `{{PROJECT_ROOT}}/Wiki/Systems/` |
| Event | `{{PROJECT_ROOT}}/Wiki/Events/` |

Create the folders if they do not exist. Filename: the entry's display name with spaces replaced by underscores, plus `.md` (`Crimson_Veil_Casino.md`).

File format:
- YAML frontmatter carries the scalar machine-readable fields: `name`, `tag`, `type`, `subtype`, and any short one-line fields (`also_known_as`, `owner`, `when`). Plain, conservative YAML only: block lists (one `- item` per line), never inline `[a, b]` lists; quote strings containing colons.
- The Markdown body carries the longer fields as `##` sections, one per field, using the field name as the heading (`## physical_description`, `## sensory_description`, `## prevents`, `## implications`, and so on).
- Lists of structured items inside a field become labeled sub-bullets, one bold field label per line (e.g. `- **members:** ...`), so they stay grep-able.

Example (`Wiki/Locations/Crimson_Veil_Casino.md`):

```markdown
---
name: Crimson Veil Casino
tag: "Neutral-ground casino where the syndicates meet; violence inside is forbidden"
type: Location
subtype: Casino
owner: Madame Iris
---

## physical_description
...

## sensory_description
...

## social_rules
...

## prevents
...
```

Writing a file for an entry name that already exists overwrites it, so re-runs and revisions are safe.

### Step 6: Return the Entry List

Return to the coordinator: the list of files written (absolute paths), and for each entry its name, type, and tag. Do not return the full file contents.

