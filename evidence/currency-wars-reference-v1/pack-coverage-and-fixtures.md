# Currency Wars pack, coverage and semantic fixtures

Batch `G12-P2-B6` closes Phase 2 with a canonical normalized-pack index,
exact-once manifest coverage and reference-only semantic fixtures. It does not
lower or execute runtime behavior.

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

All 2,367 non-presentation mechanic obligations generate both:

- an exact source-program dossier; and
- a reference-only mechanic-rule boundary.

Every rule has `runtime_lowered = false`. Its sole operation preserves the
exact source contribution for later typed review. Seventeen presentation-only
mechanic obligations remain explicit exclusions. This records complete source
ownership without interpreting untyped configuration JSON as executable
Starclock behavior.

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
