# Goal 16 Workbook Authoring Audit

`G16-P3-B2` authored four complete, isolated Excel workbooks from the 40
normalized Goal 16 families. The workbooks are the only editable production
authoring surface; normalized JSON remains research/debug staging, and Sora
0.3.0 remains the production schema/export authority.

## Authoring result

| Field | Result |
|---|---|
| Authoring adapter | bundled Python with `openpyxl==3.1.5` |
| Workbooks | 4 |
| Sheets | 40 |
| Authored rows | 10,615 |
| Normalized-file ownership | every one of 40 files maps to exactly one sheet |
| Sora metadata | rows 1–7 on every sheet; authored rows begin at row 8 |
| Semantic digest | `a9c449aab9b115e5599d3440352b7e732bdfe0414cd1e120ce76a25d80acdc1c` |
| Existing-target behavior | generation refuses to overwrite any workbook |
| Formula/error scan | no formula cells or Excel error cells |
| Round trip | every cell reconstructs the canonical normalized value |

The canonical workbook digests are:

| Workbook | SHA-256 |
|---|---|
| `GalacticBaseballerProfiles.xlsx` | `4d30a557792403941f1bc9699535a83b2426b9b5952bc6ec3bb3c0854e04c667` |
| `GalacticBaseballerArsenal.xlsx` | `6477416da03d3781f42fa0812a9acfdf757ded4cb2f774502b713ea393d80b08` |
| `GalacticBaseballerEncounters.xlsx` | `3eec8a52dbfdb4f4927950c6a4e3755bf777db5f02585df49eed242814141582` |
| `GalacticBaseballerReview.xlsx` | `62b24bee05eb6ca0da5536077eaf9dd7a035bc332807b692b118291d25e7e193` |

Two clean output directories were generated independently. All four byte
digests and the semantic digest matched each other and the committed target.
The authoring command was also directed at the populated target and failed
with the required `FileExistsError: refusing to overwrite authored
workbook(s)`.

## Structural and visual review

Each sheet freezes at `A8`, provides field descriptions and constraints,
enables filtering, wraps long values, uses deterministic widths and styles,
and marks evidence-quality policy rows for review. Every schema field column
was rendered, including columns beyond the first viewport.

| Review field | Result |
|---|---|
| Rendered sheets | 40 / 40 |
| Rendered column bands | 141 |
| Schema columns covered | all, with contiguous ordinal proof |
| Contact sheets inspected | 4 / 4 |
| Visible severe defects | 0 |
| Disposition | `PassedHumanInspection` |

The retained `workbook-review/visual-review.json` records every sheet range,
schema-column interval, row count, PNG digest and contact-sheet digest. The
four retained contact sheets were inspected for label coverage, readable
metadata/header/authored rows, clipping, overlap, error cells and broken
styles. Blank space was limited to expected short sheets and final partial
column bands.

## Reproduction

```text
/Users/mikai/.cache/codex-runtimes/codex-primary-runtime/dependencies/python/bin/python3 \
  tools/galactic-baseballer-reference/author-workbooks.py \
  --root "$PWD" --output <new-empty-directory>

/Users/mikai/.cache/codex-runtimes/codex-primary-runtime/dependencies/python/bin/python3 \
  tools/galactic-baseballer-reference/verify-workbooks.py \
  --root "$PWD" --directory config/galactic-baseballer/data

node tools/galactic-baseballer-reference/visual-review-workbooks.mjs \
  "$PWD" config/galactic-baseballer/data <new-review-directory> \
  <new-temporary-tile-directory> \
  /Users/mikai/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules

node tools/galactic-baseballer-reference/record-workbook-visual-review.mjs \
  "$PWD" <review-directory>
```

`visual-review-workbooks.mjs` receives the workspace-bundled dependency root
explicitly and resolves the Artifact Tool and `sharp` from it. The
generation-time tile directory is temporary and is not part of the Candidate
evidence.

Focused contract/reference-pack verification and
`fnm exec --using 24.15.0 node tools/repository-check/run.mjs` passed after
the committed workbooks and visual-review evidence were assembled.
