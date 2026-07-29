# Gold and Gears V1 Normalized Reference

This directory is the Goal 08 JSON research/staging representation for the
Version 4.4 Gold and Gears reference pack. It is not an authoring surface and
is never loaded by runtime code. The authoritative authoring form is the
isolated Excel workbook set; Sora 0.3.0 owns schema export.

Phase 1 topology files regenerate with:

```text
node tools/gold-and-gears-reference/import-topology.mjs
node tools/gold-and-gears-reference/verify-topology.mjs
```

Every row carries bilingual mechanical text, explicit ownership and coverage,
and ordered row-level source references. `map-edges.json` is deliberately
`ProjectPolicy`: released chessboard configs contain nodes and coordinates but
no explicit edge relation.
