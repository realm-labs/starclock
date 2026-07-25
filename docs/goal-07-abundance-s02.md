# Goal 07 Abundance Partition S02

`G07-P2-M05-S02` completes sixteen content records, sixteen mechanic-rule
records and eleven native-handler reviews. It executes both levels of
Salvation From Damnation, Candlelight Radiance, Bitter Is the Bane, Corporeal
Pellucidity and Prajna Voyage, plus level 1 of Dharma Rain. No native handler
is admitted.

## Authoritative authoring boundary

The formal source remains the openpyxl-authored workbook set:

- `Universe.xlsx` owns the Blessing definitions, levels and exact parameters;
- `UniverseBindings.xlsx` owns all sixteen assigned mechanic bindings;
- `UniverseEvidence.xlsx` owns provenance and native-handler reviews.

`tools/goal07/author-path-partition.py` opens those production workbooks with
openpyxl, rejects formula/error cells and compares the assigned rows with the
committed Sora 0.3.0 binary and debug exports. Runtime materialization reads
only the validated `.sora` bundle.

## Dewdrop cleanse

Salvation From Damnation (`612342`) reacts to the owner-scoped Dewdrop rupture
signal introduced by S01. Each rupture performs one fixed-chance draw and
removes the oldest eligible negative effect on success:

```text
L1: 65%
L2: 100%
```

The chance uses the ordinary deterministic effect-chance stream. A temporary
non-dispellable marker converts success into the shared ordered Cleanse
operation and is removed immediately afterward.

## Healing reactions

Candlelight Radiance (`612343`) observes positive effective healing attributed
to a character ability:

```text
L1: healer and healed target gain 50% ATK for 1 turn
L2: all living present allies gain 50% ATK for 1 turn
```

The buff is an ordinary dispellable effect with a percent-of-base ATK
modifier. Refresh behavior and owner-turn duration are handled by the common
effect runtime.

Prajna Voyage (`612346`) restores an additional fraction of effective healing
when the healer belongs to the player's team:

```text
L1: 30%
L2: 45%
```

The shared `Heal` Rule IR operation's exact-resolved policy consumes an amount
that has already passed the healing formula. It still clamps against missing
HP and emits the normal `Heal` event, but does not apply outgoing or incoming
healing multipliers a second time. This same policy now backs S01 shared
healing. Ability-source filtering prevents path-generated restoration from
recursively activating the paired healing rules.

## HP-derived offense and defense

Bitter Is the Bane (`612344`) executes once after an Attack resolves:

```text
L1: 36% current HP
L2: 42% MaxHP
```

One stable-random unit is selected from the action's committed target list.
The additional damage inherits the triggering attack's element, cannot CRIT
and can defeat its target. Level 1 reads live current HP; level 2 reads the
effective maximum-HP stat.

Corporeal Pellucidity (`612345`) maintains a non-dispellable marker while the
owner is at full HP. It is installed at battle start or after healing reaches
full HP, and removed after damage or HP consumption lowers HP:

```text
L1: 36% damage mitigation
L2: 36% damage mitigation and 27% Effect RES
```

The mitigation is represented for all seven ordinary damage-purpose families.
The effect is active during the hit that begins at full HP and is removed only
after that hit commits its HP result.

## Blessing-count MaxHP

Dharma Rain (`612350`) level 1 raises each character's MaxHP by 5% for every
selected Abundance Blessing, capped at six Blessings. The validated
contribution set supplies the count before battle materialization; combat core
receives only one ordinary 30%-or-lower percent-of-base HP modifier. Level 2 is
assigned to S03.

## Production verification

The production catalog and battle materializer prove:

- all six assigned families materialize from Excel/Sora as generic Rule IR;
- all eleven exceptional candidates close as `IrSufficient`;
- enhanced Dewdrop cleanse lowers to guaranteed fixed chance plus one ordered
  Cleanse;
- enhanced healing ATK is a 50%, one-turn, team-wide effect;
- enhanced HP damage uses 42% MaxHP, one committed attack target and the event
  element;
- enhanced full-HP defense carries seven 36% mitigation modifiers and one 27%
  Effect RES modifier;
- enhanced ally healing uses 45% of the effective heal without reapplying
  healing modifiers;
- Dharma Rain level 1 counts and caps six selected Abundance Blessings;
- a production two-enemy battle emits exactly one additional-damage event on
  a member of the committed attack target list.

The completion receipt is
`evidence/standard-universe-mechanics-complete-v1/partitions/G07-P2-M05-S02.json`.
