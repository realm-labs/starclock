//! Sapient Pen composes ordinary keyed-resource and recipient-local effect rules.
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
    ModifierStackingGroupId, ParticipantSpec, ProgramId, Rounding, RuleBundleId, RuleId, Scalar,
    SelectorId, SourceDefinitionId, TeamSide, TriggerId,
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
    modifier::model::{
        FormulaPurpose, FormulaStage, ModifierAggregation, ModifierDefinition,
        ModifierStackingGroup, SnapshotPolicy, StatKind,
    },
    rule::model::{
        BattleRuleDefinition, ConditionExpr, EventFilter, OnceScope, ProgramStep, ReactionPriority,
        ResourceUpdateKind, RuleActionKind, RuleEffectChancePolicy, RuleEventPoint,
        RuleOperationTemplate, RuleResourceKind, RuleSource, RuleValue, SourceClass, TriggerDef,
        TriggerPhase, ValueExpr,
    },
};
use starclock_data::{
    catalog::SimulationCatalog,
    divergent_universe_decisions::weighted_curio_elations::WeightedCurioElationPolicy,
};

impl WeightedCurioRuntime {
    pub(super) fn assemble_elations(
        &self,
        builder: &mut CombatCatalogBuilder,
        snapshot: &WeightedCurioSnapshot,
        core: &SimulationCatalog,
        players: &mut [ParticipantSpec],
        assembly_digest: [u8; 32],
    ) -> Result<(), DivergentUniverseBattleAssemblyError> {
        for (index, definition) in self.elations.iter().enumerate() {
            if !snapshot.equipped().contains(&definition.weighted_curio) {
                continue;
            }
            let WeightedCurioElationPolicy::VersionedProjectPolicyActionResolvedOriginalPartyRefresh = definition.policy;
            // Determine membership before live battle state exists. A defeated
            // original member still satisfies presence; linked actors never
            // receive an owner's bundle by virtue of sharing its Path or form.
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
            if !eligible.contains(&true) || !eligible.contains(&false) {
                continue;
            }
            let ordinal = u32::try_from(index)
                .ok()
                .and_then(|n| n.checked_add(1))
                .filter(|n| *n <= 2047)
                .ok_or_else(invalid)?;
            let address = |base: u32| base.checked_add(ordinal).ok_or_else(invalid);
            let program = ProgramId::new(address(0x7ec0_0000)?).ok_or_else(invalid)?;
            let rule = RuleId::new(address(0x7ec1_0000)?).ok_or_else(invalid)?;
            let bundle = RuleBundleId::new(address(0x7ec2_0000)?).ok_or_else(invalid)?;
            let source_id = SourceDefinitionId::new(address(0x7ec3_0000)?).ok_or_else(invalid)?;
            let effect = EffectDefinitionId::new(address(0x7ec5_0000)?).ok_or_else(invalid)?;
            let modifier = ModifierDefinitionId::new(address(0x7ec6_0000)?).ok_or_else(invalid)?;
            let group = ModifierStackingGroupId::new(address(0x7ec7_0000)?).ok_or_else(invalid)?;
            let selector = |offset: u32| {
                ordinal
                    .checked_mul(32)
                    .and_then(|n| n.checked_add(offset))
                    .and_then(|n| n.checked_add(0x7ec4_0000))
                    .and_then(SelectorId::new)
                    .ok_or_else(invalid)
            };
            let owner = selector(0)?;
            let recipients = selector(1)?;
            let team = selector(30)?;
            let linked = selector(31)?;
            let all_team = select(
                RuleSelectorOrigin::Team,
                RuleLifePredicate::Any,
                u16::MAX,
                RuleSelectorChoice::All,
                RulePresencePredicate::Any,
            )?;
            builder.add_selector(SelectorDefinition::new(team).with_rule_units(all_team.clone()));
            builder.add_selector(SelectorDefinition::new(linked).with_rule_units(
                all_team.with_predicates(vec![RuleSelectorPredicate::OwnedBy(team)]),
            ));
            // Ownership links are independent of presence. A Present memosprite
            // or shared actor must not inherit an original owner's trigger.
            builder.add_selector(
                SelectorDefinition::new(owner).with_rule_units(
                    select(
                        RuleSelectorOrigin::Owner,
                        RuleLifePredicate::Any,
                        1,
                        RuleSelectorChoice::First,
                        RulePresencePredicate::Present,
                    )?
                    .with_predicates(vec![RuleSelectorPredicate::Excludes(linked)]),
                ),
            );
            let mut members = Vec::new();
            for (offset, (player, is_elation)) in players.iter().zip(&eligible).enumerate() {
                if !is_elation {
                    continue;
                }
                let member = selector(
                    u32::try_from(offset)
                        .map_err(|_| invalid())?
                        .checked_add(2)
                        .ok_or_else(invalid)?,
                )?;
                let formation = player.formation().get();
                builder.add_selector(
                    SelectorDefinition::new(member).with_rule_units(
                        select(
                            RuleSelectorOrigin::Team,
                            RuleLifePredicate::Alive,
                            1,
                            RuleSelectorChoice::All,
                            RulePresencePredicate::Present,
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
            }
            builder.add_selector(
                SelectorDefinition::new(recipients).with_rule_units(
                    select(
                        RuleSelectorOrigin::Team,
                        RuleLifePredicate::Alive,
                        4,
                        RuleSelectorChoice::All,
                        RulePresencePredicate::Present,
                    )?
                    .with_candidate_union(members.clone())
                    .ok_or_else(invalid)?,
                ),
            );
            let mut selectors = vec![owner, recipients, team, linked];
            selectors.extend(members);
            selectors.sort_unstable();
            builder.add_modifier_group(ModifierStackingGroup {
                id: group,
                aggregation: ModifierAggregation::UniquePerSource,
                comparator: None,
            });
            builder.add_modifier(ModifierDefinition {
                id: modifier,
                stat: StatKind::Elation,
                stage: FormulaStage::Flat,
                purpose: FormulaPurpose::Stat,
                value: ValueExpr::Literal(RuleValue::Scalar(Scalar::from_scaled(
                    definition.elation_bonus_millionths,
                ))),
                stacking_group: group,
                priority: 0,
                floor: None,
                cap: None,
                cap_stage: FormulaStage::Flat,
                snapshot: SnapshotPolicy::Dynamic,
                source_stack_slot: None,
                filters: Box::new([]),
            });
            builder.add_effect(
                EffectDefinition::new(effect, Vec::new(), vec![modifier]).with_runtime_template(
                    EffectRuntimeTemplate::new(
                        EffectCategory::Buff,
                        DispelCategory::DispellableBuff,
                        1,
                        Some(ValueExpr::Literal(RuleValue::Integer(i64::from(
                            definition.duration_turns,
                        )))),
                        DurationClock::TargetTurnEnd,
                        EffectTickPhase::None,
                        EffectStackPolicy::Replace,
                    )
                    .ok_or_else(invalid)?
                    .with_teardown(EffectTeardownPolicy::PersistByScope),
                ),
            );
            builder.add_program(
                ProgramDefinition::new(
                    program,
                    Vec::new(),
                    selectors.clone(),
                    vec![effect],
                    Vec::new(),
                )
                .with_steps(vec![
                    ProgramStep::Operation(RuleOperationTemplate::ModifyResource {
                        selector: owner,
                        resource: RuleResourceKind::Team("shared.punchline".into()),
                        update: ResourceUpdateKind::Gain,
                        amount: ValueExpr::Literal(RuleValue::Scalar(
                            Scalar::checked_from_integer(i64::from(definition.punchline_gain))
                                .map_err(|_| invalid())?,
                        )),
                        scales_with_regeneration: false,
                        rounding: Rounding::Floor,
                    }),
                    ProgramStep::Operation(RuleOperationTemplate::ApplyEffect {
                        selector: recipients,
                        effect,
                        stacks: ValueExpr::Literal(RuleValue::Integer(1)),
                        chance: RuleEffectChancePolicy::Guaranteed,
                        base_chance: None,
                        rng_purpose: None,
                    }),
                ]),
            );
            let mut hash = CanonicalDigestBuilder::new();
            hash.update(b"starclock.divergent-universe.weighted-curio-elation-source");
            hash.update(assembly_digest);
            hash.update(definition.key.as_bytes());
            let source = RuleSource::new(source_id, SourceClass::Mode, Vec::new(), hash.finalize());
            let triggers = [RuleActionKind::Basic, RuleActionKind::Skill]
                .into_iter()
                .enumerate()
                .map(|(offset, kind)| {
                    let trigger = ordinal
                        .checked_mul(2)
                        .and_then(|n| n.checked_add(u32::try_from(offset).ok()?))
                        .and_then(|n| n.checked_add(0x7ec8_0000))
                        .and_then(TriggerId::new)
                        .ok_or_else(invalid)?;
                    Ok(TriggerDef {
                        id: trigger,
                        event: RuleEventPoint::ActionResolved.kind(),
                        event_point: RuleEventPoint::ActionResolved,
                        phase: TriggerPhase::AfterAction,
                        filter: EventFilter {
                            actor_selector: Some(owner),
                            action_kind: Some(kind),
                            has_action: Some(true),
                            ..EventFilter::default()
                        },
                        condition: ConditionExpr::Literal(true),
                        once_scope: OnceScope::Action,
                        priority: ReactionPriority::new(0),
                        program,
                    })
                })
                .collect::<Result<Vec<_>, DivergentUniverseBattleAssemblyError>>()?;
            builder.add_rule(
                RuleDefinition::new(rule, vec![program], selectors).with_runtime(
                    BattleRuleDefinition::new(source.clone(), Vec::new(), triggers, None),
                ),
            );
            builder.add_rule_bundle(RuleBundle::new(bundle, vec![rule]));
            let bindings = PassiveBindings {
                rule_bundles: vec![bundle],
                sources: vec![source],
                ..PassiveBindings::default()
            };
            for (player, is_elation) in players.iter_mut().zip(eligible) {
                if !is_elation {
                    *player = bind_passives(player, &bindings, assembly_digest)?;
                }
            }
        }
        Ok(())
    }
}

fn select(
    origin: RuleSelectorOrigin,
    life: RuleLifePredicate,
    maximum: u16,
    choice: RuleSelectorChoice,
    presence: RulePresencePredicate,
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
fn invalid() -> DivergentUniverseBattleAssemblyError {
    DivergentUniverseBattleAssemblyError::InvalidPlayerParticipants
}
