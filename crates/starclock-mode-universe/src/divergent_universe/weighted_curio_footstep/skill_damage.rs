//! Independently replaceable after-Skill stacks, also bound by authored equipment.
use super::{invalid, literal, multiply, select};
use crate::{
    digest::Encoder,
    divergent_universe::{
        DivergentUniverseBattleAssemblyError,
        battle_passive_bindings::{PassiveBindings, bind_passives},
    },
};
use starclock_build::light_cone::CombatPath;
use starclock_combat::{
    DispelCategory, DurationClock, EffectCategory, EffectDefinitionId, EffectRuntimeTemplate,
    EffectStackPolicy, EffectTeardownPolicy, EffectTickPhase, ModifierDefinitionId,
    ModifierStackingGroupId, ParticipantSpec, ProgramId, Rounding, RuleBundleId, RuleId, Scalar,
    SelectorId, SourceDefinitionId, StateSlotDefinitionId, TeamSide, TriggerId,
    catalog::{
        builder::CombatCatalogBuilder,
        definition::{
            EffectDefinition, ProgramDefinition, RuleBundle, RuleDefinition, SelectorDefinition,
        },
        selector::{RulePresencePredicate, RuleSelectorOrigin, RuleSelectorPredicate},
    },
    damage::DamageProducer,
    modifier::model::{
        FormulaPurpose, FormulaStage, ModifierAggregation, ModifierDefinition, ModifierFilter,
        ModifierStackingGroup, SnapshotPolicy, StatKind,
    },
    rule::model::{
        BattleRuleDefinition, Comparison, ConditionExpr, EventFilter, OnceScope, ProgramStep,
        ReactionPriority, RuleActionKind, RuleEffectChancePolicy, RuleEventPoint,
        RuleOperationTemplate, RuleSource, RuleValue, RuleValueKind, SourceClass, TriggerDef,
        TriggerPhase, ValueExpr,
    },
};
use starclock_data::catalog::SimulationCatalog;

/// Invalid caller operands are rejected before catalog mutation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SkillDamagePolicyError {
    InvalidDamagePerStack,
    InvalidMaximumStacks,
    UnrepresentableMaximumBonus,
}
impl core::fmt::Display for SkillDamagePolicyError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "Skill damage policy error: {self:?}")
    }
}
impl std::error::Error for SkillDamagePolicyError {}

/// Immutable operands and caller identity; no production default or live inputs.
#[derive(Clone, Debug)]
pub struct SkillDamagePolicy {
    damage_per_stack: Scalar,
    maximum_stacks: u16,
    identity: [u8; 32],
}
impl SkillDamagePolicy {
    /// Bind a positive additive damage ratio, nonzero stack cap and external
    /// provenance/policy digest. The maximum product must fit checked Scalar.
    pub fn new(
        damage_per_stack: Scalar,
        maximum_stacks: u16,
        identity: [u8; 32],
    ) -> Result<Self, SkillDamagePolicyError> {
        if damage_per_stack <= Scalar::ZERO {
            return Err(SkillDamagePolicyError::InvalidDamagePerStack);
        }
        if maximum_stacks == 0 {
            return Err(SkillDamagePolicyError::InvalidMaximumStacks);
        }
        Scalar::checked_from_integer(i64::from(maximum_stacks))
            .and_then(|count| count.checked_mul(damage_per_stack, Rounding::Floor))
            .map_err(|_| SkillDamagePolicyError::UnrepresentableMaximumBonus)?;
        Ok(Self {
            damage_per_stack,
            maximum_stacks,
            identity,
        })
    }
}

