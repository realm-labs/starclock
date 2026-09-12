//! One source-attributed run passive observes real damage through shared Rule IR.
use super::battle_passive_bindings::{PassiveBindings, bind_passives};
use super::{
    DivergentUniverseBattleAssemblyError, DivergentUniverseCurioSnapshot,
    DivergentUniverseRuntimeFactory,
};
use crate::{digest::CanonicalDigestBuilder, path_lowering::parse_decimal};
use starclock_combat::{
    LifeState, ParticipantSpec, PresenceState, ProgramId, Rounding, RuleBundleId, RuleId, Scalar,
    SelectorId, SourceDefinitionId, TeamSide, TriggerId,
    catalog::{
        builder::CombatCatalogBuilder,
        definition::{ProgramDefinition, RuleBundle, RuleDefinition, SelectorDefinition},
        selector::{
            RuleEmptyPoolPolicy, RuleLifePredicate, RulePresencePredicate, RuleSelectorChoice,
            RuleSelectorOrdering, RuleSelectorOrigin, RuleSelectorReference, RuleSelectorSide,
            RuleUnitSelector,
        },
    },
    modifier::model::{StatKind, StatQuerySubject},
    rule::model::{
        BattleRuleDefinition, ConditionExpr, EventFilter, OnceScope, ProgramStep, ReactionPriority,
        RuleDamageClass, RuleEventPoint, RuleOperationTemplate, RuleSource, RuleValue, SourceClass,
        TriggerDef, TriggerPhase, ValueExpr,
    },
};
use starclock_data::divergent_universe_decisions::{
    CurioBattleReactionDefinition, CurioBattleReactionPolicy,
};

#[derive(Clone, Debug)]
pub(super) struct CurioBattleReactions {
    definitions: Box<[CurioBattleReactionDefinition]>,
}

impl CurioBattleReactions {
    pub(super) fn new(factory: &DivergentUniverseRuntimeFactory) -> Self {
        Self {
            definitions: factory.decision_catalog().curio_battle_reactions().into(),
        }
    }

