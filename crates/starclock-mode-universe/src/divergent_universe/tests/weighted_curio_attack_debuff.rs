//! Sora-backed attack effects, driven by accepted shared battle commands.
use crate::divergent_universe::{
    DivergentUniverseAssembledBattle, DivergentUniverseBaselineFixture,
    tests::{
        curio_battle_grants::{ready, result},
        curio_battle_stats::assemble,
    },
    weighted_curio::WeightedCurioSlotLimit,
};
use starclock_build::light_cone::CombatPath;
use starclock_combat::{
    AbilityId, ActionEventData, ActionGaugeChangeKind, AssemblyDigest, Battle, BattleEvent,
    BattleEventKind, BattleSeed, BattleSpec, CauseActor, CombatantSpecDigest, Command,
    ConcedePolicy, DurationClock, Energy, FormationIndex, Hp, LifeState, ParticipantInitialState,
    ParticipantSource, ParticipantSpec, PresenceState, Ratio, ResolvedBuildBonuses,
    ResolvedCombatantSpec, ResolvedDefinitionBindings, Scalar, Speed, StatValue, TeamResourceSpec,
    TeamSide, TurnEventData, UnitId, UnitLevel,
    catalog::{
        action::{
            AbilityActionDefinition, AbilityKind, AbilityTag, ActionHitDefinition,
            ActionResourcePolicy, HitCritPolicy, HitOperationDefinition, HitTargetGroup,
            OrdinaryDamageDefinition, OrdinaryDamageMultipliers, ScalingDamageDefinition,
            TargetInvalidationPolicy, TargetPattern, TargetRelation, UnitTargetSelector,
        },
        builder::CombatCatalogBuilder,
        definition::{
            AbilityDefinition, EncounterDefinition, EnemyDefinition, ProgramDefinition,
            SelectorDefinition, UnitDefinition,
        },
    },
    formula::model::{CombatElement, DamageClass},
    modifier::model::StatKind,
};
use starclock_data::divergent_universe_catalog::DivergentUniverseRunFamily;

const EFFECT: u32 = 0x7e66_0001;
const FAMILIES: [DivergentUniverseRunFamily; 2] = [
    DivergentUniverseRunFamily::Ordinary,
    DivergentUniverseRunFamily::Cyclical,
];
fn id<I: TryFrom<u32>>(raw: u32) -> I
where
    I::Error: core::fmt::Debug,
{
    I::try_from(raw).unwrap()
}

#[derive(Clone, Copy)]
pub(super) struct Probe {
    pub(super) formation: u8,
    pub(super) all: bool,
    pub(super) allied: bool,
    pub(super) attack: bool,
    pub(super) hits: u16,
    pub(super) enemy_hp: i64,
    pub(super) outgoing: DamageClass,
    pub(super) scaling: Option<StatKind>,
    pub(super) coefficient: i64,
    pub(super) base_stats: Option<(i64, i64)>,
    pub(super) crit: HitCritPolicy,
    pub(super) bonuses: ResolvedBuildBonuses,
    pub(super) selected_kind: AbilityKind,
    pub(super) initial_hp: Option<i64>,
}
impl Default for Probe {
    fn default() -> Self {
        Self {
            formation: 0,
            all: false,
            allied: false,
            attack: true,
            hits: 3,
            enemy_hp: 10_000,
            outgoing: DamageClass::Direct,
            scaling: None,
            coefficient: 1_000_000,
            base_stats: None,
            crit: HitCritPolicy::Never,
            bonuses: ResolvedBuildBonuses::default(),
            selected_kind: AbilityKind::Basic,
            initial_hp: None,
        }
    }
}
pub(super) struct Scenario {
    pub(super) battle: Battle,
    pub(super) actor: UnitId,
    pub(super) target: UnitId,
    pub(super) selected: AbilityId,
    pub(super) ultimate: AbilityId,
    fallback: AbilityId,
    implicit: bool,
}

