# Goal 16 Departure Encounter, Score and Fixture Audit

`G16-P1-B4` closes the Departure encounter graph, stable enemy identities,
score/settlement facts and all 17 mechanism families applicable to the original
profile.

## Encounter closure

The 57 Departure StagePeriod rows reach five exact shared StageConfig rows:

```text
4140016, 4140026, 4140116, 4140126, 4140136
```

Their explicit `_StageInfiniteGroup` references close to:

| Family | Rows |
|---|---:|
| Encounters / StageConfig | 5 |
| Infinite groups | 5 |
| Infinite waves | 17 |
| Infinite monster groups | 17 |
| Ordered enemy candidates | 204 |
| Unique enemy variants | 27 |
| Unique enemy skills | 81 |
| Reachable MonsterStatus rows | 0 |

`MonsterList` order is preserved as an ordered candidate list; it is not
silently reinterpreted as simultaneous battlefield slots. Each candidate
records its source monster group, group/wave ordinal and exact inherited stable
enemy variant.

All 27 source MonsterIDs reconcile to existing
`content-reference/v4.4/enemy-variants.json` rows, and all 81 source SkillIDs
reconcile to existing `enemy-abilities.json` rows. The Goal 16 pack references
those stable identities and their content digests without copying or modifying
core enemy definitions.

## Score and settlement

Exact source parameters retained:

- monster base score `7000`;
- elite vector `10000,10000,0,0`;
- monster-weight vector `1,1,5,5,1`;
- time vector `2000,20,50`;
- score cap `200000`;
- final-stage extra bonus `5000`;
- scoring group `905`; and
- contribution IDs `90009` (kill), `90010` (Boss HP) and `90011` (time).

The scoring program is represented by whole-file digest plus ability, trigger
and operation identifiers. Intermediate rounding remains the explicitly
labeled fixed-point ProjectPolicy. All six displayed stages retain ordered
`C/B/A/S/SS` thresholds and a single settlement projection boundary.

## ReferenceOnly semantic closure

Departure contributes 17 mechanism families. Each has one rule and one concrete
review fixture with:

- source record IDs and evidence references;
- trigger point and state owner;
- concrete preconditions and input;
- nonempty ordered operations;
- expected facts; and
- explicit evidence/mechanism quality.

All rows are `runtime_executable=false`. The three remaining global families —
Twin synthesis, Supreme synthesis and Galactic Store progression — are
Demon King/Phase 2 obligations and are not falsely satisfied by Departure.

The fragments are intentionally separate:

```text
content-reference/galactic-baseballer-v1/fragments/
  departure-mechanic-rules.json
  departure-review-fixtures.json
```

P3-B1 deterministically aggregates them with Demon King fragments into the
contracted `mechanic-rules.json` and `review-fixtures.json`.

## Verification

```text
node tools/galactic-baseballer-reference/verify-departure-encounters.mjs \
  --source-cache .cache/galactic-baseballer-source
node tools/galactic-baseballer-reference/verify-departure-fixtures.mjs
```

The checks prove exact counts, parent reachability, inherited stable identities,
score vectors, rating order, family set, rule/fixture binding and the
ReferenceOnly boundary.
