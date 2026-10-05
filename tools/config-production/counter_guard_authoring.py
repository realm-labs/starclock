"""Clean-target authoring for the bounded representative production Counter.

This corrects admission of an existing project program, not released kit parity.
Only the explicitly owned admission expressions and owner selector may change.
"""

from __future__ import annotations

import json
from copy import copy
from hashlib import sha256
from pathlib import Path

from openpyxl import load_workbook
from openpyxl.xml.functions import tostring

ROOT = Path(__file__).resolve().parents[2]
DATA = ROOT / "config" / "data"


def guard_rows() -> dict[str, list[dict[str, object]]]:
    return {
        "ConditionExpression": [{
            "id": 24201,
            "stable_key": "program.clara.condition.admitted-counter-owner",
            "node": json.dumps({
                "type": "All", "condition_ids": [24202, 24203],
            }, separators=(",", ":")),
        }, {
            "id": 24202,
            "stable_key": "program.clara.condition.has-counter-charge",
            "node": json.dumps({
                "type": "Compare", "left_expression_id": 24204,
                "comparison": "Greater", "right_expression_id": 24202,
            }, separators=(",", ":")),
        }, {
            "id": 24203,
            "stable_key": "program.clara.condition.has-present-owner",
            "node": json.dumps({
                "type": "SelectorCardinality", "selector_id": 24251,
                "minimum_count": 1, "maximum_count": 1,
            }, separators=(",", ":")),
        }],
        "ValueExpression": [{
            "id": 24204,
            "stable_key": "program.clara.expr.counter-charges",
            "result_kind": "Integer",
            "node": json.dumps({
                "type": "ReadStateSlot", "state_slot_id": 24206,
            }, separators=(",", ":")),
        }],
        "Selector": [{
            "id": 24251, "domain": "Battle", "origin": "Owner",
            "side_relationship": "SameSide", "life": "Alive", "presence": "Present",
            "reference_point": "CurrentState", "ordering": "StableId",
            "minimum_count": "0", "maximum_count": "1", "empty_pool_policy": "NoOp",
            "choice": "First", "weight_expression_id": None,
            "rng_purpose_key": None, "allow_repeated_targets": "false",
        }],
    }


def row_location(sheet, record: dict[str, object]) -> tuple[list[str], int | None]:
    fields = [cell.value for cell in sheet[3][1:] if cell.value]
    id_column = fields.index("id") + 2
    found = [index for index in range(8, sheet.max_row + 1)
             if str(sheet.cell(index, id_column).value) == str(record["id"])]
    if len(found) > 1:
        raise ValueError(f"duplicate owned expression ID {record['id']}")
    return fields, found[0] if found else None


def native_features(sheet) -> tuple[object, ...]:
    return (
        tostring(sheet.data_validations.to_tree()),
        tuple((str(region.sqref), tuple(tostring(rule.to_tree())
                                       for rule in sheet.conditional_formatting[region]))
              for region in sheet.conditional_formatting),
        sheet.freeze_panes, tostring(sheet.auto_filter.to_tree()),
        tostring(sheet.protection.to_tree()), str(sheet.merged_cells),
        tostring(sheet.sheet_view.to_tree()),
        tuple((table.name, tostring(table.to_tree())) for table in sheet.tables.values()),
    )


def check() -> None:
    for table, records in guard_rows().items():
        workbook = load_workbook(DATA / f"{table}.xlsx")
        try:
            for record in records:
                fields, location = row_location(workbook.active, record)
                if location is None:
                    raise ValueError(f"{table} lacks Counter admission row {record['id']}")
                actual = {field: workbook.active.cell(location, index).value
                          for index, field in enumerate(fields, start=2)}
                if any(str(actual.get(field)) != str(value) for field, value in record.items()):
                    raise ValueError(f"{table} Counter admission row {record['id']} differs")
        finally:
            workbook.close()


def write_clean(output: Path) -> None:
    output = output.resolve()
    if output.exists() or output == DATA or DATA in output.parents:
        raise ValueError("Counter authoring requires a new, non-production output directory")
    output.mkdir(parents=True)
    for table, records in guard_rows().items():
        source = DATA / f"{table}.xlsx"
        source_hash = sha256(source.read_bytes()).hexdigest()
        workbook = load_workbook(source)
        try:
            sheet = workbook.active
            original_last_row = sheet.max_row
            locations = []
            next_row = sheet.max_row + 1
            for record in records:
                fields, location = row_location(sheet, record)
                if location is None:
                    location, next_row = next_row, next_row + 1
                locations.append((record, location))
            changed = {(location, index) for _, location in locations
                       for index in range(2, len(fields) + 2)}
            before = {(cell.row, cell.column): (
                cell.value, cell.data_type, copy(cell._style),
                copy(cell.comment), copy(cell.hyperlink))
                      for row in sheet for cell in row if (cell.row, cell.column) not in changed}
            features = native_features(sheet)
            template = locations[0][1]
            for record, location in locations:
                for index, field in enumerate(fields, start=2):
                    cell = sheet.cell(location, index)
                    if location > original_last_row:
                        cell._style = copy(sheet.cell(template, index)._style)
                    cell.value = record[field]
            target = output / source.name
            workbook.save(target)
        finally:
            workbook.close()
        reviewed = load_workbook(target)
        try:
            for (row, column), original in before.items():
                cell = reviewed.active.cell(row, column)
                if (cell.value, cell.data_type, cell._style, cell.comment, cell.hyperlink) != original:
                    raise ValueError(f"{table} changed unowned cell {cell.coordinate}")
            sheet = reviewed.active
            after_features = native_features(sheet)
            if features != after_features:
                raise ValueError(f"{table} changed unowned native features")
            if source_hash != sha256(source.read_bytes()).hexdigest():
                raise ValueError(f"{table} production source changed during clean authoring")
            print(f"{target}: reviewed {len(before)} unchanged cells; "
                  f"owned rows {[location for _, location in locations]}; source SHA-256 {source_hash}")
        finally:
            reviewed.close()
