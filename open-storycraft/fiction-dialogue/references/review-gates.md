# Dialogue Architecture Review Gates

Use these gates on the latest scene snapshot. Diagnose structure before rewriting lines.

## Gate 1: Character-source compliance

List the speakers and confirm that every current character sheet was read in full. For each major speaker, identify at least one recent canon exchange used for calibration.

Fail when a voice claim rests only on generic traits such as witty, kind, direct, intelligent, formal, or sarcastic.

Run a blind-speaker test on representative lines. Remove names and tags. If multiple speakers remain interchangeable, identify the shared cognition or sentence habit causing it. Do not solve the failure with catchphrases.

## Gate 2: Turn-taking and echo

Mark sequences where each line directly acknowledges, answers, reframes, or tops the immediately preceding line.

Fail when the exchange could be diagrammed primarily as A to B to A to B with no physical task, silence, setting pressure, third-party pressure, delayed answer, or change in conversational target.

Preserve necessary operational calls and intentional refrains. For other failures, delete acknowledgments, allow nonresponse, or change which pressure controls the next beat.

## Gate 3: Comeback scaffolding

For every punchline or sharp retort, test the preceding line on its own.

Fail when a line exists mainly to feed another speaker a comeback. Delete the setup line first, then decide whether the joke still belongs.

Flag chains in which three or more consecutive speakers are witty. Unless the scene is explicitly sustained banter among characters canonically capable of it, retain at most the line that does the most character or relationship work.

## Gate 4: Interview sequencing

Mark direct questions and whether the next beat supplies a clean answer.

Two consecutive question-answer pairs are a warning. Three are a failure unless the scene's form is deliberately an interrogation, briefing, examination, or interview.

Repair by combining questions, refusing one, answering incompletely, moving information into action, letting one answer create a practical consequence, or allowing the subject to remain unresolved.

Do not merely replace question marks with declarative prompts. The failure is cooperative information delivery, not punctuation.

## Gate 5: Distributed conversational intelligence

Check whether all speakers can:

- identify subtext immediately
- formulate concise thematic observations
- produce dry humor on demand
- explain emotions elegantly
- catch every implication
- answer without stumbling or choosing the wrong emphasis

Fail when the same high-level conversational competence is distributed across the cast regardless of age, profession, culture, stress, or relationship.

Repair by returning to each speaker's first attention, knowledge limits, social strategy, and pressure leak. Some characters should miss implications, speak too concretely, overexplain, choose logistics, become less verbal, or decide the conversation is not worth having.

## Gate 6: Aphorism and thematic-summary density

Flag polished lines that compress the scene's theme into a quotable observation. One can land. A sequence makes characters sound authored.

Alien, noble, formal, academic, and emotionally perceptive characters are especially vulnerable. Replace excess aphorisms with specific observations, imperfect wording, silence, or action.

Fail when multiple characters take turns stating the chapter's meaning.

## Gate 7: Cultural and moral neatness

Flag exchanges that convert awkward history, prejudice, cultural difference, authority, or grief into a clean modern lesson within a few lines.

Fail when the speaker supplies the authorially correct formulation without hesitation, personal limitation, cost, or unresolved discomfort.

Repair through narrower experience, misreading, incomplete explanation, a boundary, or a practical response. Do not force a debate when the character would disengage.

## Gate 8: Objective and state change

For every dialogue block, state what each speaker wants and what changes by the end: knowledge, trust, status, permission, risk, intimacy, commitment, or available action.

Fail lines that neither pursue an objective nor create resistance, relationship evidence, choice, or consequence. "Characterful" surface texture is not sufficient.

## Gate 9: Speakability and restraint

Read the exchange aloud or simulate performance. Flag lines that are too balanced, polished, complete, or abstract for the speaker under current pressure.

Do not roughen every line. Preserve clarity where the character would be clear. Add hesitation, incompletion, interruption, or silence only when motivated.

## Review output

For each confirmed failure, report:

```text
Location:
Gate:
Speakers:
What the exchange is doing now:
Why it fails for these characters:
Smallest structural repair:
Canon or voice risk:
Lines preserved:
```

End with a verdict:

- `PASS`: no material architecture failure.
- `REVISE`: line-level or local structural repair is sufficient.
- `REBUILD`: the conversation's objective or knowledge architecture must change before line editing.

Do not call dialogue complete while any material gate remains failed. A `REBUILD` verdict is a diagnosis, not authorization to perform the rewrite.
