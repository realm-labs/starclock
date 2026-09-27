//! Actual Sora-backed shield lowering, exercised by accepted shared commands.

use crate::divergent_universe::{
    DivergentUniverseAssembledBattle, DivergentUniverseBaselineFixture,
    tests::{
        curio_battle_grants::{ready, result},
        curio_battle_stats::assemble,
    },
    weighted_curio::WeightedCurioSlotLimit,
};
use starclock_combat::{
    AbilityId, ActionEventData, AssemblyDigest, Battle, BattleEvent, BattleEventKind, BattleSeed,
    BattleSpec, CauseActor, CombatantSpecDigest, Command, ConcedePolicy, DurationClock,
    EffectDefinitionId, Energy, FormationIndex, Hp, ParticipantSource, ParticipantSpec, Ratio,
    ResolvedCombatantSpec, ResolvedDefinitionBindings, Scalar, Speed, TeamResourceSpec, TeamSide,
    UnitId, UnitLevel,
    catalog::{
        action::{
            AbilityActionDefinition, AbilityKind, ActionHitDefinition, ActionResourcePolicy,
            HitCritPolicy, HitOperationDefinition, HitTargetGroup, OrdinaryDamageDefinition,
            OrdinaryDamageMultipliers, TargetInvalidationPolicy, TargetPattern, TargetRelation,
            UnitTargetSelector,
        },
        builder::CombatCatalogBuilder,
        definition::{
            AbilityDefinition, EncounterDefinition, EnemyDefinition, ProgramDefinition,
            SelectorDefinition, UnitDefinition,
        },
    },
};
use starclock_data::divergent_universe_catalog::DivergentUniverseRunFamily;
use std::slice;

const EFFECT: u32 = 0x7e58_0001;
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

struct Probe {
    kind: AbilityKind,
    relation: TargetRelation,
    harmony: bool,
    hits: u16,
    pattern: TargetPattern,
}
impl Default for Probe {
    fn default() -> Self {
        Self {
            kind: AbilityKind::Skill,
            relation: TargetRelation::Allied,
            harmony: true,
            hits: 3,
            pattern: TargetPattern::Single,
        }
    }
}

struct Scenario {
    battle: Battle,
    actor: UnitId,
    recipient: UnitId,
    selected: AbilityId,
    fallback: AbilityId,
    implicit_target: bool,
}

