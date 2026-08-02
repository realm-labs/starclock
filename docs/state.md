# Repository state

Starclock maintains only the current source, data, behavior and test outputs.
Git history is the only historical record.

## Input lifetime

- Old replay files, serialized state, API envelopes and generated configuration
  are not supported after the current implementation changes.
- The repository has no migration, legacy-decoder or historical-evidence
  requirement.
- Canonical hashes and current goldens may change intentionally and are replaced
  atomically with the implementation that produces them.
- Dependency and toolchain pins remain build inputs.
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

The default test profile minimizes compilation and linking for workspace
crates. Third-party dependencies and the combat hot loop retain light
optimization. Complete gameplay runs are excluded from default adapter tests
because they measure end-to-end simulation rather than local API behavior.

The Rust property/corruption corpus runs explicitly with
`cargo test -p starclock-test-kit --features exhaustive --test exhaustive_suite`.
Adapter corruption, concurrency and TCP load checks run with
`cargo test -p starclock-test-kit --test adapter_suite -- --ignored`.

Complete Agent API gameplay/replay checks run with
`cargo test -p starclock-agent-api --lib public_offers_complete_real_battles_and_export_fresh_replay -- --ignored`.
Complete CLI gameplay/replay and text/JSON parity checks run with
`cargo test -p starclock-cli --test universe_cli -- --ignored`.

The two current Universe seeded matrices run explicitly with
`cargo test -p starclock-mode-universe seeded_run_tests::frozen_matrix -- --ignored`.

The machine-readable counterpart is `policy/state.json`.

## Identity audit

Retained because they describe external facts or build inputs:

- Cargo package/dependency versions and the Node/Sora/MCP protocol pins;
- game-version labels in gameplay reference packs;
- source repository commits, access dates and evidence digests.

Removed from current runtime surfaces:

- replay v1/v2/v3 modules, alternate decoders and payload-version selectors;
- `current` forwarding modules and versioned Rust/example filenames;
- Agent API schema selection and `schema_revision` request/response fields;
- CLI schema/Goal identifiers and runtime/release evidence snapshots;
- benchmark and seed-matrix schema/workload/executor revision fields;
- textual component, controller and build revisions duplicated by exact digests;
- Activity codec/RNG/scope/handler revisions duplicated by current structure and digests;
- empty deferred relic/planar build fields and their placeholder document.

Combat and generated content modules still contain textual
`*_REVISION` domain labels used
inside digest construction. They are not compatibility branches, but they are
redundant current-tree identity and remain cleanup debt. Replace them with the
underlying canonical content/configuration digests; keep fixed binary layout
sentinels as `*_TAG` values.

The current Sora native-handler table still authors `handler_version`, but the
runtime registry no longer consumes it. Removing that generated column requires
the workbook, schema, generated reader and bundle to change together.
