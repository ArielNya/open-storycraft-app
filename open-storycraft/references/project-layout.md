# Project layout the orchestrator expects

Project root = the directory that contains `Wiki/`. Search `**/Wiki/` rather than assuming cwd.

```
<project>/
├── Wiki/
│   ├── Style/
│   │   ├── genre.md
│   │   ├── audience.md
│   │   ├── forbidden.md          # optional, from audience
│   │   ├── style_guide.md
│   │   ├── review_guide.md
│   │   └── voice_prompt.md
│   ├── Story/
│   │   ├── theme.md
│   │   └── synopsis.md
│   ├── Outline/
│   │   └── outline.md
│   ├── Characters/               # one .md per character
│   ├── Locations/
│   ├── Organizations/
│   ├── Systems/
│   ├── Events/
│   ├── Psych/
│   │   └── Chapter_NN_Psych.md
│   └── Scenes/                   # if fiction-scenes uses this folder
└── Chapters/                     # drafted prose; also accept Chapter_*.md anywhere
```

## Status-board checks

| Slot | Yes when |
|---|---|
| genre | Wiki/Style/genre.md exists and has a genre field or heading |
| audience | Wiki/Style/audience.md exists |
| theme | Wiki/Story/theme.md exists |
| synopsis | Wiki/Story/synopsis.md exists and is not an empty stub |
| style | Wiki/Style/style_guide.md exists |
| characters | Wiki/Characters/ has at least one .md besides a template |
| world | any file under Wiki/Locations, Organizations, Systems, or Events |
| outline | Wiki/Outline/outline.md exists |
| scenes | scene/beat files exist for the current chapter |
| voice | Wiki/Style/voice_prompt.md exists |
| psych | Wiki/Psych/ has a file for the current chapter |
| chapters | a chapter prose file exists |

partial = folder exists but the needed file is empty, template-only, or covers the wrong chapter.

## Creating a new project

Only in `new-project` mode after the user agrees, or when fiction-genre is about to write and no Wiki/ exists.

1. Slug the working title (`My Book` → `my-book`). Until a title exists, use `new-story`.
2. Create the Wiki subfolders listed above. Do not prefill markdown files.
3. Let the active child skill write its own output file.

If the host cwd is `/home/workdir/artifacts`, put the project at `/home/workdir/artifacts/<slug>/` unless the user named another path.

## Chapter file matching

Accept any of these, first match wins after asking if several collide:

- `**/Chapters/Chapter_{N}*`
- `**/Chapter-{N}*`
- `**/Chapter_{N}*`
- `**/Chapter {N}*`
- the explicit path the user gave

Scene files may be `Chapter_{N}_Scene_*.md`. Treat them as scene units for burstiness / edit skills, but roll reports up per chapter.

## What the orchestrator must not write

- character sheets (fiction-characters)
- synopsis body (fiction-synopsis)
- outline beats (fiction-outline)
- chapter prose (fiction-writechapter)
- editorial rewrites except by running the named edit skill

The coordinator only creates empty folders and the status board.
