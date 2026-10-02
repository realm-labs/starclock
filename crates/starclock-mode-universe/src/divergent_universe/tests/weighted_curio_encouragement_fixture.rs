//! Controlled attacks over the actual production Encouragement binding.
use crate::divergent_universe::DivergentUniverseAssembledBattle;
use starclock_combat::{
    ActionEventData, ActionGauge, AssemblyDigest, Battle, BattleEvent, BattleEventKind, BattleSeed,
    BattleSpec, CombatantSpecDigest, Command, ConcedePolicy, CountdownCatalogDefinition,
    CountdownDefinition, Energy, FormationIndex, Hp, LinkedEntityKind, LinkedUnitDefinition,
    OwnerLinkPolicy, ParticipantSource, ParticipantSpec, PresenceState, Ratio,
    ResolvedBuildBonuses, ResolvedCombatantSpec, ResolvedDefinitionBindings, Scalar, Speed,
    StatValue, TeamResourceSpec, TeamSide, UnitId, WaveLinkPolicy,
    catalog::{
        action::{
            AbilityActionDefinition, AbilityKind, AbilityProgramBinding, AbilityProgramTiming,
            AbilityTag, ActionHitDefinition, ActionResourcePolicy, HitCritPolicy,
            HitOperationDefinition, HitTargetGroup, OrdinaryDamageDefinition,
            OrdinaryDamageMultipliers, ReactionBoundary, TargetInvalidationPolicy, TargetPattern,
            TargetRelation, UnitTargetSelector, elation::ElationDamageDefinition,
        },
        builder::CombatCatalogBuilder,
        definition::{
            AbilityDefinition, EncounterDefinition, EnemyDefinition, ProgramDefinition,
            SelectorDefinition, UnitDefinition,
        },
        selector::{
            RuleEmptyPoolPolicy, RuleLifePredicate, RulePresencePredicate, RuleSelectorChoice,
            RuleSelectorOrdering, RuleSelectorOrigin, RuleSelectorPredicate, RuleSelectorReference,
            RuleSelectorSide, RuleUnitSelector,
        },
    },
    formula::model::{CombatElement, DamageClass, ResistanceInput},
    rule::model::{
        ProgramStep, ReactionPriority, RuleActionOwner, RuleActionPaymentPolicy,
        RuleEffectChancePolicy, RuleOperationTemplate, RuleValue, ValueExpr,
    },
};

pub(super) const EFFECT: u32 = 0x7ee5_0001;
pub(super) const BASIC: u32 = 0x7df0_0001;
pub(super) const FOLLOW_UP: u32 = 0x7df0_0002;
pub(super) const IDLE: u32 = 0x7df0_0003;
pub(super) const KILL_ENEMY: u32 = 0x7df0_0004;
pub(super) const SPAWN: u32 = 0x7df0_0005;
pub(super) const LINKED_ATTACK: u32 = 0x7df0_0006;
pub(super) const COUNTDOWN: u32 = 0x7df0_0007;
pub(super) const REFRESH: u32 = 0x7df0_0008;
const QUEUE_FOLLOW_UP: u32 = 0x7df0_0009;

