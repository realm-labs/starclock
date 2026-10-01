//! Source-owned Necrosis and bounded other-Burn detonation through shared Rule IR.
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
    DispelCategory, DotDetonationFilter, DotDetonationScope, DotDetonationSelection, DotFamily,
    DurationClock, EffectCategory, EffectDefinitionId, EffectRuntimeTemplate, EffectSnapshotPolicy,
    EffectStackPolicy, EffectTickPhase, ParticipantSpec, ProgramId, Rounding, RuleBundleId, RuleId,
    Scalar, SelectorId, SourceDefinitionId, TeamSide, TriggerId,
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
    formula::model::CombatElement,
    modifier::model::{FormulaPurpose, StatKind, StatQuerySubject},
    rng::types::DrawPurpose,
    rule::model::{
        BattleRuleDefinition, ConditionExpr, EventFilter, OnceScope, ProgramStep, ReactionPriority,
        RuleDamageClass, RuleDotSelection, RuleEffectChancePolicy, RuleEventPoint,
        RuleOperationTemplate, RuleSource, RuleValue, SourceClass, TriggerDef, TriggerPhase,
        ValueExpr,
    },
};
use starclock_data::{
    catalog::SimulationCatalog,
    divergent_universe_decisions::weighted_curio_necroses::WeightedCurioNecrosisPolicy,
};

