#!/usr/bin/env python3
"""
Fantasy Settlement Generator
Generates population, economy, buildings, security, and flavor
for a named settlement, driven by settlement type/location.

Usage:
  python3 generate_settlement.py "Colindo" "sky town"
  python3 generate_settlement.py "Masanieda" "coastal village"
  python3 generate_settlement.py "San Almada" "mountain city"
"""

from __future__ import annotations
import random
import sys
import math

# ─── Settlement Size ──────────────────────────────────────────────────────────

SIZE_BANDS = {
    "hamlet":     (30,   200),
    "village":    (200,  1000),
    "town":       (1000, 5000),
    "city":       (5000, 25000),
    "metropolis": (25000, 120000),
}

# Keywords that map to a size band
SIZE_KEYWORDS = {
    "hamlet": "hamlet", "thorp": "hamlet", "thorpe": "hamlet",
    "village": "village", "settlement": "village",
    "town": "town", "borough": "town",
    "city": "city", "citadel": "city", "capital": "metropolis",
    "metropolis": "metropolis", "megalopolis": "metropolis",
}

# ─── Location / Biome Economies ───────────────────────────────────────────────

ECONOMIES = {
    "sky":         ["Sky Coral Harvest", "Cloud Crystal Mining", "Airship Trade",
                    "Wind Pearl Fishing", "Cloudweave Textiles", "Aethervine Harvest",
                    "Storm-Glass Blowing", "Skyfish Salting"],
    "coastal":     ["Deep Pearl Diving", "Saltfish Trade", "Shipbuilding",
                    "Smuggling", "Sea-Glass Craft", "Whale Oil Rendering",
                    "Tide Herb Gathering", "Naval Provisioning",
                    "Wrecking & Salvage", "Fencing Stolen Cargo",
                    "Black Market Goods", "Slaving"],
    "underground": ["Gem Cutting", "Deepmushroom Farming", "Blindfish Trade",
                    "Dark Crystal Extraction", "Iron Smelting", "Bone Carving",
                    "Saltpeter Mining", "Cave Moss Weaving"],
    "forest":      ["Lumber Trade", "Hunting & Trapping", "Herb Gathering",
                    "Woodcraft", "Druidic Services", "Silk Spider Farming",
                    "Charcoal Burning", "Treesap Distilling"],
    "desert":      ["Spice Trade", "Glass Blowing", "Caravan Provisioning",
                    "Oasis Farming", "Salt Flats Mining", "Dune Silk Weaving",
                    "Scorpion Venom Harvest", "Obsidian Quarrying"],
    "mountain":    ["Iron Mining", "Stone Quarrying", "Goat & Sheep Herding",
                    "Dwarven Metalcraft", "Glacier Ice Trade", "Gem Prospecting",
                    "Highland Spirits Distilling", "Goat Cheese Trade"],
    "swamp":       ["Peat Harvesting", "Exotic Herb Alchemy", "Toad Farming",
                    "Crocodile Leather", "Fever Moss Trade", "Piranha Fishing",
                    "Bog Iron Smelting", "Will-o-Wisp Bottling"],
    "island":      ["Exotic Goods Trade", "Piracy", "Pearl Fishing",
                    "Coconut Oil Pressing", "Shipbuilding", "Turtle Shell Craft",
                    "Rum Distilling", "Coral Sculpting", "Sugar Cane Plantations",
                    "Wrecking & Salvage", "Smuggling", "Slaving"],
    "plains":      ["Grain Farming", "Cattle Ranching", "Horse Breeding",
                    "Wool Trade", "Milling", "Mercenary Hosting",
                    "Trade Road Tolls", "Honey & Mead"],
    "tundra":      ["Fur Trading", "Mammoth Ivory Carving", "Ice Fishing",
                    "Reindeer Herding", "Frost-Iron Mining", "Blubber Rendering"],
    "volcanic":    ["Obsidian Trade", "Sulfur Mining", "Lava Glass Craft",
                    "Geothermal Forging", "Ash Silk Weaving", "Fire Opal Mining"],
    "ruins":       ["Relic Salvage", "Tomb Raiding", "Cursed Artifact Brokerage",
                    "Antiquities Trade", "Dungeon Provisioning", "Scrap Metal"],
    "default":     ["General Trade", "Farming", "Craftwork", "River Tolls",
                    "Mercenary Hosting", "Livestock Trade", "Textile Weaving",
                    "Pilgrimage Services"],
    "scifi":       ["Ore Processing", "Fuel Refining", "Data Brokerage",
                    "Salvage Operations", "Black Market Tech", "Clone Labor Contracting",
                    "Zero-G Manufacturing", "Contraband Transit", "Arms Dealing",
                    "Biotech Research", "Asteroid Mining", "Passenger Transit",
                    "Cybernetics Installation", "Unlicensed Medical Services"],
}

