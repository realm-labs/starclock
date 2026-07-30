# Divergent Universe Blessing Pool Ownership

This dossier freezes the Version 4.4 Blessing, Path and group closure used by
`G11-P2-B1`. It is reference evidence only and does not lower a runtime
selector.

## Released selector boundary

- `ExcelOutput/RogueTournUseBuffType.json` selects eight Blessing types for
  `Tourn3`.
- The selected `ExcelOutput/RogueTournBuffType.json` rows define those eight
  mode-owned Path records.
- The selected `ExcelOutput/RogueTournBuff.json` rows define 414 mode-owned
  Blessing identities and 828 base/enhanced level records.
- `ExcelOutput/RogueTournBuffGroup.json` contains 118 `Tourn3` group rows and
  527 ordered direct references.

Every statement above is reproduced from
`turnbasedgamedata@fd978d6ef09f941fba644c731ab54abd6f7c3568`.
Each normalized row records the exact source path, zero-based array locator,
row digest and a short bilingual independent summary.

## Stable-ID closure

Of the 527 direct group references, 351 are `RogueBuffTag` values on a
mode-owned Blessing level and 176 refer to another one of the 118 `Tourn3`
groups. The latter comprise 54 distinct subgroup IDs. Expanding only those
explicit references produces an acyclic closure with a maximum depth of three
group rows and 2,095 unique root-group-to-terminal-level relationships.

No direct reference remains unresolved. The earlier P1-B4 classification of
the 176 occurrences as possible shared IDs is superseded by this exact
same-table subgroup closure. The source does not publish weights, so group
order is preserved and `weight_program` remains `Unspecified`.

## Shared-content reconciliation

Goal 08 checkpoint
`c283c7f195dcfe05854f3b212df73444ee89255a` contains 162 shared Standard
Universe Blessings. Their source-ID set has zero intersection with the 414
Divergent Universe Blessing source IDs. Ninety-seven English display names
match, but a matching name is not identity or reachability evidence and no
shared stable ID is reused.

Accordingly, this batch promotes no shared record. The generated
`pool-membership.json` contains only:

- eight explicit Tourn3 active-Path memberships;
- 414 explicit Tourn3 Path-to-Blessing memberships;
- 527 ordered direct group edges; and
- 2,095 transitive group-to-terminal-level closure rows.

The verifier reads the immutable Goal 08 checkpoint directly, checks its
ownership classification, proves the zero source-ID intersection and retains
the same-name count as an anti-identity regression. Goal 09 and Goal 10
checkpoint reconciliation remains isolated for the complete cross-mode audit
in P4-B3; this batch neither reads nor modifies their artifacts.
