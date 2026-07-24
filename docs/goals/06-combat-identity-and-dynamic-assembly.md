# Goal 06 — Combat Identity and Dynamic Per-Battle Assembly

## Objective

Harden the generic combat boundary and replace the Standard Universe
entry-time battle materialization with deterministic per-battle assembly from
the current authoritative Activity snapshot.

This goal makes a newly acquired, upgraded, removed, disabled or state-changed
Path/Blessing/Resonance/Formation/Curio/Ability Tree contribution eligible to
change the next `BattleSpec`. It does not implement the remaining Standard
Universe mechanics; Goal 07 owns that content work.

## Frozen prerequisites

- Goals 01–05 are immutable release snapshots in
  `policy/release-snapshots.json`.
- Goal 05 integration dispositions are the starting-debt oracle:
  2,201 content records, 786 rule bindings and 78 semantic fixtures.
- Component-addressed replay v2 remains historically verifiable.
- `Battle::create/apply`, checked fixed-point arithmetic, canonical ordering,
  command rejection without mutation and immutable catalogs remain stable
  lower-level guarantees.
- Excel remains the authoring surface. Any workbook change uses Python
  `openpyxl` and the pinned Sora 0.3.0 pipeline.

## Terminal outcome

- `starclock-combat` computes a canonical digest over every battle-visible
  input instead of trusting a caller-supplied `BattleSpecDigest`;
- an opaque upstream `AssemblyDigest` separately identifies build, Activity,
  mode and component provenance;
- replay v3 binds combat input, assembly and consumed components, reports the
  first divergence and retains historical v2 verification;
- the immutable combat catalog contains all definitions available to the
  selected released profile and is not rebuilt for every battle;
- every Standard Universe battle request is assembled from the current
  Activity inventory, lifecycle state, progression and participant carry;
- the same per-battle assembler is used by CLI, baseline runs, Agent sessions,
  MCP and replay reconstruction;
- invalid assembly, executor failure and result mismatch preserve Activity
  state, RNG counters and pending-task identity;
- cache keys are canonical, bounded and non-authoritative;
- representative acquisition/upgrade/removal fixtures change the following
  battle input/event/hash exactly when their contribution is executable;
- focused daily verification remains within 1–3 minutes on the stable runner.

## Non-goals

- implementing all 783 retained Standard Universe rule bindings;
- replacing all enemy proxies or approximate numeric policies;
- Swarm Disaster, Gold and Gears, Unknowable Domain or Divergent Universe;
- multi-pending tasks, ForkJoin or alternate child simulators;
- extracting a general nested-battle runner before a second gameplay family
  needs it;
- Bevy, UI, presentation timing or networking changes;
- mutating historical Goal 01–05 evidence or replay files.

## Architecture contract

### Identity separation

The combat-visible and outer-assembly identities are different domains:

```rust
pub struct BattleSpec {
    combat_input_digest: CombatInputDigest,
    assembly_digest: AssemblyDigest,
    // encounter, participants, resources, policies...
}
```

`CombatInputDigest` is computed by `starclock-combat` from its own versioned
canonical codec. It covers every field that can affect battle construction or
resolution. Callers cannot inject or override it.

`AssemblyDigest` is opaque to combat. It commits to the selected build,
Activity/mode snapshot, consumed component set and approved approximation
policies. Equal combat inputs may have different assembly provenance; unequal
combat inputs may never have the same `CombatInputDigest`.

### Catalog and assembly separation

```text
Excel/Sora + released components
            |
            v
Immutable validated CombatCatalog        built once
            |
Current Activity snapshot + encounter + carry
            |
            v
PerBattleAssemblyCompiler
            |
            v
BattleSpec + AssemblyDigest + result contract
```

Catalog composition adds definitions. Per-battle assembly only selects
validated definitions and supplies runtime bindings/values. It may use a
bounded cache keyed by exact component, roster, encounter and contribution
digests. Cache loss or eviction cannot change hashes or behavior.

### Transaction boundary

Preparing a pending battle must use one immutable snapshot of the Activity
state. Assembly succeeds before the no-draw battle-start marker commits.
Failure returns a typed error and preserves canonical Activity bytes, RNG
draws and replay records. A running `Battle` never reads live Activity state.

### Compatibility

- replay v2 decode/verification remains available for its released component
  set;
- new production recordings use replay v3 only;
- historical releases are verified from their registered completion trees;
- changing digest or event codecs requires explicit revision constants and
  golden fixtures.

## Execution and commit rules

- Execute the earliest unblocked batch and keep only one batch `InProgress`.
- Update the Goal 06 status ledger in every batch commit.
- Commit subjects use
  `<type>(<scope>): <batch-id> <imperative summary>`.
- Each batch includes focused tests and the quick repository gate.
- Run the full gate only at phase and release checkpoints.
- Preserve unrelated work and never regenerate frozen release evidence.
- Split touched handwritten files before they exceed 1,200 physical lines.
- Avoid new convenience `pub use` exports.

