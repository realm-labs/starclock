# Goal 19 Content Manifest Audit

## Result

`G19-P0-B3` freezes 1,904 exact-once obligations at canonical digest
`d3d6000e3709fa6fd105104ab1b6b586bf0d804ef65c4e3a6e824d6457a6684e`.

- 1,398 Fate/Star Rail Night-owned obligations;
- 93 selector-backed shared obligations;
- 413 evidence-only rows or layout files;
- 1,478 currently `DataReady` rows;
- 13 BattleEvent/BattleTarget joins retained as `ResearchRequired` rather than
  silently promoted;
- six generated exact-zero generic pool records.

The direct denominator contains every row from all 51 `Fate*.json` and
`FateRin*.json` tables and all 64 focused configuration files. The shared
closure contains eight `StageType=FateActivity` stages, their five enemy
variants, five templates and eight skills, direct BattleArea/MazeBuff/status
relationships, and conservative BattleEvent/BattleTarget candidates.

The manifest uses source-file SHA-256 plus a zero-based top-level row locator.
It does not parse 64-bit text hashes into JavaScript numbers when constructing
evidence identity. Direct rows become individual obligations without copying
upstream prose or programs.

## Exact-zero pool proof

Blessing, Curio, Occurrence, Shop, Service and generic run-currency families
close at zero. The complete Fate/FateRin table family, Fate gameplay configs,
focused layouts and selected shared closure expose no generic identity or pool
join for those families. Fate-owned buffs, Noble Phantasms, magical energy and
progression are distinct families and are not reclassified by name similarity.

## Remaining bounded research

The 13 conservative BattleEvent/BattleTarget scalar matches require typed
field or configuration proof in P2-B2. They remain inside the denominator as
`ResearchRequired`, so later coverage cannot erase them. BattleArea joins were
added to the source inventory after the initial B2 snapshot and are recorded
as the explicit B3 inventory correction.

## Reproduction

```text
node --max-old-space-size=4096 \
  tools/fate-star-rail-night-reference/inventory.mjs \
  --source-cache .cache/fate-star-rail-night-sources --check
node --max-old-space-size=4096 \
  tools/fate-star-rail-night-reference/manifest.mjs \
  --source-cache .cache/fate-star-rail-night-sources
node --max-old-space-size=4096 \
  tools/fate-star-rail-night-reference/manifest.mjs \
  --source-cache .cache/fate-star-rail-night-sources --check
```
