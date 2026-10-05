//! Typed lifecycle actions; inherited bindings are never reconstructed here.
use crate::divergent_universe::tests::weighted_curio_burn_fixture::{ATTACK, IDLE};
use starclock_combat::{
    ActionGauge, CombatantSpecDigest, CountdownCatalogDefinition, CountdownDefinition, Energy,
    FormationIndex, Hp, LinkedEntityKind, LinkedUnitDefinition, OwnerLinkPolicy, ParticipantSpec,
    PresenceState, Ratio, ResolvedCombatantSpec, ResolvedDefinitionBindings, Scalar, Speed,
    TransformEndPolicy, TransformationDefinition, UnitDefinitionId, WaveLinkPolicy,
    catalog::{
        CombatCatalog,
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

pub(super) const SETUP: u32 = 0x7dd3_0001;
pub(super) const END_TRANSFORM: u32 = 0x7dd3_0002;
pub(super) const LINKED_ATTACK: u32 = 0x7dd3_0003;
pub(super) const COUNTDOWN: u32 = 0x7dd3_0004;
const LINKED_ACTION: u32 = 0x7dd3_0005;
const OWNER: u32 = 0x7dd1_0001;
const RECEIVER: u32 = 0x7dd1_0002;
const TARGET: u32 = 0x7dd1_0003;

#[derive(Clone, Copy)]
pub(super) enum Setup {
    Transform {
        form: UnitDefinitionId,
        countdown: bool,
    },
    Linked {
        kind: LinkedEntityKind,
        formation: u8,
    },
}
fn id<I: TryFrom<u32>>(raw: u32) -> I
where
    I::Error: core::fmt::Debug,
{
    I::try_from(raw).unwrap()
}
fn selector(
    builder: &mut CombatCatalogBuilder,
    raw: u32,
    origin: RuleSelectorOrigin,
    side: RuleSelectorSide,
    predicates: Vec<RuleSelectorPredicate>,
) {
    let mut definition = SelectorDefinition::new(id(raw)).with_rule_units(
        RuleUnitSelector::new(
            origin,
            side,
            RuleLifePredicate::Alive,
            RulePresencePredicate::Any,
            RuleSelectorReference::CurrentState,
            RuleSelectorOrdering::Formation,
            0,
            1,
            RuleEmptyPoolPolicy::NoOp,
            RuleSelectorChoice::First,
            None,
            false,
        )
        .unwrap()
        .with_predicates(predicates),
    );
    if raw == OWNER {
        definition = definition.with_unit_targets(
            UnitTargetSelector::new(TargetRelation::Allied, TargetPattern::Single).unwrap(),
        );
    }
    builder.add_selector(definition);
}
fn action(
    raw: u32,
    kind: AbilityKind,
    target: u32,
    operations: Vec<HitOperationDefinition>,
) -> AbilityDefinition {
    let mut action = AbilityActionDefinition::new(
        kind,
        1,
        TargetInvalidationPolicy::CancelRemainingForTarget,
        ActionResourcePolicy::new(0, 0, Energy::ZERO, Energy::ZERO),
    )
    .unwrap()
    .with_hits(vec![ActionHitDefinition::new(operations).with_profile(
        HitTargetGroup::Selected,
        Ratio::ONE,
        Ratio::ONE,
        HitCritPolicy::Never,
    )])
    .unwrap();
    action = if matches!(raw, LINKED_ATTACK | COUNTDOWN) {
        action.with_tags(&[AbilityTag::Attack, AbilityTag::Skill, AbilityTag::Assist])
    } else {
        action.with_tags(&[AbilityTag::Skill])
    };
    AbilityDefinition::new(id(raw), id(0x7dc2_0003), id(target), vec![]).with_action(action)
}
fn damage() -> Vec<HitOperationDefinition> {
    vec![HitOperationDefinition::Damage(
        OrdinaryDamageDefinition::new(
            Scalar::checked_from_integer(100).unwrap(),
            OrdinaryDamageMultipliers::new([Ratio::ONE; 9]).unwrap(),
        )
        .unwrap(),
    )]
}
fn queue(actor: u32) -> ProgramStep {
    ProgramStep::Operation(RuleOperationTemplate::QueueAction {
        actor_selector: id(actor),
        target_selector: id(TARGET),
        ability: id(LINKED_ATTACK),
        priority: ReactionPriority::new(0),
        forced_use: true,
        boundary: ReactionBoundary::AfterAction,
        owner: RuleActionOwner::Actor,
        payment: Some(RuleActionPaymentPolicy::Suppressed),
    })
}

pub(super) fn add_setup(
    builder: &mut CombatCatalogBuilder,
    catalog: &CombatCatalog,
    player: &ParticipantSpec,
    setup: Setup,
) {
    selector(
        builder,
        OWNER,
        RuleSelectorOrigin::Actor,
        RuleSelectorSide::Same,
        vec![],
    );
    let base = player.combatant();
    let mut bindings = Vec::new();
    let operation = match setup {
        Setup::Transform { form, countdown } => {
            let replacements = catalog
                .unit(form)
                .unwrap()
                .abilities()
                .iter()
                .copied()
                .filter(|ability| {
                    catalog
                        .ability(*ability)
                        .unwrap()
                        .action()
                        .is_some_and(|action| {
                            matches!(
                                action.kind(),
                                AbilityKind::Basic | AbilityKind::Skill | AbilityKind::Ultimate
                            )
                        })
                })
                .take(3)
                .collect::<Vec<_>>();
            assert_eq!(replacements.len(), 3);
            let program = id(0x7dd2_0001);
            builder.add_program(
                ProgramDefinition::new(program, vec![], vec![id(OWNER)], vec![], vec![])
                    .with_steps(
                        replacements
                            .iter()
                            .copied()
                            .zip([ATTACK, IDLE, END_TRANSFORM])
                            .map(|(old_ability, new)| {
                                ProgramStep::Operation(RuleOperationTemplate::ReplaceAbility {
                                    selector: id(OWNER),
                                    old_ability,
                                    new_ability: id(new),
                                })
                            })
                            .collect(),
                    ),
            );
            bindings.push(
                AbilityProgramBinding::new(1, AbilityProgramTiming::AfterHits, program).unwrap(),
            );
            let countdown = countdown.then(|| {
                let definition = CountdownDefinition::new(
                    id(COUNTDOWN),
                    ActionGauge::from_scaled(100_000_000).unwrap(),
                    Speed::from_scaled(500_000_000).unwrap(),
                    OwnerLinkPolicy::Persist,
                    OwnerLinkPolicy::Persist,
                    WaveLinkPolicy::Persist,
                );
                builder.add_countdown(CountdownCatalogDefinition::new(84, definition).unwrap());
                builder.add_ability(action(
                    COUNTDOWN,
                    AbilityKind::Countdown,
                    0x7dc1_0001,
                    damage(),
                ));
                definition
            });
            HitOperationDefinition::Transform(
                TransformationDefinition::new(
                    form,
                    replacements,
                    countdown,
                    TransformEndPolicy::End,
                    TransformEndPolicy::End,
                )
                .unwrap(),
            )
        }
        Setup::Linked { kind, formation } => {
            selector(
                builder,
                TARGET,
                RuleSelectorOrigin::Encounter,
                RuleSelectorSide::Opposing,
                vec![],
            );
            selector(
                builder,
                RECEIVER,
                RuleSelectorOrigin::Team,
                RuleSelectorSide::Same,
                vec![
                    RuleSelectorPredicate::FormationRange {
                        minimum: formation,
                        maximum: formation,
                    },
                    RuleSelectorPredicate::OwnedBy(id(OWNER)),
                ],
            );
            builder.add_ability(action(
                LINKED_ATTACK,
                AbilityKind::Skill,
                0x7dc1_0001,
                damage(),
            ));
            let program = id(0x7dd2_0002);
            builder.add_program(
                ProgramDefinition::new(
                    program,
                    vec![],
                    vec![id(OWNER), id(TARGET)],
                    vec![],
                    vec![],
                )
                .with_steps(vec![queue(OWNER)]),
            );
            builder.add_ability(
                action(
                    LINKED_ACTION,
                    if kind == LinkedEntityKind::Memosprite {
                        AbilityKind::Memosprite
                    } else {
                        AbilityKind::Summon
                    },
                    0x7dc1_0001,
                    vec![],
                )
                .with_programs(vec![
                    AbilityProgramBinding::new(1, AbilityProgramTiming::AfterHits, program)
                        .unwrap(),
                ]),
            );
            let spec = ResolvedCombatantSpec::new(
                base.form(),
                base.level(),
                Hp::new(10_000).unwrap(),
                Speed::from_scaled(500_000_000).unwrap(),
                ResolvedDefinitionBindings::new(
                    vec![id(LINKED_ATTACK), id(LINKED_ACTION)],
                    base.rule_bundles().to_vec(),
                    base.modifiers().to_vec(),
                )
                .unwrap(),
                CombatantSpecDigest::new([0xd3; 32]).unwrap(),
            )
            .unwrap()
            .with_sources(base.sources().to_vec())
            .unwrap()
            .with_modifier_bindings(base.modifier_bindings().to_vec())
            .unwrap();
            if kind == LinkedEntityKind::SharedActor {
                let program = id(0x7dd2_0003);
                builder.add_program(
                    ProgramDefinition::new(
                        program,
                        vec![],
                        vec![id(RECEIVER), id(TARGET)],
                        vec![],
                        vec![],
                    )
                    .with_steps(vec![queue(RECEIVER)]),
                );
                bindings.push(
                    AbilityProgramBinding::new(1, AbilityProgramTiming::AfterHits, program)
                        .unwrap(),
                );
            }
            HitOperationDefinition::SummonLinked(Box::new(
                LinkedUnitDefinition::new(
                    spec,
                    id(0x7dd4_0001),
                    FormationIndex::new(formation).unwrap(),
                    kind,
                    PresenceState::Present,
                    (kind != LinkedEntityKind::SharedActor).then(|| id(LINKED_ACTION)),
                    ActionGauge::from_scaled(100_000_000).unwrap(),
                    OwnerLinkPolicy::Persist,
                    OwnerLinkPolicy::Persist,
                    WaveLinkPolicy::Persist,
                )
                .unwrap(),
            ))
        }
    };
    builder.add_ability(
        action(SETUP, AbilityKind::Skill, OWNER, vec![operation]).with_programs(bindings),
    );
    builder.add_ability(action(
        END_TRANSFORM,
        AbilityKind::Skill,
        OWNER,
        vec![HitOperationDefinition::EndTransformation],
    ));
}