    pub(super) fn assemble(
        &self,
        builder: &mut CombatCatalogBuilder,
        snapshot: &DivergentUniverseCurioSnapshot,
        players: &mut [ParticipantSpec],
        assembly_digest: [u8; 32],
    ) -> Result<(), DivergentUniverseBattleAssemblyError> {
        let mut bindings = PassiveBindings::default();
        for (index, definition) in self.definitions.iter().enumerate() {
            if !snapshot
                .contributions()
                .iter()
                .any(|held| held.state() == &definition.state && held.charges() > 0)
            {
                continue;
            }
            match definition.policy {
                CurioBattleReactionPolicy::VersionedProjectPolicyFirstSurvivingOrdinaryAttackPerTargetAction => {}
            }
            let ordinal = u32::try_from(index)
                .ok()
                .and_then(|value| value.checked_add(1))
                .filter(|value| *value <= 65535)
                .ok_or_else(invalid)?;
            let program = ProgramId::new(0x7e30_0000 + ordinal).ok_or_else(invalid)?;
            let rule = RuleId::new(0x7e31_0000 + ordinal).ok_or_else(invalid)?;
            let bundle = RuleBundleId::new(0x7e32_0000 + ordinal).ok_or_else(invalid)?;
            let trigger = TriggerId::new(0x7e33_0000 + ordinal).ok_or_else(invalid)?;
            let source = SourceDefinitionId::new(0x7e34_0000 + ordinal).ok_or_else(invalid)?;
            let target = SelectorId::new(0x7e35_0000 + ordinal).ok_or_else(invalid)?;
            let opponents = SelectorId::new(0x7e36_0000 + ordinal).ok_or_else(invalid)?;
            let parameter = parse_decimal(&definition.heal_fraction).map_err(|_| invalid())?;
            let exponent = Scalar::FRACTIONAL_DIGITS
                .checked_sub(u32::from(parameter.scale()))
                .ok_or_else(invalid)?;
            let fraction = Scalar::from_scaled(
                parameter
                    .coefficient()
                    .checked_mul(10_i64.checked_pow(exponent).ok_or_else(invalid)?)
                    .ok_or_else(invalid)?,
            );
            let mut digest = CanonicalDigestBuilder::new();
            digest.update(b"starclock.divergent-universe.curio-battle-reaction-source");
            digest.update(assembly_digest);
            digest.update(definition.key.as_bytes());
            let source = RuleSource::new(source, SourceClass::Mode, Vec::new(), digest.finalize());
            builder.add_selector(SelectorDefinition::new(target).with_rule_units(selector(
                RuleSelectorOrigin::PrimaryTarget,
                RuleSelectorSide::Same,
                RuleLifePredicate::Alive,
                1,
            )?));
            builder.add_selector(SelectorDefinition::new(opponents).with_rule_units(selector(
                RuleSelectorOrigin::Team,
                RuleSelectorSide::Opposing,
                RuleLifePredicate::Any,
                64,
            )?));
            builder.add_program(
                ProgramDefinition::new(
                    program,
                    Vec::new(),
                    vec![target, opponents],
                    Vec::new(),
                    Vec::new(),
                )
                .with_steps(vec![ProgramStep::Operation(
                    RuleOperationTemplate::Heal {
                        selector: target,
                        // The shared base HP query reads current maximum HP, not
                        // current HP or the attacker's HP. Final healing floors HP.
                        amount: ValueExpr::Multiply {
                            lhs: Box::new(ValueExpr::QueryBaseStat {
                                subject: StatQuerySubject::EventTarget,
                                stat: StatKind::Hp,
                            }),
                            rhs: Box::new(ValueExpr::Literal(RuleValue::Scalar(fraction))),
                            rounding: Rounding::Floor,
                        },
                        apply_formula_modifiers: false,
                    },
                )]),
            );
            let condition = ConditionExpr::Any(
                [
                    PresenceState::Present,
                    PresenceState::Untargetable,
                    PresenceState::Linked,
                    PresenceState::Transformed,
                ]
                .map(|presence| ConditionExpr::LifePresence {
                    selector: target,
                    life: Some(LifeState::Alive),
                    presence: Some(presence),
                })
                .into(),
            );
            let runtime = BattleRuleDefinition::new(
                source.clone(),
                Vec::new(),
                vec![TriggerDef {
                    id: trigger,
                    event: RuleEventPoint::DamageApplied.kind(),
                    event_point: RuleEventPoint::DamageApplied,
                    phase: TriggerPhase::AfterEvent,
                    filter: EventFilter {
                        target_selector: Some(target),
                        applier_selector: Some(opponents),
                        source_class: Some(SourceClass::Ability),
                        damage_class: Some(RuleDamageClass::Ordinary),
                        has_action: Some(true),
                        ..EventFilter::default()
                    },
                    condition,
                    once_scope: OnceScope::TargetWithinAction,
                    priority: ReactionPriority::new(0),
                    program,
                }],
                None,
            );
            builder.add_rule(
                RuleDefinition::new(rule, vec![program], vec![target, opponents])
                    .with_runtime(runtime),
            );
            builder.add_rule_bundle(RuleBundle::new(bundle, vec![rule]));
            bindings.rule_bundles.push(bundle);
            bindings.sources.push(source);
        }
        if bindings.rule_bundles.is_empty() {
            return Ok(());
        }
        if players
            .iter()
            .any(|player| player.side() != TeamSide::Player)
        {
            return Err(invalid());
        }
        // One immutable party anchor avoids duplicate reactions. Its life state
        // is not an eligibility condition for this run-owned passive.
        let anchor = players
            .iter_mut()
            .min_by_key(|player| player.formation())
            .ok_or_else(invalid)?;
        *anchor = bind_passives(anchor, &bindings, assembly_digest)?;
        Ok(())
    }
}

fn selector(
    origin: RuleSelectorOrigin,
    side: RuleSelectorSide,
    life: RuleLifePredicate,
    maximum: u16,
) -> Result<RuleUnitSelector, DivergentUniverseBattleAssemblyError> {
    RuleUnitSelector::new(
        origin,
        side,
        life,
        RulePresencePredicate::Any,
        RuleSelectorReference::CurrentState,
        RuleSelectorOrdering::StableId,
        0,
        maximum,
        RuleEmptyPoolPolicy::NoOp,
        RuleSelectorChoice::All,
        None,
        false,
    )
    .ok_or_else(invalid)
}
fn invalid() -> DivergentUniverseBattleAssemblyError {
    DivergentUniverseBattleAssemblyError::InvalidPlayerParticipants
}
