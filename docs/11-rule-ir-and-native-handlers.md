# Rule IR and Native Handlers

This document defines the shared typed rule representation used by characters, equipment, enemies, encounters, and universe modes. Sora tables are transport records; `starclock-data` validates and compiles them into this immutable IR.

## Design constraints

- One IR must express ordinary content from every source domain.
- Programs are typed data, not Rust, Lua, JSON-in-a-cell, or an unbounded expression language.
- Evaluation is deterministic, budgeted, and side-effect free until operations reach the resolver.
- Content cannot inspect mutable collections directly or choose an unspecified order.
- Exceptional Rust code is registered statically and emits normal operations.

## Rule definition

The conceptual domain shape is:

```rust
pub struct RuleDefinition {
    pub id: RuleId,
    pub domain: RuleDomain,
    pub source: RuleSource,
    pub state_slots: Vec<StateSlotDef>,
    pub triggers: Vec<TriggerDef>,
    pub native_handler: Option<NativeHandlerId>,
}

pub struct TriggerDef {
    pub id: TriggerId,
    pub event: EventKind,
    pub phase: TriggerPhase,
    pub filter: EventFilter,
    pub condition: ConditionExpr,
    pub once_scope: OnceScope,
    pub priority: ReactionPriority,
    pub program: ProgramId,
}
```

`RuleDomain` is `Battle` or `Activity`. It selects the legal event vocabulary, scopes, selectors, operations, and handler registry. A definition cannot mix the two domains; cross-boundary data uses declared BattleSpec/Result bindings.

`RuleSource` identifies Character, Light Cone, Relic, Blessing, Curio, Equation, Path Resonance, Encounter, Enemy Aura, Enemy Ability, Activity Modifier, or Mode. Source identity is retained in every produced operation and event.

## State slots

A state slot declares ID, value type, owner scope, initial expression, minimum/maximum, reset points, visibility, and persistence. Supported initial value types are bounded integer, fixed scalar, boolean, stable ID, optional ID, and a small ordered ID set.

`RuleScope` follows the generic hierarchy Activity, Section, Node, Attempt, Battle, Wave, Turn, Action, and Hit. Universe terms such as Run/Plane/Domain and challenge terms such as Stage/Side are authoring aliases compiled to Activity/Section/Node. Runtime and replay formats do not add a new scope enum for each mode.

A slot cannot outlive its owner. Promotion from a shorter to longer scope is forbidden; an explicit typed projection/aggregation operation is required. Activity/Section/Node/Attempt slots are owned by `starclock-activity`; battle and shorter slots are owned by `starclock-combat`. Neither core directly mutates the other's state.

Examples include a character counter, once-per-turn marker, boss phase, blessing battle tally, equation threshold, curio activity charge, challenge stage clock, or node-local dice effect. Do not create ad hoc fields in `BattleState` or `ActivityState` for content-specific counters.

## Value expressions

`ValueExpr` is a closed typed union:

- literal integer, fixed scalar, ratio, probability, or stable ID;
- read state slot or resource;
- query source/target stat through a declared `StatQuery`;
- read event/cause property;
- count or sum a selector result with explicit order;
- checked add, subtract, multiply, divide, minimum, maximum, clamp, or negate;
- choose by condition;
- convert through an explicit checked domain conversion and rounding policy.

Expressions cannot mutate state, draw RNG, perform unbounded iteration, recurse, read wall-clock time, or access presentation data. Invalid arithmetic becomes a typed fault rather than false/zero.

## Conditions and event filters

`ConditionExpr` supports typed comparisons, boolean composition, tag membership, life/presence checks, resource bounds, effect/state existence, weakness/broken state, selector cardinality, and event/cause predicates.

An `EventFilter` first narrows by cheap indexed fields such as source, owner, actor, applier, target, action kind, ability tags, element, damage class, and cause ancestry. The condition then evaluates contextual values. This split is an implementation optimization but must not change semantics.

## Selectors

A selector declares:

