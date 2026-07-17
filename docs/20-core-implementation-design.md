# Core Implementation Design

This document turns the combat specifications into an implementation blueprint for `starclock-combat`. It is normative for the first executable workspace. Formula semantics remain owned by the formula documents; lifecycle order remains owned by [Lifecycle and resolution](10-lifecycle-and-resolution.md); this document owns Rust type boundaries, state ownership, lowering, resolver collaboration, and the initial module shape.

## Design target

`starclock-combat` is a deterministic aggregate, not an ECS world, callback framework, or hierarchy of character classes. A caller supplies an immutable catalog, a validated `BattleSpec`, a seed, and one offered command. The aggregate returns ordered facts and another stable decision boundary.

```text
immutable CombatCatalog + BattleSpec + seed
                    |
                    v
              Battle::create
                    |
                    v
        private BattleState aggregate
                    |
             offered Command
                    v
 validate -> lower -> ActionPlan/Operation queue
                    |
                    v
 working state + forward journal + trigger queue
                    |
        settle lifecycle and boundaries
                    |
                    v
     commit -> Resolution { events, decision, hash }
```

The core follows these rules:

- authored definitions are immutable and separate from runtime instances;
- only `Battle::apply` can commit authoritative mutation;
- all behavior reaches state through a closed typed `Operation` family;
- emitted events describe completed facts and never act as mutation callbacks;
- content-specific counters use declared rule slots, not fields added to shared structs;
- storage and numeric backends remain private behind domain types;
- internal iteration order is never allowed to become an accidental rule;
- Bevy, Sora rows, workbooks, filesystem paths, wall-clock time, and presentation state do not enter the aggregate.

## Ownership layers

The implementation has four distinct layers:

| Layer | Owns | Must not own |
|---|---|---|
| Catalog definitions | Unit forms, abilities, effects, rules, modifiers, enemies, encounters, validation metadata | Current HP, stacks, gauges, RNG, runtime IDs |
| Battle state | Runtime instances, queues that persist across decisions, RNG state, scopes, encounter progress | Sora records, engine entities, unvalidated content |
| Resolution transaction | Working state, journal, ephemeral queues, snapshots, budgets, emitted events | State that survives after commit except through `BattleState` |
| Integration | Commands, read-only views, replay records, controller selection, event presentation | Direct access to mutable stores or resolver internals |

`starclock-data` constructs validated catalog definitions. `starclock-activity` constructs a `BattleSpec` and consumes a `BattleResult`. Neither receives mutable access to a running battle.

`starclock-build` is an upstream compiler, not a combat submodule. It converts progression/equipment choices into the generic `ResolvedCombatantSpec` defined by `starclock-combat`. The combat crate never depends on `starclock-build` or stores its Trace, Eidolon, Light Cone, relic, affix, or preset definitions. See [Character builds, Traces, and equipment](21-build-traces-and-equipment.md).

## Public surface

The stable public surface is deliberately small:

```rust
pub struct Battle { /* private */ }

impl Battle {
    pub fn create(
        catalog: Arc<CombatCatalog>,
        spec: BattleSpec,
        seed: BattleSeed,
    ) -> Result<Self, BattleBuildError>;

    pub fn apply(
        &mut self,
        command: Command,
    ) -> Result<Resolution, CommandError>;

    pub fn view(&self) -> BattleView<'_>;
    pub fn decision(&self) -> &DecisionPoint;
}
```

Public domain types include catalog handles/build inputs, `BattleSpec`, `BattleResult`, commands, decisions, views, events, stable IDs, revisions, digests, and typed errors. Public APIs do not expose:

- `fixnum` or another numeric backend;
- Sora-generated row types;
- mutable `BattleState` or stores;
- journals, trigger indexes, queues, or caches;
- native-handler implementation types;
- a generic numeric or storage type parameter.

`CombatCatalogBuilder` is a public integration API used by `starclock-data`, but it accepts domain definitions rather than generated transport records. Successful construction produces an immutable `Arc<CombatCatalog>`; validation cannot be bypassed by normal battle construction.

## Identity model

Definition identities and runtime identities are different domains and must not be interchangeable.

Definition IDs include `UnitDefinitionId`, `AbilityId`, `EffectDefinitionId`, `RuleId`, `ProgramId`, `SelectorId`, `RuleBundleId`, `ModifierDefinitionId`, `EnemyDefinitionId`, and `EncounterId`. They are compiled from stable authored keys and remain meaningful across battle instances for the same catalog revision.

