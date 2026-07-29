# Curio and Grand Miracle Boundary

This dossier freezes the `G11-P1-B5` interpretation of released Version 4.4
Curio, Weighted Curio and Grand Miracle/Hex source rows. It is reference
evidence only and does not claim runtime executability.

## Curio identities and mode copies

- `RogueTournMiracle` has 235 rows explicitly selected by `TournMode =
  Tourn3`.
- Those rows reference 179 distinct non-null
  `RogueTournHandbookMiracle` identities. Twelve Tourn3 mode copies publish no
  `HandbookMiracleID`; they remain source states with
  `MissingHandbookIdentity` and are not assigned an invented identity.
- Every mode copy resolves its `RogueTournMiracleDisplay` and
  `RogueMiracleEffect` row. Effect identifiers and canonical parameters are
  retained without copying source description prose.
- The fixed rows do not publish a uniform charge, activation, destruction,
  repair, replacement, simultaneous-trigger or no-legal-target program.
  These fields remain `Unspecified` under a replaceable project policy.
- All 286 `RogueTournMiracleGroup` rows expose only
  `RogueMiracleGroupID`. Candidate membership, weights, mode selectors,
  consumers and draw behavior therefore remain empty and fail closed.

## Grand Miracle/Hex boundary

Seventeen `RogueTournHex` rows explicitly select `Tourn3`. Each row directly
publishes a display, a referenced MazeBuff, optional extra effects and either
character-Path or element eligibility. The current selector contains 18 Path
occurrences and seven element occurrences across the 17 definitions.

All 17 display references resolve. None of the referenced `633401`–`633417`
MazeBuff IDs has a definition in the fixed released `RogueMazeBuff` table or
another file in the focused source closure. The IDs are retained as unresolved
effect locators; activation, duration, interaction, simultaneous ordering and
teardown remain `Unspecified`.

## Historical eligibility rows

The frozen manifest conservatively included all 57
`RogueTournHexAvatarBaseType` rows. A direct ID join proves that:

- 23 rows reference `RogueTournMiracle` entries selected by `Tourn1`;
- 34 rows reference entries selected by `Tourn2`;
- zero rows reference a `Tourn3` Miracle or any of the 17 Tourn3 Hex IDs.

These rows are retained one-for-one to close the frozen receipts, but are
classified `OtherMode` / `Excluded`. They are not used as current Grand
Miracle eligibility. Prefixes, table names and historical reuse are not
membership evidence.

## Replacement conditions

Replace an unresolved field only when released evidence supplies:

1. an exact group or offer selector with candidates, weights and consumers;
2. a charge/destruction/repair/replacement transition with timing and scope;
3. a definition or ability binding for each referenced `6334xx` MazeBuff;
4. activation, duration, simultaneous-trigger order and teardown behavior.

Until then, candidate sets remain empty and illegal operations reject without
mutation. This is a reference policy, not an observed gameplay-parity claim.
