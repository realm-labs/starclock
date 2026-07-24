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
- Blessing obtain, enhance, consume, discard and lose operations to bounded
  inventory mutations;
- Curio obtain and removal through the complete lifecycle tuple described
  below;
- authored ownership/currency costs to transaction requirements;
- `StableUniformOrderedCandidates` to one labeled `Occurrence` RNG draw;
- battle, participant-HP and special-state effects that cannot yet mutate an
  Activity-owned value to source-keyed deferred effect state. The later
  battle/carry compiler consumes this state; it is never acknowledged by an
  empty handler.

The frozen partition currently produces 283 immediate checked operations and
187 deferred effect atoms. Curio enhancement is intentionally deferred instead
of being misrepresented as acquisition of a second Curio. Deferred atoms are
explicit boundary data, not a
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

### Service, shop and currency execution

The service interaction runtime owns one immutable, digest-bearing compiler
over all 94 normalized service definitions. A concrete selection becomes a
versioned private payload containing only checked Activity operations. It
supports:

- exact Cosmic Fragment initialization, debit and scheduled reroll prices;
- one-star Blessing and Curio respite purchases and random Blessing
  enhancement;
- rarity-priced explicit Blessing enhancement;
- concrete Blessing/Curio shop purchases;
- service-use counters; and
- explicit deferred atoms for participant carry, roster and Trailblaze effects
  whose authoritative consumer belongs to the later battle/carry boundary.

Random respite content uses the `Shop` RNG stream. Candidate IDs are
canonically ordered and filtered against current inventory before the draw;
an empty candidate set faults without committing state or RNG. Production
topology suppresses options whose exact fixed price exceeds current Cosmic
Fragments.

The frozen shop definitions expose pool and price-formula identities, but not
the concrete generated offers and their prices. Entering a Transaction domain
therefore opens a source-attributed shop interaction. A concrete purchase must
come from a trusted offer compiler and carry the selected content ID, positive
price and non-zero offer digest. The service runtime validates that envelope
and applies debit, inventory grant and use count in the same transaction. It
must never invent a price from a missing row or accept an arbitrary zero-cost
purchase.

Revision 2 exposes only options whose preconditions can be represented exactly
by the current Activity condition vocabulary. Random two-Blessing enhancement
and revival are executable concrete selections, but their production offer
generators must wait for inventory-cardinality and participant-life predicates
respectively. Downloader and shop opening record explicit deferred state for
the roster/offer consumer; recording that state is not a claim that the
roster or purchase already changed.

### Curio lifecycle and boundary effects

Activity ownership of one Curio is not represented by inventory membership
alone. It is the validated tuple:

```text
(inventory membership, active state, remaining charges, pending lifecycle events)
```

The immutable Curio Activity catalog freezes this tuple's initial state and
charge count for all 61 Curios. Occurrence and service acquisition initialize
the complete tuple and record `Acquired` in the private Curio event slot in the
same transaction as payment and graph progression. Removal/discard/lose
operations remove inventory ownership and clear state and charges atomically.
An orphan inventory row is invalid input to contribution compilation.

`CurioActivityProjection` consumes exactly one recorded lifecycle event and
returns ordinary checked `ActivityOperation` values. Run-owned Fragment grants
and losses execute immediately. RNG-dependent, participant-carry, combat and
later-lifecycle effects become bounded source-keyed deferred atoms in the same
transaction; P3 owns their battle/carry consumers. Recording a deferred atom
does not claim that its effect has occurred.

### Ability Tree boundary projection

All 42 Ability Tree nodes and 50 normalized effects retain the Goal 04 typed
executor. At each declared Run or Battle boundary, the executor now also
projects every resolved target into ordinary Activity counter operations.
Each operation replaces the previous projected value by applying the checked
difference, so repeated boundary evaluation cannot silently stack the same
contribution.

The selected RunStart projection is materialized in private Activity state
during entry compilation. BattleStart, elite/boss entry and post-battle
projections use the same operation form and remain explicit boundary inputs for
the P3 combat-contribution and carry compilers. No Ability Tree row type or
numeric backend becomes part of the public battle API.

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

