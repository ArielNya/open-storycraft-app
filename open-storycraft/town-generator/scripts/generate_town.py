#!/usr/bin/env python3
"""
Town name generator — wraps generate.py + epithet.py.
Supports optional prefixes (e.g. San/Santa/Monte for Spanish towns)
and optional epithets (e.g. "the Throne of the Gods") appended to the name.
"""

from __future__ import annotations
import argparse
import random
import subprocess
import sys
from pathlib import Path

BASE = Path(__file__).parent
DATA = BASE.parent / "data"
GENERATE = BASE / "generate.py"

sys.path.insert(0, str(BASE))
from epithet import generate_epithet


def load_prefixes(prefix_file: Path) -> list[str]:
    """Load weighted prefix list. Format: prefix<TAB>weight or just prefix."""
    prefixes = []
    for line in prefix_file.read_text().splitlines():
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        parts = line.split("\t")
        prefix = parts[0].strip()
        weight = int(parts[1]) if len(parts) > 1 else 1
        prefixes.extend([prefix] * weight)
    return prefixes


def run_markov(list_path: Path, count: int, min_len: int, max_len: int, order: int) -> list[str]:
    result = subprocess.run(
        [sys.executable, str(GENERATE),
         "--list", str(list_path),
         "--count", str(count),
         "--min-len", str(min_len),
         "--max-len", str(max_len),
         "--order", str(order)],
        capture_output=True, text=True
    )
    return [line.strip() for line in result.stdout.splitlines() if line.strip()]


def main():
    parser = argparse.ArgumentParser(description="Fantasy town name generator")
    parser.add_argument("--list", type=Path, required=True)
    parser.add_argument("--count", type=int, default=5)
    parser.add_argument("--min-len", type=int, default=5)
    parser.add_argument("--max-len", type=int, default=14)
    parser.add_argument("--order", type=int, default=2, choices=[1, 2, 3])
    parser.add_argument("--epithet-chance", type=float, default=0.4,
                        help="Probability of appending an epithet (0–1, default 0.4)")
    parser.add_argument("--prefix-file", type=Path, default=None,
                        help="Weighted prefix list file (e.g. spanish-prefixes.txt)")
    parser.add_argument("--prefix-chance", type=float, default=0.35,
                        help="Probability of prepending a prefix (0–1, default 0.35)")
    args = parser.parse_args()

    prefixes = load_prefixes(args.prefix_file) if args.prefix_file else []

    names = run_markov(args.list, args.count, args.min_len, args.max_len, args.order)

    for name in names:
        # Optionally prepend a prefix
        if prefixes and random.random() < args.prefix_chance:
            name = f"{random.choice(prefixes)} {name}"

        # Optionally append an epithet
        if random.random() < args.epithet_chance:
            print(f"{name}, {generate_epithet()}")
        else:
            print(name)


if __name__ == "__main__":
    main()