pub(super) fn id<I: TryFrom<u32>>(raw: u32) -> I
where
    I::Error: core::fmt::Debug,
{
    I::try_from(raw).unwrap()
}
#[derive(Clone, Copy)]
pub(super) struct Probe {
    pub(super) crit: HitCritPolicy,
    pub(super) rate: i64,
    pub(super) all: bool,
    pub(super) hits: u16,
    pub(super) linked: Option<LinkedEntityKind>,
    pub(super) unitless: bool,
    pub(super) seed: u8,
}
impl Default for Probe {
    fn default() -> Self {
        Self {
            crit: HitCritPolicy::Shared,
            rate: 1_000_000,
            all: false,
            hits: 1,
            linked: None,
            unitless: false,
            seed: 0xea,
        }
    }
}
fn ordinary(class: DamageClass) -> HitOperationDefinition {
    HitOperationDefinition::Damage(
        OrdinaryDamageDefinition::new(
            Scalar::from_scaled(100_000_000),
            OrdinaryDamageMultipliers::new([Ratio::ONE; 9]).unwrap(),
        )
        .unwrap()
        .with_class(class),
    )
}
fn attacks() -> Vec<HitOperationDefinition> {
    vec![
        ordinary(DamageClass::Direct),
        HitOperationDefinition::ElationDamage(
            ElationDamageDefinition::new(
                Scalar::from_scaled(100_000_000),
                Ratio::ONE,
                Ratio::ONE,
                Ratio::ZERO,
                ResistanceInput {
                    target_resistance: Ratio::ZERO,
                    penetration: Ratio::ZERO,
                    minimum: Ratio::from_scaled(-1_000_000),
                    maximum: Ratio::from_scaled(900_000),
                },
                Ratio::ONE,
                CombatElement::Fire,
            )
            .unwrap(),
        ),
        ordinary(DamageClass::Additional),
    ]
}
fn owner_selector() -> RuleUnitSelector {
    RuleUnitSelector::new(
        RuleSelectorOrigin::Actor,
        RuleSelectorSide::Same,
        RuleLifePredicate::Alive,
        RulePresencePredicate::Any,
        RuleSelectorReference::CurrentState,
        RuleSelectorOrdering::StableId,
        0,
        1,
        RuleEmptyPoolPolicy::NoOp,
        RuleSelectorChoice::First,
        None,
        false,
    )
    .unwrap()
}

