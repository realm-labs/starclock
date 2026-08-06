use super::*;
use starclock_combat::formula::model::DamageClass;

const ASTA_FORM: u32 = 8;
const ASTA_TECHNIQUE: u32 = 20_012;

#[test]
fn goal07_p3_m11_s01_executes_every_assigned_curio_and_fixture_family() {
    let catalog = catalog();
    let runtime = CurioRuntimeCatalog::compile(&catalog).unwrap();
    let assigned = [
        "universe.curio.1",
        "universe.curio.102",
        "universe.curio.104",
        "universe.curio.106",
        "universe.curio.107",
        "universe.curio.11",
        "universe.curio.110",
        "universe.curio.111",
    ];
    for stable_key in assigned {
        let definition = runtime
            .definitions()
            .iter()
            .find(|definition| definition.stable_key() == stable_key)
            .expect("assigned Curio");
        assert!(
            definition
                .states()
                .iter()
                .any(|state| state.id() == definition.initial_state())
        );
        let snapshot = contributions(
            &catalog,
            "universe.path.abundance",
            None,
            Some(stable_key),
            false,
        );
        assert!(
            snapshot
                .rules()
                .iter()
                .any(|rule| rule.source_binding_key().is_some())
        );
    }
}

fn direct_damage(resolution: &starclock_combat::Resolution) -> i64 {
    resolution
        .events()
        .iter()
        .filter_map(|event| match event.kind() {
            BattleEventKind::Damage(data) if data.class == DamageClass::Direct => {
                Some(data.raw.scaled())
            }
            _ => None,
        })
        .sum()
}

fn all_damage(resolution: &starclock_combat::Resolution) -> i64 {
    resolution
        .events()
        .iter()
        .filter_map(|event| match event.kind() {
            BattleEventKind::Damage(data) => Some(data.raw.scaled()),
            _ => None,
        })
        .sum()
}

#[test]
fn family_ties_scales_allied_damage_from_the_destroyed_curio_counter() {
    let catalog = catalog();
    let plain = contributions(&catalog, "universe.path.abundance", None, None, false);
    let family = contributions_with_destroyed_curios(
        &catalog,
        "universe.path.abundance",
        "universe.curio.104",
        2,
    );
    let plain = materialize(&catalog, &plain);
    let family = materialize(&catalog, &family);
    let (mut plain_battle, _) = start(&plain, durable_spec(&plain, 0xb1, false), 0xb2);
    let (mut family_battle, _) = start(&family, durable_spec(&family, 0xb1, false), 0xb2);

    let plain_damage = direct_damage(&first_normal_action(&mut plain_battle));
    let family_damage = direct_damage(&first_normal_action(&mut family_battle));
    assert!(plain_damage > 0);
    assert_eq!(family_damage, plain_damage * 8 / 5);
}

#[test]
fn doctors_robe_enters_battle_with_full_resonance_energy() {
    let catalog = catalog();
    let plain_contributions = contributions(&catalog, "universe.path.hunt", None, None, false);
    let contributions = contributions(
        &catalog,
        "universe.path.hunt",
        None,
        Some("universe.curio.11"),
        false,
    );
    let plain_materialization = materialize(&catalog, &plain_contributions);
    let materialization = materialize(&catalog, &contributions);
    let spec = durable_spec(&materialization, 0xb3, false);
    let resonance = spec
        .resources(TeamSide::Player)
        .keyed()
        .iter()
        .find(|resource| resource.id().get() == RESONANCE_RESOURCE_RAW)
        .expect("Doctor's Robe materializes the Path Resonance resource");
    assert_eq!(resonance.initial(), resonance.maximum());
    assert!(resonance.initial() > 0);

    let (mut plain_battle, _) = start(
        &plain_materialization,
        durable_spec(&plain_materialization, 0xb3, true),
        0xb4,
    );
    let (mut battle, _) = start(&materialization, spec, 0xb4);
    let plain_resolution = use_ready_ability(&mut plain_battle, RESONANCE_ABILITY_RAW);
    let resolution = use_ready_ability(&mut battle, RESONANCE_ABILITY_RAW);
    let plain_damage = all_damage(&plain_resolution);
    let robe_damage = all_damage(&resolution);
    assert!(plain_damage > 0);
    assert_eq!(robe_damage, plain_damage * 7 / 5);
}

#[test]
fn tonic_applies_both_technique_damage_terms_to_the_selected_technique() {
    let catalog = catalog();
    let roster = roster_for_forms(
        &catalog,
        [ASTA_FORM, 1, 2, 3],
        Some((ASTA_FORM, ASTA_TECHNIQUE)),
    );
    let plain = contributions(&catalog, "universe.path.abundance", None, None, false);
    let tonic = contributions(
        &catalog,
        "universe.path.abundance",
        None,
        Some("universe.curio.111"),
        false,
    );
    let plain = selected_technique_materialization(&catalog, &roster, &plain, 0x7540_0111);
    let tonic = selected_technique_materialization(&catalog, &roster, &tonic, 0x7540_0112);

    let plain_damage = start_selected_technique_damage(&plain, 0xb5);
    let tonic_damage = start_selected_technique_damage(&tonic, 0xb5);
    assert!(plain_damage > 0);
    assert!(
        tonic_damage >= plain_damage * 3 + 200_000_000_000,
        "Tonic must add +200% and 200% of the actor's 100,000 HP; plain={plain_damage}, tonic={tonic_damage}"
    );
}

fn selected_technique_materialization(
    catalog: &Arc<UniverseCatalog>,
    roster: &UniverseBattleRoster,
    contributions: &UniverseBattleContributionSet,
    option_raw: u64,
) -> UniverseBattleMaterialization {
    UniverseBattleMaterializer
        .compile_with_technique(
            catalog,
            roster,
            contributions,
            UniverseBattleTechniqueDefinition::new(
                ActivityOptionId::new(option_raw).unwrap(),
                ParticipantId::new(1).unwrap(),
                AbilityId::new(ASTA_TECHNIQUE).unwrap(),
                1,
                TechniqueEngagement::Engage,
            )
            .unwrap(),
        )
        .unwrap()
}

fn start_selected_technique_damage(
    materialization: &UniverseBattleMaterialization,
    marker: u8,
) -> i64 {
    let selected = materialization.overlay().bindings()[0]
        .preparation()
        .variants()
        .iter()
        .find(|variant| !variant.techniques().is_empty())
        .expect("selected technique variant");
    let mut battle = Battle::create(
        Arc::clone(materialization.combat_catalog()),
        selected.battle_spec().clone(),
        BattleSeed::new([marker; 32]),
    )
    .unwrap();
    let resolution = battle
        .apply(Command::StartBattle {
            decision: battle.decision().unwrap().id(),
        })
        .unwrap();
    direct_damage(&resolution)
}
