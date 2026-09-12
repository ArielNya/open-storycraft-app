#!/usr/bin/env python3
"""
Generate fantasy town epithets using a template system.
Used to append titles like "the Throne of Evrarlon" or "City of the First Light"
to Markov-generated base town names.
"""

from __future__ import annotations
import random

NOUNS = [
    "Throne", "Crown", "Heart", "Gate", "Tower", "Spire", "Keep", "Hold",
    "Flame", "Stone", "Shadow", "Light", "Dawn", "Dusk", "Tide", "Wind",
    "Forge", "Tomb", "Cradle", "Vault", "Eye", "Fist", "Sword", "Shield",
    "Beacon", "Altar", "Maw", "Refuge", "Citadel", "Bastion", "Sanctum",
    "Pyre", "Well", "Font", "Pillar", "Root", "Key", "Seal", "Nexus",
]

ADJECTIVES = [
    "First", "Last", "Ancient", "Eternal", "Fallen", "Broken", "Hidden",
    "Forgotten", "Burning", "Frozen", "Crimson", "Silver", "Golden", "Iron",
    "Storm", "Silent", "Undying", "Lost", "Sacred", "Cursed", "Hollow",
    "Blessed", "Bitter", "Ashen", "Shining", "Dark", "High", "Deep", "Far",
]

PLACES = [
    "the Gods", "the North", "the South", "the East", "the West",
    "the Dead", "the Living", "the Fallen", "the Faithful", "the Exiled",
    "Ash", "Dust", "Fire", "Ice", "Stone", "Shadow", "Light", "Blood",
    "the Deep", "the World", "the Ages", "the Realm", "the Covenant",
    "the First Men", "the Old Kingdom", "the Ancients", "the Forgotten",
]

TEMPLATES = [
    lambda: f"the {random.choice(NOUNS)} of {random.choice(PLACES)}",
    lambda: f"City of the {random.choice(ADJECTIVES)} {random.choice(NOUNS)}",
    lambda: f"the {random.choice(ADJECTIVES)} {random.choice(NOUNS)}",
    lambda: f"{random.choice(ADJECTIVES)} {random.choice(NOUNS)} of {random.choice(PLACES)}",
    lambda: f"the {random.choice(NOUNS)} of {random.choice(ADJECTIVES)} {random.choice(NOUNS)}",
]


def generate_epithet() -> str:
    return random.choice(TEMPLATES)()


if __name__ == "__main__":
    for _ in range(10):
        print(generate_epithet())
