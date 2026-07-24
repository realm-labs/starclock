# Goal 06 Replay v3 Evidence

`G06-P1-B3` coordinates combat identity, canonical battle state and replay:

- combat state advances to `SCBS` version 3 / `sha256-v4` and binds the
  combat-input codec revision, `CombatInputDigest` and `AssemblyDigest`
  independently;
- replay format version 3 retains the bounded component-addressed schema and
  rejects every unknown record kind;
- nested start payloads record component root, combat-input codec revision,
  computed combat identity, assembly provenance and exact handoff identity;
- nested end payloads record exact result identity and digest;
- Standard Universe v3 verification reproduces every command, event, state
  hash and result, then reports the first divergence in the frozen order:
  component, assembly, combat input, command, event, state, result, Activity;
- historical replay v2 decode and verification remain separate public entry
  points. A fixed v2 envelope SHA-256 guards its exact bytes.

The v3 transport is available in this batch. CLI, Agent and MCP production
emission switches together in `G06-P3-B1` and `G06-P3-B2`; until then the
existing surfaces keep their released behavior.

The authoritative Excel `ConfigManifest.state_hash_revision` was updated with
`openpyxl`, then the pinned Sora 0.3.0 pipeline regenerated and verified the
production bundle.

## Verification

```text
node tools/goal06/verify-phase1-b3.mjs
cargo test -p starclock-combat --all-targets --all-features
cargo test -p starclock-replay --all-targets --all-features
cargo test -p starclock-mode-universe --test battle_materialization
cargo clippy -p starclock-combat -p starclock-replay -p starclock-mode-universe --all-targets --all-features -- -D warnings
node tools/config-production/verify.mjs
node tools/repository-check/run.mjs
```
