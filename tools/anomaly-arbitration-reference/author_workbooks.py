#!/usr/bin/env python3
"""Generate complete Goal 13 workbooks from the normalized reference pack."""

from __future__ import annotations

import argparse
import json
import sys
import tempfile
from pathlib import Path

SCRIPT_ROOT = Path(__file__).resolve().parent
sys.path.insert(0, str(SCRIPT_ROOT))

from workbook.common import WORKBOOKS, author, semantic_digest, sha256  # noqa: E402
from workbook.common import schema_tables, verify  # noqa: E402

PACK = Path("content-reference/anomaly-arbitration-v1")
CONTRACT = Path("content-manifests/anomaly-arbitration-v1/authoring-contract.json")
DIRECT_FIELDS = {
    "game_version",
    "source_group_id",
    "source_stage_id",
    "stage_kind",
    "difficulty",
    "slot_order",
    "source_numeric_id",
    "source_template_id",
    "binding_order",
    "wave_order",
    "skill_order",
    "scope",
    "install_order",
    "pool_family",
    "family_id",
    "source_id",
    "locator",
    "source_path",
    "row_locator",
    "peer_goal_id",
    "manifest_category",
    "manifest_record_id",
    "blocking",
    "owner_batch",
    "file_order",
    "record_order",
    "name_en",
    "name_zh_cn",
    "summary_en",
    "summary_zh_cn",
    "ownership",
    "coverage_state",
    "evidence_quality",
    "mechanism_quality",
    "runtime_executable",
}


def read_records(root: Path, file_name: str) -> list[dict]:
    payload = json.loads((root / PACK / file_name).read_text(encoding="utf-8"))
    records = payload["records"]
    if not isinstance(records, list):
        raise ValueError(f"{file_name}: records must be a list")
    return records


def canonical_json(value: object) -> str:
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":"))


def join_list(values: object) -> str:
    if not isinstance(values, list):
        raise ValueError(f"expected list, got {type(values).__name__}")
    return "|".join(str(value) for value in values)


def referenced_target_ids(row: dict, all_targets: list[str]) -> list[str]:
    direct = row.get("target_ids")
    if isinstance(direct, list) and direct:
        return direct
    ids = []
    for source in row.get("source_refs", []):
        source_id = source.get("source_id", "")
        marker = "BattleTargetConfig.json:ID="
        if marker in source_id:
            numeric = source_id.split(marker, 1)[1]
            stable = f"battle-target.{numeric}"
            if stable in all_targets and stable not in ids:
                ids.append(stable)
    return ids or all_targets


def stage_scope(row: dict, all_stages: list[str]) -> list[str]:
    direct = row.get("input_stage_ids") or row.get("stage_ids")
    if isinstance(direct, list) and direct:
        return direct
    kind = row.get("stage_kind")
    if kind == "Knight":
        return [value for value in all_stages if value.startswith("stage.knight-")]
    if kind == "KingNormal":
        return ["stage.king-normal"]
    if kind == "KingPlight":
        return ["stage.king-plight"]
    return all_stages


def type_value(field: dict, value: object) -> object:
    ty = field["ty"]
    container = ty.get("Optional") if isinstance(ty, dict) else None
    if isinstance(ty, dict) and (
        "List" in ty or isinstance(container, dict) and "List" in container
    ):
        return join_list(value)
    if ty == "String" and not isinstance(value, str):
        return str(value)
    if ty == "I32":
        return int(value)
    if ty == "Bool":
        if not isinstance(value, bool):
            raise ValueError(f"{field['name']}: expected bool")
        return value
    return value


