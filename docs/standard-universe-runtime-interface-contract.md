# Standard Universe Runtime Interface Contract

## Status and compatibility target

This document freezes the Goal 04 v1 implementation contract before runtime
code is introduced. Exact names and revision strings are normative. Internal
representation, module layout and constructors used only by validated lowering
remain private.

The existing one-battle Activity and battle replay fixtures remain supported as
legacy compatibility profiles. Goal 04 adds a graph-capable Activity API v2; it
does not reinterpret old bytes or silently relabel old hashes.

## Ownership

| Owner | Public responsibility | Must remain private |
|---|---|---|
| `starclock-activity` | Generic definitions, state, commands, decisions, events, faults, battle handoff and read-only views | journal, mutable stores, codec writer, RNG implementation and scratch buffers |
| `starclock-mode-universe` | Immutable Universe catalog, bundle validation/lowering, Standard entry and compilation | Sora rows, workbook fields, JSON staging types and content-ID dispatch |
| `starclock-data` | Existing combat/build catalog and exact bundle loading | generated rows and numeric backend |
| `starclock-replay` | Versioned Activity payloads, full-run trace and verification | decoder scratch and dependency-specific encodings |
| `starclock-ai` | Deterministic selection from offered Activity commands | mutable Activity access and hidden RNG |
| adapters | Select commands, host nested battles and render observations | raw state/resource mutation |

`starclock-combat` has no dependency on `starclock-mode-universe`. A mode crate
may depend on Activity, combat, build, data and rules, but cannot own another
mutable run aggregate or another authoritative RNG/replay implementation.

## Public Activity boundary

The v2 public shape is:

```rust
pub struct ActivityCommand {
    expected_state_hash: ActivityStateHash,
    decision: ActivityDecisionId,
    kind: ActivityCommandKind,
}

pub enum ActivityCommandKind {
    ChooseOption { option: ActivityOptionId },
    StartBattle { handoff: BattleHandoffId },
    SubmitBattleResult { result: Box<BattleResult> },
    SubmitExternalOutcome { outcome: ExternalOutcomeId },
    Abandon,
}

pub struct ActivityResolution {
    events: Box<[ActivityEvent]>,
    boundary: ActivityBoundary,
    state_hash: ActivityStateHash,
}

pub enum ActivityBoundary {
    Decision(ActivityDecisionPoint),
    Battle(BattleHandoff),
    Terminal(ActivityTerminal),
}

impl Activity {
    pub fn apply(
        &mut self,
        command: ActivityCommand,
    ) -> Result<ActivityResolution, ActivityCommandError>;
}
```

Fields use narrow getters. They are shown to freeze semantics, not to require
public struct fields. `Activity::decision()` and a newly created Activity return
the same owned/read-only decision shape used by `ActivityResolution`.
`Activity::apply` remains the sole authoritative run mutation boundary.

Exactly one boundary exists after an accepted command:

- `Decision` contains canonically ordered legal options and enough identity to
  build one exact command;
- `Battle` contains an immutable `BattleSpec`, isolated seed, result projection
  and identity; only a matching result may return to Activity;
- `Terminal` contains completed, failed, abandoned or faulted settlement.

Automatic programs settle inside one transaction until a boundary. Adapters do
not drive hidden intermediate operations.

### Commands, decisions and options

`ChooseOption` covers routes, encounter engagement, rewards, roster changes,
shops, enhancement, repair, replacement, services, occurrences, checkpoints
and retries. The option owns typed observation metadata; the command owns only
its stable identity. `StartBattle`, `SubmitBattleResult`,
`SubmitExternalOutcome` and `Abandon` work only when exactly offered.

Callers cannot submit currency deltas, arbitrary inventory values, RNG results,
enemy IDs, rule programs or graph destinations.

`ActivityDecisionPoint` contains its ID, current state hash, kind and a bounded
canonical collection of `ActivityOption`. `ActivityDecisionKind` is an AI and
presentation classification, not a second processor: `Choice`, `Route`,
`Encounter`, `Preparation`, `Reward`, `Shop`, `Service`, `Roster`,
`ExternalOutcome`, `BattleReady`, `Checkpoint`, and `Abandon`.

Options sort by `(priority, stable_option_id)`. Candidate discovery never uses
map iteration order. An opaque adapter token must reproduce the exact
decision/state/option identity.

## Events, errors and faults

Public `ActivityEvent` families are:

| Family | Examples |
|---|---|
| `Lifecycle` | activity/section/node/attempt entered or exited |
| `Decision` | options offered and option selected |
| `State` | scoped slot/resource/metric changed or reset |
| `Inventory` | modifier acquired, enhanced, charged, repaired, replaced or removed |
| `Graph` | edge traversed, interaction consumed, checkpoint captured/restored |
| `Battle` | handoff requested, result accepted and declared carry applied |
| `Rng` | purpose stream draw audit with revision/counter, never hidden state bytes |
| `Terminal` | completed, failed, abandoned or faulted |

Every event carries an `ActivityCause` with command sequence, definition,
node/attempt and optional source/option/battle identities. Events are ordered
facts produced by the committed transaction, not mutation callbacks.

Rejected input returns stable error kinds: `StaleStateHash`,
`DecisionNotOffered`, `CommandNotOffered`, `UnknownOption`, `UnknownOutcome`,
`HandoffMismatch`, `BattleResultMismatch`, `LimitExceeded`,
`ConfigurationMismatch`, or `InvalidCommandPayload`.

