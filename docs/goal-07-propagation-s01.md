# Goal 07 Propagation Partition S01

`G07-P2-M09-S01` establishes the shared Spore lifecycle and completes Spore
Discharge, Fungal Pustule, Scythe Limbs, Putrefaction Ulcer and Lytic Enzyme.
The partition also retains the released definition of Excitatory Gland; its
level implementations belong to the next fixed partition.

## Authoritative authoring boundary

`Universe.xlsx`, `UniverseBindings.xlsx` and `UniverseEvidence.xlsx` remain
the editable source. They are authored with openpyxl, exported through Sora
0.3.0 and checked against the focused partition golden. Runtime lowering
consumes validated domain records and released binding keys, never workbook
rows.

Parameters and modifier behavior were verified against the pinned
`ExcelOutput/RogueMazeBuff.json`,
`ConfigAbility/Level/Level_RogueBuff_Ability_DLC1.json` and the shared
`MLevel_Egg_Base_Rogue`, `MLevel_Egg_Father_Rogue`,
`MLevel_Egg_Son_Rogue` and `MLevel_Egg_Damage_Rogue` modifier definitions.
Released English and Chinese descriptions were cross-checked in
`TextMapEN.json` and `TextMapCHS.json`, accessed 2026-07-26.

The public rows expose every S01 coefficient except the private neutral
BattleEvent level-stat factor used as the Spore damage base. Starclock uses
the actor's level-derived `BreakBaseDamage` for that factor. This is a
reviewable numeric approximation; the threshold, stack count, element,
damage class, bonus exclusion, target lifecycle and spread behavior remain
authoritative.

## Shared Spore contract

Spore is a permanent, non-dispellable enemy debuff:

```text
default maximum stacks = 6
Fungal Pustule L2 maximum stacks = 9
burst threshold = 3 stacks
burst trigger = the affected enemy is attacked by a character
burst damage = level base factor × consumed Spore stacks
element / class = Wind / Additional
critical hits = disabled
source-side damage bonuses = ignored
```

The trigger runs at most once for a target within one action. A successful
burst snapshots the current stack count, applies the damage, consumes the
Spore instance and then performs the configured spread. The unboosted-damage
operation skips source-side Crit, DMG Boost and Weaken stages but deliberately
retains target defense, resistance, vulnerability, mitigation and
broken-state stages.

Spore application uses refresh-and-add stacking. Stable target ordering is
established before a random draw. Within one spread group, targets are chosen
without replacement; the candidate set resets for the next group.

## Skill Point blessings

Spore Discharge applies one Spore to every present, living enemy for every
Skill Point consumed:

```text
L1: one application per point
L2: one application per point; consuming all remaining Skill Points also
    grants the acting character +20% SPD for 2 owner turns
```

Fungal Pustule applies one Spore to two random enemies for every Skill Point
recovered:

```text
L1: two distinct random enemies per recovered point; maximum Spore remains 6
L2: the same application contract; maximum Spore becomes 9
```

The generic `RandomGroupedEffect` Rule IR operation represents this exactly:
the recovered-point delta is the group count, every group selects up to two
distinct enemies, and enemies become eligible again for the next recovered
point. RNG draws are journaled with a stable purpose and replay counter.

## Scythe Limbs

After the character uses an Ultimate, the next applicable Skill Point change
is accounted as one additional point:

```text
L1 applicable change = consumption
L2 applicable change = consumption or recovery
effective points = min(actual points + 1, 2)
CRIT DMG per effective point = +40% / +45%
lifetime = through the character's next attack
```

The armed state and CRIT DMG effect are owned per character, not globally.
When Spore Discharge or Fungal Pustule is selected, the additional accounted
point also enters that blessing's normal Spore-application route. The
temporary CRIT DMG effect is removed only after an action carrying the Attack
tag resolves.

## Spore lifecycle blessings

Putrefaction Ulcer changes the ordinary burst spread:

```text
L1 spread applications = 2
L2 spread applications = 3
original bearer = eligible for the spread pool
```

Lytic Enzyme changes damage and defeat settlement:

```text
L1 Spore burst damage bonus = +35%
L2 Spore burst damage bonus = +50%
defeated bearer spread = adjacent enemies / all other enemies
```

The defeat route reads the stack snapshot saved before normal defeat
settlement, then spreads at `AfterDefeatSettlement`. It cannot resurrect or
retarget the defeated unit and uses the same deterministic grouped-selection
primitive as ordinary Spore spread.

## Production verification

Focused tests prove:

- all five implemented blessings materialize as executable Rule IR without
  native handlers;
- enhanced Fungal Pustule raises the shared stack cap to nine;
- grouped random target selection is without replacement inside each group;
- a production Skill Point recovery applies one Spore to each of two enemies
  and records deterministic RNG consumption;
- enhanced Scythe Limbs keeps state per owner, handles both spend and recovery,
  grants exactly 45% CRIT DMG per accounted point and expires after an attack;
- the shared engine emits non-critical Wind Additional damage through the
  source-bonus-excluding formula route and observes the released threshold and
  spread policies.

The completion receipt is
`evidence/standard-universe-mechanics-complete-v1/partitions/G07-P2-M09-S01.json`.
