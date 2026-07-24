# Goal 06 Combat Input Identity Evidence

`G06-P1-B1` establishes two independent identities at the battle boundary.

- `CombatInputDigest` is SHA-256 over the combat-owned `SCBI` version 1
  canonical encoding.
- `AssemblyDigest` is opaque provenance supplied by the build or Activity
  assembly owner.
- `BattleSpec::new_with_assembly` canonicalizes participants before computing
  the combat-input digest. No caller parameter can override that digest.
- `BattleIdentity` retains both values independently.

The codec includes rules revision, encounter, canonical participant placement,
source and carry state, the complete resolved combatant selection, Toughness,
definition/source bindings, team resources and battle-local policy. It uses
explicit tags, fixed-width little-endian integers and length-prefixed
collections/text rather than serialization output.

The legacy `BattleSpec::new` and `BattleSpecDigest` surface remains only as a
temporary compile-safe migration bridge through `G06-P1-B4`. Its argument is
treated as `AssemblyDigest`; combat input is still computed internally.

Canonical battle-state and replay bytes deliberately remain on historical
`sha256-v3`/replay v2 in this batch. `G06-P1-B3` performs their coordinated
version transition so historical bytes are never silently relabeled.

## Verification

```text
cargo test -p starclock-combat
cargo clippy -p starclock-combat --all-targets --all-features -- -D warnings
cargo check --workspace --all-targets --all-features
node tools/goal06/verify-phase1-b1.mjs
```

Tests prove that canonical participant order is stable, assembly-only
provenance changes do not change combat identity, and each represented
top-level battle-input family does change the digest.
