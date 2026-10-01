"""Clean-target authoring for the bounded representative production Counter.

This corrects admission of an existing project program, not released kit parity.
Only the two explicitly owned expression rows may change.
"""

from __future__ import annotations

import json
from pathlib import Path

from openpyxl import load_workbook

ROOT = Path(__file__).resolve().parents[2]
DATA = ROOT / "config" / "data"


def guard_rows() -> dict[str, dict[str, object]]:
    return {
        "ConditionExpression": {
            "id": 24201,
            "stable_key": "program.clara.condition.has-counter-charge",
            "node": json.dumps({
                "type": "Compare", "left_expression_id": 24204,
                "comparison": "Greater", "right_expression_id": 24202,
            }, separators=(",", ":")),
        },
        "ValueExpression": {
            "id": 24204,
            "stable_key": "program.clara.expr.counter-charges",
            "result_kind": "Integer",
            "node": json.dumps({
                "type": "ReadStateSlot", "state_slot_id": 24206,
            }, separators=(",", ":")),
        },
    }


def row_location(sheet, record: dict[str, object]) -> tuple[list[str], int | None]:
    fields = [cell.value for cell in sheet[3][1:] if cell.value]
    id_column = fields.index("id") + 2
    found = [index for index in range(8, sheet.max_row + 1)
             if str(sheet.cell(index, id_column).value) == str(record["id"])]
    if len(found) > 1:
        raise ValueError(f"duplicate owned expression ID {record['id']}")
    return fields, found[0] if found else None


def check() -> None:
    for table, record in guard_rows().items():
        workbook = load_workbook(DATA / f"{table}.xlsx")
        try:
            fields, location = row_location(workbook.active, record)
            if location is None:
                raise ValueError(f"{table} lacks Counter admission expression")
            actual = {field: workbook.active.cell(location, index).value
                      for index, field in enumerate(fields, start=2)}
            if any(str(actual.get(field)) != str(value) for field, value in record.items()):
                raise ValueError(f"{table} Counter admission expression differs")
        finally:
            workbook.close()


def write_clean(output: Path) -> None:
    output = output.resolve()
    if output.exists() or output == DATA or DATA in output.parents:
        raise ValueError("Counter authoring requires a new, non-production output directory")
    output.mkdir(parents=True)
    for table, record in guard_rows().items():
        source = DATA / f"{table}.xlsx"
        workbook = load_workbook(source)
        try:
            sheet = workbook.active
            fields, location = row_location(sheet, record)
            if location is None:
                location = sheet.max_row + 1
            changed = {(location, index) for index in range(2, len(fields) + 2)}
            before = {(cell.row, cell.column): (cell.value, cell.data_type, cell._style)
                      for row in sheet for cell in row if (cell.row, cell.column) not in changed}
            for index, field in enumerate(fields, start=2):
                sheet.cell(location, index).value = record[field]
            target = output / source.name
            workbook.save(target)
        finally:
            workbook.close()
        reviewed = load_workbook(target)
        try:
            for (row, column), original in before.items():
                cell = reviewed.active.cell(row, column)
                if (cell.value, cell.data_type, cell._style) != original:
                    raise ValueError(f"{table} changed unowned cell {cell.coordinate}")
            print(f"{target}: reviewed {len(before)} unchanged cells; owned row {location}")
        finally:
            reviewed.close()