pub(super) fn scenario(assembled: &DivergentUniverseAssembledBattle, input: Probe) -> Scenario {
    let mut builder = CombatCatalogBuilder::from_catalog(assembled.combat_catalog(), [81; 32]);
    let selected = id(0x7d43_0001);
    let ultimate = id(0x7d43_0002);
    let fallback = id(0x7d43_0003);
    let form = id(0x7d44_0001);
    let encounter = id(0x7d46_0001);
    for (offset, ability, kind, damage) in [
        (1, selected, input.selected_kind, 1),
        (2, ultimate, AbilityKind::Ultimate, 1),
        (3, fallback, AbilityKind::Basic, 100),
    ] {
        let selector = id(0x7d41_0000 + offset);
        let program = id(0x7d42_0000 + offset);
        builder.add_selector(
            SelectorDefinition::new(selector).with_unit_targets(
                UnitTargetSelector::new(
                    if input.allied && ability != fallback {
                        TargetRelation::Allied
                    } else {
                        TargetRelation::Opposing
                    },
                    if input.all && ability != fallback {
                        TargetPattern::All
                    } else {
                        TargetPattern::Single
                    },
                )
                .unwrap(),
            ),
        );
        builder.add_program(ProgramDefinition::new(
            program,
            vec![],
            vec![selector],
            vec![],
            vec![],
        ));
        let hits = if ability == fallback { 1 } else { input.hits };
        let action = AbilityActionDefinition::new(
            kind,
            hits,
            TargetInvalidationPolicy::CancelRemainingForTarget,
            ActionResourcePolicy::new(
                0,
                0,
                if kind == AbilityKind::Ultimate {
                    Energy::from_scaled(1_000_000).unwrap()
                } else {
                    Energy::ZERO
                },
                Energy::ZERO,
            ),
        )
        .unwrap()
        .with_hits(
            (0..hits)
                .map(|_| {
                    let operation = if let Some(stat) = input.scaling {
                        HitOperationDefinition::ScalingDamage(
                            ScalingDamageDefinition::new(
                                stat,
                                Ratio::from_scaled(input.coefficient),
                                DamageClass::Direct,
                                CombatElement::Wind,
                            )
                            .unwrap(),
                        )
                    } else {
                        HitOperationDefinition::Damage(
                            OrdinaryDamageDefinition::new(
                                Scalar::checked_from_integer(damage).unwrap(),
                                OrdinaryDamageMultipliers::new([Ratio::ONE; 9]).unwrap(),
                            )
                            .unwrap()
                            .with_class(if ability == fallback {
                                input.outgoing
                            } else {
                                DamageClass::Direct
                            }),
                        )
                    };
                    ActionHitDefinition::new(vec![operation]).with_profile(
                        HitTargetGroup::Selected,
                        Ratio::ONE,
                        Ratio::ONE,
                        input.crit,
                    )
                })
                .collect(),
        )
        .unwrap();
        let action = if input.attack && ability != fallback {
            action
        } else {
            action.with_tags(&[AbilityTag::Basic])
        };
        builder.add_ability(
            AbilityDefinition::new(ability, program, selector, vec![]).with_action(action),
        );
    }
    builder.add_unit(UnitDefinition::new(
        form,
        vec![selected, ultimate, fallback],
        vec![],
    ));
    let original = assembled
        .battle_spec()
        .participants()
        .iter()
        .find(|p| p.side() == TeamSide::Player && p.formation().get() == input.formation)
        .unwrap()
        .combatant();
    let combatant = ResolvedCombatantSpec::new(
        form,
        UnitLevel::new(80).unwrap(),
        Hp::new(10_000).unwrap(),
        Speed::from_scaled(200_000_000).unwrap(),
        ResolvedDefinitionBindings::new(
            vec![selected, ultimate, fallback],
            original.rule_bundles().to_vec(),
            original.modifiers().to_vec(),
        )
        .unwrap(),
        CombatantSpecDigest::new([82; 32]).unwrap(),
    )
    .unwrap()
    .with_sources(original.sources().to_vec())
    .unwrap()
    .with_modifier_bindings(original.modifier_bindings().to_vec())
    .unwrap()
    .with_energy(
        Energy::from_scaled(2_000_000).unwrap(),
        Energy::from_scaled(2_000_000).unwrap(),
    )
    .unwrap();
    let combatant = if let Some((attack, defense)) = input.base_stats {
        combatant
            .with_base_attack_defense(
                StatValue::from_scaled(attack).unwrap(),
                StatValue::from_scaled(defense).unwrap(),
            )
            .with_build_bonuses(input.bonuses)
    } else {
        combatant
    };
    let mut participants = vec![ParticipantSpec::new(
        TeamSide::Player,
        FormationIndex::new(0).unwrap(),
        ParticipantSource::Player,
        combatant,
    )];
    let mut enemies = Vec::new();
    if let Some(hp) = input.initial_hp {
        participants[0] = participants[0]
            .clone()
            .with_initial_state(
                ParticipantInitialState::new(
                    Hp::new(hp).unwrap(),
                    Hp::new(10_000).unwrap(),
                    Energy::from_scaled(2_000_000).unwrap(),
                    Energy::from_scaled(2_000_000).unwrap(),
                    LifeState::Alive,
                    PresenceState::Present,
                )
                .unwrap(),
            )
            .unwrap();
    }
    for index in 0..3 {
        let enemy = id(0x7d45_0001 + u32::from(index));
        enemies.push(enemy);
        builder.add_enemy(EnemyDefinition::new(enemy, form, vec![fallback]));
        let combatant = ResolvedCombatantSpec::new(
            form,
            UnitLevel::new(80).unwrap(),
            Hp::new(input.enemy_hp).unwrap(),
            Speed::from_scaled(100_000_000).unwrap(),
            ResolvedDefinitionBindings::new(vec![fallback], vec![], vec![]).unwrap(),
            CombatantSpecDigest::new([83; 32]).unwrap(),
        )
        .unwrap();
        participants.push(ParticipantSpec::new(
            TeamSide::Enemy,
            FormationIndex::new(index).unwrap(),
            ParticipantSource::EncounterEnemy(enemy),
            combatant,
        ));
    }
    builder.add_encounter(EncounterDefinition::new(encounter, enemies, vec![]));
    let spec = BattleSpec::new(
        AssemblyDigest::new([84; 32]).unwrap(),
        encounter,
        participants,
        TeamResourceSpec::new(0, 5).unwrap(),
        TeamResourceSpec::new(0, 0).unwrap(),
        ConcedePolicy::Allowed,
    )
    .unwrap();
    let battle = Battle::create(builder.build().unwrap(), spec, BattleSeed::new([85; 32])).unwrap();
    let actor = battle
        .view()
        .units_by_id()
        .find(|p| p.side() == TeamSide::Player)
        .unwrap()
        .id();
    let target = if input.allied {
        actor
    } else {
        battle
            .view()
            .units_by_id()
            .find(|p| p.side() == TeamSide::Enemy && p.formation().get() == 0)
            .unwrap()
            .id()
    };
    Scenario {
        battle,
        actor,
        target,
        selected,
        ultimate,
        fallback,
        implicit: input.all,
    }
}

