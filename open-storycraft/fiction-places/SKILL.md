---
name: fiction-places
description: "Create or deepen story-ready places such as wilderness sites, ruins, landmarks, interiors, artificial habitats, uncanny zones, and other locations where scenes can happen. Use when the user asks for a location, destination, landmark, dungeon-like site, environmental obstacle, or place generator. Do not use for full settlement profiles or broad worldbuilding."
metadata:
  author: Fiction Toolkit
  version: "1.0.0"
  category: fiction-writing
  output-format: markdown
---

# Fiction Places

Create a place that changes what characters can do. This skill focuses on individual scene-bearing sites, not full towns (`fantasy-settlement`), political territories (`fiction-polities`), or a complete setting (`fiction-world`).

## Load context and anchor the site

In project mode, read applicable `AGENTS.md` and `CLAUDE.md`, then locate the current premise or synopsis, outline, characters, maps, locations, settlements, systems, and relevant scene files. Established geography and travel constraints are canon.

Determine:

- why a character goes there and what they need;
- the site's environment or engineered conditions;
- the nearest established place and credible travel relationship;
- what controls access, survival, observation, movement, and departure;
- what the story must reveal, conceal, threaten, or offer there.

If the user provides only a type, create one place. If asked for options, create 3–5 compact candidates before developing the selected one. Do not scatter random landmarks across a project.

## Generate proper names

Never freehand an invented proper name.

- Use `town-generator` for settlement-derived, regional, or geographic names when its culture lists fit.
- Use `name-generator` for names derived from people, peoples, founders, saints, species, languages, or cultural roots.
- A descriptive site name may combine a generated root with a setting-appropriate type, translation, epithet, or local nickname.

Preserve existing names. Use one naming register consistent with nearby places unless history explains a linguistic layer.

## Build the place

Read [references/place-profile.md](references/place-profile.md) and use its field checklist. Keep geography coherent: biome, weather, water, geology, ecology, technology, magic, architecture, access, hazards, inhabitants, and resources must not contradict each other without an explicit cause.

Every developed place needs:

- a dominant sensory identity;
- a spatial arrangement a writer can block action through;
- an access condition and credible relationship to nearby locations;
- a hazard or constraint that changes choices;
- inhabitants, users, custodians, or evidence of absence;
- a bounty, resource, truth, refuge, route, or opportunity worth the risk;
- a history or legend that affects present behavior;
- `prevents`, `forces`, and `enables` sentences tied to the protagonist.

Avoid postcard description, biome grab bags, hazards unrelated to the reward, empty ruins containing convenient answers, and legendary history with no present consequence. A site's mystery can remain unresolved; distinguish fact, local belief, and author-only truth.

## Deliver the result

In project mode, save under the established convention; otherwise use `Wiki/Locations/<Place_Name>.md` with `type: Location` and a specific `subtype`. New inventions use `status: Proposed` and `ai_invented: true` until accepted. Use conservative YAML for scalar identity and geography fields and `## snake_case` sections for the longer fields from the reference.

If a place is nested inside an existing settlement or polity, link it instead of duplicating the parent profile. Do not alter maps, routes, or canon coordinates unless requested.

In standalone mode, present a compact writer-facing profile and do not create files unless requested.

## Quality check

Verify that:

- every invented proper name came from `name-generator` or `town-generator` output;
- the site fits its biome, technology, magic, culture, and nearby geography;
- characters can enter, move through, and leave it in physically understandable ways;
- sensory details are specific and useful for prose;
- hazard, denizens, bounty, and history interact rather than feeling randomly paired;
- `prevents`, `forces`, and `enables` change scene options;
- the place earns its existence in the current story.

