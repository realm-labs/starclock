# Goal 07 Elation Partition S03

`G07-P2-M08-S03` completes the enhanced level of Exemplary Conduct and both
released levels of Mostly Harmful, Suspiria, Pale Fire, Back to the Lighthouse
and Doctor of Love.

## Authoritative authoring boundary

`Universe.xlsx`, `UniverseBindings.xlsx` and `UniverseEvidence.xlsx` remain
the editable source. They are authored with openpyxl, exported through Sora
0.3.0 and checked against the focused partition golden. Runtime lowering
consumes validated domain records and released binding keys, never workbook
rows.

Exact parameters and callback structure were verified against
`RogueMazeBuff.json` and `Level_RogueBuff_Ability_4.json` from the pinned
TurnBasedGameData revision. Released English and Chinese descriptions were
cross-checked in the matching `TextMapEN.json` and `TextMapCHS.json` records,
accessed 2026-07-26.

Champion's Dinner adds the Ultimate tag as an eligible route for all five
mechanics without rewriting action kind, timing, payment or cause identity.
The ordinary follow-up and counter routes remain independently addressable.

## Exemplary Conduct, enhanced

The enhanced configuration extends both constants:

```text
bonus = min(selected Elation Blessings, 9) × 12%
maximum bonus = 108%
```

The immutable battle contribution snapshot supplies the selected-Blessing
count during materialization. Three tag-filtered DamageBoost modifiers cover
follow-up, counter and Champion-enabled Ultimate damage.

## Mostly Harmful

```text
L1: +35% Weakness Break Efficiency for follow-up damage
L2: +50% Weakness Break Efficiency for follow-up damage
```

This is a source-side `ToughnessDamage` modifier in the Break purpose. It
multiplies authored Toughness reduction without changing HP damage or Break
Effect.

## Suspiria

```text
L1: +26% follow-up DMG
L2: +39% follow-up DMG
```

The value enters the ordinary DamageBoost stage once. Counter and
Champion-enabled Ultimate tags share the same stacking group, preventing an
action carrying more than one eligible tag from double-counting the source.

## Pale Fire

```text
L1: +26% CRIT Rate for follow-up damage
L2: +39% CRIT Rate for follow-up damage
```

The modifier is a source-side `CritRate` addition filtered by committed
ability tags. The generic critical sampler performs the clamped probability
query per hit; this Blessing introduces no separate RNG stream.

## Back to the Lighthouse

```text
L1: +24% Energy Regeneration Rate while resolving follow-up damage
L2: +36% Energy Regeneration Rate while resolving follow-up damage
```

The shared Energy boundary now models the released callback directly:

```text
resolved_gain = base_action_energy_gain × effective_energy_regeneration_rate
```

`EnergyRegenerationRate` has an authoritative base of `1.0`. Intrinsic action
Energy is scaled after all hits and before `ActionResolved`, using the
committed action tags and nearest-ties-even fixed-point rounding. Rule-IR
resource gains use the same query only when
`scales_with_regeneration = true`; validation rejects the flag for non-Energy
or non-gain mutations.

## Doctor of Love

```text
L1: after one follow-up action, heal its owner for 10% Max HP
L2: after one follow-up action, heal its owner for 15% Max HP
```

One `ActionResolved` trigger per eligible tag routes to the same owner-scoped
healing program. `OnceScope::Action` coalesces multi-hit attacks, and the
normal healing pipeline applies outgoing/incoming modifiers, caps at Max HP
and emits the authoritative Heal event.

## Production verification

The focused tests prove:

- all six S03 assignments materialize as generic Rule IR without native
  handlers;
- enhanced Exemplary Conduct reaches exactly 108% at the nine-Blessing cap;
- Mostly Harmful, Suspiria, Pale Fire and Back to the Lighthouse preserve
  exact values, formula stages and all three eligible tags;
- Doctor of Love is an owner-Max-HP heal once per complete action;
- a production Kafka Ultimate enabled by Champion's Dinner gains exactly
  `5 × 1.36 = 6.8` Energy and heals exactly 15% Max HP.

The completion receipt is
`evidence/standard-universe-mechanics-complete-v1/partitions/G07-P2-M08-S03.json`.
