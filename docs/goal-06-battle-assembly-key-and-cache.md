# Goal 06 Battle Assembly Key and Cache

This document records the Phase 2 catalog/assembly split. It is normative for
`G06-P2-B1` and is subordinate to
`29-combat-identity-and-dynamic-assembly.md`.

## Lifetime split

`UniverseBattleCatalogComposition` is built once from the released
`UniverseCatalog`. It validates and retains:

- the immutable core combat catalog;
- all Standard Universe encounter definitions;
- exact and approved proxy enemy mappings;
- encounter-content closure and its canonical digest;
- the composition revision and digest.

The composition contains no owned Blessing, Curio, Resonance, Formation,
Ability Tree or participant-carry state. A selected assembly starts from this
immutable composition and supplies those values separately. The current
compatibility materializer still creates a validated selected-definition
overlay; Phase 2 replaces its entry-time ownership snapshot with the current
Activity snapshot before removing the compatibility access path in Phase 3.

## Canonical key

`BattleAssemblyKey` v1 commits, in fixed order, to:

1. immutable catalog-composition digest;
2. participant-lock digest, which includes resolved build identities;
3. encounter or encounter-overlay digest;
4. selected contribution digest;
5. participant-carry digest;
6. optional preparation/technique digest.

The key stores the exact fields as well as their canonical SHA-256 digest.
Equality and ordering therefore do not trust only a hash collision boundary.
The full-overlay compatibility path uses the encounter-content digest and the
canonical empty-carry digest. Later per-encounter assembly uses the same type
with the exact prepared encounter and carry digests.

## Cache policy

`BattleAssemblyCache` is a standard-library `BTreeMap` plus FIFO insertion
order. Its default capacity is 64 immutable assemblies.

- capacity is non-zero and explicit;
- hits do not change eviction order;
- inserting a distinct entry at capacity evicts the oldest insertion;
- replacing an exact key does not grow the cache;
- a returned entry must carry the requested key;
- counters saturate and are diagnostics only;
- clear, eviction, hit counts and allocation layout are not canonical state;
- Activity, RNG, replay and battle hashes never encode cache state.

FIFO is chosen because it is deterministic and cheap to audit. It is not a
gameplay policy. A later performance batch may change the scratch policy only
if cache-enabled and cache-disabled outputs remain byte-identical.

## Failure boundary

Catalog composition and selected assembly finish before an entry can be
inserted. A key mismatch is rejected. Cache corruption is a typed error and
must trigger recomputation or fail preparation before any Activity mutation.
P2-B3 and P2-B4 add the atomic preparation and exact rollback proofs.

