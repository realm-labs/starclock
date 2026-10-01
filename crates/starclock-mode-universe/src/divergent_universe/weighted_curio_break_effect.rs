//! One entry snapshot raises both elements without per-owner compounding.
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
    EffectStackPolicy, EffectTickPhase, ModifierDefinitionId, ModifierStackingGroupId,
    ParticipantSpec, ProgramId, Rounding, RuleBundleId, RuleId, Scalar, SelectorId,
    SourceDefinitionId, StateSlotDefinitionId, TeamSide, TriggerId,
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
    formula::model::CombatElement,
    modifier::model::{
        FormulaPurpose, FormulaStage, ModifierAggregation, ModifierDefinition,
        ModifierStackingGroup, SnapshotPolicy, StatKind, StatQuerySubject,
    },
    rule::model::{
        BattleRuleDefinition, Comparison, ConditionExpr, EventFilter, OnceScope, ProgramStep,
        ReactionPriority, RuleEffectChancePolicy, RuleEventPoint, RuleOperationTemplate,
        RuleSource, RuleValue, SourceClass, TriggerDef, TriggerPhase, ValueExpr,
    },
};
use starclock_data::{
    catalog::SimulationCatalog,
    divergent_universe_decisions::weighted_curio_break_effects::WeightedCurioBreakEffectPolicy,
};
use std::collections::BTreeMap;

