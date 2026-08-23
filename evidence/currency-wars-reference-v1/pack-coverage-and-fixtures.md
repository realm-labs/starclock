# Currency Wars pack, coverage and semantic fixtures

The current canonical normalized pack keeps exact-once manifest coverage,
typed mechanic dispositions and reference semantic fixtures together. Runtime
execution evidence remains owned by Goal 21's Rust tests and execution ledger.

## Exact-once coverage

`coverage.json` contains one row for every frozen manifest obligation:

- 19,250 total obligations;
- 18,524 Currency Wars or shared obligations at `DataReady`; and
- 726 `EvidenceOnly` obligations explicitly marked `Excluded`.

The eligible denominator is therefore 18,524, with 18,524 DataReady records
and no blocking manifest row. Exclusions remain auditable through their exact
source receipt and cannot promote a normalized content identity.

`sources.json` deduplicates provenance by repository, revision, path, row
locator, evidence digest and evidence quality. Coverage rows resolve to these
source identities and, where available, the source-shaped semantic records
imported by earlier batches.

## Mechanic source boundary

All 2,367 mechanic obligations generate both:

- an exact source-program dossier; and
- a typed mechanic-rule boundary.

Eighty-five reviewed Activity progression rules have
`runtime_lowered = true`: five role-cost availability rows and 80 season
score/experience rows. Seventy-seven tutorial or world-prop presentation
programs carry closed typed audits with zero authoritative operations. The
remaining 2,205 rules preserve their exact source contribution for later typed
review. This records complete source ownership without interpreting unproved
configuration JSON as executable Starclock behavior.

Twelve remaining semantic uncertainties are nonblocking `ProjectPolicy`
research rows. Each records known facts, the selected deterministic policy,
rejected alternatives and a released-evidence or reproducible-observation
replacement condition.

## Semantic fixtures

The pack includes all 28 fixture families frozen in Phase 0 and one
deterministic base review case per family. Each case records:

- typed ordered preconditions;
- a stable-ID candidate order and deterministic seed;
- ordered reference assertions; and
- exact-or-explicit-policy expected facts.

These are review fixtures, not runtime tests. Phase 4 executes their semantic
contract and replacement checks.

## Canonical pack

`manifest.json` lists every currently generated normalized file and exact row
count. `pack-index.json` hashes every normalized file except itself, maps every
stable ID to its owning file and computes the canonical pack digest:

`e0533de73fc9e14a8ceadcf6b8be83cb96aac8ee5bc3e2e611a445bd27436efb`.

The digest is a Phase 2 checkpoint. It will change when the planned Phase 4
reconciliation-receipt file joins the normalized pack.

```text
fnm exec --using 24.15.0 node \
  tools/currency-wars-reference/generate-pack.mjs
fnm exec --using 24.15.0 node \
  tools/currency-wars-reference/verify-pack.mjs
```
