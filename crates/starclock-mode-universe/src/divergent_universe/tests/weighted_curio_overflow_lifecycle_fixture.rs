//! Accepted lifecycle setup; linked combatants deliberately inherit the bridge.
use crate::divergent_universe::tests::weighted_curio_overflow_fixture::{
    combatant_with_abilities, id,
};
use starclock_combat::{
    ActionGauge, CountdownCatalogDefinition, CountdownDefinition, Energy, FormationIndex,
    LinkedEntityKind, LinkedUnitDefinition, OwnerLinkPolicy, ParticipantSpec, PresenceState, Ratio,
    Scalar, Speed, TransformEndPolicy, TransformationDefinition, WaveLinkPolicy,
    catalog::{
        action::{
            AbilityActionDefinition, AbilityKind, AbilityProgramBinding, AbilityProgramTiming,
            AbilityTag, ActionHitDefinition, ActionResourcePolicy, HitCritPolicy,
            HitOperationDefinition, HitTargetGroup, OrdinaryDamageDefinition,
            OrdinaryDamageMultipliers, ReactionBoundary, TargetInvalidationPolicy, TargetPattern,
            TargetRelation, UnitTargetSelector,
        },
        builder::CombatCatalogBuilder,
        definition::{AbilityDefinition, ProgramDefinition, SelectorDefinition},
        selector::{
            RuleEmptyPoolPolicy, RuleLifePredicate, RulePresencePredicate, RuleSelectorChoice,
            RuleSelectorOrdering, RuleSelectorOrigin, RuleSelectorPredicate, RuleSelectorReference,
            RuleSelectorSide, RuleUnitSelector,
        },
    },
    rule::model::{
        ProgramStep, ReactionPriority, RuleActionOwner, RuleActionPaymentPolicy,
        RuleOperationTemplate,
    },
};

#[derive(Clone, Copy)]
pub(super) enum Setup {
    Transform,
    Linked(LinkedEntityKind, u8),
    Countdown,
}

pub(super) fn add_setup(
    builder: &mut CombatCatalogBuilder,
    player: &ParticipantSpec,
    setup: Setup,
) {
    builder.add_selector(SelectorDefinition::new(id(3)).with_unit_targets(
        UnitTargetSelector::new(TargetRelation::SelfUnit, TargetPattern::Single).unwrap(),
    ));
    let countdown = CountdownDefinition::new(
        id(5),
        ActionGauge::from_scaled(100_000_000).unwrap(),
        Speed::from_scaled(500_000_000).unwrap(),
        OwnerLinkPolicy::Depart,
        OwnerLinkPolicy::Depart,
        WaveLinkPolicy::Persist,
    );
    let operation = match setup {
        Setup::Transform | Setup::Countdown => HitOperationDefinition::Transform(
            TransformationDefinition::new(
                id(3),
                vec![id(1), id(4)],
                matches!(setup, Setup::Countdown).then_some(countdown),
                TransformEndPolicy::End,
                TransformEndPolicy::End,
            )
            .unwrap(),
        ),
        Setup::Linked(kind, formation) => {
            builder.add_ability(ability(
                6,
                1,
                match kind {
                    LinkedEntityKind::Summon => AbilityKind::Summon,
                    LinkedEntityKind::Memosprite => AbilityKind::Memosprite,
                    _ => AbilityKind::ExtraAction,
                },
                damage(),
            ));
            let original = player.combatant();
            // Same form, bundles and sources as the original. Only the accepted
            // link/formation/runtime identity separates this unit from its owner.
            let spec = combatant_with_abilities(
                1,
                13,
                10_000,
                true,
                original.rule_bundles().to_vec(),
                vec![id(6)],
            )
            .with_sources(original.sources().to_vec())
            .unwrap();
            HitOperationDefinition::SummonLinked(Box::new(
                LinkedUnitDefinition::new(
                    spec,
                    id(0x7f08_0000),
                    FormationIndex::new(formation).unwrap(),
                    kind,
                    PresenceState::Present,
                    (kind != LinkedEntityKind::SharedActor).then(|| id(6)),
                    ActionGauge::from_scaled(100_000_000).unwrap(),
                    OwnerLinkPolicy::Persist,
                    OwnerLinkPolicy::Persist,
                    WaveLinkPolicy::Persist,
                )
                .unwrap(),
            ))
        }
    };
    let mut setup_ability = ability(3, 3, AbilityKind::Skill, vec![operation]);
    if let Setup::Linked(LinkedEntityKind::SharedActor, formation) = setup {
        add_shared_attack(builder, formation);
        setup_ability = setup_ability.with_programs(vec![
            AbilityProgramBinding::new(1, AbilityProgramTiming::AfterHits, id(11)).unwrap(),
        ]);
    }
    builder.add_ability(setup_ability);
    builder.add_ability(ability(
        4,
        3,
        AbilityKind::Skill,
        vec![HitOperationDefinition::EndTransformation],
    ));
    if matches!(setup, Setup::Countdown) {
        builder.add_countdown(CountdownCatalogDefinition::new(81, countdown).unwrap());
        builder.add_ability(ability(5, 1, AbilityKind::Countdown, damage()));
    }
}

