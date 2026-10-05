//! Explicit, replaceable death-conversion bridge, not production equipment admission.
//!
//! Embedders may use the authored base-policy compiler or supply an explicit
//! immutable expression/digest. The normal Activity assembly must continue to
//! reject the pending Curio until ATK and complete equipment admission exist.

use crate::{
    digest::CanonicalDigestBuilder,
    divergent_universe::{
        DivergentUniverseBattleAssemblyError,
        battle_passive_bindings::{PassiveBindings, bind_passives},
        contribution_snapshot::DivergentUniverseDifficultyProtocolSnapshot,
    },
};
use starclock_combat::{
    CauseActorKind, DispelCategory, DurationClock, EffectCategory, EffectDefinitionId,
    EffectRuntimeTemplate, EffectStackPolicy, EffectTeardownPolicy, EffectTickPhase,
    ParticipantSpec, ProgramId, Rounding, RuleBundleId, RuleId, Scalar, SelectorId,
    SourceDefinitionId, StateSlotDefinitionId, TeamSide, TriggerId,
    catalog::{
        action::AbilityTag,
        builder::CombatCatalogBuilder,
        definition::{
            EffectDefinition, ProgramDefinition, RuleBundle, RuleDefinition, SelectorDefinition,
        },
        selector::{
            RuleEmptyPoolPolicy, RuleLifePredicate, RulePresencePredicate, RuleSelectorChoice,
            RuleSelectorOrdering, RuleSelectorOrigin, RuleSelectorPredicate, RuleSelectorReference,
            RuleSelectorSide, RuleUnitSelector,
        },
    },
    modifier::model::StatQuerySubject,
    rule::model::{
        BattleRuleDefinition, BattleRuleScope, Comparison, ConditionExpr, EventFilter,
        EventValueProperty, OnceScope, ProgramStep, ReactionPriority, RuleEffectChancePolicy,
        RuleEventPoint, RuleOperationTemplate, RuleSource, RuleValue, RuleValueKind, SourceClass,
        StateSlotDef, TriggerDef, TriggerPhase, ValueExpr,
    },
};
use starclock_data::divergent_universe_decisions::weighted_curio_overflows::WeightedCurioOverflowDefinition;

pub mod attack_increase;
mod base_damage;

/// Fail-closed construction errors for the separately authored base policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OverflowBaseDamagePolicyError {
    InvalidDefinition,
    InvalidProtocol,
    Arithmetic,
}

impl core::fmt::Display for OverflowBaseDamagePolicyError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "overflow base damage policy error: {self:?}")
    }
}

impl std::error::Error for OverflowBaseDamagePolicyError {}

/// Explicit base-DMG input for the bridge. Neither an expression nor its digest
/// establishes source parity or admits a pending production workbook row.
#[derive(Clone, Debug)]
pub struct OverflowBaseDamagePolicy {
    expression: ValueExpr,
    identity: [u8; 32],
}

impl OverflowBaseDamagePolicy {
    /// Compile the authored Group-1/Protocol-HP policy, not observed source parity.
    /// Reads the selected conversion target's own checked level (1..=95) through
    /// native Rule IR. Both multiplications floor to six fractional places;
    /// final integral rounding belongs to the shared TrueDamage operation.
    /// Binds the full curve, policy notes and immutable Protocol snapshot into
    /// identity. Does not read live Activity state or admit equipment. Invalid
    /// definitions, negative/overflowing Protocol scaling and any level whose
    /// base multiplications cannot fit are rejected before catalog construction.
    pub fn from_authored(
        definition: &WeightedCurioOverflowDefinition,
        protocol: &DivergentUniverseDifficultyProtocolSnapshot,
    ) -> Result<Self, OverflowBaseDamagePolicyError> {
        base_damage::compile_authored(&definition.base, protocol)
    }

    /// The expression is evaluated once for the selected conversion target as
    /// `CurrentTarget`. It must yield a nonnegative Scalar; the shared catalog
    /// validator and command fault policy handle invalid types/reads/arithmetic.
    /// Identity binds the caller's immutable policy and external inputs.
    #[must_use]
    pub fn new(expression: ValueExpr, identity: [u8; 32]) -> Self {
        Self {
            expression,
            identity,
        }
    }
}