pub(super) fn step(scenario: &mut Scenario, ability: AbilityId) -> Vec<BattleEvent> {
    let command = scenario.battle.decision().and_then(|decision| decision.legal_commands().iter()
        .find(|command| match command {
            Command::UseAbility {actor, ability: offered, primary_target, ..} => *actor == scenario.actor && *offered == ability &&
                *primary_target == if scenario.implicit && ability != scenario.fallback {None} else {Some(scenario.target)},
            Command::CommitPreparedAction {primary_target, ..} => *primary_target == if scenario.implicit {None} else {Some(scenario.target)},
            _ => false,
        }).or_else(|| decision.legal_commands().iter().find(|command| matches!(command, Command::StartBattle {..})))
        .or_else(|| decision.legal_commands().iter().find(|command| matches!(command, Command::UseAbility {ability, ..} if *ability == scenario.fallback))))
        .cloned().unwrap_or_else(|| {
            let boundary = scenario.battle.view().action_boundary().unwrap().id();
            if scenario.battle.available_ultimates().iter().any(|option| option.actor() == scenario.actor && option.ability() == ability) {
                Command::RequestUltimate {boundary, actor: scenario.actor, ability}
            } else {Command::Advance {boundary}}
        });
    let applied = scenario.battle.apply(command).unwrap();
    assert!(applied.fault().is_none(), "{:?}", applied.fault());
    applied.events().to_vec()
}
pub(super) fn action(scenario: &mut Scenario, ability: AbilityId) -> Vec<BattleEvent> {
    let mut events = Vec::new();
    for _ in 0..64 {
        let applied = step(scenario, ability);
        let finished = applied.iter().any(|event| {
            matches!(
                event.kind(),
                BattleEventKind::Action(ActionEventData::Resolved { .. })
            ) && event.cause().actor() == Some(CauseActor::Unit(scenario.actor))
                && event
                    .cause()
                    .source_definition()
                    .is_some_and(|source| source.get() == ability.get())
        });
        events.extend(applied);
        if finished {
            return events;
        }
    }
    panic!("controlled action did not resolve");
}
fn advances(events: &[BattleEvent]) -> Vec<(UnitId, i64)> {
    events
        .iter()
        .filter_map(|event| match event.kind() {
            BattleEventKind::Turn(TurnEventData::ActionGaugeChanged {
                owner,
                kind: ActionGaugeChangeKind::Advance,
                amount,
                ..
            }) => Some((*owner, amount.scaled())),
            _ => None,
        })
        .collect()
}
fn effects(battle: &Battle) -> usize {
    battle
        .view()
        .effects_by_id()
        .filter(|effect| effect.definition().get() == EFFECT)
        .count()
}

