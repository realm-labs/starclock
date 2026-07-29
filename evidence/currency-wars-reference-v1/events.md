# Currency Wars events, variants and choices

Batch `G12-P2-B3` imports the complete mechanical Pray, Present and tutorial
event closure while excluding assistant presentation messages.

## Pray events

All 88 `GridFightPrayQuest` rows become event and choice records:

- 73 delayed/conditional events reference an exact
  `GridFightPrayQuestFinishWay`;
- 15 events apply an immediate authored outcome and have no finish-way
  reference;
- accept and finish bonuses remain ordered separately; and
- Fate type, typed finish parameters, progress and backtracking flags are
  preserved exactly.

All 73 finish-way rows resolve from at least one Pray event. The normalized
pack carries released bilingual titles/descriptions and independent mechanical
summaries, not dialogue.

## Present and tutorial events

Both `GridFightPresentConfig` rows preserve their Perfect/Lose shortening
boundary and deterministic bonus. All 77 tutorial tasks retain only task
identity and mode-owned level-graph path. Dialogue, animation and presentation
are not imported; graph operations remain references for P2-B6.

## Presentation exclusion

The four `GridFightAssistantMessage` rows are frozen manifest obligations with
`EvidenceOnly` / `ExcludedPresentation` disposition. None appears in the
normalized event files.

## Result

The three normalized files contain 407 rows: 167 primary occurrences, 150
variants and 90 choices. They account for 167 direct event obligations and 73
finish-way obligations; four presentation rows are explicitly excluded. The
combined digest is
`8ccfd0ce5238349bb38d9b64933dc5f6c9f59e4e583b5ddd468827ab06b1504b`.

```text
fnm exec --using 24.15.0 node \
  tools/currency-wars-reference/import-events.mjs \
  --source-cache .cache/content-reference/turnbasedgamedata
fnm exec --using 24.15.0 node \
  tools/currency-wars-reference/verify-events.mjs \
  --source-cache .cache/content-reference/turnbasedgamedata
```