/// Compile the death-conversion contribution for one caller-proven eligible
/// original Hunt/Erudition roster member, preserving existing passive bindings.
///
/// This is a catalog construction API, not an Activity equipment bypass. The
/// caller owns Path proof and the base formula; production battle admission
/// remains fail-closed until its Sora policy is promoted. Formation must be 0..=3.
/// Admission follows the original roster identity in Present or Transformed
/// presence, not its initial form. Linked units cannot inherit that identity.
/// Reusing a formation collides in the catalog and rejects normal construction.
/// Mutates only the caller's catalog builder, never a live battle/Activity.
/// On a construction error, discard the builder; partial definitions may have
/// been appended before passive-binding validation fails.
///
/// Policy: original actor/applier Ability attacks only; marks persist from
/// attack start until conversion or attack end. Damage observations contribute
/// exact post-shield overflow only for actually defeated marked enemies. A
/// separate UnitDefeated event confirms death. HitEnded converts after all hit
/// settlements to one living marked maximum-current-HP enemy, random among all
/// exact ties. The old accumulator is captured before reset and TrueDamage;
/// converted kills retain native credit but cannot feed this collector.
///
/// State is actor-local, not a second action stack: a nested same-owner attack
/// start resets it, its end clears it, and an outer resumed HitStarted reopens
/// observation. No claim is made about source reentrant callback semantics.
pub fn bind_death_conversion_policy(
    builder: &mut CombatCatalogBuilder,
    definition: &WeightedCurioOverflowDefinition,
    player: &ParticipantSpec,
    base: &OverflowBaseDamagePolicy,
    assembly_digest: [u8; 32],
) -> Result<ParticipantSpec, DivergentUniverseBattleAssemblyError> {
    if player.side() != TeamSide::Player || player.formation().get() > 3 {
        return Err(invalid());
    }
    let ordinal = u32::from(player.formation().get()) + 1;
    let raw = |base: u32, offset: u32| {
        ordinal
            .checked_mul(32)
            .and_then(|n| n.checked_add(offset))
            .and_then(|n| n.checked_add(base))
            .ok_or_else(invalid)
    };
    let selector = |offset| SelectorId::new(raw(0x7f00_0000, offset)?).ok_or_else(invalid);
    let program = |offset| ProgramId::new(raw(0x7f01_0000, offset)?).ok_or_else(invalid);
    let rule = RuleId::new(raw(0x7f02_0000, 0)?).ok_or_else(invalid)?;
    let bundle = RuleBundleId::new(raw(0x7f03_0000, 0)?).ok_or_else(invalid)?;
    let source_id = SourceDefinitionId::new(raw(0x7f04_0000, 0)?).ok_or_else(invalid)?;
    let mark = EffectDefinitionId::new(raw(0x7f05_0000, 0)?).ok_or_else(invalid)?;
    let ready = EffectDefinitionId::new(raw(0x7f05_0000, 1)?).ok_or_else(invalid)?;
    let active = StateSlotDefinitionId::new(raw(0x7f06_0000, 0)?).ok_or_else(invalid)?;
    let total = StateSlotDefinitionId::new(raw(0x7f06_0000, 1)?).ok_or_else(invalid)?;
    let death = StateSlotDefinitionId::new(raw(0x7f06_0000, 2)?).ok_or_else(invalid)?;
    let owner = selector(0)?;
    let team = selector(1)?;
    let linked = selector(2)?;
    let all_enemies = selector(3)?;
    let hit_targets = selector(4)?;
    let dead_target = selector(5)?;
    let highest = selector(6)?;
    let current = selector(7)?;
    let marked_alive = selector(8)?;
    let owner_present = selector(9)?;
    let owner_transformed = selector(10)?;
    let all = select(
        RuleSelectorOrigin::Team,
        RuleSelectorSide::Same,
        RuleLifePredicate::Any,
        RulePresencePredicate::Any,
        u16::MAX,
        RuleSelectorChoice::All,
        None,
    )?;
    builder.add_selector(SelectorDefinition::new(team).with_rule_units(all.clone()));
    builder.add_selector(
        SelectorDefinition::new(linked)
            .with_rule_units(all.with_predicates(vec![RuleSelectorPredicate::OwnedBy(team)])),
    );
    for (id, presence) in [
        (owner_present, RulePresencePredicate::Present),
        (owner_transformed, RulePresencePredicate::Transformed),
    ] {
        builder.add_selector(SelectorDefinition::new(id).with_rule_units(select(
            RuleSelectorOrigin::Owner,
            RuleSelectorSide::Same,
            RuleLifePredicate::Any,
            presence,
            1,
            RuleSelectorChoice::First,
            None,
        )?));
    }
    builder.add_selector(
        SelectorDefinition::new(owner).with_rule_units(
            select(
                RuleSelectorOrigin::Owner,
                RuleSelectorSide::Same,
                RuleLifePredicate::Any,
                RulePresencePredicate::Any,
                1,
                RuleSelectorChoice::First,
                None,
            )?
            .with_candidate_union(vec![owner_present, owner_transformed])
            .ok_or_else(invalid)?
            .with_predicates(vec![
                RuleSelectorPredicate::FormationRange {
                    minimum: player.formation().get(),
                    maximum: player.formation().get(),
                },
                RuleSelectorPredicate::Excludes(linked),
            ]),
        ),
    );
    for (id, origin, life, presence, choice, maximum, purpose, predicates) in [
        (
            all_enemies,
            RuleSelectorOrigin::Encounter,
            RuleLifePredicate::Any,
            RulePresencePredicate::Any,
            RuleSelectorChoice::All,
            u16::MAX,
            None,
            vec![],
        ),
        (
            hit_targets,
            RuleSelectorOrigin::EventTargets,
            RuleLifePredicate::Alive,
            RulePresencePredicate::Present,
            RuleSelectorChoice::All,
            64,
            None,
            vec![],
        ),
        (
            dead_target,
            RuleSelectorOrigin::EventTargets,
            RuleLifePredicate::Defeated,
            RulePresencePredicate::Any,
            RuleSelectorChoice::All,
            64,
            None,
            vec![RuleSelectorPredicate::HasMark(mark)],
        ),
        (
            highest,
            RuleSelectorOrigin::Encounter,
            RuleLifePredicate::Alive,
            RulePresencePredicate::Present,
            RuleSelectorChoice::RngUniform,
            1,
            Some("damage-target"),
            vec![
                RuleSelectorPredicate::HasMark(mark),
                RuleSelectorPredicate::HasEffect(ready),
                RuleSelectorPredicate::MaximumValue(ValueExpr::QueryHp {
                    subject: StatQuerySubject::CurrentTarget,
                }),
            ],
        ),
        (
            marked_alive,
            RuleSelectorOrigin::Encounter,
            RuleLifePredicate::Alive,
            RulePresencePredicate::Present,
            RuleSelectorChoice::All,
            64,
            None,
            vec![RuleSelectorPredicate::HasMark(mark)],
        ),
        (
            current,
            RuleSelectorOrigin::CurrentSubject,
            RuleLifePredicate::Alive,
            RulePresencePredicate::Present,
            RuleSelectorChoice::First,
            1,
            None,
            vec![],
        ),
    ] {
        builder.add_selector(
            SelectorDefinition::new(id).with_rule_units(
                select(
                    origin,
                    RuleSelectorSide::Opposing,
                    life,
                    presence,
                    maximum,
                    choice,
                    purpose,
                )?
                .with_predicates(predicates),
            ),
        );
    }
    let selectors = vec![
        owner,
        team,
        linked,
        all_enemies,
        hit_targets,
        dead_target,
        highest,
        current,
        marked_alive,
        owner_present,
        owner_transformed,
    ];
    // Readiness is implementation state, not a second gameplay mark/debuff.
    for (effect, category) in [
        (mark, EffectCategory::Mark),
        (ready, EffectCategory::NeutralState),
    ] {
        builder.add_effect(
            EffectDefinition::new(effect, vec![], vec![]).with_runtime_template(
                EffectRuntimeTemplate::new(
                    category,
                    DispelCategory::NonDispellable,
                    1,
                    None,
                    DurationClock::Permanent,
                    EffectTickPhase::None,
                    EffectStackPolicy::Replace,
                )
                .ok_or_else(invalid)?
                .with_teardown(EffectTeardownPolicy::PersistByScope),
            ),
        );
    }
    let start = program(0)?;
    let mark_hit = program(1)?;
    let collect = program(2)?;
    let confirm = program(3)?;
    let convert = program(4)?;
    let damage = program(5)?;
    let end = program(6)?;
    let clear = || {
        op(RuleOperationTemplate::RemoveEffect {
            selector: all_enemies,
            effect: mark,
        })
    };
    let reset = || vec![set(total, zero()), set(death, boolean(false))];
    let clear_ready = || {
        op(RuleOperationTemplate::RemoveEffect {
            selector: all_enemies,
            effect: ready,
        })
    };
    let mut start_steps = reset();
    start_steps.extend([clear(), clear_ready(), set(active, boolean(true))]);
    let mut end_steps = reset();
    end_steps.extend([clear(), clear_ready(), set(active, boolean(false))]);
    let mut convert_steps = reset();
    convert_steps.extend([
        ProgramStep::ForEach {
            selector: highest,
            body: damage,
            maximum: 1,
        },
        clear(),
        clear_ready(),
    ]);
    for (id, calls, references, steps) in [
        (start, vec![], vec![owner, all_enemies], start_steps),
        (
            mark_hit,
            vec![],
            vec![owner, hit_targets],
            vec![
                set(active, boolean(true)),
                op(RuleOperationTemplate::ApplyEffect {
                    selector: hit_targets,
                    effect: mark,
                    stacks: ValueExpr::Literal(RuleValue::Integer(1)),
                    chance: RuleEffectChancePolicy::Guaranteed,
                    base_chance: None,
                    rng_purpose: None,
                }),
            ],
        ),
        (
            collect,
            vec![],
            vec![owner, dead_target],
            vec![op(RuleOperationTemplate::AddSlot {
                slot: total,
                value: ValueExpr::ReadEventProperty(EventValueProperty::DamageOverflow),
            })],
        ),
        (
            confirm,
            vec![],
            vec![owner, dead_target, marked_alive],
            vec![
                set(death, boolean(true)),
                op(RuleOperationTemplate::ApplyEffect {
                    selector: marked_alive,
                    effect: ready,
                    stacks: ValueExpr::Literal(RuleValue::Integer(1)),
                    chance: RuleEffectChancePolicy::Guaranteed,
                    base_chance: None,
                    rng_purpose: None,
                }),
            ],
        ),
        (
            convert,
            vec![damage],
            vec![owner, all_enemies, highest, current],
            convert_steps,
        ),
        (
            damage,
            vec![],
            vec![current],
            vec![op(RuleOperationTemplate::TrueDamage {
                selector: current,
                amount: ValueExpr::Add(
                    Box::new(fraction(
                        base.expression.clone(),
                        definition.base_multiplier_millionths,
                    )),
                    Box::new(fraction(
                        ValueExpr::Slot(total),
                        definition.overflow_multiplier_millionths,
                    )),
                ),
            })],
        ),
        (end, vec![], vec![owner, all_enemies], end_steps),
    ] {
        let mut references = references;
        references.sort_unstable();
        builder.add_program(
            ProgramDefinition::new(id, calls, references, vec![mark, ready], vec![])
                .with_steps(steps),
        );
    }
    let mut hash = CanonicalDigestBuilder::new();
    hash.update(b"starclock.divergent-universe.overflow.hit-ended-original-roster-present-or-transformed-policy");
    hash.update(assembly_digest);
    hash.update(base.identity);
    hash.update(definition.key.as_bytes());
    hash.update(definition.base_multiplier_millionths.to_le_bytes());
    hash.update(definition.overflow_multiplier_millionths.to_le_bytes());
    let contribution_digest = hash.finalize();
    let source = RuleSource::new(source_id, SourceClass::Mode, vec![], contribution_digest);
    let attack = EventFilter {
        actor_kind: Some(CauseActorKind::Unit),
        actor_selector: Some(owner),
        applier_selector: Some(owner),
        source_class: Some(SourceClass::Ability),
        ability_tag: Some(AbilityTag::Attack),
        has_action: Some(true),
        ..EventFilter::default()
    };
    let mut damage_filter = attack.clone();
    damage_filter.target_selector = Some(dead_target);
    let mut triggers = Vec::new();
    for (offset, point, filter, condition, program) in [
        (
            0,
            RuleEventPoint::ActionStarted,
            attack.clone(),
            ConditionExpr::Literal(true),
            start,
        ),
        (
            1,
            RuleEventPoint::HitStarted,
            attack.clone(),
            ConditionExpr::Literal(true),
            mark_hit,
        ),
        (
            2,
            RuleEventPoint::DamageApplied,
            damage_filter.clone(),
            ConditionExpr::All(
                vec![
                    is_true(active),
                    positive(ValueExpr::ReadEventProperty(
                        EventValueProperty::DamageOverflow,
                    )),
                ]
                .into_boxed_slice(),
            ),
            collect,
        ),
        (
            3,
            RuleEventPoint::UnitDefeated,
            damage_filter,
            ConditionExpr::All(
                vec![is_true(active), positive(ValueExpr::Slot(total))].into_boxed_slice(),
            ),
            confirm,
        ),
        (
            4,
            RuleEventPoint::HitEnded,
            attack.clone(),
            ConditionExpr::All(
                vec![
                    is_true(active),
                    is_true(death),
                    positive(ValueExpr::Slot(total)),
                ]
                .into_boxed_slice(),
            ),
            convert,
        ),
        (
            5,
            RuleEventPoint::ActionResolved,
            attack,
            ConditionExpr::Literal(true),
            end,
        ),
        (
            6,
            RuleEventPoint::WaveEnded,
            EventFilter::default(),
            ConditionExpr::Literal(true),
            end,
        ),
        (
            7,
            RuleEventPoint::BattleWon,
            EventFilter::default(),
            ConditionExpr::Literal(true),
            end,
        ),
        (
            8,
            RuleEventPoint::BattleLost,
            EventFilter::default(),
            ConditionExpr::Literal(true),
            end,
        ),
    ] {
        triggers.push(TriggerDef {
            id: TriggerId::new(raw(0x7f07_0000, offset)?).ok_or_else(invalid)?,
            event: point.kind(),
            event_point: point,
            phase: TriggerPhase::AfterEvent,
            filter,
            condition,
            once_scope: OnceScope::Event,
            priority: ReactionPriority::new(0),
            program,
        });
    }
    builder.add_rule(
        RuleDefinition::new(
            rule,
            vec![start, mark_hit, collect, confirm, convert, damage, end],
            selectors,
        )
        .with_runtime(BattleRuleDefinition::new(
            source.clone(),
            vec![
                StateSlotDef::new(
                    active,
                    RuleValueKind::Boolean,
                    BattleRuleScope::Battle,
                    RuleValue::Boolean(false),
                ),
                StateSlotDef::new(
                    total,
                    RuleValueKind::Scalar,
                    BattleRuleScope::Battle,
                    RuleValue::Scalar(Scalar::ZERO),
                ),
                StateSlotDef::new(
                    death,
                    RuleValueKind::Boolean,
                    BattleRuleScope::Battle,
                    RuleValue::Boolean(false),
                ),
            ],
            triggers,
            None,
        )),
    );
    builder.add_rule_bundle(RuleBundle::new(bundle, vec![rule]));
    bind_passives(
        player,
        &PassiveBindings {
            rule_bundles: vec![bundle],
            sources: vec![source],
            ..PassiveBindings::default()
        },
        contribution_digest,
    )
}

