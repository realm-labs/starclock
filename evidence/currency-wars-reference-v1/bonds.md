# Currency Wars Bonds

Batch `G12-P1-B5` creates a complete direct GridFight Bond authoring surface.
It records reference contributions and recomputation boundaries, not runtime
handlers.

## Identities and membership

The direct identity set contains 33 `GridFightTraitBasicInfo` Bonds and 16
`GridFightSubTraitBasicInfo` sub-traits. Membership is proven only by explicit
source edges:

- each of the 77 role rows contributes its exact `TraitList` member edges;
- `GridFightRoleChoose` and `GridFightCoreRoleChoose` contribute explicit
  sub-trait role selectors.

Numeric ID shape is not used to infer a parent, member or level. Sub-traits
retain their direct `FatherTraitID`.

## Levels and contributions

All 152 `GridFightTraitLayer` rows become typed Bond levels. They cover the
same 49 main/sub-trait identities and preserve authored layer, MazeBuff ID,
property binding scope, member/all-member properties, battle-event overrides
and parameter lists.

The 653 contribution rows account for:

- 152 TraitLayer contribution bindings;
- 32 TraitBonus rows and all 27 threshold rows in their three exact groups;
- all 24 TraitEffect identities and 74 effect-layer parameter rows;
- 158 TraitMazeBuff and 154 TraitMazeBuffPlus rows;
- four special battle-area rules, two module/sub-trait selectors and three
  equipment relations;
- 25 game-reference score rows and 25 season-display rows.

Each contribution names its activation, scope and ordered effects. Empty
MazeBuff binding keys remain `NoExplicitBindingKey`; no implicit ability key
is invented.

## Recompute boundary

The authoring contract recomputes main Bond membership after an ordered roster
mutation and before battle contribution projection. Sub-traits recompute after
their explicit selection changes. Same-boundary ordering across simultaneous
roster mutations remains a semantic fixture responsibility; this batch does
not claim observed runtime precedence.

## Result

The three normalized files contain 854 rows: 49 Bonds, 152 levels and 653
contributions. Their combined digest in lexicographic file order is
`739f6f992dec7c08c163dfa32f836d1729bb149ed8aae95637fdefc67f16166f`.

```text
fnm exec --using 24.15.0 node tools/currency-wars-reference/import-bonds.mjs \
  --source-cache .cache/content-reference/turnbasedgamedata
fnm exec --using 24.15.0 node tools/currency-wars-reference/verify-bonds.mjs \
  --source-cache .cache/content-reference/turnbasedgamedata
```
