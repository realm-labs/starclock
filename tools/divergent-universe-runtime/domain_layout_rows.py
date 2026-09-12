"""Reviewed current Persona layout joins; no inferred random-room membership.

These independent integer facts reproduce the hash-bound source records. The
optional --verify-source check reads only the admitted released cache; normal
openpyxl authoring remains reproducible without a source checkout.
"""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path


LAYERS = {
    103: (9007, 0, 0, 0, 1002),
    104: (9008, 0, 0, 0, 1002),
    3001: (1001, 0, 0, 1002),
    3002: (0, 0, 0, 0, 1002),
    3003: (0, 0, 1003, 1002),
    3011: (1001, 0, 0, 0, 1002),
    3012: (0, 0, 0, 1004, 0, 0, 1002),
    3013: (0, 0, 0, 1003, 1002),
    3021: (1001, 0, 0, 0, 0, 1002),
    3022: (0, 0, 0, 1004, 0, 0, 0, 1002),
    3023: (0, 0, 0, 0, 1003, 1002),
}
PRESETS = {
    1001: (3, "Battle", 3),
    1002: (1, "Boss", 1),
    1003: (10, "Respite", 1),
    1004: (23, "Conversion", 1),
    9007: (22, "Blank", 1),
    9008: (6, "Coin", 1),
}
SOURCE_HASHES = {
    "RogueTournArea": "756510177a468130464c27ef382d9ab33a7098dc993022cd8647366cc7c46db8",
    "RoguePersonaLayerRoom": "651989d8fd339eec667791495e6ff608418f7d308990d02d941c698203648998",
    "RoguePersonaRoomPreset": "a4cc8bbd6e3a4db5fecb62b83a0c5de7ad2f4a9820ad5ae9180b30ae33c346b5",
    "RoguePersonaRoomCompType": "c0223603c6e5252278dac5224d766f1c4ed60ebe5845b9839b9e3533a8ad76a5",
}


def append_domain_layout(data: dict[str, list[list[object]]]) -> None:
    for source_id, name, key, locator in [
        (64, "RoguePersonaLayerRoom", "positions", "All rows; CBCHIHEOEGK joins the 11 current Tourn3 area GLNDIILFKBN layer IDs; EEPIDJJJMAH is contiguous within each layer; optional BKHDBIFFIKP joins preset LIIPLGLNPGB"),
        (65, "RoguePersonaRoomPreset", "presets", "LIIPLGLNPGB=1001,1002,1003,1004,9007,9008; LLICIMBCNPF joins RoomCompType; AAGKEBFHLMC joins composition level; all six FJIKMHCJMKH lists empty"),
        (66, "RoguePersonaRoomCompType", "types", "LLICIMBCNPF=1,3,6,10,22,23; LHLKJIDFLIN=Boss,Battle,Coin,Respite,Blank,Conversion respectively"),
    ]:
        data["Sources"].append([
            source_id, f"du.source.domain-layout.{key}",
            "https://gitlab.com/Dimbreath/turnbasedgamedata",
            "fd978d6ef09f941fba644c731ab54abd6f7c3568", "4.4", "2026-09-12",
            f"ExcelOutput/{name}.json; {locator}", SOURCE_HASHES[name],
            "ExactStructured",
            "Exact current numeric records and source enum joins. Obfuscated field interpretation is reviewed separately. Absent presets do not prove a random pool, draw cost, repetition rule, or executable room content.",
        ])
    rows = []
    for layer, presets in sorted(LAYERS.items()):
        for ordinal, preset in enumerate(presets, 1):
            _, kind, level = PRESETS[preset] if preset else (0, "Unspecified", None)
            rows.append([
                len(rows) + 1, f"du.domain-slot.layer-{layer}.position-{ordinal}",
                f"divergent-universe.layer.{layer}", ordinal,
                str(preset) if preset else None, kind, level,
                f"RoguePersonaLayerRoom:CBCHIHEOEGK={layer};EEPIDJJJMAH={ordinal};BKHDBIFFIKP={preset or 'absent'}",
                "ReviewedSourceJoin: interpret the exact area's ordered layer ID join as its position sequence, optional preset as a fixed composition, and preset AAGKEBFHLMC as composition level. This does not infer the missing slot's deck, candidate membership, pass costs, or room behavior. The legacy Tourn LayerRoom table is not the current selector.",
                "Replace obfuscated field interpretation independently if released named schemas or executable selectors contradict it. Preserve source values and exact profile reachability. Keep unspecified positions unresolved until an authored executable card-selection policy is bound; never fill them with proxy battles implicitly.",
                "18|64|65|66",
            ])
    data["DomainLayout"] = rows


def verify_source(root: Path) -> None:
    tables = {}
    for name, expected in SOURCE_HASHES.items():
        raw = (root / "ExcelOutput" / f"{name}.json").read_bytes()
        if hashlib.sha256(raw).hexdigest() != expected:
            raise ValueError(f"unadmitted released source bytes: {name}")
        tables[name] = json.loads(raw)
    areas = [row for row in tables["RogueTournArea"] if row.get("HILINOJPLGA") == "Tourn3"]
    reached = {layer for area in areas for layer in area["GLNDIILFKBN"]}
    if len(areas) != 28 or reached != set(LAYERS):
        raise ValueError("current profile/layer reachability differs")
    actual = {}
    for row in tables["RoguePersonaLayerRoom"]:
        key = (row["CBCHIHEOEGK"], row["EEPIDJJJMAH"])
        if key in actual:
            raise ValueError("duplicate position")
        actual[key] = row.get("BKHDBIFFIKP", 0)
    expected = {(layer, ordinal): preset for layer, presets in LAYERS.items()
                for ordinal, preset in enumerate(presets, 1)}
    if actual != expected:
        raise ValueError("current Persona positions differ")
    presets = {row["LIIPLGLNPGB"]: row for row in tables["RoguePersonaRoomPreset"]}
    types = {row["LLICIMBCNPF"]: row["LHLKJIDFLIN"] for row in tables["RoguePersonaRoomCompType"]}
    for key, (composition, kind, level) in PRESETS.items():
        row = presets[key]
        if (row["LLICIMBCNPF"], types[row["LLICIMBCNPF"]], row["AAGKEBFHLMC"]) != (composition, kind, level) or row["FJIKMHCJMKH"]:
            raise ValueError(f"current preset differs: {key}")
    print("Verified 28 current areas, 11 layers, 60 positions and 6 fixed presets against admitted released bytes.")


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--verify-source", type=Path, required=True)
    verify_source(parser.parse_args().verify_source)
