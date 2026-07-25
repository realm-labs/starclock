# Goal 07 Nihility Partition S03

`G07-P2-M04-S03` completes sixteen content records, sixteen mechanic-rule
records and eleven native-handler reviews. It executes the third assigned
Nihility group: enhanced Ignosticism and both levels of Questioning of
Purpose, Blind Vision, Tragic Lecture, Sensory Labyrinth and Emotional
Decluttering. No native handler is admitted.

## Authoritative authoring boundary

The formal source remains the openpyxl-authored workbook set:

- `Universe.xlsx` owns the Blessing definitions, levels and exact parameters;
- `UniverseBindings.xlsx` owns the sixteen mechanic bindings;
- `UniverseEvidence.xlsx` owns provenance and all native reviews.

`tools/goal07/author-path-partition.py` reads those production workbooks with
openpyxl, rejects formula/error cells and compares the assigned rows with the
committed Sora 0.3.0 binary and debug exports. Runtime materialization reads
only the validated `.sora` bundle.

## Executed Blessings

### Ignosticism level 2 (`612250`)

Each selected Nihility Blessing increases all allies' DoT damage by 8%, up to
nine counted Blessings:

```text
DoT increase = 8% * min(selected Nihility Blessings, 9)
maximum = 72%
```

Selection count is supplied by the validated contribution compiler and frozen
when the battle contribution is materialized.

### Questioning of Purpose (`612251`)

All allies gain initial and base Break-effect damage through the ordinary
Break-purpose formula stage:

```text
L1: +50% Break damage
L2: +75% Break damage
```

The modifier is additive with the formula's existing Break-damage-increase
term. It does not alter Toughness reduction or Super Break.

### Blind Vision (`612252`)

Every enemy receives a persistent Effect RES reduction:

```text
L1: -12% Effect RES
L2: -18% Effect RES
```

The signed value enters the shared effect-chance stat query. It is not encoded
as a Nihility-specific probability override.

### Tragic Lecture (`612253`)

Every enemy receives a target-side DoT vulnerability:

```text
L1: +10% DoT damage taken
L2: +15% DoT damage taken
```

The modifier uses `FormulaPurpose::Dot` and the target formula subject, so
ordinary direct, Break and generated non-DoT damage remain unaffected.

### Sensory Labyrinth (`612254`)

New Bleed, Burn, Wind Shear and Shock effects applied to enemies gain:

```text
L1: +1 target turn
L2: +2 target turns
```

The duration addition is queried at effect application and applied before the
generic negative-effect duration multiplier. Existing DoTs are not
retroactively extended. A production Kafka Ultimate verifies that its normal
two-turn Shock becomes four turns at L2.

### Emotional Decluttering (`612255`)

An enemy's damage taken increases with the total current stack count of all
DoT-category effect instances attached to that enemy:

```text
L1: +3% per DoT stack, at most 4 stacks = +12%
L2: +4% per DoT stack, at most 5 stacks = +20%
```

This is a dynamic target-side vulnerability over ordinary, DoT, Break, Super
Break, additional, joint and Elation damage. The query sums active DoT stacks
at formula evaluation time, then clamps the contribution. Removing, adding or
restacking a DoT changes the next damage calculation without mutating the
Blessing effect itself.

## Shared combat additions

This partition adds no content-ID branch to combat core. It extends the shared
domain behavior with:

- an effect-category stack query available to validated value expressions;
- modifier-resolution snapshots that expose current category-stack totals;
- a generic integral DoT-duration stat applied at effect creation;
- modifier-aware Initial Break, Break-effect and Super Break formula
  preparation for their own `FormulaPurpose`;
- action-aware source damage boost and target vulnerability/mitigation
  evaluation for Break-family damage.

The category query uses the transaction's current effect state, stable
instance iteration and checked integer accumulation. The fixed-point backend
remains private, and all new formula preparation occurs inside the same
transaction as the resulting damage.

## Production verification

The production Kafka form and formal Goal 07 materialization prove:

- all sixteen assigned records and rules materialize from Excel/Sora;
- enhanced Ignosticism caps at exactly 72%;
- Questioning of Purpose increases a real initial Lightning Break by exactly
  50% at L1;
- enhanced Sensory Labyrinth turns Kafka's two-turn Shock into four turns;
- Emotional Decluttering reads current DoT-category stacks, clamps the count
  and emits the exact 12%/20% maxima;
- all eleven exceptional candidates close as `IrSufficient`.

The completion receipt is
`evidence/standard-universe-mechanics-complete-v1/partitions/G07-P2-M04-S03.json`.
