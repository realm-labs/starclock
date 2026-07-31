# Goal 19 Source Inventory Audit

## Result

`G19-P0-B2` initially inventoried 177 materialized source files and 959,455
top-level JSON rows. P0-B3 reference closure added `BattleArea.json` and
`BattleAreaUnifiedConfig.json`; the current reproducible inventory is 179
files / 959,606 top-level rows at canonical receipt digest
`13d5f1a574482f5587f2f72455ffe7133d2f432e8d98d951a469fd97f3b8d4c3`.
The large row count includes complete bilingual TextMaps and identity indexes;
it is not a content denominator.

| Category | Files | Disposition |
|---|---:|---|
| Dedicated `FateRin` tables | 25 | Selector seeds, with mission/reward/talk rows evidence-only where appropriate |
| Dedicated `Fate` tables | 26 | Newly discovered primary selector seeds |
| `Config/Gameplays/Fate` | 31 | Selector seeds |
| FateRin-focused AI/ability/event layouts | 33 | Transitive closure candidates |
| Shared Stage/battle/enemy tables | 10 | Transitive closure candidates; two BattleArea join tables added by P0-B3 |
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
