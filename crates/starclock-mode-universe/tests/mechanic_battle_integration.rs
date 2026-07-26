use std::sync::{Arc, OnceLock};

use starclock_activity::{
    ActivityOptionId, BuildDigest, LoadoutLockScope, OpaqueParticipantBuild, ParticipantId,
    ParticipantLock, ParticipantLockEntry, ParticipantPolicy, ParticipantSourceKind,
    ParticipantUniquenessScope, TechniqueEngagement,
};
use starclock_combat::{
    AbilityId, AssemblyDigest, Battle, BattleEventKind, BattleSeed, BattleSpec,
    CombatantSpecDigest, Command, Energy, Hp, KeyedTeamResourceSpec, ParticipantSource,
    ParticipantSpec, ResolvedCombatantSpec, ResolvedDefinitionBindings, Speed, StatValue,
    TeamResourceSpec, TeamResourceWavePolicy, TeamSide, UnitDefinitionId, UnitLevel,
    catalog::action::AbilityKind,
};
use starclock_mode_universe::{
    ability_runtime::{
        AbilityBoundary, AbilityExecutionContext, AbilityProjectionScope, AbilityRuntimeCatalog,
    },
    battle_contribution::{UniverseBattleContributionCompiler, UniverseBattleContributionSet},
    battle_materialization::{
        UniverseBattleMaterialization, UniverseBattleMaterializer, UniverseBattleRoster,
    },
    battle_technique::UniverseBattleTechniqueDefinition,
    blessing_runtime::BlessingRuntimeCatalog,
    catalog::UniverseCatalog,
    curio_runtime::CurioRuntimeCatalog,
    path_runtime::PathRuntimeCatalog,
    run_runtime::RunRuntimeCatalog,
};

const CORE_BUNDLE: &[u8] = include_bytes!("../../../config/generated/config.sora");
const UNIVERSE_BUNDLE: &[u8] = include_bytes!("../../../config/universe-generated/config.sora");
const RESONANCE_ABILITY_RAW: u32 = 0x7630_0001;
const RESONANCE_RESOURCE_RAW: u32 = 0x7630_0004;

#[path = "mechanic_battle_integration/abundance_s01.rs"]
mod abundance_s01;
#[path = "mechanic_battle_integration/abundance_s02.rs"]
mod abundance_s02;
#[path = "mechanic_battle_integration/abundance_s03.rs"]
mod abundance_s03;
#[path = "mechanic_battle_integration/abundance_s04.rs"]
mod abundance_s04;
#[path = "mechanic_battle_integration/destruction_s01.rs"]
mod destruction_s01;
#[path = "mechanic_battle_integration/destruction_s02.rs"]
mod destruction_s02;
#[path = "mechanic_battle_integration/destruction_s03.rs"]
mod destruction_s03;
#[path = "mechanic_battle_integration/destruction_s04.rs"]
mod destruction_s04;
#[path = "mechanic_battle_integration/elation_s01.rs"]
mod elation_s01;
#[path = "mechanic_battle_integration/hunt_s01.rs"]
mod hunt_s01;
#[path = "mechanic_battle_integration/hunt_s02.rs"]
mod hunt_s02;
#[path = "mechanic_battle_integration/hunt_s03.rs"]
mod hunt_s03;
#[path = "mechanic_battle_integration/hunt_s04.rs"]
mod hunt_s04;
#[path = "mechanic_battle_integration/nihility_s01.rs"]
mod nihility_s01;
#[path = "mechanic_battle_integration/nihility_s02.rs"]
mod nihility_s02;
#[path = "mechanic_battle_integration/nihility_s03.rs"]
mod nihility_s03;
#[path = "mechanic_battle_integration/nihility_s04.rs"]
mod nihility_s04;
#[path = "mechanic_battle_integration/preservation_s02.rs"]
mod preservation_s02;
#[path = "mechanic_battle_integration/preservation_s03.rs"]
mod preservation_s03;
#[path = "mechanic_battle_integration/preservation_s04.rs"]
mod preservation_s04;
#[path = "mechanic_battle_integration/remembrance_s01.rs"]
mod remembrance_s01;
#[path = "mechanic_battle_integration/remembrance_s02.rs"]
mod remembrance_s02;
#[path = "mechanic_battle_integration/remembrance_s03.rs"]
mod remembrance_s03;
#[path = "mechanic_battle_integration/remembrance_s04.rs"]
mod remembrance_s04;

