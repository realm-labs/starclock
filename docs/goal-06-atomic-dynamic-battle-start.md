# Goal 06 Atomic Dynamic Battle Start

This document is normative for `G06-P2-B3`.

## Boundary

Dynamic assembly occurs only after encounter preparation has selected its exact
normal/technique sequence and before the generic Activity commits the
no-random-draw started marker.

`StandardUniverseBattleAssembler::start_pending_battle` performs:

1. capture the pending placeholder and exact Activity state hash;
2. identify its encounter member and selected technique sequence;
3. project one `StandardUniverseBattleSnapshot`;
4. reject a snapshot whose source hash is not the captured state;
5. derive the exact `BattleAssemblyKey`;
6. resolve or compile an immutable selected materialization;
7. select the same encounter member and technique variant;
8. clone its `BattleBinding` and result contract;
9. atomically replace the placeholder and seal the result contract;
10. return the handoff together with its matching immutable combat catalog.

The placeholder exists only to preserve the generic preparation decision
model. It is never executed by the dynamic entry point.

## Atomic Activity operation

`GraphActivity::start_assembled_pending_battle` checks the caller's original
state hash, clones `ActivityTransactionState`, validates the replacement
encounter, participant lock and player roster, and starts the battle on that
working state. The live state is assigned only after all validation, seed
derivation, participant carry and result-contract checks succeed.

The operation consumes no RNG draw. Any error discards the working state, so
canonical Activity bytes, pending identity, RNG counters and replay records
remain unchanged.

## Carry and identity

Dynamic materialization applies the snapshot's ordered carry through
`ParticipantInitialState`. The snapshot digest is added to the assembly root,
while combat-core independently computes the resulting `CombatInputDigest`.
The handoff therefore binds:

- current selected contributions and provenance;
- carry-derived battle-visible initial state;
- exact encounter and preparation choice;
- result contract and deterministic battle seed.

The selected materialization stores every technique definition needed to
reconstruct its prepared variants. Current Standard Universe supports the
existing zero-or-one technique sequence; larger sequences fail explicitly
instead of silently selecting a different input.

## Catalog ownership

The returned `StandardUniverseDynamicBattleStart` contains the exact
`Arc<CombatCatalog>` used to validate its `BattleSpec`. Executors must use this
catalog, not the entry-time compatibility catalog. Phase 3 migrates all
production surfaces to this paired handoff/catalog result.

