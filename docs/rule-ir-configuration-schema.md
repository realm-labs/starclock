# Typed Rule IR configuration schema

Goal 01 batch `G01-P1-B8` freezes the Sora 0.3.0 transport contracts used to
author effects, modifiers and typed rule programs. Sora rows are not executable
runtime objects: `starclock-data` validates and converts them into immutable
domain definitions, and `starclock-combat` later evaluates those definitions
through the resolver operation boundary.

## Modules

- `config/schema/rule.toml` owns Battle/Activity rule domains, generic source
  attribution, state slots, event patterns, filters, triggers, once scopes and
  native-handler audit metadata.
- `config/schema/selector.toml` owns ordered selector definitions and their
  typed predicates.
- `config/schema/expression.toml` owns closed value-expression and condition
  node unions.
- `config/schema/operation.toml` owns finite structured programs, the closed
  battle-operation union and replacement proposals.
- `config/schema/effect.toml` owns effect classification, duration, stacking,
  teardown, tags, granted abilities and rule/modifier bindings.
- `config/schema/modifier.toml` owns staged stat/formula modifiers, stacking
  groups, filters and explicit snapshot captures.

These modules use typed Sora references and positive `i32` transport IDs.
Canonical fractional values remain strings ending in `_decimal`; no schema or
fixture introduces `f32`, `f64`, arbitrary JSON, field-path mutation, scripts or
function names loaded from Excel.

## Relational expression graphs

Expressions and conditions are map tables whose payload is a closed tagged
union. Child operands are references to other rows, so recursive syntax never
appears inside one Sora value. `ValueExpressionNode` supports typed literals,
slot/resource/stat/event reads, ordered selector count/sum, checked arithmetic,
min/max/clamp/negate, conditional choice and explicit conversions with rounding.
`ConditionExpressionNode` supports comparisons, boolean composition, tags,
life/presence, resource bounds, effect/weakness/broken checks, selector
cardinality and event predicates.

The graph is finite only when its references are acyclic. Sora proves that every
referenced row exists; the B8 golden verifier additionally proves the
representative graph is acyclic and rejects an injected cycle. Production
type/domain inference, illegal scope reads and complete static cycle analysis
belong to catalog construction in `G01-P1-B11`.

## Selectors

A selector declares origin, side relationship, life/presence eligibility,
current/event/action reference point, total ordering, cardinality, empty-pool
policy, choice strategy and repeat policy. Ordered predicate rows cover
formation ranges, marks, weaknesses, effects, tags, ownership and stat
comparisons. RNG choices require a purpose key, stable candidate order and an
optional per-candidate weight expression; domain validation rejects an RNG
selector without those contracts.

Every result is an ordered vector. The schema never relies on workbook row
order, map iteration or an implicit nearest/random fallback.

## Rules, slots and triggers

`RuleDefinition` retains a generic source identity/class/tag set and source
digest. The domain is `Battle` or `Activity`; this batch authors only Battle
program operations. B9 adds the minimum generic Activity handoff and its legal
operation vocabulary. It does not create challenge or universe semantics.

A `StateSlot` declares its value kind, owner scope, initial/minimum/maximum
expressions, visibility, persistence and ordered reset points. Battle rules may
own only Battle/Wave/Turn/Action/Hit slots; Activity rules may own only
Activity/Section/Node/Attempt slots. Cross-boundary values require declared
bindings rather than a cross-scope write.

Triggers independently declare event pattern, phase, indexed filter, contextual
condition, once scope, priority and program. Event patterns are a typed union of
event families; the split also stays below Excel's 255-character inline
validation-list limit under Sora 0.3.0. Event filters keep owner, actor, applier,
target and source distinct instead of inferring one cause field from another.

## Programs and operations

Programs are ordered child rows containing one of three step shapes:

- an operation reference;
- `If`, with a condition and typed child programs;
- `ForEach`, with a selector, body program and a mandatory maximum of at most 64
  iterations.

Program calls and unbounded loops do not exist. Catalog validation rejects
recursive child-program graphs and domain crossings. Replacement-phase triggers
may reach only `ProposeReplacement` operations; the golden includes a valid
proposal and rejects an ordinary mutation substituted into that program.

The closed Battle operation union covers damage/true damage, healing, shields,
HP consumption/redirection, Toughness layers/Break/Super Break, effect
application and lifetime changes, resources and slots, timeline/action queues,
summon/despawn/transform/ability replacement, fields/presence, weakness and RES
overrides, typed decisions/events, encounter transitions, replacement proposals
and static native-handler invocation. Each operation separately declares target,
condition, empty-target behavior, snapshot boundary and rollback/commit-fault
policy. Programs request resolver operations; they never mutate collections,
HP or state directly.

## Effects, modifiers and snapshots

Effects explicitly declare category, dispel class, stack limit, duration clock,
tick phase, refresh/independence policy, magnitude comparator, snapshot policy,
teardown and application priority. Ordered bindings attach tags, modifiers,
rules and granted abilities. Resistible/fixed/guaranteed chance is part of the
`ApplyEffect` operation so one independently worded application owns one
declared RNG purpose.

Modifiers declare owner and subject selectors, source rule/effect, stat,
formula purpose/stage, typed value expression/domain, one stacking group,
priority, cap/floor stage, snapshot policy, duration scope and ordered filters.
Stacking groups select exactly one aggregation policy. `SnapshotCapture` rows
name each captured source/target/event/action stat, resource, slot or expression;
fields not captured remain dynamic.

## Static native-handler boundary

`NativeHandler` rows contain a stable key, domain/version, argument-schema
digest, determinism note, owner, written IR-insufficiency reason, removal
condition and enabled flag. The synthetic fixture keeps its handler disabled and
unreachable. Production loading must reject an enabled row absent from the
compiled static registry. A handler receives read-only context and returns the
same typed operations or replacement proposal as authored IR; the schema does
not authorize dynamic libraries or runtime scripts.

## Golden and production boundaries

`config/schema-fixtures/rule-ir` composes the prior disabled character/build
fixture with a B8-only overlay in an isolated cache directory. Its verifier runs
Sora check/build/schema lock, Excel templates, Rust codegen, binary and
diagnostic exports, compares direct/configured outputs, rebuilds for drift,
checks typed references, ordered programs, domains, cycles and replacement
rules, and commits only a manifest of generated hashes. Every composed identity
is a disabled `ProjectFixture`, the native handler is disabled, and the rows do
not change coverage.

Production Rule IR remains `.xlsx`-authored and Sora-exported through the B10
pipeline. B11 owns
whole-catalog type inference, program reachability, scope/domain enforcement,
registered tags/RNG purposes/handlers, stack/comparator rules and conversion to
domain definitions. Debug JSON remains review evidence only and is never a
runtime path.
