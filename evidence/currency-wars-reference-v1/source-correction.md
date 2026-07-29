# Currency Wars GridFight source correction

Batch `G12-P0-B5` corrects the fixed Version 4.4 source oracle without
rewriting any published Goal 12 commit or any other Goal artifact.

## Direct selector

| Source | Locator | SHA-256 | Result |
|---|---:|---|---|
| `ExcelOutput/GuideRogueTab.json` | `2` | `984f6e53d53424adb2962c19dbc0a6e1cd039adad2bba3393962f4339274a976` | Guide tab `1003` has `GuideType = GridFight`; the independently resolved EN/CHS name is Currency Wars / 货币战争. |
| `ExcelOutput/GuideRogueData.json` | `5` | `3162accac44c825d114b06c9b71f08f520a78c2dfcd91a272e99bfa1c341cb5e` | Guide data `301` selects tab `1003`; the independently resolved EN/CHS name is Currency Wars / 货币战争. |

The selector establishes a GridFight namespace closure. `GridFight` in a path,
a table prefix, an ID range or a matching name is not independently accepted
as row ownership evidence.

## Reproduced source closure

The ignored isolated cache at `.cache/content-reference/` was reconstructed at:

- `turnbasedgamedata@fd978d6ef09f941fba644c731ab54abd6f7c3568`
- `StarRailRes@7b349e39ee0f6f3bf814567995829b99c95e7a93`

Both repositories were detached at the exact commit, clean, readable and
passed `git fsck --connectivity-only --no-dangling`. Two bounded remote clone
attempts failed with LibreSSL transport errors; the third succeeded. The
turnbasedgamedata clone uses a read-only Git object alternate to the main
workspace cache for already-published Goal 03 blobs; the main cache was not
modified. The interrupted partial checkout was preserved at
`/tmp/starclock-g12-interrupted-cache.SmRNNL/turnbasedgamedata`.

The fixed Git tree contains exactly:

- 153 `ExcelOutput/GridFight*.json` tables;
- 18,234 top-level rows across those tables;
- 984 GridFight configuration paths;
- 1,137 GridFight paths in total.

`source-inventory.json` now contains 3,822 files: all 2,646 inherited Goal 03
paths, all 1,137 GridFight paths and 39 identity/reconciliation inputs. Its
SHA-256 is
`4cfce0e8d4dab1f9927d2b26edddd42039f13b6568961f9317c206d98b1019dc`.

## Corrected denominator

`content-manifest.json` accounts for every one of the 18,234 GridFight table
rows exactly once, all 984 GridFight config files, two direct Guide selector
rows, two released Gambit policy obligations and 28 semantic fixture families.
The resulting denominator is 19,250 obligations in 16 frozen counter groups;
its SHA-256 is
`6bfe1c885fa22bb2c1df399a0e59f56cb7419f77e09c45e235c2dcb2036c10ef`.

Direct GridFight Curio/Miracle/Hex identities and direct GridFight Blessing
identities are proven empty only inside the complete GridFight namespace.
Shared content is not declared absent: P2-B1/P2-B2 must still prove any
reachable shared stable-ID closure.

Goal 11 remains the owner of `TournRogue` / `Tourn3` / module `6002201`.
Goal 12 owns the distinct `GridFight` selector. Reconciliation reopens only if
a typed GridFight-originating reference reaches a row claimed exclusively by
Goal 11.

Regenerating the normalized contract changed its bound manifest digest. The
six still-valid Squad HP/action-value rows were regenerated without semantic
change so that their row-level policy provenance resolves to the corrected
contract; their combined digest is
`f5b15ea1d853157c5bdfa77eb3d8d230e333781b134064cc5b146a7de8dee9ba`.
The superseded P1-B1 flow rows intentionally remain blocked until P1-B10.

## Reproduction commands

```text
tools/currency-wars-reference/fetch-sources.sh .cache/content-reference
fnm exec --using 24.15.0 node tools/currency-wars-reference/inventory.mjs --source-cache .cache/content-reference
fnm exec --using 24.15.0 node tools/currency-wars-reference/manifest.mjs --root . --source-cache .cache/content-reference/turnbasedgamedata
fnm exec --using 24.15.0 node tools/currency-wars-reference/contracts.mjs --root .
fnm exec --using 24.15.0 node tools/currency-wars-reference/verify-source-correction.mjs
fnm exec --using 24.15.0 node tools/currency-wars-reference/verify-inventory.mjs --source-cache .cache/content-reference
fnm exec --using 24.15.0 node tools/currency-wars-reference/verify-manifest.mjs --source-cache .cache/content-reference/turnbasedgamedata
fnm exec --using 24.15.0 node tools/currency-wars-reference/verify-contracts.mjs
```
