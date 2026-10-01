//! Physical victim-sourced nonlethal damage and actual enemy target-weight binding.
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
        action::{AbilityKind, AbilityTag, HitOperationDefinition, TargetPattern, TargetRelation},
        builder::CombatCatalogBuilder,
        definition::{ProgramDefinition, RuleBundle, RuleDefinition, SelectorDefinition},
        selector::{
            RuleEmptyPoolPolicy, RuleLifePredicate, RulePresencePredicate, RuleSelectorChoice,
            RuleSelectorOrdering, RuleSelectorOrigin, RuleSelectorReference, RuleSelectorSide,
            RuleUnitSelector,
        },
    },
    formula::model::{CombatElement, DamageClass},
    modifier::model::{
        FormulaPurpose, FormulaStage, ModifierAggregation, ModifierDefinition,
        ModifierStackingGroup, SnapshotPolicy, StatKind, StatQuerySubject,
    },
    rule::model::{
        BattleRuleDefinition, ConditionExpr, EventFilter, OnceScope, ProgramStep, ReactionPriority,
        RuleDamageClass, RuleEventPoint, RuleOperationTemplate, RuleSource, RuleValue, SourceClass,
        TriggerDef, TriggerPhase, ValueExpr,
    },
};
use starclock_data::{
    catalog::SimulationCatalog,
    divergent_universe_decisions::weighted_curio_retaliations::{
        WeightedCurioPathAggro, WeightedCurioRetaliationPolicy,
    },
};
use std::collections::BTreeSet;

