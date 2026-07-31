# G19-P3-B5 — Workbook Authoring Audit

Pinned `openpyxl==3.1.5` authored the four complete production workbooks from
the Sora-generated clean templates. The result contains 48 sheets and 5,934
rows. Each sheet preserves all seven Sora metadata rows, the exact field order,
and the normalized stable-key order. No formula or Excel error is permitted.

The workbooks use frozen panes, filters, hidden gridlines, bounded column widths,
wrapped text, light structural borders, semantic data validation and conditional
formatting for ProjectPolicy and EvidenceOnly rows. Two independent clean
generations are byte-identical to one another and to the committed targets:

| Workbook | SHA-256 |
|---|---|
| `FateStarRailNight.xlsx` | `9dde5a7ef1408b3923127cc8ed2e108bfe70731e13fb9e3037e0cb4b2a9ee1fe` |
| `FateStarRailNightBindings.xlsx` | `c191e5484491020946206ac3b9ead8b12d86a7dfee431ea3a47b2a03a28b31f5` |
| `FateStarRailNightCombat.xlsx` | `e053682107fc27b216fe1233cb75b5eec3511c2f4b6a781362fae14db829d30f` |
| `FateStarRailNightReview.xlsx` | `dfa590263be2a632b13baeba100c9e21c42c21913ef502050d7e80dfc47305ec` |

The workspace dependency loader returned no artifact-tool runtime path in this
desktop worktree. The repository's stricter production contract therefore
remained authoritative: the pinned openpyxl path performed authoring and
round-trip QA; P3-B6 owns LibreOffice rendering and every-sheet visual review.

Focused command:

```text
fnm exec --using 24.15.0 .cache/g19-venv/bin/python tools/fate-star-rail-night-reference/verify-workbooks.py --root . --directory config/fate-star-rail-night/data --templates config/fate-star-rail-night-generated/templates
```