fn select(
    origin: RuleSelectorOrigin,
    side: RuleSelectorSide,
    life: RuleLifePredicate,
    presence: RulePresencePredicate,
    maximum: u16,
    choice: RuleSelectorChoice,
    purpose: Option<&str>,
) -> Result<RuleUnitSelector, DivergentUniverseBattleAssemblyError> {
    RuleUnitSelector::new(
        origin,
        side,
        life,
        presence,
        RuleSelectorReference::CurrentState,
        RuleSelectorOrdering::Formation,
        0,
        maximum,
        RuleEmptyPoolPolicy::NoOp,
        choice,
        purpose.map(Into::into),
        false,
    )
    .ok_or_else(invalid)
}
fn op(operation: RuleOperationTemplate) -> ProgramStep {
    ProgramStep::Operation(operation)
}
fn set(slot: StateSlotDefinitionId, value: ValueExpr) -> ProgramStep {
    op(RuleOperationTemplate::SetSlot { slot, value })
}
fn boolean(value: bool) -> ValueExpr {
    ValueExpr::Literal(RuleValue::Boolean(value))
}
fn zero() -> ValueExpr {
    ValueExpr::Literal(RuleValue::Scalar(Scalar::ZERO))
}
fn is_true(slot: StateSlotDefinitionId) -> ConditionExpr {
    ConditionExpr::Compare {
        operator: Comparison::Equal,
        lhs: Box::new(ValueExpr::Slot(slot)),
        rhs: Box::new(boolean(true)),
    }
}
fn positive(value: ValueExpr) -> ConditionExpr {
    ConditionExpr::Compare {
        operator: Comparison::Greater,
        lhs: Box::new(value),
        rhs: Box::new(zero()),
    }
}
fn fraction(value: ValueExpr, millionths: i64) -> ValueExpr {
    ValueExpr::Multiply {
        lhs: Box::new(value),
        rhs: Box::new(ValueExpr::Literal(RuleValue::Scalar(Scalar::from_scaled(
            millionths,
        )))),
        rounding: Rounding::Floor,
    }
}
fn invalid() -> DivergentUniverseBattleAssemblyError {
    DivergentUniverseBattleAssemblyError::InvalidPlayerParticipants
}
