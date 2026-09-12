---
name: pangram
description: Evaluate fiction for patterns associated with Pangram AI detection, including predictability, statistical uniformity, lexical diversity, burstiness, and choppy construction. Use when the user asks for a Pangram check, Pangram risk assessment, or AI-detection pattern review.
---

# Pangram AI Detection Evaluation

Analyze the chapter text supplied by the user for patterns that Pangram's AI detection algorithm flags. Pangram specifically looks for: low perplexity (predictable word choices), uniform sentence structure, and statistical regularities in text distribution.

## PASS 1: PERPLEXITY BOOSTERS (Most Critical for Pangram)

Pangram flags text that's too "predictable." Check for these issues:

### A. Word Choice Predictability

Flag any sentence where the next word feels "obvious" or "expected." Replace common collocations with unexpected but natural alternatives:

- ❌ "broke the silence" → ✅ "cut through the quiet"
- ❌ "heart pounded" → ✅ "pulse hammered in his ears"
- ❌ "took a deep breath" → ✅ "filled his lungs"
- ❌ "nodded slowly" → ✅ "dipped his chin"
- ❌ "eyes widened" → ✅ "his stare went wide"
- ❌ "let out a breath" → ✅ "exhaled through his teeth"

### B. Sentence Opening Variety

Count the first word of every sentence. Flag if:

- More than 15% start with "The"
- More than 10% start with "He/She"
- More than 3 sentences in a row start with the same word
- Pronouns dominate sentence openings

### C. Transition Predictability

Flag and replace these AI-favored transitions:

- "However," "Moreover," "Furthermore," "Additionally"
- "In fact," "Indeed," "Certainly"
- "As a result," "Consequently," "Therefore"
- "With that," "At that moment," "In that instant"

Replace with natural narrative flow or character-driven transitions.

## PASS 2: STATISTICAL UNIFORMITY (Pangram's Core Detection)

### A. Sentence Length Variance

- Calculate average sentence length
- Flag if variance is too low (sentences too similar in length)
- Pangram expects human writing to have HIGH variance
- Mix: 5-word punches with 25-word flowing sentences

### B. Paragraph Structure

- Flag paragraphs that are all similar lengths
- Flag if every paragraph follows the same internal structure
- Humans write messy—some paragraphs are 1 sentence, others are 8

### C. Punctuation Distribution

- Flag overuse of commas in predictable patterns
- Flag lack of semicolons, colons, or dashes (humans use variety)
- Flag if every sentence ends with a period (add questions, exclamations naturally)

## PASS 3: LEXICAL DIVERSITY (Pangram Measures This)

### A. Vocabulary Repetition

- Flag any non-common word used more than twice per 500 words
- Flag repeated descriptive phrases
- Flag if the same verb appears more than 3 times in a scene

### B. Synonym Rotation

- Don't just rotate synonyms mechanically (Pangram catches this too)
- Instead, restructure sentences to avoid needing the same concept repeatedly

### C. Register Mixing

- Pangram expects humans to mix formal/informal naturally
- Flag if entire passages maintain identical register
- Add character voice intrusions, casual asides, or tonal shifts

## PASS 4: STYLE GUIDE VIOLATIONS (Cross-Reference)

Apply these rules from the style guide that also help with Pangram:

### Forbidden Phrases (statistically common in AI text)

- "practiced precision," "with practiced ease"
- "a mix/mixture/combination of [X] and [Y]"
- "couldn't help but [verb]"
- "found himself [verbing]"
- "something akin to," "a sense of"

### Forbidden Patterns

- **Gerund chains**: "He ran, dodging bullets, weaving through debris..."
- **Emotion-body formulas**: "Fear gripped his chest," "Dread coiled in his stomach"
- **Paired adjectives**: "dark and foreboding," "cold and unforgiving"
- **Rule of threes**: Lists of exactly 3 items (use 2 or 4 instead)

### Forbidden Words

rhythm, cacophony, symphony, verdant, tapestry, testament, sentinel, cerulean, palpable, unmistakable, undeniable, visceral, primal

## PASS 5: BURSTINESS CHECK (Critical for Pangram)

Pangram specifically measures "burstiness"—the variation in complexity across a text.

### Flag if

- All sentences have similar complexity
- No sudden shifts from simple to complex
- Dialogue and narrative have the same rhythm
- Action scenes read the same as introspective scenes

### Human writing has

- Simple sentences followed by complex ones
- Short paragraphs interrupting longer ones
- Tonal shifts within scenes
- Varying density of description

## PASS 6: CHOPPY CONSTRUCTIONS CHECK

### Flag these patterns

- Sentence fragments masquerading as description (no verb)
- Colon-list constructions: "The room was chaos: papers everywhere, chairs overturned"
- Ellipsis-separated lists
- Rapid-fire action fragments overused
- Paragraphs that read like bulleted lists

### Fix by

- Adding connecting verbs
- Using logical connectors (while, as, because, but)
- Combining with shared subjects
- Showing cause and effect

## OUTPUT FORMAT

Provide your analysis in this structure:

### Section 1: Perplexity Issues

[List predictable phrases with specific rewrites]

### Section 2: Statistical Uniformity Analysis

- Sentence length variance: [Low/Medium/High]
- Paragraph structure variance: [Low/Medium/High]
- Sentence opening diversity: [Percentage breakdown of first words]

### Section 3: Lexical Diversity Issues

[List repeated words/phrases with frequency counts]

### Section 4: Style Guide Violations

[List specific violations with line references and rewrites]

### Section 5: Burstiness Score

[Low/Medium/High] with specific recommendations for adding variance

### Section 6: Choppy Constructions

[List any choppy passages with rewrites]

## OVERALL PANGRAM RISK ASSESSMENT

**Risk Level:** [Low/Medium/High]

**Top 3 Priority Fixes:**

1. [Most critical issue]
2. [Second priority]
3. [Third priority]

**Quick Wins:** [Easy changes that will have immediate impact]

## KEY PANGRAM-SPECIFIC REMINDERS

1. **Inject imperfection intentionally** - Pangram expects human messiness
2. **Vary your rhythm dramatically** - Don't let any two paragraphs feel the same
3. **Use unexpected word choices** - Not wrong, just less predictable
4. **Break patterns** - If you've done something twice, do something different the third time
5. **Add character voice intrusions** - Internal thoughts, asides, and reactions break statistical patterns
