# Currency Wars V1 Reference Staging

This directory contains deterministic normalized research/debug JSON for Goal
12. It is not a runtime loading surface. Production authoring remains the
three isolated Excel workbooks, with Sora 0.3.0 as schema, code-generation and
export authority.

`G12-P1-B1` generates the entry, Gambit, area, difficulty, Plane/Node,
Domain-composition and flow files with:

```text
node tools/currency-wars-reference/import-flow.mjs \
  --source-cache <turnbasedgamedata-repository>
node tools/currency-wars-reference/import-flow.mjs --check \
  --source-cache <turnbasedgamedata-repository>
node tools/currency-wars-reference/verify-flow.mjs \
  --source-cache <turnbasedgamedata-repository>
```

Every factual row carries independent EN/CHS summaries and ordered source
receipts. `ProjectPolicy` fields include a note and replacement condition.
The 848 older Tourn2 room candidates remain only in the frozen manifest until
exact Stage/config evidence promotes or excludes them; `rooms.json` stays
empty and grants no shared reachability.

`G12-P1-B2` generates the Squad HP, finite/unlimited action-value,
battle-result and zero-HP failure boundary with:

```text
node tools/currency-wars-reference/import-squad-boundary.mjs \
  --source-cache <turnbasedgamedata-repository>
node tools/currency-wars-reference/import-squad-boundary.mjs --check \
  --source-cache <turnbasedgamedata-repository>
node tools/currency-wars-reference/verify-squad-boundary.mjs \
  --source-cache <turnbasedgamedata-repository>
```

The released sources do not expose a single global finite action-value or
timeout-loss constant. Those values remain node/difficulty-configured, and
same-boundary victory precedence is an explicit replaceable policy.
