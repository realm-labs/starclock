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

### Effect magnitude captures

An effect definition may bind its resolved instance magnitude to an attached
modifier-local `Scalar` slot. This is distinct from rule memory and the optional
integer stack-count slot. Bindings are strictly ordered by modifier ID; each
bound modifier belongs to exactly one effect definition and its magnitude slot
must not alias its stack slot. An effect-magnitude modifier cannot be an innate
combatant modifier, including when spawning or replacing a combatant.

Both local slots are populated before attachment snapshot evaluation. Refresh
copies the magnitude retained by the effect store: `Refresh` and
`RefreshAndAddStacks` retain the original magnitude, while a same-source
`IndependentBySource` refresh updates it. `Dynamic` modifiers read the current
local values. `OnApplication` recaptures when the retained magnitude changes,
but preserves its snapshot when that magnitude is unchanged.
`RecomputeOnStackChange` evaluates after both local slots are updated. Other
boundary snapshots retain their declared timing. Replacement creates new
attachments; expiry and removal delete the captures with their attachments.

Magnitude resolution follows the existing immutable Rule IR evaluation input.
Applying an earlier operation within one program does not resnapshot later
expressions in that program. A subsequent independently evaluated program stage
can observe the resulting state. No capture reads mutable Activity/build state
or introduces mode-specific arithmetic.

## Semantic DoT detonation filters

Ordinary effect-store DoTs may declare one `DotFamily`: Burn, Bleed, Shock or
Wind Shear. Family is immutable catalog metadata, independent of damage element,
source identity and arbitrary effect tags. Unclassified Fire damage is not Burn.
A family is legal only on a damaging `Dot` runtime or runtime template. The data
loader lowers the existing `burn`, `bleed`, `shock` and `wind-shear` authoring
tags and rejects duplicate/conflicting family tags or non-damaging declarations.
Other tags keep their independent semantics; no family is inferred from them.

`DotDetonationFilter` intersects an optional family and an excluded effect
definition with the operation's existing required tag. Catalog validation
rejects missing exclusion references in both hit operations and Rule IR. All
instances of the excluded definition are omitted. Filtering occurs before the
existing stable effect-instance order and optional labeled RandomOne draw;
zero or one surviving candidate consumes no RNG. The Rule IR `Filtered`
selection lowers to exactly the same generic operation as authored hit plans.

Detonation retains the original DoT's applier, source definition, stored formula,
stacks and captured base magnitude, while evaluating the existing dynamic
formula context. It applies the declared fraction with nearest-ties-even rounding
then floors final damage. It does not refresh/remove effects or consume stacks
or duration. Unfiltered operations retain their existing behavior.

This selection operates only on ordinary effect instances, not the separate
Break-effect store. It is a shared prerequisite, not complete execution evidence
for any mode mechanic requiring Break Burn participation. The
[native accepted-command corpus](../crates/starclock-test-kit/tests/suites/core/combat/effect_resource_pipeline/dot_family.rs)
covers both hit plans and Rule IR, family/tag/exclusion intersection,
unclassified same-element effects, empty/single/multiple candidate pools,
retention, attribution and fresh deterministic reconstruction. The
[real Sora loader tests](../crates/starclock-data/src/catalog_dot_family_tests.rs)
bind all eleven current production family declarations.

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
