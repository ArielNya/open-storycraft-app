---
name: fiction-storybible
description: "Write one storybible.md that carries a whole book — genre, audience, theme, synopsis, style, voice, characters, world, outline, scenes, psych — in the format storybible-import unpacks into Wiki/ files. Use when user says 'write a story bible', 'make a storybible', 'export my book as one file', 'bible for this story', 'plan the whole book in one document', or wants a portable single-file version of a project."
metadata:
  author: Fiction Toolkit
  version: "1.0.0"
  category: fiction-writing
  output-format: markdown
  workflow-position: 0.0
  requires: null
  next-skill: storybible-import
---

# Storybible Writer

**Output:** `storybible.md` in the book folder, beside `Wiki/`. One file, many documents. Format: [references/storybible-format.md](references/storybible-format.md). Skeleton: [assets/storybible-template.md](assets/storybible-template.md).

This skill writes the portable bible. It never writes `Wiki/` files — `storybible-import` does that, from this file.

## Step 1: Pick the mode

Read what the pack gave you.

- **Assemble** — Wiki files are in the pack. The book already exists: copy its canon into documents, verbatim.
- **Author** — no Wiki files. There is no book yet: ask, then write the bible from the answers.
- **Repair** — a `storybible.md` exists but is not importable, or is missing files the user wants. Rewrite it whole, keeping what it got right.

If it is ambiguous, ask one question with at most three options.

## Step 2: Gather input

**Assemble:** ask one question only: "Anything to leave out of the bible, or add to it? Or say 'all of it'." Do not re-interview a book that already exists.

**Author:** ask at most three questions, in one turn, then stop asking:

1. **What is the book?** "One or two sentences: who it happens to, what they want, and what the story is actually about."
2. **Genre and tone:** "Genre, subgenre, and any tropes you want — or say 'surprise me'."
3. **Scope:** "How many chapters should the outline cover, and do you want the world pack (locations, organizations, systems, events) or just the spine?"

If an answer is missing or says "surprise me", choose, mark the choice in your reply, and flag it for review.

## Step 3: Build the documents

Cover the full spine, in this order: genre, audience, theme, synopsis, style, voice, characters, world, outline, scenes, psych. Follow the field-level contract in [references/storybible-format.md](references/storybible-format.md) for every file — the frontmatter keys and section headings are what the rest of the app reads back.

Rules that matter more than volume:

- **Assemble copies, it does not rewrite.** Carry existing canon across word for word. This file is a copy of the book, not a new draft of it.
- **Author invents nothing downstream skills will be asked to honour.** If you do not know a character's secret, write the section as an open question rather than making one up.
- **One document per file.** Characters, world entries, scenes and psych passes are one document each, with `name:` or `chapter:` as the format requires.
- **Names come from the story.** No placeholders like `Protagonist` left behind.
- **Every document needs real content** (well over a short sentence), or the board will read the file as an empty stub.
- **The bible is allowed to be partial.** A spine-only bible imports cleanly; the board shows the rest as missing.

## Step 4: Save and confirm

Write the whole file to `storybible.md` in the book folder (create nothing else; the importer makes the folders).

Show the document list — the paths you emitted, in order — not the whole body. Ask: "What should I change?" Flag every choice you made that the user did not specify.

If changes are needed, rewrite `storybible.md` whole. Overwriting is safe: this file is the single home of the bible.

When approved: "Next: use `storybible-import` to unpack this bible into your book folder."
