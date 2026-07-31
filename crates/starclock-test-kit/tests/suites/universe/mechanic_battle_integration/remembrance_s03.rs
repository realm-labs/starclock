use super::*;

const MAVERICK: (&str, u32) = ("universe.blessing.612146", 2);

#[test]
fn goal07_p2_m03_s03_materializes_every_assigned_remembrance_rule() {
    let catalog = catalog();
    let required = [
        MAVERICK,
        ("universe.blessing.612150", 2),
        ("universe.blessing.612151", 2),
        ("universe.blessing.612152", 2),
        ("universe.blessing.612153", 2),
        ("universe.blessing.612154", 2),
        ("universe.blessing.612155", 2),
    ];
    let contributions = contributions_many(
        &catalog,
        "universe.path.remembrance",
        &required,
        None,
        false,
    );
    for key in [
        "StageAbility_612150",
        "StageAbility_612151",
        "StageAbility_612152",
        "StageAbility_612153",
        "StageAbility_612154",
        "StageAbility_612155",
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
        durable_spec_with_two_enemies(&materialization, 0xf0),
        0xf1,
    );
    assert!(
        start.fault().is_none(),
        "{:?}; events={:?}",
        start.fault(),
        start.events()
    );
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
        battle.view().modifier_instances_by_id().count() >= enemies * 4,
        "Freeze-linked damage, vulnerability, CRIT and resistance modifiers are active"
    );
}

#[test]
fn lost_memory_freezes_on_the_first_attack_crossing_below_half_hp() {
    let catalog = catalog();
    let contributions = contributions(
        &catalog,
        "universe.path.remembrance",
        Some(("universe.blessing.612152", 2)),
        None,
        false,
    );
    let materialization = materialize(&catalog, &contributions);
    let maximum = 10_000;
    let (mut battle, start) = start(
        &materialization,
        durable_spec_with_enemy_profile(
            &materialization,
            0xf2,
            false,
            Some(Speed::from_scaled(1_000_000).unwrap()),
            Hp::new(maximum).unwrap(),
        ),
        0xf3,
    );
    assert!(start.fault().is_none(), "{:?}", start.fault());

    let mut crossing = None;
    for _ in 0..240 {
        if battle.decision().is_none() {
            break;
        }
        let resolution = apply_kind(&mut battle, &catalog, AbilityKind::Basic);
        assert!(resolution.fault().is_none(), "{:?}", resolution.fault());
        let crossed = resolution.events().iter().any(|event| {
            matches!(
                event.kind(),
                BattleEventKind::Damage(data)
                    if data.hp_before.get() * 2 >= maximum
                        && data.hp_after.get() * 2 < maximum
            )
        });
        if crossed {
            let control_applied = resolution.events().iter().any(|event| {
                let BattleEventKind::Effect(starclock_combat::EffectEventData::Applied {
                    definition,
                    ..
                }) = event.kind()
                else {
                    return false;
                };
                materialization
                    .combat_catalog()
                    .effect(*definition)
                    .and_then(|effect| effect.runtime())
                    .is_some_and(|runtime| {
                        runtime.category() == starclock_combat::EffectCategory::Control
                    })
            });
            crossing = Some(control_applied);
            break;
        }
    }
    assert_eq!(
        crossing,
        Some(true),
        "enhanced Lost Memory applies its guaranteed one-turn Freeze on the crossing hit"
    );
}

#[test]
fn frozen_target_skill_damage_and_vulnerability_use_exact_enhanced_ratios() {
    let catalog = catalog();
    assert_damage_ratio(
        &catalog,
        &[MAVERICK],
        &[MAVERICK, ("universe.blessing.612153", 2)],
        AbilityKind::Skill,
        1_540_000,
        0xf4,
    );
    assert_damage_ratio(
        &catalog,
        &[MAVERICK],
        &[MAVERICK, ("universe.blessing.612155", 2)],
        AbilityKind::Basic,
        1_240_000,
        0xf6,
    );
}

