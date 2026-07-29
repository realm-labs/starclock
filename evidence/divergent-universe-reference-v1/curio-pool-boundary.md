# Divergent Universe Curio Pool Boundary

`G11-P2-B2` closes the released Version 4.4 Curio catalog without inventing
offer membership. It does not implement runtime Curio behavior.

## Exact mode catalog

The pinned `RogueTournMiracle` rows contain 235 `Tourn3` mode copies:

| Category | Copies |
|---|---:|
| Common | 73 |
| Rare | 96 |
| Legendary | 21 |
| Negative | 45 |

Each copy binds one `RogueMiracleEffect` row, its canonical parameter vector
and released English and Simplified Chinese effect text. The normalized state
stores independent bilingual summaries, both effect-text digests, a
deterministic trigger taxonomy, battle/cross-battle visibility and explicit
lifecycle markers. The source text mentions destruction, discard, repair,
replacement or leveling for 75 copies; absence of such text is recorded as
not declared, not inferred as impossible.

Twelve copies have no explicit `HandbookMiracleID`. Four happen to share a
numeric value with a handbook row and several share display names with other
records. Neither coincidence is treated as an identity reference. They remain
anonymous mode copies until a released foreign-key or transition program
proves the identity.

## Offer groups

All 286 `RogueTournMiracleGroup` rows publish only
`RogueMiracleGroupID`; they contain no member, weight, exclusion, draw-count
or fallback fields. Exact typed traversal of `RogueTournGambleUnit` finds 12
references to 12 distinct groups:

- three Common groups;
- three Rare groups;
- three Legendary groups; and
- three Negative groups.

Those consumer and category bindings are exact. The referenced groups still
have empty candidate and weight arrays because no released row connects an
individual mode copy to one of them. The other 274 groups have no typed Tourn3
consumer in the frozen source closure.

`curio-pool-membership.json` therefore records the 235 exact Tourn3 category
catalog memberships, with `weight=Unspecified`, empty source-group IDs and an
explicit warning that catalog membership is not offer eligibility. A category,
matching name, table prefix or numeric shape is never used to populate an
offer group.
