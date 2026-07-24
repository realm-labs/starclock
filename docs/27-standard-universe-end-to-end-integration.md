# Standard Universe End-to-End Runtime Integration

This document defines the migration from the released Goal 04 Standard
Simulated Universe orchestration/runtime to an end-to-end playable and
verifiable runtime.

Goal 04 remains an immutable historical release. Its catalogs, deterministic
Activity graph, typed mechanic evaluators, reference battle projection and
replay evidence are valid for the contract that release declared. Goal 05 does
not rewrite that evidence. It introduces new Activity, registry, battle
materialization and replay revisions.

## Problem statement

The Goal 04 runtime has four integration gaps:

1. the CLI and agent workflows settle nested battles through
   `verified-reference-projection-v1` instead of executing
   `starclock-combat`;
2. Path, Blessing, Curio, Occurrence, service and Ability Tree evaluators can
   produce typed proposals, but the complete-run workflow does not always
   lower and apply those proposals at the authoritative Activity or Battle
   mutation boundary;
3. one source domain is lowered to several physical Activity nodes, while
   domain-local values are stored in Activity-scope keyed maps;
4. mode runtimes and replay identities are manually assembled from central
   fields and whole-bundle digests.

These are integration and extension-boundary problems. Damage formulas,
combat transaction semantics, Excel/Sora ownership and the spatial-free domain
model remain valid.

## Required runtime shape

```text
offered Activity command
          |
          v
Activity transaction + composed ActivityHandlerRegistry
          |
          +--> ordinary checked ActivityOperation values
          |
          +--> ordered PendingTask::Combat
                         |
                         v
          UniverseBattleSpecCompiler(snapshot, encounter, registries)
                         |
                         v
              immutable BattleSpec + CombatCatalog
                         |
                         v
                 Battle::create / apply
                         |
                         v
              verified projected BattleResult
                         |
                         v
                 Activity settlement
```

The Standard Universe profile remains sequential and exposes at most one
pending combat task. Goal 05 does not implement general multi-pending
`ForkJoin`; it must not encode assumptions that prevent that later Activity
revision.

## Composed registries

Mode extensions contribute immutable bundles during profile/catalog
construction:

```rust
ActivityHandlerRegistry::compose([
    core_activity_handlers(),
    standard_universe_handlers(),
])?;

CombatRuleRegistry::compose([
    core_combat_rules(),
    standard_universe_combat_rules(),
])?;
```

Composition:

- rejects duplicate stable IDs and incompatible schema/revision pairs;
- validates dependency direction and deterministic canonical ordering;
- produces a registry revision and digest;
- never uses filesystem discovery, mutable global registration or dynamic
  libraries;
- lets a handler read only a validated snapshot/context and return ordinary
  typed operations, task inputs or battle contributions;
- never lets a handler mutate Activity/Battle state directly or draw
  untracked randomness.

Adding a mechanic family must add a bundle entry, not another field and
family-specific method on `StandardUniverseActivity`.

## Atomic noncombat effects

An offered interaction binds all information needed to execute it:

```rust
pub struct ActivityInteractionBinding {
    pub offered_outcome: ActivityExternalOutcomeId,
    pub handler: ActivityHandlerId,
    pub payload: CanonicalPayload,
    pub component: ConfigurationComponentId,
}
```

Acceptance of an Occurrence choice, service purchase, Curio operation or other
noncombat outcome performs one transaction:

1. validate state hash, decision, option, handler and payload;
2. validate costs and current conditions;
3. make deterministic labeled RNG draws, if declared;
4. lower the handler result to checked `ActivityOperation` values;
5. apply costs, effects, consumption and graph transition atomically;
6. emit source-attributed events and update the Activity hash.

An effect-plan API used only by a unit test is not runtime completion.
Rejected or faulting effects preserve the pre-command state and RNG counters.

### Occurrence execution and source identity

The Standard Universe Occurrence interaction runtime precompiles all 321
authored choices into private canonical handler payloads. The payload catalog
is immutable, digest-bearing and separate from the older effect-plan view.
Each payload lowers:

- Cosmic Fragment changes to checked integer or percentage operations;
- Blessing and Curio obtain, enhance, consume, discard and lose operations to
  bounded inventory mutations;
- authored ownership/currency costs to transaction requirements;
- `StableUniformOrderedCandidates` to one labeled `Occurrence` RNG draw;
- battle, participant-HP and special-state effects that cannot yet mutate an
  Activity-owned value to source-keyed deferred effect state. The later
  battle/carry compiler consumes this state; it is never acknowledged by an
  empty handler.

