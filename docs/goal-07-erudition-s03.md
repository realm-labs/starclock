# Goal 07 Erudition Partition S03

`G07-P2-M10-S03` completes the enhanced level of Throne of Engaged Gears
and both released levels of Ring of Bent Wires, Scepter of Energy Torque,
Torch of Anti-Lag Ignition, Candle of Delayed Diffraction and Canopy of
Mottled Metal. Every assigned mechanic lowers to generic modifiers, effects,
selectors, programs and triggers. This partition admits no native handler.

## Authoritative authoring boundary

`Universe.xlsx`, `UniverseBindings.xlsx` and `UniverseEvidence.xlsx` remain
the editable source. The focused openpyxl verifier reads those formal tables,
checks the assigned rows and provenance, compares them with Sora 0.3.0 debug
tables, and writes the partition golden from the production `.sora` bundle.
Runtime code consumes validated domain definitions and released binding keys;
it does not depend on workbook row types.

Parameters, descriptions and lifecycle details were verified against the
pinned `ExcelOutput/RogueMazeBuff.json`,
`ConfigAbility/Level/Level_RogueBuff_Ability_Erudition.json`, and released
English and Chinese text maps. The source snapshot was accessed on
2026-07-26. All values are exact; this partition records no numeric
approximation.

## Throne of Engaged Gears

S02 already supplied the shared selected-Erudition-Blessing counter and
Ultimate-only modifier. This partition verifies and claims the enhanced
level:

```text
L2 Ultimate DMG = +10% × min(selected Erudition Blessings, 9)
```

The count is compiled from validated contributions before battle
materialization. It is deterministic, bounded and independent of controller
or display state.

## Ring of Bent Wires and Scepter of Energy Torque

The two Blessings add ordinary source-side critical stats only while the
damage source carries the `ultimate` ability tag:

```text
                                       L1       L2
Ultimate CRIT Rate                     +18%     +27%
Ultimate CRIT DMG                      +30%     +45%
```

The modifiers use the standard `Flat` stat stage and dynamic source context.
They do not rewrite the ability's global CRIT stats, and therefore cannot
leak into Basic, Skill, follow-up, DoT or other damage sources that do not
carry the Ultimate tag.

## Torch of Anti-Lag Ignition

After an owner's Ultimate fully resolves, Rule IR applies a non-dispellable
one-charge marker:

```text
next Attack DMG                        L1 +50%
                                       L2 +75%
```

The marker owns an Attack-source DamageBoost modifier. The next resolved
Attack action consumes the marker only after every hit has used the bonus.
An attacking Ultimate first consumes an older charge and then arms a fresh
one. Signed trigger priority makes that order explicit, so the newly armed
charge is never consumed by the same Ultimate and no character-ID branch is
required.

Non-attacking actions do not consume the marker. Rule-generated damage
outside the next Attack's source context also cannot inherit the modifier.

## Candle of Delayed Diffraction and Canopy of Mottled Metal

After an owner resolves an Attack ability whose immutable catalog selector
has the `All` target pattern, the owner receives a replace-stacking timed
effect:

```text
                                       L1       L2
ATK                                    +30%     +40%
DEF                                    +30%     +40%
released duration                      2 turns  3 turns
```

ATK and DEF use the standard `PercentOfBase` stat stage. Reapplication
replaces and refreshes the same source effect rather than stacking multiple
copies.

`ActionResolved` runs immediately before the owning normal turn's end tick.
The runtime therefore retains one additional internal duration tick at
application. The same command consumes that administrative tick, leaving
exactly the released two or three future owner turns in authoritative state.
This is lifecycle compensation, not a numeric approximation.

The generic `TargetPattern::All` fact is derived from the ability catalog,
so any current or future AoE Attack can activate these Blessings without a
path-specific ability list.

## Production verification

Focused tests prove:

- all sixteen assigned records and rules materialize through generic Rule IR;
- the enhanced Engaged Gears contribution is accepted by the shared capped
  Blessing-count implementation;
- Ring and Scepter author exact Ultimate-only CRIT Rate and CRIT DMG
  modifiers;
- an Ultimate arms the exact 75% next-Attack marker and the following Basic
  Attack consumes it after resolving; and
- a production AoE Skill applies exact 40% ATK and DEF effects with three
  future owner turns remaining after the activating command.

The completion receipt is
`evidence/standard-universe-mechanics-complete-v1/partitions/G07-P2-M10-S03.json`.
