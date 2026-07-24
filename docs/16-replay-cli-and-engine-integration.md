# Replay, CLI, and Engine Integration

This document defines the replay contract, headless command-line surface,
deterministic baseline controllers, and integration boundary for Bevy or another
engine. The `starclock-cli` package owns the versioned headless validation,
coverage, synthetic Standard battle and replay-verification surface implemented
by Goal 01 batch `G01-P6-B5`. The subsequent frozen manifest import exposes all
six production Standard scenarios through the same `battle run` and `replay
verify` contracts.

## Replay identity

A replay header contains:

```text
magic = "SCRP"
replay_format_version
schema_version
game_version
rules_revision
data_revision
configuration_root_sha256
consumed_component_digests[]
artifact_bundle_sha256
numeric_policy_revision
rng_algorithm_revision
state_hash_revision
master_seed
entry_kind
entry_definition_id
entry_spec_sha256
```

`entry_kind` distinguishes low-level battle and activity replays. An activity header also identifies its mode profile and definition digest; universe/challenge/event names are metadata rather than separate replay protocols. A battle entry/spec digest covers ordered generic `CombatantSpecDigest` values. Build-aware mode/activity records additionally bind the corresponding `CombatantBuildDigest` values and BuildCatalog revision, so Trace, Eidolon, Light Cone, relic, trial, and synthetic-policy differences cannot share a build-aware replay identity. A low-level synthetic battle replay requires no build vocabulary. The header is followed by accepted commands in sequence, nested battle records, optional controller diagnostics, and expected state hashes after every command. Rejected commands are diagnostic input attempts, not part of the authoritative accepted stream.

`configuration_root_sha256` is calculated from an ordered typed set containing
the consumed combat, build, Activity-core, selected-mode, content-partition,
and composed-handler/executor registry digests. `artifact_bundle_sha256`
identifies the physical release artifact for provenance. Adding an unrelated
mode or localization-only partition may change the artifact digest but does
not invalidate a replay whose consumed component set is unchanged. Version 1
replays with only `config_bundle_sha256` retain their legacy all-or-nothing
interpretation; conversion creates a new replay header rather than changing
old bytes.

## Canonical codec

Replay and state hashing use a project-owned versioned codec:

- fixed header/field order;
- unsigned/signed fixed-width integers in little-endian order;
- one-byte enum discriminants defined by the format revision;
- length-prefixed UTF-8 strings and byte arrays;
- raw fixed-point scaled integers;
- collections sorted by stable domain key unless semantic order is already explicit;
- optional values encoded with an explicit presence byte;
- no pointer values, `usize`, struct memory, map iteration order, wall-clock timestamps, caches, or presentation fields.

Normal `serde`, JSON, Rust's `Hash`, and dependency-defined binary encodings are not canonical. They may support debug output only. State hashing applies SHA-256 to the exact canonical state byte stream and records `state_hash_revision = "sha256-v2"`.

The codec exposes one canonical sink contract. State hashing streams into a
SHA-256 sink; golden/debug tooling may collect the same bytes. Replay
verification must not allocate a full canonical-state byte vector merely to
hash it.

### Version 1 envelope

`G01-P2-B4` freezes `replay_format_version = 1`, payload
`schema_version = 1` and `state_hash_revision = "sha256-v2"`. The header starts
with `SCRP`, both little-endian versions and unknown-record policy `Reject`.
Compatibility text identities are printable ASCII bounded to 128 bytes; general
domain strings encoded inside later payloads remain length-prefixed UTF-8.

The identity block contains game/rules/data revisions, configuration digest,
numeric/RNG/state-hash revisions, controller revision/digest and master seed.
The entry block is either a low-level battle definition/spec digest or an
activity profile/definition/spec identity. Activity entries may additionally
bind one build-catalog revision/digest and participant-ordered build digests.
The header ends with a bounded record count.

Each record is exactly `(kind: u8, sequence: u64, payload_length: u32,
payload)`. Sequence starts at zero with no gaps. Version 1 reserves accepted
battle/activity commands, expected battle/activity state hashes, nested-battle
start/end and controller diagnostics. Every unknown kind is a hard failure;
skipping unknown bytes is not a compatibility mechanism. The library limit is
1,000,000 records and 16 MiB per record. Decoding validates the complete record
framing, lengths, sequence and trailing-byte boundary before allocating the
borrowed record table. Later batches define domain payload bytes inside these
reserved kinds without changing the envelope.

## Replay verification

