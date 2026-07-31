# G19-P3-B2 — Binding Sora Table Audit

Fourteen binding tables add 597 rows to the isolated Sora project. The tables
separate Master and Servant identities, Noble Phantasm definitions and levels,
rarity/tag/keyword catalogs, decks and recommendations, Command Spells and
affixes, resources, opaque rule-program digests and lifecycle bindings.

The cumulative verifier reports 28 non-empty tables and 751 unique stable keys.
No config program is copied or executed: program evidence remains a released
file digest with explicit `runtime_executable=false` at the profile boundary.
Canonical source receipts and mechanical payloads stay strings.

Focused command:

```text
fnm exec --using 24.15.0 node tools/fate-star-rail-night-reference/verify-sora-tables.mjs --root . --batch G19-P3-B2
```

Result: 28 tables / 751 rows, zero empty tables, duplicate stable keys or
generated-schema drift.
