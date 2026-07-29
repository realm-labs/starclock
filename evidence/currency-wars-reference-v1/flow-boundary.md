# Currency Wars Version 4.4 GridFight Flow Boundary

## Scope and correction

This dossier records the corrected `G12-P1-B10` entry, Gambit and three-Plane
Node-flow boundary for the pinned Version 4.4 sources. It supersedes the
published `G12-P1-B1` Tourn3/Persona interpretation without rewriting that
historical commit. It is reference evidence only and does not claim a runtime
state machine.

## Exact Currency Wars identity

The authoritative structured selectors are:

- `ExcelOutput/GuideRogueTab.json#2`, whose exact row digest is
  `984f6e53d53424adb2962c19dbc0a6e1cd039adad2bba3393962f4339274a976`;
  tab `1003` sets `GuideType = GridFight`;
- `ExcelOutput/GuideRogueData.json#5`, whose exact row digest is
  `3162accac44c825d114b06c9b71f08f520a78c2dfcd91a272e99bfa1c341cb5e`;
  guide data `301` selects tab `1003`; and
- their independent released EN/CHS TextMap values name Currency Wars /
  货币战争.

Released EN/CHS TextMap rows also independently name Standard Gambit /
标准博弈 and Overclock Gambit / 超频博弈. The four exact
`GridFightSeasonModule` rows select season `1`, sub-seasons `1` through `4`
and activity modules `7100201`, `7100301`, `7100401`, `7100501`.

The corrected profile selects `GridFight`, not `TournRogue` or `Tourn3`.
Goal 11 retains the Tourn3 sources as Divergent Universe. No corrected flow
row contains a `RogueTourn` or `RoguePersona` source reference.

## Route, Plane and Node closure

The complete `GridFightStageRoute` table contains 493 rows and 26 distinct
route IDs. Grouping only by its authored `(ID, ChapterID)` fields yields 75
route/Plane layers:

| Plane (`ChapterID`) | Route layers | Nodes |
|---:|---:|---:|
| 1 | 26 | 196 |
| 2 | 25 | 151 |
| 3 | 24 | 146 |

Every StageRoute row references exactly one of the 493 unique
`GridFightNodeTemplate` rows, and every NodeTemplate is referenced exactly
once. The normalized Node retains:

- route, Plane and authored `SectionID`;
- NodeTemplate ID, Stage ID and Node type;
- parameter IDs, penalty/bonus rule ID and basic Gold reward;
- the exact next Node inside the same route/Plane, derived only from increasing
  authored `SectionID`.

The five direct Node types are `Boss`, `CampMonster`, `EliteBranch`, `Monster`
and `Supply`. Each has one exact `GridFightNodeTypeShow` row and one normalized
room-type/domain-composition identity.

`GridFightDivisionInfo` and `GridFightDivisionStage` close one-to-one over 97
Division IDs. Each normalized difficulty records the exact progress,
Standard/Overclock score-rule IDs, weekly/Experience modifiers, environment
lists and any direct JSON path. The fixed data does not publish a
Division-to-StageRoute join, so difficulty and route topology remain explicit
separate axes instead of being linked by an ID pattern.

## Stage and terminal closure

All fifteen `GridFightStage` rows are retained as battle-stage terminal rules,
including exact total-turn and threshold fields. All six
`GridFightSettleRank` rows are retained as settlement classifications,
including exact score intervals and rank types. Story, reward and presentation
content remains excluded.

## Gambit and lifecycle boundary

The released bilingual rule text proves the two Gambit identities and their
high-level rank behavior. No fixed `GridFightStageRoute` row directly selects
Standard or Overclock. Both Gambits therefore retain the complete 26-route
set, and each route records an unresolved Gambit binding with this replacement
condition: replace it when a released structured selector directly binds a
StageRoute ID to a Gambit.

`ChapterID` and `SectionID` prove authored order. They do not prove cross-Node
carry, Plane transition mutation or terminal reset ordering. The 493 flow rows
therefore retain empty `carry_rules` and `reset_rules`, are `Researched`, and
state `UnspecifiedByStageRoute`. This avoids presenting an invented lifecycle
policy as observed parity.

## Generated result and reproduction

The thirteen normalized files contain 1,225 rows:

- one profile, two Gambits, four modules and two Guide entries;
- 21 Stage/settlement terminal conditions;
- one route group, 26 routes, 97 Divisions and 75 Plane layers;
- five Node-type rooms, five domain compositions, 493 Nodes and 493 flow rows.

Their combined digest in lexicographic file order is
`b6deeec5b8bae2de0d076d27fe3d57ba9ebd4a0da10a808d3364cf80739f7fa9`.

```text
fnm exec --using 24.15.0 node tools/currency-wars-reference/import-flow.mjs \
  --source-cache .cache/content-reference/turnbasedgamedata
fnm exec --using 24.15.0 node tools/currency-wars-reference/import-flow.mjs \
  --check --source-cache .cache/content-reference/turnbasedgamedata
fnm exec --using 24.15.0 node tools/currency-wars-reference/verify-flow.mjs \
  --source-cache .cache/content-reference/turnbasedgamedata
```
