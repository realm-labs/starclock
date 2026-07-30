# Goal 16 Departure Arsenal and Synthesis Audit

`G16-P1-B2` normalizes every Version 2.2 Departure weapon, accessory, authored
level, battle-program binding and Legendary synthesis edge.

## Exact inventory

| Family | Definitions | Levels/bindings |
|---|---:|---:|
| Standard weapons | 13 | 104 levels |
| Legendary weapons | 13 | 13 levels |
| Accessories | 16 | 64 levels |
| Weapon program bindings | 26 | one per weapon definition |
| Accessory program bindings | 16 | one per accessory definition |
| Legendary recipes | 13 | 26 ordered inputs |

The 42 `EvolveBuildGearCollection` rows split by the exact
`DamageCustomName` field: nonempty `_Base`/`_Max` values define weapons and an
empty value defines accessories. No ID range or localized-name similarity
grants membership.

All 181 `EvolveBuildGearConfig` rows resolve by the exact `(MazeBuffID, Level)`
pair to 181 distinct `EvolveBuildMazeBuff` rows. Each normalized level retains
its index lists, modifier, binding key, complete canonical parameter vector,
description hashes, rarity, series and type. The remaining 67 mode-family
MazeBuff rows are still present in the immutable denominator and are owned by
later growth/team/score batches; they are not dropped or mislabeled as gear
levels.

## Program evidence

The normalized trigger/binding rows do not copy raw ability programs. For each
exact binding key they retain:

- matching ability names;
- modifier identifiers;
- trigger event identifiers;
- ordered-operation type identifiers; and
- a canonical SHA-256 of the selected program fragment.

The whole-file source digest remains attached through the P0-B3 manifest.
These rows are ReferenceOnly and explicitly `runtime_executable=false`.

## Legendary synthesis graph

Every `EvolveBuildForgeMaterial` row has exactly two ordered prerequisites:

1. its corresponding Standard weapon at level 8, present in `CostGearList` and
   therefore consumed;
2. one exact accessory at level 1, absent from `CostGearList` and therefore
   retained.

The output is the paired Legendary weapon. All 13 outputs are disjoint from all
inputs, every output exists, and the graph is acyclic. Validation is
all-prerequisites-before-consumption. Invalid/simultaneous behavior remains
the explicitly labeled ProjectPolicy from the P0-B4 approximation register;
it is not presented as an exact source fact.

## Deterministic generation

The owning generator produces eight normalized files:

```text
node tools/galactic-baseballer-reference/normalize-departure-arsenal.mjs \
  --source-cache .cache/galactic-baseballer-source
node tools/galactic-baseballer-reference/verify-departure-arsenal.mjs \
  --source-cache .cache/galactic-baseballer-source
```

The verifier proves definition/level cardinality, consecutive authored levels,
181 exact MazeBuff mappings, nonempty program summaries, recipe arity,
prerequisite levels, consumption flags and graph acyclicity.