Runtime IDs include:

- `UnitId` for an HP/effect/target-capable combatant instance;
- `TimelineActorId` for an entry that can own Action Gauge or scheduled actions;
- `EffectInstanceId`, `ShieldInstanceId`, `RuleInstanceId`, and `ModifierInstanceId`;
- `ActionId`, `PhaseId`, `HitId`, `OperationId`, and `EventId` for resolution identity;
- `WaveInstanceId` and `SpawnSequence` for encounter instances.

All are fixed-width newtypes. Runtime IDs are monotonic within a battle and are never reused. Removing an instance leaves a tombstone or inactive slot so a stale ID cannot refer to a later object. Formation position, target index, vector index, content ID, and runtime ID are not aliases.

Human-readable keys remain available in diagnostics/catalog views but do not participate in hot-path lookups or tie-breaking after compilation.

## Immutable definition model

The catalog stores validated domain definitions, conceptually:

```rust
pub struct CombatCatalog {
    revision: CatalogRevision,
    digest: CatalogDigest,
    units: DefinitionTable<UnitDefinitionId, UnitDefinition>,
    abilities: DefinitionTable<AbilityId, AbilityDefinition>,
    effects: DefinitionTable<EffectDefinitionId, EffectDefinition>,
    rules: DefinitionTable<RuleId, RuleDefinition>,
    programs: DefinitionTable<ProgramId, ProgramDefinition>,
    selectors: DefinitionTable<SelectorId, SelectorDefinition>,
    rule_bundles: DefinitionTable<RuleBundleId, RuleBundle>,
    modifiers: DefinitionTable<ModifierDefinitionId, ModifierDefinition>,
    enemies: DefinitionTable<EnemyDefinitionId, EnemyDefinition>,
    encounters: DefinitionTable<EncounterId, EncounterDefinition>,
    trigger_index: TriggerDefinitionIndex,
}
```

`DefinitionTable` is conceptual. Its backend is private and may use dense vectors plus compiled lookup indexes. Definitions never contain closures, trait objects, `Any`, engine handles, mutable cells, or generated Sora rows.

`AbilityDefinition` points to an authored action program: cost policy, targeting policy, phases, hit plans, direct operations, rules, and metadata tags. `UnitDefinition` references abilities and innate rules rather than embedding executable Rust behavior. The catalog contains battle-domain definitions only. Upstream compilers may select and combine them, but they cannot add peripheral definition types to this catalog.

## Battle specification

`BattleSpec` is a complete immutable request to create one battle. It contains only validated domain values and IDs:

- catalog/rules/numeric/RNG compatibility revisions;
- encounter and variant IDs;
- ordered team/formation entries and generic `ResolvedCombatantSpec` values with opaque combatant/source digests;
- initial HP, Energy, Skill Point, Technique, and authored entry overrides;
- encounter and activity/mode `RuleBundle` bindings applied outside individual resolved combatants;
- clock, score-visible metric, persistence, and result-projection bindings;
- seed-stream identity supplied by the activity or standalone scenario;
- deterministic limits selected by the rules revision.

The spec cannot contain callbacks or mutable activity references. `Battle::create` resolves every ID and validates composition before allocating runtime IDs. A battle captures the spec and catalog digests and cannot hot-reload either.

## Resolved combatant input

`starclock-combat` defines the generic value accepted from upstream compilers and synthetic scenarios:

```rust
pub struct ResolvedCombatantSpec {
    pub form: UnitDefinitionId,
    pub level: UnitLevel,
    pub base_stats: BaseStatContributions,
    pub resources: ResolvedResourceDefinitions,
    pub abilities: ResolvedAbilitySet,
    pub rules: OrderedRuleBindings,
    pub modifiers: OrderedModifierBindings,
    pub entry_programs: OrderedEntryPrograms,
    pub sources: SourceBindings,
    pub digest: CombatantSpecDigest,
}
```

This is a battle-domain assembly, not an editable character build. It contains no Trace, Eidolon, Light Cone, Superimposition, relic, affix, inventory, or account types. `SourceBindings` carries generic stable identity/class/tag/digest information for attribution and filters; the resolver cannot inspect an upstream build definition or branch on a build-system ID.