pub(super) fn scenario(source: &DivergentUniverseAssembledBattle, probe: Probe) -> Battle {
    let mut builder = CombatCatalogBuilder::from_catalog(source.combat_catalog(), [0xea; 32]);
    let owner = id(0x7df1_0101);
    builder.add_selector(SelectorDefinition::new(owner).with_rule_units(owner_selector()));
    let linked_target = id(0x7df1_0102);
    builder.add_selector(
        SelectorDefinition::new(linked_target).with_rule_units(
            RuleUnitSelector::new(
                RuleSelectorOrigin::Team,
                RuleSelectorSide::Opposing,
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
            .unwrap(),
        ),
    );
    let original = source
        .battle_spec()
        .participants()
        .iter()
        .find(|p| p.side() == TeamSide::Player && p.formation().get() == 0)
        .unwrap()
        .combatant();
    builder.add_unit(UnitDefinition::new(
        id(0x7df7_0001),
        vec![id(LINKED_ATTACK)],
        vec![],
    ));
    let linked = probe.linked.map(|kind| {
        let form = id(0x7df7_0001);
        // Deliberately inherit all mode bundles, sources and modifiers. The
        // linked attack also applies the actual Curio effect to itself.
        let spec = ResolvedCombatantSpec::new(
            form,
            original.level(),
            Hp::new(10_000).unwrap(),
            Speed::from_scaled(500_000_000).unwrap(),
            ResolvedDefinitionBindings::new(
                vec![id(LINKED_ATTACK)],
                original.rule_bundles().to_vec(),
                original.modifiers().to_vec(),
            )
            .unwrap(),
            CombatantSpecDigest::new([0xeb; 32]).unwrap(),
        )
        .unwrap()
        .with_sources(original.sources().to_vec())
        .unwrap()
        .with_modifier_bindings(original.modifier_bindings().to_vec())
        .unwrap()
        .with_build_bonuses(ResolvedBuildBonuses::new(
            Scalar::from_scaled(probe.rate),
            Scalar::ZERO,
            Scalar::ZERO,
            Scalar::ZERO,
            Scalar::ZERO,
            [Scalar::ZERO; 7],
        ));
        LinkedUnitDefinition::new(
            spec,
            id(0x7df8_0001),
            FormationIndex::new(4).unwrap(),
            kind,
            PresenceState::Present,
            if kind == LinkedEntityKind::SharedActor {
                None
            } else {
                Some(id(LINKED_ATTACK))
            },
            ActionGauge::from_scaled(100_000_000).unwrap(),
            OwnerLinkPolicy::Persist,
            OwnerLinkPolicy::Persist,
            WaveLinkPolicy::Persist,
        )
        .unwrap()
    });
    builder.add_countdown(
        CountdownCatalogDefinition::new(
            17,
            CountdownDefinition::new(
                id(COUNTDOWN),
                ActionGauge::from_scaled(100_000_000).unwrap(),
                Speed::from_scaled(500_000_000).unwrap(),
                OwnerLinkPolicy::Persist,
                OwnerLinkPolicy::Persist,
                WaveLinkPolicy::Persist,
            ),
        )
        .unwrap(),
    );
    let shared_selector = id(0x7df1_0103);
    builder.add_selector(
        SelectorDefinition::new(shared_selector).with_rule_units(
            RuleUnitSelector::new(
                RuleSelectorOrigin::Team,
                RuleSelectorSide::Same,
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
            .with_predicates(vec![RuleSelectorPredicate::UnitForm(id(0x7df7_0001))]),
        ),
    );
    // A shared actor has no automatic attack; drive it through the same queue.
    let shared_program = id(0x7df3_0010);
    builder.add_program(
        ProgramDefinition::new(
            shared_program,
            vec![],
            vec![linked_target, shared_selector],
            vec![],
            vec![],
        )
        .with_steps(if probe.linked == Some(LinkedEntityKind::SharedActor) {
            vec![ProgramStep::Operation(RuleOperationTemplate::QueueAction {
                actor_selector: shared_selector,
                target_selector: linked_target,
                ability: id(LINKED_ATTACK),
                priority: ReactionPriority::new(0),
                forced_use: true,
                boundary: ReactionBoundary::AfterAction,
                owner: RuleActionOwner::Actor,
                payment: Some(RuleActionPaymentPolicy::Suppressed),
            })]
        } else {
            vec![]
        }),
    );
    let mut abilities = Vec::new();
    for raw in BASIC..=QUEUE_FOLLOW_UP {
        let selector = id(raw + 0x10000);
        let program = id(raw + 0x20000);
        let self_target = matches!(raw, SPAWN | REFRESH);
        builder.add_selector(
            SelectorDefinition::new(selector).with_unit_targets(
                UnitTargetSelector::new(
                    if self_target {
                        TargetRelation::SelfUnit
                    } else {
                        TargetRelation::Opposing
                    },
                    if probe.all && matches!(raw, BASIC | FOLLOW_UP) {
                        TargetPattern::All
                    } else {
                        TargetPattern::Single
                    },
                )
                .unwrap(),
            ),
        );
        let entry = if raw == QUEUE_FOLLOW_UP {
            vec![ProgramStep::Operation(RuleOperationTemplate::QueueAction {
                actor_selector: owner,
                target_selector: linked_target,
                ability: id(FOLLOW_UP),
                priority: ReactionPriority::new(0),
                forced_use: false,
                boundary: ReactionBoundary::AfterAction,
                owner: RuleActionOwner::Actor,
                payment: None,
            })]
        } else if matches!(raw, REFRESH | LINKED_ATTACK)
            && source.combat_catalog().effect(id(EFFECT)).is_some()
        {
            vec![ProgramStep::Operation(RuleOperationTemplate::ApplyEffect {
                selector: owner,
                effect: id(EFFECT),
                stacks: ValueExpr::Literal(RuleValue::Integer(1)),
                chance: RuleEffectChancePolicy::Guaranteed,
                base_chance: None,
                rng_purpose: None,
            })]
        } else if raw == SPAWN && probe.unitless {
            vec![ProgramStep::Operation(
                RuleOperationTemplate::CreateCountdown { code: 17 },
            )]
        } else {
            vec![]
        };
        builder.add_program(
            ProgramDefinition::new(
                program,
                vec![],
                vec![owner, linked_target],
                if entry.iter().any(|step| {
                    matches!(
                        step,
                        ProgramStep::Operation(RuleOperationTemplate::ApplyEffect { .. })
                    )
                }) {
                    vec![id(EFFECT)]
                } else {
                    vec![]
                },
                vec![],
            )
            .with_steps(entry),
        );
        let operations = match raw {
            BASIC | FOLLOW_UP | LINKED_ATTACK | COUNTDOWN => attacks(),
            KILL_ENEMY => vec![HitOperationDefinition::Damage(
                OrdinaryDamageDefinition::new(
                    Scalar::checked_from_integer(2_000_000).unwrap(),
                    OrdinaryDamageMultipliers::new([Ratio::ONE; 9]).unwrap(),
                )
                .unwrap(),
            )],
            SPAWN => linked
                .clone()
                .map(|definition| vec![HitOperationDefinition::SummonLinked(Box::new(definition))])
                .unwrap_or_default(),
            _ => vec![],
        };
        let kind = match raw {
            FOLLOW_UP => AbilityKind::FollowUp,
            COUNTDOWN => AbilityKind::Countdown,
            LINKED_ATTACK => match probe.linked {
                Some(LinkedEntityKind::Summon) => AbilityKind::Summon,
                Some(LinkedEntityKind::Memosprite) => AbilityKind::Memosprite,
                _ => AbilityKind::ExtraAction,
            },
            SPAWN | REFRESH | QUEUE_FOLLOW_UP => AbilityKind::Skill,
            _ => AbilityKind::Basic,
        };
        let hits = if matches!(raw, BASIC | FOLLOW_UP) {
            probe.hits
        } else {
            1
        };
        let mut action = AbilityActionDefinition::new(
            kind,
            hits,
            TargetInvalidationPolicy::CancelRemainingForTarget,
            ActionResourcePolicy::new(0, 0, Energy::ZERO, Energy::ZERO),
        )
        .unwrap()
        .with_hits(
            (0..hits)
                .map(|_| {
                    ActionHitDefinition::new(operations.clone()).with_profile(
                        HitTargetGroup::Selected,
                        Ratio::ONE,
                        Ratio::ONE,
                        if raw == KILL_ENEMY {
                            HitCritPolicy::Never
                        } else {
                            probe.crit
                        },
                    )
                })
                .collect(),
        )
        .unwrap();
        if matches!(raw, LINKED_ATTACK | COUNTDOWN) {
            action = action.with_tags(&[AbilityTag::Attack, AbilityTag::FollowUp]);
        }
        let mut programs =
            vec![AbilityProgramBinding::new(1, AbilityProgramTiming::Entry, program).unwrap()];
        if raw == SPAWN {
            programs.push(
                AbilityProgramBinding::new(2, AbilityProgramTiming::AfterHits, shared_program)
                    .unwrap(),
            );
        }
        builder.add_ability(
            AbilityDefinition::new(id(raw), program, selector, vec![])
                .with_action(action)
                .with_programs(programs),
        );
        abilities.push(id(raw));
    }
    let mut participants = Vec::new();
    for original in source
        .battle_spec()
        .participants()
        .iter()
        .filter(|p| p.side() == TeamSide::Player)
    {
        let base = original.combatant();
        let spec = ResolvedCombatantSpec::new(
            base.form(),
            base.level(),
            Hp::new(10_000).unwrap(),
            Speed::from_scaled(100_000_000).unwrap(),
            ResolvedDefinitionBindings::new(
                abilities.clone(),
                base.rule_bundles().to_vec(),
                base.modifiers().to_vec(),
            )
            .unwrap(),
            CombatantSpecDigest::new([original.formation().get() + 1; 32]).unwrap(),
        )
        .unwrap()
        .with_sources(base.sources().to_vec())
        .unwrap()
        .with_modifier_bindings(base.modifier_bindings().to_vec())
        .unwrap()
        .with_base_attack_defense(
            StatValue::from_scaled(200_000_000).unwrap(),
            StatValue::from_scaled(0).unwrap(),
        )
        .with_build_bonuses(ResolvedBuildBonuses::new(
            Scalar::from_scaled(probe.rate),
            Scalar::ZERO,
            Scalar::ZERO,
            Scalar::ZERO,
            Scalar::ZERO,
            [Scalar::ZERO; 7],
        ));
        participants.push(ParticipantSpec::new(
            original.side(),
            original.formation(),
            original.source(),
            spec,
        ));
    }
    let form = id(0x7df4_0001);
    builder.add_unit(UnitDefinition::new(form, vec![id(IDLE)], vec![]));
    let mut waves = Vec::new();
    for wave in 1_u16..=2 {
        let mut enemies = Vec::new();
        for formation in 0..2_u8 {
            let enemy = id(0x7df5_0000 + u32::from(wave) * 4 + u32::from(formation));
            builder.add_enemy(EnemyDefinition::new(enemy, form, vec![id(IDLE)]));
            let spec = ResolvedCombatantSpec::new(
                form,
                original.level(),
                Hp::new(1_000_000).unwrap(),
                Speed::from_scaled(100_000_000).unwrap(),
                ResolvedDefinitionBindings::new(vec![id(IDLE)], vec![], vec![]).unwrap(),
                CombatantSpecDigest::new([0xee; 32]).unwrap(),
            )
            .unwrap()
            .with_base_attack_defense(
                StatValue::from_scaled(0).unwrap(),
                StatValue::from_scaled(0).unwrap(),
            );
            participants.push(
                ParticipantSpec::new(
                    TeamSide::Enemy,
                    FormationIndex::new(formation).unwrap(),
                    ParticipantSource::EncounterEnemy(enemy),
                    spec,
                )
                .with_wave(wave)
                .unwrap(),
            );
            enemies.push(enemy);
        }
        waves.push(enemies);
    }
    let encounter = id(0x7df6_0001);
    builder.add_encounter(
        EncounterDefinition::new(encounter, vec![], vec![])
            .with_waves(waves)
            .unwrap(),
    );
    let spec = BattleSpec::new(
        AssemblyDigest::new([0xea; 32]).unwrap(),
        encounter,
        participants,
        TeamResourceSpec::new(3, 5).unwrap(),
        TeamResourceSpec::new(0, 0).unwrap(),
        ConcedePolicy::Allowed,
    )
    .unwrap();
    let mut battle = Battle::create(
        builder.build().unwrap(),
        spec,
        BattleSeed::new([probe.seed; 32]),
    )
    .unwrap();
    let command = Command::StartBattle {
        decision: battle.decision().unwrap().id(),
    };
    accept(&mut battle, command);
    battle
}
pub(super) fn accept(battle: &mut Battle, command: Command) -> Vec<BattleEvent> {
    let result = battle.apply(command).unwrap();
    assert!(result.fault().is_none(), "{:?}", result.fault());
    result.events().to_vec()
}
pub(super) fn idle(battle: &mut Battle) -> Vec<BattleEvent> {
    let command = if let Some(decision) = battle.decision() {
        decision.legal_commands().iter().find(|command|
            matches!(command, Command::UseAbility { ability, .. } if ability.get() == IDLE)).unwrap().clone()
    } else {
        battle.advance_command().unwrap()
    };
    accept(battle, command)
}
pub(super) fn cast(
    battle: &mut Battle,
    formation: u8,
    raw: u32,
    target: Option<UnitId>,
) -> Vec<BattleEvent> {
    if raw == FOLLOW_UP {
        let mut events = cast(battle, formation, QUEUE_FOLLOW_UP, target);
        for _ in 0..256 {
            if events.iter().any(|event| {
                matches!(event.kind(),
                BattleEventKind::Action(ActionEventData::Resolved { ability, .. })
                    if ability.get() == FOLLOW_UP)
            }) {
                return events;
            }
            events.extend(idle(battle));
        }
        panic!("queued follow-up did not finish");
    }
    let actor = battle
        .view()
        .units_by_id()
        .find(|unit| unit.side() == TeamSide::Player && unit.formation().get() == formation)
        .unwrap()
        .id();
    for _ in 0..256 {
        if let Some(command) = battle.decision().and_then(|decision| decision.legal_commands().iter()
            .find(|command| matches!(command, Command::UseAbility { actor: offered, ability, primary_target, .. }
                if *offered == actor && ability.get() == raw && *primary_target == target)).cloned()) {
            return accept(battle, command);
        }
        idle(battle);
    }
    panic!("controlled command not offered: {formation}/{raw}");
}
