# Current state

Starclock maintains only the current source, data, behavior and test outputs.
Git history is the only historical record.

## Compatibility policy

- Old replay files, serialized state, API envelopes and generated configuration
  are not supported after the current implementation changes.
- The repository has no migration, legacy-decoder or historical-evidence
  requirement.
- Canonical hashes and current goldens may change intentionally and are replaced
  atomically with the implementation that produces them.
- Dependency and toolchain pins remain build inputs; they are not runtime
  compatibility promises.
- Game-version labels, source revisions and access dates in gameplay reference
  packs remain factual provenance and are retained.

## Current runtime

- `starclock-combat` owns deterministic single-battle execution.
- `starclock-activity` owns deterministic cross-battle orchestration.
- Standard battle, Standard Universe, Gold and Gears, and Swarm Disaster use the
  shared combat/activity kernels.
- Replay records and verifies only data produced by the current tree.
- CLI, Agent API and MCP are current adapters over the domain crates.

## Verification

Focused development and ordinary local completion use
`cargo test -p <package> [filter]` and package-scoped Clippy directly. The full
workspace suite runs in CI and locally only for shared-boundary changes or an
explicit merge check. Current Sora/workbook/data validators run only when their
owned inputs change. Seeded matrices, large property corpora and performance
workloads are explicit exhaustive checks rather than default edit-loop gates.

The Rust property/corruption corpus runs explicitly with
`cargo test -p starclock-test-kit --features exhaustive --test exhaustive_suite`.
Adapter corruption, concurrency and TCP load checks run with
`cargo test -p starclock-test-kit --test adapter_suite -- --ignored`.

The two current Universe seeded matrices run explicitly with
`cargo test -p starclock-mode-universe seeded_run_tests::frozen_matrix -- --ignored`.

The machine-readable counterpart is `policy/current-state.json`.