# ─── Security ─────────────────────────────────────────────────────────────────

WALLS = {
    "hamlet":     ["None", "Wooden Stakes", "Thorn Hedge"],
    "village":    ["Wooden Palisade", "Earthen Berm", "Ditch & Rampart"],
    "town":       ["Wooden Palisade", "Stone Wall", "Reinforced Earthwork"],
    "city":       ["Stone Wall", "Double Stone Wall", "Crenellated Stone Wall"],
    "metropolis": ["Massive Stone Wall", "Layered Fortress Wall", "Ancient Enchanted Ramparts"],
}

WALLS_SCIFI = {
    "hamlet":     ["None", "Basic Airlock", "Emergency Bulkhead"],
    "village":    ["Pressure Seal", "Reinforced Bulkhead", "Single Hull Layer"],
    "town":       ["Double Hull Plating", "Magnetic Field Barrier", "Reinforced Pressure Seal"],
    "city":       ["Triple Hull Plating", "Energy Shield Grid", "Armored Blast Doors"],
    "metropolis": ["Layered Shield Array", "Quantum-Locked Hull", "Automated Defense Perimeter"],
}

FORTIFICATIONS = {
    "hamlet":     ["None", "Watchtower"],
    "village":    ["Watchtower", "Wooden Keep"],
    "town":       ["Stone Keep", "Motte-and-Bailey Castle"],
    "city":       ["Stone Castle", "Fortress", "Citadel"],
    "metropolis": ["Grand Fortress", "Royal Citadel", "Impregnable Stronghold"],
}

FORTIFICATIONS_SCIFI = {
    "hamlet":     ["None", "Sentry Drone Post"],
    "village":    ["Sentry Drone Post", "Guard Post with Scanner"],
    "town":       ["Security Hub", "Armed Checkpoint"],
    "city":       ["Defense Platform", "Automated Turret Array"],
    "metropolis": ["Orbital Defense Grid", "Military Command Deck"],
}

# Guards as % of population
GUARD_RATE = {
    "hamlet": 0.05, "village": 0.04, "town": 0.03,
    "city": 0.025, "metropolis": 0.02,
}

# ─── Fun Facts ────────────────────────────────────────────────────────────────

FUN_FACTS = [
    "Pirate hideout", "Founded by escaped slaves", "Built on a mass grave",
    "The mayor is secretly a vampire", "Haunted by the founder's ghost",
    "Outlawed the use of iron — all tools are bronze or stone",
    "Every building is painted a different shade of red",
    "Home to the world's oldest living tortoise (rumored to speak)",
    "No one born here has ever died of natural causes",
    "The town bell hasn't rung in 40 years — touching it is forbidden",
    "Run entirely by a thieves' guild who tax openly",
    "Three different religions claim it as a holy site",
    "The water makes newcomers sleep for three days",
    "Famous for a cheese that smells of corpses but tastes divine",
    "No one knows who built the central tower — it predates the town",
    "Refugees from a destroyed kingdom make up 60% of the population",
    "A dragon once lived here — its bones form the town hall roof",
    "Dueling is legal and encouraged for dispute resolution",
    "The town sits on a ley line — magic behaves unpredictably here",
    "All residents have the same recurring dream",
    "Slavery is practiced openly and legally",
    "The garrison has not been paid in six months",
    "A serial killer has gone uncaught for a decade — locals blame outsiders",
    "Recently sacked — half the buildings are still ash",
    "Under a curse: no children have been born here in a generation",
    "The wealthiest resident is a beggar who refuses to spend their fortune",
    "Every tavern serves the same ale — brewed by a single secretive family",
    "The church burned down; two rival sects now meet in the ruins",
    "Controlled by a single merchant family who own nearly every building",
    "Seat of a resistance movement against the current regime",
]

