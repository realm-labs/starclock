# Goal 16 Demon King Arsenal and Synthesis Audit

`G16-P2-B2` closes the Version 4.4 Demon King weapon, accessory and advanced
synthesis data without importing executable gameplay or inferring relationships
from names and ID ranges.

## Arsenal closure

| Family | Definitions | Authored levels |
|---|---:|---:|
| Standard weapons | 15 | 120 |
| Legendary weapons | 12 | 12 |
| Twin weapons | 1 | 1 |
| Supreme weapons | 1 | 1 |
| **All weapons** | **29** | **134** |
| Accessories | 16 | 64 |

Every one of the 198 `EvoBdSCGearConfig` rows resolves to a distinct
`(MazeBuffID, Lv)` row in `EvoBdSCMazeBuff`. Each level retains the complete
canonical parameter vector, indices, binding key, descriptions, rarity, series
and type. The remaining 117 MazeBuff rows are still present in the frozen
denominator and belong to P2-B3 strategy/progression closure.

All 45 collection rows and all five gear-type rows are represented. The exact
source `Type` field establishes `Plugin`, `Forge`, `DuelForge` and
`UltraForge`; the base weapon type is the explicit first type row. No tier is
derived from display-name similarity or numeric ranges.

## Trigger and actor bindings

The Demon King weapon program contributes 29 structural binding summaries and
the accessory program contributes 16. Each summary retains:

- the exact binding key;
- sorted ability and modifier names;
- trigger-event and operation-type sets;
- a canonical fragment digest;
- the whole source-program receipt.

`Ranger's Badge` (`3113003`) additionally binds all three released summoned
actor configurations for the claymore, shooter and shooter partner. Their
ability identifiers, skill types and program digests are retained. No other
weapon is assigned those actor programs, and none of the normalized summaries
is runtime executable.

## Exact synthesis graph

The 14 `EvoBdSCForgeMaterial` rows define 14 acyclic recipes and 28 ordered
inputs:

| Tier | Recipes | Exact prerequisite shape |
|---|---:|---|
| Legendary | 12 | one level-8 Standard weapon consumed; one level-1 accessory retained |
| Twin | 1 | `3113005` and `3113006`, both level 8 and consumed in source order |
| Supreme | 1 | Legendary `3113901` level 1 then Standard `3113014` level 8, both consumed in source order |

The Twin output is `3113201`, `Crest of Sol and Lune`. The Supreme output is
`3113301`, `Supreme All-Color Home Run`. The Supreme input edge from
`3113901` is explicit in the source and is validated as an ordinary DAG edge,
not rejected merely because an advanced output is reused as a later input.

Prerequisites are validated before mutation. `CostGearList` order is preserved
separately from stable validation order. Candidate precedence and
failure-without-mutation are explicit ProjectPolicy boundaries:

1. Supreme, Twin and Legendary tier precedence;
2. stable recipe ID within a tier;
3. synthesis before ordinary duplicate upgrade;
4. reject before any consumption when a prerequisite is absent.

These policies are not presented as observed parity and retain their existing
released-evidence replacement condition.

## Semantic and correction fixtures

Two new ReferenceOnly mechanism rules close the previously open families:

- `twin-weapon-synthesis`;
- `supreme-weapon-synthesis`.

Each has one successful and one rejected concrete fixture. The rejected cases
require byte-identical inventory and zero consumed inputs.

The official Version 3.4 RuinBot correction is joined to exact Version 4.4
levels 7 and 8:

| Level | Retained canonical parameter vector |
|---:|---|
| 7 | `0,9,70,14,0.75,30,0.3,0,0,0,0,0,0,0,0,0,0,0,0,0` |
| 8 | `0,12,70,14,0.75,30,0.3,0,0,0,0,0,0,0,0,0,0,0,0,0` |

Two correction fixtures assert those post-fix rows and explicitly assert that
pre-fix values are not modeled.

## Reproduction

```text
node tools/galactic-baseballer-reference/normalize-demon-arsenal.mjs \
  --source-cache .cache/galactic-baseballer-source
node tools/galactic-baseballer-reference/normalize-demon-arsenal.mjs \
  --check --source-cache .cache/galactic-baseballer-source
node tools/galactic-baseballer-reference/normalize-demon-arsenal-fixtures.mjs
node tools/galactic-baseballer-reference/normalize-demon-arsenal-fixtures.mjs \
  --check
node tools/galactic-baseballer-reference/verify-demon-arsenal.mjs \
  --source-cache .cache/galactic-baseballer-source
```

P3-B1 will merge these generated fragments into the contracted combined
arsenal, rule and fixture tables.
