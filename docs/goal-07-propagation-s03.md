# Goal 07 Propagation Partition S03

`G07-P2-M09-S03` completes level 2 of Osseus Blade and both released
levels of Spinal Spur, Channeled Needle, Conjunctiva, Scaled Wing and
Compound Eye. All mechanics lower to generic Rule IR, modifier, effect,
resource and lifecycle primitives. This partition admits no native handler.

## Authoritative authoring boundary

`Universe.xlsx`, `UniverseBindings.xlsx` and `UniverseEvidence.xlsx` remain
the editable source. They are authored with openpyxl, exported through Sora
0.3.0 and checked against the focused partition golden. Runtime lowering
consumes validated domain definitions and released binding keys, never
workbook rows.

Parameters and lifecycle behavior were verified against the pinned
`ExcelOutput/RogueMazeBuff.json` and
`ConfigAbility/Level/Level_RogueBuff_Ability_DLC1.json`. Released English and
Chinese descriptions were cross-checked in `TextMapEN.json` and
`TextMapCHS.json`, accessed 2026-07-26. Every value in this partition is an
exact public value; there are no numeric approximations.

## Osseus Blade

The shared implementation from S02 reads the selected level and validated
Propagation-blessing count:

```text
L1: 9% Basic ATK DMG per blessing, at most 6 blessings
L2: 12% Basic ATK DMG per blessing, at most 9 blessings
```

The resulting permanent source-side DamageBoost modifier is filtered by the
generic `basic` ability tag. With the six selected blessings in the focused
fixture, level 2 produces an exact 72% bonus.

## Spinal Spur and Channeled Needle

Both blessings install permanent source-side stat modifiers on each player
character:

```text
Spinal Spur:
  L1 Basic ATK CRIT Rate +24%
  L2 Basic ATK CRIT Rate +36%

Channeled Needle:
  L1 Basic ATK CRIT DMG +40%
  L2 Basic ATK CRIT DMG +60%
```

The modifiers participate in the ordinary critical-profile query only when
the action carries the generic `basic` tag. They therefore do not alter the
unit's displayed baseline critical stats for Skills, Ultimates, follow-ups or
other action families.

## Conjunctiva and Scaled Wing

After a character resolves a Basic ATK, a replace-stacking dispellable effect
is applied to that character:

```text
Conjunctiva:
  L1 DEF +40% for 1 owner turn
  L2 DEF +40% for 2 owner turns

Scaled Wing:
  L1 SPD +16% for 1 owner turn
  L2 SPD +16% for 2 owner turns
```

The trigger filters the authoritative `ActionResolved` event by the Basic
action family. DEF and SPD use the normal percent-of-base stat stage. Reusing
the same effect refreshes and replaces it instead of adding stacks, matching
the released modifier graph's `Stacking: Replace` behavior.

## Compound Eye

After each living player character's turn ends, the player team gains one
Skill Point:

```text
L1 shared battle cap = 3 recoveries
L2 shared battle cap = 5 recoveries
```

The rule is attached once to the first player, selects the current allied
turn actor and stores one battle-scoped bounded counter. This is deliberately
not one counter per character: the released description states that all
allies share the trigger limit. `TurnEnded` excludes Ultimates, follow-ups and
other out-of-turn actions while retaining ordinary Basic and Skill turns.
Team resource mutation remains checked and respects the battle's maximum
Skill Point capacity.

## Production verification

Focused tests prove:

- all six assigned blessing mechanics materialize as generic Rule IR without
  native handlers;
- enhanced Basic-only modifiers retain exact 36% CRIT Rate, 60% CRIT DMG and
  72% six-blessing Osseus values;
- enhanced Conjunctiva and Scaled Wing retain exact 40% DEF, 16% SPD and
  two-owner-turn replace-effect definitions;
- a production Basic ATK emits both timed effect applications;
- Compound Eye produces exactly one additional team Skill Point after an
  allied turn compared with the same battle without the blessing;
- Compound Eye uses one battle-scoped counter with the exact enhanced cap of
  five.

The completion receipt is
`evidence/standard-universe-mechanics-complete-v1/partitions/G07-P2-M09-S03.json`.