Construction validates referenced combat definitions, value domains, canonical ordering, source identity, and the combatant-spec digest. Game-legal progression/equipment validation is an upstream policy owned by `starclock-build`. Low-level tests may construct explicitly synthetic combatant specs; production activity/mode entry points accept only compiled/digest-bound specs offered by their validated catalogs.

## Runtime aggregate

The authoritative shape is:

```rust
struct Battle {
    catalog: Arc<CombatCatalog>,
    state: BattleState,
    scratch: Option<ResolutionScratch>,
}

struct BattleState {
    identity: BattleIdentity,
    phase: BattlePhase,
    decision: DecisionPoint,
    units: UnitStore,
    actors: TimelineActorStore,
    formations: FormationState,
    teams: TeamStateStore,
    effects: EffectStore,
    shields: ShieldStore,
    rules: RuleInstanceStore,
    modifiers: ModifierStore,
    timeline: TimelineState,
    encounter: EncounterState,
    pending: PersistentPendingState,
    rng: BattleRngState,
    sequences: SequenceState,
    revisions: MutationRevisions,
}
```

`ResolutionScratch` owns one reusable working `BattleState` plus transaction
buffers. It may be initialized lazily and released at a decision boundary under
memory pressure. It is private, non-authoritative, excluded from
views/codecs/hashes and may be dropped/recreated without changing behavior. The
exact fields remain private, but ownership does not move between modules without
updating this contract. `BattleIdentity` includes the battle/spec/catalog digests and policy
revisions required by replay verification. `MutationRevisions` supports cache
invalidation; caches themselves are non-authoritative and excluded from
canonical state.

`PersistentPendingState` contains only work that is legitimately observable across command boundaries, such as the active interrupt window or a scheduled timeline action. The synchronous resolution queue must be empty whenever `apply` returns.

## Unit and timeline-actor model

A combatant and a timeline entry are related but distinct:

```text
UnitState                         TimelineActorState
---------                         ------------------
HP / life / presence              Action Gauge / speed query
Toughness / effects               action eligibility
formation slot                    owner/source link
resources / rule bindings         queued action metadata
        ^                              ^
        +------ explicit links --------+
```

A normal character usually has one `UnitId` and one linked `TimelineActorId`. A targetable memosprite may also have both. A summon such as an action-bar mechanism may have only a timeline actor linked to its owner. A boss mechanism may schedule actions without occupying a formation slot. A reserved or departed unit may remain in `UnitStore` without timeline eligibility.

Use an explicit reference when an action can have either source:

```rust
enum ActionActor {
    Unit(UnitId),
    TimelineActor(TimelineActorId),
}
```

External player commands normally name a controllable `UnitId`; enemy, summon, memosprite, and scripted actions are lowered from deterministic internal decisions to the same action envelope. Event cause records preserve both the action actor and owning unit/source when they differ.

`LifeState` and `PresenceState` remain independent. Do not create subclasses or enum variants such as `CharacterUnit`, `SummonUnit`, or `BossUnit` whose branches select shared resolver behavior. Content tags and validated links describe those roles.

## Store policy

Authoritative runtime stores use monotonic IDs and canonical traversal. The initial implementation should prefer dense `Vec<Slot<T>>`-style storage with tombstones because battle populations are bounded and small. A store exposes explicit queries such as `get`, `iter_by_id`, `formation_order`, or `timeline_order`; it never exposes backend map iteration.

Rules for all stores:

- allocation order is deterministic and recorded by sequence state;
- removal never changes another instance's ID;
- all multi-result queries declare and implement a total order;
- secondary indexes are rebuildable caches, not authoritative state;
- `HashMap`/`HashSet` iteration cannot feed selection, event order, encoding, or hashes;
- a public view yields domain view objects rather than store references;
- canonical encoding walks fields and stores in their specified order, including inactive records only where the codec revision requires them.

Backends may later change to arenas, copy-on-write pages, or compact generations without changing public IDs or simulation semantics.

## Commands and decisions

Commands are external intent values. They contain no precomputed damage, selector result, callback, or client-provided legality flag. Initial families are:

- start battle;
- use a currently offered ability with an offered target choice;
- use or pass an interrupt/Ultimate window;
- answer a battle-local typed choice emitted by a rule;
- concede only if the selected profile explicitly offers it.

`DecisionPoint` owns the canonical ordered legal-command collection. Controllers select one existing value rather than constructing equivalent commands. A command carries the decision sequence it answers; stale commands are rejected before mutation.