fn equipped(
    fixture: &DivergentUniverseBaselineFixture,
    family: DivergentUniverseRunFamily,
) -> DivergentUniverseAssembledBattle {
    let (flow, mut activity) = ready(fixture, family);
    let selected = fixture
        .factory()
        .decision_catalog()
        .weighted_curio_attack_debuffs()[0]
        .weighted_curio
        .clone();
    let hash = activity.state_hash();
    fixture
        .factory()
        .weighted_curio_runtime()
        .unwrap()
        .replace_accepted_loadout(
            &flow,
            &mut activity,
            hash,
            WeightedCurioSlotLimit::new(1).unwrap(),
            &[selected],
        )
        .unwrap();
    assemble(fixture, &flow, &activity)
}
#[test]
fn weighted_curio_attack_debuff_executes_both_paths_once_per_attack_target_and_refreshes() {
    let fixture =
        DivergentUniverseBaselineFixture::production_for_source_party([1008, 1005, 1009, 1105])
            .unwrap();
    for (source, path) in [
        (1008, CombatPath::Destruction),
        (1005, CombatPath::Nihility),
    ] {
        let form = fixture
            .core()
            .character_form_for_source_avatar(source)
            .unwrap();
        assert_eq!(
            fixture
                .core()
                .build_catalog()
                .character(form)
                .unwrap()
                .path(),
            path
        );
    }
    for family in FAMILIES {
        let assembled = equipped(&fixture, family);
        for formation in [0, 1] {
            for all in [false, true] {
                let mut probe = scenario(
                    &assembled,
                    Probe {
                        formation,
                        all,
                        ..Probe::default()
                    },
                );
                let ability = probe.selected;
                let events = action(&mut probe, ability);
                let expected = if all { 3 } else { 1 };
                assert_eq!(advances(&events).len(), expected);
                assert!(advances(&events).iter().all(|(_, value)| *value == 200_000));
                assert_eq!(effects(&probe.battle), expected);
                assert_eq!(
                    probe
                        .battle
                        .view()
                        .modifier_instances_by_id()
                        .filter(|modifier| modifier.definition().get() >> 16 == 0x7e68)
                        .count(),
                    expected * 6
                );
                for effect in probe
                    .battle
                    .view()
                    .effects_by_id()
                    .filter(|effect| effect.definition().get() == EFFECT)
                {
                    assert_eq!(effect.remaining(), Some(1));
                    assert_eq!(effect.duration_clock(), DurationClock::TargetTurnEnd);
                }
                let ability = probe.ultimate;
                let refresh = action(&mut probe, ability);
                assert_eq!(advances(&refresh).len(), expected);
                assert_eq!(effects(&probe.battle), expected);
                assert_eq!(
                    probe
                        .battle
                        .view()
                        .modifier_instances_by_id()
                        .filter(|modifier| modifier.definition().get() >> 16 == 0x7e68)
                        .count(),
                    expected * 6
                );
                let mut repeat = scenario(
                    &assembled,
                    Probe {
                        formation,
                        all,
                        ..Probe::default()
                    },
                );
                let ability = repeat.selected;
                assert_eq!(events, action(&mut repeat, ability));
            }
        }
    }
}

#[test]
fn weighted_curio_attack_debuff_reduces_actual_enemy_damage_then_restores_after_target_turn() {
    let fixture =
        DivergentUniverseBaselineFixture::production_for_source_party([1008, 1005, 1009, 1105])
            .unwrap();
    for family in FAMILIES {
        let assembled = equipped(&fixture, family);
        for outgoing in [
            DamageClass::Direct,
            DamageClass::Dot,
            DamageClass::Additional,
            DamageClass::Elation,
        ] {
            let mut probe = scenario(
                &assembled,
                Probe {
                    outgoing,
                    ..Probe::default()
                },
            );
            let ability = probe.selected;
            action(&mut probe, ability);
            let mut amounts = Vec::new();
            let mut expired = false;
            for _ in 0..96 {
                let ability = probe.fallback;
                let events = step(&mut probe, ability);
                for event in events {
                    if let BattleEventKind::Damage(damage) = event.kind()
                        && damage.target == probe.actor
                        && event.cause().actor() == Some(CauseActor::Unit(probe.target))
                    {
                        amounts.push(damage.calculated.get());
                    }
                }
                expired |= effects(&probe.battle) == 0;
                if amounts.len() == 2 {
                    break;
                }
            }
            assert_eq!(amounts, [70, 100], "{outgoing:?}");
            assert!(expired);
            assert_eq!(effects(&probe.battle), 0);
            assert!(
                !probe
                    .battle
                    .view()
                    .modifier_instances_by_id()
                    .any(|m| m.definition().get() >> 16 == 0x7e68)
            );
        }
    }
}

