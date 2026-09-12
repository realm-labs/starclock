"""Current row-level Persona source audit, not executable or promoted content.

The owning generator preserves exact integer JSON values and hashes admitted
raw bytes. No source descriptions, assets or ability programs are exported.
"""

from __future__ import annotations

import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
OUTPUT = ROOT / "content-manifests/divergent-universe-runtime-v1/persona-reachability.json"
REVISION = "fd978d6ef09f941fba644c731ab54abd6f7c3568"
KEYS = {
    "RoguePersonaConstClient": ("ConstValueName",),
    "RoguePersonaConstCommon": ("ConstValueName",),
    "RoguePersonaLayerRoom": ("CBCHIHEOEGK", "EEPIDJJJMAH"),
    "RoguePersonaRoomAttribute": ("HHPFKDEBMGP",),
    "RoguePersonaRoomCompType": ("LLICIMBCNPF",),
    "RoguePersonaRoomComposition": ("LLICIMBCNPF", "AAGKEBFHLMC"),
    "RoguePersonaRoomPreset": ("LIIPLGLNPGB",),
    "RoguePersonaStyle": ("KLOEJIMMPJM",),
    "RoguePersonaStyleGift": ("FMDMDDCBPAM",),
    "RoguePersonaTalent": ("PHFMCACHFIJ",),
    "RoguePersonaTalentGroup": ("PHFMCACHFIJ",),
}


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def encoded(value: object) -> str:
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":"))


def load_sources(source_root: Path) -> tuple[dict, dict]:
    inventory = json.loads((ROOT / "content-manifests/divergent-universe-v1/source-inventory.json").read_text(encoding="utf-8"))
    repositories = {entry["id"]: entry for entry in inventory["snapshot"]["repositories"]}
    if repositories["turnbasedgamedata"]["revision"] != REVISION:
        raise ValueError("source revision admission mismatch")
    sources = {record["path"]: record for record in inventory["records"]
               if record["repository"] == "turnbasedgamedata"}
    actual_names = {Path(name).stem for name in sources
                    if name.startswith("ExcelOutput/RoguePersona")}
    if actual_names != set(KEYS):
        raise ValueError("Persona file closure changed; review each added or missing table")
    tables, hashes = {}, {}
    for name in ["RogueTournArea", *KEYS]:
        file = f"ExcelOutput/{name}.json"
        raw = (source_root / file).read_bytes()
        record = sources[file]
        if digest(raw) != record["sha256"]:
            raise ValueError(f"source admission mismatch: {file}")
        tables[name] = json.loads(raw)
        hashes[name] = digest(raw)
    return tables, hashes


def audit_rows(tables: dict) -> dict:
    indices, records = {}, {}
    for name, fields in KEYS.items():
        indices[name] = {}
        for index, row in enumerate(tables[name]):
            key = tuple(row[field] for field in fields)
            if key in indices[name]:
                raise ValueError(f"duplicate source key: {name}/{key}")
            locator = f"ExcelOutput/{name}.json#{index}"
            indices[name][key] = locator
            records[locator] = {
                "source": locator,
                "source_key": {field: row[field] for field in fields},
                "row_sha256": digest(encoded(row).encode("utf-8")),
                "reachability": "PendingSelectorProof",
                "parent_sources": [],
            }

    def select(name: str, key: tuple, parent: str) -> str:
        locator = indices[name].get(key)
        if locator is None:
            raise ValueError(f"dangling reviewed source join: {name}/{key}")
        record = records[locator]
        record["reachability"] = "CurrentLayerReference"
        if parent not in record["parent_sources"]:
            record["parent_sources"].append(parent)
        return locator

    areas = [(index, row) for index, row in enumerate(tables["RogueTournArea"])
             if row.get("HILINOJPLGA") == "Tourn3"]
    current_layers = {layer for _, area in areas for layer in area["GLNDIILFKBN"]}
    positions = tables["RoguePersonaLayerRoom"]
    for layer in current_layers:
        matching = [row for row in positions if row["CBCHIHEOEGK"] == layer]
        ordinals = sorted(row["EEPIDJJJMAH"] for row in matching)
        if not ordinals or ordinals != list(range(1, len(ordinals) + 1)):
            raise ValueError(f"current layer missing or noncontiguous: {layer}")
    presets = {row["LIIPLGLNPGB"]: row for row in tables["RoguePersonaRoomPreset"]}
    for area_index, area in areas:
        for row in positions:
            if row["CBCHIHEOEGK"] not in area["GLNDIILFKBN"]:
                continue
            position = select("RoguePersonaLayerRoom", (row["CBCHIHEOEGK"], row["EEPIDJJJMAH"]),
                              f"ExcelOutput/RogueTournArea.json#{area_index}")
            if "BKHDBIFFIKP" not in row:
                continue
            preset_key = row["BKHDBIFFIKP"]
            preset = select("RoguePersonaRoomPreset", (preset_key,), position)
            definition = presets[preset_key]
            kind, level = definition["LLICIMBCNPF"], definition["AAGKEBFHLMC"]
            select("RoguePersonaRoomCompType", (kind,), preset)
            select("RoguePersonaRoomComposition", (kind, level), preset)
            # Attribute semantics and their selector require separate review.
            if definition["FJIKMHCJMKH"]:
                raise ValueError("current fixed preset gained unreviewed attributes")

    for record in records.values():
        record["parent_sources"].sort()
    counts = Counter(record["reachability"] for record in records.values())
    # The area field also contains 501/601/701/801, absent from the style keys.
    # Matching the single integer 901 is therefore not a reviewed style join.
    style_keys = {row["KLOEJIMMPJM"] for row in tables["RoguePersonaStyle"]}
    area_values = {area["ILPNADCAIBL"] for _, area in areas if "ILPNADCAIBL" in area}
    return {
        "summary": {"source_tables": len(KEYS), "source_rows": len(records),
                    "current_areas": len(areas), "current_layers": len(current_layers),
                    "current_layer_reference_rows": counts["CurrentLayerReference"],
                    "pending_selector_rows": counts["PendingSelectorProof"]},
        "rejected_shortcuts": {
            "area_ILPNADCAIBL_is_style_selector": {
                "admitted": False, "area_values": sorted(area_values),
                "missing_style_keys": sorted(area_values - style_keys),
                "reason": "A partial integer-key overlap is not a reviewed field relationship.",
            },
            "persona_prefix_is_other_mode": False,
            "unselected_rows_are_excluded": False,
        },
        "tables": [
            {"source": f"ExcelOutput/{name}.json", "rows": len(tables[name]),
             "current_layer_reference_rows": sum(
                 records[locator]["reachability"] == "CurrentLayerReference"
                 for locator in indices[name].values())}
            for name in KEYS
        ],
        "rows": list(records.values()),
    }