fn scenario(assembled: &DivergentUniverseAssembledBattle, input: Probe) -> Scenario {
    let mut builder = CombatCatalogBuilder::from_catalog(assembled.combat_catalog(), [71; 32]);
    let selected = id(0x7d33_0001);
    let fallback = id(0x7d33_0002);
    let form = id(0x7d34_0001);
    let encounter = id(0x7d36_0001);
    for (ability, selector, program, kind, relation, hits, damage) in [
        (
            selected,
            id(0x7d31_0001),
            id(0x7d32_0001),
            input.kind,
            input.relation,
            input.hits,
            0,
        ),
        (
            fallback,
            id(0x7d31_0002),
            id(0x7d32_0002),
            AbilityKind::Basic,
            TargetRelation::Opposing,
            1,
            5,
        ),
    ] {
        builder.add_selector(
            SelectorDefinition::new(selector).with_unit_targets(
                UnitTargetSelector::new(
                    relation,
                    if ability == selected {
                        input.pattern
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
        let operations = (0..hits)
            .map(|_| {
                ActionHitDefinition::new(vec![HitOperationDefinition::Damage(
                    OrdinaryDamageDefinition::new(
                        Scalar::checked_from_integer(damage).unwrap(),
                        OrdinaryDamageMultipliers::new([Ratio::ONE; 9]).unwrap(),
                    )
                    .unwrap(),
                )])
                .with_profile(
                    HitTargetGroup::Selected,
                    Ratio::ONE,
                    Ratio::ONE,
                    HitCritPolicy::Never,
                )
            })
            .collect();
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
        .with_hits(operations)
        .unwrap();
        builder.add_ability(
            AbilityDefinition::new(ability, program, selector, vec![]).with_action(action),
        );
    }
    builder.add_unit(UnitDefinition::new(form, vec![selected, fallback], vec![]));
    let original = assembled
        .battle_spec()
        .participants()
        .iter()
        .find(|p| {
            p.side() == TeamSide::Player && p.formation().get() == if input.harmony { 2 } else { 0 }
        })
        .unwrap()
        .combatant();
    let mut participants = Vec::new();
    for (index, hp) in [101_i64, 203, 307].into_iter().enumerate() {
        let bindings = if index == 0 {
            ResolvedDefinitionBindings::new(
                vec![selected, fallback],
                original.rule_bundles().to_vec(),
                original.modifiers().to_vec(),
            )
            .unwrap()
        } else {
            ResolvedDefinitionBindings::new(
                vec![fallback],
                original
                    .rule_bundles()
                    .iter()
                    .copied()
                    .filter(|bundle| bundle.get() >> 16 == 0x7e5c)
                    .collect(),
                vec![],
            )
            .unwrap()
        };
        let mut combatant = ResolvedCombatantSpec::new(
            form,
            UnitLevel::new(80).unwrap(),
            Hp::new(hp).unwrap(),
            Speed::from_scaled(if index == 0 { 200_000_000 } else { 100_000_000 }).unwrap(),
            bindings,
            CombatantSpecDigest::new([72; 32]).unwrap(),
        )
        .unwrap();
        if index == 0 {
            combatant = combatant
                .with_sources(original.sources().to_vec())
                .unwrap()
                .with_modifier_bindings(original.modifier_bindings().to_vec())
                .unwrap()
                .with_energy(
                    Energy::from_scaled(1_000_000).unwrap(),
                    Energy::from_scaled(1_000_000).unwrap(),
                )
                .unwrap();
        } else {
            combatant = combatant
                .with_sources(
                    original
                        .sources()
                        .iter()
                        .filter(|source| source.definition().get() >> 16 == 0x7e54)
                        .cloned()
                        .collect(),
                )
                .unwrap();
        }
        participants.push(ParticipantSpec::new(
            TeamSide::Player,
            FormationIndex::new(u8::try_from(index).unwrap()).unwrap(),
            ParticipantSource::Player,
            combatant,
        ));
    }
    let enemy = id(0x7d35_0001);
    builder.add_enemy(EnemyDefinition::new(enemy, form, vec![fallback]));
    let enemy_spec = ResolvedCombatantSpec::new(
        form,
        UnitLevel::new(80).unwrap(),
        Hp::new(10_000).unwrap(),
        Speed::from_scaled(150_000_000).unwrap(),
        ResolvedDefinitionBindings::new(vec![fallback], vec![], vec![]).unwrap(),
        CombatantSpecDigest::new([73; 32]).unwrap(),
    )
    .unwrap();
    participants.push(ParticipantSpec::new(
        TeamSide::Enemy,
        FormationIndex::new(0).unwrap(),
        ParticipantSource::EncounterEnemy(enemy),
        enemy_spec,
    ));
    builder.add_encounter(EncounterDefinition::new(encounter, vec![enemy], vec![]));
    let spec = BattleSpec::new(
        AssemblyDigest::new([74; 32]).unwrap(),
        encounter,
        participants,
        TeamResourceSpec::new(0, 5).unwrap(),
        TeamResourceSpec::new(0, 0).unwrap(),
        ConcedePolicy::Allowed,
    )
    .unwrap();
    let battle = Battle::create(builder.build().unwrap(), spec, BattleSeed::new([75; 32])).unwrap();
    let actor = battle
        .view()
        .units_by_id()
        .find(|p| p.side() == TeamSide::Player && p.formation().get() == 0)
        .unwrap()
        .id();
    let recipient = battle
        .view()
        .units_by_id()
        .find(|p| {
            p.side()
                == if input.relation != TargetRelation::Opposing {
                    TeamSide::Player
                } else {
                    TeamSide::Enemy
                }
                && p.formation().get()
                    == if input.relation != TargetRelation::Opposing {
                        if input.relation == TargetRelation::SelfUnit {
                            0
                        } else {
                            1
                        }
                    } else {
                        0
                    }
        })
        .unwrap()
        .id();
    Scenario {
        battle,
        actor,
        recipient,
        selected,
        fallback,
        implicit_target: input.relation == TargetRelation::SelfUnit
            || input.pattern == TargetPattern::All,
    }
}

fn action(scenario: &mut Scenario, ability: AbilityId, recipient: UnitId) -> Vec<BattleEvent> {
    let mut events = Vec::new();
    for _ in 0..64 {
        let command = scenario.battle.decision().and_then(|decision| decision.legal_commands().iter()
            .find(|command| match command {
                Command::UseAbility {actor, ability: offered, primary_target, ..} =>
                    *actor == scenario.actor && *offered == ability && *primary_target == if scenario.implicit_target {None} else {Some(recipient)},
                Command::CommitPreparedAction {primary_target, ..} => *primary_target == if scenario.implicit_target {None} else {Some(recipient)},
                _ => false,
            }).or_else(|| decision.legal_commands().iter().find(|command| matches!(command, Command::StartBattle {..})))
            .or_else(|| decision.legal_commands().iter().find(|command| matches!(command, Command::UseAbility {ability, ..} if *ability == scenario.fallback))))
            .cloned().unwrap_or_else(|| {
                let boundary = scenario.battle.view().action_boundary().unwrap().id();
                if scenario.battle.available_ultimates().iter().any(|option| option.actor() == scenario.actor && option.ability() == ability) {
                    Command::RequestUltimate {boundary, actor: scenario.actor, ability}
                } else { Command::Advance {boundary} }
            });
        let applied = scenario.battle.apply(command).unwrap();
        assert!(applied.fault().is_none(), "{:?}", applied.fault());
        let finished = applied.events().iter().any(|event| {
            matches!(
                event.kind(),
                BattleEventKind::Action(ActionEventData::Resolved { .. })
            ) && event.cause().actor() == Some(CauseActor::Unit(scenario.actor))
                && event
                    .cause()
                    .source_definition()
                    .is_some_and(|source| source.get() == ability.get())
        });
        events.extend_from_slice(applied.events());
        if finished {
            return events;
        }
    }
    panic!("selected controlled action was not resolved");
}

fn capacities(battle: &Battle) -> Vec<(u8, i64)> {
    let mut result = battle
        .view()
        .shields_by_id()
        .filter(|shield| shield.source_effect() == Some(id::<EffectDefinitionId>(EFFECT)))
        .map(|shield| {
            (
                battle
                    .view()
                    .units_by_id()
                    .find(|p| p.id() == shield.owner())
                    .unwrap()
                    .formation()
                    .get(),
                shield.remaining().get(),
            )
        })
        .collect::<Vec<_>>();
    result.sort();
    result
}

#[test]
fn weighted_curio_shield_executes_per_recipient_hp_once_for_all_three_ally_actions_both_families() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let fresh = DivergentUniverseBaselineFixture::production().unwrap();
    for family in FAMILIES {
        let (flow, mut activity) = ready(&fixture, family);
        let plain = assemble(&fixture, &flow, &activity);
        let selected = fixture
            .factory()
            .decision_catalog()
            .weighted_curio_shields()[0]
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
                slice::from_ref(&selected),
            )
            .unwrap();
        let before = activity.canonical_state_bytes();
        let equipped = assemble(&fixture, &flow, &activity);
        let (fresh_flow, mut fresh_activity) = ready(&fresh, family);
        let hash = fresh_activity.state_hash();
        fresh
            .factory()
            .weighted_curio_runtime()
            .unwrap()
            .replace_accepted_loadout(
                &fresh_flow,
                &mut fresh_activity,
                hash,
                WeightedCurioSlotLimit::new(1).unwrap(),
                &[selected],
            )
            .unwrap();
        let rebuilt = assemble(&fresh, &fresh_flow, &fresh_activity);
        assert_eq!(equipped.assembly_digest(), rebuilt.assembly_digest());
        for (old, new) in plain
            .battle_spec()
            .participants()
            .iter()
            .zip(equipped.battle_spec().participants())
        {
            assert_eq!(old.locked_combatant_digest(), new.locked_combatant_digest());
            assert_eq!(
                old.combatant().build_bonuses(),
                new.combatant().build_bonuses()
            );
            if old.side() == TeamSide::Enemy {
                assert_eq!(old.combatant(), new.combatant());
            }
        }
        for kind in [
            AbilityKind::Basic,
            AbilityKind::Skill,
            AbilityKind::Ultimate,
        ] {
            let mut probe = scenario(
                &equipped,
                Probe {
                    kind,
                    ..Probe::default()
                },
            );
            let ability = probe.selected;
            let recipient = probe.recipient;
            let events = action(&mut probe, ability, recipient);
            assert_eq!(capacities(&probe.battle), [(0, 35), (1, 71), (2, 107)]);
            assert_eq!(probe.battle.view().rng_draw_count(), 0);
            assert_eq!(
                probe
                    .battle
                    .view()
                    .effects_by_id()
                    .filter(|effect| effect.definition().get() == EFFECT)
                    .count(),
                3
            );
            for effect in probe
                .battle
                .view()
                .effects_by_id()
                .filter(|effect| effect.definition().get() == EFFECT)
            {
                assert_eq!(effect.duration_clock(), DurationClock::TargetTurnEnd);
                assert!(matches!(effect.remaining(), Some(1 | 2)));
            }
            let mut reconstructed = scenario(
                &rebuilt,
                Probe {
                    kind,
                    ..Probe::default()
                },
            );
            assert_eq!(events, action(&mut reconstructed, ability, recipient));
            let mut unequipped = scenario(
                &plain,
                Probe {
                    kind,
                    ..Probe::default()
                },
            );
            action(&mut unequipped, ability, recipient);
            assert!(capacities(&unequipped.battle).is_empty());
        }
        for input in [
            Probe {
                relation: TargetRelation::SelfUnit,
                ..Probe::default()
            },
            Probe {
                pattern: TargetPattern::All,
                ..Probe::default()
            },
        ] {
            let mut probe = scenario(&equipped, input);
            let ability = probe.selected;
            let recipient = probe.recipient;
            action(&mut probe, ability, recipient);
            assert_eq!(capacities(&probe.battle), [(0, 35), (1, 71), (2, 107)]);
        }
        for input in [
            Probe {
                relation: TargetRelation::Opposing,
                ..Probe::default()
            },
            Probe {
                harmony: false,
                ..Probe::default()
            },
        ] {
            let mut probe = scenario(&equipped, input);
            let ability = probe.selected;
            let recipient = probe.recipient;
            action(&mut probe, ability, recipient);
            assert!(capacities(&probe.battle).is_empty());
        }
        assert_eq!(activity.canonical_state_bytes(), before);
        assert!(result(&fixture, &flow, &mut activity).values().len() >= 4);
    }
}

#[test]
fn weighted_curio_shield_refreshes_without_capacity_stacking_then_expires_on_recipient_turns() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    for family in FAMILIES {
        let (flow, mut activity) = ready(&fixture, family);
        let selected = fixture
            .factory()
            .decision_catalog()
            .weighted_curio_shields()[0]
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
        let equipped = assemble(&fixture, &flow, &activity);
        let mut probe = scenario(&equipped, Probe::default());
        let ability = probe.selected;
        let recipient = probe.recipient;
        action(&mut probe, ability, recipient);
        let second = action(&mut probe, ability, recipient);
        assert_eq!(capacities(&probe.battle), [(0, 35), (1, 71), (2, 107)]);
        let mut damage_absorbed = second.iter().any(|event|
            matches!(event.kind(), BattleEventKind::Damage(damage) if damage.absorbed.get() > 0));
        for _ in 0..96 {
            let command = probe.battle.decision().and_then(|d| d.legal_commands().iter()
                .find(|command| matches!(command, Command::UseAbility {ability, ..} if *ability == probe.fallback)))
                .cloned().unwrap_or_else(|| Command::Advance {boundary: probe.battle.view().action_boundary().unwrap().id()});
            let applied = probe.battle.apply(command).unwrap();
            assert!(applied.fault().is_none());
            damage_absorbed |= applied
                .events()
                .iter()
                .any(|e| matches!(e.kind(), BattleEventKind::Damage(d) if d.absorbed.get() > 0));
            if capacities(&probe.battle).is_empty() {
                break;
            }
        }
        assert!(capacities(&probe.battle).is_empty());
        assert!(
            !probe
                .battle
                .view()
                .effects_by_id()
                .any(|effect| effect.definition().get() == EFFECT)
        );
        assert!(damage_absorbed);
    }
}