#[test]
fn pain_and_suffering_consumes_two_enhanced_critical_exposures_by_action() {
    let catalog = catalog();
    let plain = materialize_with_skill_roster(&catalog, &[MAVERICK]);
    let exposed =
        materialize_with_skill_roster(&catalog, &[MAVERICK, ("universe.blessing.612154", 2)]);
    let (mut plain, plain_start) = start(&plain, durable_spec(&plain, 0xf8, false), 0xf9);
    let (mut exposed, exposed_start) = start(&exposed, durable_spec(&exposed, 0xf8, false), 0xf9);
    assert!(plain_start.fault().is_none(), "{:?}", plain_start.fault());
    assert!(
        exposed_start.fault().is_none(),
        "{:?}",
        exposed_start.fault()
    );
    assert_eq!(enemy_mark_count(&exposed), 2);

    let mut comparisons = Vec::new();
    let mut remaining = Vec::new();
    for _ in 0..3 {
        let plain_resolution = apply_kind(&mut plain, &catalog, AbilityKind::Basic);
        let exposed_resolution = apply_kind(&mut exposed, &catalog, AbilityKind::Basic);
        assert!(
            exposed_resolution.fault().is_none(),
            "{:?}",
            exposed_resolution.fault()
        );
        comparisons.push((
            direct_raw(&plain_resolution),
            direct_raw(&exposed_resolution),
        ));
        remaining.push(enemy_mark_count(&exposed));
    }
    assert_eq!(remaining, [1, 0, 0]);
    assert!(
        comparisons[..2]
            .iter()
            .all(|(plain, exposed)| exposed >= plain)
            && comparisons[..2]
                .iter()
                .any(|(plain, exposed)| exposed > plain),
        "the first two actions receive guaranteed CRIT exposure: {comparisons:?}"
    );
    assert_eq!(
        comparisons[2].0, comparisons[2].1,
        "after both charges are consumed the same seeded action matches the baseline"
    );
}

fn assert_damage_ratio(
    catalog: &Arc<UniverseCatalog>,
    plain_rules: &[(&str, u32)],
    buffed_rules: &[(&str, u32)],
    kind: AbilityKind,
    ratio: i64,
    marker: u8,
) {
    let plain = materialize_with_skill_roster(catalog, plain_rules);
    let buffed = materialize_with_skill_roster(catalog, buffed_rules);
    let (mut plain, plain_start) = start(&plain, durable_spec(&plain, marker, false), marker + 1);
    let (mut buffed, buffed_start) =
        start(&buffed, durable_spec(&buffed, marker, false), marker + 1);
    assert!(plain_start.fault().is_none(), "{:?}", plain_start.fault());
    assert!(buffed_start.fault().is_none(), "{:?}", buffed_start.fault());
    let plain = direct_raw(&apply_kind(&mut plain, catalog, kind));
    let buffed = direct_raw(&apply_kind(&mut buffed, catalog, kind));
    assert_eq!(
        i128::from(buffed) * 1_000_000,
        i128::from(plain) * i128::from(ratio)
    );
}

fn materialize_with_skill_roster(
    catalog: &Arc<UniverseCatalog>,
    required: &[(&str, u32)],
) -> UniverseBattleMaterialization {
    let contributions =
        contributions_many(catalog, "universe.path.remembrance", required, None, false);
    let roster = roster_for_forms_with_ability_kinds(
        catalog,
        [1, 2, 3, 4],
        None,
        &[AbilityKind::Skill],
        false,
    );
    UniverseBattleMaterializer
        .compile(catalog, &roster, &contributions)
        .unwrap()
}

fn apply_kind(
    battle: &mut Battle,
    catalog: &Arc<UniverseCatalog>,
    kind: AbilityKind,
) -> starclock_combat::Resolution {
    if battle
        .decision()
        .is_some_and(|decision| decision.kind() == starclock_combat::DecisionKind::InterruptWindow)
    {
        let decision = battle.decision().unwrap();
        battle
            .apply(Command::PassInterruptWindow {
                decision: decision.id(),
            })
            .unwrap();
    }
    let decision = battle.decision().expect("nonterminal action decision");
    let command = decision
        .legal_commands()
        .iter()
        .find(|command| {
            matches!(
                command,
                Command::UseAbility { ability, .. }
                    if catalog
                        .simulation_catalog()
                        .combat_catalog()
                        .ability(*ability)
                        .and_then(|definition| definition.action())
                        .is_some_and(|action| action.kind() == kind)
            )
        })
        .expect("requested fixture ability is legal")
        .clone();
    battle.apply(command).unwrap()
}

fn direct_raw(resolution: &starclock_combat::Resolution) -> i64 {
    resolution
        .events()
        .iter()
        .find_map(|event| match event.kind() {
            BattleEventKind::Damage(data)
                if data.class == starclock_combat::formula::model::DamageClass::Direct =>
            {
                Some(data.raw.scaled())
            }
            _ => None,
        })
        .expect("fixture action deals direct damage")
}

fn enemy_mark_count(battle: &Battle) -> usize {
    battle
        .view()
        .effects_by_id()
        .filter(|effect| {
            effect.category() == starclock_combat::EffectCategory::Mark
                && effect.remaining().is_none()
                && battle
                    .view()
                    .units_by_id()
                    .any(|unit| unit.id() == effect.target() && unit.side() == TeamSide::Enemy)
        })
        .count()
}
