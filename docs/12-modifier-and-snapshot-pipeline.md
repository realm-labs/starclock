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

Native `StatKind::Elation` is an independent additive ratio with an explicit
zero neutral base in live, initial-capture and selector-snapshot queries. Its
native modifiers use the staged checked Scalar pipeline; no hidden clamp or
ordinary-DMG-Boost alias is installed. A dedicated native damage operation reads
this property and projects filtered common stages; production intrinsic bases
and Sora operation authoring/lowering remain pending. See the
[Elation calculator and query boundary](combat-elation-formula.md).

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

### Additive damage semantics

`DamageSemantics` is a bounded, canonical per-damage label set, independent of
`AbilityTags`, action kind and calculator family. The current native label is
`FollowUp`. Direct damage authored by a FollowUp-tagged ability carries this
label; the ability's other damage families do not inherit it. An explicit
Ultimate-semantics override suppresses that default. This is a native contract,
not observed released parity for hidden secondary-damage programs.

An executable `EffectDefinition` can add labels for named non-DoT calculator
families through strictly ordered, unique `DamageClassification` entries.
Eligibility uses the formula producer's live effect holder, not its applier,
queried victim or a linked combat unit's owner. Unitless timeline actors retain
the shared formula's explicit owner fallback for statistics, independently of
actual producer eligibility. Optional `DamageProducer` filters distinguish
original units, linked units and unitless timeline actors in classification and
damage modifier queries. Countdown action causes retain their timeline actor;
an owner fallback cannot bypass an original-unit filter. Labels from multiple instances or
definitions form an idempotent union, not a multiplier. The operation-entry
`FormulaInputs` capture determines the labels for all targets of that operation;
later operations requery after application, removal or expiry.

The original class tag remains present. `FollowUp` adds the sorted, unique
`follow_up` damage tag to source, critical and incoming formula contexts without
adding a FollowUp ability tag. Shared/per-target CRIT caches retain only the
decision/draw; CRIT DMG is queried separately for each operation's class and
labels. Noncritical damage does not receive a CRIT DMG stat contribution.
Source-unboosted/True damage and DoT settlement carry no attack labels; Break
and Super Break retain their separate events and formulas.

`DamageEventData.semantics` retains the captured labels through guards, effect
consumption and defeat settlement. `RuleEventFacts.damage_semantics` comes from
that payload, never a later scan of live effects. `EventFilter.damage_semantic`
matches conjunctively with the original class/action/ability filters.
`RuleSelectorPredicate::EventDamageSemantic` gates the candidate pool by the
triggering event's label, not by each candidate's own attacks. No extra action,
hit, event or reaction is generated merely by an additional label.

The current replay payload encodes the label bits immediately after the damage
class. Unknown bits cannot be constructed through the public checked bit-set constructor.
The [accepted-command corpus](../crates/starclock-test-kit/tests/suites/core/combat/effect_resource_pipeline/damage_semantics.rs)
checks mixed classes, critical scopes, holder/applier separation, target factors,
duplicate providers, removal, True damage, guard consumption, filters, selectors
and fresh event/state/RNG reconstruction. Sora classification/filter/predicate
authoring remains unbound; this shared capability alone grants no mode-content
execution credit.

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

## Effect-owned shield capacity adjustments

`QueryEffectShield` reads only instances matching one effect definition on the
queried recipient, using that recipient's absorption policy (largest or checked
sum). It is distinct from total visible `QueryShield`. A known recipient without
matching capacity returns zero; a missing recipient/reader or unrepresentable
capacity is a typed evaluation failure, not a guessed zero. Both queries read
the immutable program-input snapshot. Later ordered mutations in that program
do not change previously evaluated operands. Independently evaluated reaction
programs read their own fresh snapshot, which can include later mutations; event
facts retain the observed event's exact signed capacity delta.

`AdjustEffectShield` declares recipient selector, effect definition, absorption
policy, Increase/Decrease and a nonnegative Scalar amount. Amount finalization
floors once. Adjustment requires that effect to be live on the recipient; a
missing effect or zero amount is a no-op. Exactly one matching positive instance
may be adjusted. Multiple matching instances or a conflicting owner policy fault
with transactional rollback. Increase uses checked addition; Decrease explicitly
removes at most the existing capacity. This is exact capacity arithmetic, without
reapplying creation bonuses or incoming-damage modifiers.

