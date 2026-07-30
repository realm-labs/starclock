# Currency Wars position, Empowerment and battle overrides

Batch `G12-P1-B4` records reference contributions only. It does not lower a
skill, ability, battle event or public rule into runtime code.

## Positions and role mappings

All 77 `GridFightRoleBasicInfo` roles have exactly two
`GridFightRoleSkillDisplay` rows: one Front and one Back. Fifty-seven role rows
directly author `FrontBackType` (`35` Front and `22` Back). Twenty omit the
field. Released Currency Wars text states that some roles are Front-Back, but
the released JSON does not include the enum schema needed to prove that
omission encodes that value. Those twenty mappings therefore retain both exact
display positions as candidates and remain `Researched`.

The normalized position identities are Front, Back and an explicitly
policy-bound Front-Back candidate. Placement activation and teardown are
reference facts, not runtime handlers.

## Character Empowerments

The generated data accounts exactly once for:

- 154 role/position display rows;
- 4,052 `GridFightFrontSkill` level rows;
- 446 `GridFightBackBESkillConfig` rows.

Every skill preserves its skill ID, authored level, trigger key, cooldowns,
SP/delay ratios and parameter vector. A numeric skill-ID prefix is not used to
infer avatar ownership; skill rows without an explicit role join leave
`avatar_id` empty.

## Structured battle overrides

`battle-overrides.json` accounts exactly once for:

- 119 back battle-event configurations;
- 24 Front special-SP rows;
- six role-global saved-value modifiers;
- 124 rank/skill parameter modifiers;
- two summon battle-event JSON overrides;
- 63 Cyrene skill modifiers.

These rows preserve ordered parameter indexes/operators, property
contributions, ability names and config paths. They do not claim that
Starclock can execute those programs.

## Released cross-battle rules

Released bilingual Currency Wars rules prove:

- defeating an enemy restores 50% of regular-combat defeat energy;
- lethal damage prevents incapacitation, immediately restores some HP and
  reduces the remaining battle countdown.

The `0.5` energy ratio is exact public text. The rescue HP amount and countdown
loss are not stated there, so they remain `ConfiguredByBattleRule` and
`Researched`; no numeric value is invented.

### Phase 4 semantic-review addendum

Batch `G12-P4-B2` promotes the independent CHS/EN Version 4.4 TextMap rule
that on-field Currency Wars characters automatically use their Techniques in
combat. It adds one `ExactPublicText` battle-override row with a
`BeforeBattleStart` reference boundary and does not lower that rule into
runtime behavior. The current four-file surface therefore contains 5,073 rows,
including 341 battle overrides, with digest
`e531af0bb3859c91e2887809733adfb140ac96e985717b948ad00a0ddb66768f`.

## Result

The four normalized files contain 5,072 rows: 77 role mappings, three
positions, 4,652 Empowerments and 340 battle overrides. Their combined digest
in lexicographic file order is
`f3f57aed05bc7543530e89ba77221bdb3ae9b849279cd88481a0aaaabf023aef`.
Those figures remain the immutable `G12-P1-B4` batch result; the Phase 4
addendum above records the current additive reference row.

```text
fnm exec --using 24.15.0 node \
  tools/currency-wars-reference/import-position-empowerment.mjs \
  --source-cache .cache/content-reference/turnbasedgamedata
fnm exec --using 24.15.0 node \
  tools/currency-wars-reference/verify-position-empowerment.mjs \
  --source-cache .cache/content-reference/turnbasedgamedata
```
