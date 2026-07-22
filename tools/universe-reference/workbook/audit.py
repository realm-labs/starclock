"""Source, coverage, review-fixture and pack-index workbook rows."""

from __future__ import annotations

from pathlib import Path

from workbook.data import canonical_json, joined, load_json, optional_joined, stable_ids


def build_rows(root: Path) -> dict[str, list[dict]]:
    sources = load_json(root, "sources.json")
    coverage = load_json(root, "coverage.json")
    fixtures = load_json(root, "review-fixtures.json")
    pack_index = load_json(root, "pack-index.json")
    source_ids = stable_ids(sources)
    fixture_ids = stable_ids(fixtures)
    content: list[tuple[str, dict]] = []
    for category in coverage["categories"]:
        content.extend((category["file"], record) for record in load_json(root, category["file"]))
    content_ids = stable_ids([record for _, record in content])
    rows: dict[str, list[dict]] = {
        "UniverseSourceRecord": [],
        "UniverseContentAudit": [],
        "UniverseCoverage": [],
        "UniverseReviewFixture": [],
        "UniversePackFile": [],
    }
    for record in sources:
        rows["UniverseSourceRecord"].append({
            "id": source_ids[record["id"]],
            "stable_key": record["id"],
            "source_kind": record["source_kind"],
            "repository_or_url": record["repository_or_url"],
            "revision_or_access_date": record["revision_or_access_date"],
            "game_version": record["game_version"],
            "relative_path_or_page": record["relative_path_or_page"],
            "row_locator": record["row_locator"],
            "evidence_sha256": record["evidence_sha256"],
            "quality": record["quality"],
            "license_note": record["license_note"],
            "note": record["note"],
        })
    for source_file, record in content:
        provenance = [source_ids[key] for key in record["provenance_ids"]]
        rows["UniverseContentAudit"].append({
            "id": content_ids[record["id"]],
            "content_stable_key": record["id"],
            "source_file": source_file,
            "enabled": record["enabled"],
            "mode_owner": record["mode_owner"],
            "quality": record["quality"],
            "mechanism_quality": record["mechanism_quality"],
            "coverage_state": record["coverage_state"],
            "provenance_ids": joined(provenance),
            "source_ids": optional_joined(record["source_ids"]),
            "note": record["note"] or None,
        })
    for coverage_id, record in enumerate(coverage["categories"], start=1):
        rows["UniverseCoverage"].append({
            "id": coverage_id,
            "category": record["category"],
            "source_file": record["file"],
            "required": record["required"],
            "accounted": record["accounted"],
            "data_ready": record["data_ready"],
            "coverage_percent_decimal": record["coverage_percent"],
        })
    for record in fixtures:
        provenance = [source_ids[key] for key in record["provenance_ids"]]
        rows["UniverseReviewFixture"].append({
            "id": fixture_ids[record["id"]],
            "stable_key": record["id"],
            "mechanic_family": record["mechanic_family"],
            "input_stable_keys": joined(record["input_ids"]),
            "initial_state_json": canonical_json(record["initial_state"]),
            "commands_json": canonical_json(record["commands"]),
            "expected_facts_json": canonical_json(record["expected_facts"]),
            "quality_floor": record["quality_floor"],
            "provenance_ids": joined(provenance),
        })
    for pack_id, record in enumerate(pack_index["files"], start=1):
        rows["UniversePackFile"].append({
            "id": pack_id,
            "relative_path": record["file"],
            "bytes": record["bytes"],
            "rows": record["rows"],
            "sha256": record["sha256"],
        })
    return rows
