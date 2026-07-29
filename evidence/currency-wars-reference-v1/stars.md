# Currency Wars star states

Batch `G12-P1-B6` imports all direct GridFight star authoring and records the
three-copy lifecycle without adding runtime combination behavior.

## Exact states and scaling

`GridFightRoleStar` contains 266 states for all 77 roles. Every state closes
over exactly six `GridFightRankAttachment` rows, accounting for all 1,596
attachments. `GridFightServantStar` adds 29 explicit servant states across
nine role IDs.

The authored state boundary is not globally capped at star 3:

- 42 roles publish stars 1–3;
- 35 roles publish stars 1–4;
- seven servant families publish stars 1–3;
- two servant families publish stars 1–4.

This agrees with released text saying star 3 is “usually” the strongest form;
it does not justify deleting explicit star-4 rows.

Every role state preserves battle-event ID, skill overrides, front/back skill
lists, ability/config/AI paths, property modifiers and authored combat scalar
fields. Every servant state preserves its servant ID, config/AI paths, skill
overrides and HP/Speed inheritance fields.

## Combination and lifecycle

Released bilingual rules prove that three copies of the same star
automatically combine. A transition is emitted only when the same role has an
exact next RoleStar row. This produces 189 legal transitions, including 35
star-3-to-star-4 transitions. Copy counts to reach stars 1–4 are therefore
`1`, `3`, `9`, `27`.

The acquisition, sale and maximum-star rules remain `Researched`
`ProjectPolicy` boundaries. They record deterministic reference behavior and
exact roster sale-price linkage but do not claim observed precedence for
simultaneous purchase, sale, replacement or overflow.

## Result

The three normalized files contain 487 rows: 295 states, 189 combination
rules and three lifecycle rules. Their combined digest in lexicographic file
order is
`0b1dce92ce9685679a6bace32785a5ff303ab91d50524146ba3f0a66740efb9b`.

```text
fnm exec --using 24.15.0 node tools/currency-wars-reference/import-stars.mjs \
  --source-cache .cache/content-reference/turnbasedgamedata
fnm exec --using 24.15.0 node tools/currency-wars-reference/verify-stars.mjs \
  --source-cache .cache/content-reference/turnbasedgamedata
```