def build(source_root: Path) -> dict:
    tables, hashes = load_sources(source_root)
    return {
        "generated_by": "tools/divergent-universe-runtime/persona_reachability.py",
        "source_revision": REVISION, "game_version": "4.4", "access_date": "2026-09-12",
        "source_repository": "https://gitlab.com/Dimbreath/turnbasedgamedata",
        "source_file_sha256": hashes,
        "row_hash_codec": "UTF-8 Python JSON; sorted keys; compact separators; exact integers",
        "boundary": "Source reference closure only. Obfuscated field roles are reviewed inference. No room execution, selector probability, style eligibility or runtime disposition is implied.",
        "reference_package_promotion_complete": False,
        "runtime_promotion_complete": False,
        **audit_rows(tables),
    }


def reconciliation_defects(report: dict, manifest: dict) -> list[str]:
    """Require row accounting without accepting a blanket other-mode exclusion."""
    exclusions = manifest["exclusions"]
    defects = []
    if "RoguePersona" in exclusions["mode_prefixes"]:
        defects.append("RoguePersona is still blanket-excluded")
    excluded_files = {record["source"] for record in exclusions["named_mode_source_files"]}
    proven = {record["source"] for record in report["rows"]
              if record["reachability"] == "CurrentLayerReference"}
    conflicts = {source.split("#")[0] for source in proven} & excluded_files
    if conflicts:
        defects.append(f"{len(conflicts)} current-reachable source files are excluded")
    admitted = Counter(record["source"] for category in manifest["categories"].values()
                       for record in category["records"])
    explicit_exclusions = {record["source"] for record in exclusions["historical_rows"]
                           if record.get("reason") and record.get("reachability") == "Excluded"}
    duplicate = [record["source"] for record in report["rows"]
                 if admitted[record["source"]] + (record["source"] in explicit_exclusions) > 1]
    if duplicate:
        defects.append(f"{len(duplicate)} Persona source rows have duplicate manifest accounting")
    if proven & explicit_exclusions:
        defects.append("current layer-reachable rows have contradictory explicit exclusions")
    missing = [record["source"] for record in report["rows"]
               if record["source"] not in admitted and record["source"] not in explicit_exclusions]
    if missing:
        defects.append(f"{len(missing)} Persona source rows lack manifest accounting")
    return defects


def promotion_defects(report: dict, coverage: list[dict]) -> list[str]:
    """Manifest membership must also reach normalized accounting before release."""
    covered = Counter(row["source_locator"] for row in coverage)
    missing = sum(covered[row["source"]] == 0 for row in report["rows"])
    duplicates = sum(covered[row["source"]] > 1 for row in report["rows"])
    defects = []
    if missing:
        defects.append(f"{missing} Persona source rows lack normalized coverage")
    if duplicates:
        defects.append(f"{duplicates} Persona source rows have duplicate normalized coverage")
    return defects


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source-cache", type=Path, default=ROOT / ".cache/content-reference/turnbasedgamedata")
    parser.add_argument("--check", action="store_true")
    parser.add_argument("--require-reconciled", action="store_true",
                        help="release gate: fail if the current reference manifest contradicts or omits these rows")
    parser.add_argument("--require-promoted", action="store_true",
                        help="also require exact-once normalized reference coverage; this does not prove runtime execution")
    args = parser.parse_args()
    result = build(args.source_cache)
    text = json.dumps(result, ensure_ascii=False, indent=2) + "\n"
    if args.check:
        if OUTPUT.read_text(encoding="utf-8") != text:
            raise ValueError("Persona reachability generated drift")
    else:
        OUTPUT.write_text(text, encoding="utf-8", newline="\n")
    print(json.dumps(result["summary"], sort_keys=True))
    if args.require_reconciled or args.require_promoted:
        manifest = json.loads((ROOT / "content-manifests/divergent-universe-v1/content-manifest.json").read_text(encoding="utf-8"))
        defects = reconciliation_defects(result, manifest)
        if defects:
            raise SystemExit("Persona reference reconciliation incomplete: " + "; ".join(defects))
    if args.require_promoted:
        coverage = json.loads((ROOT / "content-reference/divergent-universe-v1/coverage.json").read_text(encoding="utf-8"))
        defects = promotion_defects(result, coverage)
        if defects:
            raise SystemExit("Persona reference promotion incomplete: " + "; ".join(defects))


if __name__ == "__main__":
    main()