- origin and side relationship;
- required life and presence states;
- formation, mark, weakness, effect, tag, or ownership predicates;
- ordering: formation, timeline, HP ratio, stat, event order, or stable ID;
- cardinality and empty-pool behavior;
- deterministic choice: all, first, primary-plus-adjacent, or RNG choice with purpose/weights;
- whether repeated targets are allowed.

Every selector result is a stable ordered vector. A selector may refer to the event snapshot, action snapshot, or current state; the reference point is explicit.

## Programs and operations

A `Program` declares `Battle` or `Activity` execution ownership and is a finite ordered list of operations plus structured `If`/bounded iteration blocks. Battle operations are:

- damage, true damage, heal, shield, HP consumption, and damage redirection;
- Toughness reduction, layer creation/removal, Break, and Super Break;
- apply, remove, refresh, transfer, or modify an effect;
- modify personal/team resources and battle-or-shorter state slots;
- advance/delay action, queue/cancel action, and create an extra turn;
- summon, despawn, transform, replace ability, set field, and change presence;
- add/remove weakness or resistance override;
- emit a typed decision request or an informational rule event;
- request an encounter phase/wave transition;
- invoke a validated native handler.

Programs cannot directly change collections or HP. They produce resolver operations, which enforce target legality, rounding, attribution, events, reactions, and budgets.

Activity programs use the same expression/condition discipline but emit only the graph, participant, inventory, clock, metric, objective, decision, and BattleSpec operations defined in [Activity core and mode extension](19-activity-core-and-mode-extension.md). Battle programs cannot write activity slots; activity programs cannot mutate live battle state. Cross-boundary values use declared `BattleSpec` bindings and `BattleResultProjection` fields.

## Trigger phases and once scopes

Trigger phase is independent of observed event kind. Supported phases include Before, Replace, AfterMutation, AfterDefeatSettlement, AfterEvent, AfterAction, and Boundary. Replacement triggers cannot themselves mutate; they return a replacement proposal ordered by priority.

`OnceScope` keys include per event, hit, target within hit, ability, action, turn, wave, battle, attempt, node, section, and activity. Mode-facing aliases compile to the generic scope. The key always includes rule instance and trigger ID. Multi-target and bounce behavior therefore cannot accidentally multiply a once-per-action passive.

## Native handler boundary

The static native registry maps `(HandlerDomain, NativeHandlerId)` to a versioned battle or activity handler implementation. Dynamic libraries, runtime scripting, and arbitrary function names from Excel are forbidden.

A handler receives a read-only rule context, validated arguments, the triggering event/cause, and budget access. It may return typed operations, state-slot deltas, or a replacement proposal. It may not:

- mutate battle/run state directly;
- call random APIs outside the supplied deterministic request interface;
- emit events without resolver operations;
- bypass costs, target validation, fault policy, or trigger limits;
- depend on a character-ID branch in a shared handler;
- access filesystem, network, platform time, Bevy entities, or global mutable state.

Add a native handler only after documenting why selectors, expressions, triggers, and operations cannot express the mechanic without unreasonable duplication or complexity. Handler ID, argument schema, determinism notes, owner, and removal condition belong in the native-handler registry document/data.

## Validation

Bundle loading rejects:

- unresolved program, rule, selector, state-slot, source, or handler IDs;
- type-invalid expressions or operations;
- illegal scope reads or writes;
- battle programs containing activity operations or activity programs containing combat mutations;
- unbounded loops, recursive program calls, and statically cyclic program graphs;
- selector/RNG operations without stable ordering and empty-pool policy;
- replacement triggers that emit ordinary mutations;
- once scopes missing a constructible key;
- state defaults or bounds outside their domain;
- enabled handlers absent from the compiled registry.

Potential runtime cycles between events are allowed only under explicit budgets and require focused tests.

## Required tests

- expression arithmetic covers domain and rounding boundaries;
- selectors return identical order from differently ordered input collections;
- every once scope distinguishes and coalesces the intended event set;
- event causes retain owner/actor/applier/source distinctions through nested triggers;
- invalid programs fail during catalog construction;
- native handlers and equivalent IR fixtures emit the same normal operation/event shape;
- reaction-budget exhaustion enters the documented fault path.