fn catalog() -> Arc<UniverseCatalog> {
    static CATALOG: OnceLock<Arc<UniverseCatalog>> = OnceLock::new();
    Arc::clone(CATALOG.get_or_init(|| {
        let core = starclock_data::catalog::load(CORE_BUNDLE).expect("core catalog");
        UniverseCatalog::load(UNIVERSE_BUNDLE, core).expect("Universe catalog")
    }))
}

fn roster(catalog: &UniverseCatalog) -> UniverseBattleRoster {
    roster_for_forms(catalog, [1, 2, 3, 4], None)
}

fn roster_for_forms(
    catalog: &UniverseCatalog,
    forms: [u32; 4],
    technique: Option<(u32, u32)>,
) -> UniverseBattleRoster {
    roster_for_forms_with_ability_kinds(catalog, forms, technique, &[], false)
}

fn roster_for_forms_with_ability_kinds(
    catalog: &UniverseCatalog,
    forms: [u32; 4],
    technique: Option<(u32, u32)>,
    extra_kinds: &[AbilityKind],
    full_energy: bool,
) -> UniverseBattleRoster {
    roster_for_forms_with_ability_kinds_and_energy(
        catalog,
        forms,
        technique,
        extra_kinds,
        full_energy,
        100_000_000,
    )
}

fn roster_for_forms_with_ability_kinds_and_energy(
    catalog: &UniverseCatalog,
    forms: [u32; 4],
    technique: Option<(u32, u32)>,
    extra_kinds: &[AbilityKind],
    full_energy: bool,
    maximum_energy_scaled: i64,
) -> UniverseBattleRoster {
    let policy = ParticipantPolicy::new(
        1,
        1,
        4,
        ParticipantUniquenessScope::Activity,
        LoadoutLockScope::Activity,
    )
    .unwrap();
    let mut lock_entries = Vec::new();
    let mut combatants = Vec::new();
    for (index, form_raw) in forms.into_iter().enumerate() {
        let index = u8::try_from(index).unwrap();
        let form = UnitDefinitionId::new(form_raw).unwrap();
        let unit = catalog
            .simulation_catalog()
            .combat_catalog()
            .unit(form)
            .unwrap();
        let basic = unit
            .abilities()
            .iter()
            .copied()
            .find(|ability| {
                catalog
                    .simulation_catalog()
                    .combat_catalog()
                    .ability(*ability)
                    .and_then(|definition| definition.action())
                    .is_some_and(|action| action.kind() == AbilityKind::Basic)
            })
            .unwrap();
        let mut abilities = vec![basic];
        for kind in extra_kinds {
            let ability = unit
                .abilities()
                .iter()
                .copied()
                .find(|ability| {
                    catalog
                        .simulation_catalog()
                        .combat_catalog()
                        .ability(*ability)
                        .and_then(|definition| definition.action())
                        .is_some_and(|action| action.kind() == *kind)
                })
                .expect("requested ability kind is available on fixture form");
            abilities.push(ability);
        }
        if let Some((technique_form, technique_ability)) = technique
            && technique_form == form_raw
        {
            abilities.push(AbilityId::new(technique_ability).unwrap());
        }
        abilities.sort_unstable();
        abilities.dedup();
        let maximum_energy = Energy::from_scaled(maximum_energy_scaled).unwrap();
        let combatant = ResolvedCombatantSpec::new(
            form,
            UnitLevel::new(80).unwrap(),
            Hp::new(100_000).unwrap(),
            Speed::from_scaled(200_000_000 - i64::from(index) * 1_000_000).unwrap(),
            ResolvedDefinitionBindings::new(abilities, Vec::new(), Vec::new()).unwrap(),
            CombatantSpecDigest::new([index + 1; 32]).unwrap(),
        )
        .unwrap()
        .with_base_attack_defense(
            StatValue::from_scaled(100_000_000).unwrap(),
            StatValue::from_scaled(100_000_000).unwrap(),
        )
        .with_energy(
            if full_energy {
                maximum_energy
            } else {
                Energy::ZERO
            },
            maximum_energy,
        )
        .unwrap();
        let participant = ParticipantId::new(u32::from(index) + 1).unwrap();
        lock_entries.push(
            ParticipantLockEntry::new(
                participant,
                0,
                index,
                form,
                OpaqueParticipantBuild::new(
                    combatant.digest(),
                    BuildDigest::new([index + 17; 32]).unwrap(),
                    "mechanic-integration-v1",
                    ParticipantSourceKind::FixedResolved,
                )
                .unwrap(),
            )
            .unwrap(),
        );
        combatants.push((participant, combatant));
    }
    let lock = ParticipantLock::seal(policy, lock_entries).unwrap();
    UniverseBattleRoster::new(&lock, combatants).unwrap()
}

