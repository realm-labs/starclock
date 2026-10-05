//! Genuine same-owner queued attacks, not direct recursive state mutation.
use crate::divergent_universe::{
    battle_passive_bindings::{PassiveBindings, bind_passives},
    tests::weighted_curio_overflow_fixture::id,
};
use starclock_combat::{
    Energy, ParticipantSpec, Ratio, Scalar,
    catalog::{
        action::{
            AbilityActionDefinition, AbilityKind, AbilityTag, ActionHitDefinition,
            ActionResourcePolicy, HitCritPolicy, HitOperationDefinition, HitTargetGroup,
            OrdinaryDamageDefinition, OrdinaryDamageMultipliers, ReactionBoundary,
            TargetInvalidationPolicy, TargetPattern, TargetRelation, UnitTargetSelector,
        },
        builder::CombatCatalogBuilder,
        definition::{
            AbilityDefinition, ProgramDefinition, RuleBundle, RuleDefinition, SelectorDefinition,
        },
        selector::{
            RuleEmptyPoolPolicy, RuleLifePredicate, RulePresencePredicate, RuleSelectorChoice,
            RuleSelectorOrdering, RuleSelectorOrigin, RuleSelectorReference, RuleSelectorSide,
            RuleUnitSelector,
        },
    },
    rule::model::{
        BattleRuleDefinition, ConditionExpr, EventFilter, OnceScope, ProgramStep, ReactionPriority,
        RuleActionOwner, RuleEventPoint, RuleOperationTemplate, RuleSource, SourceClass,
        TriggerDef, TriggerPhase,
    },
};

pub(super) fn bind_queued_probe(
    builder: &mut CombatCatalogBuilder,
    player: &ParticipantSpec,
    boundary: ReactionBoundary,
    single: bool,
) -> ParticipantSpec {
    let owner = id(21);
    let targets = id(22);
    for (selector, origin, side, maximum, choice) in [
        (
            owner,
            RuleSelectorOrigin::Owner,
            RuleSelectorSide::Same,
            1,
            RuleSelectorChoice::First,
        ),
        (
            targets,
            RuleSelectorOrigin::Encounter,
            RuleSelectorSide::Opposing,
            if single { 1 } else { 64 },
            if single {
                RuleSelectorChoice::First
            } else {
                RuleSelectorChoice::All
            },
        ),
    ] {
        builder.add_selector(
            SelectorDefinition::new(selector).with_rule_units(
                RuleUnitSelector::new(
                    origin,
                    side,
                    RuleLifePredicate::Alive,
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
                .unwrap(),
            ),
        );
    }
    builder.add_selector(
        SelectorDefinition::new(id(8)).with_unit_targets(
            UnitTargetSelector::new(
                TargetRelation::Opposing,
                if single {
                    TargetPattern::Single
                } else {
                    TargetPattern::All
                },
            )
            .unwrap(),
        ),
    );
    builder.add_ability(
        AbilityDefinition::new(id(8), id(1), id(8), vec![]).with_action(
            AbilityActionDefinition::new(
                AbilityKind::FollowUp,
                1,
                TargetInvalidationPolicy::CancelRemainingForTarget,
                ActionResourcePolicy::new(0, 0, Energy::ZERO, Energy::ZERO),
            )
            .unwrap()
            .with_tags(&[AbilityTag::Attack, AbilityTag::FollowUp])
            .with_hits(vec![
                ActionHitDefinition::new(vec![HitOperationDefinition::Damage(
                    OrdinaryDamageDefinition::new(
                        Scalar::checked_from_integer(250).unwrap(),
                        OrdinaryDamageMultipliers::new([Ratio::ONE; 9]).unwrap(),
                    )
                    .unwrap(),
                )])
                .with_profile(
                    HitTargetGroup::Selected,
                    Ratio::ONE,
                    Ratio::ONE,
                    HitCritPolicy::Never,
                ),
            ])
            .unwrap(),
        ),
    );
    builder.add_program(
        ProgramDefinition::new(id(21), vec![], vec![owner, targets], vec![], vec![]).with_steps(
            vec![ProgramStep::Operation(RuleOperationTemplate::QueueAction {
                actor_selector: owner,
                target_selector: targets,
                ability: id(8),
                priority: ReactionPriority::new(0),
                forced_use: false,
                boundary,
                owner: RuleActionOwner::Actor,
                payment: None,
            })],
        ),
    );
    let identity = [0x91 + boundary as u8 + u8::from(single) * 4; 32];
    let source = RuleSource::new(id(21), SourceClass::Mode, vec![], identity);
    builder.add_rule(
        RuleDefinition::new(id(21), vec![id(21)], vec![owner, targets]).with_runtime(
            BattleRuleDefinition::new(
                source.clone(),
                vec![],
                vec![TriggerDef {
                    id: id(21),
                    event: RuleEventPoint::HitEnded.kind(),
                    event_point: RuleEventPoint::HitEnded,
                    phase: TriggerPhase::AfterEvent,
                    filter: EventFilter {
                        actor_selector: Some(owner),
                        applier_selector: Some(owner),
                        source_class: Some(SourceClass::Ability),
                        ability_tag: Some(AbilityTag::Basic),
                        has_action: Some(true),
                        ..EventFilter::default()
                    },
                    condition: ConditionExpr::Literal(true),
                    once_scope: OnceScope::Battle,
                    priority: ReactionPriority::new(1),
                    program: id(21),
                }],
                None,
            ),
        ),
    );
    builder.add_rule_bundle(RuleBundle::new(id(21), vec![id(21)]));
    bind_passives(
        player,
        &PassiveBindings {
            rule_bundles: vec![id(21)],
            sources: vec![source],
            ..PassiveBindings::default()
        },
        identity,
    )
    .unwrap()
}
