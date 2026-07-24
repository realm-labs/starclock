# Goal 06 Activity Battle Identity Evidence

`G06-P1-B2` carries the two battle identities through the complete Activity
boundary:

- pending battle and player views;
- one-battle and graph-Activity handoff;
- deterministic battle-seed derivation;
- result envelope and result digest;
- awaiting-battle canonical state;
- settlement identity validation.

`CombatInputDigest` and `AssemblyDigest` are compared independently. Tests
submit a result with only one digest changed and prove rejection leaves the
complete Activity state hash byte-identical.

Because these fields change authoritative Activity state bytes, the current
state codec advances from `SCAS`/`starclock-activity-state-v2`/`sha256-v4` to
version 3 / `starclock-activity-state-v3` / `sha256-v5`. Existing Goal 04 and
Goal 05 evidence remains historical and immutable.

The current Activity command, nested-boundary and battle-result payloads use
their dual-identity version 2; graph-Activity command payloads use version 3.
The released single-digest payload versions remain accepted by explicit
legacy decoders and are not emitted for new recordings. Component-addressed
replay v3 and first-divergence reporting are completed in `G06-P1-B3`.

## Verification

```text
node tools/goal06/verify-phase1-b2.mjs
cargo test -p starclock-activity
cargo test -p starclock-replay
cargo test -p starclock-mode-universe --lib --tests
cargo check --workspace --all-targets --all-features
```
