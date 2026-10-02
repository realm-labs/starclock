//! Encouragement composes original-owner damage labels and scoped CRIT DMG.
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
    EffectStackPolicy, EffectTeardownPolicy, EffectTickPhase, ModifierDefinitionId,
    ModifierStackingGroupId, ParticipantSpec, ProgramId, RuleBundleId, RuleId, Scalar, SelectorId,
    SourceDefinitionId, TeamSide, TriggerId,
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
    damage::{DamageClassification, DamageProducer, DamageSemantic, DamageSemantics},
    formula::model::DamageClass,
    modifier::model::{
        FormulaPurpose, FormulaStage, ModifierAggregation, ModifierDefinition, ModifierFilter,
        ModifierStackingGroup, SnapshotPolicy, StatKind,
    },
    rule::model::{
        BattleRuleDefinition, Comparison, ConditionExpr, EventFilter, OnceScope, ProgramStep,
        ReactionPriority, RuleEffectChancePolicy, RuleEventPoint, RuleOperationTemplate,
        RuleSource, RuleValue, SourceClass, TriggerDef, TriggerPhase, ValueExpr,
    },
};
use starclock_data::{
    catalog::SimulationCatalog,
    divergent_universe_decisions::weighted_curio_encouragements::WeightedCurioEncouragementPolicy,
};

impl WeightedCurioRuntime {
    pub(super) fn assemble_encouragements(
        &self,
        builder: &mut CombatCatalogBuilder,
        snapshot: &WeightedCurioSnapshot,
        core: &SimulationCatalog,
        players: &mut [ParticipantSpec],
        assembly_digest: [u8; 32],
    ) -> Result<(), DivergentUniverseBattleAssemblyError> {
        for (index, definition) in self.encouragements.iter().enumerate() {
            if !snapshot.equipped().contains(&definition.weighted_curio) {
                continue;
            }
            let WeightedCurioEncouragementPolicy::VersionedProjectPolicyOriginalElationFollowUpDamage = definition.policy;
            if players.is_empty() || players.len() > 4 {
                return Err(invalid());
            }
            let eligible = players
                .iter()
                .map(|player| {
                    if player.side() != TeamSide::Player {
                        return Err(invalid());
                    }
                    core.build_catalog()
                        .character(player.combatant().form())
                        .map(|character| character.path() == CombatPath::Elation)
                        .ok_or_else(invalid)
                })
                .collect::<Result<Vec<_>, _>>()?;
            if !eligible.contains(&true) {
                continue;
            }
            let ordinal = u32::try_from(index)
                .ok()
                .and_then(|n| n.checked_add(1))
                .filter(|n| *n <= 2047)
                .ok_or_else(invalid)?;
            let address = |base: u32| base.checked_add(ordinal).ok_or_else(invalid);
            let program = ProgramId::new(address(0x7ee0_0000)?).ok_or_else(invalid)?;
            let rule = RuleId::new(address(0x7ee1_0000)?).ok_or_else(invalid)?;
            let bundle = RuleBundleId::new(address(0x7ee2_0000)?).ok_or_else(invalid)?;
            let source_id = SourceDefinitionId::new(address(0x7ee3_0000)?).ok_or_else(invalid)?;
            let effect = EffectDefinitionId::new(address(0x7ee5_0000)?).ok_or_else(invalid)?;
            let modifier = ModifierDefinitionId::new(address(0x7ee6_0000)?).ok_or_else(invalid)?;
            let group = ModifierStackingGroupId::new(address(0x7ee7_0000)?).ok_or_else(invalid)?;
            let trigger = TriggerId::new(address(0x7ee8_0000)?).ok_or_else(invalid)?;
            let selector = |offset| {
                ordinal
                    .checked_mul(32)
                    .and_then(|n| n.checked_add(offset))
                    .and_then(|n| n.checked_add(0x7ee4_0000))
                    .and_then(SelectorId::new)
                    .ok_or_else(invalid)
            };
            let owner = selector(0)?;
            let team = selector(30)?;
            let linked = selector(31)?;
            let all = select(RuleSelectorOrigin::Team, u16::MAX, RuleSelectorChoice::All)?;
            builder.add_selector(SelectorDefinition::new(team).with_rule_units(all.clone()));
            builder.add_selector(
                SelectorDefinition::new(linked).with_rule_units(
                    all.with_predicates(vec![RuleSelectorPredicate::OwnedBy(team)]),
                ),
            );
            builder.add_selector(
                SelectorDefinition::new(owner).with_rule_units(
                    select(RuleSelectorOrigin::Owner, 1, RuleSelectorChoice::First)?
                        .with_predicates(vec![RuleSelectorPredicate::Excludes(linked)]),
                ),
            );
            let selectors = vec![owner, team, linked];
            builder.add_modifier_group(ModifierStackingGroup {
                id: group,
                aggregation: ModifierAggregation::UniquePerSource,
                comparator: None,
            });
            builder.add_modifier(ModifierDefinition {
                id: modifier,
                stat: StatKind::CritDamage,
                stage: FormulaStage::Flat,
                purpose: FormulaPurpose::Stat,
                value: ValueExpr::Literal(RuleValue::Scalar(Scalar::from_scaled(
                    definition.follow_up_crit_damage_millionths,
                ))),
                stacking_group: group,
                priority: 0,
                floor: None,
                cap: None,
                cap_stage: FormulaStage::Flat,
                snapshot: SnapshotPolicy::Dynamic,
                source_stack_slot: None,
                filters: vec![
                    ModifierFilter::DamageTag("follow_up".into()),
                    ModifierFilter::DamageProducer(DamageProducer::OriginalUnit),
                ]
                .into_boxed_slice(),
            });
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
                        .with_teardown(EffectTeardownPolicy::PersistByScope),
                    )
                    .with_damage_classifications(vec![DamageClassification {
                        class: DamageClass::Elation,
                        semantics: DamageSemantics::new(DamageSemantic::FollowUp),
                        producer: Some(DamageProducer::OriginalUnit),
                    }]),
            );
            builder.add_program(
                ProgramDefinition::new(program, vec![], selectors.clone(), vec![effect], vec![])
                    .with_steps(vec![ProgramStep::Operation(
                        RuleOperationTemplate::ApplyEffect {
                            selector: owner,
                            effect,
                            stacks: ValueExpr::Literal(RuleValue::Integer(1)),
                            chance: RuleEffectChancePolicy::Guaranteed,
                            base_chance: None,
                            rng_purpose: None,
                        },
                    )]),
            );
            let mut hash = CanonicalDigestBuilder::new();
            hash.update(b"starclock.divergent-universe.weighted-curio-encouragement-source");
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
                            condition: ConditionExpr::SelectorCardinality {
                                selector: owner,
                                operator: Comparison::Equal,
                                count: 1,
                            },
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
            for (player, qualifies) in players.iter_mut().zip(eligible) {
                if qualifies {
                    *player = bind_passives(player, &bindings, assembly_digest)?;
                }
            }
        }
        Ok(())
    }
}

fn select(
    origin: RuleSelectorOrigin,
    maximum: u16,
    choice: RuleSelectorChoice,
) -> Result<RuleUnitSelector, DivergentUniverseBattleAssemblyError> {
    RuleUnitSelector::new(
        origin,
        RuleSelectorSide::Same,
        RuleLifePredicate::Any,
        RulePresencePredicate::Any,
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
fn invalid() -> DivergentUniverseBattleAssemblyError {
    DivergentUniverseBattleAssemblyError::InvalidPlayerParticipants
}
