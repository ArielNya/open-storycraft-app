---
name: storybible-import
description: Unpack a storybible.md into a real book folder — Wiki/Style, Wiki/Story, Wiki/Characters, Wiki/Outline, Wiki/Psych, Chapters — so the Open Storycraft spine can continue. Use when the user says 'import my story bible', 'turn this bible into a book', 'set up the project from storybible.md', 'create the book folder from my bible', or hands over a bible file for a project that does not exist yet.
metadata:
  author: Fiction Toolkit
  version: "1.0.0"
  category: fiction-writing
  output-format: markdown
  workflow-position: 0.0
  requires: null
  next-skill: fiction-genre
---

# Storybible Import

**Output:** one file per document in `storybible.md`, written under the book folder (`Wiki/Style/*`, `Wiki/Story/*`, `Wiki/Characters/*`, `Wiki/Outline/*`, `Wiki/Psych/*`, `Chapters/*`).

This is a local tool, not a model call. It runs on device with no provider and no API key, and it is deterministic: the same bible always produces the same files. It never invents content — every byte it writes comes from the bible.

## Step 1: Find the bible

In order:

1. `<folder>/storybible.md`
2. `<folder>/Wiki/storybible.md`
3. `<folder>/<one subfolder>/storybible.md`

The **book folder is the folder that holds the bible.** Files land beside it: `Wiki/…` for the three cases above, or `<subfolder>/Wiki/…` when the bible was found one level down. That is how a bible dropped into `Books/storybible.md` becomes the book in `Books/<working-title>/`.

If no bible is found, stop and say so. Do not guess a book from the folder name.

## Step 2: Read the documents

The format is documented in full at [../fiction-storybible/references/storybible-format.md](../fiction-storybible/references/storybible-format.md). In short: one `---` block per document, carrying `path:` or `slot:` plus the file's own frontmatter, then the body.

| In the frontmatter | Lands at |
|---|---|
| `slot: genre` / `audience` / `theme` / `synopsis` / `style` / `voice` / `outline` | the matching `Wiki/` spine file |
| `slot: character` + `name:` | `Wiki/Characters/<Name>.md` |
| `slot: location` / `organization` / `system` / `event` + `name:` | `Wiki/<Locations|Organizations|Systems|Events>/<Name>.md` |
| `slot: scene` + `chapter:` | `Wiki/Outline/Chapter_NN_Scene.md` |
| `slot: psych` + `chapter:` | `Wiki/Psych/Chapter_NN_Psych.md` |
| `slot: chapter` + `chapter:` | `Chapters/Chapter-NNN.md` |
| `path: <relative .md>` | exactly there |

Text outside the documents (a title, a table of contents) is ignored. The routing keys are stripped, so each written file keeps only its own frontmatter.

## Step 3: Show the plan, then wait

Preview first. The run produces a preview that lists every destination and then repeats every document; nothing is written until it is saved. Show the destination list and confirm the count: "12 files into `Books/salt-ledger/`. Nothing is written until you save."

Stop and ask when the bible:

- names a `slot:` that is not in the table above;
- names a `name:` slot with no `name:`, or a chapter slot with no numeric `chapter:`;
- sends a `path:` outside the book folder, or to something that is not `.md`;
- has no documents at all.

Report the offending document and the reason. Do not repair the bible in place — send the user to `fiction-storybible` to rewrite it, or edit the one line yourself if the user names the fix.

## Step 4: Save and hand back

On save, every document is written, creating folders as needed, and files that already exist are replaced. Rejecting the preview leaves the folder exactly as it was.

Then report what the book now reports:

```
storycraft status --project <book folder>
```

The board reads the new files, marks the filled slots `yes`, and `Next:` names the first skill still missing — usually `fiction-audience` after a genre-only bible, or `fiction-writechapter` when the bible carried the whole spine.

Two things worth saying out loud when they apply:

- If `Next: storybible-import` still appears, the bible was found but nothing importable was in it.
- `storybible.md` stays where it is. It is the portable copy of the book; the Wiki is canon from here on. Re-importing is how a bible edit reaches the book.

When approved: "Next: run your first missing spine skill, or edit the bible and import again."
