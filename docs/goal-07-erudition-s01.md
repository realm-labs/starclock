# Goal 07 Erudition Partition S01

`G07-P2-M10-S01` completes BCI-34 Gray Matter, SMR-2 Amygdala,
VEP-18 Occipital Lobe, Attachment: Vestibular System, Imitation:
Transmitter Synthesis and Implant: Explicit Memory. The shared Brain in a
Vat lifecycle and every assigned blessing lower to generic Rule IR, effects,
modifiers, resources and ordinary interrupt actions. This partition admits
no native handler.

## Authoritative authoring boundary

`Universe.xlsx`, `UniverseBindings.xlsx` and `UniverseEvidence.xlsx` remain
the editable source. They are authored with openpyxl, exported through Sora
0.3.0 and checked against the focused partition golden. Runtime lowering
consumes validated domain definitions and released binding keys, never
workbook rows.

Parameters and lifecycle behavior were verified against the pinned
`ExcelOutput/RogueMazeBuff.json`,
`ConfigAbility/Level/Level_RogueBuff_Ability_Erudition.json` and the
corresponding global `MLevel_Rogue_Knowledge` configurations. Released
English and Chinese descriptions were cross-checked in `TextMapEN.json` and
`TextMapCHS.json`, accessed 2026-07-26.

Transmitter Synthesis L1 has one registered decimal normalization. The
upstream float-shaped parameter is `0.007999999`, while both released
localized descriptions state exactly 0.8%. Authoring and execution therefore
use the six-decimal value `0.008000`; this changes no released mechanic.

## Shared Brain in a Vat lifecycle

Brain charge is a non-dispellable, per-character effect with 1,000 integer
stacks:

```text
1 stack = 0.1% Brain charge
1,000 stacks = 100% Brain charge
```

Charge changes use the normal capped effect-stack operation. When a
character with full charge resolves a normal Ultimate, the engine removes
the charge, marks the next Ultimate as Brain-powered and restores that
character's Energy to its live maximum. The restored Energy exposes the
ordinary legal interrupt command; no hidden action queue or path-specific
command is introduced.

At the next Ultimate's action start, battle-scoped Rule IR state identifies
the action as Brain-powered. At action resolution the marker is removed.
That action cannot charge itself again through the full-charge trigger, so
one full Brain charge always authorizes exactly one additional Ultimate.

## Gray Matter and Amygdala

Gray Matter grants and accumulates exact charge:

```text
                         L1       L2
battle-entry charge      65%      100%
Weakness Break charge    35%      40%
hit broken enemy         —        5%
```

The enhanced broken-enemy gain is once per enemy within one attack, not once
per hit. Weakness Break attribution uses the breaking actor.

Amygdala grants 50% charge when the owner defeats an enemy. At L2, every
transition to full charge grants 20% SPD for two owner turns. The effect
application and stack-change event routes cover both a new charge effect and
an existing partial charge reaching its cap.

## Occipital Lobe

Occipital Lobe contributes source-side Ultimate RES PEN through the ordinary
Resistance formula stage:

```text
L1: 20% Ultimate RES PEN
    next Ultimate gains 3% RES PEN per target hit by the previous Ultimate

L2: 25% Ultimate RES PEN
    gain 3% RES PEN for the greatest number of targets hit by one Ultimate
    during the current battle
```

The committed action-target snapshot supplies the count. A permanent
stack-backed effect stores one plus the effective target count, allowing the
modifier expression to remain non-negative without content-specific
resolver state. Source and Ultimate filters prevent the penetration from
affecting other actors or damage families.

## Vestibular System

When the owner starts a Brain-powered Ultimate, Vestibular System applies an
ordinary CRIT DMG effect:

```text
L1: +80% CRIT DMG until that Ultimate resolves
L2: +90% CRIT DMG through the end of the next attack after that Ultimate
```

The enhanced lifetime uses a bounded battle slot and the generic Attack tag.
Non-attacking actions do not consume the retained bonus.

## Transmitter Synthesis

Energy overflow is now an explicit scalar event property. Transmitter
Synthesis converts only actual discarded Energy:

```text
L1 Brain stacks = overflow Energy × 0.008 × 1,000
L2 Brain stacks = overflow Energy × 0.012 × 1,000
```

For example, 20 overflow Energy produces 160 stacks at L1 and 240 at L2.
The conversion uses checked fixed-point multiplication and
nearest-ties-even integer finalization. Effective Energy gain that remains
below the maximum is not overflow and does not charge this blessing.

## Explicit Memory

After a Brain-powered Ultimate resolves, Explicit Memory creates a dedicated
shield on that character:

```text
L1: 36% of live Max HP for two owner turns
L2: 45% of live Max HP for three owner turns
```

The shield uses the generic shield store and carries its source-effect
identity. It participates in the existing absorption, replacement, duration
and replay-hash contracts without adding Brain-specific shield logic.

## Production verification

Focused tests prove:

- all sixteen assigned level rules materialize as generic Rule IR;
- enhanced Gray Matter starts every player at exactly 1,000 Brain stacks;
- a full charge exposes one additional Ultimate, restores exact maximum
  Energy and cannot recursively authorize another Ultimate;
- the enhanced Brain-powered Ultimate retains +90% CRIT DMG and creates an
  exact 45,000 shield for a 100,000-Max-HP fixture;
- 20 actual overflow Energy becomes exactly 160 L1 Brain stacks;
- Occipital Lobe authors exact 25% base and 3%-per-target Resistance-stage
  modifiers; and
- a combat-core formula fixture proves source-side 25% penetration produces
  exactly 125% of zero-resistance ordinary damage.

The completion receipt is
`evidence/standard-universe-mechanics-complete-v1/partitions/G07-P2-M10-S01.json`.
