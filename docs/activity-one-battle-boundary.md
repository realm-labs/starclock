# One-Battle Activity boundary

Goal 01 batch `G01-P6-B1` implements the smallest authoritative
`starclock-activity` aggregate: one generic Activity contains one Section, one
Battle Node, one Attempt and one Battle handoff, then selects exactly one
Complete, Failed or Faulted Terminal node. Challenge, universe, route,
fork/join, reward and shop semantics remain absent.

## Scope and slot ownership

The public scope path is `Activity -> Section -> Node -> Attempt`; Battle and
shorter scopes remain owned by `starclock-combat`. Activity slot definitions use
a closed tagged value domain, explicit owner scope, checked bounds and canonical
reset points. A reset cannot occur before its owner exists. Construction enters
the four generic scopes in order, `StartBattle` applies `BattleStart`, and an
accepted result applies `BattleEnd`. Duplicate slot identities and noncanonical
reset sequences are rejected before aggregate construction.

## Participant and build locks

Participant rows retain only stable participant/character/formation identity,
source kind, build-catalog revision and opaque build/resolved-spec SHA-256
values. The lock validates team bounds, uniqueness and formation occupancy,
sorts rows canonically, and verifies a revisioned participant-lock digest.
Neither build selections nor progression/equipment concepts enter Activity or
combat state.

## Handoff and result return

`StartBattle` returns an immutable `BattleHandoff` containing the opaque
`BattleSpec`, a purpose-derived `BattleSeed`, and the expected result identity.
The result identity binds the Activity/Section/Node/Attempt/Battle sequence,
activity definition and configuration digests, participant-lock digest,
BattleSpec digest and seed. The seed encoder is domain-separated and includes
the authored seed-stream label and BattleSpec policy revision.

`BattleResultProjection` is declared before start. The four verification fields
— outcome, final battle-state hash, event digest and optional terminal fault —
must each occur once; typed metrics are accepted only when declared by exact key
and value kind. Submission verifies every identity component, recomputes the
result digest, matches the projection positionally, and requires a fault exactly
for a Faulted outcome.

Both commands carry the expected activity-state hash. All validation occurs
before mutation, so stale commands and rejected result identities, hashes or
projections preserve the complete canonical hash. Golden tests pin the initial,
pending-handoff and terminal activity hashes plus the derived battle seed.

## Dependency direction

`starclock-activity` depends on `starclock-combat` only for battle-domain types
and privately reuses the already reviewed pinned SHA-256 primitive. The combat
manifest has no reverse dependency; an architecture test and the workspace
dependency verifier protect that direction. `BattleStateHash::from_bytes`
permits verified replay/activity transports to reconstruct an opaque returned
hash without exposing combat state internals.
