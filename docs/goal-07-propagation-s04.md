# Goal 07 Propagation Partition S04

`G07-P2-M09-S04` completes Sporangium, Vesicle and the released Propagation
Path Resonance with Proboscis, Phenol Compounds and Crystal Pincers. All
assigned mechanics lower to generic Rule IR, modifier, effect, resource,
timeline and queued-action primitives. This partition admits no native
handler.

## Authoritative authoring boundary

`Universe.xlsx`, `UniverseBindings.xlsx` and `UniverseEvidence.xlsx` remain
the editable source. They are authored with openpyxl, exported through Sora
0.3.0 and checked against the focused partition golden. Runtime lowering
consumes validated domain definitions and released binding keys, never
workbook rows.

Parameters and lifecycle behavior were verified against the pinned
`ExcelOutput/RogueMazeBuff.json`,
`ConfigAbility/Level/Level_RogueBuff_Ability_DLC1.json` and
`ConfigAbility/BattleEvent/Avatar_RogueBattleevent127_Ability.json`.
Released English and Chinese descriptions were cross-checked in
`TextMapEN.json` and `TextMapCHS.json`, accessed 2026-07-26. Every value in
this partition is an exact public value; there are no numeric
approximations.

## Sporangium and Vesicle

Both blessings observe a character's negative Skill Point resource delta.
The magnitude is the number of Skill Points spent by that character:

```text
Sporangium:
  L1 restore 3 Energy per Skill Point spent
  L2 restore 4 Energy per Skill Point spent

Vesicle:
  L1 restore HP equal to 10% Max HP per Skill Point spent
  L2 restore HP equal to 15% Max HP per Skill Point spent
```

Energy recovery uses the normal checked Energy mutation without energy
regeneration scaling. Healing reads the owner's live maximum HP at the
trigger boundary and enters the generic healing operation. Zero and positive
Skill Point deltas do not trigger either blessing.

## Base Propagation Resonance

The manual Path Resonance costs exactly 100 points of the keyed Propagation
resonance resource and selects one living allied character. Resolution is:

```text
1. Remove Metamorphosis from every allied character.
2. Apply Metamorphosis to the selected character.
3. Recover 2 team Skill Points.
4. Advance the selected character by 100%.
```

The explicit remove-then-apply sequence preserves the released
latest-target-only contract. The resonance is an allied assist action, not
an attack, and therefore does not manufacture damage or attack triggers.
Without Proboscis, Metamorphosis lasts one owner turn.

## Proboscis

Proboscis changes Metamorphosis to two owner turns. When a character under
Metamorphosis defeats an enemy, the player team restores exactly 20% of the
current Propagation resonance maximum:

```text
maximum 100 -> 20 Resonance Energy
maximum 200 -> 40 Resonance Energy
```

The trigger executes after defeat settlement and uses the event actor, so a
defeat caused by another character cannot borrow the holder's effect.

## Phenol Compounds

Phenol Compounds increases the keyed resonance resource maximum from 100 to
200. Every Skill Point spent or recovered restores resonance energy equal to
1% of the 200-point maximum per point, or exactly two energy:

```text
1 Skill Point changed -> 2 Resonance Energy
2 Skill Points recovered by Resonance -> 4 Resonance Energy
```

The resource event is processed after the original Skill Point mutation and
uses its signed fixed-point delta. The resonance's own two-point recovery
therefore participates in the formation effect deterministically.

## Crystal Pincers

Crystal Pincers adds 40% DMG to the Metamorphosis holder for ordinary, DoT,
Break, Super Break, Additional, Joint and Elation damage formula purposes.
Only that holder can burst enemy Spore stacks.

Each Spore burst emits the consumed stack snapshot. Crystal Pincers caps that
snapshot at three, stores it in an action-end marker on the same enemy and
queues an auxiliary action owned by the Metamorphosis holder:

```text
extra damage = holder ATK * 80% * min(consumed Spore stacks, 3)
```

The auxiliary action carries the generic `basic` and `additional-damage`
tags, so Basic ATK formula modifiers apply. Its action kind remains
`ExtraAction` and it deliberately omits the generic attack tag; it cannot
produce a second Basic ATK action, pay action resources or recursively fire
attack-family triggers. The new generic
`DamageFromActorBasicElement` operation resolves the damage element from the
actor's authored Basic ability and faults on an absent or ambiguous element.

## Production verification

Focused tests prove:

- all ten assigned records and rules materialize without native handlers;
- the two blessings listen only to Skill Point resource changes;
- the resonance remains a single-allied-target assist action with an exact
  two-Skill-Point gain and full action advance;
- enhanced Metamorphosis lasts two owner turns and carries the exact 40%
  ordinary DamageBoost;
- Crystal Pincers is represented by an auxiliary ExtraAction whose Basic tag
  participates in formulas without impersonating a Basic attack action;
- Phenol Compounds materializes a 200-point maximum;
- executing a charged production resonance leaves exactly one
  Metamorphosis holder and recharges four resonance energy from the two Skill
  Points it restores.

The completion receipt is
`evidence/standard-universe-mechanics-complete-v1/partitions/G07-P2-M09-S04.json`.
