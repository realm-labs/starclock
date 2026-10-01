//! Real entry HP capacity and owner-turn HP consumption/replacement shield.
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
    EffectStackPolicy, EffectTickPhase, Hp, ParticipantSpec, ProgramId, Ratio, Rounding,
    RuleBundleId, RuleId, Scalar, SelectorId, SourceDefinitionId, TeamSide, TriggerId,
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
    modifier::model::StatQuerySubject,
    rule::model::{
        BattleRuleDefinition, ConditionExpr, EventFilter, OnceScope, ProgramStep, ReactionPriority,
        RuleEffectChancePolicy, RuleEventPoint, RuleOperationTemplate, RuleSource, RuleValue,
        SourceClass, TriggerDef, TriggerPhase, ValueExpr,
    },
};
use starclock_data::{
    catalog::SimulationCatalog,
    divergent_universe_decisions::weighted_curio_prayers::WeightedCurioPrayerPolicy,
};

impl WeightedCurioRuntime {
    pub(super) fn assemble_prayers(
        &self,
        builder: &mut CombatCatalogBuilder,
        snapshot: &WeightedCurioSnapshot,
        core: &SimulationCatalog,
        players: &mut [ParticipantSpec],
        assembly_digest: [u8; 32],
    ) -> Result<(), DivergentUniverseBattleAssemblyError> {
        for (index, definition) in self.prayers.iter().enumerate() {
            if !snapshot.equipped().contains(&definition.weighted_curio) {
                continue;
            }
            match definition.policy {
                WeightedCurioPrayerPolicy::VersionedProjectPolicyEntryHpAndTurnStartConsumeShield => {}
            }
            let ordinal = u32::try_from(index)
                .ok()
                .and_then(|n| n.checked_add(1))
                .filter(|n| *n <= 65535)
                .ok_or_else(invalid)?;
            let address = |base: u32| base.checked_add(ordinal).ok_or_else(invalid);
            let program = ProgramId::new(address(0x7e80_0000)?).ok_or_else(invalid)?;
            let rule = RuleId::new(address(0x7e81_0000)?).ok_or_else(invalid)?;
            let bundle = RuleBundleId::new(address(0x7e82_0000)?).ok_or_else(invalid)?;
            let source = SourceDefinitionId::new(address(0x7e83_0000)?).ok_or_else(invalid)?;
            let owner = SelectorId::new(address(0x7e84_0000)?).ok_or_else(invalid)?;
            let effect = EffectDefinitionId::new(address(0x7e85_0000)?).ok_or_else(invalid)?;
            let trigger = TriggerId::new(address(0x7e86_0000)?).ok_or_else(invalid)?;
            let mut hash = CanonicalDigestBuilder::new();
            hash.update(b"starclock.divergent-universe.weighted-curio-prayer-source");
            hash.update(assembly_digest);
            hash.update(definition.key.as_bytes());
            let source = RuleSource::new(source, SourceClass::Mode, Vec::new(), hash.finalize());
            builder.add_selector(
                SelectorDefinition::new(owner).with_rule_units(
                    RuleUnitSelector::new(
                        RuleSelectorOrigin::Owner,
                        RuleSelectorSide::Same,
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
            builder.add_effect(
                EffectDefinition::new(effect, Vec::new(), Vec::new()).with_runtime_template(
                    EffectRuntimeTemplate::new(
                        EffectCategory::Shield,
                        DispelCategory::NonDispellable,
                        1,
                        None,
                        DurationClock::Permanent,
                        EffectTickPhase::None,
                        EffectStackPolicy::Replace,
                    )
                    .ok_or_else(invalid)?,
                ),
            );
            builder.add_program(
                ProgramDefinition::new(program, Vec::new(), vec![owner], vec![effect], Vec::new())
                    .with_steps(vec![
                        ProgramStep::Operation(RuleOperationTemplate::ConsumeHp {
                            selector: owner,
                            amount: fraction(
                                ValueExpr::QueryHp {
                                    subject: StatQuerySubject::Owner,
                                },
                                definition.consume_millionths,
                            ),
                            floor: ValueExpr::Literal(RuleValue::Scalar(Scalar::ONE)),
                        }),
                        ProgramStep::Operation(RuleOperationTemplate::RemoveShield {
                            selector: owner,
                            effect,
                        }),
                        ProgramStep::Operation(RuleOperationTemplate::RemoveEffect {
                            selector: owner,
                            effect,
                        }),
                        ProgramStep::Operation(RuleOperationTemplate::ApplyEffect {
                            selector: owner,
                            effect,
                            stacks: ValueExpr::Literal(RuleValue::Integer(1)),
                            chance: RuleEffectChancePolicy::Guaranteed,
                            base_chance: None,
                            rng_purpose: None,
                        }),
                        ProgramStep::Operation(RuleOperationTemplate::Shield {
                            selector: owner,
                            effect,
                            amount: fraction(
                                ValueExpr::QueryMaximumHp(StatQuerySubject::Owner),
                                definition.shield_millionths,
                            ),
                        }),
                    ]),
            );
            builder.add_rule(
                RuleDefinition::new(rule, vec![program], vec![owner]).with_runtime(
                    BattleRuleDefinition::new(
                        source.clone(),
                        Vec::new(),
                        vec![TriggerDef {
                            id: trigger,
                            event: RuleEventPoint::TurnStarted.kind(),
                            event_point: RuleEventPoint::TurnStarted,
                            phase: TriggerPhase::AfterEvent,
                            filter: EventFilter {
                                actor_selector: Some(owner),
                                ..EventFilter::default()
                            },
                            condition: ConditionExpr::Literal(true),
                            once_scope: OnceScope::Turn,
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
                    CombatPath::Erudition | CombatPath::Nihility
                ) {
                    continue;
                }
                let base = player.combatant().maximum_hp();
                // Use the pinned domain backend; round only at resource capacity.
                let bonus = Scalar::checked_from_integer(base.get())
                    .and_then(|value| {
                        Ratio::from_scaled(definition.hp_millionths)
                            .checked_apply(value, Rounding::Floor)
                    })
                    .and_then(|value| value.rounded_integer(Rounding::Floor))
                    .map_err(|_| invalid())?;
                let maximum_hp = base
                    .get()
                    .checked_add(bonus)
                    .and_then(|value| Hp::new(value).ok())
                    .ok_or_else(invalid)?;
                *player = bind_passives(
                    player,
                    &PassiveBindings {
                        rule_bundles: vec![bundle],
                        sources: vec![source.clone()],
                        maximum_hp: Some(maximum_hp),
                        ..PassiveBindings::default()
                    },
                    assembly_digest,
                )?;
            }
        }
        Ok(())
    }
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
