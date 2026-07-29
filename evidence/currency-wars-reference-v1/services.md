# Currency Wars currencies, items and services

Batch `G12-P2-B4` imports the complete run-relevant item/service closure and
keeps presentation-only resource rows out of semantic identity.

## Managed functions and consumables

All nine `GridFightFuncManage` rows preserve their unlock boundary and locked
visibility. All seven consumables preserve rule, typed parameters,
stackability and consumption behavior, including Roll, Upgrade, Copy and
recommended-equipment operations.

These rows are authoring references. They do not lower a service handler or
interpret graph programs at runtime.

## Items, special goods and season availability

The pack imports:

- 165 item catalog identities and priorities;
- 43 special goods with group, quality, price, parameters and mode-owned
  configuration path; and
- 164 exact season-to-item availability edges.

The five exact character Shop price tiers were already imported by P1-B3.
Together, these rows account for every non-presentation obligation in the
frozen service counter group.

No direct Gamble, curse-chest or Adventure-outcome table exists in the
GridFight closure, so their normalized families are canonical empty arrays.

## Currency identity

Released bilingual text proves Gold Coin is the run-local recruitment and
refresh currency. The two `GridFightGamePlayResource` rows contain only
generic resource labels/icons and are frozen as
`EvidenceOnly` / `ExcludedPresentation`; neither is promoted by numeric ID.

The stable `currency-wars.currency.gold-coin` identity therefore remains
`Researched` / `ProjectPolicy` until released structured data binds a generic
resource row to Gold Coin mechanics.

## Result

The nine normalized files contain 389 rows: 388 direct service obligations and
one policy-bound Gold Coin identity. Two presentation-only resource rows are
explicitly excluded. The combined digest is
`169adf20dd7b1ce79622a67abf423fe191ed521b4d19042c27aca21abd88008b`.

```text
fnm exec --using 24.15.0 node \
  tools/currency-wars-reference/import-services.mjs \
  --source-cache .cache/content-reference/turnbasedgamedata
fnm exec --using 24.15.0 node \
  tools/currency-wars-reference/verify-services.mjs \
  --source-cache .cache/content-reference/turnbasedgamedata
```
