//! Full Sora-backed crit and additional-damage mechanic via accepted commands.
use crate::divergent_universe::{
    DivergentUniverseAssembledBattle, DivergentUniverseBaselineFixture,
    tests::{
        curio_battle_grants::{ready, result},
        curio_battle_stats::assemble,
        weighted_curio_attack_debuff::{Probe, action, scenario},
    },
    weighted_curio::WeightedCurioSlotLimit,
};
use starclock_build::light_cone::CombatPath;
use starclock_combat::{
    BattleEvent, BattleEventKind, CauseActor, ResolvedBuildBonuses, Scalar, TeamSide,
    catalog::action::{AbilityKind, HitCritPolicy},
    formula::model::{CombatElement, DamageClass},
    modifier::model::StatKind,
};
use starclock_data::{
    divergent_universe_catalog::DivergentUniverseRunFamily,
    divergent_universe_curio_catalog::DivergentUniverseCurioStateId,
};

const FAMILIES: [DivergentUniverseRunFamily; 2] = [
    DivergentUniverseRunFamily::Ordinary,
    DivergentUniverseRunFamily::Cyclical,
];
const PARTY: [u32; 4] = [1009, 1001, 1105, 1005];

fn equipped(
    fixture: &DivergentUniverseBaselineFixture,
    family: DivergentUniverseRunFamily,
) -> DivergentUniverseAssembledBattle {
    let (flow, mut activity) = ready(fixture, family);
    let selected = fixture
        .factory()
        .decision_catalog()
        .weighted_curio_support_attacks()[0]
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
fn probe() -> Probe {
    Probe {
        scaling: Some(StatKind::Atk),
        base_stats: Some((200_000_000, 300_000_000)),
        enemy_hp: 10_000_000,
        ..Probe::default()
    }
}
fn additional(events: &[BattleEvent]) -> Vec<(u64, i64)> {
    events
        .iter()
        .filter_map(|event| {
            if let BattleEventKind::Damage(damage) = event.kind()
                && damage.class == DamageClass::Additional
            {
                assert_eq!(damage.element, Some(CombatElement::Wind));
                assert_eq!(
                    event.cause().source_definition().unwrap().get(),
                    0x7e73_0001
                );
                Some((damage.target.get(), damage.calculated.get()))
            } else {
                None
            }
        })
        .collect()
}

#[test]
fn weighted_curio_support_attack_executes_all_paths_once_per_target_for_multihit_and_ultimate() {
    let fixture = DivergentUniverseBaselineFixture::production_for_source_party(PARTY).unwrap();
    for (source, path) in [
        (1009, CombatPath::Harmony),
        (1001, CombatPath::Preservation),
        (1105, CombatPath::Abundance),
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
        for formation in 0..3 {
            for all in [false, true] {
                let mut test = scenario(
                    &assembled,
                    Probe {
                        formation,
                        all,
                        ..probe()
                    },
                );
                let selected = test.selected;
                let events = action(&mut test, selected);
                assert_eq!(additional(&events).len(), if all { 3 } else { 1 });
                assert!(
                    additional(&events)
                        .iter()
                        .all(|(_, amount)| *amount == 10_500)
                );
                assert_eq!(events.iter().filter(|event| matches!(event.kind(), BattleEventKind::Damage(damage) if damage.class == DamageClass::Direct)
                    && event.cause().actor() == Some(CauseActor::Unit(test.actor))).count(), if all {9} else {3});
                let ultimate = test.ultimate;
                assert_eq!(
                    additional(&action(&mut test, ultimate)),
                    additional(&events)
                );
            }
        }
    }
}

#[test]
fn weighted_curio_support_attack_crit_stats_scale_with_roster_and_additional_cannot_crit() {
    for (party, count) in [
        ([1009, 1005, 1008, 1308], 1),
        ([1009, 1105, 1008, 1308], 2),
        (PARTY, 3),
        ([1009, 1001, 1105, 1101], 4),
    ] {
        let fixture = DivergentUniverseBaselineFixture::production_for_source_party(party).unwrap();
        for family in FAMILIES {
            let assembled = equipped(&fixture, family);
            for (stat, base, increment) in [
                (StatKind::CritRate, 50_000, 150_000),
                (StatKind::CritDamage, 500_000, 300_000),
            ] {
                let mut test = scenario(
                    &assembled,
                    Probe {
                        scaling: Some(stat),
                        coefficient: 1_000_000_000_000,
                        hits: 1,
                        ..probe()
                    },
                );
                let selected = test.selected;
                let events = action(&mut test, selected);
                let direct = events
                    .iter()
                    .find_map(|event| match event.kind() {
                        BattleEventKind::Damage(damage) if damage.class == DamageClass::Direct => {
                            Some(damage.calculated.get())
                        }
                        _ => None,
                    })
                    .unwrap();
                assert_eq!(direct, base + count * increment, "{stat:?}, count={count}");
                assert_eq!(additional(&events)[0].1, 10_500);
            }
            let bonuses = ResolvedBuildBonuses::new(
                Scalar::ONE,
                Scalar::ZERO,
                Scalar::ZERO,
                Scalar::ZERO,
                Scalar::ZERO,
                [Scalar::ZERO; 7],
            );
            let mut test = scenario(
                &assembled,
                Probe {
                    crit: HitCritPolicy::PerTarget,
                    bonuses,
                    hits: 1,
                    ..probe()
                },
            );
            let selected = test.selected;
            let events = action(&mut test, selected);
            let direct = events
                .iter()
                .find_map(|event| match event.kind() {
                    BattleEventKind::Damage(damage) if damage.class == DamageClass::Direct => {
                        Some(damage.calculated.get())
                    }
                    _ => None,
                })
                .unwrap();
            assert_eq!(direct, 300 + count * 60);
            assert_eq!(additional(&events)[0].1, 10_500);
            // Saturated rate is consumed by shared sampling, not a mode cap.
            if count == 4 {
                let bonuses = ResolvedBuildBonuses::new(
                    Scalar::from_scaled(400_000),
                    Scalar::ZERO,
                    Scalar::ZERO,
                    Scalar::ZERO,
                    Scalar::ZERO,
                    [Scalar::ZERO; 7],
                );
                let mut roll = scenario(
                    &assembled,
                    Probe {
                        crit: HitCritPolicy::PerTarget,
                        bonuses,
                        hits: 1,
                        ..probe()
                    },
                );
                let selected = roll.selected;
                let events = action(&mut roll, selected);
                assert!(events.iter().any(|event| matches!(event.kind(), BattleEventKind::Damage(damage) if damage.class == DamageClass::Direct && damage.calculated.get() == direct)));
                assert_eq!(additional(&events)[0].1, 10_500);
            }
        }
    }
}

#[test]
fn weighted_curio_support_attack_rejects_noneligible_nonattack_allied_and_defeated_targets() {
    let fixture = DivergentUniverseBaselineFixture::production_for_source_party(PARTY).unwrap();
    for family in FAMILIES {
        let assembled = equipped(&fixture, family);
        for input in [
            Probe {
                formation: 3,
                ..probe()
            },
            Probe {
                attack: false,
                ..probe()
            },
            Probe {
                allied: true,
                ..probe()
            },
            Probe {
                enemy_hp: 1,
                hits: 1,
                ..probe()
            },
        ] {
            let mut test = scenario(&assembled, input);
            let selected = test.selected;
            assert!(additional(&action(&mut test, selected)).is_empty());
        }
        let mut lethal = scenario(
            &assembled,
            Probe {
                enemy_hp: 1_000,
                hits: 1,
                ..probe()
            },
        );
        let selected = lethal.selected;
        let events = action(&mut lethal, selected);
        assert_eq!(additional(&events)[0].1, 10_500);
        assert!(events.iter().any(
            |event| matches!(event.kind(), BattleEventKind::Damage(damage)
            if damage.class == DamageClass::Additional && damage.hp_after.get() == 0)
        ));
    }
}

#[test]
fn weighted_curio_support_attack_preserves_build_activity_fresh_battles_and_unequip() {
    let fixture = DivergentUniverseBaselineFixture::production_for_source_party(PARTY).unwrap();
    let fresh = DivergentUniverseBaselineFixture::production_for_source_party(PARTY).unwrap();
    for family in FAMILIES {
        let (flow, mut activity) = ready(&fixture, family);
        let plain = assemble(&fixture, &flow, &activity);
        let runtime = fixture.factory().weighted_curio_runtime().unwrap();
        let selected = fixture
            .factory()
            .decision_catalog()
            .weighted_curio_support_attacks()[0]
            .weighted_curio
            .clone();
        let hash = activity.state_hash();
        runtime
            .replace_accepted_loadout(
                &flow,
                &mut activity,
                hash,
                WeightedCurioSlotLimit::new(1).unwrap(),
                std::slice::from_ref(&selected),
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
            assert_eq!(old.combatant().maximum_hp(), new.combatant().maximum_hp());
            assert_eq!(old.combatant().base_attack(), new.combatant().base_attack());
            assert_eq!(
                old.combatant().base_defense(),
                new.combatant().base_defense()
            );
            assert_eq!(
                old.combatant().build_bonuses(),
                new.combatant().build_bonuses()
            );
            if old.side() == TeamSide::Enemy || old.formation().get() == 3 {
                assert_eq!(old.combatant(), new.combatant());
            } else {
                assert_eq!(
                    new.combatant().modifiers().len(),
                    old.combatant().modifiers().len() + 2
                );
                assert!(
                    new.combatant()
                        .modifier_bindings()
                        .iter()
                        .filter(|binding| binding.definition().get() >> 16 == 0x7e78)
                        .all(|binding| !binding.applies_to_linked_subjects())
                );
            }
        }
        let mut first = scenario(&assembled, probe());
        let selected_ability = first.selected;
        let events = action(&mut first, selected_ability);
        let mut second = scenario(&rebuilt, probe());
        let selected_ability = second.selected;
        assert_eq!(events, action(&mut second, selected_ability));
        assert_eq!(first.battle.state_hash(), second.battle.state_hash());
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
        let mut test = scenario(&removed, probe());
        let selected_ability = test.selected;
        assert!(additional(&action(&mut test, selected_ability)).is_empty());
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
        assert!(result(&fixture, &flow, &mut activity).values().len() >= 4);
    }
}

#[test]
fn weighted_curio_support_attack_uses_maximum_not_current_hp_and_accepts_skill_attack() {
    let fixture = DivergentUniverseBaselineFixture::production_for_source_party(PARTY).unwrap();
    for family in FAMILIES {
        let assembled = equipped(&fixture, family);
        let mut test = scenario(
            &assembled,
            Probe {
                selected_kind: AbilityKind::Skill,
                initial_hp: Some(1),
                hits: 4,
                ..probe()
            },
        );
        let selected = test.selected;
        assert_eq!(
            additional(&action(&mut test, selected)),
            vec![(test.target.get(), 10_500)]
        );
        let unit = test
            .battle
            .view()
            .units_by_id()
            .find(|unit| unit.id() == test.actor)
            .unwrap();
        assert_eq!(
            (unit.current_hp().get(), unit.maximum_hp().get()),
            (1, 10_000)
        );
    }
    let fixture =
        DivergentUniverseBaselineFixture::production_for_source_party([1308, 1005, 1008, 1003])
            .unwrap();
    for family in FAMILIES {
        let assembled = equipped(&fixture, family);
        assert!(
            !assembled
                .battle_spec()
                .participants()
                .iter()
                .any(|player| player
                    .combatant()
                    .modifiers()
                    .iter()
                    .any(|modifier| modifier.get() >> 16 == 0x7e78))
        );
        let mut test = scenario(&assembled, probe());
        let selected = test.selected;
        assert!(additional(&action(&mut test, selected)).is_empty());
    }
}

#[test]
fn weighted_curio_support_attack_composes_with_real_ordinary_curio_damage_modifier() {
    let fixture = DivergentUniverseBaselineFixture::production_for_source_party(PARTY).unwrap();
    for family in FAMILIES {
        let (flow, mut activity) = ready(&fixture, family);
        let selected = fixture
            .factory()
            .decision_catalog()
            .weighted_curio_support_attacks()[0]
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
        let hash = activity.state_hash();
        fixture
            .factory()
            .curio_runtime()
            .unwrap()
            .acquire_accepted_state(
                &mut activity,
                hash,
                &DivergentUniverseCurioStateId::new("divergent-universe.curio-state.9073").unwrap(),
            )
            .unwrap();
        let assembled = assemble(&fixture, &flow, &activity);
        let mut test = scenario(
            &assembled,
            Probe {
                hits: 1,
                base_stats: Some((200_750_000, 300_500_000)),
                ..probe()
            },
        );
        let selected = test.selected;
        let events = action(&mut test, selected);
        // (10,000 + 200.75 + 300.5) * 1.5 = 15,751.875, then damage floor.
        assert_eq!(additional(&events), vec![(test.target.get(), 15_751)]);
    }
}