impl WeightedCurioRuntime {
    pub(super) fn assemble_necroses(
        &self,
        builder: &mut CombatCatalogBuilder,
        snapshot: &WeightedCurioSnapshot,
        core: &SimulationCatalog,
        players: &mut [ParticipantSpec],
        assembly_digest: [u8; 32],
    ) -> Result<(), DivergentUniverseBattleAssemblyError> {
        for (index, definition) in self.necroses.iter().enumerate() {
            if !snapshot.equipped().contains(&definition.weighted_curio) {
                continue;
            }
            let WeightedCurioNecrosisPolicy::VersionedProjectPolicyAttackResolvedNecrosisAndDotDamageBurnDetonation = definition.policy;
            let ordinal = u32::try_from(index)
                .ok()
                .and_then(|n| n.checked_add(1))
                .filter(|n| *n <= 4095)
                .ok_or_else(invalid)?;
            let address = |base: u32| base.checked_add(ordinal).ok_or_else(invalid);
            let apply = ProgramId::new(address(0x7eb0_0000)?).ok_or_else(invalid)?;
            let detonate = ProgramId::new(address(0x7eb1_0000)?).ok_or_else(invalid)?;
            let rule = RuleId::new(address(0x7eb2_0000)?).ok_or_else(invalid)?;
            let source_id = SourceDefinitionId::new(address(0x7eb3_0000)?).ok_or_else(invalid)?;
            let owner = SelectorId::new(address(0x7eb4_0000)?).ok_or_else(invalid)?;
            let targets = SelectorId::new(address(0x7eb5_0000)?).ok_or_else(invalid)?;
            let effect = EffectDefinitionId::new(address(0x7eb6_0000)?).ok_or_else(invalid)?;
            let application = TriggerId::new(address(0x7eb7_0000)?).ok_or_else(invalid)?;
            let damage = TriggerId::new(address(0x7eb8_0000)?).ok_or_else(invalid)?;
            let bundle = RuleBundleId::new(address(0x7eb9_0000)?).ok_or_else(invalid)?;
            let purpose = ordinal
                .checked_add(41_000)
                .and_then(|n| u16::try_from(n).ok())
                .and_then(DrawPurpose::new)
                .ok_or_else(invalid)?;
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
            let magnitude = ValueExpr::Multiply {
                lhs: Box::new(ValueExpr::QueryStat {
                    subject: StatQuerySubject::Owner,
                    stat: StatKind::Atk,
                    purpose: FormulaPurpose::Stat,
                }),
                rhs: Box::new(value(definition.attack_multiplier_millionths)),
                rounding: Rounding::NearestTiesEven,
            };
            builder.add_effect(
                EffectDefinition::new(effect, vec![], vec![])
                    .with_runtime_template(
                        EffectRuntimeTemplate::new(
                            EffectCategory::Dot,
                            DispelCategory::DispellableDebuff,
                            1,
                            Some(ValueExpr::Literal(RuleValue::Integer(i64::from(
                                definition.duration_turns,
                            )))),
                            DurationClock::TargetTurnEnd,
                            EffectTickPhase::TurnStart,
                            EffectStackPolicy::Replace,
                        )
                        .ok_or_else(invalid)?
                        .with_comparison(Some(magnitude), 0)
                        .with_snapshot(EffectSnapshotPolicy::SourceSnapshotTargetDynamic)
                        .with_dot(CombatElement::Fire, None)
                        .ok_or_else(invalid)?,
                    )
                    .with_dot_family(DotFamily::Burn),
            );
            builder.add_program(
                ProgramDefinition::new(apply, vec![], vec![owner, targets], vec![effect], vec![])
                    .with_steps(vec![ProgramStep::Operation(
                        RuleOperationTemplate::ApplyEffect {
                            selector: targets,
                            effect,
                            stacks: ValueExpr::Literal(RuleValue::Integer(1)),
                            chance: RuleEffectChancePolicy::Resistible,
                            base_chance: Some(value(definition.base_chance_millionths)),
                            rng_purpose: Some(purpose),
                        },
                    )]),
            );
            builder.add_program(
                ProgramDefinition::new(
                    detonate,
                    vec![],
                    vec![owner, targets],
                    vec![effect],
                    vec![],
                )
                .with_steps(vec![ProgramStep::Operation(
                    RuleOperationTemplate::DetonateDot {
                        selector: targets,
                        fraction: value(definition.detonation_fraction_millionths),
                        required_tag: None,
                        selection: RuleDotSelection::Filtered {
                            filter: DotDetonationFilter::family(DotFamily::Burn).excluding(effect),
                            selection: DotDetonationSelection::All,
                            scope: DotDetonationScope::OrdinaryAndBreakEffects,
                        },
                    },
                )]),
            );
            let mut hash = CanonicalDigestBuilder::new();
            hash.update(b"starclock.divergent-universe.weighted-curio-necrosis-source");
            hash.update(assembly_digest);
            hash.update(definition.key.as_bytes());
            let source = RuleSource::new(source_id, SourceClass::Mode, vec![], hash.finalize());
            builder.add_rule(
                RuleDefinition::new(rule, vec![apply, detonate], vec![owner, targets])
                    .with_runtime(BattleRuleDefinition::new(
                        source.clone(),
                        vec![],
                        vec![
                            TriggerDef {
                                id: application,
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
                                program: apply,
                            },
                            TriggerDef {
                                id: damage,
                                event: RuleEventPoint::DamageApplied.kind(),
                                event_point: RuleEventPoint::DamageApplied,
                                phase: TriggerPhase::AfterEvent,
                                filter: EventFilter {
                                    source: Some(source_id),
                                    applier_selector: Some(owner),
                                    element: Some(CombatElement::Fire),
                                    damage_class: Some(RuleDamageClass::Dot),
                                    ..EventFilter::default()
                                },
                                condition: ConditionExpr::Literal(true),
                                once_scope: OnceScope::Event,
                                priority: ReactionPriority::new(0),
                                program: detonate,
                            },
                        ],
                        None,
                    )),
            );
            builder.add_rule_bundle(RuleBundle::new(bundle, vec![rule]));
            let bindings = PassiveBindings {
                rule_bundles: vec![bundle],
                sources: vec![source],
                ..PassiveBindings::default()
            };
            for player in &mut *players {
                if player.side() != TeamSide::Player {
                    return Err(invalid());
                }
                let character = core
                    .build_catalog()
                    .character(player.combatant().form())
                    .ok_or_else(invalid)?;
                if character.path() == CombatPath::Abundance {
                    *player = bind_passives(player, &bindings, assembly_digest)?;
                }
            }
        }
        Ok(())
    }
}
fn value(scaled: i64) -> ValueExpr {
    ValueExpr::Literal(RuleValue::Scalar(Scalar::from_scaled(scaled)))
}
fn invalid() -> DivergentUniverseBattleAssemblyError {
    DivergentUniverseBattleAssemblyError::InvalidPlayerParticipants
}
