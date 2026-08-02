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

Complete dynamic Universe replay reconstruction runs with
`cargo test -p starclock-test-kit --test universe_suite dynamic_battle_assembly::dynamic_replay_reconstructs_each_snapshot_and_reports_first_divergence -- --exact --ignored`.

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
- replay header, component-set and record-payload revision prefixes; the current
  codec keeps only framing magic, semantic type discriminants and bounds;
- `current` forwarding modules and versioned Rust/example filenames;
- Agent API schema selection and `schema_revision` request/response fields;
- CLI schema/Goal identifiers and runtime/release evidence snapshots;
- benchmark and seed-matrix schema/workload/executor revision fields;
- textual component, controller and build revisions duplicated by exact digests;
- Activity codec/RNG/scope/handler revisions duplicated by current structure and digests;
- Combat catalog/rules/numeric/RNG/state-codec revisions and the duplicate
  `BattleSpecDigest` wrapper;
- Combat input/state codec revision sentinels; `SCBI`/`SCBS` framing magic and
  semantic field discriminants remain;
- generated `ConfigManifest` data/rules/numeric/RNG/state/replay revision
  labels and the old Goal coverage digest; the manifest now carries only
  gameplay `game_version`, source `snapshot_date` and pinned
  `sora_cli_version`;
- the production configuration golden registry and its `--bless` path;
  verification now rebuilds directly from the current schema/workbooks and
  compares current generated artifacts;
- the deleted Goal-manifest verifier dependency from production bootstrap;
- four obsolete partition authoring scripts (3,215 lines) and their
  `G01-P7-*` partition ledger; the remaining current ConfigManifest author is
  part of a focused current-workbook adapter;
- Goal schema/id/generated-date metadata from the retained core-combat gameplay
  selection manifests;
- the unused `handler_version` Native Handler column across schema, workbook,
  generated reader and bundle;
- nine Standard Universe path-runtime revision constants, their duplicate
  digest inputs and byte-for-byte digest snapshots;
- the Standard Universe entry revision and its hard-coded core catalog digest;
  composition now requires the current combat and build catalogs to agree;
- the remaining Standard Universe path, blessing, ability, curio, occurrence,
  service, encounter and run revision constants and duplicate digest inputs;
- Standard Universe battle assembly, contribution, materialization, snapshot
  and event-commitment revisions and their byte-for-byte digest snapshots;
- Gold and Gears runtime-coverage, baseline-fixture and seeded-run revisions;
- Swarm Disaster runtime-coverage digest, baseline-fixture revision and
  seeded-run revision;
- twelve Swarm Disaster mechanic-rule runtime revisions and their exact digest
  snapshots; current behavior remains covered by contract and execution tests;
- seven Swarm Disaster entry-policy revision constants and the Communing Trail
  digest snapshot;
- Swarm Disaster content, occurrence, Path, semantic-fixture, service and
  adventure runtime revisions and their fixed digest snapshots;
- Swarm Disaster encounter, enemy-composition, battle-materialization and
  battle-snapshot revisions and their byte-for-byte digest snapshots;
- Swarm Disaster baseline entry/controller and performance-matrix hash-domain
  versions, plus fixed baseline controller digest snapshots;
- seven Gold and Gears mechanic execution/profile revisions and their direct
  fixed digest snapshots;
- Gold and Gears `VersionedProjectPolicy` accuracy naming; current inferred
  rules are now classified simply as `ProjectPolicy`;
- empty deferred relic/planar build fields and their placeholder document.

Mode and generated content modules still contain textual `*_REVISION` domain
labels used inside digest construction. They are not compatibility branches,
but they are redundant current-tree identity and remain cleanup debt. Replace
them with the underlying canonical content/configuration digests. Retain only
semantic type/variant discriminants; remove numeric sentinels whose only
meaning is a codec or payload revision.

The core-combat gameplay reference manifests remain. Their per-row
`implementation_state` labels and `standard-v1` stable IDs still mix workflow
state/version naming into gameplay selections and remain cleanup debt; the
actual character, Light Cone, enemy, encounter and scenario references must be
preserved while that metadata is removed.
