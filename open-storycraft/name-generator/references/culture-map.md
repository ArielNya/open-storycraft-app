# Culture → Generator Map

Used by fiction-characters and fiction-world workers to select the right
name/town list based on the setting's cultural identity.

## Name Generator

Script: [generate.py](../scripts/generate.py)

| Setting / Culture         | List file(s)                          | Notes                        |
|---------------------------|---------------------------------------|------------------------------|
| Modern English            | `english-modern.txt`                  | Contemporary, male & female  |
| Modern French             | `french-modern.txt`                   | Contemporary, male & female  |
| Modern Spanish            | `spanish-modern.txt`                  | Contemporary, male & female  |
| Modern Italian            | `italian-modern.txt`                  | Contemporary, male & female  |
| Anglo-Saxon / Old English | `anglo.txt`                           | Pre-Norman English           |
| Norse / Viking            | `norwegian.txt`                       | Male & female mixed          |
| Ancient Greek             | `greek.txt`                           |                              |
| Roman / Latin             | `latin.txt`                           |                              |
| Medieval Spanish          | `spanish.txt`                         | Iberian, Moorish era         |
| Medieval French           | `french.txt`                          |                              |
| Medieval Italian          | `italian.txt`                         |                              |
| Germanic / Central Euro   | `german.txt`                          |                              |
| Japanese                  | `japanese-male.txt` / `japanese-female.txt` | Use by character gender |
| Korean                    | `korean-male.txt` / `korean-female.txt`     |                         |
| Ancient Egyptian          | `egypt-male.txt` / `egypt-female.txt` | Use by character gender      |
| Mesoamerican / Aztec      | `aztec.txt`                           |                              |
| Mesopotamian / Babylonian | `babylon.txt`                         |                              |
| Phoenician / Carthaginian | `phoenicia.txt`                       |                              |
| Generic Fantasy / Elvish  | `fantasy.txt`                         |                              |

### Usage

Run [generate.py](../scripts/generate.py) with the host's available Python interpreter:

```text
--list ../data/<list> --count 30 --order 2 --min-len 4 --max-len 12
```

Generate 20–30 names, pick from them for your characters. Use names from the
list as a pool — don't take them verbatim if they feel too close to real names,
but use them as inspiration and starting points.

---

## Town Generator

Script: [generate_town.py](../../town-generator/scripts/generate_town.py)

| Setting / Culture         | List file                  | Prefix file                         |
|---------------------------|----------------------------|-------------------------------------|
| Medieval English          | `english.txt`              | —                                   |
| Norse / Viking            | `norse.txt`                | —                                   |
| Roman / Latin             | `roman.txt`                | —                                   |
| Generic Fantasy           | `fantasy.txt`              | —                                   |
| Spanish / Iberian         | `spanish-cities.txt`       | `spanish-prefixes.txt`              |
| Italian                   | `italian-cities.txt`       | —                                   |
| Chinese                   | `chinese-cities.txt`       | —                                   |
| Japanese                  | `japanese-cities.txt`      | —                                   |

### Usage

Run the sibling town generator with the host's available Python interpreter.

Without prefix (most cultures):
```text
--list ../../town-generator/data/<list> --count 10 --order 2 --epithet-chance 0.3
```

With prefix (Spanish):
```text
--list ../../town-generator/data/spanish-cities.txt --prefix-file ../../town-generator/data/spanish-prefixes.txt --prefix-chance 0.35 --epithet-chance 0.25 --count 10
```

Generate 10–15 town names, pick the ones that fit the story's tone.
Epithets (e.g. "Vorheim, the Throne of the Dead") work well for major cities;
use bare names for villages and minor settlements.
