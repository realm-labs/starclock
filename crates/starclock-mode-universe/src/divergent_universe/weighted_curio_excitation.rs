//! Effective Skill Point gains and action-local Excitation through shared Rule IR.
use crate::{
    digest::CanonicalDigestBuilder,
    divergent_universe::{
        DivergentUniverseBattleAssemblyError,
        battle_participant_element::basic_element,
        battle_passive_bindings::{PassiveBindings, bind_passives},
        weighted_curio::{WeightedCurioRuntime, WeightedCurioSnapshot},
    },
};
use starclock_combat::{
    DispelCategory, DurationClock, EffectCategory, EffectDefinitionId, EffectRuntimeTemplate,
    EffectStackPolicy, EffectTeardownPolicy, EffectTickPhase, ParticipantSpec, ProgramId, Ratio,
    Rounding, RuleBundleId, RuleId, Scalar, SelectorId, SourceDefinitionId, TeamSide, TriggerId,
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
    formula::{
        model::{CombatElement, DamageClass},
        toughness::BreakDamageDefinition,
    },
    modifier::model::{FormulaPurpose, StatKind, StatQuerySubject},
    rng::types::DrawPurpose,
    rule::model::{
        BattleRuleDefinition, Comparison, ConditionExpr, EventFilter, EventValueProperty,
        OnceScope, ProgramStep, ReactionPriority, RuleActionKind, RuleDamageClass,
        RuleEffectChancePolicy, RuleEventPoint, RuleOperationTemplate, RuleResourceEventKind,
        RuleResourceKind, RuleSource, RuleValue, SourceClass, TriggerDef, TriggerPhase, ValueExpr,
    },
};
use starclock_data::{
    catalog::SimulationCatalog,
    divergent_universe_decisions::weighted_curio_excitations::WeightedCurioExcitationPolicy,
};

