use super::*;

const RESONANCE_ABILITY: u32 = 0x7630_0001;

#[test]
fn goal07_p2_m02_s04_executes_shield_conditioned_critical_stats() {
    let catalog = catalog();
    let required = [
        ("universe.blessing.612051", 2),
        ("universe.blessing.612056", 2),
        ("universe.blessing.612057", 2),
    ];
    let contributions = contributions_many(
        &catalog,
        "universe.path.preservation",
        &required,
        None,
        false,
    );
    let materialization = materialize(&catalog, &contributions);
    let mut observed = None;
    for marker in 0xc0..=0xff {
        let (mut battle, start) = start(
            &materialization,
            durable_spec(&materialization, marker, false),
            marker.wrapping_add(1),
        );
        assert!(start.fault().is_none(), "{:?}", start.fault());
        let action = first_normal_action(&mut battle);
        if let Some(raw) = action.events().iter().find_map(|event| match event.kind() {
            BattleEventKind::Damage(data) if data.raw.scaled() > 0 => Some(data.raw.scaled()),
            _ => None,
        }) && raw % 1_000_000 != 0
        {
            observed = Some((marker, raw));
            break;
        }
    }
    assert_eq!(
        observed,
        Some((0xc0, 97_500_000)),
        "frozen seed 0xc1 must observe 24% Concentration and 45% Burst"
    );
}

#[test]
fn goal07_p2_m02_s04_executes_resonance_damage_and_forced_critical_formation() {
    let catalog = catalog();
    let required = [
        ("universe.blessing.612050", 1),
        ("universe.blessing.612051", 1),
        ("universe.blessing.612052", 1),
        ("universe.blessing.612055", 1),
        ("universe.blessing.612056", 2),
        ("universe.blessing.612057", 2),
    ];
    let resonance_damage = |formations: &[&str], marker| {
        let contributions = contributions_many_with_formations(
            &catalog,
            "universe.path.preservation",
            &required,
            formations,
            None,
            false,
        );
        let materialization = materialize(&catalog, &contributions);
        let (mut battle, start) = start(
            &materialization,
            durable_spec(&materialization, marker, true),
            marker.wrapping_add(1),
        );
        assert!(start.fault().is_none(), "{:?}", start.fault());
        let resolution = use_ready_ability(&mut battle, RESONANCE_ABILITY);
        resolution
            .events()
            .iter()
            .find_map(|event| match event.kind() {
                BattleEventKind::Damage(data) => Some(data.raw.scaled()),
                _ => None,
            })
            .unwrap_or_else(|| {
                panic!(
                    "Path Resonance damage; fault={:?}; events={:?}",
                    resolution.fault(),
                    resolution.events()
                )
            })
    };
    assert_eq!(resonance_damage(&[], 0xb0), 160_000_000_000);
    assert_eq!(
        resonance_damage(&["universe.resonance.612021"], 0xb1,),
        336_000_000_000
    );
}

#[test]
fn goal07_p2_m02_s04_executes_eutectic_shields_amber_and_energy_formation() {
    let catalog = catalog();
    let required = [
        ("universe.blessing.612051", 1),
        ("universe.blessing.612056", 1),
        ("universe.blessing.612057", 1),
    ];
    let energy = contributions_many_with_formations(
        &catalog,
        "universe.path.preservation",
        &required,
        &["universe.resonance.612023"],
        None,
        false,
    );
    let energy_materialization = materialize(&catalog, &energy);
    let (_, start_energy) = start(
        &energy_materialization,
        durable_spec(&energy_materialization, 0xb2, false),
        0xb3,
    );
    assert!(start_energy.events().iter().any(|event| {
        matches!(
            event.kind(),
            BattleEventKind::Resource(starclock_combat::ResourceEventData::TeamResource {
                before: 0,
                after: 40,
                ..
            })
        )
    }));

    let eutectic = contributions_many_with_formations(
        &catalog,
        "universe.path.preservation",
        &required,
        &["universe.resonance.612022"],
        None,
        false,
    );
    let eutectic_materialization = materialize(&catalog, &eutectic);
    let (mut battle, start_eutectic) = start(
        &eutectic_materialization,
        durable_spec(&eutectic_materialization, 0xb4, true),
        0xb5,
    );
    assert!(
        start_eutectic.fault().is_none(),
        "{:?}",
        start_eutectic.fault()
    );
    let resolution = use_ready_ability(&mut battle, RESONANCE_ABILITY);
    let shields = resolution
        .events()
        .iter()
        .filter_map(|event| match event.kind() {
            BattleEventKind::Shield(starclock_combat::ShieldEventData::Applied {
                amount, ..
            }) if amount.get() == 1_000 => Some(amount.get()),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(shields, vec![1_000; 4]);
    assert_eq!(
        resolution
            .events()
            .iter()
            .filter(|event| {
                matches!(
                    event.kind(),
                    BattleEventKind::Effect(starclock_combat::EffectEventData::Applied { .. })
                )
            })
            .count(),
        4
    );
}
