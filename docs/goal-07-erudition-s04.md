# Goal 07 Erudition Partition S04

`G07-P2-M10-S04` completes the Erudition path with both released levels of
Utmost Compression and Recursive Causal Link, Erudition Resonance and its
Melt Core, Chain Contagion and Memetic Inversion Formations. Every assigned
mechanic lowers to shared domain services and generic Rule IR. This partition
admits no native handler.

## Authoritative authoring boundary

`Universe.xlsx`, `UniverseBindings.xlsx` and `UniverseEvidence.xlsx` remain
the editable source. The focused openpyxl verifier reads those formal tables,
checks all assigned rows and provenance, compares them with the Sora 0.3.0
debug tables, and writes the partition golden from the production `.sora`
bundle. Runtime code consumes validated domain definitions and released
binding keys; it does not depend on workbook row types.

Parameters, descriptions and lifecycle details were verified against the
pinned `ExcelOutput/RogueMazeBuff.json`,
`ConfigAbility/Level/Level_RogueBuff_Ability_Erudition.json`,
`BattleEvent/Avatar_RogueBattleevent_Erudition_S1_Ability.json`,
`BattleEvent/Avatar_RogueBattleevent128_Ability.json`, and the released
English and Chinese text maps. The source snapshot was accessed on
2026-07-26. All values are exact; this partition records no numeric
approximation.

## Utmost Compression

After a character's Ultimate fully resolves, the character restores:

```text
healing = character MaxHP × 16%          level 1
healing = character MaxHP × 24%          level 2
```

The trigger excludes Path Resonance and uses the normal healing formula
pipeline. Each character owns its own rule instance and receives only its own
restoration.

## Recursive Causal Link

When a character would receive lethal damage, the shared team-once defeat
guard keeps that character alive and emits a typed guard signal. The guarded
character then consumes all current Energy and restores:

```text
healing = MaxHP × current Energy / maximum Energy × 50%     level 1
healing = MaxHP × current Energy / maximum Energy × 100%    level 2
```

The heal is an exact post-guard restoration and therefore does not apply
healing modifiers. The Energy snapshot is consumed after the heal expression
is evaluated. A shared effect source makes every character's guard disappear
after the first team activation. Rules attach only to player characters, so
servants and memosprites neither receive nor consume the guard.

## Erudition Resonance and Synapse

The Resonance costs 100 Resonance Energy. It applies a permanent, shared
15-charge Synapse effect to every present enemy and then deals Imaginary
damage equal to 70% of each target's current maximum HP.

After a character Attack hits at least one linked enemy:

1. choose one linked enemy from the committed damage targets with the labeled
   deterministic random stream;
2. choose the living linked enemy with the highest maximum HP, breaking ties
   by stable ID;
3. deal 30% of the attacking character's ATK to both selected targets; and
4. consume one shared Synapse charge from every linked enemy.

The two damage instances use the attacker's Basic element and Ultimate damage
semantics for modifier queries. They remain auxiliary Rule IR operations:
they do not open an Ultimate lifecycle, spend character Energy, or trigger
rules that listen for a character using an Ultimate. If both selectors resolve
to the same enemy, that enemy receives both instances.

The Resonance action carries the dedicated `path_resonance` tag. This keeps
interrupt legality explicit while preventing character Attack and Ultimate
listeners from observing the Resonance as a character action.

## Resonance Formations

Melt Core adds 50% of the attacking character's ATK to the highest-MaxHP
Synapse target when the triggering action is an Ultimate. Its damage uses the
same element and Ultimate semantics as the base Synapse hit.

Chain Contagion triggers when a linked enemy is defeated and deals two
additional 30%-ATK Synapse instances to the highest-MaxHP surviving linked
enemy. These propagation instances do not consume the 15 shared charges.
When the defeating action is an Ultimate and Melt Core is selected, each
propagated instance also receives the 50%-ATK Melt Core share.

Memetic Inversion restores Resonance Energy whenever enemies appear:

```text
energy per appearing enemy =
    5% × sum(maximum Energy of all player characters)
```

Battle entry and later wave entry use grouped deterministic selectors.
Mid-battle summons use the generic `UnitSummoned` event point, so encounter
scripts do not need an Erudition-specific callback.

## Production verification

Focused tests prove:

- all ten assigned records and rules materialize through generic Rule IR;
- both exact Ultimate-healing levels and the shared lethal-guard contract are
  preserved structurally;
- production Resonance activation applies 15 shared Synapse charges and the
  next legal character Attack consumes exactly one;
- Melt Core authors the exact 50% share, Chain Contagion authors two exact
  30% shares, and Memetic Inversion authors the exact 5% appearance value;
  and
- neither the Resonance action nor its generated damage enters a character
  Ultimate lifecycle.

The completion receipt is
`evidence/standard-universe-mechanics-complete-v1/partitions/G07-P2-M10-S04.json`.
