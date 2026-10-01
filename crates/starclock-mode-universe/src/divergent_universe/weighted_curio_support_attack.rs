//! Roster-count crit passives and one shared additional operation per attack.
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
    ModifierDefinitionId, ModifierStackingGroupId, ParticipantSpec, ProgramId,
    ResolvedModifierBinding, Rounding, RuleBundleId, RuleId, Scalar, SelectorId,
    SourceDefinitionId, TeamSide, TriggerId,
    catalog::{
        action::AbilityTag,
        builder::CombatCatalogBuilder,
        definition::{ProgramDefinition, RuleBundle, RuleDefinition, SelectorDefinition},
        selector::{
            RuleEmptyPoolPolicy, RuleLifePredicate, RulePresencePredicate, RuleSelectorChoice,
            RuleSelectorOrdering, RuleSelectorOrigin, RuleSelectorReference, RuleSelectorSide,
            RuleUnitSelector,
        },
    },
    formula::model::DamageClass,
    modifier::model::{
        FormulaPurpose, FormulaStage, ModifierAggregation, ModifierDefinition,
        ModifierStackingGroup, SnapshotPolicy, StatKind, StatQuerySubject,
    },
    rule::model::{
        BattleRuleDefinition, ConditionExpr, EventFilter, OnceScope, ProgramStep, ReactionPriority,
        RuleEventPoint, RuleOperationTemplate, RuleSource, RuleValue, SourceClass, TriggerDef,
        TriggerPhase, ValueExpr,
    },
};
use starclock_data::{
    catalog::SimulationCatalog,
    divergent_universe_decisions::weighted_curio_support_attacks::WeightedCurioSupportAttackPolicy,
};