# ─── Building Ratios (per 100 population) ─────────────────────────────────────
# (min_per_100, max_per_100)

BUILDING_RATIOS = {
    "Taverns":         (0.4,  0.9),
    "Inns":            (0.2,  0.5),
    "Bakeries":        (0.3,  0.7),
    "Blacksmiths":     (0.1,  0.3),
    "Stables":         (0.1,  0.3),
    "Temples / Shrines": (0.1, 0.4),
    "Healers":         (0.05, 0.2),
    "Apothecaries":    (0.05, 0.15),
    "Markets / Stalls":(0.3,  0.8),
    "Brothels":        (0.1,  0.4),
    "Guildhalls":      (0.02, 0.1),
    "Jails":           (0.02, 0.08),
}

# Scifi building ratios — replaces BUILDING_RATIOS entirely for scifi genre
BUILDING_RATIOS_SCIFI = {
    "Docking Bays":          (0.05, 0.2),
    "Med Bays":              (0.05, 0.15),
    "Cantinas / Mess Halls": (0.3,  0.7),
    "Bunk Blocks":           (0.2,  0.5),
    "Black Market Stalls":   (0.1,  0.4),
    "Tech Shops":            (0.1,  0.3),
    "Cargo Warehouses":      (0.2,  0.6),
    "Security Posts":        (0.05, 0.2),
    "Recycling Plants":      (0.05, 0.15),
    "Data Terminals":        (0.05, 0.2),
    "Power Relay Stations":  (0.02, 0.1),
    "Detention Blocks":      (0.02, 0.08),
}

# Extra buildings for specific biomes
EXTRA_BUILDINGS = {
    "sky":         {"Airship Docks": (0.05, 0.2), "Skysail Repair Yards": (0.02, 0.1)},
    "coastal":     {"Shipyards": (0.05, 0.2), "Fishmongers": (0.2, 0.6)},
    "underground": {"Mineshaft Entrances": (0.1, 0.4), "Mushroom Farms": (0.1, 0.3)},
    "forest":      {"Lumber Mills": (0.05, 0.2), "Hunter Lodges": (0.05, 0.2)},
    "desert":      {"Caravanserais": (0.05, 0.2), "Water Merchants": (0.05, 0.15)},
    "mountain":    {"Mines": (0.1, 0.4), "Assay Offices": (0.02, 0.1)},
    "swamp":       {"Alchemist Dens": (0.05, 0.2), "Boatyards": (0.05, 0.15)},
    "island":      {"Fishmongers": (0.2, 0.6), "Shipyards": (0.05, 0.2)},
    "volcanic":    {"Forges": (0.1, 0.3), "Obsidian Cutters": (0.05, 0.15)},
    "ruins":       {"Salvage Yards": (0.05, 0.2), "Relic Dealers": (0.05, 0.15)},
}

# ─── Genre Detection ──────────────────────────────────────────────────────────

SCIFI_KEYWORDS = {
    "space", "asteroid", "station", "orbital", "colony", "sector", "void",
    "cyber", "neo", "quantum", "plasma", "synth", "drone", "android",
    "alien", "galactic", "stellar", "starship", "hull", "module", "habitat",
    "zero-g", "warp", "airlock",
}

def detect_genre(description: str) -> str:
    desc = set(description.lower().split())
    return "scifi" if desc & SCIFI_KEYWORDS else "fantasy"


# ─── NPCs ─────────────────────────────────────────────────────────────────────

