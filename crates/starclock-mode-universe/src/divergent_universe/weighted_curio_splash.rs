//! Source-attributed copied attack damage; the shared battle owns all mutation.

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
    ParticipantSpec, ProgramId, Rounding, RuleBundleId, RuleId, Scalar, SelectorId,
    SourceDefinitionId, TeamSide, TriggerId,
    catalog::{
        builder::CombatCatalogBuilder,
        definition::{ProgramDefinition, RuleBundle, RuleDefinition, SelectorDefinition},
        selector::{
            RuleEmptyPoolPolicy, RuleLifePredicate, RulePresencePredicate, RuleSelectorChoice,
            RuleSelectorOrdering, RuleSelectorOrigin, RuleSelectorReference, RuleSelectorSide,
            RuleUnitSelector,
        },
    },
    rule::model::{
        BattleRuleDefinition, Comparison, ConditionExpr, EventFilter, EventValueProperty,
        OnceScope, ProgramStep, ReactionPriority, RuleDamageClass, RuleEventPoint,
        RuleOperationTemplate, RuleSource, RuleValue, RuleValueKind, SourceClass, TriggerDef,
        TriggerPhase, ValueExpr,
    },
};
use starclock_data::{
    catalog::SimulationCatalog,
    divergent_universe_decisions::weighted_curio_splashes::WeightedCurioSplashPolicy,
};

impl WeightedCurioRuntime {
    pub(super) fn assemble(
        &self,
        builder: &mut CombatCatalogBuilder,
        snapshot: &WeightedCurioSnapshot,
        core: &SimulationCatalog,
        players: &mut [ParticipantSpec],
        assembly_digest: [u8; 32],
    ) -> Result<(), DivergentUniverseBattleAssemblyError> {
        if snapshot.equipped().is_empty() {
            return Ok(());
        }
        for (index, definition) in self.splashes.iter().enumerate() {
            if !snapshot.equipped().contains(&definition.weighted_curio) {
                continue;
            }
            match definition.policy {
                WeightedCurioSplashPolicy::VersionedProjectPolicyHitCalculatedCopyAdjacentTrueDamage => {}
            }
            let ordinal = u32::try_from(index)
                .ok()
                .and_then(|value| value.checked_add(1))
                .filter(|value| *value <= 65535)
                .ok_or_else(invalid)?;
            let program = ProgramId::new(0x7e40_0000 + ordinal).ok_or_else(invalid)?;
            let rule = RuleId::new(0x7e41_0000 + ordinal).ok_or_else(invalid)?;
            let bundle = RuleBundleId::new(0x7e42_0000 + ordinal).ok_or_else(invalid)?;
            let trigger = TriggerId::new(0x7e43_0000 + ordinal).ok_or_else(invalid)?;
            let source = SourceDefinitionId::new(0x7e44_0000 + ordinal).ok_or_else(invalid)?;
            let owner = SelectorId::new(0x7e45_0000 + ordinal).ok_or_else(invalid)?;
            let adjacent = SelectorId::new(0x7e46_0000 + ordinal).ok_or_else(invalid)?;
            let mut digest = CanonicalDigestBuilder::new();
            digest.update(b"starclock.divergent-universe.weighted-curio-splash-source");
            digest.update(assembly_digest);
            digest.update(definition.key.as_bytes());
            let source = RuleSource::new(source, SourceClass::Mode, Vec::new(), digest.finalize());
            let fraction = Scalar::from_scaled(definition.fraction_millionths);
            builder.add_selector(SelectorDefinition::new(owner).with_rule_units(selector(
                RuleSelectorOrigin::Owner,
                RuleSelectorSide::Same,
                RuleLifePredicate::Any,
                RuleSelectorChoice::First,
                1,
            )?));
            // The damaged enemy remains the event primary even after defeat;
            // side is relative to the rule owner, while adjacency uses the event.
            builder.add_selector(SelectorDefinition::new(adjacent).with_rule_units(selector(
                RuleSelectorOrigin::PrimaryTarget,
                RuleSelectorSide::Opposing,
                RuleLifePredicate::Alive,
                RuleSelectorChoice::AdjacentToPrimary,
                2,
            )?));
            builder.add_program(
                ProgramDefinition::new(
                    program,
                    Vec::new(),
                    vec![owner, adjacent],
                    Vec::new(),
                    Vec::new(),
                )
                .with_steps(vec![ProgramStep::Operation(
                    RuleOperationTemplate::TrueDamage {
                        selector: adjacent,
                        amount: ValueExpr::Multiply {
                            lhs: Box::new(calculated_hit_damage()),
                            rhs: Box::new(ValueExpr::Literal(RuleValue::Scalar(fraction))),
                            rounding: Rounding::Floor,
                        },
                    },
                )]),
            );
            builder.add_rule(
                RuleDefinition::new(rule, vec![program], vec![owner, adjacent]).with_runtime(
                    BattleRuleDefinition::new(
                        source.clone(),
                        Vec::new(),
                        vec![TriggerDef {
                            id: trigger,
                            event: RuleEventPoint::DamageApplied.kind(),
                            event_point: RuleEventPoint::DamageApplied,
                            phase: TriggerPhase::AfterEvent,
                            filter: EventFilter {
                                actor_selector: Some(owner),
                                applier_selector: Some(owner),
                                source_class: Some(SourceClass::Ability),
                                damage_class: Some(RuleDamageClass::Ordinary),
                                has_action: Some(true),
                                ..EventFilter::default()
                            },
                            condition: ConditionExpr::Compare {
                                lhs: Box::new(calculated_hit_damage()),
                                operator: Comparison::Greater,
                                rhs: Box::new(ValueExpr::Literal(RuleValue::Scalar(Scalar::ZERO))),
                            },
                            once_scope: OnceScope::Event,
                            priority: ReactionPriority::new(0),
                            program,
                        }],
                        None,
                    ),
                ),
            );
            builder.add_rule_bundle(RuleBundle::new(bundle, vec![rule]));
            let bindings = PassiveBindings {
                modifiers: Vec::new(),
                rule_bundles: vec![bundle],
                sources: vec![source],
            };
            for player in &mut *players {
                if player.side() != TeamSide::Player {
                    return Err(invalid());
                }
                let character = core
                    .build_catalog()
                    .character(player.combatant().form())
                    .ok_or_else(invalid)?;
                if character.path() == CombatPath::Hunt {
                    *player = bind_passives(player, &bindings, assembly_digest)?;
                }
            }
        }
        self.assemble_shields(builder, snapshot, core, players, assembly_digest)
    }
}

fn selector(
    origin: RuleSelectorOrigin,
    side: RuleSelectorSide,
    life: RuleLifePredicate,
    choice: RuleSelectorChoice,
    maximum: u16,
) -> Result<RuleUnitSelector, DivergentUniverseBattleAssemblyError> {
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
    .ok_or_else(invalid)
}

fn invalid() -> DivergentUniverseBattleAssemblyError {
    DivergentUniverseBattleAssemblyError::InvalidPlayerParticipants
}

/// The generic DamageAmount event property is effective HP loss. Copy the
/// floored raw formula result instead, so shields and overkill cannot alter the
/// authored basis. Both conversions use the existing checked Rule IR codec.
fn calculated_hit_damage() -> ValueExpr {
    ValueExpr::Convert {
        value: Box::new(ValueExpr::Convert {
            value: Box::new(ValueExpr::ReadEventProperty(
                EventValueProperty::DamageRawAmount,
            )),
            target: RuleValueKind::Integer,
            rounding: Rounding::Floor,
        }),
        target: RuleValueKind::Scalar,
        rounding: Rounding::Floor,
    }
}
