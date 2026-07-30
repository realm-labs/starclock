# Divergent Universe Occurrence Graph Boundary

`G11-P2-B3` freezes all released Version 4.4 Tourn3 Occurrence identities and
variant bindings while excluding story text and refusing to invent unavailable
mechanical choices.

## Exact structured closure

- 118 `RogueTournHandBookEvent` rows define the current handbook identities.
- Every identity has exactly one distinct current variant after ordered
  `UnlockNPCProgressIDList` traversal.
- 97 `RogueTournNPC` rows define those variants.
- 83 variants bind one handbook identity, seven bind two identities and seven
  bind three identities.
- Duplicate appearances of a current variant inside one handbook unlock list
  are retained as ordered unlock receipts but deduplicated in stable identity
  arrays.

Every normalized identity and variant records its exact source path, zero-based
row locator and evidence digest. No identity is inferred from a title, numeric
range or neighboring row.

## Missing mechanical graphs

All 97 current NPC rows publish paths under
`Config/Level/Rogue/RogueNPC/RogueNPC_410/`. At
`turnbasedgamedata@fd978d6ef09f941fba644c731ab54abd6f7c3568`, every one of
those paths is absent according to Git object lookup. The absence is therefore
part of the pinned public source, not a sparse-checkout or local-cache
condition.

The source tree contains many Tourn1/Tourn2 occurrence graphs with similar
identifiers. They are other-mode evidence and are not substituted. Without the
exact Tourn3 graphs, option order, conditions, costs, results, target pools and
weights cannot be audited.

Consequently:

- `occurrence-choices.json` is an intentionally empty generated array;
- every variant preserves its published missing path and has
  `graph_resolution=MissingAtPinnedRevision`;
- every identity and variant remains `Researched`/`ProjectPolicy`;
- empty choice sets reject without mutation; and
- the replacement condition is a released exact-path graph or another
  released table that explicitly binds the variant to its mechanical graph.