NPC_NAME_SYLLABLES = [
    "Ar", "Bel", "Cor", "Dal", "En", "Far", "Gal", "Hel", "Is", "Jor",
    "Kel", "Lor", "Mal", "Nor", "Or", "Pel", "Ral", "Sel", "Tor", "Ul",
    "Van", "Wex", "Xar", "Yor", "Zan",
]
NPC_NAME_ENDINGS = [
    "an", "en", "in", "on", "ath", "ix", "us", "a", "ia", "or", "ek",
    "ara", "is", "eth", "wyn", "eld", "and", "ack", "un", "os",
]
SCIFI_NPC_NAMES = [
    "Vex-7", "Unit 44", "Koss", "Yula", "Drexl", "Pav", "Zuri", "Naks",
    "Orah", "Silon", "Brix", "Maka", "Cael", "Jova", "Ryze",
]

def random_npc_name(genre: str) -> str:
    if genre == "scifi" and random.random() < 0.4:
        return random.choice(SCIFI_NPC_NAMES)
    s1 = random.choice(NPC_NAME_SYLLABLES)
    s2 = random.choice(NPC_NAME_SYLLABLES).lower()
    end = random.choice(NPC_NAME_ENDINGS)
    return f"{s1}{s2}{end}" if random.random() < 0.5 else f"{s1}{end}"


NPC_ROLES = {
    "default": [
        "Mayor", "Crime Lord", "Head Innkeeper", "Master Merchant",
        "High Priest", "Chief Healer", "Spymaster", "Tax Collector",
        "Retired Soldier", "Notorious Fence", "Wandering Scholar",
        "Corrupt Judge", "Beggar King", "Black Market Broker",
    ],
    "sky": ["Airship Admiral", "Cloud Farmer", "Sky Cartographer", "Wind Merchant"],
    "coastal": ["Harbor Master", "Privateer Captain", "Pearl Broker", "Tidepriest"],
    "underground": ["Mine Overseer", "Tunnel Guide", "Deepwarden", "Gem Assessor"],
    "forest": ["Warden", "Master Trapper", "Druidic Elder", "Woodcutter Boss"],
    "desert": ["Caravan Master", "Water Baron", "Dune Prophet", "Sand Witch"],
    "mountain": ["Mine Baron", "High Warden", "Glacier Monk", "Ore Assessor"],
    "swamp": ["Bog Witch", "Peat Baron", "Fever Herbalist", "Marsh Warden"],
    "island": ["Pirate Queen", "Pearl Merchant", "Island Shaman", "Tide Watcher"],
    "ruins": ["Relic Broker", "Tomb Raider", "Curse Breaker", "Archaeologist"],
    "scifi": [
        "Station Commander", "Salvage Chief", "Data Broker", "Chief Engineer",
        "Black Market Tech", "Medtech", "Jump Pilot", "Corporate Liaison",
        "AI Warden", "Contraband Assessor", "Sector Judge",
    ],
}

NPC_QUIRKS = [
    "has a price on their head in three jurisdictions",
    "claims to be the rightful heir to something important",
    "knows exactly one secret that could topple the current order — and knows it",
    "collects teeth from everyone they've ever wronged",
    "lost their memory four years ago and has been faking it since",
    "is secretly running two rival factions against each other",
    "owes an enormous debt to someone nobody wants to cross",
    "has survived three assassination attempts this season alone",
    "is dying — and has made peace with it in a very unsettling way",
    "speaks only in questions",
    "has not slept in six days and nobody knows why",
    "is visibly terrified of something they refuse to name",
    "has a twin no one knows about",
    "was present at a famous disaster and will not discuss it",
    "runs a legitimate business as cover for something far more interesting",
    "is genuinely, sincerely kind — which makes everyone deeply suspicious",
    "once held enormous power and lost it all in a single day",
    "is an informant for at least two competing interests",
    "has a child they've never met who is about to become a problem",
    "believes they are the chosen one — and might not be wrong",
]