impl WeightedCurioRuntime {
    pub(super) fn assemble_break_effects(
        &self,
        builder: &mut CombatCatalogBuilder,
        snapshot: &WeightedCurioSnapshot,
        core: &SimulationCatalog,
        players: &mut [ParticipantSpec],
        assembly_digest: [u8; 32],
    ) -> Result<(), DivergentUniverseBattleAssemblyError> {
        for (index, definition) in self.break_effects.iter().enumerate() {
            if !snapshot.equipped().contains(&definition.weighted_curio) {
                continue;
            }
            let WeightedCurioBreakEffectPolicy::VersionedProjectPolicyEntryHighestTeamBreakEffectCapture = definition.policy;
            let mut forms = BTreeMap::new();
            for player in &*players {
                if player.side() != TeamSide::Player
                    || core
                        .build_catalog()
                        .character(player.combatant().form())
                        .is_none()
                {
                    return Err(invalid());
                }
                let eligible = matches!(
                    basic_element(core, player)?,
                    CombatElement::Wind | CombatElement::Lightning
                );
                forms.insert(player.combatant().form(), eligible);
            }
            if forms.values().all(|eligible| !eligible) {
                continue;
            }
            if forms.len() > 4 {
                return Err(invalid());
            }
            let ordinal = u32::try_from(index)
                .ok()
                .and_then(|n| n.checked_add(1))
                .filter(|n| *n <= 4095)
                .ok_or_else(invalid)?;
            let address = |base: u32| base.checked_add(ordinal).ok_or_else(invalid);
            let program = ProgramId::new(address(0x7ea0_0000)?).ok_or_else(invalid)?;
            let rule = RuleId::new(address(0x7ea1_0000)?).ok_or_else(invalid)?;
            let bundle = RuleBundleId::new(address(0x7ea2_0000)?).ok_or_else(invalid)?;
            let source_id = SourceDefinitionId::new(address(0x7ea3_0000)?).ok_or_else(invalid)?;
            let effect = EffectDefinitionId::new(address(0x7ea5_0000)?).ok_or_else(invalid)?;
            let trigger = TriggerId::new(address(0x7ea6_0000)?).ok_or_else(invalid)?;
            let modifier = ModifierDefinitionId::new(address(0x7ea8_0000)?).ok_or_else(invalid)?;
            let slot = StateSlotDefinitionId::new(address(0x7ea9_0000)?).ok_or_else(invalid)?;
            let group = ModifierStackingGroupId::new(address(0x7eaa_0000)?).ok_or_else(invalid)?;
            let selector = |offset: u32| {
                ordinal
                    .checked_mul(32)
                    .and_then(|n| n.checked_add(offset))
                    .and_then(|n| n.checked_add(0x7ea4_0000))
                    .and_then(SelectorId::new)
                    .ok_or_else(invalid)
            };
            let owner = selector(0)?;
            let members = selector(1)?;
            let captain = selector(2)?;
            let other_owner = selector(3)?;
            let highest = selector(4)?;
            let targets = selector(5)?;
            let mut member_selectors = Vec::new();
            let mut eligible_selectors = Vec::new();
            let mut selectors = vec![owner, members, captain, other_owner, highest, targets];
            for (offset, (form, eligible)) in forms.iter().enumerate() {
                let id = selector(
                    u32::try_from(offset)
                        .ok()
                        .and_then(|n| n.checked_add(8))
                        .ok_or_else(invalid)?,
                )?;
                builder.add_selector(
                    SelectorDefinition::new(id).with_rule_units(
                        select(
                            RuleSelectorOrigin::Team,
                            RuleSelectorOrdering::Formation,
                            RuleSelectorChoice::All,
                            16,
                        )?
                        .with_predicates(vec![RuleSelectorPredicate::UnitForm(*form)]),
                    ),
                );
                member_selectors.push(id);
                selectors.push(id);
                if *eligible {
                    eligible_selectors.push(id);
                }
            }
            for (id, pool, ordering, choice, maximum) in [
                (
                    members,
                    member_selectors,
                    RuleSelectorOrdering::Formation,
                    RuleSelectorChoice::All,
                    16,
                ),
                (
                    captain,
                    vec![members],
                    RuleSelectorOrdering::Formation,
                    RuleSelectorChoice::First,
                    1,
                ),
                (
                    highest,
                    vec![members],
                    RuleSelectorOrdering::StatDescending,
                    RuleSelectorChoice::First,
                    1,
                ),
                (
                    targets,
                    eligible_selectors,
                    RuleSelectorOrdering::Formation,
                    RuleSelectorChoice::All,
                    16,
                ),
            ] {
                let plan = select(RuleSelectorOrigin::Team, ordering, choice, maximum)?
                    .with_candidate_union(pool)
                    .ok_or_else(invalid)?;
                let plan = if id == highest {
                    plan.with_weight(Some(effective_break()))
                } else {
                    plan
                };
                builder.add_selector(SelectorDefinition::new(id).with_rule_units(plan));
            }
            builder.add_selector(SelectorDefinition::new(owner).with_rule_units(select(
                RuleSelectorOrigin::Owner,
                RuleSelectorOrdering::Formation,
                RuleSelectorChoice::First,
                1,
            )?));
            builder.add_selector(
                SelectorDefinition::new(other_owner).with_rule_units(
                    select(
                        RuleSelectorOrigin::Owner,
                        RuleSelectorOrdering::Formation,
                        RuleSelectorChoice::First,
                        1,
                    )?
                    .with_predicates(vec![RuleSelectorPredicate::Excludes(captain)]),
                ),
            );
            builder.add_modifier_group(ModifierStackingGroup {
                id: group,
                aggregation: ModifierAggregation::UniquePerSource,
                comparator: None,
            });
            builder.add_modifier(ModifierDefinition {
                id: modifier,
                stat: StatKind::BreakEffect,
                stage: FormulaStage::Flat,
                purpose: FormulaPurpose::Stat,
                value: ValueExpr::Slot(slot),
                stacking_group: group,
                priority: 0,
                floor: None,
                cap: None,
                cap_stage: FormulaStage::Flat,
                snapshot: SnapshotPolicy::OnApplication,
                source_stack_slot: None,
                filters: Box::new([]),
            });
            let maximum = ValueExpr::SelectorSum {
                selector: highest,
                value: Box::new(effective_break()),
            };
            let delta = ValueExpr::Maximum(
                Box::new(ValueExpr::Subtract(
                    Box::new(ValueExpr::Multiply {
                        lhs: Box::new(maximum),
                        rhs: Box::new(ValueExpr::Literal(RuleValue::Scalar(Scalar::from_scaled(
                            definition.multiplier_millionths,
                        )))),
                        rounding: Rounding::NearestTiesEven,
                    }),
                    Box::new(effective_break()),
                )),
                Box::new(ValueExpr::Literal(RuleValue::Scalar(Scalar::ZERO))),
            );
            builder.add_effect(
                EffectDefinition::new(effect, vec![], vec![modifier])
                    .with_runtime_template(
                        EffectRuntimeTemplate::new(
                            EffectCategory::Buff,
                            DispelCategory::NonDispellable,
                            1,
                            None,
                            DurationClock::Permanent,
                            EffectTickPhase::None,
                            EffectStackPolicy::Replace,
                        )
                        .ok_or_else(invalid)?
                        .with_comparison(Some(delta), 0),
                    )
                    .with_modifier_magnitude_slots(vec![(modifier, slot)]),
            );
            // RuleDefinition/ProgramDefinition reference lists are canonical sets.
            selectors.sort_unstable();
            builder.add_program(
                ProgramDefinition::new(program, vec![], selectors.clone(), vec![effect], vec![])
                    .with_steps(vec![ProgramStep::Operation(
                        RuleOperationTemplate::ApplyEffect {
                            selector: targets,
                            effect,
                            stacks: ValueExpr::Literal(RuleValue::Integer(1)),
                            chance: RuleEffectChancePolicy::Guaranteed,
                            base_chance: None,
                            rng_purpose: None,
                        },
                    )]),
            );
            let mut hash = CanonicalDigestBuilder::new();
            hash.update(b"starclock.divergent-universe.weighted-curio-break-effect-source");
            hash.update(assembly_digest);
            hash.update(definition.key.as_bytes());
            let source = RuleSource::new(source_id, SourceClass::Mode, vec![], hash.finalize());
            builder.add_rule(
                RuleDefinition::new(rule, vec![program], selectors).with_runtime(
                    BattleRuleDefinition::new(
                        source.clone(),
                        vec![],
                        vec![TriggerDef {
                            id: trigger,
                            event: RuleEventPoint::BattleStarted.kind(),
                            event_point: RuleEventPoint::BattleStarted,
                            phase: TriggerPhase::AfterEvent,
                            filter: EventFilter::default(),
                            condition: ConditionExpr::All(
                                vec![
                                    ConditionExpr::SelectorCardinality {
                                        selector: owner,
                                        operator: Comparison::Equal,
                                        count: 1,
                                    },
                                    ConditionExpr::SelectorCardinality {
                                        selector: other_owner,
                                        operator: Comparison::Equal,
                                        count: 0,
                                    },
                                ]
                                .into_boxed_slice(),
                            ),
                            once_scope: OnceScope::Battle,
                            priority: ReactionPriority::new(0),
                            program,
                        }],
                        None,
                    ),
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

fn effective_break() -> ValueExpr {
    ValueExpr::QueryStat {
        subject: StatQuerySubject::CurrentTarget,
        stat: StatKind::BreakEffect,
        purpose: FormulaPurpose::Stat,
    }
}
fn select(
    origin: RuleSelectorOrigin,
    ordering: RuleSelectorOrdering,
    choice: RuleSelectorChoice,
    maximum: u16,
) -> Result<RuleUnitSelector, DivergentUniverseBattleAssemblyError> {
    RuleUnitSelector::new(
        origin,
        RuleSelectorSide::Same,
        RuleLifePredicate::Alive,
        RulePresencePredicate::Present,
        RuleSelectorReference::CurrentState,
        ordering,
        0,
        maximum,
        RuleEmptyPoolPolicy::NoOp,
        choice,
        None,
        false,
    )
    .ok_or_else(invalid)
}
fn invalid() -> DivergentUniverseBattleAssemblyError {
    DivergentUniverseBattleAssemblyError::InvalidPlayerParticipants
}
