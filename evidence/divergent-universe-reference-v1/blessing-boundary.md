# Divergent Universe Blessing Boundary

## Active Path and identity closure

The exact `Tourn3` `RogueTournUseBuffType` row selects Path types `121`,
`122`, `124`, `125`, `126`, `127`, `128` and `129`.

Filtering `RogueTournBuff` by those selectors yields:

- 414 stable Blessing identities;
- 828 level rows, exactly base level 1 and enhanced level 2 per identity;
- 184 `Common`, 161 `Rare` and 69 `Legendary` identities; and
- 54 identities for each of Path types 121–128 except type 123, which is not
  selected, and 36 identities for type 129.

Every level resolves an exact same-ID/same-level `RogueMazeBuff` row.
Normalized levels preserve bilingual names, Path/category, authored tag,
extra-effect IDs, modifier and binding keys, canonical parameters and
base/enhanced state. Ability programs are evidence locators only and are not
runtime-lowered.

## Equation contribution

Both authored levels use the same `MazeBuffID`; Equation recipes count required
Blessings by Path and quantity, not by level. Goal 11 therefore treats the
owned Blessing identity as the contribution unit:

- base and enhanced forms each contribute one count;
- enhancement preserves the identity contribution;
- removing or replacing an identity removes its prior contribution; and
- an accepted output identity contributes to every owned Equation whose main
  or sub Path matches.

The generated contribution row for each Blessing lists every matching current
Equation and refreshes when the owned Blessing identity set changes.

## Group boundary

There are 118 exact `Tourn3` `RogueTournBuffGroup` rows:

- 37 groups contain two source candidates;
- 34 contain three;
- 25 contain seven; and
- 22 contain eight.

Fifty-seven groups resolve entirely through current mode-owned
`RogueTournBuffTag` values. Sixty-one groups contain one or more IDs belonging
to shared Rogue source families; across all groups there are 176 unresolved
source-ID occurrences. Those rows remain `Researched` with
`membership_resolution = DeferredToP2B1`. Source order is preserved, but
membership and weights are not invented.

## Enhancement and rewrite boundary

Each identity has an exact base-to-enhanced transition supported by its two
`RogueTournBuff` rows. Generic replacement or Path rewrite still lacks a
released candidate set, cost, target order and fallback. The two generic
rules are explicit `ProjectPolicy`: inputs and accepted outputs use stable
IDs, and invalid/no-legal operations reject without mutation.

The policy must be replaced by released workbench/service programs or
reproducible observations. It is reference semantics only and does not claim
runtime executability.

## Reproduction

```text
node tools/divergent-universe-reference/import-blessings.mjs \
  --source-cache <turnbasedgamedata-cache>
node tools/divergent-universe-reference/import-blessings.mjs --check \
  --source-cache <turnbasedgamedata-cache>
node tools/divergent-universe-reference/verify-blessings.mjs \
  --source-cache <turnbasedgamedata-cache>
```

The verifier checks all manifest obligations, active Path and
category distributions, exact two-level joins, group candidate accounting,
enhancement transitions, fail-closed shared group IDs and all
Blessing-to-Equation edges.
