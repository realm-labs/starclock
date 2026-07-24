# Mode Extension and Evolution Contract

This document defines which Starclock contracts are stable, which extension
points may evolve, and how a new gameplay family moves from a mechanic probe to
a released profile. It prevents the current Standard/Universe implementation
from becoming an accidental permanent limit.

## Stable lower-level guarantees

New gameplay must continue to preserve:

- command-based mutation and rejection without state change;
- checked fixed-point authoritative arithmetic;
- project-owned deterministic RNG, canonical ordering, codec, hashes, and
  replay verification;
- immutable validated definitions and digest-bound runtime inputs;
- no mode-ID or content-ID branches in shared resolvers;
- typed operations and bounded static native handlers rather than arbitrary
  workbook code;
- separation of combat, build compilation, activity orchestration, adapters,
  and presentation.

These guarantees are compatibility boundaries. The current set of Activity
node kinds, four authoring scope aliases, one pending battle, one monolithic
configuration digest, and one global handler table are not.

## Ordered child-task boundary

`BattleSpec`/`BattleResult` remains the ordinary combat protocol. Activity
orchestration evolves around a generic, closed child-task envelope:

```rust
pub enum ActivityTaskSpec {
    Combat(BattleSpec),
    External(ExternalTaskSpec),
    ChildActivity(ChildActivitySpec),
    Registered(RegisteredTaskSpec),
}

pub struct PendingActivityTask {
    pub task_id: ActivityTaskId,
    pub branch_id: ActivityBranchId,
    pub executor: ActivityExecutorId,
    pub spec_digest: ActivityTaskSpecDigest,
    pub payload: ActivityTaskSpec,
}
```

An Activity resolution may expose an ordered bounded collection of pending
tasks. The current single-battle API is the cardinality-one compatibility
profile. `ForkJoin`, multi-team stages, tournaments, and isolated batch
simulation require the collection form before they may claim implementation.

Registered tasks are not callbacks. Their executor ID, input/result schema,
limits, deterministic policy, registry revision, and canonical payload codec
must be known before Activity construction. Task results are verified against
the offered task identity and declared projection before mutation.

An alternate combat model that cannot truthfully compile to `BattleSpec` uses
a separately versioned registered executor. It does not add mode branches to
`starclock-combat` or weaken Activity transaction, RNG, budget, and replay
rules.

## Physical lifetimes and logical scopes

The physical engine lifetimes remain:

```text
Activity -> Section -> Node -> Attempt
```

They define transaction ownership, reset boundaries, battle handoff, and
canonical storage. Modes may additionally declare a bounded logical scope tree:

```rust
pub struct LogicalScopeDefinition {
    pub class: LogicalScopeClassId,
    pub parent_class: Option<LogicalScopeClassId>,
    pub physical_owner: PhysicalActivityScope,
    pub max_depth: u8,
    pub max_instances: u32,
}
```

Names such as Run, Plane, Stage, Side, Round, Bracket, Match, Domain, or Room
are logical classes rather than hard-coded core enum variants. Every logical
instance has a stable ID, a declared physical owner, reset/carry rules, bounds,
and canonical parent. It cannot outlive or mutate above its physical owner
without a typed projection.

Simple modes may continue to map their names directly to
Activity/Section/Node/Attempt. A mode must not force two orthogonal concepts
into one physical alias merely to avoid declaring a logical scope.

## Static handler and executor bundles

Exceptional implementations are composed from immutable bundles:

```rust
RuleRegistry::compose([
    core_rule_handlers(),
    character_rule_handlers(),
    selected_mode.rule_handlers(),
])

ActivityRegistry::compose([
    core_activity_handlers(),
    selected_mode.activity_handlers(),
    selected_mode.task_executors(),
])
```

Composition validates globally unique IDs, schemas, revisions, dependency
direction, and deterministic ordering. The resulting registry digest enters
the activity/replay identity. This remains static linking; dynamic libraries,
runtime scripting, filesystem discovery, and global mutable registration are
forbidden.

A new mode owns its bundle. Adding it must not require editing a central match
statement or changing another mode's bundle.

## Component configuration identity

Runtime configuration has a composite identity:

```text
ConfigurationRootDigest
  = CombatCatalogDigest
  + BuildCatalogDigest
  + ActivityCoreCatalogDigest
  + selected ModeCatalogDigest
  + ordered selected ContentPartitionDigest values
  + composed RegistryDigest
```

The shipping `.sora` artifact may remain one physical file, but replay
compatibility is based on the ordered component digest set actually consumed
by the entry. The full artifact digest remains release/provenance metadata.
Adding an unrelated mode or localization-only partition must not invalidate an
existing replay.

Changing a consumed component still causes a hard replay mismatch unless an
explicit archive or migration is selected.

## Content maturity

Mode content uses three delivery lanes:

| Lane | Allowed inputs | Claims |
|---|---|---|
| `Experimental` | Small isolated `.xlsx` workbooks, synthetic/original fixtures, partial local manifest | No production CLI/MCP, completeness, or compatibility claim |
| `Candidate` | Sora-exported production schema, provenance, bounded manifests, deterministic fixtures | May enter integration and compatibility testing |
| `Released` | Frozen promised manifest, complete bilingual/provenance fields, coverage and replay/cross-platform gates | Production entry points and stable compatibility claim |

JSON remains debug/staging output, not an alternative production authoring
path. Experimental content must still use authoritative domain types,
fixed-point values, deterministic RNG, and validation; the lane relaxes
coverage and publication work, not simulation correctness.

Promotion creates a new digest-bound revision. Experimental records cannot be
silently counted as released coverage or loaded by production profiles.

## Historical releases

A completed Goal is immutable historical evidence. Its release verifier binds
the completion commit/tree and reads policy, status, and evidence from that
snapshot. It must not compare historical source hashes with the current
working tree or require re-blessing old evidence after an additive feature.

Current compatibility is proved by current tests, current generated-data
checks, and the new goal's evidence. Removing a released API or changing a
canonical protocol requires an explicit compatibility/replay revision; it is
not authorized by making a historical evidence file match new source.

## Change classification

| Change | Expected ownership |
|---|---|
| New data-only profile using existing operations | Mode workbook/compiler/tests only |
| New ordinary cross-battle mechanic expressible by Activity IR | Mode schema/compiler plus generic definitions |
| Exceptional deterministic mechanic | Mode-owned static handler bundle |
| New logical hierarchy name or depth | Mode logical-scope definitions; no core enum variant |
| Multiple pending branches | Activity task-set revision and replay migration |
| Genuinely different child simulation | Registered executor protocol and adapter crate |
| New formula/event/operation semantics shared by modes | Reviewed core revision with canonical/golden tests |

## Migration state

- Immutable completion-snapshot verification replaces live-source historical
  Goal validation.
- The component digest set, logical scope tree, composed registries, and
  ordered task collection are normative extension targets.
- The current Activity v2 single pending battle remains valid for sequential
  Standard and Universe profiles.
- A mode requiring multiple pending tasks, an alternate executor, or logical
  scopes that cannot map without collision is blocked from a release claim
  until the corresponding Activity/replay revision is implemented.
