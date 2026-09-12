#!/usr/bin/env python3
"""Markov chain name generator."""

from __future__ import annotations

import argparse
import random
import sys
from collections import defaultdict
from pathlib import Path


def build_model(names: list[str], order: int) -> dict:
    model = defaultdict(list)
    for name in names:
        name = name.strip().lower()
        if len(name) < order + 1:
            continue
        padded = "\x00" * order + name + "\x01"  # \x00 = start, \x01 = end
        for i in range(len(padded) - order):
            key = padded[i : i + order]
            model[key].append(padded[i + order])
    return model


def generate_name(model: dict, order: int, min_len: int, max_len: int, max_attempts: int = 200) -> str | None:
    for _ in range(max_attempts):
        key = "\x00" * order
        result = []
        for _ in range(max_len + order):
            choices = model.get(key)
            if not choices:
                break
            next_char = random.choice(choices)
            if next_char == "\x01":
                break
            result.append(next_char)
            key = key[1:] + next_char
        name = "".join(result)
        if min_len <= len(name) <= max_len:
            return name.capitalize()
    return None


def load_names(path: Path) -> list[str]:
    text = path.read_text(encoding="utf-8")
    names = [line.strip() for line in text.splitlines() if line.strip()]
    if not names:
        print(f"Error: no names found in {path}", file=sys.stderr)
        sys.exit(1)
    return names


def main():
    parser = argparse.ArgumentParser(description="Markov chain name generator")
    parser.add_argument("--list", required=True, type=Path, help="Path to name list .txt file")
    parser.add_argument("--count", type=int, default=10, help="Number of names to generate")
    parser.add_argument("--min-len", type=int, default=4, help="Minimum name length")
    parser.add_argument("--max-len", type=int, default=12, help="Maximum name length")
    parser.add_argument("--order", type=int, default=2, choices=[1, 2, 3], help="Markov chain order")
    parser.add_argument("--seed", type=int, default=None, help="Random seed for reproducibility")
    args = parser.parse_args()

    if args.seed is not None:
        random.seed(args.seed)

    if not args.list.exists():
        print(f"Error: file not found: {args.list}", file=sys.stderr)
        sys.exit(1)

    names = load_names(args.list)
    model = build_model(names, args.order)

    generated = []
    seen = set()
    attempts = 0
    max_total_attempts = args.count * 500

    while len(generated) < args.count and attempts < max_total_attempts:
        attempts += 1
        name = generate_name(model, args.order, args.min_len, args.max_len)
        if name and name.lower() not in seen:
            seen.add(name.lower())
            generated.append(name)

    if not generated:
        print("Could not generate any names. Try a larger name list or different parameters.", file=sys.stderr)
        sys.exit(1)

    for name in generated:
        print(name)

    if len(generated) < args.count:
        print(f"\n(Only {len(generated)} unique names could be generated with these parameters)", file=sys.stderr)


if __name__ == "__main__":
    main()
