# Goal 19 Source Inventory Audit

## Result

`G19-P0-B2` inventories 177 materialized source files and 959,455 top-level
JSON rows at canonical receipt digest
`48ebe84694b6508aa75c7f95cb6a4e1b85e206dd4b1b1bd886552a73ca20f025`.
The large row count includes complete bilingual TextMaps and identity indexes;
it is not a content denominator.

| Category | Files | Disposition |
|---|---:|---|
| Dedicated `FateRin` tables | 25 | Selector seeds, with mission/reward/talk rows evidence-only where appropriate |
| Dedicated `Fate` tables | 26 | Newly discovered primary selector seeds |
| `Config/Gameplays/Fate` | 31 | Selector seeds |
| FateRin-focused AI/ability/event layouts | 33 | Transitive closure candidates |
| Shared Stage/battle/enemy tables | 8 | Transitive closure candidates |
| CHS/EN TextMaps | 2 | Bilingual evidence only |
| StarRailRes identity indexes | 48 | Independent identity cross-checks only |
| Source metadata | 4 | Evidence only |

The planning oracle was too narrow: it named the 25 `FateRin` tables but not
the separate 26-table `Fate` family that carries areas, difficulties, Masters,
Noble Phantasms, Command Spells, Affixes, phases, buffs and monster pools.
The fetcher and plan now retain both families. P0-B3 still owns row-level
membership and exact denominators.

## Named adjacent exclusions

- 166 `Config/Activity/RtBattle` paths remain excluded because no
  Fate-originating selector reaches them.
- 23 Currency Wars `GridFight*Trait*.json` paths remain excluded; Fate-themed
  display text inside a Currency Wars Bond/Trait row is not shared identity.
- `FateRinResidentReward` and `FateRinSwitchDayTalk` remain evidence-only
  unless a concrete row proves a mechanical unlock or graph locator.

## Reproduction

```text
tools/fate-star-rail-night-reference/fetch-sources.sh \
  .cache/fate-star-rail-night-sources \
  /Users/mikai/CLionProjects/starclock/.cache/content-reference
node --max-old-space-size=4096 \
  tools/fate-star-rail-night-reference/inventory.mjs \
  --source-cache .cache/fate-star-rail-night-sources
node --max-old-space-size=4096 \
  tools/fate-star-rail-night-reference/inventory.mjs \
  --source-cache .cache/fate-star-rail-night-sources --check
```