Verification resolves the exact consumed component digest set and composed
registry digest, checks every policy revision, rebuilds the entry state,
applies commands, and compares hashes after each accepted command. When a
legacy replay provides only a bundle digest, verification loads that exact
archived bundle. On divergence it reports the first command/event/state
boundary, expected/actual digest, and stable structural path if a diagnostic
decoder is available.

Unknown revisions, mismatched bundles, invalid commands, missing archived configuration, and trailing/truncated records are hard failures. A migration creates a new replay with a new header; it never pretends the old canonical bytes were produced under new rules.

Verification consumes commands and expected hashes in sequence. It may stream
or discard reproduced events after comparing the current boundary; it does not
need to retain every intermediate `BattleState`, `Resolution`, or event from the
entire replay. Diagnostic retention is bounded and enabled only as required to
report the first divergence.

## Headless and server verification profiles

The library supports two service-side execution profiles without adding network
or account policy to the combat core:

1. **Incremental authoritative session.** The service retains one `Battle` at
   its current decision boundary, accepts only an offered command, applies it
   once, and stores the new state/hash. Work is proportional to the new command.
2. **One-shot audit replay.** The service rebuilds a battle from the bound
   catalog/spec/seed and executes the accepted command stream once, comparing
   every expected hash. Work is proportional to replay length.

A stateless endpoint must not rebuild prefixes `1`, `1..2`, `1..3`, and so on
for every live command; that turns one N-command battle into O(N²) verification.
If a deployment cannot retain live battle state, checkpoint design belongs to a
separate authenticated save/session contract and must bind the same catalog,
spec, policy revisions, sequence and state hash. Goal 01 does not define that
network protocol.

A verifier process should cache one immutable validated catalog per exact
configuration digest and share it through `Arc`. Each replay/battle job owns its
state, RNG, scratch transaction and diagnostic buffer and runs on a bounded
worker pool. No cross-job mutable state or scheduler order may affect simulation.

Before allocating replay-declared collections, the service adapter validates
magic/version, fixed header lengths, configuration availability, command count,
record lengths and rules-revision budgets. The combat/replay libraries expose
typed limits and errors; HTTP/RPC authentication, rate limiting, authority,
anti-cheat policy and persistence remain outside their scope.

## Verification performance workloads

Versioned benchmarks use release builds and fixed catalog/spec/seed/command
streams. At minimum measure:

- ordinary and trigger-heavy incremental `Battle::apply`;
- invalid/stale command rejection before transaction preparation;
- streaming canonical hash for small, medium and large battle states;
- one-shot 100-command and 500-command replay verification;
- many independent replay jobs sharing one catalog on a bounded worker pool;
- peak transient bytes/job, allocations/command, semantic state-copy bytes,
  journal/event/operation counts and commands/second/core.

Strict latency/throughput/allocation budgets belong to a documented stable
runner. Shared cross-platform CI uses deterministic result checks and only a
generous order-of-magnitude regression ceiling.

Goal 01 Phase 3 implements the first revision of this contract as
`g01-phase3-benchmark-v1`; its fixture boundary, broad CI ceilings, designated
strict runner and five-sample baseline are recorded in
[Phase 3 performance benchmark baseline](performance-benchmark-baseline.md).
The Phase 3 trigger-heavy row is explicitly an eight-operation/event proxy
until the trigger kernel replaces or versions it in Phase 4.

## Headless CLI surface

```text
starclock config validate [--bundle PATH] [--json]
starclock catalog coverage [--goal core-combat-v1] [--category NAME] [--json]
starclock battle run --scenario ID --seed U64 [--controller baseline|replay] [--replay-out PATH] [--json]
starclock activity run --profile ID --activity ID --seed U64 --controller baseline [--json]
starclock replay verify FILE [--json]
starclock universe run --mode ID --seed U64 --controller baseline [--json]
starclock challenge run --mode ID --stage ID --seed U64 --controller baseline [--json]
```

`universe run` and `challenge run` are convenience front ends that resolve a profile/activity ID and call the same activity runtime. They do not own separate save, RNG, result, or replay behavior.

Exit code `0` means the requested validation/simulation completed successfully. Distinct nonzero codes represent usage, configuration, replay incompatibility/divergence, invalid scenario, simulation fault, or internal tool error. JSON mode writes one versioned result object to stdout and diagnostics to stderr; human mode may format the same domain result for reading.

The Goal 01 v1 exit classes are usage `2`, configuration/bundle `3`, replay
incompatibility or divergence `4`, invalid scenario `5`, simulation fault `6`
and I/O/internal tool failure `7`. JSON objects identify
`schema_revision = "starclock-cli-v1"`.