Legality validation reads the current state and catalog and returns a private `ValidatedCommand`. It verifies phase, decision identity, controller ownership, actor/life/presence, costs, ability availability, target choice, and rule-specific restrictions. Failed validation consumes no IDs or RNG and leaves the canonical state byte-identical.

## Lowering to actions

A validated action command is lowered to a finite `ActionPlan`:

```rust
struct ActionPlan {
    id: ActionId,
    actor: ActionActor,
    owner: Option<UnitId>,
    ability: AbilityId,
    kind: ActionKind,
    normal_turn: Option<TimelineActorId>,
    cause: CauseSeed,
    cost: CostPlan,
    target_commitment: TargetCommitment,
    phases: Vec<ActionPhasePlan>,
}

struct ActionPhasePlan {
    id: PhaseId,
    steps: Vec<ActionStepPlan>,
}
```

An `ActionStepPlan` is a hit plan, a direct operation template, or a declared reaction/boundary window. A `HitPlan` defines hit identity, target reference point, target invalidation policy, damage/toughness/effect operation templates, snapshots, and per-hit RNG purposes. It does not contain animation timing.

The plan captures authored structure and command commitments, not every dynamic result. Selectors marked current-state, conditional expressions, stat queries, retargeting, and trigger results are evaluated at their declared operation boundary.

Ultimates, extra turns, follow-ups, counters, enemy actions, summons, memosprites, joint actions, and scripted encounter actions all use `ActionPlan`. Differences are expressed by `ActionKind`, ownership, cost, priority, duration-clock policy, target policy, and rule programs rather than separate executors.

## Operation model

`Operation` is the only language allowed to request authoritative mutation. Use a closed enum with typed payload structs rather than `Box<dyn Operation>`:

```rust
enum Operation {
    Damage(DamageOp),
    Heal(HealOp),
    ConsumeHp(ConsumeHpOp),
    Shield(ShieldOp),
    Toughness(ToughnessOp),
    Break(BreakOp),
    Effect(EffectOp),
    RuleState(RuleStateOp),
    Resource(ResourceOp),
    Timeline(TimelineOp),
    QueueAction(QueueActionOp),
    Unit(UnitOp),
    Presence(PresenceOp),
    Weakness(WeaknessOp),
    Encounter(EncounterOp),
    Decision(DecisionOp),
    Informational(InformationalOp),
}
```

Payload enums/structs distinguish apply/remove/refresh, summon/despawn/transform, advance/delay, layer creation/removal, and other typed suboperations. Do not create a universal string-key/value operation. Every operation declares source/cause, selector or committed targets, snapshot boundary, empty-target policy, formula/rounding policy where applicable, and fault policy.

An operation executor performs the normative atomic sequence: select/revalidate, allocate identity, snapshot, calculate without mutation, mutate working state, emit facts, settle replacements/defeat/target invalidation, collect triggers, and enqueue reactions.

Damage, Toughness reduction, Break, effect application, and resource generation remain separate ordered operations even when one authored hit contains all of them. This preserves kit-specific timing without embedding character checks into formulas.

## Event and cause model

Events are immutable facts produced after a mutation or explicit lifecycle boundary:

```rust
pub struct BattleEvent {
    pub id: EventId,
    pub cause: Cause,
    pub kind: BattleEventKind,
}

pub enum BattleEventKind {
    Battle(BattleEventData),
    Wave(WaveEventData),
    Turn(TurnEventData),
    Action(ActionEventData),
    Hit(HitEventData),
    Damage(DamageEventData),
    Heal(HealEventData),
    Shield(ShieldEventData),
    Toughness(ToughnessEventData),
    Break(BreakEventData),
    Effect(EffectEventData),
    Resource(ResourceEventData),
    Timeline(TimelineEventData),
    Unit(UnitEventData),
    Encounter(EncounterEventData),
    RuleState(RuleStateEventData),
    Decision(DecisionEventData),
    Fault(FaultEventData),
}
```

Families keep source-compatible public organization without forcing one enormous flat enum. Payloads contain authoritative domain values before presentation formatting. Informational events cannot masquerade as mutations.

`Cause` records root command, parent event, action, phase, hit, rule owner, action actor, applier, source definition, primary target, and optional activity-supplied source. Nested triggers copy the root and link the immediate parent; they do not reconstruct attribution later.

## Rules, triggers, and native handlers

