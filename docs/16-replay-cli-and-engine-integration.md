# Replay, CLI, and Engine Integration

This document defines the planned replay contract, headless command-line surface, deterministic baseline controllers, and integration boundary for Bevy or another engine. It is an implementation specification; no CLI is claimed to exist during the documentation-only milestone.

## Replay identity

A replay header contains:

```text
magic = "SCRP"
replay_format_version
schema_version
game_version
rules_revision
data_revision
config_bundle_sha256
numeric_policy_revision
rng_algorithm_revision
state_hash_revision
master_seed
entry_kind
entry_definition_id
entry_spec_sha256
```

`entry_kind` distinguishes low-level battle and activity replays. An activity header also identifies its mode profile and definition digest; universe/challenge/event names are metadata rather than separate replay protocols. A battle entry/spec digest covers ordered generic `CombatantSpecDigest` values. Build-aware mode/activity records additionally bind the corresponding `CombatantBuildDigest` values and BuildCatalog revision, so Trace, Eidolon, Light Cone, relic, trial, and synthetic-policy differences cannot share a build-aware replay identity. A low-level synthetic battle replay requires no build vocabulary. The header is followed by accepted commands in sequence, nested battle records, optional controller diagnostics, and expected state hashes after every command. Rejected commands are diagnostic input attempts, not part of the authoritative accepted stream.

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

Normal `serde`, JSON, Rust's `Hash`, and dependency-defined binary encodings are not canonical. They may support debug output only. State hashing applies SHA-256 to the exact canonical state byte stream and records `state_hash_revision = "sha256-v1"`.

## Replay verification

Verification loads the exact config bundle digest, checks every policy revision, rebuilds the entry state, applies commands, and compares hashes after each accepted command. On divergence it reports the first command/event/state boundary, expected/actual digest, and stable structural path if a diagnostic decoder is available.

Unknown revisions, mismatched bundles, invalid commands, missing archived configuration, and trailing/truncated records are hard failures. A migration creates a new replay with a new header; it never pretends the old canonical bytes were produced under new rules.

## Planned CLI surface

```text
starclock config validate [--bundle PATH] [--json]
starclock catalog coverage [--category NAME] [--json]
starclock battle run --scenario ID --seed U64 [--controller baseline|replay] [--json]
starclock activity run --profile ID --activity ID --seed U64 --controller baseline [--json]
starclock replay verify FILE [--json]
starclock universe run --mode ID --seed U64 --controller baseline [--json]
starclock challenge run --mode ID --stage ID --seed U64 --controller baseline [--json]
```

`universe run` and `challenge run` are convenience front ends that resolve a profile/activity ID and call the same activity runtime. They do not own separate save, RNG, result, or replay behavior.

Exit code `0` means the requested validation/simulation completed successfully. Distinct nonzero codes represent usage, configuration, replay incompatibility/divergence, invalid scenario, simulation fault, or internal tool error. JSON mode writes one versioned result object to stdout and diagnostics to stderr; human mode may format the same domain result for reading.

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
- baseline controllers select identically under reordered internal collections;
- human and JSON CLI modes represent the same result and use documented exit classes;
- a minimal adapter test proves presentation timing and entity iteration do not affect hashes;
- Windows x86-64, Linux x86-64, and macOS ARM64 execute identical golden fixtures before cross-platform compatibility is claimed.
