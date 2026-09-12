---
name: name-generator
description: Generate names using a Markov chain model trained on name lists. Use when
  user says "generate names", "create names", "make up names", "fantasy names", or
  "name generator".
metadata:
  author: Custom
  version: "1.0.0"
  category: utilities
---

# Markov Name Generator

Generates names using a Markov chain model trained on name lists in `data/`.

## Default Behavior: Full Sweep

By default, generate **3 names from every list** in `data/` and present them all grouped by culture. Do NOT ask questions first — just run it and show the results.

Enumerate every `data/*.txt` file. For each file, use [scripts/generate.py](scripts/generate.py) with the host's available Python interpreter and these arguments:

```text
--list <data-file> --count 3 --min-len 4 --max-len 12 --order 2
```

Use the data filename without `.txt` as the culture heading.

Present the output as a clean grouped list, e.g.:

**Anglo** — Wulfstan, Eadric, Aethelmere
**Aztec** — Coyotl, Xiuhtla, Huitzin
...

## After Showing Results

Offer to:
- Generate more from a specific list
- Re-roll the full sweep
- Adjust count or Markov order

