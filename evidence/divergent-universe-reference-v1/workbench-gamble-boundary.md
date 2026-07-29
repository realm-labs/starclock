# Workbench, Gamble and Curse Chest Boundary

This dossier freezes the `G11-P1-B8` service interpretation. It records exact
operations and explicit unknowns without adding runtime handlers.

## Workbenches

Eleven workbench rows expose ordered references to six function definitions:
Blessing enhancement, Blessing overwrite, Equation overwrite, Curio synthesis,
Weighted Curio overwrite and Weighted Curio equipment/adjustment.

Released bilingual descriptions prove the input/output kind and these
constraints:

- Blessing enhancement consumes Workbench Heat, which resets at each
  Workbench.
- Blessing and Equation overwrite prices increase with accepted overwrite
  count; the Equation result keeps identical quality.
- Curio synthesis consumes equal-rarity Curios and yields a random Curio of
  equal or higher rarity.
- Weighted Curio operations overwrite or adjust the owned loadout.

Numeric prices, candidate identities, weights, target order, rerolls,
availability and empty-pool behavior are not published by these tables. They
remain empty/`Unspecified`, with reject-without-mutation fallback.

## Gamble groups and units

The 126 groups comprise 51 Slot Machine and 75 Fortune Wheel rows. Group rows
contain no unit IDs, weights, draw count or consumer selector. The 89 unit rows
contain only a type and one parameter:

- 75 Blessing source-group parameters;
- 12 Curio source-group parameters;
- two exact run-currency outcomes of 20 and 40.

No group-unit relationship is inferred from ID adjacency or row order.
Blessing/Curio source parameters await stable-ID pool closure in Phase 2.

## Curse Chests

Twenty-one Treasure and eight Fountain rows resolve all four referenced
display locators. Parameters and display templates prove ordered operation
shapes for negative Curios, Cosmic Fragment ranges, random Blessings/Curios,
Equation acquisition, random overwrite, all-content replacement, Path bias
and Benediction Shards.

Random candidate identities and weights remain unavailable. Each choice set
keeps an explicit final `LeaveWithoutMutation` operation, and no source prose
or presentation asset is imported.

## Replacement conditions

Candidate IDs, weights, prices and refresh behavior may be populated only from
a released service selector, operation program or exact stable-ID pool closure.
Until then, empty pools reject or leave without changing authoritative state.
