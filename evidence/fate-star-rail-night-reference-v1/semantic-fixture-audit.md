# G19-P4-B2 — Semantic Fixture and Replacement Audit

The executor resolved every fixture to a normalized stable identity, matched
its exact path/locator/digest receipt and evaluated 118 source-backed equality
assertions. All 58 fixtures pass. All thirteen policy rows retain
`IdentityOnlyNoOperationLowering`, at least two rejected alternatives, an
existing fixture binding and a released-evidence replacement condition.

The first execution exposed that the policies named BattleEvent and
BattleTarget fixtures which had not been materialized because both families are
disabled `ResearchRequired` facts. The generator now adds one explicit disabled
policy-bound fixture per family. This changes no observed operation claim. The
correction was propagated through the pack index (58 fixtures, digest
`3a931ae7…4018`), workbooks (5,936 rows), Sora debug/binary export, standalone
loader and every-sheet visual evidence. Independent workbook and Sora release
regeneration remain byte-identical.

Focused command:

```text
fnm exec --using 24.15.0 node tools/fate-star-rail-night-reference/verify-semantic-fixtures.mjs .
```
