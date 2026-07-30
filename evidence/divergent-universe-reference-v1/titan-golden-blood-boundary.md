# Titan and Golden Blood Boundary

This dossier freezes the `G11-P1-B6` interpretation of the released Titan,
Golden Blood's Boon and permanent Titan-talent tables. It does not lower any
row into runtime behavior.

## Exact catalog

- `RogueTournTitanType` defines 12 Titan types, split exactly into six `Day`
  and six `Night` types.
- Each type has seven `RogueTournTitanBless` rows: one level-1, three level-2
  and three level-3 Golden Blood's Boons.
- All 84 Boons resolve a level-1 `RogueMazeBuff`. Every resolved buff binds
  `StageAbilityBeforeCharacterBorn`, so its battle contribution is an exact
  pre-combat locator with canonical parameters.
- Twelve Boon rows publish `BlessRatio`; nine publish both Day and Night in
  `BlessBattleDisplayCategoryList`. These values are preserved without
  interpreting `BlessRatio` as an offer weight.
- Each Titan type has three `RogueTournTitanTalent` levels. All 36 levels use
  item `281020` with costs 50, 75 and 100 at levels 1, 2 and 3.
- Exact prerequisite IDs are retained even though they form a cross-type
  progression graph rather than twelve isolated chains.

## Talent contributions

The structured talent rows bind bilingual released descriptions and canonical
parameters. They normalize to 26 battle-scoped and 10 activity-scoped
contributions. Battle contributions include Day/Night conditioned combat
statistics and an enter-Day heal. Activity contributions cover Boon offer
count, starting currency, Occurrence options, domain probabilities and
Weighted Curio overwrite count.

Four probability contributions publish no numeric magnitude. Their direction,
target and condition are exact, while the value remains `Unspecified`.

Each talent's `ActJson` is retained only as a source locator. Inspection shows
dialogue, option and presentation tasks ending in `FinishLevelGraph`, not the
authoritative talent-effect program. Story dialogue and presentation are
excluded from the normalized pack.

## Choice policy

Grouping by exact Titan type and level yields deterministic candidate
denominators of one, three and three. The tables do not publish offer timing,
rerolls, simultaneous selection ordering or no-legal-candidate behavior.
Choice rows therefore use a replaceable policy with one selection, stable-ID
ordering and reject-without-mutation fallback.

Replace this policy only when released selector/service programs prove the
exact offer lifecycle. In particular, do not infer weights from
`BlessRatio`, row order, ID shape or the number of alternatives.