Positive resize preserves shield ID and original source operation. Increase from
zero creates an instance; reaching zero removes it, with a later increase using
a new ID. Adjustment and ordinary absorption never refresh the owning effect's
stacks, duration or captures. `source_effect` remains a definition-based identity,
not an automatic instance-lifetime attachment: authored teardown explicitly emits
`RemoveShield`, separately from effect removal/expiry. Content that needs linked
teardown must supply that operation in its lifecycle program.

Every effective adjustment emits `ShieldEventData::Adjusted`, retaining operation,
shield, recipient, effect, kind, requested amount and before/after capacities in
canonical replay bytes. Typed `RuleShieldEventKind` filters distinguish Applied,
Absorbed, Removed and Adjusted; optional shield-effect filters conjunctively match
the Adjusted payload, not an unrelated effect-lifecycle fact. Exact lost-capacity
healing uses the negative committed delta rather than a later shield query and
the existing unmodified Heal operation. No-op changes emit no shield event.

The [native command corpus](../crates/starclock-test-kit/tests/suites/core/combat/ability_program_execution/effect_shield.rs)
verifies queries, identity, absorption, explicit teardown, rejected commands,
fault rollback, signed-event healing and fresh reconstruction. These capabilities
have no production Sora authoring or Divergent Universe admission credit yet.

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
ordinary instances of the excluded definition are omitted. Filtering occurs before
the stable effect-instance order and optional labeled RandomOne draw;
zero or one surviving candidate consumes no RNG. The Rule IR `Filtered`
selection declares the candidate scope and lowers to exactly the same generic
operation as authored hit plans.

Detonation retains the original DoT's applier, source definition, stored formula,
stacks and captured base magnitude, while evaluating the existing dynamic
formula context. It applies the declared fraction with nearest-ties-even rounding
then floors final damage. It does not refresh/remove effects or consume stacks
or duration. Unfiltered operations retain their existing behavior.

`DotDetonationScope::OrdinaryEffects` selects ordinary instances.
`OrdinaryAndBreakEffects` additionally admits active base Break Bleed, Burn,
Shock and Wind Shear. Both stores share effect-instance IDs; the eligible pool
is merged and sorted by that key before any random choice. The base Break
status's established element semantics determine its family. This never infers
the family of an ordinary effect from its damage element. Ice and Quantum expiry
damage and Imaginary control are not periodic DoTs and remain ineligible.

Base Break statuses have neither authored tags nor an effect-definition ID.
A required tag excludes them rather than treating their source ID as a tag;
an excluded ordinary definition does not exclude them. Break detonation uses
the same captured base and Wind stack rule as natural ticks, the original
applier/source, the dedicated Break formula and its final source multiplier.
Both candidate kinds use the operation-entry formula inputs, apply the fraction
to the complete unfloored result, then floor once for damage settlement.
Guards, shields, HP bounds, damage credit and defeat use the existing shared
settlement. `BreakDamageKind::EffectDetonation` and
`ToughnessEventData::BaseEffectDetonated` distinguish immediate damage from
natural ticks and preserve operation/effect identity, element and fraction in
canonical event bytes. The corresponding typed Toughness filter is available
to Rule IR. Detonation does not consume or reset the base status's lifetime,
stacks, source operation, captured base or control behavior.

These are shared capabilities, not production execution credit for a mode
mechanic that has not yet supplied its complete authored program. The
[native accepted-command corpus](../crates/starclock-test-kit/tests/suites/core/combat/effect_resource_pipeline/dot_family.rs)
covers both hit plans and Rule IR, family/tag/exclusion intersection,
unclassified same-element effects, empty/single/multiple candidate pools,
retention, attribution and fresh deterministic reconstruction. The
[real Sora loader tests](../crates/starclock-data/src/catalog_dot_family_tests.rs)
bind all eleven current production family declarations.
The [cross-store native corpus](../crates/starclock-test-kit/tests/suites/core/combat/effect_resource_pipeline/break_detonation.rs)
covers all four periodic Break statuses, dedicated formulas and original source
ownership, mixed-store ordering, explicit scopes, filters, control exclusion,
random pool boundaries and fresh canonical-event reconstruction through both
hit plans and Rule IR.

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
