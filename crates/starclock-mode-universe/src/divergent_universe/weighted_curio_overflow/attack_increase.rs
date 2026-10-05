//! Original-roster ATK policy; never static inheritable participant modifiers.
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
    ModifierStackingGroupId, ParticipantSpec, ProgramId, RuleBundleId, RuleId, Scalar, SelectorId,
    SourceDefinitionId, TeamSide, TriggerId,
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
        BattleRuleDefinition, Comparison, ConditionExpr, EventFilter, OnceScope, ProgramStep,
        ReactionPriority, RuleEffectChancePolicy, RuleEventPoint, RuleOperationTemplate,
        RuleSource, RuleValue, SourceClass, TriggerDef, TriggerPhase, ValueExpr,
    },
};
use starclock_data::{
    catalog::SimulationCatalog,
    divergent_universe_decisions::weighted_curio_overflows::WeightedCurioOverflowDefinition,
};

/// Construct the authored ATK contribution for one original player (formation
/// 0..=3). The immutable entry form must resolve in the supplied build catalog;
/// Hunt/Erudition qualify, other Paths return the unchanged participant without
/// appending definitions. Never inspect a live transformed form for eligibility.
///
/// At BattleStarted, one non-dispellable permanent effect adds 80% of base ATK
/// to the original unit. It persists through transformations and waves and is
/// explicitly removed at BattleWon/Lost. Owned linked units cannot receive it,
/// even when they inherit bundles, sources, form and formation. Borrowing the
/// original unit's actual ATK still follows the shared stat-query contract.
/// This explicit construction API does not admit pending Activity equipment.
/// On any construction error discard the builder; definitions may be appended
/// before passive validation fails. No live battle or Activity is mutated.
pub fn bind_attack_increase_policy(
    builder: &mut CombatCatalogBuilder,
    definition: &WeightedCurioOverflowDefinition,
    core: &SimulationCatalog,
    player: &ParticipantSpec,
    assembly_digest: [u8; 32],
) -> Result<ParticipantSpec, DivergentUniverseBattleAssemblyError> {
    if player.side() != TeamSide::Player
        || player.formation().get() > 3
        || definition.attack_increase_millionths != 800_000
    {
        return Err(invalid());
    }
    let character = core
        .build_catalog()
        .character(player.combatant().form())
        .ok_or_else(invalid)?;
    if !matches!(character.path(), CombatPath::Hunt | CombatPath::Erudition) {
        return Ok(player.clone());
    }
    let ordinal = u32::from(player.formation().get()) + 1;
    let raw = |base: u32, offset: u32| {
        ordinal
            .checked_mul(32)
            .and_then(|n| n.checked_add(base))
            .and_then(|n| n.checked_add(offset))
            .ok_or_else(invalid)
    };
    let selector = |offset| SelectorId::new(raw(0x7f09_0000, offset)?).ok_or_else(invalid);
    let owner = selector(0)?;
    let team = selector(1)?;
    let linked = selector(2)?;
    let present = selector(3)?;
    let transformed = selector(4)?;
    let teardown = selector(5)?;
    let apply = ProgramId::new(raw(0x7f0a_0000, 0)?).ok_or_else(invalid)?;
    let remove = ProgramId::new(raw(0x7f0a_0000, 1)?).ok_or_else(invalid)?;
    let rule = RuleId::new(raw(0x7f0b_0000, 0)?).ok_or_else(invalid)?;
    let bundle = RuleBundleId::new(raw(0x7f0c_0000, 0)?).ok_or_else(invalid)?;
    let source_id = SourceDefinitionId::new(raw(0x7f0d_0000, 0)?).ok_or_else(invalid)?;
    let effect = EffectDefinitionId::new(raw(0x7f0e_0000, 0)?).ok_or_else(invalid)?;
    let modifier = ModifierDefinitionId::new(raw(0x7f0f_0000, 0)?).ok_or_else(invalid)?;
    let group = ModifierStackingGroupId::new(raw(0x7f10_0000, 0)?).ok_or_else(invalid)?;
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
    builder.add_selector(SelectorDefinition::new(teardown).with_rule_units(
        select(RuleSelectorOrigin::Owner, RulePresencePredicate::Any, 1)?.with_predicates(vec![
            RuleSelectorPredicate::FormationRange {
                minimum: player.formation().get(),
                maximum: player.formation().get(),
            },
            RuleSelectorPredicate::Excludes(linked),
        ]),
    ));
    builder.add_modifier_group(ModifierStackingGroup {
        id: group,
        aggregation: ModifierAggregation::UniquePerSource,
        comparator: None,
    });
    builder.add_modifier(ModifierDefinition {
        id: modifier,
        stat: StatKind::Atk,
        stage: FormulaStage::PercentOfBase,
        purpose: FormulaPurpose::Stat,
        value: ValueExpr::Literal(RuleValue::Scalar(Scalar::from_scaled(
            definition.attack_increase_millionths,
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
        EffectDefinition::new(effect, vec![], vec![modifier]).with_runtime_template(
            EffectRuntimeTemplate::new(
                EffectCategory::Buff,
                DispelCategory::NonDispellable,
                1,
                None,
                DurationClock::Permanent,
                EffectTickPhase::None,
                EffectStackPolicy::Replace,
            )
            .ok_or_else(invalid)?
            .with_teardown(EffectTeardownPolicy::PersistByScope),
        ),
    );
    let selectors = vec![owner, team, linked, present, transformed, teardown];
    for (id, operation) in [
        (
            apply,
            RuleOperationTemplate::ApplyEffect {
                selector: owner,
                effect,
                stacks: ValueExpr::Literal(RuleValue::Integer(1)),
                chance: RuleEffectChancePolicy::Guaranteed,
                base_chance: None,
                rng_purpose: None,
            },
        ),
        (
            remove,
            RuleOperationTemplate::RemoveEffect {
                selector: teardown,
                effect,
            },
        ),
    ] {
        let target = if id == apply { owner } else { teardown };
        builder.add_program(
            ProgramDefinition::new(id, vec![], vec![target], vec![effect], vec![])
                .with_steps(vec![ProgramStep::Operation(operation)]),
        );
    }
    let mut digest =
        Encoder::new(b"starclock.divergent-universe.overflow.original-roster-atk-policy");
    digest.digest(assembly_digest);
    digest.text(&definition.key);
    digest.i64(definition.attack_increase_millionths);
    digest.digest(
        core.build_catalog()
            .character_digest(player.combatant().form())
            .ok_or_else(invalid)?
            .bytes(),
    );
    let identity = digest.finish();
    let source = RuleSource::new(source_id, SourceClass::Mode, vec![], identity);
    let mut triggers = Vec::new();
    for (offset, point, program, once_scope) in [
        (0, RuleEventPoint::BattleStarted, apply, OnceScope::Battle),
        (1, RuleEventPoint::BattleWon, remove, OnceScope::Event),
        (2, RuleEventPoint::BattleLost, remove, OnceScope::Event),
    ] {
        triggers.push(TriggerDef {
            id: TriggerId::new(raw(0x7f11_0000, offset)?).ok_or_else(invalid)?,
            event: point.kind(),
            event_point: point,
            phase: TriggerPhase::AfterEvent,
            filter: EventFilter::default(),
            condition: ConditionExpr::SelectorCardinality {
                selector: if program == apply { owner } else { teardown },
                operator: Comparison::Equal,
                count: 1,
            },
            once_scope,
            priority: ReactionPriority::new(0),
            program,
        });
    }
    builder.add_rule(
        RuleDefinition::new(rule, vec![apply, remove], selectors).with_runtime(
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

fn select(
    origin: RuleSelectorOrigin,
    presence: RulePresencePredicate,
    maximum: u16,
) -> Result<RuleUnitSelector, DivergentUniverseBattleAssemblyError> {
    RuleUnitSelector::new(
        origin,
        RuleSelectorSide::Same,
        RuleLifePredicate::Any,
        presence,
        RuleSelectorReference::CurrentState,
        RuleSelectorOrdering::Formation,
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