## Delivery phases

### Phase 0 — Contract and observable debt

| Batch | Deliverable |
|---|---|
| `G06-P0-B1` | Freeze this plan, ledger and launch prompt; bind the Goal 05 completion snapshot and define identity/assembly/replay-v3 terminology. |
| `G06-P0-B2` | Add probes proving caller-supplied digest aliasing, entry-time-only materialization and production-interface parity debt; freeze representative acquire/upgrade/remove scenarios. |
| `G06-P0-B3` | Freeze replay compatibility, performance/cache workloads, dependency/license baseline and the Goal 06 release-contract scaffold. |

### Phase 1 — Combat identity and replay v3

| Batch | Deliverable |
|---|---|
| `G06-P1-B1` | Add the combat-owned canonical input codec and computed `CombatInputDigest`; separate opaque `AssemblyDigest` in `BattleSpec` and battle identity. |
| `G06-P1-B2` | Migrate Activity handoff/result identity, participant preparation and settlement validation to the two-digest contract with byte-identical rejection tests. |
| `G06-P1-B3` | Add component-addressed replay v3 encoding/decoding and first-divergence verification while retaining released replay-v2 verification. |
| `G06-P1-B4` | Migrate build/data/standard-battle construction and event attribution; resolve the unused `activity_source` field under the new codec revision and split touched near-limit core files. |

### Phase 2 — Dynamic per-battle assembly

| Batch | Deliverable |
|---|---|
| `G06-P2-B1` | Separate immutable Standard Universe catalog composition from selected-contribution assembly; define canonical `BattleAssemblyKey` and bounded cache policy. |
| `G06-P2-B2` | Project the current Activity Path, Blessing levels, Resonance/Formations, Curio lifecycle states, Ability Tree values and carry into one immutable contribution snapshot. |
| `G06-P2-B3` | Assemble the pending encounter after preparation from that snapshot and emit `BattleSpec`, assembly identity and result contract atomically. |
| `G06-P2-B4` | Add cache hit/eviction, stale snapshot, invalid definition, budget failure and retry tests proving canonical state/RNG preservation. |
| `G06-P2-B5` | Prove acquire, upgrade, disable/remove and cross-battle carry changes through representative Blessing, Curio, Resonance and Ability Tree battle fixtures. |

### Phase 3 — Production surfaces and replay

| Batch | Deliverable |
|---|---|
| `G06-P3-B1` | Migrate CLI and baseline universe runs to per-battle assembly and replay v3; remove access to the frozen entry-time materialization path. |
| `G06-P3-B2` | Migrate Agent sessions and MCP tools through the same assembler without changing authorization, quotas, idempotency or opaque-action authority. |
| `G06-P3-B3` | Reconstruct every battle snapshot during replay verification; add corruption, concurrent-session and cross-surface command/event/hash parity tests. |

### Phase 4 — Hardening and release

| Batch | Deliverable |
|---|---|
| `G06-P4-B1` | Measure assembly/cache/state-copy/hash costs, enforce the focused 1–3 minute budget and finish responsibility-based splits of touched near-limit files. |
| `G06-P4-B2` | Run the 33-entry matrix, property/corruption tests and native CI profiles; publish dynamic-assembly and replay-v3 evidence. |
| `G06-P4-B3` | Freeze documentation, coverage and release evidence; pass full clean-worktree verification, commit the batch and register the immutable Goal 06 snapshot. |

## Acceptance

- two combat-visible inputs differing in any canonical field have different
  `CombatInputDigest` values;
- changing only upstream provenance changes `AssemblyDigest` without changing
  `CombatInputDigest`;
- an untrusted caller cannot provide either a forged combat digest or a
  presealed battle result;
- replay v3 detects component, assembly, combat-input, command, event and state
  divergence at the first differing boundary;
- replay v2 remains verifiable only under its released compatibility path;
- acquiring an executable Blessing or Curio changes the next battle assembly
  and real battle hash, not the already-running battle;
- removing/disabling the contribution reverses the next assembly without
  stale cache reuse;
- all production surfaces assemble the same battle for equal components,
  Activity state, encounter and seed;
- catalog composition is not repeated per battle;
- assembly/replay failures preserve authoritative Activity state and RNG;
- no Universe type or branch is added to `starclock-combat`;
- final clean full repository verification passes.

## Terminal checklist

- [ ] All 18 batches are committed with ledger evidence.
- [ ] Combat input and assembly identities are separated and canonical.
- [ ] Replay v3 records and verifies dynamic assemblies; v2 remains historical.
- [ ] Every production Standard Universe battle uses the current Activity snapshot.
- [ ] Representative acquire/upgrade/remove mechanics affect the next real battle.
- [ ] Cache, rollback, concurrency and cross-surface parity gates pass.
- [ ] Focused and full clean-worktree release verification pass.