fn contributions(
    catalog: &Arc<UniverseCatalog>,
    path_key: &str,
    required_blessing: Option<(&str, u32)>,
    curio_key: Option<&str>,
    ability_tree: bool,
) -> UniverseBattleContributionSet {
    let required = required_blessing.into_iter().collect::<Vec<_>>();
    contributions_many(catalog, path_key, &required, curio_key, ability_tree)
}

fn contributions_many(
    catalog: &Arc<UniverseCatalog>,
    path_key: &str,
    required_blessings: &[(&str, u32)],
    curio_key: Option<&str>,
    ability_tree: bool,
) -> UniverseBattleContributionSet {
    contributions_many_with_formations(
        catalog,
        path_key,
        required_blessings,
        &[],
        curio_key,
        ability_tree,
    )
}

fn contributions_many_with_formations(
    catalog: &Arc<UniverseCatalog>,
    path_key: &str,
    required_blessings: &[(&str, u32)],
    formation_keys: &[&str],
    curio_key: Option<&str>,
    ability_tree: bool,
) -> UniverseBattleContributionSet {
    let path_definition = catalog
        .paths()
        .iter()
        .find(|path| path.stable_key() == path_key)
        .unwrap();
    let required = required_blessings.iter().map(|(key, level)| {
        (
            catalog
                .blessings()
                .iter()
                .find(|blessing| blessing.stable_key() == *key)
                .unwrap()
                .id(),
            *level,
        )
    });
    let mut owned = required.collect::<Vec<_>>();
    let required_count = match formation_keys.len() {
        0 => 3,
        1 => 6,
        2 => 10,
        _ => 14,
    };
    for blessing in path_definition.blessings() {
        if owned.len() >= required_count {
            break;
        }
        if owned.iter().all(|entry| entry.0 != *blessing) {
            owned.push((*blessing, 1));
        }
    }
    owned.sort_unstable_by_key(|entry| entry.0);
    let blessings = BlessingRuntimeCatalog::compile(catalog)
        .unwrap()
        .contributions_from_owned(&owned)
        .unwrap();
    let formations = formation_keys
        .iter()
        .map(|key| {
            (
                catalog
                    .resonances()
                    .iter()
                    .find(|formation| formation.stable_key() == *key)
                    .unwrap()
                    .id(),
                1,
            )
        })
        .collect::<Vec<_>>();
    let path = PathRuntimeCatalog::compile(catalog)
        .unwrap()
        .contributions_with_formation_slots(
            path_definition.id(),
            &blessings,
            &formations,
            u8::try_from(formations.len()).unwrap(),
        )
        .unwrap();

    let curio_runtime = CurioRuntimeCatalog::compile(catalog).unwrap();
    let selected = curio_key.map(|key| {
        curio_runtime
            .definitions()
            .iter()
            .find(|definition| definition.stable_key() == key)
            .unwrap()
    });
    let inventory = selected
        .iter()
        .map(|definition| (definition.curio(), 1))
        .collect::<Vec<_>>();
    let states = selected
        .iter()
        .map(|definition| (definition.curio(), definition.initial_state()))
        .collect::<Vec<_>>();
    let charges = selected
        .iter()
        .map(|definition| {
            let state = definition
                .states()
                .iter()
                .find(|state| state.id() == definition.initial_state())
                .unwrap();
            (definition.curio(), state.maximum_charges().unwrap_or(0))
        })
        .collect::<Vec<_>>();
    let curios = curio_runtime
        .contributions_from_owned(&inventory, &states, &charges)
        .unwrap();

    let selected_abilities = if ability_tree {
        catalog
            .ability_tree_nodes()
            .iter()
            .map(|node| node.id())
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };
    let abilities = RunRuntimeCatalog::compile(catalog)
        .unwrap()
        .ability_contributions(&selected_abilities)
        .unwrap();
    let projection = AbilityRuntimeCatalog::compile(catalog)
        .unwrap()
        .project(
            &selected_abilities,
            AbilityExecutionContext::new(
                AbilityProjectionScope::Battle,
                AbilityBoundary::BattleStart,
                3,
                false,
            ),
        )
        .unwrap();
    UniverseBattleContributionCompiler::compile(Arc::clone(catalog))
        .unwrap()
        .compile_snapshot(&path, &blessings, &curios, &abilities, &projection)
        .unwrap()
}

