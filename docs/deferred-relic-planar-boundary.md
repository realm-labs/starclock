# Deferred Relic and Planar Boundary

Goal 01 batch `G01-P5-B6` protects the future relic/planar integration point
without importing or claiming the excluded full dataset.

## Narrow compatibility contract

`DeferredRelicBoundary` is an explicit, unconstructably nonempty field of every
normalized `CombatantBuildSpec`. Its revision is
`relic-planar-deferred-empty-v1` and its piece count is exactly zero. The field
has no set ID, affix ID, magnitude, roll, level, rarity, source row or runtime
binding. Revision verification rejects future/unknown values instead of
silently interpreting them as the current empty boundary.

The closed slot-family mapping records only the stable structural seam:

- Cavern: Head, Hands, Body and Feet;
- Planar: Planar Sphere and Link Rope.

No set bonus, stat curve, affix legality, roll reachability, piece-count
threshold or inventory behavior is implemented in this batch.

## Digest and combat behavior

The normalized selected-build encoder now includes the boundary revision and an
explicit zero piece count. Because that changes the canonical byte layout, its
domain is advanced to `starclock-combatant-build-v2` and the selected-build
golden is re-pinned. Character, Light Cone, build-catalog and resolved-combatant
digest domains remain unchanged.

The compiler has no relic stage or hidden default contribution. The combat
crate receives no relic/planar family, slot, set, affix, piece or boundary type.
The workspace dependency verifier continues to enforce the one-way
`starclock-build -> starclock-combat` edge.

[`relic_boundary.rs`](../crates/starclock-build/tests/relic_boundary.rs) pins
the empty revision, zero count, incompatible-revision rejection and complete
six-slot family mapping. The existing build-identity golden proves the empty
boundary participates in the normalized build digest.

Production remains Excel/Sora-only and unchanged. No workbook row or prepared
reference record is enabled, no relic or planar definition is imported, and
Goal 01 coverage remains 0/283 `DataReady`.
