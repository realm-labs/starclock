use super::*;

const THRILL: (&str, u32) = ("universe.blessing.612156", 2);
const RESPONSIVE: (&str, u32) = ("universe.blessing.612157", 2);
const TOTAL: &str = "universe.resonance.612121";
const RICH: &str = "universe.resonance.612122";
const FIRST_LOVE: &str = "universe.resonance.612123";

#[test]
fn goal07_p2_m03_s04_materializes_every_assigned_remembrance_rule() {
    let catalog = catalog();
    let contributions = contributions_many_with_formations(
        &catalog,
        "universe.path.remembrance",
        &[THRILL, RESPONSIVE],
        &[TOTAL, RICH, FIRST_LOVE],
        None,
        false,
    );
    for key in [
        "StageAbility_612156",
        "StageAbility_612157",
        "StageAbility_612120",
        "StageAbility_612121",
        "StageAbility_612122",
        "StageAbility_612123",
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
    assert!(
        materialization
            .combat_catalog()
            .ability(AbilityId::new(RESONANCE_ABILITY_RAW).unwrap())
            .is_some()
    );
}

#[test]
fn remembrance_resonance_orders_total_eonian_damage_and_freeze() {
    let catalog = catalog();
    let contributions = contributions_many_with_formations(
        &catalog,
        "universe.path.remembrance",
        &[THRILL, RESPONSIVE],
        &[TOTAL, RICH, FIRST_LOVE],
        None,
        false,
    );
    let materialization = materialize(&catalog, &contributions);
    let (mut battle, start) = start(
        &materialization,
        durable_spec(&materialization, 0x10, true),
        0x11,
    );
    assert!(start.fault().is_none(), "{:?}", start.fault());
    assert_eq!(
        battle.view().team(TeamSide::Player).keyed_resource(
            starclock_combat::SourceDefinitionId::new(RESONANCE_RESOURCE_RAW).unwrap()
        ),
        Some((100, 100))
    );

    let resolution = use_resonance(&mut battle);
    assert!(resolution.fault().is_none(), "{:?}", resolution.fault());
    let damage = resolution
        .events()
        .iter()
        .find_map(|event| match event.kind() {
            BattleEventKind::Damage(data)
                if data.element == Some(starclock_combat::formula::model::CombatElement::Ice) =>
            {
                Some(data.raw.scaled())
            }
            _ => None,
        });
    assert_eq!(damage, Some(60_000_000_000));

    let enemy = battle
        .view()
        .units_by_id()
        .find(|unit| unit.side() == TeamSide::Enemy)
        .unwrap()
        .id();
    let negative = battle
        .view()
        .effects_by_id()
        .filter(|effect| effect.target() == enemy)
        .map(|effect| (effect.category(), effect.remaining()))
        .collect::<Vec<_>>();
    assert!(
        negative.contains(&(starclock_combat::EffectCategory::Control, Some(2))),
        "Eonian River doubles the subsequently applied one-turn Freeze: {negative:?}"
    );
    assert!(
        negative.contains(&(starclock_combat::EffectCategory::Debuff, Some(1))),
        "both one-turn Formation debuffs remain active: {negative:?}"
    );
    assert_eq!(
        battle.view().team(TeamSide::Player).keyed_resource(
            starclock_combat::SourceDefinitionId::new(RESONANCE_RESOURCE_RAW).unwrap()
        ),
        Some((5, 100)),
        "First Love restores 5% after the one enemy becomes Frozen; events={:?}",
        resolution.events()
    );
}

#[test]
fn enhanced_freeze_blessings_restore_twelve_energy_and_create_exact_shield() {
    let catalog = catalog();
    let contributions = contributions_many(
        &catalog,
        "universe.path.remembrance",
        &[THRILL, RESPONSIVE],
        None,
        false,
    );
    let materialization = materialize(&catalog, &contributions);
    let (mut battle, start) = start(
        &materialization,
        durable_spec(&materialization, 0x12, true),
        0x13,
    );
    assert!(start.fault().is_none(), "{:?}", start.fault());
    let resolution = use_resonance(&mut battle);
    assert!(resolution.fault().is_none(), "{:?}", resolution.fault());
    assert!(
        resolution.events().iter().any(|event| {
            matches!(
                event.kind(),
                BattleEventKind::Resource(starclock_combat::ResourceEventData::Energy {
                    before,
                    after,
                    ..
                }) if before.scaled() == 0 && after.scaled() == 12_000_000
            )
        }),
        "events={:?}",
        resolution.events()
    );
    assert!(resolution.events().iter().any(|event| {
        matches!(
            event.kind(),
            BattleEventKind::Shield(starclock_combat::ShieldEventData::Applied {
                amount,
                ..
            }) if amount.get() == 24_000
        )
    }));
}

#[test]
fn first_love_starts_an_uncharged_battle_at_forty_resonance_energy() {
    let catalog = catalog();
    let contributions = contributions_many_with_formations(
        &catalog,
        "universe.path.remembrance",
        &[THRILL],
        &[FIRST_LOVE],
        None,
        false,
    );
    let materialization = materialize(&catalog, &contributions);
    let (battle, start) = start(
        &materialization,
        durable_spec(&materialization, 0x14, false),
        0x15,
    );
    assert!(start.fault().is_none(), "{:?}", start.fault());
    assert_eq!(
        battle.view().team(TeamSide::Player).keyed_resource(
            starclock_combat::SourceDefinitionId::new(RESONANCE_RESOURCE_RAW).unwrap()
        ),
        Some((40, 100))
    );
}

fn use_resonance(battle: &mut Battle) -> starclock_combat::Resolution {
    use_ready_ability(battle, RESONANCE_ABILITY_RAW)
}