### Immutable battle-contribution snapshot

`UniverseBattleContributionCompiler` is the only Standard Universe
Activity-to-combat rule boundary. It freezes all 786 normalized mechanic-rule
records and validates their exact family denominators before any run starts:
42 Ability Tree, 162 Blessing definitions, 324 Blessing levels, 61 Curio
definitions, 67 Curio states, 36 Resonance/Formation and 94 run-service rows.
Duplicate `(kind, source_record)` identities fail catalog construction.

For one battle boundary it reads one immutable authoritative snapshot and
produces a canonical `UniverseBattleContributionSet`:

- the selected Path identity, selected-Path Blessing count and Path digest;
- both definition and selected-level rule bindings for every owned Blessing;
- the unlocked Resonance and each selected Formation binding;
- both definition and current-state bindings for every valid owned Curio;
- only Battle or Run-and-Battle Ability Tree source bindings;
- combat modifier definitions for party ATK, DEF, maximum HP, CRIT Rate,
  Speed, CRIT DMG and Effect Hit Rate; and
- all remaining Ability Tree values as typed boundary resources for the
  Resonance/resource compiler.

Every source binding carries a stable `RuleId`, `RuleBundleId`,
`SourceDefinitionId`, source-record key, source-binding key, mechanic tags and
canonical source digest. IDs use a reviewed reserved namespace and are checked
against the core combat catalog for collisions. Static Ability Tree modifiers
use combat-owned `ModifierDefinition` and `ModifierStackingGroup` values and
are validated by the normal combat modifier registry.

The contribution batch freezes binding declarations and modifier definitions.
P3-B2 composes definitions that are already executable, including the seven
static Ability Tree modifiers. Event-driven Rule IR definitions remain an
explicit zero-count coverage field until P3-B4 translates and proves them; a
binding declaration alone is not evidence that an event-driven Blessing,
Resonance or Curio effect has resolved.

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

### Executable encounter materialization revision 1

`UniverseBattleMaterializer` accepts three validated domain inputs:

```rust
UniverseBattleMaterializer::compile(
    universe_catalog,
    &locked_resolved_roster,
    &battle_contributions,
)
```

It returns one immutable `UniverseBattleMaterialization` containing a composed
`CombatCatalog`, the complete 173-member Activity overlay, 182 ordered
difficulty enemy battle specifications, per-enemy definition disposition,
coverage and a canonical root digest. The locked roster must match participant,
formation, character form and pre-mode combatant digest exactly. Party
modifiers are added to a new source-attributed combatant assembly; the
participant build lock itself remains the upstream pre-mode identity.

The generic combat composition seam is
`CombatCatalogBuilder::from_catalog`. It copies Starclock-owned domain
definitions from an already validated catalog, accepts additional mode-owned
definitions and runs the complete catalog validator again. It neither mutates
the base catalog nor exposes private definition-table or generated row types.

The frozen Version 4.4 materialization denominator is:

| Item | Materialized | Exactness |
|---|---:|---|
| structured encounter members | 173 | exact member identity and order |
| encounter waves | 173 | exact authored topology |
| member enemy slots | 538 | exact slot order, source key and stage level |
| difficulty boss/elite bindings | 182 | exact ordered source key, role and level |
| distinct referenced enemy variants | 86 | 13 exact core definitions; 73 explicit proxies |
| event-driven Universe rule bindings | snapshot-dependent | declared, not yet materialized in P3-B2 |
| static Ability Tree modifiers | up to 7 | executable combat modifier definitions |

Only 13 of the 86 referenced variants exist in the Goal 01 representative
combat bundle. Each of the other 73 records therefore has
`ApproximateProxy`, its original stable key, selected proxy key and proxy enemy
ID. Proxy selection is deterministic by authored rank token: Minion,
MinionLv2, Elite or BigBoss. It never claims that the proxy reproduces the
missing enemy's skills, phases or AI.

