# Goal 07 Erudition Partition S02

`G07-P2-M10-S02` completes the assigned levels of Implant: Explicit Memory,
Mimesis: Tactile Pathway, Analysis: Subliminal Sensation, Load: Striated
Cortex, Stimulation: Saltatory Conduction and Throne of Engaged Gears. Every
mechanic lowers to generic Rule IR, selectors, state slots, effects and
modifiers. This partition admits no native handler.

## Authoritative authoring boundary

`Universe.xlsx`, `UniverseBindings.xlsx` and `UniverseEvidence.xlsx` remain
the editable source. They are read and verified with openpyxl, exported by
Sora 0.3.0, and compared with the focused partition golden. Runtime code
consumes the validated domain catalog and released binding keys rather than
workbook rows.

Parameters and lifecycle behavior were verified against the pinned
`ExcelOutput/RogueMazeBuff.json`,
`ConfigAbility/Level/Level_RogueBuff_Ability_Erudition.json`, the matching
global `MLevel_Rogue_Knowledge` configurations, and released English and
Chinese text maps. The source snapshot was accessed on 2026-07-26. This
partition requires no numeric approximation.

Explicit Memory L1 and L2 are assigned to this partition by the frozen
exact-once ledger. Their executable implementation is the shared Brain in a
Vat slice introduced by S01: 36% Max HP for two owner turns at L1 and 45% for
three owner turns at L2.

## Generic target-shape observation

An ability's target shape belongs to its catalog selector, not to its action
family. Rule event facts therefore expose the authored `TargetPattern` and
`EventFilter` can require `Single`, `Blast` or `All`.

This is a reusable combat-core seam. It lets Striated Cortex identify a real
AoE ability without character IDs, synthetic tags, controller target counts
or source-game data structures. The fact is derived from the immutable
ability-to-selector relationship and remains identical for every event in
the action's cause chain.

## Mimesis: Tactile Pathway

After an owner resolves an Attack, the complete committed action-target
snapshot receives one Additional-DMG operation:

```text
L1 amount per target = owner ATK × 15% × attacked-target count
L2 amount per target = owner ATK × 20% × effective-target count

effective-target count =
    min(attacked targets + enemies defeated this battle, 5)
```

The L2 defeated-enemy count is a bounded per-owner battle slot. Defeat
settlement increments it independently of the defeating actor. Action
snapshots retain attacked targets even when an earlier hit defeated them.
The generated damage inherits the resolved action's element, uses the
ordinary Additional-DMG formula and may CRIT. Source exclusion prevents
recursive activation.

## Analysis: Subliminal Sensation

Battle entry applies an Ultimate-only DamageBoost effect and restores a
percentage of live maximum Energy:

```text
                         L1       L2
Ultimate DMG             +50%     +50%
entry Energy             60%      100%
bonus lifetime           first    permanent
                         Ultimate
```

L1 removes the effect only after the owner's first Ultimate fully resolves,
so every hit in that Ultimate receives the bonus. L2 omits the removal
trigger. Energy restoration uses the ordinary checked personal-resource
operation and reports overflow normally.

## Load: Striated Cortex

For an authored `All` target-pattern ability that commits exactly one enemy,
Rule IR accumulates the original action's committed raw damage in an
action-scoped scalar slot:

```text
L1 fixed follow-up = accumulated original damage × 40%
L2 fixed follow-up = accumulated original damage × 60%
```

Only ordinary ability damage contributes. Rule-generated damage and other
damage classes are excluded. At action resolution the fixed fraction is
applied once to the sole committed target as already-resolved True Damage;
source and target formula stages are not applied a second time. The slot is
reset at every action start, making multi-hit accumulation explicit and
preventing cross-action leakage.

## Stimulation: Saltatory Conduction

Each enemy owns an independent bounded trigger counter:

```text
                         L1       L2
Ultimate-hit delay       16%      24%
triggers per broken
period                    3        3
```

When that enemy becomes Weakness Broken, the counter resets to zero. An
opposing character's Ultimate damage against the currently broken enemy
delays that enemy once per action and increments the counter. Multi-hit
Ultimates do not consume multiple charges for the same target. Attaching the
rule to every enemy supplies target-local state without a map-valued slot or
native handler.

## Throne of Engaged Gears

The contribution compiler counts selected Erudition Blessings and compiles
the capped value into a permanent Ultimate-only DamageBoost modifier:

```text
L1 Ultimate DMG = +7% × min(selected Erudition Blessings, 6)
```

The six selected Blessings in the focused fixture therefore produce exactly
42%. The enhanced level belongs to the next frozen partition and is not
claimed by this receipt.

## Production verification

Focused tests prove:

- all sixteen assigned records and rules materialize with generic Rule IR;
- Striated Cortex filters the catalog-authored `All` target pattern,
  accumulates raw action damage and emits the exact 60% fixed fraction;
- Subliminal Sensation authors exact 50% Ultimate damage and 60% entry
  Energy for L1;
- Engaged Gears caps six selected Blessings at exactly 42%;
- Tactile Pathway authors exact 20%-per-target damage and a total-count cap
  of five; and
- Saltatory Conduction authors exact 24% delay, a three-trigger bound and a
  Weakness-Break reset for every enemy-local rule instance.

The completion receipt is
`evidence/standard-universe-mechanics-complete-v1/partitions/G07-P2-M10-S02.json`.