impl WeightedCurioRuntime {
    pub(super) fn assemble_support_attacks(
        &self,
        builder: &mut CombatCatalogBuilder,
        snapshot: &WeightedCurioSnapshot,
        core: &SimulationCatalog,
        players: &mut [ParticipantSpec],
        assembly_digest: [u8; 32],
    ) -> Result<(), DivergentUniverseBattleAssemblyError> {
        // Membership belongs to the immutable mapped character roster, not
        // live combat life/presence or independently created linked actors.
        let eligible = players
            .iter()
            .map(|player| {
                if player.side() != TeamSide::Player {
                    return Err(invalid());
                }
                let character = core
                    .build_catalog()
                    .character(player.combatant().form())
                    .ok_or_else(invalid)?;
                Ok(matches!(
                    character.path(),
                    CombatPath::Harmony | CombatPath::Abundance | CombatPath::Preservation
                ))
            })
            .collect::<Result<Vec<_>, DivergentUniverseBattleAssemblyError>>()?;
        let count = i64::try_from(eligible.iter().filter(|value| **value).count())
            .map_err(|_| invalid())?;
        for (index, definition) in self.support_attacks.iter().enumerate() {
            if !snapshot.equipped().contains(&definition.weighted_curio) {
                continue;
            }
            match definition.policy {
                WeightedCurioSupportAttackPolicy::VersionedProjectPolicyRosterCountCritAndAttackResolvedAdditional => {}
            }
            let ordinal = u32::try_from(index)
                .ok()
                .and_then(|value| value.checked_add(1))
                .filter(|value| *value <= 8191)
                .ok_or_else(invalid)?;
            let address = |base: u32| base.checked_add(ordinal).ok_or_else(invalid);
            let program = ProgramId::new(address(0x7e70_0000)?).ok_or_else(invalid)?;
            let rule = RuleId::new(address(0x7e71_0000)?).ok_or_else(invalid)?;
            let bundle = RuleBundleId::new(address(0x7e72_0000)?).ok_or_else(invalid)?;
            let source_id = SourceDefinitionId::new(address(0x7e73_0000)?).ok_or_else(invalid)?;
            let owner = SelectorId::new(address(0x7e74_0000)?).ok_or_else(invalid)?;
            let targets = SelectorId::new(address(0x7e75_0000)?).ok_or_else(invalid)?;
            let trigger = TriggerId::new(address(0x7e76_0000)?).ok_or_else(invalid)?;
            let mut hash = CanonicalDigestBuilder::new();
            hash.update(b"starclock.divergent-universe.weighted-curio-support-attack-source");
            hash.update(assembly_digest);
            hash.update(definition.key.as_bytes());
            hash.update(count.to_le_bytes());
            let source = RuleSource::new(source_id, SourceClass::Mode, Vec::new(), hash.finalize());
            let mut bindings = PassiveBindings {
                rule_bundles: vec![bundle],
                sources: vec![source.clone()],
                ..PassiveBindings::default()
            };
            for (offset, stat, per_character) in [
                (0_u32, StatKind::CritRate, definition.crit_rate_millionths),
                (1, StatKind::CritDamage, definition.crit_damage_millionths),
            ] {
                let suffix = ordinal
                    .checked_mul(2)
                    .and_then(|value| value.checked_add(offset))
                    .ok_or_else(invalid)?;
                let modifier = ModifierDefinitionId::new(
                    0x7e78_0000_u32.checked_add(suffix).ok_or_else(invalid)?,
                )
                .ok_or_else(invalid)?;
                let group = ModifierStackingGroupId::new(
                    0x7e79_0000_u32.checked_add(suffix).ok_or_else(invalid)?,
                )
                .ok_or_else(invalid)?;
                builder.add_modifier_group(ModifierStackingGroup {
                    id: group,
                    aggregation: ModifierAggregation::UniquePerSource,
                    comparator: None,
                });
                builder.add_modifier(ModifierDefinition {
                    id: modifier,
                    stat,
                    stage: FormulaStage::Flat,
                    purpose: FormulaPurpose::Stat,
                    value: ValueExpr::Literal(RuleValue::Scalar(Scalar::from_scaled(
                        per_character.checked_mul(count).ok_or_else(invalid)?,
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
                bindings
                    .modifiers
                    .push(ResolvedModifierBinding::new(modifier, source_id));
            }
            for (id, origin, side, life, maximum, choice) in [
                (
                    owner,
                    RuleSelectorOrigin::Owner,
                    RuleSelectorSide::Same,
                    RuleLifePredicate::Any,
                    1,
                    RuleSelectorChoice::First,
                ),
                (
                    targets,
                    RuleSelectorOrigin::EventTargets,
                    RuleSelectorSide::Opposing,
                    RuleLifePredicate::Alive,
                    16,
                    RuleSelectorChoice::All,
                ),
            ] {
                builder.add_selector(
                    SelectorDefinition::new(id).with_rule_units(
                        RuleUnitSelector::new(
                            origin,
                            side,
                            life,
                            RulePresencePredicate::Present,
                            RuleSelectorReference::CurrentState,
                            RuleSelectorOrdering::Formation,
                            0,
                            maximum,
                            RuleEmptyPoolPolicy::NoOp,
                            choice,
                            None,
                            false,
                        )
                        .ok_or_else(invalid)?,
                    ),
                );
            }
            let stats = ValueExpr::Add(
                Box::new(effective(StatKind::Hp)),
                Box::new(ValueExpr::Add(
                    Box::new(effective(StatKind::Def)),
                    Box::new(effective(StatKind::Atk)),
                )),
            );
            builder.add_program(
                ProgramDefinition::new(
                    program,
                    Vec::new(),
                    vec![owner, targets],
                    Vec::new(),
                    Vec::new(),
                )
                .with_steps(vec![ProgramStep::Operation(
                    RuleOperationTemplate::DamageFromActorBasicElement {
                        selector: targets,
                        amount: ValueExpr::Multiply {
                            lhs: Box::new(stats),
                            rhs: Box::new(ValueExpr::Literal(RuleValue::Scalar(
                                Scalar::from_scaled(definition.additional_millionths),
                            ))),
                            rounding: Rounding::NearestTiesEven,
                        },
                        class: DamageClass::Additional,
                        can_crit: false,
                        can_defeat: true,
                    },
                )]),
            );
            builder.add_rule(
                RuleDefinition::new(rule, vec![program], vec![owner, targets]).with_runtime(
                    BattleRuleDefinition::new(
                        source,
                        Vec::new(),
                        vec![TriggerDef {
                            id: trigger,
                            event: RuleEventPoint::ActionResolved.kind(),
                            event_point: RuleEventPoint::ActionResolved,
                            phase: TriggerPhase::AfterAction,
                            filter: EventFilter {
                                actor_selector: Some(owner),
                                ability_tag: Some(AbilityTag::Attack),
                                has_action: Some(true),
                                ..EventFilter::default()
                            },
                            condition: ConditionExpr::Literal(true),
                            once_scope: OnceScope::Action,
                            priority: ReactionPriority::new(0),
                            program,
                        }],
                        None,
                    ),
                ),
            );
            builder.add_rule_bundle(RuleBundle::new(bundle, vec![rule]));
            for (player, applies) in players.iter_mut().zip(&eligible) {
                if *applies {
                    *player = bind_passives(player, &bindings, assembly_digest)?;
                }
            }
        }
        Ok(())
    }
}

fn effective(stat: StatKind) -> ValueExpr {
    ValueExpr::QueryStat {
        subject: StatQuerySubject::Owner,
        stat,
        purpose: FormulaPurpose::Stat,
    }
}
fn invalid() -> DivergentUniverseBattleAssemblyError {
    DivergentUniverseBattleAssemblyError::InvalidPlayerParticipants
}
