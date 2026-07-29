# Currency Wars Blessing and formula closure

Batch `G12-P2-B1` closes the complete generated Blessing/level/formula counter
group without treating enemy Affixes or generic MazeBuff enhancements as
Blessings.

## Frozen denominator

The generated manifest contains exactly 125 obligations in this counter group:

- 51 `GridFightAffixConfig` rows;
- 67 `GridFightAffixMazebuff` rows; and
- seven `GridFightMazeBuffEnhance` rows.

P1-B9 already accounts for all 118 enemy Affix rows. P2-B1 imports all seven
MazeBuff enhancements with their released bilingual names, exact ability
bindings and canonical parameters.

## Proven-empty categories

The complete direct and transitive GridFight closure contains no reachable:

- Blessing or Blessing path identity;
- Blessing offer group or enhanced Blessing level;
- Equation-like formula or recipe;
- formula display, contribution or progress state; or
- formula randomizer.

Two generated closure rows bind that zero result to the frozen manifest
digest. The nine normalized file families still exist, with deterministic
empty arrays where the category is absent. This is a machine-checked zero, not
a conclusion from table names or Wiki totals.

`GridFightMazeBuffEnhance` rows carry
`blessing_id = none:maze-buff-enhancement` and an explicit
`not-a-blessing` tag. Enemy Affixes remain the independent system imported by
P1-B9.

## Result

The nine normalized files contain nine rows: seven exact source rows and two
generated closure proofs. Only the seven source rows account against the
manifest denominator. Their combined file digest is
`7185370483c40c2ef7df3421547cb7ad775a59c4d67786fdaee16414ba203f15`.

```text
fnm exec --using 24.15.0 node \
  tools/currency-wars-reference/import-blessing-closure.mjs \
  --source-cache .cache/content-reference/turnbasedgamedata
fnm exec --using 24.15.0 node \
  tools/currency-wars-reference/verify-blessing-closure.mjs \
  --source-cache .cache/content-reference/turnbasedgamedata
```
