"""Verify one Goal 07 world-structure partition in authoritative Excel and Sora."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
from typing import Any

from openpyxl import load_workbook
from openpyxl.cell.cell import TYPE_ERROR, TYPE_FORMULA

ROOT = Path(__file__).resolve().parents[2]
GOAL = "standard-universe-mechanics-complete-v1"
MANIFEST = ROOT / "content-manifests" / GOAL / "content-partitions.json"
REFERENCE = ROOT / "content-reference" / "standard-universe-v1"
DATA = ROOT / "config" / "data"
DEBUG = ROOT / "config" / "universe-generated" / "debug-json"

DOMAIN_KINDS = {
    "combat-primary": "CombatPrimary",
    "combat-secondary": "CombatSecondary",
    "occurrence": "Occurrence",
    "encounter": "Encounter",
    "respite": "Respite",
    "elite": "Elite",
    "boss": "Boss",
    "transaction": "Transaction",
    "adventure": "Adventure",
}


def rows(sheet: Any) -> list[dict[str, Any]]:
    fields = {
        str(cell.value): cell.column
        for cell in sheet[3]
        if cell.value not in (None, "#field")
    }
    output: list[dict[str, Any]] = []
    for cells in sheet.iter_rows(min_row=8):
        row = {field: cells[column - 1].value for field, column in fields.items()}
        if all(value is None for value in row.values()):
            continue
        if any(
            cells[column - 1].data_type in (TYPE_ERROR, TYPE_FORMULA)
            for column in fields.values()
        ):
            raise ValueError(f"{sheet.title}: formulas and errors are forbidden")
        output.append(row)
    return output


def keyed(values: list[dict[str, Any]], field: str) -> dict[Any, dict[str, Any]]:
    output: dict[Any, dict[str, Any]] = {}
    for row in values:
        if row[field] in output:
            raise ValueError(f"duplicate {field}={row[field]}")
        output[row[field]] = row
    return output


def unwrap(value: Any) -> Any:
    if value == "Null":
        return None
    if isinstance(value, list):
        return [unwrap(item) for item in value]
    if isinstance(value, dict):
        if len(value) == 1:
            tag, inner = next(iter(value.items()))
            if tag in {"String", "Integer", "Bool", "Decimal"}:
                return inner
            if tag == "List":
                return [unwrap(item) for item in inner]
        return {key: unwrap(inner) for key, inner in value.items()}
    return value


def sora_rows(table: str) -> list[dict[str, Any]]:
    payload = json.loads((DEBUG / f"{table}.json").read_text(encoding="utf-8"))
    return [
        {key: unwrap(value) for key, value in row["values"].items()}
        for row in payload["table"]["rows"]
    ]


def digest(values: list[dict[str, Any]]) -> str:
    ordered = sorted(
        values,
        key=lambda row: json.dumps(
            row, ensure_ascii=False, sort_keys=True, default=str
        ),
    )
    encoded = json.dumps(
        ordered,
        ensure_ascii=False,
        sort_keys=True,
        separators=(",", ":"),
        default=str,
    )
    return hashlib.sha256(encoded.encode()).hexdigest()


def check_source_domain(reference: dict[str, Any], authored: dict[str, Any]) -> None:
    expected = {
        "stable_key": reference["id"],
        "source_type": int(reference["source_ids"][0]),
        "kind": DOMAIN_KINDS[reference["kind"]],
        "decision_policy": reference["decision_policy"],
        "terminal": reference["terminal"],
        "name_en": reference["name_en"],
        "name_zh_cn": reference["name_zh_cn"],
        "summary_en": reference["summary_en"],
        "summary_zh_cn": reference["summary_zh_cn"],
    }
    actual = {field: authored[field] for field in expected}
    if actual != expected:
        raise ValueError(f"{reference['id']}: Excel domain differs from public source")


def build_domain_partition(
    partition: dict[str, Any],
    universe: Any,
    bindings: Any,
    evidence: Any,
) -> dict[str, list[dict[str, Any]]]:
    source = keyed(
        json.loads((REFERENCE / "domains.json").read_text(encoding="utf-8")),
        "id",
    )
    domains = keyed(rows(universe["UniverseDomain"]), "stable_key")
    selected_domains = [domains[key] for key in partition["record_ids"]]
    for row in selected_domains:
        check_source_domain(source[row["stable_key"]], row)

    domain_ids = {int(row["id"]) for row in selected_domains}
    selected_bindings = [
        row
        for row in rows(bindings["UniverseActivityDomainBinding"])
        if int(row["domain_id"]) in domain_ids
    ]
    if len(selected_bindings) != len(selected_domains):
        raise ValueError(
            f"{partition['id']}: Activity bindings do not exactly cover the domains"
        )
    by_domain = keyed(selected_bindings, "domain_id")
    for row in selected_domains:
        source_type = int(row["source_type"])
        binding = by_domain[int(row["id"])]
        expected_decision = (
            "BattleCommand"
            if row["decision_policy"] == "BattleHandoff"
            else "ExternalOutcome"
            if row["kind"] == "Adventure"
            else "RunCommand"
        )
        if int(binding["sequence"]) != source_type:
            raise ValueError(f"{row['stable_key']}: Activity sequence differs")
        if binding["decision_kind"] != expected_decision:
            raise ValueError(f"{row['stable_key']}: Activity decision differs")

    audits = keyed(rows(evidence["UniverseContentAudit"]), "content_stable_key")
    sources = keyed(rows(evidence["UniverseSourceRecord"]), "id")
    for key in partition["record_ids"]:
        audit = audits.get(key)
        provenance = [] if audit is None else str(audit["provenance_ids"]).split("|")
        if (
            audit is None
            or not audit["enabled"]
            or audit["mode_owner"] != "Standard"
            or audit["quality"] != "ExactStructured"
            or audit["mechanism_quality"] != "ExactStructured"
            or audit["coverage_state"] != "DataReady"
            or not provenance
            or any(int(item) not in sources for item in provenance)
        ):
            raise ValueError(f"{key}: exact enabled provenance audit is incomplete")

    return {
        "UniverseDomain": selected_domains,
        "UniverseActivityDomainBinding": selected_bindings,
    }


def build(partition_id: str) -> dict[str, Any]:
    manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
    partition = next(
        (value for value in manifest["partitions"] if value["id"] == partition_id),
        None,
    )
    if (
        partition is None
        or partition["mechanic_family"]
        != "enemies-encounters-worlds-difficulty-carry"
        or partition["lane"] != "domain-graph"
    ):
        raise ValueError(f"{partition_id}: world author currently supports domain-graph")

    universe = load_workbook(DATA / "Universe.xlsx", read_only=True, data_only=False)
    bindings = load_workbook(
        DATA / "UniverseBindings.xlsx", read_only=True, data_only=False
    )
    evidence = load_workbook(
        DATA / "UniverseEvidence.xlsx", read_only=True, data_only=False
    )
    selected = build_domain_partition(partition, universe, bindings, evidence)
    selected_ids = {
        int(row["id"]) for row in selected["UniverseDomain"]
    }
    for table, selected_rows in selected.items():
        exported = sora_rows(table)
        if table == "UniverseDomain":
            keys = {row["stable_key"] for row in selected_rows}
            exported = [row for row in exported if row.get("stable_key") in keys]
        else:
            exported = [
                row for row in exported if int(row.get("domain_id", -1)) in selected_ids
            ]
        if digest(selected_rows) != digest(exported):
            raise ValueError(f"{partition_id}: Sora {table} rows differ from Excel")

    return {
        "schema_revision": "starclock.goal07-world-partition-golden.v1",
        "partition_id": partition_id,
        "lane": partition["lane"],
        "record_ids": partition["record_ids"],
        "rule_ids": partition["rule_ids"],
        "fixture_ids": partition["fixture_ids"],
        "tables": {
            table: {"rows": len(values), "semantic_sha256": digest(values)}
            for table, values in selected.items()
        },
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--partition", required=True)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--check", action="store_true")
    mode.add_argument("--write", action="store_true")
    args = parser.parse_args()
    target = ROOT / "evidence" / GOAL / "goldens" / f"{args.partition}.json"
    encoded = json.dumps(build(args.partition), ensure_ascii=False, indent=2) + "\n"
    if args.write:
        target.parent.mkdir(parents=True, exist_ok=True)
        with target.open("w", encoding="utf-8", newline="\n") as stream:
            stream.write(encoded)
        print(f"Wrote {target.relative_to(ROOT)}.")
    elif not target.is_file() or target.read_text(encoding="utf-8") != encoded:
        raise ValueError(f"{args.partition}: world partition golden drifted")
    else:
        print(f"Goal 07 world partition {args.partition} matches Excel and Sora.")


if __name__ == "__main__":
    main()
