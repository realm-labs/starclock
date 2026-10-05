//! Generic lifecycle operations over normal equipment bindings, never rebinding.
use crate::divergent_universe::DivergentUniverseAssembledBattle;
use crate::divergent_universe::tests::{
    weighted_curio_footstep_equipment_fixture::{DAMAGE, IDLE, LOSS, scalar},
    weighted_curio_overflow_fixture::id,
};
use starclock_combat::{
    AbilityId, ActionGauge, CombatantSpecDigest, CountdownCatalogDefinition, CountdownDefinition,
    EffectDefinitionId, Energy, FormationIndex, Hp, LinkedEntityKind, LinkedUnitDefinition,
    OwnerLinkPolicy, ParticipantSpec, PresenceState, Ratio, ResolvedCombatantSpec,
    ResolvedDefinitionBindings, Scalar, Speed, TransformEndPolicy, TransformationDefinition,
    UnitDefinitionId, WaveLinkPolicy,
    catalog::{
        action::{
            AbilityActionDefinition, AbilityKind, AbilityProgramBinding, AbilityProgramTiming,
            AbilityTag, ActionHitDefinition, ActionResourcePolicy, HitCritPolicy,
            HitOperationDefinition, HitTargetGroup, OrdinaryDamageDefinition,
            OrdinaryDamageMultipliers, ReactionBoundary, TargetInvalidationPolicy,
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
        RuleEffectChancePolicy, RuleOperationTemplate, RuleValue, ValueExpr,
    },
};

pub(super) const SETUP: u32 = 0x7f60_0005;
pub(super) const RESTORE: u32 = 0x7f60_0006;
pub(super) const LINKED_SKILL: u32 = 0x7f60_0007;
pub(super) const COUNTDOWN: u32 = 0x7f60_0008;
const LINKED_ACTION: u32 = 0x7f60_0009;

#[derive(Clone, Copy)]
pub(super) enum Setup {
    Transform {
        form: UnitDefinitionId,
        countdown: bool,
    },
    Linked {
        kind: LinkedEntityKind,
        formation: u8,
        force_effect: bool,
    },
}

fn action(
    raw: u32,
    kind: AbilityKind,
    selector: u32,
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
    if raw == LINKED_SKILL {
        action = action.with_tags(&[AbilityTag::Attack, AbilityTag::Skill, AbilityTag::Assist]);
    }
    AbilityDefinition::new(id(raw), id(IDLE + 0x20000), id(selector), vec![]).with_action(action)
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
fn selector(
    builder: &mut CombatCatalogBuilder,
    raw: u32,
    origin: RuleSelectorOrigin,
    side: RuleSelectorSide,
    predicates: Vec<RuleSelectorPredicate>,
) {
    builder.add_selector(
        SelectorDefinition::new(id(raw)).with_rule_units(
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
        ),
    );
}

pub(super) fn add_setup(
    builder: &mut CombatCatalogBuilder,
    player: &ParticipantSpec,
    setup: Setup,
    authored_effect: Option<EffectDefinitionId>,
    source: &DivergentUniverseAssembledBattle,
) {
    let base = player.combatant();
    let self_target = IDLE + 0x10000;
    let countdown = CountdownDefinition::new(
        id(COUNTDOWN),
        ActionGauge::from_scaled(100_000_000).unwrap(),
        Speed::from_scaled(500_000_000).unwrap(),
        OwnerLinkPolicy::Persist,
        OwnerLinkPolicy::Persist,
        WaveLinkPolicy::Persist,
    );
    let operation = match setup {
        Setup::Transform {
            form,
            countdown: has_countdown,
        } => {
            if has_countdown {
                builder.add_countdown(CountdownCatalogDefinition::new(83, countdown).unwrap());
                builder.add_ability(action(
                    COUNTDOWN,
                    AbilityKind::Countdown,
                    DAMAGE + 0x10000,
                    damage(),
                ));
            }
            let replacements: Vec<AbilityId> = source
                .battle_spec()
                .participants()
                .iter()
                .find(|participant| participant.combatant().form() == form)
                .unwrap()
                .combatant()
                .abilities()
                .iter()
                .copied()
                .filter(|ability| {
                    source
                        .combat_catalog()
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
                .collect();
            assert_eq!(replacements.len(), 3);
            // Transform into a real registered core form, then use declared
            // ReplaceAbility operations for controlled observations. Never
            // overwrite its production UnitDefinition or rebind a Curio policy.
            let owner = 0x7f61_0100;
            selector(
                builder,
                owner,
                RuleSelectorOrigin::Actor,
                RuleSelectorSide::Same,
                vec![],
            );
            let program = id(SETUP + 0x20000);
            builder.add_program(
                ProgramDefinition::new(program, vec![], vec![id(owner)], vec![], vec![])
                    .with_steps(
                        replacements
                            .iter()
                            .copied()
                            .zip([LOSS, DAMAGE, RESTORE])
                            .map(|(old_ability, new)| {
                                ProgramStep::Operation(RuleOperationTemplate::ReplaceAbility {
                                    selector: id(owner),
                                    old_ability,
                                    new_ability: id(new),
                                })
                            })
                            .collect(),
                    ),
            );
            HitOperationDefinition::Transform(
                TransformationDefinition::new(
                    form,
                    replacements,
                    has_countdown.then_some(countdown),
                    TransformEndPolicy::End,
                    TransformEndPolicy::End,
                )
                .unwrap(),
            )
        }
        Setup::Linked {
            kind,
            formation,
            force_effect,
        } => {
            let owner = 0x7f61_0100;
            let receiving = 0x7f61_0101;
            selector(
                builder,
                owner,
                RuleSelectorOrigin::Actor,
                RuleSelectorSide::Same,
                vec![],
            );
            selector(
                builder,
                receiving,
                RuleSelectorOrigin::Team,
                RuleSelectorSide::Same,
                vec![
                    RuleSelectorPredicate::FormationRange {
                        minimum: formation,
                        maximum: formation,
                    },
                    RuleSelectorPredicate::OwnedBy(id(owner)),
                ],
            );
            let effect = force_effect.then_some(authored_effect).flatten();
            let program = id(LINKED_SKILL + 0x20000);
            let mut steps = vec![ProgramStep::Operation(RuleOperationTemplate::ConsumeHp {
                selector: id(owner),
                amount: scalar(30),
                floor: scalar(1),
            })];
            // Force the genuine authored effect onto this inherited linked unit.
            // A producer filter must reject the bonus, not only acquisition.
            if let Some(effect) = effect {
                steps.push(ProgramStep::Operation(RuleOperationTemplate::ApplyEffect {
                    selector: id(owner),
                    effect,
                    stacks: ValueExpr::Literal(RuleValue::Integer(10)),
                    chance: RuleEffectChancePolicy::Guaranteed,
                    base_chance: None,
                    rng_purpose: None,
                }));
            }
            builder.add_program(
                ProgramDefinition::new(
                    program,
                    vec![],
                    vec![id(owner)],
                    effect.into_iter().collect(),
                    vec![],
                )
                .with_steps(steps),
            );
            // A genuinely forced Assist Skill retains Skill kind: exclusion
            // cannot rely only on Summon/Memosprite kinds or queue admission.
            builder.add_ability(
                action(LINKED_SKILL, AbilityKind::Skill, DAMAGE + 0x10000, damage()).with_programs(
                    vec![
                        AbilityProgramBinding::new(1, AbilityProgramTiming::Entry, program)
                            .unwrap(),
                    ],
                ),
            );
            let natural_program = id(LINKED_ACTION + 0x20000);
            let target = 0x7f61_0102;
            selector(
                builder,
                target,
                RuleSelectorOrigin::Encounter,
                RuleSelectorSide::Opposing,
                vec![],
            );
            builder.add_program(
                ProgramDefinition::new(
                    natural_program,
                    vec![],
                    vec![id(owner), id(target)],
                    vec![],
                    vec![],
                )
                .with_steps(vec![ProgramStep::Operation(
                    RuleOperationTemplate::QueueAction {
                        actor_selector: id(owner),
                        target_selector: id(target),
                        ability: id(LINKED_SKILL),
                        priority: ReactionPriority::new(0),
                        forced_use: true,
                        boundary: ReactionBoundary::AfterAction,
                        owner: RuleActionOwner::Actor,
                        payment: Some(RuleActionPaymentPolicy::Suppressed),
                    },
                )]),
            );
            builder.add_ability(
                action(
                    LINKED_ACTION,
                    if kind == LinkedEntityKind::Memosprite {
                        AbilityKind::Memosprite
                    } else {
                        AbilityKind::Summon
                    },
                    DAMAGE + 0x10000,
                    vec![],
                )
                .with_programs(vec![
                    AbilityProgramBinding::new(1, AbilityProgramTiming::AfterHits, natural_program)
                        .unwrap(),
                ]),
            );
            let spec = ResolvedCombatantSpec::new(
                base.form(),
                base.level(),
                Hp::new(100).unwrap(),
                Speed::from_scaled(500_000_000).unwrap(),
                ResolvedDefinitionBindings::new(
                    vec![id(LINKED_SKILL), id(LINKED_ACTION)],
                    base.rule_bundles().to_vec(),
                    base.modifiers().to_vec(),
                )
                .unwrap(),
                CombatantSpecDigest::new([0xb6; 32]).unwrap(),
            )
            .unwrap()
            .with_sources(base.sources().to_vec())
            .unwrap()
            .with_modifier_bindings(base.modifier_bindings().to_vec())
            .unwrap();
            HitOperationDefinition::SummonLinked(Box::new(
                LinkedUnitDefinition::new(
                    spec,
                    id(0x7f6e_0001),
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
    let mut ability = action(SETUP, AbilityKind::Skill, self_target, vec![operation]);
    if matches!(setup, Setup::Transform { .. }) {
        ability = ability.with_programs(vec![
            AbilityProgramBinding::new(1, AbilityProgramTiming::AfterHits, id(SETUP + 0x20000))
                .unwrap(),
        ]);
    }
    if let Setup::Linked {
        kind: LinkedEntityKind::SharedActor,
        ..
    } = setup
    {
        let target = 0x7f61_0102;
        let program = id(SETUP + 0x20000);
        builder.add_program(
            ProgramDefinition::new(
                program,
                vec![],
                vec![id(0x7f61_0101), id(target)],
                vec![],
                vec![],
            )
            .with_steps(vec![ProgramStep::Operation(
                RuleOperationTemplate::QueueAction {
                    actor_selector: id(0x7f61_0101),
                    target_selector: id(target),
                    ability: id(LINKED_SKILL),
                    priority: ReactionPriority::new(0),
                    forced_use: true,
                    boundary: ReactionBoundary::AfterAction,
                    owner: RuleActionOwner::Actor,
                    payment: Some(RuleActionPaymentPolicy::Suppressed),
                },
            )]),
        );
        ability = ability.with_programs(vec![
            AbilityProgramBinding::new(1, AbilityProgramTiming::AfterHits, program).unwrap(),
        ]);
    }
    builder.add_ability(ability);
    builder.add_ability(action(
        RESTORE,
        AbilityKind::Skill,
        self_target,
        vec![HitOperationDefinition::EndTransformation],
    ));
}
