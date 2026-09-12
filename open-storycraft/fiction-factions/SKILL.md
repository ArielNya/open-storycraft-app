---
name: fiction-factions
description: "Create or deepen story factions, organizations, institutions, political blocs, guilds, crews, cults, corporations, and rival networks within any established or new setting. Use when the user asks to create factions, design an organization, build political groups, map faction conflict, or make a setting's power structure story-ready."
metadata:
  author: Fiction Toolkit
  version: "1.0.0"
  category: fiction-writing
  output-format: markdown
---

# Fiction Factions

Create factions that exert pressure on characters and can generate scenes. Match the setting's genre, technology, magic, culture, scale, and naming register; do not default to medieval guilds or modern corporations.

## Choose the working mode

- **Project mode:** Use when a fiction project or setting files are available. Read its applicable `AGENTS.md` and `CLAUDE.md`, then locate the current synopsis/premise, outline, character files, and relevant world files. Treat established material as canon.
- **Standalone mode:** Use when the user provides a setting in conversation. Work from that description. Ask one concise question only if the missing answer would substantially change the factions; otherwise state a light assumption and proceed.
- **Revision mode:** If faction files already exist, preserve their established facts and revise only the requested fields or contradictions. Do not regenerate the setting around them.

If the user requests a specific number, create that many. Otherwise create one faction for a singular request and 3–5 factions for a faction landscape, scaled to the story rather than to the size of the world.

## Build from the setting

Before inventing, identify:

- the territory, population, economy, faith, law, technology or magic the faction can actually draw on;
- the protagonist's desire and the pressure the story needs;
- existing institutions, named people, settlements, resources, conflicts, and reader promises;
- the setting's naming logic and linguistic register.

Do not import facts from Writer's Tavern or another book. Use its generator logic only as a design principle: **scale determines reach, membership, leadership depth, holdings, forces, and subordinate groups.** A neighborhood crew should not possess national intelligence networks; an interstellar power should not feel like twelve people in a back room.

When any proper name must be invented, use the local `name-generator` skill; do not invent it freehand. Read its culture map, choose the list that fits the setting, generate a pool, and select from that pool.

- **People:** Every newly named leader, officer, founder, agent, or other faction member must come from the generated personal-name pool. Preserve user-supplied and existing canon names unchanged.
- **Factions:** Use the generated pool as the linguistic seed for faction names. A faction may take a generated founder, dynasty, people, or place name directly, or adapt a generated root into the setting's institutional pattern (for example a house, ministry, crew, union, order, corporation, collective, army, or hostile nickname). The final organization name need not look like a person's name, but its invented proper-name element must trace to the generated pool.

Generate enough candidates to make deliberate selections rather than accepting the first result. Use one coherent culture pool per faction roster unless the setting establishes a mixed, colonial, diasporic, adopted, or deliberately cosmopolitan naming pattern. Organization names may arise from founders, places, ideals, functions, symbols, legal charters, slang, acronyms, or hostile nicknames; vary the pattern across a landscape.

## Design the faction

Read [references/faction-profile.md](references/faction-profile.md) for the field checklist and scale guidance. Generate only details that the story can reveal, contest, or use.

Every faction needs:

1. **A legitimate purpose or grievance.** Some members should reasonably believe the faction improves the world.
2. **A concrete objective.** Define what it wants now and what observable success looks like.
3. **A power base.** Name the resources, access, legitimacy, information, territory, labor, wealth, magic, or technology it controls.
4. **Methods and limits.** State what it does, what it refuses to do, and what pressure might make it cross that line.
5. **Internal disagreement.** Give at least two recognizable tendencies, offices, generations, classes, or personalities that disagree about means or ends.
6. **A live instability.** Use a current crisis, succession problem, exposure risk, schism, shortage, rival move, or impossible obligation—not only ancient history.
7. **Story leverage.** State what it prevents, forces, and enables in scenes involving the protagonist.

Avoid pure-evil monoliths, perfect allies, interchangeable members, encyclopedic history, and secrets that never affect a decision. A secret may remain hidden from the reader, but it must change present behavior.

## Connect multiple factions

For a landscape, build relationships after drafting the individual factions:

- Every faction must want something another faction controls.
- Give each relationship a specific object of conflict: a route, office, law, relic, labor pool, market, secret, territory, constituency, technology, or person.
- Include at least one asymmetric relationship: dependence disguised as dominance, public alliance/private sabotage, ideological hostility/economic reliance, or patron/client resentment.
- Rival pairs should compete over the same scarce thing or constituency; a shared emblem or naming noun is optional, not a universal convention.
- Check how the protagonist can shift the balance and what each side would demand in return.

Do not create a faction merely to fill a moral or aesthetic slot. Cut any group that cannot alter a choice, close a route, offer a costly resource, or change the consequences of a scene.

## Deliver the result

### Project mode

Save one Markdown file per faction under the project's existing organization/faction folder. Prefer `Wiki/Organizations/<Faction_Name>.md` when the project uses the Fiction Toolkit layout; preserve another established convention if present. Replace spaces with underscores in filenames.

Use conservative YAML frontmatter for scalar identity fields:

```yaml
---
name: The Example Compact
tag: "One-line writer-facing statement of what it is and why it matters"
type: Organization
subtype: Trade compact
scale: Regional
status: Proposed
ai_invented: true
---
```

Use `status: Proposed` for newly invented material unless the user explicitly approves it as canon. Omit `ai_invented` when the faction already existed in user-authored canon. Quote YAML strings containing colons. Use block lists, never inline bracket lists.

Put the longer fields in `## snake_case` sections following the reference checklist. Relationship entries use one `### <Faction or Character>` subsection per named relationship so they remain searchable. If a target file already exists, update it deliberately; do not overwrite unrelated canon.

After writing, report the files created or changed and summarize the power structure in a few lines. Clearly separate established canon from proposed invention.

### Standalone mode

Present each faction as a compact, writer-usable profile using the same field logic. End with a short relationship map when more than one faction was created. Do not create files unless the user asked for an artifact or supplied a project location.

## Quality check

Before finishing, verify:

- scale, member count, reach, forces, holdings, and leadership agree;
- names and institutions belong to this setting;
- every invented person and faction proper name is traceable to a `name-generator` pool;
- every faction contains sincere believers and an internal fault line;
- the objective can succeed or fail visibly;
- the crisis is active at story start;
- relationships name the contested resource and are not all symmetric;
- `prevents`, `forces`, and `enables` describe scene consequences, not lore abstractions;
- no invented detail silently contradicts canon or changes the story's approved premise.

