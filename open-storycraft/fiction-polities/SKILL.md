---
name: fiction-polities
description: "Create or deepen fictional polities and their territorial divisions: city-states, kingdoms, nations, empires, colonies, federations, planetary governments, corporate sovereignties, and stranger political orders. Use when the user asks for a government, realm, nation, province, political geography, or coherent state-level setting profile."
metadata:
  author: Fiction Toolkit
  version: "1.0.0"
  category: fiction-writing
  output-format: markdown
---

# Fiction Polities

Build a polity as a system of people, territory, logistics, legitimacy, and unequal obligations—not a crown plus a list of laws. Adapt to the setting: a polity may be territorial, diasporic, nomadic, orbital, networked, corporate, theocratic, ecological, artificial, or otherwise non-national.

## Load canon and choose scope

In project mode, read applicable `AGENTS.md` and `CLAUDE.md`, then current premise or synopsis, outline, characters, locations, systems, religions, factions, and existing maps or population evidence. Preserve canon and the approved reader promise. In standalone mode, use the supplied setting and state any light assumption.

Determine whether the user wants:

- **One polity profile:** default for a singular request.
- **One territorial division:** province, state, district, habitat ring, march, colony, or equivalent.
- **A polity package:** central government plus selected divisions, capital, faith or ideology, factions, and neighbors. Create this only when requested or clearly needed.
- **A political landscape:** 2–5 polities with specific dependencies and disputes, not a continent-wide encyclopedia.

Read [references/polity-profile.md](references/polity-profile.md) for the profile and consistency rules.

## Generate names through the existing tools

Never freehand an invented proper name.

- Use `name-generator` for rulers, governors, founders, dynasties, peoples, cultures, eponymic realms, and other personal or root names.
- Use `town-generator` for capitals, settlements, and territorial place names when its cultural lists fit.
- Use the selected generated pools to form polity and division names in the setting's own register. Preserve canon names unchanged.

One polity should normally share a coherent naming tradition across rulers and territory. Mixed naming must reflect history such as conquest, migration, federation, translation, caste, species, colonial rule, or deliberate reform.

## Compose the polity

Build from the top down only far enough to establish constraints, then verify from the bottom up:

1. Government form, source of legitimacy, succession or continuity, actual decision path, and limits on authority.
2. Capital or mobile center, administrative divisions, settlement network, transport, communication, and resource base.
3. Population expressed as a plausible rounded estimate with visible components.
4. Economy, law, military or enforcement, faith or civic ideology, factions, and neighboring powers.
5. Present tension: the concrete crisis that makes the polity act now.

Use `fiction-religions` for a faith requiring a full profile, `fiction-factions` for story-important internal organizations, `fiction-places` for important sites, and `fantasy-settlement` only where its output fits. Do not invoke every related skill automatically; expand only components that matter to the story.

Treat provinces or divisions as real administrative and geographic units with their own leader, material base, standing with the center, settlements, and tension. The capital belongs to a real division unless the setting explicitly creates a separate district.

## Weave, do not merely nest

Cross-link generated parts:

- internal factions have headquarters in actual places and compete over named institutions or resources;
- laws alter conduct in settlements rather than existing only as trivia;
- state faith or civic ideology changes offices, calendars, buildings, rights, or enforcement;
- military capacity follows population, economy, transport, and political control;
- neighboring polities have a specific relationship involving trade, water, routes, labor, defense, migration, debt, ideology, data, magic, or disputed territory.

Include dependence as well as hostility. At least one relationship in a landscape should be asymmetric or publicly misrepresented.

## Deliver the result

In project mode, save the primary polity under the established convention; otherwise use `Wiki/Organizations/<Polity_Name>.md` with `type: Organization` and `subtype: Polity`. Keep divisions nested in the main file by default. Create separate `Wiki/Locations/` files only for divisions or settlements that need their own scene-facing profiles or when the user requests the full package.

New inventions use `status: Proposed` and `ai_invented: true` until accepted. Use conservative YAML for scalar identity fields and `## snake_case` sections for the reference fields. Do not overwrite unrelated canon or silently redraw borders.

In standalone mode, present a compact profile followed by a readable territory and relationship summary. Do not create files unless requested.

## Quality check

Verify that:

- every invented proper name came from `name-generator` or `town-generator` output;
- population totals equal the displayed components when exact components are shown;
- named cities and towns are not accidentally dwarfed by implied unnamed settlements;
- government titles, laws, military, economy, transport, and technology or magic agree;
- nominal authority is distinguished from power that can actually be exercised;
- divisions have different pressures without feeling like unrelated biomes;
- factions, faith, laws, and territory are cross-linked;
- the current crisis creates decisions for the protagonist;
- no canon boundary, population, allegiance, or succession fact was changed without marking it as a proposal.

