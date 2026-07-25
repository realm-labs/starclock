use super::*;

const DISSOCIATION: u32 = 0x76f0_0001;
const FREEZE: u32 = 0x76f0_0002;
const DIZZINESS: u32 = 0x76f0_0011;

#[test]
fn goal07_p2_m03_s01_executes_freeze_dissociation_and_removal_damage() {
    let catalog = catalog();
    let required = [
        ("universe.blessing.612130", 2),
        ("universe.blessing.612132", 2),
        ("universe.blessing.612141", 2),
    ];
    let contributions = contributions_many(
        &catalog,
        "universe.path.remembrance",
        &required,
        None,
        false,
    );
    let materialization = materialize(&catalog, &contributions);
    let (mut battle, start) = start(
        &materialization,
        durable_spec_with_enemy_hp(
            &materialization,
            0xd0,
            false,
            starclock_combat::Hp::new(9_000_000_000_000).unwrap(),
        ),
        0xd1,
    );
    assert!(start.fault().is_none(), "{:?}", start.fault());

    advance_until_effect(&mut battle, FREEZE, 40);
    advance_until_effect(&mut battle, DISSOCIATION, 20);
    let active = battle
        .view()
        .effects_by_id()
        .map(|effect| effect.definition().get())
        .collect::<Vec<_>>();
    assert!(active.contains(&DISSOCIATION));
    assert!(active.contains(&DIZZINESS));
    assert!(
        battle.decision().is_some(),
        "battle ended before natural removal: phase={:?}, units={:?}",
        battle.view().phase(),
        battle
            .view()
            .units_by_id()
            .map(|unit| (unit.id(), unit.current_hp(), unit.maximum_hp()))
            .collect::<Vec<_>>()
    );

    let mut natural = None;
    let mut observed_ice = Vec::new();
    let mut enemy_turns = 0;
    for _ in 0..80 {
        if battle.decision().is_none() {
            break;
        }
        let resolution = advance(&mut battle);
        assert!(resolution.fault().is_none(), "{:?}", resolution.fault());
        enemy_turns += resolution
            .events()
            .iter()
            .filter(|event| {
                matches!(
                    event.kind(),
                    BattleEventKind::Turn(starclock_combat::TurnEventData::Started {
                        owner,
                        ..
                    }) if owner.get() == 5
                )
            })
            .count();
        observed_ice.extend(
            resolution
                .events()
                .iter()
                .filter_map(|event| match event.kind() {
                    BattleEventKind::Damage(data)
                        if data.element
                            == Some(starclock_combat::formula::model::CombatElement::Ice) =>
                    {
                        Some(data.raw.scaled())
                    }
                    _ => None,
                }),
        );
        natural = resolution
            .events()
            .iter()
            .find_map(|event| match event.kind() {
                BattleEventKind::Damage(data)
                    if data.class == starclock_combat::formula::model::DamageClass::Additional
                        && data.element
                            == Some(starclock_combat::formula::model::CombatElement::Ice)
                        && data.raw.scaled() == 4_989_600_000_000_000_000 =>
                {
                    Some((data.raw.scaled(), data.applied.get()))
                }
                _ => None,
            });
        if natural.is_some() {
            break;
        }
    }
    let (raw, applied) = natural.unwrap_or_else(|| {
        panic!(
            "enhanced Fuli removal damage executes; phase={:?}; enemy_turns={enemy_turns}; units={:?}; observed={observed_ice:?}",
            battle.view().phase(),
            battle
                .view()
                .units_by_id()
                .map(|unit| (unit.id(), unit.current_hp(), unit.maximum_hp()))
                .collect::<Vec<_>>()
        )
    });
    assert_eq!(raw, 4_989_600_000_000_000_000);
    assert!(applied > 0);
}