def generate_npcs(biome: str, genre: str, count: int = 3) -> list[tuple[str, str, str]]:
    roles = NPC_ROLES["default"][:]
    roles += NPC_ROLES.get(biome, [])
    if genre == "scifi":
        roles += NPC_ROLES["scifi"]
    random.shuffle(roles)
    result = []
    seen_roles = set()
    seen_names = set()
    quirks = NPC_QUIRKS[:]
    random.shuffle(quirks)
    quirk_iter = iter(quirks)
    for role in roles:
        if role in seen_roles or len(result) >= count:
            continue
        seen_roles.add(role)
        # Generate a unique name
        for _ in range(20):
            name = random_npc_name(genre)
            if name not in seen_names:
                seen_names.add(name)
                break
        result.append((name, role, next(quirk_iter)))
    return result


# ─── Factions ─────────────────────────────────────────────────────────────────

ORG_TYPES_FANTASY = [
    "Guild", "Brotherhood", "Sisterhood", "Confederation", "Order",
    "Society", "Lodge", "Syndicate", "Compact", "Circle", "Cabal",
    "Fellowship", "Union", "Collective", "Assembly", "Council",
]
ORG_TYPES_SCIFI = [
    "Cooperative", "Syndicate", "Collective", "Cartel", "Union",
    "Consortium", "Assembly", "Network", "Bloc", "Coalition", "Corp",
]

FACTION_ADJECTIVES = [
    "Honorable", "Ancient", "Silent", "Gilded", "Crimson", "Iron",
    "Forgotten", "Blessed", "Broken", "Wandering", "Hidden", "Eternal",
    "Midnight", "Amber", "Pale", "Twisted", "Merry", "Reluctant",
    "Notorious", "Eccentric", "Glorious", "Dubious", "Unsanctioned",
    "Sovereign", "Wretched", "Illustrious", "Rogue", "Vigilant",
]

FACTION_NOUNS_FANTASY = [
    "Bakers", "Smugglers", "Healers", "Scholars", "Miners", "Sailors",
    "Brewers", "Weavers", "Scribes", "Cartographers", "Hunters",
    "Fishers", "Tinkers", "Gamblers", "Gravediggers", "Candlemakers",
    "Rat Catchers", "Taxidermists", "Tooth Pullers", "Inkmakers",
    "Spice Merchants", "Dream Sellers", "Debt Collectors", "Herbalists",
]
FACTION_NOUNS_SCIFI = [
    "Recyclers", "Miners", "Data Runners", "Pilots", "Engineers",
    "Medics", "Scouts", "Hackers", "Smugglers", "Clone Workers",
    "Scrappers", "Signal Trackers", "Fuel Merchants", "Body Modders",
    "Memory Brokers", "Drone Wranglers", "Zero-G Welders",
]

FACTION_PURPOSES = {
    "fantasy": [
        "control the {noun} trade in the region",
        "protect their members from {noun} exploitation",
        "maintain a monopoly on {noun} licensing",
        "gather intelligence and sell it to the highest bidder",
        "collect debts — violently if necessary",
        "provide loans at interest rates that should be illegal",
        "run the only reliable courier service in the settlement",
        "enforce quality standards — at swordpoint",
        "smuggle contraband past the garrison",
        "worship something the local temple considers heresy",
    ],
    "scifi": [
        "control access to {noun} in this sector",
        "run unlicensed medical procedures at a fraction of legal cost",
        "provide black market data storage and transmission",
        "operate a fleet of unregistered salvage vessels",
        "traffic in stolen corporate intellectual property",
        "sell counterfeit identification and transit papers",
        "provide protection for a fee — and collect on time",
        "broker deals between factions that officially hate each other",
        "run an underground fight circuit with significant betting pools",
        "operate the only functioning FTL relay in fifty kilometers",
    ],
}

