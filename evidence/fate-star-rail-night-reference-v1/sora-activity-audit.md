# G19-P3-B1 — Activity Sora Table Audit

The isolated `starclock_fate_star_rail_night_reference` project now declares
fourteen activity tables sourced only from `FateStarRailNight.xlsx`. They hold
154 rows: 153 normalized records and one derived Candidate profile index. The
derived row binds the released pack digest and explicitly does not enlarge the
frozen 1,904-obligation denominator.

The table verifier proves every sheet is non-empty, each stable key appears
once in this partition, the schema is generator-owned and regeneration is
byte-identical. All payload vectors and source receipts remain canonical JSON
strings, so no 64-bit identifier or decimal passes through a floating type.
The project is reference-only and has no runtime import.

Focused commands:

```text
fnm exec --using 24.15.0 node tools/fate-star-rail-night-reference/generate-sora-schema.mjs --root . --batch G19-P3-B1
fnm exec --using 24.15.0 node tools/fate-star-rail-night-reference/verify-sora-tables.mjs --root . --batch G19-P3-B1
```

Result: fourteen tables and 154 rows verified with zero empty tables, duplicate
stable keys or generated-schema drift.
