# Goal 16 Demon King Encounter and Score Audit

`G16-P2-B4` closes Version 4.4 Demon King encounters, shared enemy identity
references, team bonuses, Devil boss phases, score and settlement data. It
adds no battle handler or executable gameplay.

## Exact encounter closure

All 56 authored stage-period rows resolve to one of 18 exact shared
`StageConfig` rows. Recursion through stable numeric references closes:

| Family | Demon King rows |
|---|---:|
| Shared StageConfig encounters | 18 |
| Ordered infinite waves | 61 |
| Ordered enemy candidate positions | 1,573 |
| Inherited enemy variants | 77 |
| Inherited enemy abilities | 258 |
| Reachable status locators | 10 |

The candidate positions preserve `StageInfiniteMonsterGroup.MonsterList`
order and are labeled `OrderedCandidateNotAssumedSimultaneousSlot`; source
array position is not reinterpreted as a simultaneous spawn slot or hidden
weight.

Every MonsterID resolves exactly to an existing frozen Version 4.4 stable
enemy variant and every SkillID resolves to an existing stable enemy ability.
The Goal 16 rows reference those identities and their source receipts without
copying or modifying core combat definitions. The ten recursively reachable
MonsterStatus rows retain exact modifier, type, dispel flag, source hashes and
parameter-name locators in the isolated reference partition.

## Team bonuses and complete MazeBuff ownership

The seven Demon King stages each resolve to one distinct level-one team-bonus
MazeBuff and one exact structural binding in
`EvolveBuild_07_TeamBonus.json`:

| Stage | MazeBuff |
|---|---:|
| Initial Planet | `3113607` |
| V612 — Volcanic Planet | `3113602` |
| C996 — Cogwheel Planet | `3113601` |
| F233 — Sugarfrost Planet | `3113603` |
| M078 — Miniature Planet | `3113604` |
| D007 — Blissdream Planet | `3113605` |
| Demon King's Den | `3113606` |

The structural rows retain binding keys, parameter vectors, trigger-event and
operation-type sets, and canonical program-fragment digests. They are not
runtime executable.

These seven rows finish exact ownership of all 315 `EvoBdSCMazeBuff` rows:
198 weapon/accessory levels, 56 Adventure Strategies, 54 Cosmic Store effect
levels and seven team bonuses. No row is removed from the denominator.

## Score and settlement

The Demon King profile repeats the exact base score `7000`, elite vector
`10000,10000,0,0`, monster-weight vector `1,1,5,5,1`, score cap `200000` and
final-stage bonus `5000`. It changes the exact scoring identities to group
`913`, monster-kill `90019`, boss-HP `90020` and time `90021`.

Unlike Departure, the Demon King constant table does not publish the
`Score_Time` parameter vector. The normalized field is therefore `null`; the
Departure vector is not inherited by assumption. Intermediate rounding
remains an explicit ReferenceOnly ProjectPolicy boundary.

All seven stages have ordered `C/B/A/S/SS` thresholds and one settlement row.
The clear boundary projects one result after the authored terminal period and
rejects projection if a required period remains unresolved.

## D007 released correction

The official Version 3.4 notice states that abnormal Adventure Score
acquisition on D007 was corrected but does not publish the obsolete trigger
or mutation. The retained fixture binds only Version 4.4 facts:

- terminal period `424053`;
- special MonsterID `403202302` contributes `3000`;
- stage score is `4500`;
- period score is `45000`;
- the obsolete abnormal path is not modeled.

Guessing a link to the D007 team bonus or enemy composition and retaining the
obsolete defect as an optional rule remain rejected alternatives with an
explicit released-trace replacement condition.

## Demon King boss phases

`Demon King's Den` owns 39 authored ordered periods. Its boss-phase row binds
Adventure Strategy `3113799` / level ID `31137991` and the whole
`EvolveBuildSC_11_Devil.json` program receipt:

- 40 named abilities;
- 112 structural operation types;
- battle-local Devil state ownership;
- no executable program import.

The semantic fixture preserves period order, finalizes the exact boss-HP
contribution, applies the final-stage bonus once, caps the score, selects the
rating and projects settlement once.

## Semantic review

Five ReferenceOnly rules and six concrete fixtures cover:

- stage/difficulty selection;
- wave and battle-phase progression;
- team-bonus installation and teardown;
- score, rating and clear;
- boss phase and final settlement;
- the dedicated D007 retained-correction case.

Every rule names its trigger, state owner, preconditions and ordered
operations. Every fixture includes concrete input and expected facts, source
record IDs, evidence references and `runtime_executable=false`.

## Reproduction

```text
node tools/galactic-baseballer-reference/normalize-departure-encounters.mjs \
  --profile demon-king \
  --source-cache .cache/galactic-baseballer-source
node tools/galactic-baseballer-reference/normalize-departure-encounters.mjs \
  --profile demon-king --check \
  --source-cache .cache/galactic-baseballer-source
node tools/galactic-baseballer-reference/normalize-demon-encounter-fixtures.mjs
node tools/galactic-baseballer-reference/normalize-demon-encounter-fixtures.mjs \
  --check
node tools/galactic-baseballer-reference/verify-demon-encounters.mjs \
  --source-cache .cache/galactic-baseballer-source
```

P3-B1 will merge the isolated fragments into the contracted combined encounter,
enemy-reference, score, settlement, rule and fixture tables.