The frozen partition currently produces 284 immediate checked operations and
186 deferred effect atoms. Deferred atoms are explicit boundary data, not a
claim that the combat/carry effect has already resolved.

When frozen authored data omits a scalar amount, the revision-1 lowering uses
an explicit one-unit or positive-balance project policy. Coverage and
provenance must not label that approximation as an exact game value.

Room `source_content_id` values are upstream scene-content identities, not
Occurrence IDs. A room may bind concrete choices only when its source is
provably the exact Occurrence variant source. In the frozen data, `40398`
resolves to `universe.occurrence.1.variant.40398`; values such as `0`, `22`,
`32`, `101` and `102` remain external content-selection seams. The compiler
must not turn those values into guessed Occurrences by numeric coincidence.

## Domain logical scope

Physical engine ownership remains:

```text
Activity -> Section -> Node -> Attempt
```

Standard Universe additionally declares `DomainVisit` logical instances. All
physical micrograph nodes for one source domain visit reference the same
logical scope:

```text
DomainVisit(topology, source_node, visit_sequence)
  resolution -> content -> member -> battle -> reward -> formation -> route
```

Selected room/member, clear policy, rerolls and domain-local consumption belong
to this logical instance. Run-wide Path, Blessing, Curio and currency state
remains Activity-owned. Plane-wide state remains Section-owned.

The logical identity, parent, limits and visit sequence enter canonical state
encoding. A revisit creates a new instance; it does not reuse a stale global
counter keyed only by source node ID.

## Battle materialization

`UniverseBattleSpecCompiler` consumes an immutable snapshot:

- selected World/difficulty and resolved encounter member/waves;
- participant carry/build bindings;
- selected Path, owned Blessing levels, Resonance/Formations and Curios;
- Ability Tree inputs and active run modifiers;
- technique/preemptive preparation;
- combat catalog and composed registry identities.

It produces:

- combatants and waves referencing validated combat definitions;
- resolved participant rule/modifier bindings;
- encounter/environment rule bundles;
- deterministic battle seed and purpose identity;
- a `BattleSpecDigest` covering every consumed contribution;
- a declared result projection/carry contract.

All enabled structured Standard Universe encounter members must materialize.
Where Version 4.4 public data lacks an exact enemy behavior, the existing
documented approximation policy may be used, but the battle must remain
executable and the approximation must be visible in coverage/provenance.

Production CLI, baseline, agent and MCP workflows execute the resulting battle
through `starclock-combat`. A reference projection is allowed only in
explicitly named tests/fixtures and cannot satisfy release acceptance.

## Component identity and replay

The new replay revision records the ordered consumed components:

```text
CombatCatalog
BuildCatalog
ActivityCore
StandardUniverseProfile
StandardUniverseContent partitions
ActivityHandlerRegistry
CombatRuleRegistry
EncounterOverlay
Controller
```

Each component has stable kind, ID, revision and digest. The ordered component
root, not a physical `.sora` file digest, is authoritative for simulation
compatibility. Artifact digests remain provenance.

Replay verification reconstructs exactly that component set, repeats Activity
commands and real nested battles, and detects the first component, command,
event, RNG, battle or state-hash divergence. Legacy Goal 04 replays remain
verifiable through their archived release snapshot; the new runtime need not
emit the legacy format.

## Acceptance slices

At minimum, end-to-end tests prove:

- a Blessing changes a real combat event/result hash relative to the same
  battle without it;
- a Resonance is offered, consumes energy and resolves through normal combat
  operations;
- a Curio or Ability Tree contribution changes a real BattleSpec or Activity
  settlement;
- an Occurrence atomically changes currency/inventory and graph position;
- a service atomically charges its cost and applies its purchase;
- a rejected/stale interaction and rejected battle result preserve exact
  bytes, hash and RNG counters;
- revisiting a synthetic domain creates a fresh `DomainVisit`;
- a production CLI run contains real nested battle commands/events rather than
  a reference Won projection;
- replay verification detects altered registry/component identity and nested
  battle divergence.

## Migration order

1. introduce registries and logical scope without changing Goal 04 behavior;
2. bind and atomically apply noncombat effects;
3. materialize and execute real nested battles;
4. migrate CLI/agent/MCP and replay identities;
5. remove production access to reference settlement and freeze new evidence.

Compatibility shims must be isolated and deleted once every production caller
uses the new revision. No new universe family should build on the Goal 04
reference projection seam.