SIDE_BUSINESSES = [
    "lingerie and intimate apparel", "exotic pet breeding",
    "artisanal pickles", "unlicensed dentistry",
    "competitive eating tournaments", "hat restoration",
    "erotic sculpture", "rat racing",
    "bootleg theatrical performances", "dream interpretation",
    "handwritten love letters (for a fee)", "competitive knife sharpening",
    "unsanctioned taxidermy", "underground cheese aging",
    "illicit cartography of places that shouldn't exist",
    "very small boat racing", "memorial portrait miniatures",
    "curse removal (results not guaranteed)", "gossip brokerage",
    "secondhand prosthetics", "competitive pie judging",
    "anonymous confession booth services", "bespoke poison blending",
    "illegal fireworks", "discreet corpse disposal",
    "forged academic credentials", "rental of expensive-looking outfits",
    "mood-altering candles", "astrology for livestock",
]

FACTION_DISPOSITIONS = [
    "Friendly — will trade information for coin",
    "Neutral — business is business",
    "Suspicious of outsiders — prove yourself first",
    "Hostile — you've stepped into their territory",
    "Recruiting — badly",
    "Desperate — something has gone very wrong for them recently",
    "Expansionist — actively absorbing smaller operations",
    "Secretive — their existence is an open secret nobody confirms",
]

def generate_factions(biome: str, genre: str, count: int = 2) -> list[dict]:
    org_types = ORG_TYPES_SCIFI if genre == "scifi" else ORG_TYPES_FANTASY
    nouns = FACTION_NOUNS_SCIFI if genre == "scifi" else FACTION_NOUNS_FANTASY
    purposes = FACTION_PURPOSES[genre]

    factions = []
    used_nouns = set()
    for _ in range(count * 5):
        if len(factions) >= count:
            break
        noun = random.choice(nouns)
        if noun in used_nouns:
            continue
        used_nouns.add(noun)
        adj = random.choice(FACTION_ADJECTIVES)
        org = random.choice(org_types)
        name = f"The {adj} {noun} {org}"
        purpose = random.choice(purposes).format(noun=noun.lower())
        side = random.choice(SIDE_BUSINESSES)
        disposition = random.choice(FACTION_DISPOSITIONS)
        factions.append({
            "name": name, "purpose": purpose,
            "side": side, "disposition": disposition,
        })
    return factions


# ─── Core Logic ───────────────────────────────────────────────────────────────

def detect_size(description: str) -> str:
    desc = description.lower()
    for keyword, size in SIZE_KEYWORDS.items():
        if keyword in desc:
            return size
    return "town"  # default


def detect_biome(description: str) -> str:
    desc = description.lower()
    biomes = ["sky", "coastal", "coast", "underground", "cave", "forest", "jungle",
              "desert", "mountain", "swamp", "marsh", "island", "plains", "tundra",
              "arctic", "volcanic", "volcano", "ruins", "ruin"]
    biome_map = {
        "coast": "coastal", "jungle": "forest", "cave": "underground",
        "marsh": "swamp", "arctic": "tundra", "volcano": "volcanic", "ruin": "ruins",
    }
    for b in biomes:
        if b in desc:
            return biome_map.get(b, b)
    return "default"


# ─── Wealth-weighted economy ──────────────────────────────────────────────────
# Poorer/smaller settlements lean toward illicit, black-market, pirate-style
# trades; larger/richer ones lean toward clean, legitimate commerce.
SHADY_ECONOMIES = {
    "Smuggling", "Piracy", "Wrecking & Salvage", "Fencing Stolen Cargo",
    "Black Market Goods", "Slaving", "Tomb Raiding", "Cursed Artifact Brokerage",
    "Relic Salvage", "Scrap Metal", "Scorpion Venom Harvest", "Mercenary Hosting",
    "Will-o-Wisp Bottling", "Contraband Transit", "Arms Dealing", "Black Market Tech",
    "Unlicensed Medical Services", "Clone Labor Contracting",
}

# Probability the PRIMARY economy is an illicit one, by settlement size.
SHADY_BIAS = {
    "hamlet":     0.78,
    "village":    0.62,
    "town":       0.42,
    "city":       0.22,
    "metropolis": 0.10,
}


