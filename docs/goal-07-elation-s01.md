# Goal 07 Elation Partition S01

`G07-P2-M08-S01` establishes the executable Aftertaste foundation for the
Elation path. It completes both levels of Auto-Harmonica, Slaughterhouse,
Champion's Dinner, Portrait and Just Keep on Crying, and retains the base
Hourglass Kindergarten record assigned to this partition. The Hourglass
levels and their executable ATK reduction are owned by the next frozen
partition.

## Authoritative authoring boundary

`Universe.xlsx`, `UniverseBindings.xlsx` and `UniverseEvidence.xlsx` remain
the editable source. They are regenerated and inspected with openpyxl; the
focused partition gate compares every assigned row with the committed Sora
0.3.0 binary and debug exports and rejects formulas or error cells.

Released structured values are the numeric source of record. Public mechanic
wording was cross-checked against the
[Simulated Universe Paths reference](https://honkai-star-rail.fandom.com/wiki/Simulated_Universe/Paths)
and the
[Star Rail Station Blessing catalog](https://starrailstation.com/en/simuniverse/current/blessings),
accessed 2026-07-26.

## Aftertaste damage primitive

Aftertaste is represented as ordinary typed damage with
`DamageClass::Elation`. It is not a character callback. The reusable
`RandomRepeatedDamage` operation:

- receives one target at a time from a labeled, without-replacement randomized
  traversal of the originating action's committed target list;
- draws an inclusive hit count with a dedicated stable RNG purpose;
- draws each hit's element independently from a canonical seven-element list;
- can exclude the triggering event's element;
- emits each hit as an ordinary sequential damage operation, retaining the
  rule source and cause parent;
- uses checked fixed-point arithmetic and the normal modifier pipeline.

The operation makes replay draw counts and element selection explicit. Fixed
hit-count variants do not consume a count draw.

## Auto-Harmonica: Whitest Night

After each follow-up attack or counter, Auto-Harmonica traverses every
opposing unit hit by the action in deterministic randomized order and deals
an independently rolled number of Aftertaste hits to each:

```text
L1: random hit count = 1..3; each hit = 55% of the owner's ATK
L2: random hit count = 1..3; each hit = 60% of the owner's ATK
    all Aftertaste DMG dealt by the team +35%
```

When Champion's Dinner is selected, Ultimate actions enter the same trigger
set. Every hit independently selects Physical, Fire, Ice, Lightning, Wind,
Quantum or Imaginary.

## Slaughterhouse No. 4: Rest in Peace

After each follow-up attack or counter, Slaughterhouse traverses every enemy
hit by that action and deals Aftertaste equal to 80% of the owner's ATK to
each. It emits one normal hit per target, plus additional hits for each target
that is individually Weakness Broken:

```text
L1: 1 hit normally; 2 hits while broken
L2: 1 hit normally; 3 hits while broken
```

Champion's Dinner also makes Ultimate actions eligible. Each emitted hit uses
an independently selected element.

## Champion's Dinner: Cat's Cradle

Champion's Dinner treats Ultimate damage as follow-up damage for Elation
Blessing triggers and increases damage tagged as follow-up, counter or
Ultimate:

```text
L1: +15% DMG
L2: +55% DMG
```

The implementation deliberately keeps the ability's native action kind and
tags intact. It adds the Ultimate trigger route to Aftertaste rules and
installs ordinary tag-filtered Damage Boost modifiers. This avoids rewriting
an Ultimate into a follow-up action, which would corrupt interrupt, resource
and replay semantics.

## Portrait of A Man On Fire

Every non-Portrait Aftertaste instance creates exactly one extra Aftertaste
instance on the same target:

```text
L1: extra damage = triggering applied Aftertaste damage × 60%
L2: extra damage = triggering applied Aftertaste damage × 90%
```

The extra hit scales the triggering event's pre-mitigation raw amount, then
passes through the ordinary Elation damage formula once. It chooses uniformly
from the six elements different from the triggering instance and retains that
instance as its cause parent. The rule excludes its own source so the generated
hit cannot recursively trigger another Portrait hit.

## Just Keep on Crying!

An enemy receives one refreshable effect for each distinct Aftertaste element
that damages it. Each active elemental effect independently adds:

```text
L1: +8% damage taken
L2: +12% damage taken
duration: through the affected target's next action end
```

Seven definitions preserve the distinct-element identity. Repeated damage of
one element refreshes that element instead of adding another stack; different
elements sum through the ordinary Vulnerability stage. The modifier covers
ordinary, DoT, Break, Super Break, Additional, Joint and Elation damage
purposes.

## Production verification

Production tests prove:

- every selected level materializes without a native handler;
- follow-up attacks, counters and Champion-enabled Ultimates expose the
  expected triggers and exact damage modifiers;
- a production Kafka Ultimate proves every committed target is visited,
  receives an independent deterministic 1–3 hit Auto-Harmonica roll, and
  receives the normal unbroken Slaughterhouse hit;
- Portrait emits exactly one nonrecursive, different-element hit per original
  Aftertaste instance while preserving the cause chain;
- Just Keep on Crying installs seven independent one-target-turn effects and
  exact additive vulnerability modifiers.

The completion receipt is
`evidence/standard-universe-mechanics-complete-v1/partitions/G07-P2-M08-S01.json`.
