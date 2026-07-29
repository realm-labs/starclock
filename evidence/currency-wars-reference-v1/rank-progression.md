# Currency Wars rank, affix and progression boundaries

Batch `G12-P1-B9` imports only rank and persistent-progress facts that change a
legal entry, difficulty, starting scalar, available choice or battle
contribution.

## Rank and Gambit boundary

Released bilingual rules prove that Standard Gambit advances the current rank,
that losses do not reduce it and that Overclock Gambit may not select a
difficulty above the highest Standard rank. The direct structured closure adds:

- ten season division display levels;
- 23 chapter/section base Attack and HP rows; and
- 23 Stage-specific base Attack and HP rows.

The normalized rows retain the Standard-to-Overclock cap without importing
rank reward quests. No field claims an unproven route-to-Gambit mapping.

## Enemy affixes and difficulty scaling

All 51 `GridFightAffixConfig` rows, 67 `GridFightAffixMazebuff` rows and 603
`GridFightEnemyDifficultyLv` rows are imported exactly once. Affix definitions
retain configuration paths, direct MazeBuff references and canonical
parameters. Every referenced MazeBuff ID resolves in the retained MazeBuff
set.

Difficulty rows preserve exact per-Chapter Attack, Defence, HP, Speed and
Stance ratios for all authored difficulty levels. They remain distinct from
affix selection because no name, nearby ID or shared config path proves a
specific rank-to-affix assignment.

## Mechanically relevant permanent progression

The 162 retained rows comprise:

- 80 season score/Experience mappings by Division, score rule, Chapter and
  section;
- 77 role in-game reference scores;
- two explicit module-to-banned-role edges; and
- three entry unlock conditions.

`GridFightScoreReward` and the reward quest fields in division display rows are
excluded. They describe account rewards, not legal entry, starting state,
offered content or battle contribution.

## Result

The three normalized files contain 939 rows: 56 rank/base-value boundaries,
721 affix/difficulty rows and 162 persistent progression rows. Their combined
digest in lexicographic file order is
`6778ec18cf265959139f7983a1e526b0c0831ed042627db64e9915d1acc2c102`.

```text
fnm exec --using 24.15.0 node \
  tools/currency-wars-reference/import-rank-progression.mjs \
  --source-cache .cache/content-reference/turnbasedgamedata
fnm exec --using 24.15.0 node \
  tools/currency-wars-reference/verify-rank-progression.mjs \
  --source-cache .cache/content-reference/turnbasedgamedata
```
