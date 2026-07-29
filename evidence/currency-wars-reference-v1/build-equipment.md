# Currency Wars build and equipment mappings

Batch `G12-P1-B7` imports the direct GridFight owned/trial identity boundary,
off-field conversion rows and equipment lifecycle without lowering any of
them into runtime behavior.

## Owned and trial build boundary

All 77 `GridFightRoleBasicInfo` rows publish an account `AvatarID` and a
mode-local `SpecialAvatarID`. The normalized pack preserves those two
identities and released bilingual text proving that Currency Wars may use an
owned character or provide/strengthen a trial build.

The pinned snapshot does not publish an explicit row-level join from each
GridFight role to the shared upgrade-avatar tables. Consequently:

- the 77 identity rows are `DataReady`;
- the 77 level, Trace, Light Cone and relic mappings remain `Researched`
  `ProjectPolicy`;
- the two selection/teardown rules expressly prohibit account mutation; and
- all six shared build-source files remain
  `PendingExplicitRoleRowJoin`.

This is a fail-closed ownership boundary. File membership, avatar names and
numeric adjacency do not promote a shared build row.

## Off-field conversions

The pack imports all 252 `GridFightBackRoleRank` rows and all 165
`GridFightBackEquipment` rows. Each conversion preserves its exact eligibility
keys, owner and all-member property contributions, skill modifications,
ability references and canonical decimal parameters. These 417 rows are
direct position-system obligations rather than inferred account Eidolon or
Light Cone mappings.

## Equipment lifecycle

The 518 direct equipment rows comprise:

- 148 equipment definitions;
- 14 equipment categories and their count limits;
- 32 equipment tags;
- 37 explicit predecessor-to-upgrade edges;
- 133 equipment-to-role recommendations; and
- 154 role/position-to-equipment recommendations.

Recommendations are retained as advisory evidence and never treated as
authoritative selection or replacement behavior. Upgrade and category rows
preserve only their directly authored replacement and capacity boundaries.

## Result

The six normalized files contain 1,097 rows: 77 build-reference identities,
six fail-closed shared source files, 77 policy-bound build mappings, two
substitution/teardown rules, 417 off-field conversions and 518 equipment rows.
Their combined digest in lexicographic file order is
`6b11eb6843e4ee97e6e009e7ae6a32a8ba698c2a64df9ac4341311565a92f667`.

```text
fnm exec --using 24.15.0 node \
  tools/currency-wars-reference/import-build-equipment.mjs \
  --source-cache .cache/content-reference/turnbasedgamedata
fnm exec --using 24.15.0 node \
  tools/currency-wars-reference/verify-build-equipment.mjs \
  --source-cache .cache/content-reference/turnbasedgamedata
```
