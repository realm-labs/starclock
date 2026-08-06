use super::*;

#[test]
fn goal07_p2_m03_s02_executes_entry_freeze_and_resistance_rules() {
    let catalog = catalog();
    let required = [
        ("universe.blessing.612142", 2),
        ("universe.blessing.612143", 2),
        ("universe.blessing.612144", 2),
        ("universe.blessing.612145", 2),
        ("universe.blessing.612146", 2),
        ("universe.blessing.612150", 1),
    ];
    let contributions = contributions_many(
        &catalog,
        "universe.path.remembrance",
        &required,
        None,
        false,
    );
    for key in [
        "StageAbility_612142",
        "StageAbility_612143",
        "StageAbility_612144",
        "StageAbility_612145",
        "StageAbility_612146",
        "StageAbility_612150",
    ] {
        assert!(
            contributions
                .rules()
                .iter()
                .any(|rule| rule.source_binding_key() == Some(key)),
            "{key} contribution is selected"
        );
    }
    let materialization = materialize(&catalog, &contributions);
    let (battle, start) = start(
        &materialization,
        durable_spec_with_two_enemies(&materialization, 0xe0),
        0xe1,
    );
    assert!(start.fault().is_none(), "{:?}", start.fault());
    let enemies = battle
        .view()
        .units_by_id()
        .filter(|unit| unit.side() == TeamSide::Enemy)
        .count();
    let frozen = battle
        .view()
        .effects_by_id()
        .filter(|effect| {
            effect.category() == starclock_combat::EffectCategory::Control
                && battle
                    .view()
                    .units_by_id()
                    .any(|unit| unit.id() == effect.target() && unit.side() == TeamSide::Enemy)
        })
        .count();
    assert_eq!(frozen, enemies);
    assert!(
        battle.view().modifier_instances_by_id().count() >= enemies.saturating_mul(2),
        "enhanced Maverick SPD and Freeze RES reduction modifiers are active"
    );
}

