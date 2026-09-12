# Open Storycraft

Unpacked from the single-file library dump into the intended skill architecture.

Forty-two craft-first skills plus the `open-storycraft` orchestrator. Each skill is a folder with `SKILL.md` plus optional `references/`, `assets/`, `scripts/`, and `data/`.

Start a book or resume one by invoking **open-storycraft**. It does not rewrite the child skills. It finds the project Wiki, prints a status board, and runs one skill at a time along the planning spine or the editorial ladder.

## Layout

```
open-storycraft/
├── burstiness-check/
│   └── SKILL.md
├── coldread/
│   └── SKILL.md
├── fantasy-settlement/
│   └── SKILL.md
├── fiction-aiism-editor/
│   └── SKILL.md
├── fiction-audience/
│   └── SKILL.md
├── fiction-characters/
│   └── SKILL.md
├── fiction-dev-editor/
│   └── SKILL.md
├── fiction-dialogue/
│   └── SKILL.md
├── fiction-factions/
│   └── SKILL.md
├── fiction-full-editor/
│   └── SKILL.md
├── fiction-genre/
│   └── SKILL.md
├── fiction-line-editor/
│   └── SKILL.md
├── fiction-outline/
│   └── SKILL.md
├── fiction-places/
│   └── SKILL.md
├── fiction-polities/
│   └── SKILL.md
├── fiction-prose-editor/
│   └── SKILL.md
├── fiction-psych/
│   └── SKILL.md
├── fiction-religions/
│   └── SKILL.md
├── fiction-reviewchapter/
│   └── SKILL.md
├── fiction-scenes/
│   └── SKILL.md
├── fiction-story-sparks/
│   └── SKILL.md
├── fiction-style/
│   └── SKILL.md
├── fiction-synopsis/
│   └── SKILL.md
├── fiction-theme/
│   └── SKILL.md
├── fiction-voiceprompt/
│   └── SKILL.md
├── fiction-world/
│   └── SKILL.md
├── fiction-writechapter/
│   └── SKILL.md
├── fragment-hunter/
│   └── SKILL.md
├── kill-chapter/
│   └── SKILL.md
├── kill-crutch/
│   └── SKILL.md
├── kill-flat/
│   └── SKILL.md
├── kill-opinion-personification/
│   └── SKILL.md
├── kill-person-who/
│   └── SKILL.md
├── kill-simile/
│   └── SKILL.md
├── kill-soft/
│   └── SKILL.md
├── kill-the-whole/
│   └── SKILL.md
├── levelup/
│   └── SKILL.md
├── name-generator/
│   └── SKILL.md
├── nominalization-hunt/
│   └── SKILL.md
├── pangram/
│   └── SKILL.md
├── skill-builder/
│   └── SKILL.md
├── town-generator/
│   └── SKILL.md
```

Companion files live beside each `SKILL.md` in `references/`, `assets/`, `scripts/`, or `data/` as declared by that skill.

## Skills

| Skill | Files | Role |
|---|---:|---|
| `burstiness-check` | 2 | Statistical flatness diagnostic |
| `coldread` | 1 | Cold-reader pass |
| `fantasy-settlement` | 2 | Settlement generator |
| `fiction-aiism-editor` | 2 | AI-tell phrasing editor |
| `fiction-audience` | 9 | Target reader profile |
| `fiction-characters` | 6 | Character sheets |
| `fiction-dev-editor` | 2 | Developmental edit |
| `fiction-dialogue` | 3 | Dialogue craft gates |
| `fiction-factions` | 2 | Faction profiles |
| `fiction-full-editor` | 1 | Full editorial pass |
| `fiction-genre` | 93 | Genre / subgenre / trope profile |
| `fiction-line-editor` | 2 | Line edit |
| `fiction-outline` | 8 | Outline |
| `fiction-places` | 2 | Place profiles |
| `fiction-polities` | 2 | Polity profiles |
| `fiction-prose-editor` | 2 | Prose edit |
| `fiction-psych` | 2 | Psychological profile pass |
| `fiction-religions` | 2 | Religion profiles |
| `fiction-reviewchapter` | 3 | Chapter review orchestrator |
| `fiction-scenes` | 8 | Scene list / scene cards |
| `fiction-story-sparks` | 1 | Card-style story sparks |
| `fiction-style` | 15 | Style bible |
| `fiction-synopsis` | 6 | Synopsis |
| `fiction-theme` | 6 | Theme commitments |
| `fiction-voiceprompt` | 1 | Voice prompt for the project |
| `fiction-world` | 8 | World bible |
| `fiction-writechapter` | 6 | Chapter draft |
| `fragment-hunter` | 2 | Sentence fragment hunter |
| `kill-chapter` | 5 | Four crutch-kill passes in sequence |
| `kill-crutch` | 1 | Generic filler-word killer |
| `kill-flat` | 1 | Figurative "flat" killer |
| `kill-opinion-personification` | 1 | Opinion-personification killer |
| `kill-person-who` | 1 | "quality of a person who" killer |
| `kill-simile` | 1 | Explain-it simile killer |
| `kill-soft` | 1 | Figurative "soft" killer |
| `kill-the-whole` | 1 | "the whole of X" killer |
| `levelup` | 2 | Filter-word scanner |
| `name-generator` | 29 | Culture-aware name generator |
| `nominalization-hunt` | 1 | Nominalization hunter |
| `pangram` | 1 | Pangram-oriented AI-detection review |
| `skill-builder` | 2 | Build new portable skills |
| `town-generator` | 16 | Town name / place generator |

## How to use with Grok

A copy of each skill folder is also installed under `/home/workdir/.grok/skills/` so Grok can discover them as user skills.

To use them elsewhere, copy any skill folder next to your other Agent Skills so the host can see `SKILL.md`.

