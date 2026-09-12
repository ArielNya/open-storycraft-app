# Voice Inheritance

How character voice from the character document integrates with the style guide. **Key principle**: Don't duplicate personality information. Reference it.

---

## The Three-Layer Model

Narrative voice has three layers that combine:

### Layer 1: Baseline Narrative Voice (Style Guide)
The book's default prose style regardless of POV character.
- Tense (past/present)
- Register (formal/casual)
- Sentence rhythm patterns
- Forbidden words/phrases
- Genre conventions

**Defined in**: Style guide

### Layer 2: Character Individual Voice (Character Document)
Unique traits of the POV character that color perception.
- What they notice (based on personality)
- How they interpret situations
- Internal monologue patterns
- Vocabulary specific to them
- Emotional reactions

**Defined in**: `characters.csv` or character profiles

### Layer 3: Combined Voice (The Prose)
Where Layers 1 and 2 merge on the page.
- Baseline voice FLEXES around character voice
- Same scene reads differently through different POV characters
- Style rules apply, but filtered through character personality

---

## What Goes Where

### In the Style Guide (Layer 1)
- POV type (first person, third limited, etc.)
- Tense
- Pacing patterns
- Forbidden phrases and words
- Sentence structure variation rules
- Genre-specific conventions
- Show-don't-tell requirements

### In the Character Document (Layer 2)
- Character personality traits
- What they value, fear, want
- How they speak (dialogue style)
- What they notice based on background
- Their worldview and biases
- Age and experience level
- Coping mechanisms

### NOT Duplicated
The style guide should NOT include:
- Individual character personality descriptions
- Character-specific vocabulary
- Backstory that shapes perception
- Relationship dynamics

**Reference these**: "See `characters.csv` for individual character voice patterns."

---

## How to Reference Character Voice

In the generated style guide, include a section like:

```markdown
## Character Voice Integration

The narrative voice flexes around the POV character's personality.

**For POV character voice details, see `characters.csv`:**
- Personality column: What they're like
- Dialogue Style column: How they speak
- Backstory column: What shapes their worldview

DO NOT duplicate personality information here.
```

---

## Same Scene, Different POV

The power of this model: identical scenes read differently based on POV character.

### The Room

**Through a suspicious character:**
> The bar had two exits - front door, back through the kitchen. Three guys at the pool table, one watching the door. The bartender kept glancing at something under the counter. Great.

**Through a romantic character:**
> The bar was dimly lit, intimate. A couple in the corner booth leaned toward each other, oblivious to the room. The bartender smiled at regulars like old friends. Warm.

**Through a practical character:**
> The bar needed work - loose floorboard by the pool table, one light flickering. Someone had fixed the jukebox with duct tape. It worked, barely.

Same room. Three perceptions. The style guide's baseline (sentence structure, forbidden words) stays constant. The character layer changes everything else.

---

## Implementation

When writing prose:

1. **Check style guide** for baseline rules (what to avoid, how to structure)
2. **Check character document** for this POV character's personality
3. **Filter descriptions** through that character's worldview
4. **Verify** observations match what this character would notice

### Example Workflow

**Baseline rule** (from style guide): No em dashes in narrative. Vary sentence structure.

**Character trait** (from characters.csv): Rowan - protective, sarcastic, 20 years old, notices tactical details first.

**Combined prose**:
> The building was freezing. Rowan wondered if the IFC's budget didn't cover heat, or if Director Vale was just used to Boston winters. He counted exits while she talked. Two doors, one window. The window was painted shut.

Baseline voice (no em dashes, varied sentences) + Character voice (notices cold, sarcastic thought, counts exits).

---

## When Characters Speak

Dialogue is fully character-specific. The character document defines:
- Vocabulary level
- Speech patterns
- Verbal tics
- What they avoid saying
- How they deflect

The style guide provides:
- Dialogue tag rules (use "said")
- Formatting conventions
- Action beat guidance

But HOW each character speaks comes from their character profile.

---

## Key Reminders

1. **One source of truth**: Character personality lives in the character document
2. **No duplication**: Style guide references, doesn't repeat
3. **Flexible baseline**: Style rules apply, but bend around character voice
4. **Different POVs, different reads**: Same scene changes with POV character

---

## Cross-References

- See `character-lens-filtering.md` for the full filtering framework
- See `age-appropriate-voice.md` for age-specific patterns
- See `pov-integrity.md` for POV boundaries
