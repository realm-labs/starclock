# Goal 19 Noble Phantasm Catalog Audit

`G19-P1-B3` normalizes 172 enabled records at digest
`7f7c6ef2c14dabb5ba11c8f80bb30541b79f7346c5170d88a9548c5d5d452223`:
34 core Noble Phantasms, 107 FateRin configuration rows, three rarities, five
tags, twelve keywords, four decks and seven deck recommendations.

The records retain catalog, grouping, rarity, tag, keyword and deck
relationships independently. A matching display name does not merge a core
Noble Phantasm with its FateRin configuration row. Acquisition and effect
program meaning remains explicit source-shaped data until later rule bindings
and fixtures type it.

```text
node --max-old-space-size=4096 tools/fate-star-rail-night-reference/normalize.mjs \
  --source-cache .cache/fate-star-rail-night-sources --batch G19-P1-B3 --check
```
