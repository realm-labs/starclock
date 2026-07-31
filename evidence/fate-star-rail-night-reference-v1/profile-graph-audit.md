# Goal 19 Profile and Graph Audit

`G19-P1-B1` normalizes 99 enabled records from twelve released structured
families at digest
`0054237e308da60c54362cf5e95a5acf36ef93893316f17f3f88436feaf60b9b`.
The pack contains three areas, seven difficulties, ten phases, twelve battle
zones, eight difficulty-progress rows, seven FateRin day-progress rows, six
Case Boards, eighteen Case Board nodes, four challenge fights, six story-fight
locators, three Noble-Phantasm map groups and fifteen map fights.

Source ordering and scalar relationships are transported exactly. Obfuscated
upstream field keys remain visible as reviewable source-shaped payload fields;
the pack does not invent field meanings. Bilingual labels use a short released
CHS/EN TextMap value when both are available and otherwise use an independent
family/ordinal label. Long text, assets and presentation paths are omitted.

```text
node --max-old-space-size=4096 \
  tools/fate-star-rail-night-reference/normalize.mjs \
  --source-cache .cache/fate-star-rail-night-sources --batch G19-P1-B1
node --max-old-space-size=4096 \
  tools/fate-star-rail-night-reference/normalize.mjs \
  --source-cache .cache/fate-star-rail-night-sources --batch G19-P1-B1 --check
```
