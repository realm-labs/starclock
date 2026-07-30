# Currency Wars encounters, waves and bosses

Batch `G12-P2-B5` closes every direct encounter obligation and records the
bounded shared StageConfig closure without promoting rows by prefix, name or
adjacent ID.

## Direct GridFight catalogs

The normalized pack imports all:

- 25 `GridFightCamp` encounter groups;
- 160 `GridFightMonster` candidates;
- 146 `GridFightEliteGroup` scaling records; and
- five `GridFightFormationWave` boundaries.

Camp `MonsterList` values resolve 152 distinct GridFight monster identities.
The eight remaining monsters and 138 remaining elite-scaling records stay in
the pack because they are direct, frozen mode obligations; reachability is not
invented for them. All eight elite-group references present on monster rows
resolve exactly.

## Shared StageConfig closure

Camp rows directly publish 46 distinct `BattleAreaID` roots through
`BattleAreaList` and `BossBattleArea`. The stable StageConfig group key closes
a Stage row only when `floor(StageID / 100)` equals that exact referenced
BattleArea ID.

This produces 840 released shared StageConfig rows. Each dossier preserves the
Stage type, level, elite group, level graph, Stage ability configuration,
sub-level graphs, configuration payload, monster waves, terminal conditions
and release flag. The remaining 21 exact BattleArea roots have no StageConfig
row at the pinned released snapshot and are retained as `Researched` gaps with
an evidence-triggered replacement condition.

This closure proves Stage reachability from Camp rows. It does not assert that
numeric monster IDs inside generic StageConfig wave payloads are GridFight
monster IDs.

## Boss boundary

Ten Camp rows publish a `BossBattleArea`. They do not publish a
BattleArea-to-GridFightMonster join. The pack therefore records ten
policy-bound boss boundaries and the complete Camp-wide candidate set, while
leaving exact boss identity unresolved. A name match or numeric shape is not
sufficient to narrow those candidates.

## Result

The five normalized files contain 1,207 rows: 336 newly accounted direct
GridFight obligations, 840 resolved shared StageConfig rows, 21 exact
StageConfig gaps and ten derived boss-boundary rows. Their combined digest is
`1262189b33478aabe8d17798f722bbe5f79cc893472fcc227cc9be8f066341ab`.

```text
fnm exec --using 24.15.0 node \
  tools/currency-wars-reference/import-encounters.mjs \
  --source-cache .cache/content-reference/turnbasedgamedata
fnm exec --using 24.15.0 node \
  tools/currency-wars-reference/verify-encounters.mjs \
  --source-cache .cache/content-reference/turnbasedgamedata
```
