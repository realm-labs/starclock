//! Recipient-local special shields use only shared effect, event and capacity operations.

use crate::{
    digest::CanonicalDigestBuilder,
    divergent_universe::{
        DivergentUniverseBattleAssemblyError,
        battle_passive_bindings::{PassiveBindings, bind_passives},
        weighted_curio::{WeightedCurioRuntime, WeightedCurioSnapshot},
    },
};
use starclock_build::light_cone::CombatPath;
use starclock_combat::{
    DispelCategory, DurationClock, EffectCategory, EffectDefinitionId, EffectRuntimeTemplate,
    EffectStackPolicy, EffectTeardownPolicy, EffectTickPhase, ParticipantSpec, ProgramId, Rounding,
    RuleBundleId, RuleId, Scalar, SelectorId, SourceDefinitionId, TeamSide, TriggerId,
    catalog::{
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
    formula::shield::{ShieldAbsorptionPolicy, ShieldAdjustmentKind},
    modifier::model::StatQuerySubject,
    rule::model::{
        BattleRuleDefinition, Comparison, ConditionExpr, EventFilter, EventValueProperty,
        OnceScope, ProgramStep, ReactionPriority, RuleEffectChancePolicy, RuleEventPoint,
        RuleOperationTemplate, RuleShieldEventKind, RuleSource, RuleValue, SourceClass, TriggerDef,
        TriggerPhase, ValueExpr,
    },
};
use starclock_data::{
    catalog::SimulationCatalog,
    divergent_universe_decisions::weighted_curio_transfers::WeightedCurioTransferPolicy,
};

impl WeightedCurioRuntime {
    pub(super) fn assemble_transfers(
        &self,
        builder: &mut CombatCatalogBuilder,
        snapshot: &WeightedCurioSnapshot,
        core: &SimulationCatalog,
        players: &mut [ParticipantSpec],
        assembly_digest: [u8; 32],
    ) -> Result<(), DivergentUniverseBattleAssemblyError> {
        for (index, definition) in self.transfers.iter().enumerate() {
            if !snapshot.equipped().contains(&definition.weighted_curio) {
                continue;
            }
            let WeightedCurioTransferPolicy::VersionedProjectPolicyOtherShieldAppliedOwnerTurnExcessDecay = definition.policy;
            if players.is_empty() || players.len() > 4 {
                return Err(invalid());
            }
            let paths = players
                .iter()
                .map(|player| {
                    if player.side() != TeamSide::Player {
                        return Err(invalid());
                    }
                    core.build_catalog()
                        .character(player.combatant().form())
                        .map(|character| character.path())
                        .ok_or_else(invalid)
                })
                .collect::<Result<Vec<_>, _>>()?;
            if !paths.contains(&CombatPath::Preservation) {
                continue;
            }
            let ordinal = u32::try_from(index)
                .ok()
                .and_then(|n| n.checked_add(1))
                .filter(|n| *n <= 2047)
                .ok_or_else(invalid)?;
            let address = |base: u32| base.checked_add(ordinal).ok_or_else(invalid);
            let initialize = ProgramId::new(address(0x7ef0_0000)?).ok_or_else(invalid)?;
            let gain = ProgramId::new(address(0x7ef1_0000)?).ok_or_else(invalid)?;
            let decay = ProgramId::new(address(0x7ef2_0000)?).ok_or_else(invalid)?;
            let heal = ProgramId::new(address(0x7ef3_0000)?).ok_or_else(invalid)?;
            let cleanup = ProgramId::new(address(0x7ef4_0000)?).ok_or_else(invalid)?;
            let rule = RuleId::new(address(0x7ef5_0000)?).ok_or_else(invalid)?;
            let bundle = RuleBundleId::new(address(0x7ef6_0000)?).ok_or_else(invalid)?;
            let source_id = SourceDefinitionId::new(address(0x7ef7_0000)?).ok_or_else(invalid)?;
            let effect = EffectDefinitionId::new(address(0x7ef8_0000)?).ok_or_else(invalid)?;
            let selector = |offset| {
                ordinal
                    .checked_mul(16)
                    .and_then(|n| n.checked_add(offset))
                    .and_then(|n| n.checked_add(0x7ef9_0000))
                    .and_then(SelectorId::new)
                    .ok_or_else(invalid)
            };
            let owner = selector(0)?;
            let any_owner = selector(1)?;
            let team = selector(14)?;
            let linked = selector(15)?;
            let all = select(
                RuleSelectorOrigin::Team,
                RuleLifePredicate::Any,
                RulePresencePredicate::Any,
                u16::MAX,
                RuleSelectorChoice::All,
            )?;
            builder.add_selector(SelectorDefinition::new(team).with_rule_units(all.clone()));
            builder.add_selector(
                SelectorDefinition::new(linked).with_rule_units(
                    all.with_predicates(vec![RuleSelectorPredicate::OwnedBy(team)]),
                ),
            );
            for (id, life, presence) in [
                (
                    owner,
                    RuleLifePredicate::Alive,
                    RulePresencePredicate::Present,
                ),
                (
                    any_owner,
                    RuleLifePredicate::Any,
                    RulePresencePredicate::Any,
                ),
            ] {
                builder.add_selector(
                    SelectorDefinition::new(id).with_rule_units(
                        select(
                            RuleSelectorOrigin::Owner,
                            life,
                            presence,
                            1,
                            RuleSelectorChoice::First,
                        )?
                        .with_predicates(vec![RuleSelectorPredicate::Excludes(linked)]),
                    ),
                );
            }
            let selectors = vec![owner, any_owner, team, linked];
            builder.add_effect(
                EffectDefinition::new(effect, vec![], vec![]).with_runtime_template(
                    EffectRuntimeTemplate::new(
                        EffectCategory::Shield,
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
            let programs = vec![initialize, gain, decay, heal, cleanup];
            for (id, steps, calls) in [
                (
                    initialize,
                    vec![op(RuleOperationTemplate::ApplyEffect {
                        selector: owner,
                        effect,
                        stacks: ValueExpr::Literal(RuleValue::Integer(1)),
                        chance: RuleEffectChancePolicy::Guaranteed,
                        base_chance: None,
                        rng_purpose: None,
                    })],
                    vec![],
                ),
                (
                    gain,
                    vec![
                        ProgramStep::If {
                            condition: ConditionExpr::Not(Box::new(ConditionExpr::EffectExists {
                                selector: owner,
                                effect,
                            })),
                            then_program: initialize,
                            else_program: None,
                        },
                        op(RuleOperationTemplate::AdjustEffectShield {
                            selector: owner,
                            effect,
                            kind: ShieldAdjustmentKind::Increase,
                            policy: ShieldAbsorptionPolicy::ConcurrentLargest,
                            amount: fraction(delta(), definition.transfer_millionths),
                        }),
                    ],
                    vec![initialize],
                ),
                (
                    decay,
                    vec![op(RuleOperationTemplate::AdjustEffectShield {
                        selector: owner,
                        effect,
                        kind: ShieldAdjustmentKind::Decrease,
                        policy: ShieldAbsorptionPolicy::ConcurrentLargest,
                        amount: fraction(
                            ValueExpr::Maximum(
                                Box::new(ValueExpr::Subtract(
                                    Box::new(ValueExpr::QueryEffectShield {
                                        subject: StatQuerySubject::Owner,
                                        effect,
                                    }),
                                    Box::new(fraction(
                                        ValueExpr::QueryMaximumHp(StatQuerySubject::Owner),
                                        definition.threshold_millionths,
                                    )),
                                )),
                                Box::new(zero()),
                            ),
                            definition.decay_millionths,
                        ),
                    })],
                    vec![],
                ),
                (
                    heal,
                    vec![op(RuleOperationTemplate::Heal {
                        selector: owner,
                        apply_formula_modifiers: false,
                        amount: fraction(
                            ValueExpr::Negate(Box::new(delta())),
                            definition.heal_millionths,
                        ),
                    })],
                    vec![],
                ),
                (
                    cleanup,
                    vec![op(RuleOperationTemplate::RemoveShield {
                        selector: any_owner,
                        effect,
                    })],
                    vec![],
                ),
            ] {
                builder.add_program(
                    ProgramDefinition::new(id, calls, selectors.clone(), vec![effect], vec![])
                        .with_steps(steps),
                );
            }
            let mut hash = CanonicalDigestBuilder::new();
            hash.update(b"starclock.divergent-universe.weighted-curio-transfer-source");
            hash.update(assembly_digest);
            hash.update(definition.key.as_bytes());
            let source = RuleSource::new(source_id, SourceClass::Mode, vec![], hash.finalize());
            let mut triggers = Vec::new();
            for (offset, point, phase, filter, condition, once_scope, priority, program) in [
                (
                    0,
                    RuleEventPoint::BattleStarted,
                    TriggerPhase::AfterEvent,
                    EventFilter::default(),
                    ConditionExpr::Not(Box::new(ConditionExpr::EffectExists {
                        selector: owner,
                        effect,
                    })),
                    OnceScope::Battle,
                    0,
                    initialize,
                ),
                (
                    1,
                    RuleEventPoint::ShieldChanged,
                    TriggerPhase::AfterMutation,
                    EventFilter {
                        target_selector: Some(owner),
                        shield_event: Some(RuleShieldEventKind::Applied),
                        ..EventFilter::default()
                    },
                    ConditionExpr::Compare {
                        operator: Comparison::Greater,
                        lhs: Box::new(delta()),
                        rhs: Box::new(zero()),
                    },
                    OnceScope::Event,
                    0,
                    gain,
                ),
                (
                    2,
                    RuleEventPoint::TurnStarted,
                    TriggerPhase::AfterEvent,
                    EventFilter {
                        actor_selector: Some(owner),
                        ..EventFilter::default()
                    },
                    ConditionExpr::EffectExists {
                        selector: owner,
                        effect,
                    },
                    OnceScope::Turn,
                    0,
                    decay,
                ),
                (
                    3,
                    RuleEventPoint::ShieldChanged,
                    TriggerPhase::AfterMutation,
                    EventFilter {
                        target_selector: Some(owner),
                        shield_event: Some(RuleShieldEventKind::Adjusted),
                        shield_effect: Some(effect),
                        ..EventFilter::default()
                    },
                    ConditionExpr::Compare {
                        operator: Comparison::Less,
                        lhs: Box::new(delta()),
                        rhs: Box::new(zero()),
                    },
                    OnceScope::Event,
                    1,
                    heal,
                ),
                (
                    4,
                    RuleEventPoint::EffectRemoved,
                    TriggerPhase::AfterEvent,
                    EventFilter {
                        target_selector: Some(any_owner),
                        effect_definition: Some(effect),
                        ..EventFilter::default()
                    },
                    ConditionExpr::Not(Box::new(ConditionExpr::EffectExists {
                        selector: any_owner,
                        effect,
                    })),
                    OnceScope::Event,
                    0,
                    cleanup,
                ),
            ] {
                let id = ordinal
                    .checked_mul(8)
                    .and_then(|n| n.checked_add(offset))
                    .and_then(|n| n.checked_add(0x7efa_0000))
                    .and_then(TriggerId::new)
                    .ok_or_else(invalid)?;
                triggers.push(TriggerDef {
                    id,
                    event: point.kind(),
                    event_point: point,
                    phase,
                    filter,
                    condition,
                    once_scope,
                    priority: ReactionPriority::new(priority),
                    program,
                });
            }
            builder.add_rule(RuleDefinition::new(rule, programs, selectors).with_runtime(
                BattleRuleDefinition::new(source.clone(), vec![], triggers, None),
            ));
            builder.add_rule_bundle(RuleBundle::new(bundle, vec![rule]));
            let bindings = PassiveBindings {
                rule_bundles: vec![bundle],
                sources: vec![source],
                ..PassiveBindings::default()
            };
            for player in &mut *players {
                *player = bind_passives(player, &bindings, assembly_digest)?;
            }
        }
        Ok(())
    }
}

fn select(
    origin: RuleSelectorOrigin,
    life: RuleLifePredicate,
    presence: RulePresencePredicate,
    maximum: u16,
    choice: RuleSelectorChoice,
) -> Result<RuleUnitSelector, DivergentUniverseBattleAssemblyError> {
    RuleUnitSelector::new(
        origin,
        RuleSelectorSide::Same,
        life,
        presence,
        RuleSelectorReference::CurrentState,
        RuleSelectorOrdering::Formation,
        0,
        maximum,
        RuleEmptyPoolPolicy::NoOp,
        choice,
        None,
        false,
    )
    .ok_or_else(invalid)
}
fn op(operation: RuleOperationTemplate) -> ProgramStep {
    ProgramStep::Operation(operation)
}
fn delta() -> ValueExpr {
    ValueExpr::ReadEventProperty(EventValueProperty::ShieldChangeAmount)
}
fn zero() -> ValueExpr {
    ValueExpr::Literal(RuleValue::Scalar(Scalar::ZERO))
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