impl WeightedCurioRuntime {
    pub(super) fn assemble_retaliations(
        &self,
        builder: &mut CombatCatalogBuilder,
        snapshot: &WeightedCurioSnapshot,
        core: &SimulationCatalog,
        players: &mut [ParticipantSpec],
        enemies: &[ParticipantSpec],
        assembly_digest: [u8; 32],
    ) -> Result<(), DivergentUniverseBattleAssemblyError> {
        for (index, definition) in self.retaliations.iter().enumerate() {
            if !snapshot.equipped().contains(&definition.weighted_curio) {
                continue;
            }
            let WeightedCurioRetaliationPolicy::VersionedProjectPolicyPhysicalAggroAndOwnerAdditional = definition.policy;
            let ordinal = u32::try_from(index)
                .ok()
                .and_then(|n| n.checked_add(1))
                .filter(|n| *n <= 4095)
                .ok_or_else(invalid)?;
            let address = |base: u32| base.checked_add(ordinal).ok_or_else(invalid);
            let program = ProgramId::new(address(0x7e90_0000)?).ok_or_else(invalid)?;
            let rule = RuleId::new(address(0x7e91_0000)?).ok_or_else(invalid)?;
            let bundle = RuleBundleId::new(address(0x7e92_0000)?).ok_or_else(invalid)?;
            let source_id = SourceDefinitionId::new(address(0x7e93_0000)?).ok_or_else(invalid)?;
            let owner = SelectorId::new(address(0x7e94_0000)?).ok_or_else(invalid)?;
            let attacker = SelectorId::new(address(0x7e95_0000)?).ok_or_else(invalid)?;
            let trigger = TriggerId::new(address(0x7e96_0000)?).ok_or_else(invalid)?;
            let primary = SelectorId::new(address(0x7e97_0000)?).ok_or_else(invalid)?;
            let bonus = ModifierDefinitionId::new(address(0x7e98_0000)?).ok_or_else(invalid)?;
            let bonus_group =
                ModifierStackingGroupId::new(address(0x7e99_0000)?).ok_or_else(invalid)?;
            let mut hash = CanonicalDigestBuilder::new();
            hash.update(b"starclock.divergent-universe.weighted-curio-retaliation-source");
            hash.update(assembly_digest);
            hash.update(definition.key.as_bytes());
            let source = RuleSource::new(source_id, SourceClass::Mode, vec![], hash.finalize());
            for (id, origin, side) in [
                (owner, RuleSelectorOrigin::Owner, RuleSelectorSide::Same),
                (
                    attacker,
                    RuleSelectorOrigin::Actor,
                    RuleSelectorSide::Opposing,
                ),
            ] {
                builder.add_selector(
                    SelectorDefinition::new(id).with_rule_units(
                        RuleUnitSelector::new(
                            origin,
                            side,
                            RuleLifePredicate::Alive,
                            RulePresencePredicate::Present,
                            RuleSelectorReference::CurrentState,
                            RuleSelectorOrdering::Formation,
                            0,
                            1,
                            RuleEmptyPoolPolicy::NoOp,
                            RuleSelectorChoice::First,
                            None,
                            false,
                        )
                        .ok_or_else(invalid)?,
                    ),
                );
            }
            builder.add_program(
                ProgramDefinition::new(program, vec![], vec![owner, attacker], vec![], vec![])
                    .with_steps(vec![ProgramStep::Operation(
                        RuleOperationTemplate::DamageFromOwner {
                            selector: attacker,
                            amount: ValueExpr::Multiply {
                                lhs: Box::new(ValueExpr::QueryStat {
                                    subject: StatQuerySubject::Owner,
                                    stat: StatKind::Atk,
                                    purpose: FormulaPurpose::Stat,
                                }),
                                rhs: Box::new(ValueExpr::Literal(RuleValue::Scalar(
                                    Scalar::from_scaled(definition.additional_millionths),
                                ))),
                                rounding: Rounding::Floor,
                            },
                            class: DamageClass::Additional,
                            element: CombatElement::Physical,
                            can_crit: false,
                            can_defeat: false,
                        },
                    )]),
            );
            builder.add_rule(
                RuleDefinition::new(rule, vec![program], vec![owner, attacker]).with_runtime(
                    BattleRuleDefinition::new(
                        source.clone(),
                        vec![],
                        vec![TriggerDef {
                            id: trigger,
                            event: RuleEventPoint::DamageApplied.kind(),
                            event_point: RuleEventPoint::DamageApplied,
                            phase: TriggerPhase::AfterEvent,
                            filter: EventFilter {
                                target_selector: Some(owner),
                                actor_selector: Some(attacker),
                                ability_tag: Some(AbilityTag::Attack),
                                has_action: Some(true),
                                source_class: Some(SourceClass::Ability),
                                excluded_source: Some(source_id),
                                damage_class: Some(RuleDamageClass::Ordinary),
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
            modifier(
                builder,
                bonus,
                bonus_group,
                FormulaStage::PercentOfBase,
                Scalar::from_scaled(definition.aggro_millionths),
            );
            // The content-owned baseline is explicit and applied only once, after
            // the normalized factor. No second base-weight multiplier is drawn.
            for (offset, path) in [
                CombatPath::Destruction,
                CombatPath::Hunt,
                CombatPath::Erudition,
                CombatPath::Harmony,
                CombatPath::Nihility,
                CombatPath::Preservation,
                CombatPath::Abundance,
                CombatPath::Remembrance,
                CombatPath::Elation,
            ]
            .into_iter()
            .enumerate()
            {
                let suffix = ordinal
                    .checked_mul(16)
                    .and_then(|n| n.checked_add(u32::try_from(offset).ok()?))
                    .ok_or_else(invalid)?;
                let baseline = ModifierDefinitionId::new(
                    0x7e9a_0000_u32.checked_add(suffix).ok_or_else(invalid)?,
                )
                .ok_or_else(invalid)?;
                let group = ModifierStackingGroupId::new(
                    0x7e9b_0000_u32.checked_add(suffix).ok_or_else(invalid)?,
                )
                .ok_or_else(invalid)?;
                modifier(
                    builder,
                    baseline,
                    group,
                    FormulaStage::FinalMultiply,
                    Scalar::checked_from_integer(path_weight(definition.base_aggro, path))
                        .map_err(|_| invalid())?,
                );
                for player in &mut *players {
                    if player.side() != TeamSide::Player {
                        return Err(invalid());
                    }
                    let character = core
                        .build_catalog()
                        .character(player.combatant().form())
                        .ok_or_else(invalid)?;
                    if character.path() != path {
                        continue;
                    }
                    let physical = basic_element(core, player)? == CombatElement::Physical;
                    let bindings = PassiveBindings {
                        modifiers: if physical {
                            vec![
                                ResolvedModifierBinding::new(bonus, source_id),
                                ResolvedModifierBinding::new(baseline, source_id),
                            ]
                        } else {
                            vec![ResolvedModifierBinding::new(baseline, source_id)]
                        },
                        rule_bundles: if physical { vec![bundle] } else { vec![] },
                        sources: vec![source.clone()],
                        maximum_hp: None,
                    };
                    *player = bind_passives(player, &bindings, assembly_digest)?;
                }
            }
            bind_enemy_primaries(builder, core, enemies, primary)?;
        }
        Ok(())
    }
}

fn bind_enemy_primaries(
    builder: &mut CombatCatalogBuilder,
    core: &SimulationCatalog,
    enemies: &[ParticipantSpec],
    primary: SelectorId,
) -> Result<(), DivergentUniverseBattleAssemblyError> {
    builder.add_selector(
        SelectorDefinition::new(primary).with_rule_units(
            RuleUnitSelector::new(
                RuleSelectorOrigin::Encounter,
                RuleSelectorSide::Opposing,
                RuleLifePredicate::Alive,
                RulePresencePredicate::Present,
                RuleSelectorReference::CurrentState,
                RuleSelectorOrdering::Formation,
                1,
                1,
                RuleEmptyPoolPolicy::Fault,
                RuleSelectorChoice::RngWeighted,
                Some("aggro-target".into()),
                false,
            )
            .ok_or_else(invalid)?
            .with_weight(Some(ValueExpr::QueryStat {
                subject: StatQuerySubject::CurrentTarget,
                stat: StatKind::Aggro,
                purpose: FormulaPurpose::Aggro,
            })),
        ),
    );
    let mut abilities = BTreeSet::new();
    for enemy in enemies {
        if enemy.side() != TeamSide::Enemy {
            return Err(invalid());
        }
        abilities.extend(enemy.combatant().abilities().iter().copied());
    }
    for id in abilities {
        let definition = core.combat_catalog().ability(id).ok_or_else(invalid)?;
        let Some(action) = definition.action() else {
            continue;
        };
        let selector = core
            .combat_catalog()
            .selector(definition.selector())
            .and_then(|row| row.unit_targets())
            .ok_or_else(invalid)?;
        if !action.kind().is_normal_turn()
            || definition.automatic_primary_selector().is_some()
            || selector.relation() != TargetRelation::Opposing
            || !matches!(
                selector.pattern(),
                TargetPattern::Single | TargetPattern::Blast
            )
        {
            continue;
        }
        if !builder.replace_ability(definition.clone().with_automatic_primary_selector(primary)) {
            return Err(invalid());
        }
    }
    Ok(())
}

fn basic_element(
    core: &SimulationCatalog,
    player: &ParticipantSpec,
) -> Result<CombatElement, DivergentUniverseBattleAssemblyError> {
    let mut element = None;
    for id in player.combatant().abilities() {
        let definition = core.combat_catalog().ability(*id).ok_or_else(invalid)?;
        let Some(action) = definition
            .action()
            .filter(|action| action.kind() == AbilityKind::Basic)
        else {
            continue;
        };
        for operation in action.hits().iter().flat_map(|hit| hit.operations()) {
            if let HitOperationDefinition::ScalingDamage(damage) = operation {
                if element.is_some_and(|previous| previous != damage.element()) {
                    return Err(invalid());
                }
                element = Some(damage.element());
            }
        }
    }
    element.ok_or_else(invalid)
}

fn path_weight(weights: WeightedCurioPathAggro, path: CombatPath) -> i64 {
    match path {
        CombatPath::Destruction => weights.destruction,
        CombatPath::Hunt => weights.hunt,
        CombatPath::Erudition => weights.erudition,
        CombatPath::Harmony => weights.harmony,
        CombatPath::Nihility => weights.nihility,
        CombatPath::Preservation => weights.preservation,
        CombatPath::Abundance => weights.abundance,
        CombatPath::Remembrance => weights.remembrance,
        CombatPath::Elation => weights.elation,
    }
}

fn modifier(
    builder: &mut CombatCatalogBuilder,
    id: ModifierDefinitionId,
    group: ModifierStackingGroupId,
    stage: FormulaStage,
    value: Scalar,
) {
    builder.add_modifier_group(ModifierStackingGroup {
        id: group,
        aggregation: ModifierAggregation::UniquePerSource,
        comparator: None,
    });
    builder.add_modifier(ModifierDefinition {
        id,
        stat: StatKind::Aggro,
        stage,
        purpose: FormulaPurpose::Aggro,
        value: ValueExpr::Literal(RuleValue::Scalar(value)),
        stacking_group: group,
        priority: 0,
        floor: None,
        cap: None,
        cap_stage: stage,
        snapshot: SnapshotPolicy::Dynamic,
        source_stack_slot: None,
        filters: Box::new([]),
    });
}
fn invalid() -> DivergentUniverseBattleAssemblyError {
    DivergentUniverseBattleAssemblyError::InvalidPlayerParticipants
}
