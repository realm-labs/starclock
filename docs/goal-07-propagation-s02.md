# Goal 07 Propagation Partition S02

`G07-P2-M09-S02` completes Metabolic Cavity, Excitatory Gland,
Exposed Brain Matter, Intersegmental Membrane, Catalyst and level 1 of
Osseus Blade. All six mechanics lower to generic Rule IR and shared combat
primitives; this partition admits no native handler.

## Authoritative authoring boundary

`Universe.xlsx`, `UniverseBindings.xlsx` and `UniverseEvidence.xlsx` remain
the editable source. They are authored with openpyxl, exported through Sora
0.3.0 and checked against the focused partition golden. Runtime lowering
consumes validated domain definitions and released binding keys, never
workbook rows.

Parameters, trigger order and action flags were verified against the pinned
`ExcelOutput/RogueMazeBuff.json` and
`ConfigAbility/Level/Level_RogueBuff_Ability_DLC1.json`. Released English and
Chinese descriptions were cross-checked in `TextMapEN.json` and
`TextMapCHS.json`, accessed 2026-07-26.

The enhanced Metabolic Cavity parameter is retained upstream as
`0.007999999`, while the public description specifies 0.8%. Formula lowering
normalizes that transcription tail to six decimal places using deterministic
nearest-ties-even rounding, producing exactly `0.008000`.

## Metabolic Cavity

Every Spore consumed by a burst heals the living ally with the lowest current
HP ratio:

```text
L1 healing per consumed Spore = 10% of selected ally maximum HP
L2 healing per consumed Spore = 12% of selected ally maximum HP
L2 ally damage reduction      = 0.8% per Spore held by all enemies
```

The shared Spore engine emits a typed signal after burst damage and before
removing the Spore effect. Its payload is the snapshotted consumed stack
count, so the heal does not depend on the post-removal enemy state.

For level 2, Rule IR observes Spore application, stack change and removal.
It recomputes the global enemy Spore count and mirrors that count into one
permanent effect on every ally. The effect's stack-backed target-side
Mitigation modifiers cover ordinary, DoT, Break, Super Break, Additional,
Joint and Elation damage. This avoids evaluating selector expressions inside
the modifier resolver and keeps the effective value replay-visible.

## Excitatory Gland

At Basic ATK start, the rule snapshots whether the player team has zero Skill
Points. If true, it resolves after that Basic ATK:

```text
L1: recover 1 additional Skill Point
L2: recover 1 additional Skill Point, then make an independent fixed 50%
    roll to recover 1 more
```

The zero-point observation occurs before the Basic ATK's ordinary recovery.
The enhanced roll uses the stable effect-chance RNG stream, and the transient
marker is removed immediately after the bonus recovery.

## Exposed Brain Matter

Each Basic ATK damage event emits nonrecursive Additional damage based on that
event's raw amount:

```text
L1: 30% of original damage to one uniformly selected adjacent enemy
L2: 35% of original damage to every adjacent enemy
element: inherited from the observed damage event
source-side Crit and DMG Boost: not applied a second time
target-side defense, resistance, vulnerability and mitigation: retained
```

`AdjacentToPrimary` is a generic selector predicate. It expands the
primary-target selector to the opposing candidate pool before filtering by
formation distance, so it composes with `RngUniform` instead of being tied to
a special selector choice. Candidate ordering is stable before the draw.
The generated damage excludes this blessing's source, preventing recursion.

## Intersegmental Membrane

Each Skill Point consumed applies one stack to the acting character:

```text
damage reduction per stack = 8%
duration                   = 1 owner turn
L1 maximum stacks          = 2
L2 maximum stacks          = 3
```

The signed Skill Point resource delta supplies the exact number of stacks.
The dispellable owner-turn effect refreshes and adds stacks, and its
stack-backed target-side Mitigation modifiers cover every supported damage
family.

## Catalyst

The rule arms when the owner starts a Skill. Any `Attack`-tagged hit from that
Skill clears the arm. If the Skill resolves while still armed, all allies
gain one stack:

```text
L1 damage bonus per stack = 20%
L2 damage bonus per stack = 30%
duration                  = 1 owner turn for each recipient
maximum stacks            = 3
```

This models the released `Action_IsAttack` flag rather than inferring
non-attacking behavior from damage totals. The stack-backed source-side
DamageBoost modifiers cover all supported damage families.

## Osseus Blade

Level 1 grants Basic ATK damage according to the number of selected
Propagation blessings:

```text
bonus per selected Propagation blessing = 9%
count cap                               = 6
maximum level-1 bonus                   = 54%
```

The contribution compiler counts selected blessings in the validated path
catalog. A BattleStarted program installs the resulting permanent
Basic-tag-filtered modifier on every character. Level 2 belongs to
`G07-P2-M09-S03`.

## Production verification

Focused tests prove:

- all six assigned rules materialize together as generic Rule IR without
  native handlers;
- enhanced Metabolic Cavity mirrors two global Spores as two modifier stacks
  on each of four allies;
- enhanced Intersegmental Membrane and Catalyst retain exact stack caps and
  per-stack values;
- level-1 Osseus Blade compiles six selected Propagation blessings to an
  exact 54% Basic ATK modifier;
- Exposed Brain Matter level 1 selects exactly one adjacent enemy, consumes
  deterministic RNG and emits one Additional-damage event with the original
  element;
- upstream decimal tails use an explicit six-place nearest-ties-even formula
  boundary.

The completion receipt is
`evidence/standard-universe-mechanics-complete-v1/partitions/G07-P2-M09-S02.json`.