def build_rows(root: Path) -> dict[str, list[dict[str, object]]]:
    contract = json.loads((root / CONTRACT).read_text(encoding="utf-8"))
    file_for_sheet: dict[str, str] = {}
    for workbook in contract["workbooks"]:
        tables = [
            table
            for table in schema_tables(root)
            if table["source"]["file"] == workbook["file"]
        ]
        sheets = [table["source"]["sheet"] for table in tables]
        normalized = workbook["normalized_files"]
        if len(sheets) != len(normalized):
            raise ValueError(f"{workbook['file']}: sheet/file count mismatch")
        file_for_sheet.update(dict(zip(sheets, normalized, strict=True)))

    records_by_sheet = {
        sheet: read_records(root, file_name)
        for sheet, file_name in file_for_sheet.items()
    }
    ids = {
        sheet: {row["id"]: index for index, row in enumerate(rows, start=1)}
        for sheet, rows in records_by_sheet.items()
    }
    all_stages = list(ids["Stages"])
    all_options = list(ids["QuadrantOptions"])
    all_targets = list(ids["Targets"])
    result: dict[str, list[dict[str, object]]] = {}

    for table in schema_tables(root):
        sheet = table["source"]["sheet"]
        authored_rows = []
        for row_order, source in enumerate(records_by_sheet[sheet], start=1):
            values: dict[str, object] = {}
            for field in table["fields"]:
                name = field["name"]
                value: object
                if name == "id":
                    value = row_order
                elif name == "stable_key":
                    value = source["id"]
                elif name == "row_order":
                    value = row_order
                elif name == "payload_json":
                    value = canonical_json(source)
                elif name == "manifest_record_ids":
                    value = source["manifest_record_ids"]
                elif name == "source_ref_ids":
                    value = [
                        reference["source_id"]
                        for reference in source["source_refs"]
                    ]
                elif name == "tags":
                    value = source.get("tags", [])
                elif name == "profile_id":
                    value = 1
                elif name == "game_version":
                    value = source.get("game_version", "4.4")
                elif name == "period_id":
                    value = ids["Periods"][source["period_id"]]
                elif name == "stage_id":
                    value = ids["Stages"][source["stage_id"]]
                elif name == "encounter_id":
                    value = ids["Encounters"][source["encounter_id"]]
                elif name == "enemy_id":
                    value = ids["Enemies"][source["enemy_id"]]
                elif name == "stage_order":
                    value = source.get("stage_order", source["display_order"])
                elif name == "stage_stable_key":
                    value = source["stage_id"]
                elif name == "stage_stable_keys":
                    value = stage_scope(source, all_stages)
                elif name == "option_stable_keys":
                    value = source.get("offered_option_ids") or all_options
                elif name == "target_stable_keys":
                    value = referenced_target_ids(source, all_targets)
                elif name == "owner_stable_key":
                    value = source.get("owner_id", source.get("enemy_id"))
                elif name == "evidence_ref_ids":
                    value = [
                        ids["Sources"][stable_key]
                        for stable_key in source["evidence_refs"]
                    ]
                elif name in DIRECT_FIELDS:
                    value = source[name]
                else:
                    raise ValueError(f"{sheet}/{source['id']}: no mapping for {name}")
                values[name] = type_value(field, value)
            authored_rows.append(values)
        result[sheet] = authored_rows
    return result


def compare_directory(expected: Path, actual: Path) -> None:
    mismatches = [
        name
        for name in WORKBOOKS
        if sha256(expected / name) != sha256(actual / name)
    ]
    if mismatches:
        raise ValueError(f"authored workbook drift: {', '.join(mismatches)}")


def check(root: Path, expected: Path) -> dict[str, int]:
    counts = verify(root, expected)
    with tempfile.TemporaryDirectory(prefix="starclock-g13-workbooks-") as temporary:
        generated = Path(temporary)
        author(root, generated, build_rows(root))
        compare_directory(expected, generated)
    return counts


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=Path, default=Path.cwd())
    parser.add_argument(
        "--output",
        type=Path,
        default=Path("config/anomaly-arbitration/data"),
    )
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    root = args.root.resolve()
    output = args.output if args.output.is_absolute() else root / args.output
    if args.check:
        counts = check(root, output)
        action = "Verified"
    else:
        counts = author(root, output, build_rows(root))
        action = "Authored"
    print(
        f"{action} {sum(counts.values())} rows across {len(counts)} sheets; "
        f"semantic digest {semantic_digest(output)}."
    )


if __name__ == "__main__":
    main()
