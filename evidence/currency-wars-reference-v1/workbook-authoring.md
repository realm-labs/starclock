# Currency Wars workbook authoring

Batch `G12-P3-B5` authors all 102 normalized tables into the three isolated
Sora workbooks with the requested openpyxl 3.1.5 path:

| Workbook | Sheets | SHA-256 |
|---|---:|---|
| `CurrencyWars.xlsx` | 60 | `3cd9dce6a45aca023249c247a2910f63a325b7cca06cc566220e795df25350c0` |
| `CurrencyWarsBindings.xlsx` | 32 | `86d3c65b3303b5ae9af77148fe06c948840ee6632e42895b56e41847f04dc4b2` |
| `CurrencyWarsReview.xlsx` | 10 | `689d654b32d4cba3b54957407376853613142466cedab67bbe64add50ac138b2` |

Together they contain 75,643 authored rows. The semantic digest over ordered
workbook, sheet and cell values is
`34305b47ba90ec9f4ad0b7f091c317553f18e6f66d76b4b4726de88802d7981b`.

## Authoring behavior

Each sheet preserves Sora metadata rows 1–7 byte-semantically and starts data
at row 8. It has:

- `A8` freeze panes and a complete field-row filter;
- deterministic private numeric IDs and normalized stable keys;
- bilingual summary, ownership, coverage and evidence columns;
- source references normalized to stable IDs in the `Sources` sheet;
- wrapped, width-capped cells and alternating row fills;
- list validation for ownership, coverage and evidence quality; and
- conditional highlighting for approximate or policy-bound evidence.

The source-reference normalization avoids Excel's 32,767-character cell limit
without losing provenance. The canonical pack index is split into 234
source-equivalent rows for the same reason. That authoring correction changes
the current pack digest to
`16d508a4f3ae0c1537b548650979c44a0f651f2efa1c4a4bbd13c29f65472f4c`
while preserving exact stable-ID membership.

Domain fields that are absent or legitimately empty now permit zero-length
Sora strings. Regenerated current schema-lock and reader digests are
`8158f1dfe2a7c0565fd6b9156f0b5a2f0c3c77aa212254a61e6b303d8b8d9a77`
and
`641230b81fd1af999de25a8bc0b2d858c617a9fba98442ed2c6a2136eba47f7a`.

The author refuses to run when any target workbook exists. Structural QA
reloads all workbooks, compares sheet order and metadata with the Sora
templates, checks every normalized row count/stable key, rejects formulas,
errors and cell overflow, and verifies all 102 authoring surfaces.