impl WeightedCurioRuntime {
    pub(super) fn assemble_excitations(
        &self,
        builder: &mut CombatCatalogBuilder,
        snapshot: &WeightedCurioSnapshot,
        core: &SimulationCatalog,
        players: &mut [ParticipantSpec],
        assembly_digest: [u8; 32],
    ) -> Result<(), DivergentUniverseBattleAssemblyError> {
        for (index, definition) in self.excitations.iter().enumerate() {
            if !snapshot.equipped().contains(&definition.weighted_curio) {
                continue;
            }
            let WeightedCurioExcitationPolicy::VersionedProjectPolicyEffectiveGainActionResolvedTeamConsumption = definition.policy;
            if players.is_empty() || players.len() > 4 {
                return Err(invalid());
            }
            let quantum = players
                .iter()
                .map(|player| {
                    if player.side() != TeamSide::Player {
                        return Err(invalid());
                    }
                    basic_element(core, player).map(|element| element == CombatElement::Quantum)
                })
                .collect::<Result<Vec<_>, _>>()?;
            let ordinal = u32::try_from(index)
                .ok()
                .and_then(|n| n.checked_add(1))
                .filter(|n| *n <= 2047)
                .ok_or_else(invalid)?;
            let address = |base: u32| base.checked_add(ordinal).ok_or_else(invalid);
            let gain = ProgramId::new(address(0x7ed0_0000)?).ok_or_else(invalid)?;
            let consume = ProgramId::new(address(0x7ed1_0000)?).ok_or_else(invalid)?;
            let apply = ProgramId::new(address(0x7ed2_0000)?).ok_or_else(invalid)?;
            let rule = RuleId::new(address(0x7ed3_0000)?).ok_or_else(invalid)?;
            let source_id = SourceDefinitionId::new(address(0x7ed4_0000)?).ok_or_else(invalid)?;
            let stacks = EffectDefinitionId::new(address(0x7ed6_0000)?).ok_or_else(invalid)?;
            let entanglement =
                EffectDefinitionId::new(address(0x7ed7_0000)?).ok_or_else(invalid)?;
            let bundle = RuleBundleId::new(address(0x7ed8_0000)?).ok_or_else(invalid)?;
            let purpose = ordinal
                .checked_add(42_000)
                .and_then(|n| u16::try_from(n).ok())
                .and_then(DrawPurpose::new)
                .ok_or_else(invalid)?;
            let selector = |offset: u32| {
                ordinal
                    .checked_mul(32)
                    .and_then(|n| n.checked_add(offset))
                    .and_then(|n| n.checked_add(0x7ed5_0000))
                    .and_then(SelectorId::new)
                    .ok_or_else(invalid)
            };
            let owner = selector(0)?;
            let recipients = selector(1)?;
            let quantum_owner = selector(2)?;
            let targets = selector(3)?;
            let damaged_target = selector(4)?;
            let not_owner = selector(5)?;
            let team = selector(30)?;
            let linked = selector(31)?;
            let all = select(
                RuleSelectorOrigin::Team,
                RuleSelectorSide::Same,
                RuleLifePredicate::Any,
                RulePresencePredicate::Any,
                u16::MAX,
            )?;
            builder.add_selector(SelectorDefinition::new(team).with_rule_units(all.clone()));
            builder.add_selector(
                SelectorDefinition::new(linked).with_rule_units(
                    all.with_predicates(vec![RuleSelectorPredicate::OwnedBy(team)]),
                ),
            );
            let mut members = Vec::new();
            let mut quantum_members = Vec::new();
            for (offset, (player, eligible)) in players.iter().zip(quantum).enumerate() {
                let member = selector(
                    u32::try_from(offset)
                        .map_err(|_| invalid())?
                        .checked_add(8)
                        .ok_or_else(invalid)?,
                )?;
                let formation = player.formation().get();
                builder.add_selector(
                    SelectorDefinition::new(member).with_rule_units(
                        select(
                            RuleSelectorOrigin::Team,
                            RuleSelectorSide::Same,
                            RuleLifePredicate::Any,
                            RulePresencePredicate::Any,
                            1,
                        )?
                        .with_predicates(vec![
                            RuleSelectorPredicate::UnitForm(player.combatant().form()),
                            RuleSelectorPredicate::FormationRange {
                                minimum: formation,
                                maximum: formation,
                            },
                            RuleSelectorPredicate::Excludes(linked),
                        ]),
                    ),
                );
                members.push(member);
                if eligible {
                    quantum_members.push(member);
                }
            }
            builder.add_selector(
                SelectorDefinition::new(recipients).with_rule_units(
                    select(
                        RuleSelectorOrigin::Team,
                        RuleSelectorSide::Same,
                        RuleLifePredicate::Alive,
                        RulePresencePredicate::Present,
                        4,
                    )?
                    .with_candidate_union(members.clone())
                    .ok_or_else(invalid)?,
                ),
            );
            builder.add_selector(
                SelectorDefinition::new(owner).with_rule_units(
                    select(
                        RuleSelectorOrigin::Owner,
                        RuleSelectorSide::Same,
                        RuleLifePredicate::Any,
                        RulePresencePredicate::Present,
                        1,
                    )?
                    .with_predicates(vec![RuleSelectorPredicate::Excludes(linked)]),
                ),
            );
            builder.add_selector(
                SelectorDefinition::new(not_owner).with_rule_units(
                    select(
                        RuleSelectorOrigin::Team,
                        RuleSelectorSide::Same,
                        RuleLifePredicate::Any,
                        RulePresencePredicate::Any,
                        u16::MAX,
                    )?
                    .with_predicates(vec![RuleSelectorPredicate::Excludes(owner)]),
                ),
            );
            // An empty Quantum roster cannot gain stacks. Avoid an illegal empty
            // candidate union without inventing any eligible original member.
            let mut quantum_selector = select(
                RuleSelectorOrigin::Owner,
                RuleSelectorSide::Same,
                RuleLifePredicate::Any,
                RulePresencePredicate::Present,
                1,
            )?;
            quantum_selector = if quantum_members.is_empty() {
                quantum_selector.with_predicates(vec![RuleSelectorPredicate::Excludes(owner)])
            } else {
                quantum_selector
                    .with_candidate_union(quantum_members)
                    .ok_or_else(invalid)?
                    .with_predicates(vec![RuleSelectorPredicate::Excludes(not_owner)])
            };
            builder.add_selector(
                SelectorDefinition::new(quantum_owner).with_rule_units(quantum_selector),
            );
            for (id, origin) in [
                (targets, RuleSelectorOrigin::EventTargets),
                (damaged_target, RuleSelectorOrigin::PrimaryTarget),
            ] {
                builder.add_selector(SelectorDefinition::new(id).with_rule_units(select(
                    origin,
                    RuleSelectorSide::Opposing,
                    RuleLifePredicate::Alive,
                    RulePresencePredicate::Present,
                    16,
                )?));
            }
            let mut selectors = vec![
                owner,
                recipients,
                quantum_owner,
                targets,
                damaged_target,
                not_owner,
                team,
                linked,
            ];
            selectors.extend(members);
            selectors.sort_unstable();
            let effects = vec![stacks, entanglement];
            builder.add_effect(
                EffectDefinition::new(stacks, vec![], vec![]).with_runtime_template(
                    EffectRuntimeTemplate::new(
                        EffectCategory::Buff,
                        DispelCategory::NonDispellable,
                        definition.policy_maximum_stacks,
                        None,
                        DurationClock::Permanent,
                        EffectTickPhase::None,
                        EffectStackPolicy::RefreshAndAddStacks,
                    )
                    .ok_or_else(invalid)?
                    .with_teardown(EffectTeardownPolicy::PersistByScope),
                ),
            );
            builder.add_effect(
                EffectDefinition::new(entanglement, vec![], vec![]).with_runtime_template(
                    EffectRuntimeTemplate::new(
                        EffectCategory::Control,
                        DispelCategory::CleanseableControl,
                        1,
                        Some(integer(i64::from(definition.duration_turns))),
                        DurationClock::TargetTurnStart,
                        EffectTickPhase::None,
                        EffectStackPolicy::Refresh,
                    )
                    .ok_or_else(invalid)?
                    .with_teardown(EffectTeardownPolicy::PersistByScope)
                    .with_entanglement_from_applier_level(BreakDamageDefinition {
                        attacker_level_multiplier: Scalar::ZERO,
                        ability_multiplier: Ratio::ONE,
                        break_effect: Ratio::ZERO,
                        break_damage_increase: Ratio::ZERO,
                        defense_multiplier: Ratio::ONE,
                        resistance_multiplier: Ratio::ONE,
                        vulnerability_multiplier: Ratio::ONE,
                        mitigation_multiplier: Ratio::ONE,
                        unbroken_multiplier: Ratio::from_scaled(900_000),
                    })
                    .ok_or_else(invalid)?,
                ),
            );
            builder.add_program(
                ProgramDefinition::new(gain, vec![], selectors.clone(), effects.clone(), vec![])
                    .with_steps(vec![ProgramStep::Operation(
                        RuleOperationTemplate::ApplyEffect {
                            selector: recipients,
                            effect: stacks,
                            stacks: integer(i64::from(definition.stack_gain)),
                            chance: RuleEffectChancePolicy::Guaranteed,
                            base_chance: None,
                            rng_purpose: None,
                        },
                    )]),
            );
            builder.add_program(
                ProgramDefinition::new(consume, vec![], selectors.clone(), effects.clone(), vec![])
                    .with_steps(vec![
                        ProgramStep::Operation(RuleOperationTemplate::AdjustEffectStacks {
                            selector: recipients,
                            effect: stacks,
                            delta: integer(-i64::from(definition.stack_consumption)),
                        }),
                        ProgramStep::Operation(RuleOperationTemplate::DamageFromOwner {
                            selector: targets,
                            amount: ValueExpr::Multiply {
                                lhs: Box::new(ValueExpr::QueryStat {
                                    subject: StatQuerySubject::Owner,
                                    stat: StatKind::Atk,
                                    purpose: FormulaPurpose::Stat,
                                }),
                                rhs: Box::new(scalar(definition.attack_multiplier_millionths)),
                                rounding: Rounding::NearestTiesEven,
                            },
                            class: DamageClass::Additional,
                            element: CombatElement::Quantum,
                            can_crit: true,
                            can_defeat: true,
                        }),
                    ]),
            );
            builder.add_program(
                ProgramDefinition::new(apply, vec![], selectors.clone(), effects, vec![])
                    .with_steps(vec![ProgramStep::Operation(
                        RuleOperationTemplate::ApplyEffect {
                            selector: damaged_target,
                            effect: entanglement,
                            stacks: integer(1),
                            chance: RuleEffectChancePolicy::Resistible,
                            base_chance: Some(scalar(definition.base_chance_millionths)),
                            rng_purpose: Some(purpose),
                        },
                    )]),
            );
            let trigger = |offset: u32| {
                ordinal
                    .checked_mul(4)
                    .and_then(|n| n.checked_add(offset))
                    .and_then(|n| n.checked_add(0x7ed9_0000))
                    .and_then(TriggerId::new)
                    .ok_or_else(invalid)
            };
            let mut triggers = vec![TriggerDef {
                id: trigger(0)?,
                event: RuleEventPoint::ResourceChanged.kind(),
                event_point: RuleEventPoint::ResourceChanged,
                phase: TriggerPhase::AfterMutation,
                filter: EventFilter {
                    actor_selector: Some(quantum_owner),
                    resource: Some(RuleResourceKind::SkillPoints),
                    resource_event: Some(RuleResourceEventKind::BalanceChanged),
                    ..EventFilter::default()
                },
                condition: positive(
                    ValueExpr::ReadEventProperty(EventValueProperty::ResourceDelta),
                    scalar(0),
                ),
                once_scope: OnceScope::Event,
                priority: ReactionPriority::new(0),
                program: gain,
            }];
            for (offset, kind) in [RuleActionKind::Basic, RuleActionKind::Skill]
                .into_iter()
                .enumerate()
            {
                triggers.push(TriggerDef {
                    id: trigger(u32::try_from(offset).map_err(|_| invalid())? + 1)?,
                    event: RuleEventPoint::ActionResolved.kind(),
                    event_point: RuleEventPoint::ActionResolved,
                    phase: TriggerPhase::AfterAction,
                    filter: EventFilter {
                        actor_selector: Some(owner),
                        action_kind: Some(kind),
                        ability_tag: Some(AbilityTag::Attack),
                        has_action: Some(true),
                        ..EventFilter::default()
                    },
                    condition: positive(
                        ValueExpr::QueryEffectStacks {
                            subject: StatQuerySubject::Owner,
                            effect: stacks,
                        },
                        integer(0),
                    ),
                    once_scope: OnceScope::Action,
                    priority: ReactionPriority::new(0),
                    program: consume,
                });
            }
            triggers.push(TriggerDef {
                id: trigger(3)?,
                event: RuleEventPoint::DamageApplied.kind(),
                event_point: RuleEventPoint::DamageApplied,
                phase: TriggerPhase::AfterEvent,
                filter: EventFilter {
                    source: Some(source_id),
                    applier_selector: Some(owner),
                    damage_class: Some(RuleDamageClass::Additional),
                    element: Some(CombatElement::Quantum),
                    ..EventFilter::default()
                },
                condition: ConditionExpr::Literal(true),
                once_scope: OnceScope::Event,
                priority: ReactionPriority::new(0),
                program: apply,
            });
            let mut hash = CanonicalDigestBuilder::new();
            hash.update(b"starclock.divergent-universe.weighted-curio-excitation-source");
            hash.update(assembly_digest);
            hash.update(definition.key.as_bytes());
            let source = RuleSource::new(source_id, SourceClass::Mode, vec![], hash.finalize());
            builder.add_rule(
                RuleDefinition::new(rule, vec![gain, consume, apply], selectors).with_runtime(
                    BattleRuleDefinition::new(source.clone(), vec![], triggers, None),
                ),
            );
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
    side: RuleSelectorSide,
    life: RuleLifePredicate,
    presence: RulePresencePredicate,
    maximum: u16,
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
        if origin == RuleSelectorOrigin::Owner {
            RuleSelectorChoice::First
        } else {
            RuleSelectorChoice::All
        },
        None,
        false,
    )
    .ok_or_else(invalid)
}
fn integer(value: i64) -> ValueExpr {
    ValueExpr::Literal(RuleValue::Integer(value))
}
fn scalar(value: i64) -> ValueExpr {
    ValueExpr::Literal(RuleValue::Scalar(Scalar::from_scaled(value)))
}
fn positive(lhs: ValueExpr, rhs: ValueExpr) -> ConditionExpr {
    ConditionExpr::Compare {
        lhs: Box::new(lhs),
        operator: Comparison::Greater,
        rhs: Box::new(rhs),
    }
}
fn invalid() -> DivergentUniverseBattleAssemblyError {
    DivergentUniverseBattleAssemblyError::InvalidPlayerParticipants
}