fn add_shared_attack(builder: &mut CombatCatalogBuilder, formation: u8) {
    for (raw, origin, side, predicates) in [
        (
            11,
            RuleSelectorOrigin::Team,
            RuleSelectorSide::Same,
            vec![
                RuleSelectorPredicate::FormationRange {
                    minimum: formation,
                    maximum: formation,
                },
                RuleSelectorPredicate::OwnedBy(id(13)),
            ],
        ),
        (
            12,
            RuleSelectorOrigin::Encounter,
            RuleSelectorSide::Opposing,
            vec![],
        ),
        (
            13,
            RuleSelectorOrigin::Owner,
            RuleSelectorSide::Same,
            vec![],
        ),
    ] {
        builder.add_selector(
            SelectorDefinition::new(id(raw)).with_rule_units(
                RuleUnitSelector::new(
                    origin,
                    side,
                    RuleLifePredicate::Alive,
                    RulePresencePredicate::Present,
                    RuleSelectorReference::CurrentState,
                    RuleSelectorOrdering::Formation,
                    0,
                    if raw == 12 { 64 } else { 1 },
                    RuleEmptyPoolPolicy::NoOp,
                    if raw == 12 {
                        RuleSelectorChoice::All
                    } else {
                        RuleSelectorChoice::First
                    },
                    None,
                    false,
                )
                .unwrap()
                .with_predicates(predicates),
            ),
        );
    }
    builder.add_program(
        ProgramDefinition::new(id(11), vec![], vec![id(11), id(12)], vec![], vec![]).with_steps(
            vec![ProgramStep::Operation(RuleOperationTemplate::QueueAction {
                actor_selector: id(11),
                target_selector: id(12),
                ability: id(6),
                priority: ReactionPriority::new(0),
                forced_use: true,
                boundary: ReactionBoundary::AfterAction,
                owner: RuleActionOwner::Actor,
                payment: Some(RuleActionPaymentPolicy::Suppressed),
            })],
        ),
    );
}

fn ability(
    raw: u32,
    selector: u32,
    kind: AbilityKind,
    operations: Vec<HitOperationDefinition>,
) -> AbilityDefinition {
    AbilityDefinition::new(id(raw), id(1), id(selector), vec![]).with_action(
        AbilityActionDefinition::new(
            kind,
            1,
            TargetInvalidationPolicy::CancelRemainingForTarget,
            ActionResourcePolicy::new(0, 0, Energy::ZERO, Energy::ZERO),
        )
        .unwrap()
        .with_tags(&[AbilityTag::Attack])
        .with_hits(vec![ActionHitDefinition::new(operations).with_profile(
            HitTargetGroup::Selected,
            Ratio::ONE,
            Ratio::ONE,
            HitCritPolicy::Never,
        )])
        .unwrap(),
    )
}

fn damage() -> Vec<HitOperationDefinition> {
    vec![HitOperationDefinition::Damage(
        OrdinaryDamageDefinition::new(
            Scalar::checked_from_integer(250).unwrap(),
            OrdinaryDamageMultipliers::new([Ratio::ONE; 9]).unwrap(),
        )
        .unwrap(),
    )]
}