Catalog compilation creates trigger indexes by event kind, phase, and cheap filter keys. At runtime:

1. the executor emits an event into the transaction;
2. `TriggerMatcher` retrieves candidate rule instances from the compiled indexes;
3. candidates are filtered against event/cause/current or declared snapshot context;
4. candidates are totally ordered by phase, reaction priority, owner/formation, source ID, rule ID, and instance sequence;
5. `RuleEvaluator` evaluates conditions/selectors/value expressions without mutation;
6. the program produces operation templates, queued actions, or replacement proposals;
7. the resolver materializes them with cause and budget metadata and appends them to the appropriate queue.

`RuleInstanceState` stores only definition ID, owner/scope links, lifecycle flags, and declared slot values. Character-specific fields are forbidden in `UnitState`. Once-scope keys live in the rule-state subsystem and use the IDs defined by the corresponding event/action/scope.

Static native handlers implement the same evaluation contract: read-only context in, typed proposals/templates out. They do not receive mutable stores or a `&mut BattleState`. The registry is versioned and looked up by validated `NativeHandlerId`.

## Stat, modifier, and formula services

Calculators are pure services over explicit contexts:

```text
StatResolver      BattleState + StatQuery       -> domain value / query fault
DamageCalculator DamageContext + stat results   -> DamageCalculation
HealCalculator   HealContext + stat results     -> HealCalculation
ShieldCalculator ShieldContext + stat results   -> ShieldCalculation
BreakCalculator  BreakContext + stat results    -> BreakCalculation
```

They never mutate state or emit events. The operation executor applies the returned calculation and creates the corresponding facts. Calculation results may include a structured trace for tests/diagnostics, but only explicitly selected authoritative fields enter canonical events.

`ModifierStore` contains runtime instances; definitions remain in the catalog. `StatResolver` queries applicable instances through the staged pipeline and detects cycles using an explicit query stack. Optional query/index caches are scoped to the transaction or guarded by mutation revisions, excluded from hashes, and required to produce identical results when disabled.

## Resolution transaction

The first implementation uses an owned working-state transaction backed by
reusable battle-local scratch storage. Command legality is validated before the
scratch state is prepared, so forged/stale commands do not pay state-copy or
journal-allocation cost.

```rust
struct ResolutionTxn<'a> {
    catalog: &'a CombatCatalog,
    before: &'a BattleState,
    working: BattleState,
    journal: Vec<JournalEntry>,
    events: Vec<BattleEvent>,
    operations: OperationQueue,
    reactions: ReactionQueue,
    query_stack: QueryStack,
    budgets: ResolutionBudgets,
}
```

`working` begins as a semantic clone of the authoritative state. The battle
retains one non-authoritative scratch `BattleState` and reusable journal, queue,
selector, trigger-candidate, snapshot, and query buffers. Preparation uses
`clone_from`-style semantic copying into existing capacity instead of allocating
a fresh object graph for every accepted command. Scratch capacity, caches and
buffer layout are never canonical state.

The append-only journal records every mutation, ID allocation, RNG draw,
snapshot, queue insertion, and event in canonical sequence. This makes command
atomicity easy to audit without requiring inverse mutations.

On success, the transaction settles to a decision/terminal boundary, verifies
invariants and empty ephemeral queues, computes the canonical hash from
`working`, and swaps it into `Battle`; the previous authoritative allocation
becomes the reusable scratch state for the next command. On `Rollback`, it
discards the semantic contents of `working` while retaining reusable capacity
and commits a deterministic `Faulted` state derived from `before`. On
`CommitFault`, it commits completed atomic operations in `working`, appends the
stable fault fact, and enters `Faulted`.

Returned `Resolution.events` own their public values; the battle cannot reuse
that event vector until ownership has moved out. Internal journals and ephemeral
queues are cleared and reused after settlement. Capacity retention must obey a
documented upper bound so one pathological but legal battle cannot permanently
bloat every pooled verifier job. A service adapter may request scratch eviction
for an idle session, but cannot observe or depend on retained capacity.

The journal is not the replay format and is not public. Copy-on-write pages,
reversible patches, incremental hashes, or arena backends may replace the
baseline only after profiling identifies the relevant cost. Such a change must
preserve fault semantics, events, IDs, RNG draws, and hashes, and must not leak a
backend type through the public API.

## Queue model

Use explicit queues, never recursive trigger calls:

- `OperationQueue` executes the current authored action/phase/hit work;
- `ReactionQueue` holds trigger-produced actions/operations ordered by reaction key;
- `InterruptWindowState` persists only while awaiting an external interrupt/pass command;
- `TimelineQueue` derives the next eligible actor from canonical Action Gauge ordering;
- encounter boundary requests are collected separately and settled only at allowed boundaries.

Every queue entry has a stable insertion sequence and a complete total-order key. Resolution budgets limit operations, events, reactions, trigger depth, queued actions, target count, hit count, spawns, and active instances. Exceeding a budget is a typed deterministic fault.

## Lifecycle and encounter settlement

Lifecycle is not spread across damage, effect, summon, and wave modules. The resolver calls focused settlers at declared boundaries:

- `ReplacementResolver` selects death prevention, redirection, and replacement proposals;
- `DefeatSettler` transitions Downed/Defeated, invalidates targets/actions, and assigns credit;
- `LinkSettler` applies authored owner/summon/memosprite teardown or transfer policies;
- `ActionBoundarySettler` handles duration ticks, turn completion, and queued reactions;
- `EncounterBoundarySettler` handles boss transitions, spawn programs, waves, victory, and loss.

These services produce ordinary operations/events through the transaction; they do not silently mutate unrelated stores. Formula modules may report zero HP or zero Toughness, but only lifecycle settlement changes life state, resolves Break, advances a wave, or terminates battle.

## Canonical state and views

The canonical battle state includes identity/revisions, phase/decision, authoritative store records, formations/teams, timeline, encounter progress, persistent pending work, RNG stream state/draw counters, and sequence allocators in fixed codec order. It excludes `Arc` addresses, definition bodies already represented by the catalog digest, indexes, caches, journal history, diagnostic strings, and presentation data.

Canonical encoding writes directly into a sink. The authoritative hash path
streams fields into the SHA-256 sink and must not first allocate a complete
canonical `Vec<u8>`. Tests/debug tools may select a byte-collecting sink from the
same encoder. Streaming versus collecting cannot change field order or bytes.
The initial codec remains a full-state digest; an incremental/Merkle scheme is a
new `state_hash_revision`, not a transparent optimization.

`BattleView` is a borrowed read-only projection with explicit query methods. It may expose units in formation or stable-ID order, legal commands, timeline order, visible effects/resources, encounter progress, and exact domain values. It cannot expose internal mutable references or container types. A controller and engine adapter consume the same view/decision contract.

## Throughput contract

One battle remains logically single-threaded. A headless verifier shares one
validated immutable `Arc<CombatCatalog>` per configuration digest and runs many
isolated `Battle` values on a bounded worker pool. Combat code does not add an
async runtime, global mutable cache, or cross-battle lock.

The implementation must measure, by versioned workload:

- ordinary and trigger-heavy `Battle::apply` latency;
- state semantic-copy bytes/time and retained scratch capacity;
- canonical bytes hashed and hashing time;
- journal/event/operation counts and allocation count/bytes;
- full replay commands/second/core and peak transient bytes/job;
- scaling when many independent replay jobs share one catalog.

Catalog compilation builds event/phase/filter trigger indexes, stable lookup
indexes and modifier-query indexes once. Runtime must not repeatedly scan the
whole catalog or sort an unchanged global definition set. Transaction-local or
revision-guarded stat caches and candidate buffers may be reused, but disabling
them must produce identical events and hashes.

## Module layout

The initial `starclock-combat` layout should be responsibility-oriented:

```text
src/
  lib.rs                    small documented public facade
  id.rs                     fixed-width domain IDs
  numeric/                  private backend and public domain newtypes
  catalog/
    definition.rs           immutable domain definitions
    builder.rs              validated catalog construction
    index.rs                compiled definition/trigger indexes
  battle/
    aggregate.rs            Battle public methods and commit boundary
    state.rs                private authoritative state ownership
    build.rs                BattleSpec validation and instance allocation
    view.rs                 read-only external projection
  command/
    model.rs                public command/decision values
    legal.rs                legal-command construction
    validate.rs             command -> ValidatedCommand
  actor/                    units, timeline actors, formation, links
  resource/                 team/personal resources and bounds
  target/                   selectors, commitments, patterns, retargeting
  action/                   action/phase/hit plans and lowering
  operation/                closed operation model and typed payloads
  resolver/
    transaction.rs          working state and journal
    execute.rs              atomic operation execution
    queue.rs                operation/reaction ordering
    settle.rs               action/terminal boundaries
    budget.rs               deterministic limits
  event/                    event families, Cause, collection
  rule/                     instances, slots, matching, evaluation
  modifier/                 instances, stacking, applicability
  stat/                     stat queries and cycle detection
  formula/                  damage, healing, shields, Break
  effect/                   effect/shield lifetime and operations
  timeline/                 Action Gauge and turn ownership
  encounter/                waves, phases, spawns, terminal policy
  rng/                      versioned deterministic mappings
  codec/                    canonical battle-state encoding
  error/                    build, command, query, and fault types
```

