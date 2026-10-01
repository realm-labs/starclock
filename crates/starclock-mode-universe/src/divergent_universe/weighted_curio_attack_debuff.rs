//! Attack-local enemy advance and timed source-owned damage reduction.

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
    EffectStackPolicy, EffectTickPhase, ModifierDefinitionId, ModifierStackingGroupId,
    ParticipantSpec, ProgramId, RuleBundleId, RuleId, Scalar, SelectorId, SourceDefinitionId,
    TeamSide, TriggerId,
    catalog::{
        action::AbilityTag,
        builder::CombatCatalogBuilder,
        definition::{
            EffectDefinition, ProgramDefinition, RuleBundle, RuleDefinition, SelectorDefinition,
        },
        selector::{
            RuleEmptyPoolPolicy, RuleLifePredicate, RulePresencePredicate, RuleSelectorChoice,
            RuleSelectorOrdering, RuleSelectorOrigin, RuleSelectorReference, RuleSelectorSide,
            RuleUnitSelector,
        },
    },
    modifier::model::{
        FormulaPurpose, FormulaStage, ModifierAggregation, ModifierDefinition,
        ModifierStackingGroup, SnapshotPolicy, StatKind,
    },
    rule::model::{
        BattleRuleDefinition, ConditionExpr, EventFilter, OnceScope, ProgramStep, ReactionPriority,
        RuleEffectChancePolicy, RuleEventPoint, RuleOperationTemplate, RuleSource, RuleValue,
        SourceClass, TriggerDef, TriggerPhase, ValueExpr,
    },
};
use starclock_data::{
    catalog::SimulationCatalog,
    divergent_universe_decisions::weighted_curio_attack_debuffs::WeightedCurioAttackDebuffPolicy,
};