#[test]
fn weighted_curio_attack_debuff_rejects_non_attack_allied_noneligible_and_dead_targets() {
    let fixture =
        DivergentUniverseBaselineFixture::production_for_source_party([1008, 1005, 1009, 1105])
            .unwrap();
    for family in FAMILIES {
        let assembled = equipped(&fixture, family);
        for input in [
            Probe {
                formation: 2,
                ..Probe::default()
            },
            Probe {
                attack: false,
                ..Probe::default()
            },
            Probe {
                allied: true,
                ..Probe::default()
            },
            Probe {
                enemy_hp: 1,
                ..Probe::default()
            },
        ] {
            let mut probe = scenario(&assembled, input);
            let ability = probe.selected;
            let events = action(&mut probe, ability);
            assert!(advances(&events).is_empty());
            assert_eq!(effects(&probe.battle), 0);
        }
    }
}

#[test]
fn weighted_curio_attack_debuff_preserves_build_activity_and_reconstructs_fresh_battles() {
    let fixture =
        DivergentUniverseBaselineFixture::production_for_source_party([1008, 1005, 1009, 1105])
            .unwrap();
    let fresh =
        DivergentUniverseBaselineFixture::production_for_source_party([1008, 1005, 1009, 1105])
            .unwrap();
    for family in FAMILIES {
        let (flow, mut activity) = ready(&fixture, family);
        let plain = assemble(&fixture, &flow, &activity);
        let runtime = fixture.factory().weighted_curio_runtime().unwrap();
        let selected = fixture
            .factory()
            .decision_catalog()
            .weighted_curio_attack_debuffs()[0]
            .weighted_curio
            .clone();
        let hash = activity.state_hash();
        runtime
            .replace_accepted_loadout(
                &flow,
                &mut activity,
                hash,
                WeightedCurioSlotLimit::new(1).unwrap(),
                &[selected],
            )
            .unwrap();
        let before = activity.canonical_state_bytes();
        let assembled = assemble(&fixture, &flow, &activity);
        let rebuilt = equipped(&fresh, family);
        assert_eq!(assembled.assembly_digest(), rebuilt.assembly_digest());
        for (old, new) in plain
            .battle_spec()
            .participants()
            .iter()
            .zip(assembled.battle_spec().participants())
        {
            assert_eq!(old.locked_combatant_digest(), new.locked_combatant_digest());
            assert_eq!(old.combatant().base_attack(), new.combatant().base_attack());
            assert_eq!(
                old.combatant().build_bonuses(),
                new.combatant().build_bonuses()
            );
            if old.side() == TeamSide::Enemy {
                assert_eq!(old.combatant(), new.combatant());
            }
        }
        let mut first = scenario(&assembled, Probe::default());
        let ability = first.selected;
        let events = action(&mut first, ability);
        let mut second = scenario(&rebuilt, Probe::default());
        let ability = second.selected;
        assert_eq!(events, action(&mut second, ability));
        assert_eq!(activity.canonical_state_bytes(), before);
        let hash = activity.state_hash();
        runtime
            .replace_accepted_loadout(
                &flow,
                &mut activity,
                hash,
                WeightedCurioSlotLimit::new(1).unwrap(),
                &[],
            )
            .unwrap();
        let removed = assemble(&fixture, &flow, &activity);
        assert!(!removed.battle_spec().participants().iter().any(|p| {
            p.combatant()
                .rule_bundles()
                .iter()
                .any(|bundle| bundle.get() >> 16 == 0x7e62)
        }));
        let mut probe = scenario(&removed, Probe::default());
        let ability = probe.selected;
        assert!(advances(&action(&mut probe, ability)).is_empty());
        let selected = fixture
            .factory()
            .decision_catalog()
            .weighted_curio_attack_debuffs()[0]
            .weighted_curio
            .clone();
        let hash = activity.state_hash();
        runtime
            .replace_accepted_loadout(
                &flow,
                &mut activity,
                hash,
                WeightedCurioSlotLimit::new(1).unwrap(),
                &[selected],
            )
            .unwrap();
        // Start the actual production handoff last: equipment mutation is not
        // a command for an Activity awaiting its live nested battle result.
        assert!(result(&fixture, &flow, &mut activity).values().len() >= 4);
    }
}
