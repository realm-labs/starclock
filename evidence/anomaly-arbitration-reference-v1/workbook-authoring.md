# Anomaly Arbitration Workbook Authoring

Goal 13 batch `G13-P3-B5` promotes the complete normalized reference pack into
three isolated Excel authoring surfaces. The writer uses the bundled Python
runtime with `openpyxl 3.1.5`, starts from the Sora 0.3.0 templates, and refuses
to overwrite any existing target workbook.

## Frozen outputs

| Workbook | Sheets | Rows | SHA-256 |
|---|---:|---:|---|
| `AnomalyArbitration.xlsx` | 17 | 68 | `f4c4f7522742e649dd4d51c96d4e94462c629fd5c02afea60973d8b1f8fc4901` |
| `AnomalyArbitrationBindings.xlsx` | 12 | 416 | `65ee37d3355a58e226f287e4fa6ef418e65a6dd8cd48c5ff72f4e8e5651ffbe5` |
| `AnomalyArbitrationReview.xlsx` | 8 | 1,619 | `1b1ed4ad711b2d491d5c22f4c13b66889582e250917e4704931a7dc37536d749` |

The 37 sheets contain 2,103 rows. Their canonical semantic digest is
`f4ee771a47c5ee91212d1214bfd552010e169e062f5b1309fe23ef41b7cfe389`.

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
