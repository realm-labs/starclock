use super::*;

const FUNERAL: (&str, u32) = ("universe.blessing.612230", 2);
const MAN_IN_COVER: (&str, u32) = ("universe.blessing.612231", 2);
const EVERYTHING: (&str, u32) = ("universe.blessing.612232", 2);
const BEGINNING: (&str, u32) = ("universe.blessing.612240", 2);
const CAFE: (&str, u32) = ("universe.blessing.612241", 2);
const WILDERNESS: (&str, u32) = ("universe.blessing.612242", 2);
const SUSPICION: u32 = 0x77e0_0001;
const KAFKA_FORM: u32 = 45;
const KAFKA_ULTIMATE: u32 = 20_033;

#[test]
fn goal07_p2_m04_s01_materializes_every_assigned_nihility_rule() {
    let catalog = catalog();
    let contributions = contributions_many(
        &catalog,
        "universe.path.nihility",
        &[
            FUNERAL,
            MAN_IN_COVER,
            EVERYTHING,
            BEGINNING,
            CAFE,
            WILDERNESS,
        ],
        None,
        false,
    );
    for key in [
        "StageAbility_612230_2",
        "StageAbility_612231",
        "StageAbility_612232",
        "StageAbility_612240",
        "StageAbility_612241",
        "StageAbility_612242",
    ] {
        assert!(
            contributions
                .rules()
                .iter()
                .any(|rule| rule.source_binding_key() == Some(key)),
            "{key} contribution is selected"
        );
    }
    let roster = kafka_roster(&catalog);
    let materialization = materialize_with_roster(&catalog, &roster, &contributions);
    assert!(
        materialization
            .combat_catalog()
            .effect(starclock_combat::EffectDefinitionId::new(SUSPICION).unwrap())
            .is_some()
    );
}

#[test]
fn enhanced_suspicion_application_doubles_stacks_and_never_decays() {
    let catalog = catalog();
    let contributions = contributions_many(
        &catalog,
        "universe.path.nihility",
        &[
            FUNERAL,
            MAN_IN_COVER,
            EVERYTHING,
            BEGINNING,
            CAFE,
            WILDERNESS,
        ],
        None,
        false,
    );
    let roster = kafka_roster(&catalog);
    let materialization = materialize_with_roster(&catalog, &roster, &contributions);
    let (mut battle, started) = start(
        &materialization,
        durable_spec_with_enemy_hp(
            &materialization,
            0x20,
            false,
            Hp::new(9_000_000_000_000).unwrap(),
        ),
        0x21,
    );
    assert!(started.fault().is_none(), "{:?}", started.fault());

    let applied = use_kafka_ultimate(&mut battle);
    assert!(applied.fault().is_none(), "{:?}", applied.fault());
    let enemy = enemy(&battle);
    let initial = suspicion_stacks(&battle, enemy).unwrap_or_else(|| {
        panic!(
            "Kafka Shock applies Suspicion; events={:?}",
            applied.events()
        )
    });
    assert!(
        initial >= 6 && initial.is_multiple_of(2),
        "enhanced Café doubles every positive application: {initial}; events={:?}",
        applied.events()
    );
    let suspicion = battle
        .view()
        .effects_by_id()
        .find(|effect| effect.target() == enemy && effect.definition().get() == SUSPICION)
        .unwrap();
    assert_eq!(
        battle
            .view()
            .modifier_instances_by_id()
            .filter(|modifier| modifier.source_effect() == Some(suspicion.id()))
            .count(),
        3,
        "Suspicion owns vulnerability plus enhanced Wilderness ATK and Effect RES modifiers"
    );

    let detonations = advance_through_enemy_turn(&mut battle, enemy);
    assert!(
        detonations > 0,
        "Everything Disappeared detonates current DoTs at enemy turn start"
    );
    let after = suspicion_stacks(&battle, enemy).expect("persistent Suspicion remains");
    assert!(
        after >= initial,
        "enhanced Funeral prevents the ordinary two-stack decay: {initial} -> {after}"
    );
}

#[test]
fn ordinary_suspicion_loses_exactly_two_stacks_after_the_enemy_turn() {
    let catalog = catalog();
    let contributions = contributions_many(
        &catalog,
        "universe.path.nihility",
        &[
            ("universe.blessing.612231", 2),
            ("universe.blessing.612240", 2),
            ("universe.blessing.612241", 1),
        ],
        None,
        false,
    );
    let roster = kafka_roster(&catalog);
    let materialization = materialize_with_roster(&catalog, &roster, &contributions);
    let (mut battle, started) = start(
        &materialization,
        durable_spec_with_enemy_hp(
            &materialization,
            0x22,
            false,
            Hp::new(9_000_000_000_000).unwrap(),
        ),
        0x23,
    );
    assert!(started.fault().is_none(), "{:?}", started.fault());

    let applied = use_kafka_ultimate(&mut battle);
    assert!(applied.fault().is_none(), "{:?}", applied.fault());
    let enemy = enemy(&battle);
    let initial = suspicion_stacks(&battle, enemy).unwrap_or_else(|| {
        panic!(
            "Shock application creates Suspicion; events={:?}",
            applied.events()
        )
    });
    assert_eq!(initial, 4, "3 application stacks plus Café's extra stack");

    let _ = advance_through_enemy_turn(&mut battle, enemy);
    assert_eq!(
        suspicion_stacks(&battle, enemy),
        Some(2),
        "ordinary Suspicion decays by exactly two stacks"
    );
}

