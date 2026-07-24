# Goal 06 Current Activity Battle Snapshot

This document is normative for `G06-P2-B2`. It defines the immutable input
captured before dynamic Standard Universe battle assembly.

## Snapshot contents

`StandardUniverseBattleSnapshot` owns exact typed projections of:

- source `ActivityStateHash`;
- participant-lock digest;
- selected Path passive;
- exact owned Blessing levels;
- unlocked Resonance and Formations;
- each owned Curio's current lifecycle state and charge;
- selected Ability Tree nodes;
- evaluated battle-boundary Ability Tree values;
- compiled rule, modifier, resource and source contributions;
- ordered participant HP, Energy, life and presence carry.

Each component remains inspectable and retains its own digest. The snapshot
also computes a carry digest and a full provenance digest. It does not borrow
the live Activity and cannot observe later commands.

## Authoritative context

`StandardUniverseActivity::battle_start_snapshot()` is the production entry
point. It derives:

- chosen-Path Blessing count from the current typed Path contribution;
- first-battle completion from the generic Activity completed-battle count;
- carry from the current player-visible carry ledger;
- the source state hash from the same immutable Activity borrow.

The lower-level context-taking method remains for explicit boundary tests and
rejects a caller context that disagrees with current Activity state.

## Identity and stale detection

The full snapshot digest binds the source Activity state hash, participant
lock, evaluation context, every component digest, the final contribution
digest and the carry digest. Consequently:

- equal executable contributions after a provenance-only Activity command may
  retain their contribution/carry digests but receive a different snapshot
  digest;
- a stale snapshot is detectable before preparation commits;
- acquiring, upgrading, disabling or removing a battle contribution changes
  the corresponding typed component and final contribution digest;
- carry-only changes are visible even when selected rules are unchanged.

P2-B3 consumes this object exactly once while preparing the pending encounter.
No running battle reads `StandardUniverseActivity`.

