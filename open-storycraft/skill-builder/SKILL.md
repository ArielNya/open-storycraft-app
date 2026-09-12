---
name: skill-builder
description: Interactive, provider-neutral skill designer. Walks through use-case
  definition, shared folder structure, SKILL.md generation, script planning, and
  validation. Use when user says "create a skill", "build a skill", "new skill",
  "help me design a skill", or "I want to make a skill".
metadata:
  author: Custom
  version: "2.0.0"
  category: meta
---

# Skill Builder

Helps design and generate complete, production-ready skills for the shared `.agents/skills/` library so compatible agents, including Codex and Gemini, can use the same skill.

Read the full portable reference: [references/guide.md](references/guide.md).

---

## Step 1: Understand the Use Case

Ask the user ONE question that combines:

> Tell me about the skill you want to build:
> - **What does it do?** (one sentence)
> - **What triggers it?** (what would a user say to activate it?)
> - **What does it produce?** (a file, output text, a generated artifact, a focused worker?)
> - **Does it need scripts, external data, or reference docs?**
>
> Answer as much or as little as you know — I'll fill in the gaps.

Wait for the response before proceeding.

---

## Step 2: Classify the Skill

Based on the answer, determine which pattern applies:

| Pattern | Use when |
|---|---|
| **Document / Asset Creator** | Skill produces a file or artifact (report, outline, profile, config) |
| **Workflow Orchestrator** | Skill runs a multi-step pipeline, uses focused workers, coordinates tools |
| **Generator + Script** | Skill wraps a Python or shell script and presents results |
| **Reference + Advisor** | Skill loads domain knowledge and applies it to user input |
| **Meta / Builder** | Skill creates other skills or structured artifacts |

Tell the user which pattern you're applying and why. If it spans multiple patterns, say so.

---

## Step 3: Design the Structure

Based on the pattern, propose the folder layout:

```
skill-name/
├── SKILL.md              # Always required
├── scripts/              # If wrapping executable code
│   └── main.py
├── references/           # If loading domain knowledge or guides
│   └── topic.md
└── assets/               # If providing templates
    └── template.md
```

For orchestrator skills, identify whether the skill runs inline or uses focused workers.

- **Inline:** instructions run directly in the coordinating conversation.
- **Focused worker:** use when the task is long, produces files, or benefits from isolated context. Give the worker fully self-contained instructions. Run workers in parallel when the host supports it; otherwise preserve the same tasks sequentially.

Ask the user to confirm or adjust the structure before continuing.

---

## Step 4: Generate the SKILL.md

Write the complete `SKILL.md`. Always follow these rules.

### Frontmatter

- `name`: kebab-case and matches the folder name.
- `description`: what it does plus when to trigger, including phrases the user would say. Keep it under 1,024 characters and do not use `<` or `>`.
- Add `metadata` with `author`, `version`, and `category`.

### Body structure by pattern

**Generator + Script skills:**

```markdown
## Step 1: Gather inputs if needed
## Step 2: Run the Script
[Host-neutral interpreter instruction with exact script path and arguments]
## Step 3: Present Results
[What to show and what to offer next]
```

**Orchestrator / Focused-worker skills:**

```markdown
## Step 1: Identify Project
[Search for existing files and extract working title/context]
## Step 2: Run Focused Work
[State the task division and whether parallel execution is optional or required]
## Step 3: Present Results
[How to show output, offer revisions, suggest next skill]

---
## Worker Instructions
[Everything each worker needs — reads, writes, and outputs — fully self-contained]
```

**Reference + Advisor skills:**

```markdown
## Step 1: Read [relative reference file]
## Step 2: Apply to User Input
[How to use the reference against what the user provided]
## Step 3: Output
[Format, what to include, what to offer next]
```

### Quality rules

- Every step has a clear trigger and output.
- Script instructions name the required interpreter and exact relative path and flags without assuming a provider-specific shell.
- Worker instructions are fully self-contained; workers do not inherit conversation context reliably.
- Paths inside the skill are relative to the skill folder or linked `SKILL.md`.
- Do not name a proprietary model tier unless the task genuinely requires one; describe the capability needed.
- Offer the user a next step at the end: rerun, adjust, or chain to another skill.

---

## Step 5: Plan Supporting Files

For each referenced file:

- Write it now if the user has the content.
- Stub it with a placeholder and say what it needs.
- Point to an existing shared file with a relative link if it already exists.

For scripts, write a Python or shell skeleton with the main function signature, argument parsing, and output format even if the logic is not filled in.

---

## Step 6: Validate

Run through this checklist before presenting the final output:

- [ ] Folder name is kebab-case.
- [ ] `name` in frontmatter matches the folder name.
- [ ] `description` includes what and when, with concrete trigger phrases.
- [ ] `description` is under 1,024 characters and contains no `<` or `>`.
- [ ] Every step has a clear action and expected output.
- [ ] Worker instructions are self-contained.
- [ ] Scripts use real relative paths and flag names.
- [ ] No provider-specific home paths, tool names, model names, or invocation syntax remain unless explicitly required.
- [ ] Every referenced local file resolves.
- [ ] The host can discover exactly one skill entry from the new folder.
- [ ] The skill offers a next step.

Fix validation failures before delivery.

---

## Step 7: Deliver

Create the skill in `.agents/skills/skill-name/` unless the user requested a different shared root. Present:

1. The complete folder structure.
2. The created `SKILL.md`.
3. Any scripts, references, or assets.
4. Validation results and the installed shared path.

Then ask:

> Want me to write any remaining supporting files or adjust anything?

