# Selector and Target-Set Runtime Boundary

This document fixes the executable selector contract introduced by
`G07-P1-B2`. The contract applies to character, equipment, enemy and mode
rules. Content code must not bypass it with ID branches or unordered
collection scans.

## Authored source

`Selector.xlsx` and `SelectorPredicate.xlsx` are the authoritative editable
sources. Sora 0.3.0 validates and exports them, and `starclock-data` lowers
their generated rows into private `RuleUnitSelector` values. A populated
predicate table is a supported production table; it is not metadata that may
be discarded during catalog construction.

The selector owns:

- origin, side relationship, life and presence eligibility;
- `CurrentState`, `EventSnapshot` or `ActionSnapshot` reference point;
- ordered predicates and deterministic ordering;
- minimum/maximum cardinality and empty-pool control;
- all/first/primary-plus-adjacent/uniform/weighted choice;
- an RNG purpose and explicit repeated-target policy.

`weight_expression_id` is the per-candidate key for `StatAscending` and
`StatDescending`, and the non-negative integer/fixed-point weight for
`RngWeighted`. Catalog construction rejects either use without an expression.

## Candidate reference points

`CurrentState` reads the authoritative transaction state when the selector is
resolved.

`EventSnapshot` reads the compact battlefield projection captured immediately
when the observed event was emitted. `ActionSnapshot` reads the first
projection captured for that action identity, normally the declared action
envelope before costs and hit mutations. Historical projections retain unit
side, formation, life, presence, HP, timeline gauge, weaknesses, active
effects/tags, active owner links, base stats and modifier instances.

Historical stat expressions therefore resolve against historical bases and
modifiers. Until resource snapshots are part of the state-slot/resource batch,
catalog validation rejects historical selector expressions that read mutable
resources or use current-state life/effect/weakness/broken conditions. This is
a fail-closed boundary, not permission to substitute current state.

Action and event snapshots are transaction scratch. They are derived
deterministically, never serialized as authoritative battle state, and are
allocated only when the immutable catalog contains a historical selector.

## Origins and dynamic subjects

- `Owner` is the unit that owns the active rule instance.
- `Source` is the unit attributed by the observed cause; ability programs bind
  it to their owning unit.
- `Actor`, `Applier` and `PrimaryTarget` retain their distinct cause roles.
- `CurrentSubject` is the exact nested `ForEach` subject and falls back to the
  primary target only outside an iteration.
- `Team` and `Encounter` begin with the complete side-filtered battlefield
  pool.

Selector dependencies, including `OwnedBy` and selector reads inside value
expressions, are resolved in topological order. Results remain sorted by
selector ID for evaluator lookup. Missing dependencies and cycles fail catalog
construction.

## Predicates

Predicate rows execute in authored sequence as an intersection:

- `FormationRange` compares the exact formation index;
- `HasMark` requires the exact effect definition and catalog validation proves
  that definition is a Mark;
- `HasEffect` matches the exact active effect definition;
- `HasWeakness` reads the selected reference-point weakness set;
- `HasTag` matches a content-identity tag on an active effect;
- `OwnedBy` matches an active linked-unit owner selected by another selector;
- `StatCompare` compares the effective candidate stat with a scalar expression.

An evaluation error in a predicate does not turn into a match. Invalid static
types and references are rejected before a battle is created.

## Stable ordering and choice

Formation, timeline gauge, HP ratio, stat expression, observed event target
order and stable unit ID all use explicit total orders. Stable unit ID breaks
ties. `EventOrder` places targets from the observed event first in payload
order, followed by all remaining candidates in stable-ID order.

Uniform and weighted selectors draw through the battle-owned RNG with one of
the registered purposes. A non-repeating selector removes each selected
candidate before the next draw. A repeating selector redraws with replacement;
it does not copy the first result. Empty and all-zero weighted pools consume no
draw. Every raw draw is journaled.

## Empty-pool control

Cardinality is checked after filtering and choice:

- `NoOp` supplies an empty selector result and continues the program;
- `Skip` skips the current program/candidate without committing its once key;
- `CancelRemaining` stops the remaining trigger candidates and phases for the
  observed event;
- `Fault` enters the normal typed rollback/fault policy.

Ability-owned programs have no trigger-candidate chain, so `Skip` and
`CancelRemaining` both cancel that program invocation.

## Verification

The runtime golden proves that an `ActionSnapshot` still selects an enemy that
was alive at action declaration after the hit has defeated it. The same battle
executes a repeated weighted selector and proves its requested cardinality.
Separate goldens distinguish all four empty-pool policies. Catalog tests reject
missing weighted keys, invalid RNG contracts and selector dependency cycles.
The production Sora bundle contains a behavior-neutral formation predicate so
the Excel → Sora → generated reader → domain selector path cannot regress to an
unlowered table.