impl WeightedCurioRuntime {
    pub(super) fn assemble_attack_debuffs(
        &self,
        builder: &mut CombatCatalogBuilder,
        snapshot: &WeightedCurioSnapshot,
        core: &SimulationCatalog,
        players: &mut [ParticipantSpec],
        assembly_digest: [u8; 32],
    ) -> Result<(), DivergentUniverseBattleAssemblyError> {
        for (index, definition) in self.attack_debuffs.iter().enumerate() {
            if !snapshot.equipped().contains(&definition.weighted_curio) {
                continue;
            }
            match definition.policy {
                WeightedCurioAttackDebuffPolicy::VersionedProjectPolicyAttackResolvedAdvanceAndTargetTurnFinalReduction => {}
            }
            let ordinal = u32::try_from(index)
                .ok()
                .and_then(|value| value.checked_add(1))
                .filter(|value| *value <= 8191)
                .ok_or_else(invalid)?;
            let program = ProgramId::new(0x7e60_0000_u32.checked_add(ordinal).ok_or_else(invalid)?)
                .ok_or_else(invalid)?;
            let rule = RuleId::new(0x7e61_0000_u32.checked_add(ordinal).ok_or_else(invalid)?)
                .ok_or_else(invalid)?;
            let bundle =
                RuleBundleId::new(0x7e62_0000_u32.checked_add(ordinal).ok_or_else(invalid)?)
                    .ok_or_else(invalid)?;
            let source_id =
                SourceDefinitionId::new(0x7e63_0000_u32.checked_add(ordinal).ok_or_else(invalid)?)
                    .ok_or_else(invalid)?;
            let owner = SelectorId::new(0x7e64_0000_u32.checked_add(ordinal).ok_or_else(invalid)?)
                .ok_or_else(invalid)?;
            let targets =
                SelectorId::new(0x7e65_0000_u32.checked_add(ordinal).ok_or_else(invalid)?)
                    .ok_or_else(invalid)?;
            let effect =
                EffectDefinitionId::new(0x7e66_0000_u32.checked_add(ordinal).ok_or_else(invalid)?)
                    .ok_or_else(invalid)?;
            let trigger = TriggerId::new(0x7e67_0000_u32.checked_add(ordinal).ok_or_else(invalid)?)
                .ok_or_else(invalid)?;
            let mut hash = CanonicalDigestBuilder::new();
            hash.update(b"starclock.divergent-universe.weighted-curio-attack-debuff-source");
            hash.update(assembly_digest);
            hash.update(definition.key.as_bytes());
            let source = RuleSource::new(source_id, SourceClass::Mode, Vec::new(), hash.finalize());
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
            let factor = Scalar::ONE
                .checked_sub(Scalar::from_scaled(definition.reduction_millionths))
                .map_err(|_| invalid())?;
            let mut modifiers = Vec::new();
            for (offset, purpose) in [
                FormulaPurpose::OrdinaryDamage,
                FormulaPurpose::Dot,
                FormulaPurpose::AdditionalDamage,
                FormulaPurpose::ElationDamage,
                FormulaPurpose::Break,
                FormulaPurpose::SuperBreak,
            ]
            .into_iter()
            .enumerate()
            {
                let address = ordinal
                    .checked_mul(8)
                    .and_then(|value| value.checked_add(u32::try_from(offset).ok()?))
                    .ok_or_else(invalid)?;
                let modifier = ModifierDefinitionId::new(
                    0x7e68_0000_u32.checked_add(address).ok_or_else(invalid)?,
                )
                .ok_or_else(invalid)?;
                let group = ModifierStackingGroupId::new(
                    0x7e69_0000_u32.checked_add(address).ok_or_else(invalid)?,
                )
                .ok_or_else(invalid)?;
                builder.add_modifier_group(ModifierStackingGroup {
                    id: group,
                    aggregation: ModifierAggregation::Product,
                    comparator: None,
                });
                builder.add_modifier(ModifierDefinition {
                    id: modifier,
                    stat: StatKind::Atk,
                    stage: FormulaStage::DamageFinalMultiply,
                    purpose,
                    value: ValueExpr::Literal(RuleValue::Scalar(factor)),
                    stacking_group: group,
                    priority: 0,
                    floor: None,
                    cap: None,
                    cap_stage: FormulaStage::DamageFinalMultiply,
                    snapshot: SnapshotPolicy::Dynamic,
                    source_stack_slot: None,
                    filters: Box::new([]),
                });
                modifiers.push(modifier);
            }
            builder.add_effect(
                EffectDefinition::new(effect, Vec::new(), modifiers).with_runtime_template(
                    EffectRuntimeTemplate::new(
                        EffectCategory::Debuff,
                        DispelCategory::NonDispellable,
                        1,
                        Some(ValueExpr::Literal(RuleValue::Integer(i64::from(
                            definition.duration_turns,
                        )))),
                        DurationClock::TargetTurnEnd,
                        EffectTickPhase::None,
                        EffectStackPolicy::Replace,
                    )
                    .ok_or_else(invalid)?,
                ),
            );
            builder.add_program(
                ProgramDefinition::new(
                    program,
                    Vec::new(),
                    vec![owner, targets],
                    vec![effect],
                    Vec::new(),
                )
                .with_steps(vec![
                    ProgramStep::Operation(RuleOperationTemplate::AdvanceAction {
                        selector: targets,
                        amount: ValueExpr::Literal(RuleValue::Scalar(Scalar::from_scaled(
                            definition.advance_millionths,
                        ))),
                    }),
                    // Explicit removal makes cross-caster replacement unambiguous;
                    // the shared effect store removes its bound modifiers too.
                    ProgramStep::Operation(RuleOperationTemplate::RemoveEffect {
                        selector: targets,
                        effect,
                    }),
                    ProgramStep::Operation(RuleOperationTemplate::ApplyEffect {
                        selector: targets,
                        effect,
                        stacks: ValueExpr::Literal(RuleValue::Integer(1)),
                        chance: RuleEffectChancePolicy::Guaranteed,
                        base_chance: None,
                        rng_purpose: None,
                    }),
                ]),
            );
            builder.add_rule(
                RuleDefinition::new(rule, vec![program], vec![owner, targets]).with_runtime(
                    BattleRuleDefinition::new(
                        source.clone(),
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
            for player in &mut *players {
                if player.side() != TeamSide::Player {
                    return Err(invalid());
                }
                let character = core
                    .build_catalog()
                    .character(player.combatant().form())
                    .ok_or_else(invalid)?;
                if !matches!(
                    character.path(),
                    CombatPath::Destruction | CombatPath::Nihility
                ) {
                    continue;
                }
                *player = bind_passives(
                    player,
                    &PassiveBindings {
                        modifiers: Vec::new(),
                        rule_bundles: vec![bundle],
                        sources: vec![source.clone()],
                    },
                    assembly_digest,
                )?;
            }
        }
        Ok(())
    }
}

fn invalid() -> DivergentUniverseBattleAssemblyError {
    DivergentUniverseBattleAssemblyError::InvalidPlayerParticipants
}
