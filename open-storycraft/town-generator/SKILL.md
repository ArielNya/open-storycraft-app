---
name: town-generator
description: Generate fantasy town and settlement names using a Markov chain model. Use
  when user says "generate town names", "fantasy towns", "settlement names", "village
  names", or "town generator".
metadata:
  author: Custom
  version: "1.0.0"
  category: utilities
---

# Fantasy Town Name Generator

Skill resources are relative to this `SKILL.md`.

Generates fantasy town and settlement names using a Markov chain model trained on place name lists in `data/`.

## Default Behavior: Full Sweep

By default, generate **3 names from every list** in `data/` and present them grouped by style/culture. Do NOT ask questions first — just run it and show results.

Enumerate every `data/*.txt` file. For each file, use [scripts/generate.py](scripts/generate.py) with the host's available Python interpreter and these arguments:

```text
--list <data-file> --count 3 --min-len 4 --max-len 14 --order 2
```

Use the data filename without `.txt` as the style/culture heading.

Present output as a clean grouped list:

**Anglo-Saxon** — Aldenmere, Wulfhaven, Eadburgh
**Norse** — Grimholt, Skaldvik, Bjornstad
...

## After Showing Results

Offer to:
- Re-roll the full sweep
- Generate more from a specific style
- Adjust count or Markov order