#[test]
fn remembrance_melancholia_detonates_existing_dissociation_once_per_target_action() {
    let catalog = catalog();
    let required = [
        ("universe.blessing.612130", 2),
        ("universe.blessing.612132", 2),
        ("universe.blessing.612140", 2),
        ("universe.blessing.612141", 2),
    ];
    let contributions = contributions_many(
        &catalog,
        "universe.path.remembrance",
        &required,
        None,
        false,
    );
    assert!(
        contributions
            .rules()
            .iter()
            .any(|rule| rule.source_binding_key() == Some("StageAbility_612140")),
        "Melancholia rule binding is selected"
    );
    let materialization = materialize(&catalog, &contributions);
    let (mut battle, start) = start(
        &materialization,
        durable_spec_with_enemy_profile(
            &materialization,
            0xd2,
            false,
            Some(starclock_combat::Speed::from_scaled(1_000_000).unwrap()),
            starclock_combat::Hp::new(9_000_000_000_000).unwrap(),
        ),
        0xd3,
    );
    assert!(start.fault().is_none(), "{:?}", start.fault());

    advance_until_effect(&mut battle, FREEZE, 40);
    advance_until_effect(&mut battle, DISSOCIATION, 20);
    assert!(
        battle.decision().is_some(),
        "battle ended before Melancholia: phase={:?}, units={:?}",
        battle.view().phase(),
        battle
            .view()
            .units_by_id()
            .map(|unit| (unit.id(), unit.current_hp(), unit.maximum_hp()))
            .collect::<Vec<_>>()
    );
    let mut detonation_complete = false;
    let mut observed_ice = Vec::new();
    let mut observed_damage = Vec::new();
    let mut dissociation_presence = Vec::new();
    let mut dissociation_removals = 0;
    for _ in 0..80 {
        if battle.decision().is_none() {
            break;
        }
        let resolution = advance(&mut battle);
        assert!(resolution.fault().is_none(), "{:?}", resolution.fault());
        observed_damage.extend(
            resolution
                .events()
                .iter()
                .filter_map(|event| match event.kind() {
                    BattleEventKind::Damage(data) => {
                        Some((data.class, data.element, data.raw.scaled()))
                    }
                    _ => None,
                }),
        );
        dissociation_removals += resolution
            .events()
            .iter()
            .filter(|event| {
                matches!(
                    event.kind(),
                    BattleEventKind::Effect(
                        starclock_combat::EffectEventData::Removed { definition, .. }
                    ) if definition.get() == DISSOCIATION
                )
            })
            .count();
        dissociation_presence.push(
            battle
                .view()
                .effects_by_id()
                .any(|effect| effect.definition().get() == DISSOCIATION),
        );
        observed_ice.extend(
            resolution
                .events()
                .iter()
                .filter_map(|event| match event.kind() {
                    BattleEventKind::Damage(data)
                        if data.element
                            == Some(starclock_combat::formula::model::CombatElement::Ice) =>
                    {
                        Some(data.raw.scaled())
                    }
                    _ => None,
                }),
        );
        detonation_complete = dissociation_removals >= 1 && observed_ice.len() >= 2;
        if detonation_complete {
            break;
        }
    }
    assert!(
        detonation_complete
            && observed_ice.len() == 2
            && dissociation_removals == 1
            && observed_ice.iter().copied().map(i128::from).sum::<i128>()
                == 9_979_200_000_000_000_000,
        "200% of enhanced removal damage; phase={:?}; units={:?}; presence={dissociation_presence:?}; ice={observed_ice:?}; damage={observed_damage:?}",
        battle.view().phase(),
        battle
            .view()
            .units_by_id()
            .map(|unit| (unit.id(), unit.current_hp(), unit.maximum_hp()))
            .collect::<Vec<_>>()
    );
}

fn advance_until_effect(battle: &mut Battle, definition: u32, maximum: usize) {
    for _ in 0..maximum {
        if battle
            .view()
            .effects_by_id()
            .any(|effect| effect.definition().get() == definition)
        {
            return;
        }
        let resolution = advance(battle);
        assert!(resolution.fault().is_none(), "{:?}", resolution.fault());
    }
    panic!("effect {definition:#x} did not become active");
}

fn advance(battle: &mut Battle) -> starclock_combat::Resolution {
    let decision = battle.decision().expect("nonterminal fixture").clone();
    let command = decision
        .legal_commands()
        .iter()
        .find(|command| matches!(command, Command::UseAbility { .. }))
        .or_else(|| {
            decision
                .legal_commands()
                .iter()
                .find(|command| matches!(command, Command::UseInterrupt { .. }))
        })
        .or_else(|| {
            decision
                .legal_commands()
                .iter()
                .find(|command| matches!(command, Command::PassInterruptWindow { .. }))
        })
        .unwrap_or_else(|| {
            panic!(
                "fixture decision has a progress command: kind={:?}, legal={:?}",
                decision.kind(),
                decision.legal_commands()
            )
        })
        .clone();
    battle.apply(command).expect("accepted fixture command")
}
