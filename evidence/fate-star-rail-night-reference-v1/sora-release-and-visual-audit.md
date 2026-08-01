# G19-P3-B6 — Sora Release and Visual Audit

Pinned Sora 0.3.0 exported 48 debug tables and a Zstandard binary bundle. All
5,936 rows retain their private ordinal and normalized stable key. Independent
full regeneration is byte-identical at generated-tree digest
`47179254fe22123e0c03f888cf310a35dbaab3faa14ffcecd42570153ad1f5df`;
the bundle digest is
`f2897da1190ebfe5d6634982382b1bcd5eadcda50b2a050ef1be247b78343336`.
The standalone locked Rust loader parses the bundle and iterates all 48
non-empty generated readers for exactly 5,936 rows. Nothing is imported by a
runtime crate.

Artifact-tool 2.8.6+ rendered every sheet in three contiguous six-column bands:
48 sheets, 144 bands and all eighteen physical columns. The four contact sheets
were inspected for label completeness, metadata/header readability, clipping,
overlap, formula errors and broken style. No severe visual defect was observed;
`visual-review.json` records `PassedHumanInspection` and the exact PNG digests.

Focused commands:

```text
fnm exec --using 24.15.0 node tools/fate-star-rail-night-reference/verify-sora-release.mjs --root . --python .cache/g19-venv/bin/python
fnm exec --using 24.15.0 node tools/fate-star-rail-night-reference/visual-review-workbooks.mjs . config/fate-star-rail-night/data evidence/fate-star-rail-night-reference-v1/workbook-review .cache/g19-workbook-tiles /Users/mikai/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules
fnm exec --using 24.15.0 node tools/fate-star-rail-night-reference/record-workbook-visual-review.mjs . evidence/fate-star-rail-night-reference-v1/workbook-review
```
