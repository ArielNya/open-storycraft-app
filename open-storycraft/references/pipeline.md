# Open Storycraft — skill catalog and routing

Read this when choosing a child skill. Paths are under `/home/workdir/.grok/skills/<name>/`.

## Planning spine (run in order for a new book)

| Order | Skill | Writes | Requires | Next |
|---:|---|---|---|---|
| 0.1 | fiction-genre | Wiki/Style/genre.md | — | fiction-audience |
| 0.2 | fiction-audience | Wiki/Style/audience.md | genre | fiction-theme |
| 0.3 | fiction-theme | Wiki/Story/theme.md | audience | fiction-synopsis |
| 0.4 | fiction-synopsis | Wiki/Story/synopsis.md | theme (preferred) | fiction-style |
| 2 | fiction-style | Wiki/Style/style_guide.md, review_guide.md | synopsis | fiction-characters |
| 3 | fiction-characters | Wiki/Characters/*.md | synopsis | fiction-voiceprompt |
| 2.5 | fiction-voiceprompt | Wiki/Style/voice_prompt.md | synopsis, style, characters | fiction-world or outline |
| 4 | fiction-world | Wiki/Locations, Organizations, Systems, Events | synopsis, characters | fiction-outline |
| 5 | fiction-outline | Wiki/Outline/outline.md | synopsis, characters | fiction-scenes |
| 6 | fiction-scenes | scene / beat files | outline | fiction-psych |
| 7 | fiction-psych | Wiki/Psych/Chapter_NN_Psych.md | outline, characters | fiction-writechapter |
| 8 | fiction-writechapter | chapter prose | synopsis, outline | fiction-reviewchapter |
| 9 | fiction-reviewchapter | edit report | a written chapter | editorial ladder |

fiction-story-sparks sits before the spine. It writes nothing into Wiki/.

## Phrase → skill

| User says | Skill |
|---|---|
| spark, random prompt, fortune room, writing exercise | fiction-story-sparks |
| pick a genre, tropes, subgenre | fiction-genre |
| who is this for, age group, heat, rating | fiction-audience |
| what is it about, theme, motifs | fiction-theme |
| braindump, logline, synopsis, premise | fiction-synopsis |
| style guide, voice rules, review guide | fiction-style |
| protagonist voice prompt | fiction-voiceprompt |
| character sheet, cast, who are they | fiction-characters |
| how they talk, dialogue pass | fiction-dialogue |
| interior / psych pass for a chapter | fiction-psych |
| build the world, setting bible | fiction-world |
| town profile, what's in this city | fantasy-settlement |
| town names, village names | town-generator |
| character names, culture names | name-generator |
| a place, ruin, landmark, interior | fiction-places |
| kingdom, empire, government | fiction-polities |
| guild, crew, cult, corporation | fiction-factions |
| pantheon, cult, ritual, faith | fiction-religions |
| outline, chapter list, what must happen | fiction-outline |
| scene beats, scene cards | fiction-scenes |
| write chapter N, draft the chapter | fiction-writechapter |
| cold read, does this sound like her | coldread |
| developmental edit, plot problem, why is this scene here | fiction-dev-editor |
| review chapter against the guides | fiction-reviewchapter |
| AI scan, AIisms | fiction-aiism-editor |
| prose mechanics, choppy, rule of threes | fiction-prose-editor |
| line edit, tighten sentences | fiction-line-editor |
| full edit, edit everything | fiction-full-editor |
| filter words, levelup | levelup |
| fragments, audiobook choppy | fragment-hunter |
| nominalizations, abstract self-explanation | nominalization-hunt |
| kill-chapter, strip crutches | kill-chapter |
| kill similes / just / soft / flat / the whole / person who / opinions | matching kill-* |
| burstiness, Pangram flat | burstiness-check |
| Pangram check, detector risk | pangram |
| make a skill | skill-builder |

## Editorial ladder (narrowest first)

1. coldread or fiction-dev-editor
2. fiction-reviewchapter
3. fiction-aiism-editor
4. fiction-prose-editor
5. fiction-line-editor
6. fiction-full-editor (only when they want all micro checks at once)
7. levelup / fragment-hunter / nominalization-hunt as named
8. kill-chapter or a single kill-*
9. burstiness-check (report only) → pangram

burstiness-check never rewrites. After it, send phrasing issues to fiction-aiism-editor and rhythm issues to fiction-prose-editor.

## Optional overlays (off by default)

These are user skills, not Storycraft. Activate only on request.

- ao3-writer / ao3-narrative-voice / ao3-dialogue-engine / ao3-scene-review
- anti-slop-editor