fn materialize(
    catalog: &Arc<UniverseCatalog>,
    contributions: &UniverseBattleContributionSet,
) -> UniverseBattleMaterialization {
    UniverseBattleMaterializer
        .compile(catalog, &roster(catalog), contributions)
        .unwrap()
}

fn materialize_with_roster(
    catalog: &Arc<UniverseCatalog>,
    roster: &UniverseBattleRoster,
    contributions: &UniverseBattleContributionSet,
) -> UniverseBattleMaterialization {
    UniverseBattleMaterializer
        .compile(catalog, roster, contributions)
        .unwrap()
}

fn durable_spec(
    materialization: &UniverseBattleMaterialization,
    marker: u8,
    charged_resonance: bool,
) -> BattleSpec {
    durable_spec_with_enemy_speed(materialization, marker, charged_resonance, None)
}

fn durable_spec_with_two_enemies(
    materialization: &UniverseBattleMaterialization,
    marker: u8,
) -> BattleSpec {
    durable_spec_with_two_enemy_hp(
        materialization,
        marker,
        [
            Hp::new(2_000_000_000).unwrap(),
            Hp::new(2_000_000_000).unwrap(),
        ],
    )
}

fn durable_spec_with_two_enemy_hp(
    materialization: &UniverseBattleMaterialization,
    marker: u8,
    enemy_hp: [Hp; 2],
) -> BattleSpec {
    let original = materialization
        .overlay()
        .bindings()
        .iter()
        .flat_map(|binding| binding.preparation().variants())
        .map(|variant| variant.battle_spec())
        .find(|spec| {
            spec.participants()
                .iter()
                .filter(|participant| participant.side() == TeamSide::Enemy)
                .count()
                >= 2
        })
        .unwrap();
    let mut enemy_index = 0_usize;
    let participants = original
        .participants()
        .iter()
        .enumerate()
        .map(|(index, participant)| {
            if participant.side() != TeamSide::Enemy {
                return participant.clone();
            }
            let source = match participant.source() {
                ParticipantSource::EncounterEnemy(source) => source,
                _ => panic!("fixture enemy source"),
            };
            let base = participant.combatant();
            let hp = enemy_hp[enemy_index.min(enemy_hp.len() - 1)];
            enemy_index += 1;
            let mut combatant = ResolvedCombatantSpec::new(
                base.form(),
                base.level(),
                hp,
                base.speed(),
                ResolvedDefinitionBindings::new(
                    base.abilities().to_vec(),
                    base.rule_bundles().to_vec(),
                    base.modifiers().to_vec(),
                )
                .unwrap(),
                CombatantSpecDigest::new([marker.wrapping_add(u8::try_from(index).unwrap()); 32])
                    .unwrap(),
            )
            .unwrap()
            .with_base_attack_defense(base.base_attack(), base.base_defense())
            .with_energy(base.current_energy(), base.maximum_energy())
            .unwrap()
            .with_sources(base.sources().to_vec())
            .unwrap()
            .with_modifier_bindings(base.modifier_bindings().to_vec())
            .unwrap();
            if !base.toughness_layers().is_empty() {
                combatant = combatant
                    .with_toughness(
                        base.rank(),
                        base.weaknesses().to_vec(),
                        base.toughness_layers().to_vec(),
                    )
                    .unwrap();
            }
            ParticipantSpec::new(
                TeamSide::Enemy,
                participant.formation(),
                ParticipantSource::EncounterEnemy(source),
                combatant,
            )
            .with_wave(participant.wave())
            .unwrap()
        })
        .collect::<Vec<_>>();
    BattleSpec::new(
        original.rules_revision(),
        AssemblyDigest::new([marker.wrapping_add(3); 32]).unwrap(),
        original.encounter(),
        participants,
        original.resources(TeamSide::Player).clone(),
        original.resources(TeamSide::Enemy).clone(),
        original.concede_policy(),
    )
    .unwrap()
}

fn durable_spec_with_enemy_speed(
    materialization: &UniverseBattleMaterialization,
    marker: u8,
    charged_resonance: bool,
    enemy_speed: Option<Speed>,
) -> BattleSpec {
    durable_spec_with_enemy_profile(
        materialization,
        marker,
        charged_resonance,
        enemy_speed,
        Hp::new(2_000_000_000).unwrap(),
    )
}