Rejection preserves canonical state, RNG counters, command sequence and pending
boundary. Internal evaluation/overflow/invariant failures are not command
errors: the transaction either rolls back and commits an explicit
`ActivityTerminal::Faulted(ActivityFault)` or, before mutation, returns a
construction error. No undocumented partial state is allowed.

## Mode and catalog boundary

The public flow is concrete rather than a plugin-owned state machine:

```rust
let catalogs = UniverseCatalog::load(universe_bundle, combat_catalog.clone())?;
let profile = StandardUniverseProfile::new(catalogs.clone());
let compiled = profile.compile(StandardUniverseEntry::new(
    world, difficulty, participants, ability_tree,
))?;
let mut activity = compiled.start(instance_id, master_seed)?;
```

The public domain types are `UniverseCatalog`, `UniverseCatalogIdentity`,
borrowed `WorldDefinition`, `DifficultyDefinition`, and `PathDefinition` views,
`StandardUniverseEntry`, `StandardUniverseProfile`, and `CompiledActivity`.
Catalogs and compiled definitions are immutable and normally shared through
`Arc`.

Loading accepts authoritative `.sora` bytes. Only private generated readers
interpret them. The loader rejects wrong schema, version, revision, digest,
references and incompatible combat/build identities. There is no runtime JSON,
TSV or `.xlsx` loader.

The composed configuration digest hashes domain tag
`starclock-activity-config-v1`, ordered component labels, each component
revision and each 32-byte digest. Standard Universe order is `combat`, `build`,
`universe`, `activity-profile`. Missing components encode an explicit absence
byte; arbitrary concatenation is forbidden.

## Revision freeze

| Identity | Legacy/current | Goal 04 graph Activity | Migration rule |
|---|---|---|---|
| numeric policy | `fixed-i64-6dp-v1` | unchanged | arithmetic/rounding change requires a revision |
| combat RNG | `chacha8-rand-0.10.2-intmap-v1` | unchanged | distribution helpers remain forbidden |
| replay envelope | `SCRP`, format `1` | format `1` | framing remains readable |
| replay payload schema | `1` | `2` | v2 adds full-run Activity records; v1 remains readable |
| Activity command payload | `1` | `2` | decoder dispatches by payload version |
| nested battle payload | `1` | `1` | result identity contract is retained |
| controller diagnostic payload | `1` | `1` | diagnostics remain non-authoritative |
| battle/legacy state hash | `sha256-v3` | nested battle remains v3 | old fixtures retain exact bytes |
| Activity state codec | `starclock-activity-state-v1` | `starclock-activity-state-v2` | v2 uses shared little-endian canonical encoding |
| Activity replay state hash | `sha256-v3` | `sha256-v4` | v4 never relabels v1 Activity bytes |
| Activity seed derivation | legacy battle-seed v1 | `starclock-activity-rng-v2` | purpose streams are independent |
| Activity API | one-battle v1 | `starclock-activity-api-v2` | pre-0.1 compile-time migration is intentional |
| Universe catalog | staging only | `standard-universe-v4.4-runtime-v1` | exact Goal 03 bundle is required |
| Standard Universe profile | absent | `standard-universe-main-world-v1` | other families use separate profiles/data |

### Activity state v2 canonical order

Activity state v2 streams sections in fixed order through the shared
little-endian encoder:

1. magic `SCAS`, codec version `2`, API/rules/numeric/RNG/hash revisions;
2. composed configuration and immutable definition identities;
3. instance, phase, command sequence and current boundary identity;
4. section/node/attempt identities, visits and consumed edges;
5. scoped slots, inventories, counters, modifiers, clocks, metrics/objectives;
6. participant/loadout locks and carry snapshots;
7. labeled RNG states/counters sorted by stream ID;
8. pending ordered options or immutable battle handoff identity;
9. checkpoints and completed nested result digests;
10. terminal/fault settlement when present.

Counts are bounded `u32`, numeric IDs are fixed-width, fixed scalar values use
raw signed `i64`, and optionals use one presence byte. Semantic collections
preserve authored order; other collections sort by stable key. No `usize`,
Serde, Rust `Hash`, capacity, pointers, cache, presentation or wall clock enters
the stream.

### Activity RNG v2

The master seed derives independent `graph`, `encounter`, `reward`, `shop`,
`occurrence`, `spawn`, `external-outcome-test` and per-`battle` streams. The
derivation hashes its domain tag/revision, master seed, composed config digest,
profile/definition/instance, applicable scope/battle sequence and bounded ASCII
purpose label. Fixed integers use Activity v2 canonical little-endian encoding;
the first 32 digest bytes seed ChaCha8 directly.

Candidate ordering, range/rejection mapping, weighted selection and
no-candidate draw consumption are project-owned. A missing candidate consumes
no draw. A substream never clones a live stream. Draws in one purpose cannot
shift another purpose or combat stream.

## Compatibility and acceptance

- Legacy replay/state paths become read-only compatibility behavior; new graph
  Activities never emit v1 Activity bytes.
- Unsupported revision combinations reject before state construction. Migration
  re-executes commands against an explicitly chosen new configuration; byte
  relabeling is forbidden.
- Cross-revision byte relabeling is forbidden under every migration path.
- Public debug JSON has its own schema revision and is never authoritative.
- Mode compilation produces generic Activity definitions/programs and normal
  battle contributions only.
- No public signature names a generated Sora module, row, workbook column,
  `fixnum`, RNG backend or unbounded map.
- Invalid command/result tests preserve bytes and every RNG counter.
- v1 goldens remain exact while v2 gets independent codec/RNG goldens.
- CLI, AI, agent and MCP consume the same ordered decisions/options.
