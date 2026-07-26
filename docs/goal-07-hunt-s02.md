# Goal 07 Hunt Partition S02

`G07-P2-M06-S02` completes sixteen assigned content records and sixteen
mechanic-rule records for the effective levels of Adept's Bow, Mistwraith
Pursuit, Starlit Hunt, Borisin Chase and Rainbow Fang, plus the first released
level of Vermeil Bow and White Arrow. All behavior is generic Rule IR and no
native handler is admitted.

## Authoritative authoring boundary

The editable source is the openpyxl-authored workbook set:

- `Universe.xlsx` owns the Blessing, level and exact parameter rows;
- `UniverseBindings.xlsx` owns all sixteen assigned mechanic bindings;
- `UniverseEvidence.xlsx` owns provenance, audit and review data.

`tools/goal07/author-path-partition.py` loads those files with openpyxl,
rejects formula and error cells, and proves semantic parity with the committed
Sora 0.3.0 binary and debug exports. Runtime materialization consumes only the
validated `.sora` bundle.

Released structured rows remain the numeric source of record. Public
mechanism descriptions were cross-checked against the
[Simulated Universe Paths reference](https://honkai-star-rail.fandom.com/wiki/Simulated_Universe/Paths)
and the
[Star Rail Station Blessing catalog](https://starrailstation.com/en/simuniverse/current/blessings),
accessed 2026-07-26.

## Critical Boost inheritance

Adept's Bow (`612442`) observes allied action starts. An Ultimate inherits the
party's current Critical Boost stack count and adds one stack. At the enhanced
level, follow-up attacks use the same transition. The rule does nothing when
no Critical Boost exists.

Inheritance is one ordered operation:

1. read the aggregate stack count from the immutable event snapshot;
2. remove the shared effect from its prior holder;
3. apply the inherited count plus one to the acting ally.

The shared S01 lifecycle now also transfers Critical Boost at ordinary allied
turn starts and clears it when an ally is attacked. Adept's Bow does not own a
parallel status or counter.

## Consecutive actions

Mistwraith Pursuit (`612443`) stores the previous allied turn actor as an
optional stable ID. When the next allied turn belongs to the same actor, that
actor gains one permanent sequence-local ATK stack:

```text
per stack: +40% ATK
maximum: 2 stacks
```

When a different ally begins a turn, the sequence effect is removed before the
new actor is remembered. The enhanced level also makes one labeled 50% fixed
chance roll per qualifying turn; success restores one team Skill Point through
the ordinary checked resource service.

## Defeat, Break and turn-cycle rewards

Starlit Hunt (`612444`) restores personal Energy to the character credited
with an enemy defeat:

```text
L1: 60% of that character's Max Energy
L2: 100% of that character's Max Energy
```

The core Rule IR now exposes a read-only `QueryMaximumEnergy` expression. It
reads the resolved combatant snapshot and keeps Energy limits out of Universe
content code. Resource mutation still owns cap enforcement and event
publication.

Rainbow Fang (`612446`) restores 48% MaxHP to the character credited with an
enemy defeat. The enhanced level additionally restores 18% MaxHP when that
character inflicts Weakness Break. Both are normal Heal operations, so shared
healing modifiers, clamping and attribution remain authoritative.

Borisin Chase (`612445`) owns one team-wide, bounded six-turn counter. At the
end of every sixth allied turn, the current actor receives a 100% normal
action advance. The enhanced level initializes the counter at five. A
previous-beneficiary guard excludes the immediately advanced turn from the
next cycle, then clears itself so single-character teams can start another
six-turn cycle.

## Blessing-count speed

Vermeil Bow and White Arrow (`612450`) is lowered from the validated number of
selected Hunt Blessings:

```text
L1: +3% SPD per Hunt Blessing, at most 6 stacks
L2: +4% SPD per Hunt Blessing, at most 9 stacks
```

The current partition owns the L1 record; the same lowering accepts the L2
effective row assigned to S03. The result is one percent-of-base SPD modifier,
not a mode-specific runtime branch.

## Production verification

Production tests prove:

- every selected S02 effective level materializes as native-handler-free Rule
  IR;
- enhanced Starlit Hunt restores exactly one complete Max Energy value after a
  real enemy defeat;
- Rainbow Fang heals exactly 48% MaxHP on the same production defeat;
- enhanced Mistwraith Pursuit retains the 40%, two-stack and 50% fixed-chance
  policies;
- enhanced Borisin Chase starts at five and advances at the sixth allied
  turn-end boundary;
- enhanced Adept's Bow owns both Ultimate and follow-up inheritance triggers;
- the shared Critical Boost definition remains singular when S01 and S02
  Blessings are selected together.

The completion receipt is
`evidence/standard-universe-mechanics-complete-v1/partitions/G07-P2-M06-S02.json`.