fn durable_spec_with_enemy_hp(
    materialization: &UniverseBattleMaterialization,
    marker: u8,
    charged_resonance: bool,
    enemy_hp: Hp,
) -> BattleSpec {
    durable_spec_with_enemy_profile(materialization, marker, charged_resonance, None, enemy_hp)
}

fn durable_spec_with_enemy_profile(
    materialization: &UniverseBattleMaterialization,
    marker: u8,
    charged_resonance: bool,
    enemy_speed: Option<Speed>,
    enemy_hp: Hp,
) -> BattleSpec {
    let original = materialization.difficulty_specs()[0].battle_spec();
    let mut participants = original.participants().to_vec();
    let enemy_index = participants
        .iter()
        .position(|participant| participant.side() == TeamSide::Enemy)
        .unwrap();
    let enemy = &participants[enemy_index];
    let source = match enemy.source() {
        ParticipantSource::EncounterEnemy(source) => source,
        _ => panic!("fixture enemy source"),
    };
    let enemy_formation = enemy.formation();
    let enemy_wave = enemy.wave();
    let base = enemy.combatant().clone();
    let mut combatant = ResolvedCombatantSpec::new(
        base.form(),
        base.level(),
        enemy_hp,
        enemy_speed.unwrap_or(base.speed()),
        ResolvedDefinitionBindings::new(
            base.abilities().to_vec(),
            base.rule_bundles().to_vec(),
            base.modifiers().to_vec(),
        )
        .unwrap(),
        CombatantSpecDigest::new([marker; 32]).unwrap(),
    )
    .unwrap()
    .with_base_attack_defense(base.base_attack(), base.base_defense())
    .with_energy(base.current_energy(), base.maximum_energy())
    .unwrap()
    .with_sources(base.sources().to_vec())
    .unwrap()
    .with_modifier_bindings(base.modifier_bindings().to_vec())
    .unwrap();
    if !base.toughness_layers().is_empty() {
        combatant = combatant
            .with_toughness(
                base.rank(),
                base.weaknesses().to_vec(),
                base.toughness_layers().to_vec(),
            )
            .unwrap();
    }
    participants[enemy_index] = ParticipantSpec::new(
        TeamSide::Enemy,
        enemy_formation,
        ParticipantSource::EncounterEnemy(source),
        combatant,
    )
    .with_wave(enemy_wave)
    .unwrap();
    let player_resources = if charged_resonance {
        TeamResourceSpec::new(3, 5)
            .unwrap()
            .with_keyed(vec![
                KeyedTeamResourceSpec::new(
                    starclock_combat::SourceDefinitionId::new(RESONANCE_RESOURCE_RAW).unwrap(),
                    100,
                    100,
                    TeamResourceWavePolicy::Persist,
                )
                .unwrap()
                .with_stable_key("standard-universe.path-resonance-energy")
                .unwrap(),
            ])
            .unwrap()
    } else {
        original.resources(TeamSide::Player).clone()
    };
    BattleSpec::new(
        original.rules_revision(),
        AssemblyDigest::new([marker.wrapping_add(1); 32]).unwrap(),
        original.encounter(),
        participants,
        player_resources,
        original.resources(TeamSide::Enemy).clone(),
        original.concede_policy(),
    )
    .unwrap()
}

fn start(
    materialization: &UniverseBattleMaterialization,
    spec: BattleSpec,
    marker: u8,
) -> (Battle, starclock_combat::Resolution) {
    let mut battle = Battle::create(
        Arc::clone(materialization.combat_catalog()),
        spec,
        BattleSeed::new([marker; 32]),
    )
    .unwrap();
    let resolution = battle
        .apply(Command::StartBattle {
            decision: battle.decision().unwrap().id(),
        })
        .unwrap();
    (battle, resolution)
}

fn first_normal_action(battle: &mut Battle) -> starclock_combat::Resolution {
    if battle
        .decision()
        .is_some_and(|decision| decision.kind() == starclock_combat::DecisionKind::InterruptWindow)
    {
        battle
            .apply(Command::PassInterruptWindow {
                decision: battle.decision().unwrap().id(),
            })
            .unwrap();
    }
    let command = battle
        .decision()
        .unwrap()
        .legal_commands()
        .iter()
        .find(|command| matches!(command, Command::UseAbility { .. }))
        .unwrap()
        .clone();
    battle.apply(command).unwrap()
}

