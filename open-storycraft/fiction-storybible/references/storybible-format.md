# Storybible format

A storybible is **one markdown file that carries a whole book**. Each document
inside it becomes one file in the book folder. `fiction-storybible` writes a
storybible; `storybible-import` unpacks it into `Wiki/` and `Chapters/`.

Nothing else in Open Storycraft reads this file. The Wiki is still canon; the
bible is the portable copy of it.

## 1. Document syntax

A document is a `---` frontmatter block, then a body:

```markdown
---
path: Wiki/Style/genre.md
working_title: "Salt Ledger"
genre: Mystery
---

# Genre

Body markdown for that file.
```

- The frontmatter must contain a **routing key**: `path:` or `slot:`.
- Everything else in the frontmatter is written to the file unchanged.
- `path:` and `slot:` are consumed by the host and never written to a file.
- Text outside documents — a title, a table of contents, notes — is ignored by
  the host and kept for the reader. Use it.
- A body may contain `---` (a horizontal rule) as long as the next block of
  `key: value` lines that follows it has no `path:` or `slot:`.
- Write documents in spine order: genre, audience, theme, synopsis, style,
  voice, characters, world, outline, scenes, psych.

## 2. Naming the destination

Use `path:` when you know the file name (world entries, characters, anything
unusual). Use `slot:` for the fixed spine files.

### Fixed slots (`slot:` → one destination)

| `slot:` | File |
|---|---|
| `genre` | `Wiki/Style/genre.md` |
| `audience` | `Wiki/Style/audience.md` |
| `theme` | `Wiki/Story/theme.md` |
| `synopsis` | `Wiki/Story/synopsis.md` |
| `style` | `Wiki/Style/style_guide.md` |
| `voice` | `Wiki/Style/voice_prompt.md` |
| `outline` | `Wiki/Outline/outline.md` |

### Named slots (`slot:` + `name:` → one file per name)

| `slot:` | File |
|---|---|
| `character` | `Wiki/Characters/<Name>.md` |
| `location` | `Wiki/Locations/<Name>.md` |
| `organization` | `Wiki/Organizations/<Name>.md` |
| `system` | `Wiki/Systems/<Name>.md` |
| `event` | `Wiki/Events/<Name>.md` |

Spaces in `name:` become underscores (`Kael Veyra` → `Wiki/Characters/Kael_Veyra.md`).

### Chapter-scoped slots (`slot:` + `chapter:`)

| `slot:` | File |
|---|---|
| `scene` | `Wiki/Outline/Chapter_NN_Scene.md` |
| `psych` | `Wiki/Psych/Chapter_NN_Psych.md` |
| `chapter` | `Chapters/Chapter-NNN.md` |

`chapter:` is a bare number (`chapter: 3`). The host pads it: scene and psych
files use two digits (`Chapter_03_Scene.md`), chapter prose uses three
(`Chapters/Chapter-003.md`).

**Any other `slot:` is rejected.** If a file does not fit a slot, use `path:`.
A `path:` must be relative, must stay inside the book folder (no `..`, no
leading `/`), and must end in `.md`.

## 3. What each file must contain

Downstream skills read these shapes. Keep the frontmatter keys and section
headings; write the content.

### `Wiki/Style/genre.md`

Frontmatter: `working_title`, `genre`, `subgenre`, `flavor` (underscored, e.g.
`Harbour_Fog`), `tropes` (block list, snake_case).

Body: `# Genre`, `## tone_notes` (one paragraph, specific to this story), then
`## Trope Commitments` with one `### <trope_id>` per trope and the bold labels
`role:` (`genre_contract` | `structural_engine` | `supporting_texture` |
`deliberate_subversion`), `reader_expectation:`, `planning_obligation:`.

`genre:` or a `# Genre` heading is mandatory — the status board reads it to
decide whether the genre slot is filled.

### `Wiki/Style/audience.md`

Frontmatter: `age_group`, `typical_reader_age`, `tone_variant`.

Body: `# Audience Profile`, `## Protagonist Ages`, `## Content Boundaries`
(`violence:`, `sexual_content:`, `language:`, `darkness:` each with `level: 1-5`
and `note:`), `## Structural Norms` (`chapter_length_words:`,
`total_word_count:`, `pov_structure:`, `series_potential:`).

### `Wiki/Story/theme.md`

Frontmatter: `central_question`.

Body: `## Tone`, then `## Motifs` with one `### <Motif Name>` per motif and the
bold labels `type:` and `what_it_is:`.

### `Wiki/Story/synopsis.md`

Frontmatter: `title`, `central_question`, `reader_promise`,
`characters_in_play` (block list), `locations_in_play` (block list).