#[test]
fn beginning_and_end_spreads_suspicion_after_a_real_enemy_defeat() {
    let catalog = catalog();
    let contributions = contributions_many(
        &catalog,
        "universe.path.nihility",
        &[
            ("universe.blessing.612231", 1),
            ("universe.blessing.612232", 1),
            BEGINNING,
        ],
        None,
        false,
    );
    let roster = kafka_roster(&catalog);
    let materialization = materialize_with_roster(&catalog, &roster, &contributions);
    let (mut battle, started) = start(
        &materialization,
        durable_spec_with_two_enemy_hp(
            &materialization,
            0x24,
            [Hp::new(500).unwrap(), Hp::new(9_000_000_000_000).unwrap()],
        ),
        0x25,
    );
    assert!(started.fault().is_none(), "{:?}", started.fault());

    let applied = use_kafka_ultimate(&mut battle);
    assert!(applied.fault().is_none(), "{:?}", applied.fault());
    let source = battle
        .view()
        .units_by_id()
        .filter(|unit| unit.side() == TeamSide::Enemy)
        .min_by_key(|unit| unit.maximum_hp())
        .expect("low-HP enemy")
        .id();
    assert!(
        suspicion_stacks(&battle, source).is_some(),
        "the low-HP enemy survives long enough to own Suspicion"
    );

    for _ in 0..32 {
        let resolution = advance_targeting(&mut battle, source);
        assert!(resolution.fault().is_none(), "{:?}", resolution.fault());
        if !resolution.events().iter().any(|event| {
            matches!(
                event.kind(),
                BattleEventKind::Unit(starclock_combat::UnitEventData::Defeated {
                    unit,
                    ..
                }) if *unit == source
            )
        }) {
            continue;
        }
        assert!(resolution.events().iter().any(|event| {
            matches!(
                event.kind(),
                BattleEventKind::Effect(starclock_combat::EffectEventData::Refreshed {
                    target,
                    stacks_before,
                    stacks_after,
                    ..
                }) if *target != source && stacks_after > stacks_before
            )
        }));
        assert_eq!(suspicion_stacks(&battle, source), None);
        return;
    }
    panic!("bounded production actions did not defeat the low-HP enemy");
}

fn kafka_roster(catalog: &UniverseCatalog) -> UniverseBattleRoster {
    roster_for_forms_with_ability_kinds_and_energy(
        catalog,
        [KAFKA_FORM, 1, 2, 3],
        None,
        &[AbilityKind::Ultimate],
        true,
        120_000_000,
    )
}

fn use_kafka_ultimate(battle: &mut Battle) -> starclock_combat::Resolution {
    for _ in 0..12 {
        if let Some(command) = battle
            .available_ultimates()
            .into_iter()
            .find(|option| option.ability().get() == KAFKA_ULTIMATE)
            .and_then(|option| battle.request_ultimate_command(option))
        {
            return apply_action_command(battle, command);
        }
        let resolution = advance(battle);
        assert!(resolution.fault().is_none(), "{:?}", resolution.fault());
    }
    panic!("Kafka Ultimate interrupt was not offered");
}

fn advance_through_enemy_turn(battle: &mut Battle, enemy: starclock_combat::UnitId) -> usize {
    let mut detonations = 0;
    for _ in 0..80 {
        let resolution = advance(battle);
        assert!(resolution.fault().is_none(), "{:?}", resolution.fault());
        detonations += resolution
            .events()
            .iter()
            .filter(|event| {
                matches!(
                    event.kind(),
                    BattleEventKind::Effect(starclock_combat::EffectEventData::Detonated {
                        target,
                        ..
                    }) if *target == enemy
                )
            })
            .count();
        if resolution.events().iter().any(|event| {
            matches!(
                event.kind(),
                BattleEventKind::Turn(starclock_combat::TurnEventData::Ended {
                    owner,
                    ..
                }) if *owner == enemy
            )
        }) {
            return detonations;
        }
    }
    panic!("enemy turn did not end");
}

fn advance_targeting(
    battle: &mut Battle,
    target: starclock_combat::UnitId,
) -> starclock_combat::Resolution {
    if battle.view().phase() == starclock_combat::BattlePhase::ReadyToAdvance {
        return battle.advance().expect("action boundary advances");
    }
    let decision = battle.decision().expect("nonterminal fixture").clone();
    let command = decision
        .legal_commands()
        .iter()
        .find(|command| {
            matches!(
                command,
                Command::UseAbility {
                    primary_target: Some(primary),
                    ..
                } if *primary == target
            )
        })
        .or_else(|| {
            decision
                .legal_commands()
                .iter()
                .find(|command| matches!(command, Command::UseAbility { .. }))
        })
        .expect("fixture has a progress command")
        .clone();
    battle.apply(command).expect("fixture command is accepted")
}

fn advance(battle: &mut Battle) -> starclock_combat::Resolution {
    if battle.view().phase() == starclock_combat::BattlePhase::ReadyToAdvance {
        return battle.advance().expect("action boundary advances");
    }
    let decision = battle.decision().expect("nonterminal fixture").clone();
    let command = decision
        .legal_commands()
        .iter()
        .find(|command| matches!(command, Command::UseAbility { .. }))
        .expect("fixture has a progress command")
        .clone();
    battle.apply(command).expect("fixture command is accepted")
}

fn enemy(battle: &Battle) -> starclock_combat::UnitId {
    battle
        .view()
        .units_by_id()
        .find(|unit| unit.side() == TeamSide::Enemy)
        .expect("enemy")
        .id()
}

fn suspicion_stacks(battle: &Battle, target: starclock_combat::UnitId) -> Option<u16> {
    battle
        .view()
        .effects_by_id()
        .find(|effect| effect.target() == target && effect.definition().get() == SUSPICION)
        .map(|effect| effect.stacks())
}