Goal 01 did not retain full scalable enemy stat rows for this broader enemy
set. Revision 1 consequently uses the explicit
`goal01-executable-enemy-proxy-stats-v1` policy for every Universe enemy
occurrence: exact authored level with the existing Goal 01 executable proxy
HP/Speed assembly. This policy is approximate even when the enemy definition
itself is an exact match. It exists to make every structural request
executable without fabricating authoritative statistics; importing complete
Excel/Sora enemy definitions and curves is a later data expansion, not a
silent revision of these coverage claims.

Member encounter IDs, difficulty encounter IDs and wave IDs occupy separate
reviewed ranges. Catalog composition rejects collisions. Every emitted
`BattleSpec` is immediately passed to `Battle::create` during materialization;
a missing unit, ability, modifier, enemy, encounter, wave participant or
source binding fails the whole compiler before an Activity can expose the
overlay. The materialization root golden is
`afc6a00b2adf0d106adb01d64ec61ba8b1202c5fae8b07a5cf510a921b9e0dc4`;
the explicit coverage golden is
`2fa0e46786809544478f9c224ea45539540f278ff5fed3548a6e5c119aded9f3`.

The production nested executor now executes the resulting battle through
`starclock-combat`. CLI, agent and MCP caller migration is owned by P4; until
then their legacy Goal 04 paths remain truthfully labeled
`verified-reference-projection-v1`. A reference projection is allowed only in
explicitly named tests/fixtures after that migration and cannot satisfy
release acceptance.

### Production nested execution revision 1

`UniverseNestedBattleExecutor` owns one immutable composed `CombatCatalog` and
executes the exact `ActivityBattleHandoff` synchronously:

1. bind every Activity participant ID to one player formation slot;
2. initialize the first battle from the resolved combatant and later battles
   from the exact HP, Energy, life and presence carry ledger;
3. call `Battle::create` with the Activity-derived battle seed;
4. answer system boundaries canonically, player boundaries with a stable
   legal-command policy and enemy normal actions through the authored
   `EnemyController`;
5. repeatedly call `Battle::apply` until Won, Lost or Faulted;
6. construct `ProjectedValue` entries in the exact order declared by the
   handoff projection; and
7. let Activity verify and settle the sealed result atomically.

The handoff exposes the projection and an explicit
`(ParticipantId, FormationIndex)` mapping. It never requires an executor to
guess positional participant identity. A first handoff supplies full/default
carry values instead of an ambiguous empty ledger.

`ParticipantSpec` separates two identities:

- `locked_combatant_digest` is the pre-mode resolved build checked by
  `ParticipantLock`;
- `combatant().digest()` identifies the actual runtime assembly after
  Universe modifiers and rule bindings have been added.

Equating those digests would reject every legitimate mode contribution.
`ParticipantInitialState` is the generic cross-battle initialization seam; it
contains no Universe-specific state and standalone battles retain their
existing full-HP defaults.

Executor infrastructure failures return
`NestedBattleExecutionError` without submitting a fabricated result. Activity
rolls back only the adapter-owned started marker; because battle start consumes
no Activity command or RNG draw, the pre-call Activity hash is restored and
the same handoff can be derived and retried. Only a combat-owned terminal
fault becomes `BattleOutcome::Faulted` plus the exact `BattleFault`
projection. The executor enforces a 10,000-command default budget and retains
the accepted command/controller/state-hash/event-count trace for replay
integration.

The revision-1 `EventDigest` is a versioned SHA-256 commitment to catalog and
rules identity, seed/spec identity, every accepted command, every resulting
canonical state hash, and every ordered event ID/cause/family. Those frozen
inputs deterministically imply the complete event payload. P4-B1 replaces
this input-and-shape commitment with a payload-direct event replay component;
the two formats have different revision labels and must never be compared as
the same codec.

The first real Activity fixture completes three nested battles using 15
accepted combat commands. Its final Activity state hash is
`8c09daedc0e35920f50d0bebd698415b2c42e2815f49f8bc9fc060270938c024`;
the first battle event commitment is
`30ffe825c5e350df5191981de13d380b6751070672adb1c55c6ec3be79f4a751`.

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
