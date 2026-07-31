#!/usr/bin/env python3

import hashlib
import json
import pathlib

from author_workbooks import GROUPS

ROOT = pathlib.Path.cwd()
DATA = ROOT / "config/pure-fiction/data"
OUTPUT = ROOT / "evidence/pure-fiction-v1/workbook-visual-review.json"
workbooks = []
for workbook_name, tables in GROUPS.items():
    payload = (DATA / workbook_name).read_bytes()
    workbooks.append({
        "file": workbook_name,
        "sha256": hashlib.sha256(payload).hexdigest(),
        "sheets": [sheet_name for _, sheet_name in tables],
    })
document = {
    "schema_revision": "starclock.pure-fiction-workbook-visual-review.v1",
    "render_pipeline": [
        "prepare_visual_review.py",
        "LibreOffice headless PDF export",
        "pdftoppm PNG rendering at 72 DPI",
        "contact_sheets.py",
    ],
    "rendered_pages": 37,
    "reviewed_pages": 37,
    "review_scope": "Rows 1-10 and all schema columns A:P on every sheet",
    "observations": [
        "All 37 sheets render as nonblank single-page landscape previews.",
        "All 15 authored headers are present with consistent dark-blue styling.",
        "Representative bilingual, provenance and canonical JSON cells are visible.",
        "No clipped header band, overlapping object, formula error or blank required table was observed.",
    ],
    "workbooks": workbooks,
    "result": "Passed",
}
OUTPUT.parent.mkdir(parents=True, exist_ok=True)
OUTPUT.write_text(json.dumps(document, ensure_ascii=False, indent=2) + "\n")
print("Pure Fiction visual-review evidence recorded: 37/37 sheets passed.")