Body: `# <Title>: Synopsis`, `## Premise`, `## Story Answer`, `## Protagonists`,
`## Mechanism`, `## Trope Payoff Plan` (one `### <trope_id>` with bold `role:`,
`setup:`, `development:`, `payoff:`), then `## Act 1`, `## Act 2`, `## Act 3`
each with bold `shape:` / `ends_with:` and the act's own labels, and
`## Open Threads`.

### `Wiki/Style/style_guide.md`

Frontmatter: `title`, `pov_mode` (`single` | `dual`), `person` (`first` |
`third`), `tense` (`past` | `present`), `pov_1`, `pov_2`, `switching`.

Body: `## Style` — one paragraph, injected verbatim into every chapter prompt.
Add `## Banned` when the story has specific tell-words to avoid.

### `Wiki/Style/voice_prompt.md`

No frontmatter.

Body: `# Voice Prompt: <Protagonist> Close-<POV> Rewrite`, `## Core POV Rule`,
`## Who <Protagonist> Is`, `## Interior Voice`, `## What They Notice`,
`## Emotional Translation Rules`, `## Language Firewall`, `## Before / After
Examples`, `## Final Standard`.

### `Wiki/Outline/outline.md`

Frontmatter: `title`, `chapter_count`, `premise`, `central_question`,
`reader_promise` (these are carried verbatim from the synopsis).

Body: one `## Chapter NN: <Title>` per chapter, zero-padded two digits, each
with the bold labels `chapter_number:`, `title:`, `what_must_happen:`,
`ends_with:`. Add `premise_movement:`, `question_pressure:`,
`promise_delivery:`, `trope_obligations:` when they carry real information.

The host reads `## Chapter NN` headings to work out which chapter is next, so
the heading format matters.

### `Wiki/Characters/<Name>.md`

Frontmatter: `name`, `role` (`protagonist` | `antagonist` | `supporting` | …),
`age`, `pronouns`.

Body: `## personality`, `## want`, `## need`, `## flaw`, `## voice` (or
`## dialogue_style`), `## backstory`. Antagonists add `## motivations`,
`## methods`, `## relationship_to_protagonist`. Supporting cast add
`## role_in_story`, `## relationship_to_protagonist`.

### World entries (`Wiki/Locations|Organizations|Systems|Events/<Name>.md`)

Frontmatter: `name`, plus `type` where useful.

Body: what it is, what it does to the story, who controls it, and one concrete
sensory detail a chapter could use.

### `Wiki/Outline/Chapter_NN_Scene.md`

Body: `# Chapter <N>: <Title>`, `## Chapter spine` with the bold labels `Want:`,
`Pressure:`, `Decision:`, `Irreversible change:`, `Why this chapter cannot be
cut:`, then `## Beats` (numbered) or `## Scene 1`, `## Scene 2` with bold
`Goal:`, `Action and turn:`, `Interior movement:`, `Consequence or hook:`.

### `Wiki/Psych/Chapter_NN_Psych.md`

Body: `# Chapter NN: Psychological Interior Pass`, the bold header lines
`Chapter:`, `POV:`, `Setting:`, `External arc:`, `Emotional arc:`, then
`## Start State`, `## Beat-by-Beat Interior Cost`, `## End State`,
`## Show-Don't-Tell Gaps`, `## Bottom Line`.

### `Chapters/Chapter-NNN.md`

Only include this when the book already has drafted prose to carry across.
Drafting new prose is `fiction-writechapter`'s job, not the bible's.

## 4. Rules that keep the import valid

1. **Every file needs substance.** Under about 40 characters of real content is
   treated as an empty stub, and the slot shows as `partial` on the board.
2. **Plain, conservative YAML.** One `- item` per line for lists, never inline
   `[a, b]`. Quote any value containing a colon.
3. **Never invent canon while assembling.** If the packed Wiki files already
   hold a file, carry its text across verbatim. Split, reorder, and route —
   do not paraphrase, summarise, or "improve" it.
4. **A partial bible is legal.** Import whatever exists; the status board shows
   the rest as missing and the spine picks up there.
5. **Do not translate names.** Keep the spelling the book uses.

## 5. What the importer refuses

`storybible-import` stops with a message, and writes nothing, when:

- no document is found (no `---` block with `path:` or `slot:`);
- a `slot:` is not in the tables above;
- a named slot has no `name:`, or a chapter-scoped slot has no numeric
  `chapter:`;
- a `path:` is absolute, contains `..`, or is not a `.md` file.

## 6. Smallest legal bible

```markdown
# My book

---

slot: genre
working_title: "Working Title"
genre: Fantasy
---

# Genre

## tone_notes

One paragraph of tone notes, specific to this story and long enough to matter.

---

slot: character
name: Mira
role: protagonist
---

# Mira

## personality

Precise, not cold. Keeps her own record of everything the town says out loud.
```

Import that and the board shows genre and characters filled, everything else
missing, and `Next: fiction-audience`.
