---
name: fiction-religions
description: "Create or deepen religions, cults, philosophies, ancestor traditions, civic faiths, machine creeds, and other belief systems for fiction. Use when the user asks to design a religion, pantheon, deity, ritual tradition, religious conflict, or lived faith within a setting."
metadata:
  author: Fiction Toolkit
  version: "1.0.0"
  category: fiction-writing
  output-format: markdown
---

# Fiction Religions

Build a lived belief system that shapes conduct, institutions, conflict, and meaning. Match the setting; do not assume gods are real, religion is organized, faith is medieval, or disbelief is socially neutral.

## Load the setting

In a project, read applicable `AGENTS.md` and `CLAUDE.md`, then locate the current premise or synopsis, characters, outline, locations, factions, systems, and any existing religious material. Established details are canon. In standalone work, use the setting supplied in conversation and ask only when a missing choice would substantially change the result.

Identify what people in this setting fear, depend on, cannot explain, remember, owe, and hope survives them. Determine whether supernatural claims are verified, disputed, symbolic, technologically mediated, or unknowable.

## Name through the generator

Use the sibling `name-generator` skill for every invented proper name: faith, deity, prophet, saint, founder, clergy member, sacred place, named text, holiday, schism, or sect. Read its culture map, generate a culture-appropriate pool, and select or adapt roots from that pool. Never freehand a proper name. Preserve user-supplied and existing canon names.

Use one coherent naming tradition unless history establishes conquest, syncretism, translation, diaspora, colonial layering, or deliberate reform. Explain mixed naming only when it matters on the page.

## Design the faith

Read [references/religion-profile.md](references/religion-profile.md) and use its field checklist. Start with the social and emotional function, then select a form that fits it: one god, dual powers, pantheon, spirits, ancestors, sacred law, philosophical discipline, civic cult, distributed intelligence, cosmic principle, or a setting-specific structure.

Every generated faith needs:

- a central claim about reality and why adherents find it credible;
- practices ordinary believers actually perform;
- obligations, taboos, and costs that affect choices;
- an institution or transmission method, even if decentralized;
- disagreement among sincere believers;
- a present schism, reform, persecution, revelation, succession, legitimacy crisis, or doctrinal pressure;
- a concrete relationship to the protagonist and other powers.

Do not equate doctrine with behavior. Distinguish official teaching, common practice, elite practice, regional variation, private doubt, and hostile caricature when relevant. Avoid generic “church controls everything,” uniformly fanatical adherents, decorative pantheons, and rituals that never constrain a scene.

## Deliver the result

If the user requests a number, create that many. Otherwise create one faith for a singular request and 2–4 only when asked for a religious landscape. For multiple faiths, define a specific contested practice, place, constituency, truth claim, or institution between each important pair.

In project mode, save the belief system under the project's established convention. Otherwise use `Wiki/Systems/<Faith_Name>.md` with `type: System` and `subtype: Religion`. If a separate clergy or temple hierarchy acts independently, create or update a linked `Wiki/Organizations/` entry only when the story needs it. Do not multiply files merely to mirror the profile headings.

New inventions use `status: Proposed` until the user accepts them and `ai_invented: true`; omit `ai_invented` for existing user-authored canon. Use conservative YAML with scalar identity fields, quoted colon-bearing strings, and block lists. Put longer material in `## snake_case` sections from the reference.

In standalone mode, present a compact profile and do not create files unless requested.

## Quality check

Verify that:

- every proper name is traceable to a `name-generator` pool;
- belief structure, clergy, rituals, texts, sacred places, and material resources agree with the setting;
- ordinary believers have reasons beyond fear or stupidity;
- at least two sincere internal positions disagree;
- the taboo and ritual can change what happens in a scene;
- the active religious crisis is happening now;
- the protagonist can comply, resist, reinterpret, exploit, or be changed by the faith;
- invented theology does not silently settle mysteries the story needs to preserve.

