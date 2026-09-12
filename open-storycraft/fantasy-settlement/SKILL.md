---
name: fantasy-settlement
description: "Generate a fantasy settlement profile — population, economy, buildings, security, fun facts. Use when user says 'settlement', 'generate a town', 'town profile', 'what's in this city', or provides a town name and type."
metadata:
  author: Custom
  version: "1.0.0"
  category: utilities
---

# Fantasy Settlement Generator

Skill resources are relative to this `SKILL.md`.

Generates a full settlement profile from a name + description.

## Step 1: Get Name and Type

If the user didn't provide a name and settlement type, ask:
- **Name** — the settlement name (e.g. "Colindo")
- **Type** — describe it freely (e.g. "sky town", "coastal city", "underground village", "mountain hamlet")

You can combine multiple descriptors: "ruined coastal city", "swamp village", "volcanic trading town".

## Step 2: Run the Script

Run [scripts/generate_settlement.py](scripts/generate_settlement.py) with the host's available Python interpreter, passing `"<name>" "<type description>"`.

Capture the script output as UTF-8. On a host using a legacy console encoding, enable UTF-8 for this run before invoking the unchanged script.

Examples:
```text
python generate_settlement.py "Colindo" "sky town"
python generate_settlement.py "San Almada" "coastal city"
python generate_settlement.py "Masanieda" "underground village"
```

## Step 3: Present Results

Show the output as-is. Then offer to:
- Re-roll (run again for different numbers / facts)
- Generate another settlement
- Adjust the type (e.g. "make it a city instead")

## Available Biomes

sky · coastal · underground · forest · desert · mountain · swamp · island · plains · tundra · volcanic · ruins

Any size word works: hamlet · village · town · city · metropolis

