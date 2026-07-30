# Currency Wars investment systems

Batch `G12-P1-B8` replaces the superseded Persona-shaped authoring contract
with the complete direct GridFight Augment, Portal, Orb, Projection, Talent
and enhancement closure.

## Contract correction

The corrected Version 4.4 selector is `GuideType = GridFight`. No
`RoguePersona*` table participates in the direct Currency Wars closure.
Accordingly, this batch replaces thirteen unused `persona-*` normalized file
contracts with nineteen source-shaped GridFight files. The normalized schema
now contains 102 files, partitioned as 60 main-workbook files, 32 binding files
and ten review files.

The owning contract generator rebuilt:

- `normalized-schema.json` with SHA-256
  `6a88c3981a69450b40ffd29e411f08d992d989829fbfe407e02193b72c3805fb`;
- `authoring-contract.json` with SHA-256
  `3d67eb2c50a9286256af056dcda46b325b1c9c67111765916be60994dbdf5fcc`;
  and
- all previously imported policy-bound rows whose provenance binds the
  normalized-schema digest.

Published batch digests remain immutable historical receipts. This migration
does not edit their completion records or claim that old bytes remain current.

## Exact source closure

The batch accounts exactly once for every one of the 1,422 frozen obligations:

| Source family | Rows |
|---|---:|
| Augment definitions, season membership, remarks and MazeBuffs | 735 |
| Augment monster rules and module bans | 33 |
| Portal definitions, season membership, remarks and MazeBuffs | 180 |
| Portal module bans | 2 |
| Orb definitions and display locators | 380 |
| Projection definitions and MazeBuffs | 4 |
| Talent, season talent and Talent MazeBuff rows | 56 |
| Enhancement and selected-enhancement rows | 32 |
| Total | 1,422 |

Every row retains the exact direct table locator, evidence digest, canonical
parameters, configuration/buff references, prerequisite/successor edges,
season membership and module exclusion where present. The 334 Augment rows
and 84 Portal rows preserve their released bilingual names and descriptions
as evidence while using independent bilingual mechanical summaries.

## Semantic boundary

Released bilingual text proves that Currency Wars has run-long Investment
Environments and selectable Investment Strategies, including quality,
selection and refresh rules. The direct GridFight tables publish Augment and
Portal identities plus their configuration programs. This batch preserves
that source vocabulary and does not pretend the old Persona style/gift/room
model exists.

Configuration-program execution, offer ordering, reroll timing and same-boundary
activation remain reference contributions for `G12-P2-B6` and semantic fixture
review. No runtime lowering or handler is introduced.

## Result

The nineteen normalized files contain 1,422 rows. Their combined digest in
lexicographic file order is
`3f28ef7f6a7e8150aa30973da1bf5e62d32d2dd292d2e7cc699de363276c48b7`.

```text
fnm exec --using 24.15.0 node \
  tools/currency-wars-reference/import-investment-systems.mjs \
  --source-cache .cache/content-reference/turnbasedgamedata
fnm exec --using 24.15.0 node \
  tools/currency-wars-reference/verify-investment-systems.mjs \
  --source-cache .cache/content-reference/turnbasedgamedata
```
