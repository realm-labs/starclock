# Currency Wars roster and economy

Batch `G12-P1-B3` imports only direct GridFight Version 4.4 rows. It does not
add runtime purchase, refresh, roster or battle behavior.

## Exact roster and prices

`GridFightRoleBasicInfo` contains 77 distinct role IDs, all marked `IsInPool`.
Each normalized role preserves its upstream role ID, avatar ID, authored
front/back value when present, rarity, trait list, book/pool flags, special
avatar ID and backend rank list. Seven avatar IDs have multiple distinct
GridFight role variants, so role ID—not avatar ID—is the stable pack identity.

`GridFightShopPrice` publishes five rarity tiers. One-star buy prices are
exactly 1, 2, 3, 4 and 5 Gold; the normalized role `cost` uses this direct
rarity price. Each tier also preserves exact buy and sell prices for star
levels one through four. For rarity five, the exact vectors are:

- buy: `5`, `15`, `45`, `135`;
- sell: `5`, `14`, `44`, `132`.

## Offers, refreshes and levels

The ten `GridFightLevelV2` rows and ten
`GridFightConstValueCommonV2` card-weight rows publish rarity weights summing
to 100 at every level. Each offer row contains every one of the 77 roles whose
rarity has positive weight at that level. `GridFight_CardNumberPerRefresh = 5`
and `GridFight_LotteryRefreshGold = 2`.

`GridFightPlayerLevel`, `GridFightLevelV2` and
`GridFightRarityWeight` independently close over levels 1 through 10. The
normalized team-state rows retain the exact `AvatarMaxNumber` progression
`1` through `10`, next-level Experience, rarity weights and general property
additions. `AvatarMaxNumber` is preserved under its source field name; it is
not conflated with the separate direct positional constants:

- front minimum/maximum: `1` / `4`;
- back initial/maximum: `6` / `9`;
- authored bench count/overflow: `9` / `100`.

## Economy constants

The exact Standard constants include:

- wave/boss-wave Experience: `2` / `10`;
- direct level-up Experience/Gold: `4` / `4`;
- one interest per `10` deposited Gold, capped at `5`;
- Overclock interest cap `0`;
- Overclock wave Experience list `6`, `8`, `12`;
- Overclock boss-wave Experience list `10`, `12`, `16`.

Released bilingual Currency Wars text names Gold Coins, while the structured
rows encode `Gold` fields without a standalone stable resource identity.
`currency-wars.currency.gold-coins` is therefore a `ProjectPolicy` identity
with a replacement condition tied to a released GridFight resource row; all
numeric values remain `ExactStructured`.

## Result

The five normalized files contain 103 rows: 77 roles, one economy rule, ten
offer levels, five transaction tiers and ten team-size states. Their combined
digest in lexicographic file order is
`491cf88920bc4f042bb55b3f1a53c518de1a8a9ac49a603bceacfccf42428a9c`.

```text
fnm exec --using 24.15.0 node tools/currency-wars-reference/import-economy.mjs \
  --source-cache .cache/content-reference/turnbasedgamedata
fnm exec --using 24.15.0 node tools/currency-wars-reference/verify-economy.mjs \
  --source-cache .cache/content-reference/turnbasedgamedata
```