`config validate` loads the embedded production `config.sora` by default or an
explicit `--bundle`; both paths use the generated Sora reader and the complete
production catalog validator. `catalog coverage` derives the six frozen Goal
01 denominator categories directly from those validated Sora identities. It
does not read diagnostic JSON or promote disabled bootstrap rows.

B5 keeps `synthetic-standard-v1` as the only runnable battle until B6 imports
the production Standard manifests. Its `baseline` option executes the versioned
baseline scorer and binds a digest of the exact synthetic authored hints into
the replay header. `replay` without an accepted stream is a usage error directing
the caller to `replay verify FILE`; the CLI never fabricates replay commands.
Replay verification currently resolves the low-level synthetic entry. The B4
library already verifies Activity payloads; B6 owns resolving production
Activity/profile entries from imported catalogs.

CLI flags cannot alter authoritative rounding, RNG mapping, budgets, or event order without selecting a different versioned rules revision. Scenario, activity, and mode IDs resolve through validated immutable catalogs rather than filesystem naming conventions.

## Library boundary

The stable external surface consists of immutable catalog handles, `Battle`, `Activity`, read-only views, commands, resolutions, decision points, events, IDs, `BattleSpec`/`BattleResult`, and typed errors. Run/challenge/event terminology belongs to profiles and optional convenience APIs, not additional generic state types. Generated Sora records, fixed-point backend types, journals, queues, and native-handler implementations remain private.

Avoid broad re-exports. Consumers import from explicit modules, with only a small documented facade if later ergonomics justify it. The core has no Bevy, async runtime, filesystem, terminal, or platform-time dependency.

## Baseline battle controller

The baseline controller receives only a read-only view and canonically ordered legal commands. It never creates commands independently.

Its deterministic policy is:

1. reject commands that lose immediately when a legal survival action is identified by authored AI hints;
2. consider ready Ultimates/interrupts by authored priority;
3. score legal Skills using healing/survival need, break opportunity, target value, resource reserve, and authored synergy tags;
4. otherwise choose a Basic or mandatory action;
5. score targets through authored hints and current visible state;
6. break equal scores by command/target stable IDs.

All scores use integers/fixed-point values. The controller is a reproducible smoke-test policy, not an optimal search agent. Enemy AI uses its authored graph rather than this controller.

## Baseline activity controller

The activity controller consumes only canonically ordered legal commands. It scores route, roster, shop, reward, modifier, and external-outcome options using profile-provided hints, participant tags, build synergy, guaranteed resource delta, risk, and progress. It uses stable option-ID tie-breaking and never fabricates a command or outcome.

Controller selection and optional score components are replay diagnostics. Only the selected accepted command affects authoritative state.

For a pending Battle node, it starts the offered `BattleSpec`, runs it through the battle controller, and submits the verified result. Universe/challenge profiles may supply scoring hints; they do not replace the controller protocol or modify scores, clocks, and results.

## Bevy and other engines

An adapter should:

- own one `Activity` plus its current `Battle`, or communicate with one dedicated simulation worker;
- translate input/AI intent into an offered command;
- publish resolution events for animation, audio, HUD, camera, and logs;
- map stable `UnitId` values to engine entities in adapter components;
- render `ActivityView`/`BattleView` and never infer flow, score, HP, effects, or action order from presentation state;
- keep presentation acknowledgements and pacing outside the core;
- preserve command order if simulation runs on another thread.

Frame rate, delta time, ECS iteration, entity IDs, task scheduling, animation cancellation, and UI localization cannot affect simulation. An adapter may pause before submitting the next command; it cannot pause halfway through an accepted command.

## Save and network boundary

A replay is an audit input, not an account save. Activity checkpoint/save files may reuse canonical primitives but require their own schema and migration policy. Networked play, authority reconciliation, anti-cheat, and live-service account state are outside the current target.

## Acceptance tests

- codec golden vectors cover every primitive, optional value, enum, collection, and fixed-point sign/boundary;
- replay truncation, wrong digest, wrong revision, mutated command, and mutated expected hash are rejected;
- battle and activity replays compare hashes after every command and nested battle submission;
- incremental authoritative execution and one-shot replay produce identical
  events and hashes for the same accepted command stream;
- replay verification streams intermediate work with bounded diagnostic
  retention and rejects oversized headers/records before unbounded allocation;
- baseline controllers select identically under reordered internal collections;
- human and JSON CLI modes represent the same result and use documented exit classes;
- a minimal adapter test proves presentation timing and entity iteration do not affect hashes;
- Windows x86-64, Linux x86-64, and macOS ARM64 execute identical golden fixtures before cross-platform compatibility is claimed.
