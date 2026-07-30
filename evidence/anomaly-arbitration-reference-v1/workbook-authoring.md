# Anomaly Arbitration Workbook Authoring

Goal 13 batch `G13-P3-B5` promotes the complete normalized reference pack into
three isolated Excel authoring surfaces. The writer uses the bundled Python
runtime with `openpyxl 3.1.5`, starts from the Sora 0.3.0 templates, and refuses
to overwrite any existing target workbook.

## Frozen outputs

| Workbook | Sheets | Rows | SHA-256 |
|---|---:|---:|---|
| `AnomalyArbitration.xlsx` | 17 | 68 | `0119370d9000e4aec476a7f5ec35b78031d3757101928c8e03828189f3c1618c` |
| `AnomalyArbitrationBindings.xlsx` | 12 | 416 | `f6373e0addd351ebfd46fe84c872fbfef0dfb7ec42b10d425db3a3b53cc50aa3` |
| `AnomalyArbitrationReview.xlsx` | 8 | 1,619 | `a218a87f3a2cc546c221f0c200615a2a10206a5aac9d632a3f3dbe6d8d09cf9a` |

The 37 sheets contain 2,103 rows. Their canonical semantic digest is
`d740894821b6ffbcdec0e0cf9de88441f546f627f5b35f864b3c1e22510a27e0`.

## Authoring and QA contract

- `tools/anomaly-arbitration-reference/author_workbooks.py` reads all 37
  normalized files in the authoring-contract order and emits complete clean
  workbooks.
- Stable source keys remain visible while deterministic positive integer IDs
  satisfy Sora map and reference fields.
- Every row retains canonical `payload_json`, manifest IDs and source IDs.
  Semantic QA reloads each workbook and proves those projections against the
  payload.
- Every sheet preserves Sora metadata and validation, freezes at `A8`, enables
  filtering, wraps headers and data, uses bounded widths, and rejects formulas
  and Excel error cells.
- Fixed workbook properties and canonical archive metadata make independent
  generation byte-identical. `--check` regenerates all three workbooks in a
  temporary directory and compares their SHA-256 values.
- A normal authoring invocation against the committed target exits with
  `FileExistsError`, proving the no-overwrite boundary.

The authored workbooks pass:

```text
/Users/mikai/.cache/codex-runtimes/codex-primary-runtime/dependencies/python/bin/python3 \
  tools/anomaly-arbitration-reference/author_workbooks.py --check
.cache/tools/sora-cli-0.3.0/bin/sora check \
  --project config/anomaly-arbitration/project.toml
```

Binary/debug export, generated-reader loading, double-generation evidence and
rendered inspection belong to `G13-P3-B6`.
