//! Ally-directed Harmony actions lower to shared recipient-local timed shields.

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
    EffectStackPolicy, EffectTickPhase, ParticipantSpec, ProgramId, Rounding, RuleBundleId, RuleId,
    Scalar, SelectorId, SourceDefinitionId, TeamSide, TriggerId,
    catalog::{
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
    modifier::model::{FormulaPurpose, StatKind, StatQuerySubject},
    rule::model::{
        BattleRuleDefinition, Comparison, ConditionExpr, EventFilter, OnceScope, ProgramStep,
        ReactionPriority, RuleActionKind, RuleEffectChancePolicy, RuleEventPoint,
        RuleOperationTemplate, RuleSource, RuleValue, SourceClass, TriggerDef, TriggerPhase,
        ValueExpr,
    },
};
use starclock_data::{
    catalog::SimulationCatalog,
    divergent_universe_decisions::weighted_curio_shields::WeightedCurioShieldPolicy,
};

impl WeightedCurioRuntime {
    pub(super) fn assemble_shields(
        &self,
        builder: &mut CombatCatalogBuilder,
        snapshot: &WeightedCurioSnapshot,
        core: &SimulationCatalog,
        players: &mut [ParticipantSpec],
        assembly_digest: [u8; 32],
    ) -> Result<(), DivergentUniverseBattleAssemblyError> {
        for (index, definition) in self.shields.iter().enumerate() {
            if !snapshot.equipped().contains(&definition.weighted_curio) {
                continue;
            }
            match definition.policy {
                WeightedCurioShieldPolicy::VersionedProjectPolicyAllyActionResolvedReplaceTargetTurnShield => {}
            }
            let ordinal = u32::try_from(index)
                .ok()
                .and_then(|value| value.checked_add(1))
                .filter(|value| *value <= 65535)
                .ok_or_else(invalid)?;
            let root = ProgramId::new(0x7e50_0000 + ordinal).ok_or_else(invalid)?;
            let body = ProgramId::new(0x7e51_0000 + ordinal).ok_or_else(invalid)?;
            let rule = RuleId::new(0x7e52_0000 + ordinal).ok_or_else(invalid)?;
            let bundle = RuleBundleId::new(0x7e53_0000 + ordinal).ok_or_else(invalid)?;
            let source = SourceDefinitionId::new(0x7e54_0000 + ordinal).ok_or_else(invalid)?;
            let owner = SelectorId::new(0x7e55_0000 + ordinal).ok_or_else(invalid)?;
            let team = SelectorId::new(0x7e56_0000 + ordinal).ok_or_else(invalid)?;
            let recipient = SelectorId::new(0x7e57_0000 + ordinal).ok_or_else(invalid)?;
            let effect = EffectDefinitionId::new(0x7e58_0000 + ordinal).ok_or_else(invalid)?;
            let directed = SelectorId::new(0x7e5e_0000 + ordinal).ok_or_else(invalid)?;
            let cleanup = ProgramId::new(0x7e5a_0000 + ordinal).ok_or_else(invalid)?;
            let cleanup_rule = RuleId::new(0x7e5b_0000 + ordinal).ok_or_else(invalid)?;
            let cleanup_bundle = RuleBundleId::new(0x7e5c_0000 + ordinal).ok_or_else(invalid)?;
            let cleanup_trigger = TriggerId::new(0x7e5d_0000 + ordinal).ok_or_else(invalid)?;
            let mut digest = CanonicalDigestBuilder::new();
            digest.update(b"starclock.divergent-universe.weighted-curio-shield-source");
            digest.update(assembly_digest);
            digest.update(definition.key.as_bytes());
            let source = RuleSource::new(source, SourceClass::Mode, Vec::new(), digest.finalize());
            for (id, origin, maximum, choice) in [
                (
                    owner,
                    RuleSelectorOrigin::Owner,
                    1,
                    RuleSelectorChoice::First,
                ),
                (team, RuleSelectorOrigin::Team, 16, RuleSelectorChoice::All),
                (
                    recipient,
                    RuleSelectorOrigin::CurrentSubject,
                    1,
                    RuleSelectorChoice::First,
                ),
            ] {
                builder.add_selector(
                    SelectorDefinition::new(id).with_rule_units(
                        RuleUnitSelector::new(
                            origin,
                            RuleSelectorSide::Same,
                            if origin == RuleSelectorOrigin::Owner {
                                RuleLifePredicate::Any
                            } else {
                                RuleLifePredicate::Alive
                            },
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
            builder.add_effect(
                EffectDefinition::new(effect, Vec::new(), Vec::new()).with_runtime_template(
                    EffectRuntimeTemplate::new(
                        EffectCategory::Shield,
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
                    root,
                    vec![body],
                    vec![owner, team, directed],
                    Vec::new(),
                    Vec::new(),
                )
                .with_steps(vec![ProgramStep::ForEach {
                    selector: team,
                    body,
                    maximum: 16,
                }]),
            );
            builder.add_program(
                ProgramDefinition::new(body, Vec::new(), vec![recipient], vec![effect], Vec::new())
                    .with_steps(vec![
                        ProgramStep::Operation(RuleOperationTemplate::RemoveShield {
                            selector: recipient,
                            effect,
                        }),
                        ProgramStep::Operation(RuleOperationTemplate::RemoveEffect {
                            selector: recipient,
                            effect,
                        }),
                        ProgramStep::Operation(RuleOperationTemplate::ApplyEffect {
                            selector: recipient,
                            effect,
                            stacks: ValueExpr::Literal(RuleValue::Integer(1)),
                            chance: RuleEffectChancePolicy::Guaranteed,
                            base_chance: None,
                            rng_purpose: None,
                        }),
                        ProgramStep::Operation(RuleOperationTemplate::Shield {
                            selector: recipient,
                            effect,
                            amount: ValueExpr::Multiply {
                                lhs: Box::new(ValueExpr::QueryStat {
                                    subject: StatQuerySubject::CurrentTarget,
                                    stat: StatKind::Hp,
                                    purpose: FormulaPurpose::Stat,
                                }),
                                rhs: Box::new(ValueExpr::Literal(RuleValue::Scalar(
                                    Scalar::from_scaled(definition.fraction_millionths),
                                ))),
                                rounding: Rounding::Floor,
                            },
                        }),
                    ]),
            );
            let mut triggers = Vec::new();
            builder.add_selector(
                SelectorDefinition::new(directed).with_rule_units(
                    RuleUnitSelector::new(
                        RuleSelectorOrigin::EventTargets,
                        RuleSelectorSide::Same,
                        RuleLifePredicate::Any,
                        RulePresencePredicate::Any,
                        RuleSelectorReference::CurrentState,
                        RuleSelectorOrdering::Formation,
                        0,
                        16,
                        RuleEmptyPoolPolicy::NoOp,
                        RuleSelectorChoice::All,
                        None,
                        false,
                    )
                    .ok_or_else(invalid)?,
                ),
            );
            for (offset, kind) in [
                RuleActionKind::Basic,
                RuleActionKind::Skill,
                RuleActionKind::Ultimate,
            ]
            .into_iter()
            .enumerate()
            {
                let trigger_raw = ordinal
                    .checked_mul(4)
                    .and_then(|value| value.checked_add(u32::try_from(offset).ok()?))
                    .and_then(|value| value.checked_add(0x7e59_0000))
                    .ok_or_else(invalid)?;
                triggers.push(TriggerDef {
                    id: TriggerId::new(trigger_raw).ok_or_else(invalid)?,
                    event: RuleEventPoint::ActionResolved.kind(),
                    event_point: RuleEventPoint::ActionResolved,
                    phase: TriggerPhase::AfterAction,
                    filter: EventFilter {
                        actor_selector: Some(owner),
                        action_kind: Some(kind),
                        has_action: Some(true),
                        ..EventFilter::default()
                    },
                    condition: ConditionExpr::SelectorCardinality {
                        selector: directed,
                        operator: Comparison::Greater,
                        count: 0,
                    },
                    once_scope: OnceScope::Action,
                    priority: ReactionPriority::new(0),
                    program: root,
                });
            }
            builder.add_rule(
                RuleDefinition::new(
                    rule,
                    vec![root, body],
                    vec![owner, team, recipient, directed],
                )
                .with_runtime(BattleRuleDefinition::new(
                    source.clone(),
                    Vec::new(),
                    triggers,
                    None,
                )),
            );
            builder.add_rule_bundle(RuleBundle::new(bundle, vec![rule]));
            // The dedicated shield store does not infer lifetime from an effect
            // definition. Each recipient owns its explicit expiry cleanup, even
            // after a caster leaves. Ignore delayed removal of an old effect
            // when a replacement with the same definition already exists.
            builder.add_program(
                ProgramDefinition::new(cleanup, Vec::new(), vec![owner], vec![effect], Vec::new())
                    .with_steps(vec![ProgramStep::Operation(
                        RuleOperationTemplate::RemoveShield {
                            selector: owner,
                            effect,
                        },
                    )]),
            );
            builder.add_rule(
                RuleDefinition::new(cleanup_rule, vec![cleanup], vec![owner]).with_runtime(
                    BattleRuleDefinition::new(
                        source.clone(),
                        Vec::new(),
                        vec![TriggerDef {
                            id: cleanup_trigger,
                            event: RuleEventPoint::EffectRemoved.kind(),
                            event_point: RuleEventPoint::EffectRemoved,
                            phase: TriggerPhase::AfterEvent,
                            filter: EventFilter {
                                target_selector: Some(owner),
                                effect_definition: Some(effect),
                                ..EventFilter::default()
                            },
                            condition: ConditionExpr::Not(Box::new(ConditionExpr::EffectExists {
                                selector: owner,
                                effect,
                            })),
                            once_scope: OnceScope::Event,
                            priority: ReactionPriority::new(0),
                            program: cleanup,
                        }],
                        None,
                    ),
                ),
            );
            builder.add_rule_bundle(RuleBundle::new(cleanup_bundle, vec![cleanup_rule]));
            for player in &mut *players {
                if player.side() != TeamSide::Player {
                    return Err(invalid());
                }
                let character = core
                    .build_catalog()
                    .character(player.combatant().form())
                    .ok_or_else(invalid)?;
                let mut rule_bundles = vec![cleanup_bundle];
                if character.path() == CombatPath::Harmony {
                    rule_bundles.push(bundle);
                }
                let bindings = PassiveBindings {
                    modifiers: Vec::new(),
                    rule_bundles,
                    sources: vec![source.clone()],
                    maximum_hp: None,
                };
                *player = bind_passives(player, &bindings, assembly_digest)?;
            }
        }
        Ok(())
    }
}

fn invalid() -> DivergentUniverseBattleAssemblyError {
    DivergentUniverseBattleAssemblyError::InvalidPlayerParticipants
}