/// Compile the Skill clause for one original Destruction/Remembrance player.
///
/// VersionedProjectPolicy: after one complete Skill ActionResolved, priority
/// zero and OnceScope::Action, add one permanent, non-dispellable buff stack
/// up to the caller cap. Nondamaging Skills count; individual hits/targets and
/// other action kinds do not. The completed action cannot receive its new
/// stack retroactively. Each eligible original has separate effect ownership.
/// Present/transformed anchoring and formation proof exclude inherited linked
/// actors and unrelated originals. Original-unit formula filters also prevent
/// borrowing the bonus through a linked/timeline actor's statistic fallback.
///
/// Dynamic additive DamageBoost uses the shared Ordinary/DoT/Additional
/// channels, not True/Break/SuperBreak or the dedicated Elation calculator.
/// Removal/reapplication reuses the shared effect and modifier lifecycle; waves
/// retain stacks, win/loss removes them, and fresh battles start with none.
/// Checked faults roll back the command; there are no RNG draws or native
/// handlers. Discard a builder on construction error (append may have begun).
/// This API does not admit normal equipment, Forge or full-run source parity.
pub fn bind_skill_damage_policy(
    builder: &mut CombatCatalogBuilder,
    core: &SimulationCatalog,
    player: &ParticipantSpec,
    policy: &SkillDamagePolicy,
    assembly_digest: [u8; 32],
) -> Result<ParticipantSpec, DivergentUniverseBattleAssemblyError> {
    if player.side() != TeamSide::Player || player.formation().get() > 3 {
        return Err(invalid());
    }
    let character = core
        .build_catalog()
        .character(player.combatant().form())
        .ok_or_else(invalid)?;
    if !matches!(
        character.path(),
        CombatPath::Destruction | CombatPath::Remembrance
    ) {
        return Ok(player.clone());
    }
    let ordinal = u32::from(player.formation().get()) + 1;
    let raw = |base: u32, offset: u32| {
        ordinal
            .checked_mul(16)
            .and_then(|value| value.checked_add(base))
            .and_then(|value| value.checked_add(offset))
            .ok_or_else(invalid)
    };
    let selector = |offset| SelectorId::new(raw(0x7f27_0000, offset)?).ok_or_else(invalid);
    let owner = selector(0)?;
    let team = selector(1)?;
    let linked = selector(2)?;
    let present = selector(3)?;
    let transformed = selector(4)?;
    let all = select(
        RuleSelectorOrigin::Team,
        RulePresencePredicate::Any,
        u16::MAX,
    )?;
    builder.add_selector(SelectorDefinition::new(team).with_rule_units(all.clone()));
    builder.add_selector(
        SelectorDefinition::new(linked)
            .with_rule_units(all.with_predicates(vec![RuleSelectorPredicate::OwnedBy(team)])),
    );
    for (id, presence) in [
        (present, RulePresencePredicate::Present),
        (transformed, RulePresencePredicate::Transformed),
    ] {
        builder.add_selector(SelectorDefinition::new(id).with_rule_units(select(
            RuleSelectorOrigin::Owner,
            presence,
            1,
        )?));
    }
    builder.add_selector(
        SelectorDefinition::new(owner).with_rule_units(
            select(RuleSelectorOrigin::Owner, RulePresencePredicate::Any, 1)?
                .with_candidate_union(vec![present, transformed])
                .ok_or_else(invalid)?
                .with_predicates(vec![
                    RuleSelectorPredicate::FormationRange {
                        minimum: player.formation().get(),
                        maximum: player.formation().get(),
                    },
                    RuleSelectorPredicate::Excludes(linked),
                ]),
        ),
    );
    let selectors = vec![owner, team, linked, present, transformed];
    let grant = ProgramId::new(raw(0x7f28_0000, 0)?).ok_or_else(invalid)?;
    let reset = ProgramId::new(raw(0x7f28_0000, 1)?).ok_or_else(invalid)?;
    let rule = RuleId::new(raw(0x7f29_0000, 0)?).ok_or_else(invalid)?;
    let bundle = RuleBundleId::new(raw(0x7f2a_0000, 0)?).ok_or_else(invalid)?;
    let source_id = SourceDefinitionId::new(raw(0x7f2b_0000, 0)?).ok_or_else(invalid)?;
    let effect = EffectDefinitionId::new(raw(0x7f2c_0000, 0)?).ok_or_else(invalid)?;
    let group = ModifierStackingGroupId::new(raw(0x7f2e_0000, 0)?).ok_or_else(invalid)?;
    let stack_slot = StateSlotDefinitionId::new(raw(0x7f2f_0000, 0)?).ok_or_else(invalid)?;
    builder.add_modifier_group(ModifierStackingGroup {
        id: group,
        aggregation: ModifierAggregation::UniquePerSource,
        comparator: None,
    });
    let mut modifiers = vec![];
    for (offset, purpose) in [
        FormulaPurpose::OrdinaryDamage,
        FormulaPurpose::Dot,
        FormulaPurpose::AdditionalDamage,
    ]
    .into_iter()
    .enumerate()
    {
        let id = ModifierDefinitionId::new(raw(
            0x7f2d_0000,
            u32::try_from(offset).map_err(|_| invalid())?,
        )?)
        .ok_or_else(invalid)?;
        builder.add_modifier(ModifierDefinition {
            id,
            stat: StatKind::Atk,
            stage: FormulaStage::DamageBoost,
            purpose,
            value: multiply(
                literal(policy.damage_per_stack),
                ValueExpr::Convert {
                    value: Box::new(ValueExpr::Slot(stack_slot)),
                    target: RuleValueKind::Scalar,
                    rounding: Rounding::Floor,
                },
            ),
            stacking_group: group,
            priority: 0,
            floor: None,
            cap: None,
            cap_stage: FormulaStage::DamageBoost,
            snapshot: SnapshotPolicy::Dynamic,
            source_stack_slot: Some(stack_slot),
            filters: vec![ModifierFilter::DamageProducer(DamageProducer::OriginalUnit)]
                .into_boxed_slice(),
        });
        modifiers.push(id);
    }
    builder.add_effect(
        EffectDefinition::new(effect, vec![], modifiers).with_runtime_template(
            EffectRuntimeTemplate::new(
                EffectCategory::Buff,
                DispelCategory::NonDispellable,
                policy.maximum_stacks,
                None,
                DurationClock::Permanent,
                EffectTickPhase::None,
                EffectStackPolicy::RefreshAndAddStacks,
            )
            .ok_or_else(invalid)?
            .with_teardown(EffectTeardownPolicy::PersistByScope),
        ),
    );
    builder.add_program(
        ProgramDefinition::new(grant, vec![], selectors.clone(), vec![effect], vec![]).with_steps(
            vec![ProgramStep::Operation(RuleOperationTemplate::ApplyEffect {
                selector: owner,
                effect,
                stacks: ValueExpr::Literal(RuleValue::Integer(1)),
                chance: RuleEffectChancePolicy::Guaranteed,
                base_chance: None,
                rng_purpose: None,
            })],
        ),
    );
    // A direct owner selector clears even absent/defeated instances. It is not
    // the original-presence eligibility selector used to acquire stacks.
    let cleanup = selector(5)?;
    builder.add_selector(SelectorDefinition::new(cleanup).with_rule_units(select(
        RuleSelectorOrigin::Owner,
        RulePresencePredicate::Any,
        1,
    )?));
    builder.add_program(
        ProgramDefinition::new(reset, vec![], vec![cleanup], vec![effect], vec![]).with_steps(
            vec![ProgramStep::Operation(
                RuleOperationTemplate::RemoveEffect {
                    selector: cleanup,
                    effect,
                },
            )],
        ),
    );
    let mut digest =
        Encoder::new(b"starclock.divergent-universe.footstep.after-skill-original-damage-policy");
    digest.digest(assembly_digest);
    digest.digest(policy.identity);
    digest.i64(policy.damage_per_stack.scaled());
    digest.u32(u32::from(policy.maximum_stacks));
    digest.digest(
        core.build_catalog()
            .character_digest(player.combatant().form())
            .ok_or_else(invalid)?
            .bytes(),
    );
    digest.u8(player.formation().get());
    let identity = digest.finish();
    let source = RuleSource::new(source_id, SourceClass::Mode, vec![], identity);
    let mut triggers = vec![TriggerDef {
        id: TriggerId::new(raw(0x7f26_0000, 4)?).ok_or_else(invalid)?,
        event: RuleEventPoint::ActionResolved.kind(),
        event_point: RuleEventPoint::ActionResolved,
        phase: TriggerPhase::AfterAction,
        filter: EventFilter {
            actor_selector: Some(owner),
            action_kind: Some(RuleActionKind::Skill),
            has_action: Some(true),
            ..EventFilter::default()
        },
        condition: ConditionExpr::SelectorCardinality {
            selector: owner,
            operator: Comparison::Equal,
            count: 1,
        },
        once_scope: OnceScope::Action,
        priority: ReactionPriority::new(0),
        program: grant,
    }];
    for (offset, point) in [
        (5, RuleEventPoint::BattleWon),
        (6, RuleEventPoint::BattleLost),
    ] {
        triggers.push(TriggerDef {
            id: TriggerId::new(raw(0x7f26_0000, offset)?).ok_or_else(invalid)?,
            event: point.kind(),
            event_point: point,
            phase: TriggerPhase::AfterEvent,
            filter: EventFilter::default(),
            condition: ConditionExpr::Literal(true),
            once_scope: OnceScope::Event,
            priority: ReactionPriority::new(0),
            program: reset,
        });
    }
    let mut rule_selectors = selectors;
    rule_selectors.push(cleanup);
    builder.add_rule(
        RuleDefinition::new(rule, vec![grant, reset], rule_selectors).with_runtime(
            BattleRuleDefinition::new(source.clone(), vec![], triggers, None),
        ),
    );
    builder.add_rule_bundle(RuleBundle::new(bundle, vec![rule]));
    bind_passives(
        player,
        &PassiveBindings {
            rule_bundles: vec![bundle],
            sources: vec![source],
            ..PassiveBindings::default()
        },
        identity,
    )
}