Split files by responsibility before the 1,200-line limit. `lib.rs` lists modules and a deliberately small facade; internal modules import defining paths and do not depend on broad `pub use` exports.

Dependency direction inside the crate is conceptual even though Rust modules share a crate:

```text
IDs/numeric/definitions
        -> state and pure contexts
        -> formulas/rules/plans
        -> operations/events
        -> resolver transaction
        -> Battle aggregate/public views
```

Lower layers cannot call the aggregate or integration adapters. Avoid a `context` or `manager` module that becomes a second mutable root.

## Explicit non-abstractions

Do not introduce the following in the first implementation:

- a `Character` trait implemented once per released character;
- inheritance-like `Unit` variants that choose formula or lifecycle behavior;
- `dyn Effect`, `dyn Ability`, or `dyn Operation` object graphs;
- arbitrary Lua/Rhai/JavaScript or expressions stored as JSON cells;
- a service locator, global registry with mutable state, or `Any` downcasts;
- a public `BattleState` with setters;
- a generic numeric backend in public signatures;
- Bevy ECS components as authoritative combat state;
- mode/character ID branches in shared formula, target, timeline, or lifecycle code;
- a second operation/event language for encounters or universe effects.

Add an abstraction only when at least two concrete mechanics need the same semantic extension and the owner/lifecycle/order can be stated. Prefer a new typed operation, selector, filter, snapshot policy, or state slot over a new trait hierarchy.

## Initial implementation sequence

Build vertically in this order:

1. workspace crates, domain numeric newtypes, IDs, revisions, digests, and minimal combat catalog builder;
2. minimal synthetic `ResolvedCombatantSpec` validation with generic base contributions, abilities, rules, modifiers, sources, and digest;
3. `BattleSpec` construction, unit/actor stores, formation, battle phases, views, and command legality;
4. working-state transaction, journal, event/cause model, budgets, and canonical hash skeleton;
5. timeline plus one normal Basic ATK lowered to action/phase/hit/operations;
6. stat query and ordinary damage formula through mutation, defeat, terminal settlement, and replay fixture;
7. resources, multi-hit/Blast/AoE/Bounce targeting, healing, shields, Toughness, Break, and effects;
8. rule slots, trigger indexes, conditions/selectors/program interpreter, reactions, and modifiers;
9. follow-ups, counters, Ultimates/interrupts, extra turns, summons/memosprites, transformation, waves, and boss phases;
10. enemy AI/encounters and the representative content slices;
11. upstream `starclock-build` and `starclock-activity` handoff after the single-battle contracts are golden-tested.

Each step must produce a command-to-hash golden fixture. Do not import the full content catalog before the operation, rule, lifecycle, and validation contracts used by that content exist.

## Required implementation tests

- catalog construction rejects unresolved, mistyped, cyclic, or cross-domain definitions;
- runtime IDs are monotonic, never reused, and stable under container-layout changes;
- legal-command enumeration is canonical and stale/forged commands do not mutate state;
- every operation family completes its atomic sequence and emits exact cause-linked facts;
- working-state rollback and commit-fault paths preserve the documented state boundary;
- queue insertion/container order does not affect selected reactions, events, or hashes;
- calculators are pure and cache-enabled/disabled stat queries are identical;
- units and timeline-only actors cover characters, action-bar summons, targetable memosprites, and boss mechanisms;
- rule slots express representative character counters without shared-state fields;
- multi-hit death, retargeting, revival, transformation, linked teardown, boss phase, and every wave policy have golden fixtures;
- canonical state round trips through the versioned codec and ignores caches/journals;
- a controller and a presentation adapter can complete the same battle using only public views, offered commands, and events;
- Windows, Linux, macOS, and supported CPU architectures produce identical accepted-command event/state hashes.
