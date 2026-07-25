# Goal 07 Preservation Partition S03

`G07-P2-M02-S03` completes 16 content records, 16 mechanic-rule records,
one production semantic fixture and eleven native-handler reviews. It covers
Assemble level 2 and both released levels of Sentinel, Patch, Compensation,
Firmness and Rotation. No native handler is admitted.

## Authoritative authoring boundary

The formal source remains the openpyxl-authored workbook set:

- `Universe.xlsx` contains the Blessing definitions, levels and exact
  parameters;
- `UniverseBindings.xlsx` binds definition and level rows to their released
  mechanic sources;
- `UniverseEvidence.xlsx` retains the break-family fixture, audit rows and
  provenance.

`tools/goal07/author-path-partition.py` rejects formulas and spreadsheet error
cells, proves exact partition ownership, and compares the assigned workbook
rows with the committed Sora 0.3.0 debug and production exports. Runtime
materialization consumes the validated `.sora` bundle, never staging JSON.

## Exact mechanics

All ratios use six-decimal fixed point. `MaxHP` is the current derived maximum
HP of the character receiving a shield. Shield lifetimes use that character's
normal turn-end clock.

### Construct: Assemble (`612050`)

The enhanced level completes the S02 modifier:

```text
DEF bonus(L2) = 0.08 * min(owned Preservation Blessings, 9)
```

The selected-Blessing count is frozen by the validated contribution compiler
before battle construction.

### Construct: Sentinel (`612051`)

At battle start every player character receives an independent shield:

```text
L1: shield = 0.16 * MaxHP, duration = 2 owner turns
L2: shield = 0.24 * MaxHP, duration = 2 owner turns
```

The production vector uses four 100,000-HP characters and produces exactly
four 16,000 or four 24,000 shield instances.

### Construct: Patch (`612052`)

The rule accumulates effective HP loss across the complete observed action,
then applies one shield after `ActionResolved`:

```text
shield = 0.18 * total HP lost by the character in this action
L1 duration = 1 owner turn
L2 duration = 2 owner turns
```

Shield absorption is not counted as HP loss. Multi-hit actions therefore use
their complete effective loss instead of only the first hit. The accumulator
is reset atomically after the shield proposal.

### Construct: Compensation (`612053`)

When an enemy enters the globally broken state, every living player character
receives a shield calculated from their own current MaxHP:

```text
L1: shield = 0.14 * MaxHP, duration = 2 owner turns
L2: shield = 0.18 * MaxHP, duration = 3 owner turns
```

The production break fixture force-breaks a real Toughness layer through the
ordinary resolver path and observes four exact 18,000 shields at L2.

### Construct: Firmness (`612054`)

While a character's current effective shield is positive:

```text
L1: reducible damage taken -16%
L2: reducible damage taken -24%
```

The dynamic mitigation applies to ordinary, DoT, additional, joint, Elation,
Break and Super Break damage. It intentionally does not alter true damage.
The exact L1 fixture reduces a 1,000 calculated hit to 840.

### Construct: Rotation (`612055`)

Whenever a character gains a positive shield:

```text
L1: 20% fixed chance to remove at most one negative effect
L2: 30% fixed chance to remove at most one negative effect
```

The removal pool contains dispellable debuffs and cleanseable control effects
under one shared maximum. Stable effect-instance order resolves ambiguity and
keeps replay behavior deterministic.

## Generic core additions

The partition adds only one new mutation primitive:

```text
Cleanse { selector, maximum }
```

It lowers to the existing effect-removal operation with a merged negative
effect category and a single global bound. Catalog validation rejects a zero
maximum. The resolver remains unaware of Blessing IDs.

The rest of the partition composes existing generic features:

- action-scoped scalar accumulation from `HpChangeAmount`;
- `ActionResolved`, `WeaknessBroken`, `ShieldChanged` and owner-turn events;
- current derived MaxHP and current shield queries;
- source-scoped shield replacement and bounded duration counters;
- dynamic mitigation modifiers split by formula purpose;
- fixed-chance effect application using the labeled effect-chance RNG stream.

## Production verification

The production integration tests prove:

- exact Sentinel L1/L2 four-character shield vectors;
- exact Patch shields from effective action-wide HP loss;
- exact Firmness L1 mitigation;
- a real forced Break event followed by four Compensation L2 shields;
- an executable Rotation chance success for a frozen seed;
- generic Rule IR cleansing removes a real dispellable negative effect;
- debuffs and cleanseable controls share one deterministic bounded pool.

The completion receipt is
`evidence/standard-universe-mechanics-complete-v1/partitions/G07-P2-M02-S03.json`.