#[test]
fn real_blessing_and_curio_change_authoritative_combat_events() {
    let catalog = catalog();
    let without = contributions(&catalog, "universe.path.abundance", None, None, true);
    let with_blessing = contributions(
        &catalog,
        "universe.path.abundance",
        Some(("universe.blessing.612344", 2)),
        None,
        true,
    );
    let with_curio = contributions(
        &catalog,
        "universe.path.abundance",
        None,
        Some("universe.curio.8"),
        true,
    );
    assert_eq!(without.materialized_rule_binding_count(), 4);
    assert_eq!(with_blessing.materialized_rule_binding_count(), 4);
    assert_eq!(with_curio.materialized_rule_binding_count(), 5);

    let without = materialize(&catalog, &without);
    let with_blessing = materialize(&catalog, &with_blessing);
    let with_curio = materialize(&catalog, &with_curio);
    let first_curio_player = with_curio.difficulty_specs()[0]
        .battle_spec()
        .participants()
        .iter()
        .find(|participant| {
            participant.side() == TeamSide::Player && participant.formation().get() == 0
        })
        .unwrap();
    assert!(
        !first_curio_player.combatant().rule_bundles().is_empty(),
        "the mode-global Curio rule bundle must be attached to one canonical owner"
    );

    let (mut plain, plain_start) = start(&without, durable_spec(&without, 0x31, false), 0x41);
    let (mut blessed, blessed_start) = start(
        &with_blessing,
        durable_spec(&with_blessing, 0x31, false),
        0x41,
    );
    assert!(
        plain_start
            .events()
            .iter()
            .all(|event| !matches!(event.kind(), BattleEventKind::Damage(_)))
    );
    assert!(
        blessed_start
            .events()
            .iter()
            .all(|event| !matches!(event.kind(), BattleEventKind::Damage(_)))
    );
    let plain_action = first_normal_action(&mut plain);
    let blessed_action = first_normal_action(&mut blessed);
    let plain_damage = plain_action
        .events()
        .iter()
        .filter(|event| matches!(event.kind(), BattleEventKind::Damage(_)))
        .count();
    let blessed_damage = blessed_action
        .events()
        .iter()
        .filter(|event| matches!(event.kind(), BattleEventKind::Damage(_)))
        .count();
    assert_eq!(blessed_damage, plain_damage + 1);
    assert_ne!(blessed_action.state_hash(), plain_action.state_hash());

    let (curio_battle, curio_start) =
        start(&with_curio, durable_spec(&with_curio, 0x32, false), 0x42);
    assert!(
        curio_battle.view().rule_instances_by_id().count() >= 1,
        "the Curio rule must instantiate before BattleStarted dispatch"
    );
    let curio_damage = curio_start
        .events()
        .iter()
        .filter_map(|event| match event.kind() {
            BattleEventKind::Damage(data) => Some((data.class, data.applied.get())),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        curio_damage,
        vec![(
            starclock_combat::formula::model::DamageClass::Additional,
            600_000_000
        )]
    );
}

#[test]
fn ability_tree_projection_changes_battle_spec_and_active_modifier_state() {
    let catalog = catalog();
    let without_tree = contributions(&catalog, "universe.path.abundance", None, None, false);
    let with_tree = contributions(&catalog, "universe.path.abundance", None, None, true);
    assert!(without_tree.modifiers().is_empty());
    assert!(!with_tree.modifiers().is_empty());
    assert_ne!(without_tree.digest(), with_tree.digest());

    let without_tree = materialize(&catalog, &without_tree);
    let with_tree = materialize(&catalog, &with_tree);
    let without_spec = without_tree.difficulty_specs()[0].battle_spec();
    let with_spec = with_tree.difficulty_specs()[0].battle_spec();
    assert_ne!(without_spec.assembly_digest(), with_spec.assembly_digest());
    assert!(
        without_spec
            .participants()
            .iter()
            .filter(|participant| participant.side() == TeamSide::Player)
            .all(|participant| participant.combatant().modifier_bindings().is_empty())
    );
    assert!(
        with_spec
            .participants()
            .iter()
            .filter(|participant| participant.side() == TeamSide::Player)
            .all(|participant| !participant.combatant().modifier_bindings().is_empty())
    );

    let (plain, plain_start) = start(&without_tree, without_spec.clone(), 0x81);
    let (buffed, buffed_start) = start(&with_tree, with_spec.clone(), 0x81);
    assert_eq!(plain.view().modifier_instances_by_id().count(), 0);
    assert!(buffed.view().modifier_instances_by_id().count() > 0);
    assert_ne!(plain_start.state_hash(), buffed_start.state_hash());
}

#[test]
fn selected_asta_technique_executes_before_the_first_timeline_turn() {
    const ASTA_FORM: u32 = 8;
    const ASTA_TECHNIQUE: u32 = 20_012;
    let catalog = catalog();
    let roster = roster_for_forms(
        &catalog,
        [ASTA_FORM, 1, 2, 3],
        Some((ASTA_FORM, ASTA_TECHNIQUE)),
    );
    let contributions = contributions(&catalog, "universe.path.abundance", None, None, true);
    let option = ActivityOptionId::new(0x7540_0100).unwrap();
    let technique = UniverseBattleTechniqueDefinition::new(
        option,
        ParticipantId::new(1).unwrap(),
        AbilityId::new(ASTA_TECHNIQUE).unwrap(),
        1,
        TechniqueEngagement::Engage,
    )
    .unwrap();
    let materialization = UniverseBattleMaterializer
        .compile_with_technique(&catalog, &roster, &contributions, technique)
        .unwrap();
    let preparation = materialization.overlay().bindings()[0].preparation();
    assert_eq!(preparation.techniques().len(), 1);
    assert_eq!(preparation.variants().len(), 2);
    let selected = preparation
        .variants()
        .iter()
        .find(|variant| variant.techniques() == [option])
        .unwrap();
    let normal = preparation
        .variants()
        .iter()
        .find(|variant| variant.techniques().is_empty())
        .unwrap();
    assert_ne!(
        selected.battle_spec().assembly_digest(),
        normal.battle_spec().assembly_digest()
    );

    let mut battle = Battle::create(
        Arc::clone(materialization.combat_catalog()),
        selected.battle_spec().clone(),
        BattleSeed::new([0x91; 32]),
    )
    .unwrap();
    let resolution = battle
        .apply(Command::StartBattle {
            decision: battle.decision().unwrap().id(),
        })
        .unwrap();
    let technique_index = resolution
        .events()
        .iter()
        .position(|event| {
            matches!(
                event.kind(),
                BattleEventKind::Action(starclock_combat::ActionEventData::Declared {
                    ability,
                    origin: starclock_combat::ActionOrigin::Forced,
                    ..
                }) if ability.get() == ASTA_TECHNIQUE
            )
        })
        .expect("selected technique must become a forced combat action");
    let first_turn = resolution.events().iter().position(|event| {
        matches!(
            event.kind(),
            BattleEventKind::Turn(starclock_combat::TurnEventData::Started { .. })
        )
    });
    assert!(first_turn.is_none_or(|turn| technique_index < turn));
    assert!(resolution.events().iter().any(|event| {
        matches!(
            event.kind(),
            BattleEventKind::Damage(data)
                if data.element
                    == Some(starclock_combat::formula::model::CombatElement::Fire)
        )
    }));
}

#[test]
fn hunt_resonance_is_a_legal_shared_resource_transition() {
    let catalog = catalog();
    let contributions = contributions(&catalog, "universe.path.hunt", None, None, true);
    assert_eq!(contributions.materialized_rule_binding_count(), 4);
    let materialization = materialize(&catalog, &contributions);
    let (mut battle, _) = start(
        &materialization,
        durable_spec(&materialization, 0x51, true),
        0x61,
    );
    let command = battle
        .decision()
        .unwrap()
        .legal_commands()
        .iter()
        .find(|command| {
            matches!(
                command,
                Command::UseInterrupt { ability, .. } if ability.get() == RESONANCE_ABILITY_RAW
            )
        })
        .expect("charged Hunt Resonance is offered as a combat interrupt")
        .clone();
    let resolution = battle.apply(command).unwrap();
    assert!(resolution.events().iter().any(|event| {
        matches!(
            event.kind(),
            BattleEventKind::Resource(starclock_combat::ResourceEventData::TeamResource {
                resource,
                attempted: 100,
                effective: 100,
                before: 100,
                after: 0,
                ..
            }) if resource.get() == RESONANCE_RESOURCE_RAW
        )
    }));
    assert!(resolution.events().iter().any(|event| {
        matches!(
            event.kind(),
            BattleEventKind::Damage(data)
                if data.element == Some(starclock_combat::formula::model::CombatElement::Wind)
        )
    }));
}

#[test]
fn goal07_p2_m02_s01_executes_every_assigned_rule_and_operation_fixture() {
    let catalog = catalog();
    let contributions = contributions_many(
        &catalog,
        "universe.path.preservation",
        &[
            ("universe.blessing.612030", 2),
            ("universe.blessing.612032", 2),
            ("universe.blessing.612040", 2),
            ("universe.blessing.612041", 2),
            ("universe.blessing.612042", 2),
        ],
        None,
        false,
    );
    assert_eq!(contributions.materialized_rule_binding_count(), 5);
    let materialization = materialize(&catalog, &contributions);
    assert_eq!(
        materialization
            .combat_catalog()
            .trigger_ids(
                starclock_combat::rule::model::RuleEventKind::Damage,
                starclock_combat::rule::model::TriggerPhase::AfterEvent,
            )
            .filter(|(rule, _)| rule.get() == 1_879_048_249)
            .count(),
        2
    );
    let (mut battle, start_resolution) = start(
        &materialization,
        durable_spec_with_two_enemies(&materialization, 0xa1),
        0xa2,
    );
    assert!(
        start_resolution.fault().is_none(),
        "{:?} {:?}",
        start_resolution.fault(),
        start_resolution.events()
    );
    let applied = start_resolution
        .events()
        .iter()
        .filter_map(|event| match event.kind() {
            BattleEventKind::Shield(starclock_combat::ShieldEventData::Applied {
                amount, ..
            }) => Some(amount.get()),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        applied,
        vec![10_000; 4],
        "Macrosegregation shields every rule owner"
    );
    assert_eq!(battle.view().shields_by_id().count(), 4);
    assert_eq!(
        battle
            .view()
            .rule_instances_by_id()
            .filter(|rule| rule.rule().get() == 1_879_048_249)
            .count(),
        4
    );

    let resolution = first_normal_action(&mut battle);
    assert!(
        resolution.fault().is_none(),
        "{:?} {:?}",
        resolution.fault(),
        resolution.events()
    );
    let quake = resolution
        .events()
        .iter()
        .filter_map(|event| match event.kind() {
            BattleEventKind::Damage(data)
                if data.class == starclock_combat::formula::model::DamageClass::Additional
                    && data.element
                        == Some(starclock_combat::formula::model::CombatElement::Physical) =>
            {
                Some(data.applied.get())
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        quake,
        [18_538, 5_561],
        "enhanced Quake includes 120% current DEF before its 15% boost and 30% splash"
    );
    assert!(
        resolution.events().iter().any(|event| {
            matches!(
                event.kind(),
                BattleEventKind::Effect(starclock_combat::EffectEventData::Applied { .. })
            )
        }),
        "{:?} {:?}",
        resolution.events(),
        contributions
            .rules()
            .iter()
            .map(|rule| (
                rule.source_binding_key(),
                rule.source().definition().get(),
                rule.rule().get(),
            ))
            .collect::<Vec<_>>()
    );
    assert!(
        battle
            .view()
            .effects_by_id()
            .any(|effect| effect.category() == starclock_combat::EffectCategory::Dot),
        "enhanced Quake applies the production Bleed effect"
    );
    let mut cycle_reset = false;
    for _ in 0..5 {
        let resolution = first_normal_action(&mut battle);
        cycle_reset |= resolution.events().iter().any(|event| {
            matches!(
                event.kind(),
                BattleEventKind::Shield(starclock_combat::ShieldEventData::Removed { .. })
            )
        });
    }
    assert!(
        cycle_reset,
        "Macrosegregation removes and reissues its special shield every two owner turns"
    );

    let defense = contributions_many(
        &catalog,
        "universe.path.preservation",
        &[
            ("universe.blessing.612031", 2),
            ("universe.blessing.612032", 2),
            ("universe.blessing.612043", 1),
        ],
        None,
        false,
    );
    assert_eq!(defense.materialized_rule_binding_count(), 4);
    let defense = materialize(&catalog, &defense);
    let (defense_battle, defense_start) = start(
        &defense,
        durable_spec_with_enemy_speed(
            &defense,
            0xb1,
            false,
            Some(Speed::from_scaled(400_000_000).unwrap()),
        ),
        0xb2,
    );
    assert!(
        defense_start.fault().is_none(),
        "{:?}",
        defense_start.fault()
    );
    assert_eq!(
        defense_battle
            .view()
            .rule_instances_by_id()
            .filter(|rule| rule.rule().get() == 1_879_048_240)
            .count(),
        4,
        "Metastatic Field is attached to every player rule owner"
    );
}
