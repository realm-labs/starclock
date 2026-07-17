# Modifier and Snapshot Pipeline

This document defines how authored stats and conditional modifiers become formula inputs. It applies equally to character kits, equipment, enemies, encounters, and universe-mode rules.

## Stat query

A `StatQuery` contains subject, stat, source/cause context, target context, ability and damage tags, element, timing snapshot, and formula purpose. A query without context is valid only for a stat whose modifiers cannot filter on those fields.

The base stat pipeline is:

```text
base = curve(level, promotion) + BaseAdd
percented = base * (1 + PercentOfBase)
flat = percented + Flat
final_added = flat + FinalAdd
result = final_added * product(FinalMultiply groups)
```

The exact stages used by each stat are schema metadata. HP, ATK, DEF, and SPD use the full pipeline. Ratios such as CRIT Rate may start at an authored base and skip `PercentOfBase`; the compiler must not guess.

## Modifier definition

Each modifier declares:

- stable definition and instance IDs;
- source rule/effect, owner, applier, and subject selector;
- stat or formula stage being modified;
- value expression and value domain;
- ability, damage, element, action-kind, life/presence, source, and target filters;
- stacking group and aggregation policy;
- priority and stable tie-break ID;
- cap/floor and the stage where it applies;
- snapshot policy and duration scope.

Modifiers are immutable definitions plus runtime instances. The same definition may produce several independent instances when its stacking policy permits.

## Aggregation and stacking

The initial aggregation policies are Sum, Product, Maximum, Minimum, Latest, Earliest, StrongestByComparator, UniquePerSource, and ReplaceGroup. A group defines exactly one policy; content cannot mix Sum and Maximum within the same group.

Evaluation order is:

1. collect applicable instances;
2. sort by formula stage, priority, source ID, instance sequence;
3. partition by stacking group;
4. aggregate each group using its declared policy;
5. combine groups according to the stage rule;
6. apply the stage cap/floor;
7. continue to the next formula stage.

An inactive weaker `StrongestByComparator` instance remains present and can become active if the stronger instance expires. Comparators must define all relevant fields and a stable tie-break.

## Damage and sustain contexts

Formula modifiers do not masquerade as generic stats. `DamageContext` gathers distinct stages for:

- base coefficient/scaling and flat base additions;
- CRIT eligibility, forced result, rate, and damage;
- ordinary DMG Boost by element/ability/damage tag;
- Weaken/outgoing reduction;
- DEF reduction/ignore and attacker level inputs;
- RES and RES penetration;
- vulnerability/incoming damage;
- mitigation/damage taken reduction;
- broken multiplier;
- Break, Super Break, DoT, additional, joint, Elation, and true-damage-specific stages.

Healing and shields use their own contexts for base values, outgoing/incoming healing, shield modifiers, received modifiers, and stacking behavior. Reusing an ordinary DMG stage requires an explicit tag contract, not a coincidentally similar formula.

## Caps and normalization

Caps belong to a named formula stage. Examples include final probability clamping, stat-domain minimums, resistance bounds where supported by the selected rules revision, and HP/resource legal ranges. Do not clamp each contributing modifier individually unless the rule says so.

Negative values are permitted only for declared domains. A negative SPD, maximum HP, weight, shield capacity, or probability threshold is a validation/fault condition rather than a convenient clamp.

## Snapshot policies

Every delayed value binding chooses one policy:

- `Dynamic`: query all inputs at trigger/tick time;
- `OnApplication`: capture selected source/target/formula values when the instance is applied;
- `OnActionStart`, `OnPhaseStart`, or `OnHitStart`;
- `SourceSnapshotTargetDynamic`;
- `SourceDynamicTargetSnapshot`;
- `RecomputeOnStackChange` with a declared captured field set;
- `ExplicitFields`, listing each captured value and boundary.

Snapshots store domain values and source revisions, not references to mutable stat blocks. A modifier affecting a value after its snapshot boundary cannot retroactively change that captured value.

Fields not listed by the policy remain dynamic. Content import must not assume that an entire DoT, shield, summon, field, or delayed attack snapshots merely because one coefficient is captured.

## Query dependency and cycle detection

Stat/value queries carry a stack of `(subject, query kind, context key)`. Re-entering the same key before completion is a cycle. Catalog validation rejects statically visible cycles; runtime conditional cycles become a stable `StatQueryCycle` fault containing the ordered key path.

Cached results include every context field that can affect applicability and the relevant mutation revisions. Caches are excluded from canonical state hashes and may be dropped without changing behavior.

## Ownership and attribution

The modifier source, effect owner, applier, queried subject, damage actor, and target remain distinct. This is necessary for borrowed buffs, fields that persist after an owner leaves, summons scaling from owners, joint attacks, and mode effects supplied by the run rather than a unit.

When an owner leaves or transforms, the effect definition's teardown policy decides whether its modifier is removed, transferred, frozen at snapshot, or persists under a team/mode owner.

## Validation and tests

- every modifier targets a legal stat/formula stage and value domain;
- each stacking group has one comparator/aggregation policy;
- filters reference registered tags and constructible contexts;
- caps and rounding boundaries are explicit;
- snapshots list valid fields and scopes;
- synthetic tests cover every base-stat layer and formula-specific stage;
- strongest-wins fallback, independent sources, expiry, and replacement are tested;
- static and runtime cycles are detected deterministically;
- cache-enabled and cache-disabled runs produce identical events and hashes.