#[test]
fn remembrance_sentimentality_spreads_exact_enhanced_ice_damage_without_recursion() {
    let catalog = catalog();
    let contributions = contributions(
        &catalog,
        "universe.path.remembrance",
        Some(("universe.blessing.612143", 2)),
        None,
        false,
    );
    let roster = roster_for_forms(&catalog, [19, 2, 3, 4], None);
    let materialization = UniverseBattleMaterializer
        .compile(&catalog, &roster, &contributions)
        .unwrap();
    let spec = durable_spec_with_two_enemies(&materialization, 0xe2);
    let enemy_count = spec
        .participants()
        .iter()
        .filter(|participant| participant.side() == TeamSide::Enemy)
        .count();
    let (mut battle, start) = start(&materialization, spec, 0xe3);
    assert!(start.fault().is_none(), "{:?}", start.fault());
    let resolution = first_normal_action(&mut battle);
    assert!(resolution.fault().is_none(), "{:?}", resolution.fault());
    let direct = resolution
        .events()
        .iter()
        .find_map(|event| match event.kind() {
            BattleEventKind::Damage(data)
                if data.class == starclock_combat::formula::model::DamageClass::Direct
                    && data.element
                        == Some(starclock_combat::formula::model::CombatElement::Ice) =>
            {
                Some(data.applied.get())
            }
            _ => None,
        })
        .expect("the Ice character's basic attack deals direct Ice damage");
    let splash = resolution
        .events()
        .iter()
        .filter_map(|event| match event.kind() {
            BattleEventKind::Damage(data)
                if data.class == starclock_combat::formula::model::DamageClass::Additional
                    && data.element
                        == Some(starclock_combat::formula::model::CombatElement::Ice) =>
            {
                Some(data.applied.get())
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        splash.len(),
        enemy_count - 1,
        "Sentimentality emits one event per other enemy: {:?}",
        resolution
            .events()
            .iter()
            .map(|event| (event.cause(), event.kind()))
            .collect::<Vec<_>>()
    );
    assert!(
        splash
            .iter()
            .all(|amount| *amount == direct.saturating_mul(24) / 100),
        "enhanced Sentimentality deals exactly 24% to every other enemy: direct={direct}, splash={splash:?}"
    );
}

#[test]
fn remembrance_shudder_selects_an_eligible_enemy_and_expires_after_two_target_turns() {
    use starclock_combat::{ToughnessEventData, TurnEventData, formula::model::CombatElement};

    let catalog = catalog();
    let contributions = contributions(
        &catalog,
        "universe.path.remembrance",
        Some(("universe.blessing.612145", 2)),
        None,
        false,
    );
    let roster = roster_for_forms_with_ability_kinds(
        &catalog,
        [19, 2, 3, 4],
        None,
        &[AbilityKind::Ultimate],
        true,
    );
    let materialization = UniverseBattleMaterializer
        .compile(&catalog, &roster, &contributions)
        .unwrap();

    let mut selected = None;
    for marker in 0xe4..=0xef {
        let spec = durable_spec_with_two_enemies(&materialization, marker);
        let (mut battle, start) = start(&materialization, spec, marker.wrapping_add(1));
        assert!(start.fault().is_none(), "{:?}", start.fault());
        let eligible = battle
            .view()
            .units_by_id()
            .filter(|unit| {
                unit.side() == TeamSide::Enemy && !unit.weaknesses().contains(&CombatElement::Ice)
            })
            .map(|unit| unit.id())
            .collect::<Vec<_>>();
        if eligible.is_empty() {
            continue;
        }
        let option = battle
            .available_ultimates()
            .into_iter()
            .find(|option| {
                catalog
                    .simulation_catalog()
                    .combat_catalog()
                    .ability(option.ability())
                    .and_then(|definition| definition.action())
                    .is_some_and(|action| action.kind() == AbilityKind::Ultimate)
            })
            .expect("one authored Ultimate is legal");
        let command = battle.request_ultimate_command(option).unwrap();
        let resolution = apply_action_command(&mut battle, command);
        assert!(resolution.fault().is_none(), "{:?}", resolution.fault());
        let added = resolution
            .events()
            .iter()
            .find_map(|event| match event.kind() {
                BattleEventKind::Toughness(ToughnessEventData::WeaknessAdded {
                    target,
                    element: CombatElement::Ice,
                    duration_turns: Some(2),
                    ..
                }) => Some(*target),
                _ => None,
            });
        if let Some(target) = added {
            selected = Some((battle, target, eligible));
            break;
        }
    }
    let (mut battle, target, eligible) =
        selected.expect("a deterministic 70% Shudder draw succeeds in the bounded seed fixture");
    assert!(
        eligible.contains(&target),
        "enhanced Shudder only selects enemies that lacked Ice weakness"
    );
    assert!(
        battle
            .view()
            .units_by_id()
            .find(|unit| unit.id() == target)
            .unwrap()
            .weaknesses()
            .contains(&CombatElement::Ice)
    );

    let mut target_turns = 0;
    let mut removed = false;
    for _ in 0..80 {
        if battle.view().phase().is_terminal() {
            break;
        }
        let resolution = advance_battle(&mut battle);
        assert!(resolution.fault().is_none(), "{:?}", resolution.fault());
        target_turns += resolution
            .events()
            .iter()
            .filter(|event| {
                matches!(
                    event.kind(),
                    BattleEventKind::Turn(TurnEventData::Started { owner, .. })
                        if *owner == target
                )
            })
            .count();
        removed |= resolution.events().iter().any(|event| {
            matches!(
                event.kind(),
                BattleEventKind::Toughness(ToughnessEventData::WeaknessRemoved {
                    target: removed_target,
                    element: CombatElement::Ice,
                    ..
                }) if *removed_target == target
            )
        });
        if removed {
            break;
        }
    }
    assert_eq!(target_turns, 2);
    assert!(
        removed,
        "the temporary weakness emits its typed removal event"
    );
    assert!(
        !battle
            .view()
            .units_by_id()
            .find(|unit| unit.id() == target)
            .unwrap()
            .weaknesses()
            .contains(&CombatElement::Ice)
    );
}

fn advance_battle(battle: &mut Battle) -> starclock_combat::Resolution {
    if battle.view().phase() == starclock_combat::BattlePhase::ReadyToAdvance {
        return battle.advance().expect("action boundary advances");
    }
    let decision = battle.decision().expect("nonterminal fixture").clone();
    let command = decision
        .legal_commands()
        .iter()
        .find(|command| matches!(command, Command::UseAbility { .. }))
        .expect("fixture decision exposes a progress command")
        .clone();
    battle.apply(command).expect("accepted fixture command")
}
