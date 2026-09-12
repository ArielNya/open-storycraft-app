# Character CSV Schema (21 Columns)

## Header Row

```csv
Name,Role,Pronouns,Groups,Other Names,Personality,Backstory,Physical Description,Dialogue Style,Age,Items,Abilities,Race,Uniform/Typical Attire,Skills or Resources,Relationship to Protagonist,Role in Story,Chemistry with Protagonist,Romantic Obstacles,Shared History,Relationship Dynamics
```

---

## Column Guide

| # | Column | What to Write |
|---|--------|---------------|
| 1 | **Name** | Full name |
| 2 | **Role** | protagonist, antagonist, loveInterest, ally, mentor, supporting |
| 3 | **Pronouns** | she/her, he/him, they/them |
| 4 | **Groups** | Factions, organizations, families |
| 5 | **Other Names** | Nicknames, titles, aliases |
| 6 | **Personality** | 2-3 paragraphs: traits, flaws, how they present vs who they are |
| 7 | **Backstory** | History, secrets, formative events |
| 8 | **Physical Description** | Appearance, movement, distinguishing features |
| 9 | **Dialogue Style** | How they speak - cadence, vocabulary, verbal tics, what they avoid |
| 10 | **Age** | Specific or range |
| 11 | **Items** | Signature possessions, meaningful objects |
| 12 | **Abilities** | Skills, powers, talents AND weaknesses |
| 13 | **Race** | Species if relevant to world |
| 14 | **Uniform/Typical Attire** | What they wear and why |
| 15 | **Skills or Resources** | What they bring - knowledge, connections, wealth |
| 16 | **Relationship to Protagonist** | How they connect, what tension exists |
| 17 | **Role in Story** | Narrative function, when they appear, what they do |
| 18 | **Chemistry with Protagonist** | The dynamic - tension, comfort, attraction, conflict |
| 19 | **Romantic Obstacles** | What prevents easy romance (if applicable, else "N/A") |
| 20 | **Shared History** | Past connections with protagonist or other characters |
| 21 | **Relationship Dynamics** | Power balance, communication style, how they interact |

---

## Formatting Rules

### Wrap Fields with Commas
Any field containing commas MUST be wrapped in double quotes:
```csv
"Fierce, political, slow to trust"
```

### Escape Internal Quotes
Double any quotation marks inside a field:
```csv
"She calls him ""sunshine"" mockingly"
```

### One Character Per Row
Each character occupies exactly one row, all 21 columns filled.

### Multi-Paragraph Content
For Personality and Backstory, separate paragraphs with a space:
```csv
"First paragraph here. Second paragraph here."
```

---

## Example Row

```csv
"Elara Vance","deuteragonist","she/her","Royal bloodline","The Prisoner","Fierce, political, slow to trust. Two years imprisoned for knowing the truth hardened her into something sharp. She sees everyone as either tool or threat, calculating value before offering anything resembling warmth.","King's niece who discovered proof of the coup. Imprisoned under guise of 'madness' to silence her. Lost her mother to the same conspiracy.","Tall and lean with sharp features. Dark hair kept short (prison necessity turned habit). Pale from years without sunlight. Moves with coiled tension.","Clipped, precise sentences when guarded. Formal court language deployed as weapon. Opens up in fragments when trusting.","24","A smuggled knife, her mother's ring on a cord","Political intelligence, network of allies, reads people; weakness: difficulty trusting","Human","Prison shift initially, later practical traveling clothes","Court politics knowledge, noble house connections, legitimacy as heir","Needs his griffin and bloodline; initially sees him as tool; gradually recognizes partner","Rescued at midpoint; drives political plot; provides proof of conspiracy","Friction to respect to trust. Impressed by his choices, frustrated by his naivety","She's been alone with the truth for years; trusting is terrifying","None before story","Push and pull - she leads politics, he leads action"
```

---

## Output Template: characters.md

After generating the CSV, also create a readable markdown summary:

```markdown
# Characters

## The Cast

### [Name] - [Role]
[One paragraph summary: who they are, what they want, their key relationship to the story]

### [Name] - [Role]
[etc.]

---

## Relationship Web

### Protagonist Relationships
- → [Character]: [Nature of relationship + tension point]
- → [Character]: [etc.]

### Key Conflicts
- [Character A] vs [Character B]: [Source of conflict]

### Secrets & Information
- [Character] knows: [What others don't]
- [Character] is hiding: [Secret]

---

## What I Invented

**Characters from your synopsis:**
- [List characters that were named or clearly described]

**Characters I created to serve the story:**
- **[Name]**: [Why this role was needed] → Keep / Change?
- **[Name]**: [Why this role was needed] → Keep / Change?

Let me know what to change.
```