def pick_economies(pool: list, size: str) -> tuple:
    shady = [e for e in pool if e in SHADY_ECONOMIES]
    clean = [e for e in pool if e not in SHADY_ECONOMIES]
    bias = SHADY_BIAS.get(size, 0.4)

    def weighted_pick(options):
        want_shady = shady and random.random() < bias
        bucket = shady if (want_shady and shady) else (clean if clean else shady)
        bucket = [e for e in bucket if e in options] or options
        return random.choice(bucket)

    primary = weighted_pick(pool)
    rest = [e for e in pool if e != primary]
    secondary = weighted_pick(rest) if rest else ""
    return primary, secondary


def generate_population(size: str) -> int:
    lo, hi = SIZE_BANDS[size]
    # Skew toward lower end (most settlements are small)
    raw = random.triangular(lo, hi, lo + (hi - lo) * 0.3)
    return int(raw)


def count_buildings(pop: int, ratios: dict) -> dict[str, int]:
    result = {}
    for name, (lo, hi) in ratios.items():
        rate = random.uniform(lo, hi)
        count = max(0, round(pop * rate / 100))
        if count > 0:
            result[name] = count
    return result


def generate(name: str, description: str) -> str:
    size = detect_size(description)
    biome = detect_biome(description)
    genre = detect_genre(description)

    pop = generate_population(size)

    # Economy — scifi genre overrides biome economy
    if genre == "scifi":
        pool = ECONOMIES["scifi"]
    else:
        pool = ECONOMIES.get(biome, ECONOMIES["default"])
    primary, secondary = pick_economies(pool, size)

    # Fun fact
    fun_fact = random.choice(FUN_FACTS)

    # Security
    if genre == "scifi":
        wall = random.choice(WALLS_SCIFI[size])
        fort = random.choice(FORTIFICATIONS_SCIFI[size])
    else:
        wall = random.choice(WALLS[size])
        fort = random.choice(FORTIFICATIONS[size])
    guards = max(1, round(pop * GUARD_RATE[size]))

    # Buildings
    if genre == "scifi":
        buildings = count_buildings(pop, BUILDING_RATIOS_SCIFI)
    else:
        buildings = count_buildings(pop, BUILDING_RATIOS)
        extra = EXTRA_BUILDINGS.get(biome, {})
        buildings.update(count_buildings(pop, extra))

    # NPCs & Factions
    npc_count = 2 if pop < 500 else 3 if pop < 10000 else 4
    npcs = generate_npcs(biome, genre, npc_count)
    faction_count = 1 if pop < 300 else 2 if pop < 8000 else 3
    factions = generate_factions(biome, genre, faction_count)

    # ─── Format Output ────────────────────────────────────────────────────────

    lines = [
        f"╔══════════════════════════════════════════╗",
        f"  {name}",
        f"  {description.title()}",
        f"╚══════════════════════════════════════════╝",
        "",
        f"  Population:    {pop:,}",
        f"  Economy:       {primary}",
    ]
    if secondary:
        lines.append(f"  Secondary:     {secondary}")
    lines += [
        f"  Fun Fact:      {fun_fact}",
        "",
        "  ── Security ─────────────────────────────",
        f"  Walls:         {wall}",
        f"  Fortification: {fort}",
        f"  Guard Force:   {guards:,}",
        "",
        "  ── Buildings ────────────────────────────",
    ]
    for building, count in sorted(buildings.items()):
        lines.append(f"  {building:<22} {count}")

    lines += ["", "  ── Notable NPCs ─────────────────────────"]
    for npc_name, role, quirk in npcs:
        lines.append(f"  {npc_name} — {role}")
        lines.append(f"    → {quirk}")

    lines += ["", "  ── Factions ─────────────────────────────"]
    for f in factions:
        lines.append(f"  {f['name']}")
        lines.append(f"    Purpose:  {f['purpose'].capitalize()}")
        lines.append(f"    Side biz: {f['side'].capitalize()}")
        lines.append(f"    Attitude: {f['disposition']}")

    return "\n".join(lines)


# ─── Entry Point ──────────────────────────────────────────────────────────────

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: generate_settlement.py <name> [description]")
        sys.exit(1)

    name = sys.argv[1]
    description = " ".join(sys.argv[2:]) if len(sys.argv) > 2 else "town"
    print(generate(name, description))
