"""Explicit source-deck selections; not proof of a released mask-selection pool."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path


REVISION = "fd978d6ef09f941fba644c731ab54abd6f7c3568"
DECKS = {
    101: ("gladiator", (1005, 1005, 1006, 1007, 1007, 1008, 1009, 1009, 1010, 1011, 1017, 1020, 1025, 1023)),
    102: ("camera", (1005, 1005, 1006, 1007, 1007, 1008, 1009, 1010, 1011, 1017, 1020, 1021, 1022)),
    103: ("mechatron", (1005, 1005, 1005, 1023, 1006, 1007, 1007, 1007, 1021, 1008, 1008, 1008, 1024, 1009, 1010, 1011, 1017, 1020)),
    104: ("horse", (1005, 1005, 1023, 1006, 1007, 1007, 1021, 1008, 1009, 1010, 1011, 1017, 1020)),
    105: ("fortune-cat", (1005, 1005, 1006, 1007, 1007, 1008, 1008, 1009, 1026, 1010, 1011, 1017, 1020, 1024)),
    106: ("cringe", (1005, 1005, 1023, 1006, 1007, 1007, 1021, 1008, 1009, 1009, 1010, 1011, 1017, 1017, 1020)),
    107: ("slipshod", (1005, 1006, 1007, 1007, 1008, 1009, 1010, 1011, 1017, 1020, 1021, 1023)),
    108: ("chariot", (1005, 1005, 1023, 1006, 1007, 1007, 1008, 1009, 1010, 1011, 1017, 1020, 1025)),
    110: ("two", (1005, 1005, 1005, 1006, 1007, 1007, 1007, 1008, 1009, 1010, 1011, 1017, 1020)),
}
PRESETS = {
    1005: (3, "Battle", 1), 1006: (4, "Encounter", 1),
    1007: (5, "Event", 1), 1008: (6, "Coin", 1),
    1009: (7, "Shop", 1), 1010: (8, "Reward", 1),
    1011: (9, "Adventure", 1), 1017: (21, "Reforge", 1),
    1020: (2, "Elite", 1), 1021: (5, "Event", 2),
    1022: (4, "Encounter", 2), 1023: (3, "Battle", 2),
    1024: (6, "Coin", 2), 1025: (2, "Elite", 2),
    1026: (7, "Shop", 2),
}
HASHES = {
    "RoguePersonaStyle": "49b1c63d1f9dcad5201ae6f6446c01b9f401506b5625c39fc4da4b62378c6834",
    "RoguePersonaRoomPreset": "a4cc8bbd6e3a4db5fecb62b83a0c5de7ad2f4a9820ad5ae9180b30ae33c346b5",
    "RoguePersonaRoomCompType": "c0223603c6e5252278dac5224d766f1c4ed60ebe5845b9839b9e3533a8ad76a5",
}
NOTE = (
    "VersionedProjectPolicyExplicitSourceDeck: caller selects this reviewed source record; "
    "interpret JEHDKAKMCGC as an ordered initial deck and repeated presets as distinct card instances. "
    "Interpret preset AAGKEBFHLMC as level. Exact numeric joins do not prove playable mask eligibility, "
    "unlock conditions, initial offer weights, draw width, pass costs or room behavior. "
    "No automatic profile membership or terminal runtime coverage is granted."
)
REPLACEMENT = (
    "Replace field-role inference and caller-selected admission when released named selectors or "
    "reproducible observations establish initial decks and eligibility. Retain duplicate instances "
    "and exact source order. Style 901 and all other unselected records remain pending, not excluded."
)


def append_domain_decks(data: dict[str, list[list[object]]]) -> None:
    for source_id, (name, digest) in enumerate(HASHES.items(), 67):
        data["Sources"].append([
            source_id, f"du.source.domain-deck.{name.lower()}",
            "https://gitlab.com/Dimbreath/turnbasedgamedata", REVISION, "4.4", "2026-09-19",
            f"ExcelOutput/{name}.json; exact keys retained per deck/card row", digest,
            "ExactStructured", "Exact source values only; field roles and runtime admission remain explicit policy.",
        ])
    decks, cards = [], []
    for deck_id, (source_id, (slug, presets)) in enumerate(DECKS.items(), 1):
        key = f"du.domain-deck.{slug}"
        decks.append([deck_id, key, str(source_id), "ExplicitSourceDeck", len(presets), NOTE, REPLACEMENT,
                      f"RoguePersonaStyle:KLOEJIMMPJM={source_id};JEHDKAKMCGC", "67|68|69"])
        for ordinal, preset in enumerate(presets, 1):
            _, kind, level = PRESETS[preset]
            cards.append([len(cards) + 1, f"{key}.card-{ordinal}", deck_id, ordinal,
                          str(preset), kind, level,
                          f"RoguePersonaStyle:KLOEJIMMPJM={source_id};JEHDKAKMCGC[{ordinal - 1}];RoguePersonaRoomPreset:LIIPLGLNPGB={preset}",
                          "67|68|69"])
    data["DomainDecks"], data["DomainCards"] = decks, cards


def verify_tables(tables: dict) -> None:
    styles = {row["KLOEJIMMPJM"]: row for row in tables["RoguePersonaStyle"]}
    presets = {row["LIIPLGLNPGB"]: row for row in tables["RoguePersonaRoomPreset"]}
    types = {row["LLICIMBCNPF"]: row["LHLKJIDFLIN"] for row in tables["RoguePersonaRoomCompType"]}
    for source_id, (_, expected) in DECKS.items():
        if tuple(styles[source_id]["JEHDKAKMCGC"]) != expected:
            raise ValueError(f"source deck order or multiplicity differs: {source_id}")
    if {preset for _, cards in DECKS.values() for preset in cards} != set(PRESETS):
        raise ValueError("selected preset closure differs")
    for source_id, (kind, name, level) in PRESETS.items():
        row = presets[source_id]
        if (row["LLICIMBCNPF"], types[row["LLICIMBCNPF"]], row["AAGKEBFHLMC"]) != (kind, name, level) or row["FJIKMHCJMKH"]:
            raise ValueError(f"source preset differs or gained unreviewed attributes: {source_id}")


def verify_source(root: Path) -> None:
    tables = {}
    for name, digest in HASHES.items():
        raw = (root / "ExcelOutput" / f"{name}.json").read_bytes()
        if hashlib.sha256(raw).hexdigest() != digest:
            raise ValueError(f"unadmitted source bytes: {name}")
        tables[name] = json.loads(raw)
    verify_tables(tables)
    print(f"Verified {len(DECKS)} explicit source decks, {sum(len(cards) for _, cards in DECKS.values())} instances and {len(PRESETS)} presets; no selector or gameplay credit.")


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--verify-source", type=Path, required=True)
    verify_source(parser.parse_args().verify_source)
