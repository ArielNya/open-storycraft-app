---
name: storybible-convert
description: "Convert any story bible — a Google Docs or Word export, a Notion or World Anvil page, a Scrivener outline, loose notes — into a storybible.md that storybible-import can unpack into the book's Wiki files. Use when the user says 'convert my story bible', 'import my notes', 'use my existing bible', 'my bible is in a different format', or when storybible.md exists but import reports no documents."
metadata:
  author: Fiction Toolkit
  version: "1.0.0"
  category: fiction-writing
  output-format: markdown
  workflow-position: 0.0
  requires: null
  next-skill: storybible-import
---

# Storybible Converter

**Input:** `storybible.md` in the book folder, in whatever shape the author wrote it.
**Output:** the same file rewritten as storybible documents, in the format [../fiction-storybible/references/storybible-format.md](../fiction-storybible/references/storybible-format.md). On save, the author's original is kept beside it as `storybible.source.md`.

This skill only restructures. It does not write Wiki files — `storybible-import` does that next, on device — and it does not write new canon. Every sentence in the output comes from the source.

## How the host runs this

Bibles are long, so the host sends the source **one section at a time** (cut at headings) and merges the answers: documents that land on the same file are joined, the first value of each frontmatter key wins. That has three consequences for every answer:

1. **Convert only the section in front of you.** Do not repeat, summarise, or anticipate material from other sections. If a character is only named here, do not write a profile for them.
2. **Answer with documents only.** No title, no table of contents, no notes to the user, no code fences. Text outside documents is discarded.
3. **Nothing to keep?** Answer exactly `NO DOCUMENTS`. Use this for a cover page, a table of contents, a changelog — never for story material you could not place (see Step 3).

## Step 1: Recognise what each passage is

Bibles name things differently. Route by **what the passage is**, not by its heading.

| The source has | Route to |
|---|---|
| Genre, subgenre, tropes, comparable titles, "vibe", tone keywords | `slot: genre` |
| Target reader, age range, content limits, heat level, word count | `slot: audience` |
| Theme, "what it is about", central question, motifs, symbols | `slot: theme` |
| Premise, logline, blurb, plot summary, act structure, ending | `slot: synopsis` |
| POV, tense, prose rules, banned words, voice samples for narration | `slot: style` |
| A protagonist's interior voice, how they think and notice | `slot: voice` |
| A cast member: bio, appearance, personality, arc, relationships | `slot: character` + `name:` |
| A place, city, region, building, ship | `slot: location` + `name:` |
| A faction, guild, family, company, government, religion's institution | `slot: organization` + `name:` |
| A magic system, technology, economy, law, language, calendar | `slot: system` + `name:` |
| A historical event, war, disaster, prophecy, backstory incident | `slot: event` + `name:` |
| A chapter list, chapter-by-chapter plan | `slot: outline` (one document for the whole list) |
| Beats or scenes for one chapter | `slot: scene` + `chapter:` |
| A character's emotional state across one chapter | `slot: psych` + `chapter:` |
| Drafted prose for a chapter | `slot: chapter` + `chapter:` |

A passage that mixes several — a character bio that describes their hometown — is split: the hometown part goes to the location document, the rest to the character.

## Step 2: Shape each document

Use the frontmatter keys and section headings the format reference lists for that file. Downstream skills read them.

- **Keep the author's words.** Move sentences under the right heading; do not paraphrase, summarise, polish, or translate them. Light trimming of filler ("as I said above") is fine.
- **Keep the author's language.** A bible written in Portuguese stays in Portuguese. Only frontmatter keys, slot names, and section headings use the English names the format requires — values and bodies stay as written.
- **Frontmatter values come from the source or are left out.** If the bible never says the protagonist's age, write no `age:` key. A missing key shows as `partial` on the status board, which is honest; an invented one becomes canon.
- **Names stay exactly as spelled.** `name:` uses the bible's spelling; the importer turns spaces into underscores for the file name.
- **Outline chapters** get `## Chapter NN: <Title>` headings with zero-padded numbers. The host counts them.
- Plain YAML: one `- item` per line for lists, quote any value containing a colon.

## Step 3: Never drop story material

If a passage is story material and fits no slot — a glossary, a timeline, research notes, a playlist the author uses for mood, open questions — keep it:

```markdown
---
path: Wiki/Notes/<Short_Topic>.md
---

# <Topic>

<the passage, verbatim>
```

The importer writes `Wiki/Notes/` like any other folder. Nothing in the source is lost; the author can delete what they do not need.

## Step 4: What the author does next

The preview shows the converted bible and a diff against the original. Nothing is written until they save. After saving:

1. The board shows `Next: storybible-import`. Run it: it lists every file it will write and asks before writing.
2. The board then shows which spine slots the bible filled and which skill comes next.

Point out the gaps worth filling first, in spine order — usually missing frontmatter on genre and audience, then characters with no `want`/`need`/`flaw`.

## Example

Source section:

```markdown
## Mira Solano
28, she/her. Tide clerk at the harbour office in Port Vell. Counts everything twice.
Wants: to prove the posted tide tables are rigged. Fear: being the one who is wrong in public.
Port Vell itself is a fog town built on stilts; the office smells of tar and wet paper.
```

Answer:

```markdown
---
slot: character
name: Mira Solano
role: protagonist
age: 28
pronouns: she/her
---

# Mira Solano

## personality

Counts everything twice.

## want

To prove the posted tide tables are rigged.

## flaw

Fear: being the one who is wrong in public.

## backstory

Tide clerk at the harbour office in Port Vell.

---
slot: location
name: Port Vell
---

# Port Vell

A fog town built on stilts; the office smells of tar and wet paper.
```

`role: protagonist` is only right if the source says or plainly implies it. When it does not, leave `role:` out.
