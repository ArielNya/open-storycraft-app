# Storybible Template

Fill this in, keep the `---` blocks, delete what the story does not need.
Every document you keep becomes one file. See
[storybible-format.md](../references/storybible-format.md) for the field-level
contract of each file.

```markdown
# <Working Title> — Storybible

One paragraph for a reader of this file: what the book is, and what is still
undecided.

---

slot: genre
working_title: "<Working Title>"
genre: <Genre>
subgenre: "<Subgenre>"
flavor: <Underscored_Flavor>
tropes:
  - <trope_one>
  - <trope_two>
---

# Genre

## tone_notes

<One paragraph. What the surface experience feels like, and what runs
underneath it.>

## Trope Commitments

### <trope_one>

- **role:** structural_engine
- **reader_expectation:** <what the trope promises the reader>
- **planning_obligation:** <what the synopsis and outline must set up and pay off>

---

slot: audience
age_group: <Adult | Young Adult | …>
typical_reader_age: "<range>"
tone_variant: "<two words>"
---

# Audience Profile

<Who reads this, what they want on the page, what they will not forgive.>

---

slot: theme
central_question: "<the question the book argues with>"
---

# Theme

## Tone

<How the theme is carried: restrained, comic, brutal, tender.>

## Motifs

### <Motif Name>

- **type:** <object | gesture | phrase | place>
- **what_it_is:** <the motif and what it means each time it returns>

---

slot: synopsis
title: "<Working Title>"
central_question: "<same question>"
reader_promise: "<what the reader is guaranteed>"
characters_in_play:
  - <Name>
locations_in_play:
  - <Place>
---

# <Working Title>: Synopsis

## Premise

<The situation that starts the book, in one paragraph.>

## Story Answer

<The answer the book finally gives to the central question.>

## Protagonists

### <Role label>

<Who carries the story, what they want, what it costs them.>

## Mechanism

### <How the plot actually works>

<The engine: what makes the next chapter happen.>

## Trope Payoff Plan

### <trope_one>

- **role:** structural_engine
- **setup:** <where it is planted>
- **development:** <how it escalates>
- **payoff:** <the moment it delivers>

## Act 1

- **shape:** <what Act 1 does>
- **ends_with:** <the turn into Act 2>

## Act 2

- **shape:** <what Act 2 does>
- **midpoint:** <the thing that cannot be undone>
- **ends_with:** <the turn into Act 3>

## Act 3

- **shape:** <what Act 3 does>
- **payoff:** <how the promise is kept>
- **ends_with:** <the final image>

## Open Threads

- <thread deliberately left for a sequel, or "none">

---

slot: style
title: "Style Guide: <Working Title>"
pov_mode: single
person: third
tense: past
pov_1: <Protagonist>
---

# Style Guide

## Style

<One paragraph, injected verbatim into every chapter prompt. Sentence habits,
what the narration notices, what it never does.>

## Banned

- <phrase or habit this book must not use>

---

slot: voice
---

# Voice Prompt: <Protagonist> Close-Third Rewrite

## Core POV Rule

<Person, tense, and how close the camera sits.>

## Who <Protagonist> Is

<The person behind the voice, in their own register.>

## Interior Voice

<How they argue with themselves.>

## What They Notice

<The three or four categories of detail they always catch.>

## Emotional Translation Rules

<What fear, anger, and affection look like in their body and their habits.>

## Language Firewall

<Words and rhythms that would break the voice.>

## Before / After Examples

- **Before:** <generic line>
- **After:** <the same line in this voice>

## Final Standard

<One sentence a draft must satisfy.>

---

slot: character
name: <Protagonist Name>
role: protagonist
age: <age>
pronouns: <pronouns>
---

# <Protagonist Name>

## personality

<How they behave when it costs something.>

## want

<What they are chasing.>

## need

<What they actually need instead.>

## flaw

<The belief that makes the want expensive.>

## voice

<How they talk: register, habits, what they never say.>

---

slot: character
name: <Antagonist Name>
role: antagonist
age: <age>
pronouns: <pronouns>
---

# <Antagonist Name>

## personality

<Their competence, and the courtesy or charm that hides the damage.>

## want

<What they are protecting.>

## flaw

<Why their method is the harm.>

## motivations

<What they believe they are doing, in their own words.>

## methods

<What they actually do.>

## relationship_to_protagonist

<The history between them, and what each one still wants from the other.>

---

slot: location
name: <Place Name>
---

# <Place Name>

<What it is, who controls it, what it does to the story, and one concrete
sensory detail a chapter can use.>

---

slot: outline
title: "<Working Title>"
chapter_count: <n>
premise: "<one line from the synopsis>"
central_question: "<same question>"
reader_promise: "<same promise>"
---

# Outline

## Chapter 01: <Chapter Title>

- **chapter_number:** 1
- **title:** <Chapter Title>
- **what_must_happen:** <the load-bearing event>
- **ends_with:** <the line the chapter must land on>

## Chapter 02: <Chapter Title>

- **chapter_number:** 2
- **title:** <Chapter Title>
- **what_must_happen:** <the load-bearing event>
- **ends_with:** <the line the chapter must land on>

---

slot: scene
chapter: 1
---

# Chapter 1: <Chapter Title>

## Chapter spine

- **Want:** <what the POV character is trying to get>
- **Pressure:** <what stands in the way, right now>
- **Decision:** <what they choose>
- **Irreversible change:** <what is now different>
- **Why this chapter cannot be cut:** <the structural reason>

## Beats

1. <beat>
2. <beat>
3. <beat>

---

slot: psych
chapter: 1
---

# Chapter 01: Psychological Interior Pass

**Chapter:** <Chapter Title>
**POV:** <name, age>
**Setting:** <where and when>
**External arc:** <what happens>
**Emotional arc:** <from what, to what>

## Start State

- **Surface affect:** <what shows>
- **Underneath:** <what it covers>
- **Body:** <what the body is doing>

## Beat-by-Beat Interior Cost

### <beat>

- **Interior cost:** <what it costs them inwardly>
- **Body expression:** <how it leaks>
- **What they won't say:** <the sentence they refuse>

## End State

<Where the chapter leaves them, and what they now know.>

## Bottom Line

<One sentence for the drafting pass.>
```
