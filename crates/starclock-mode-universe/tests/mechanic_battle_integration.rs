use std::sync::{Arc, OnceLock};

use starclock_activity::{
    ActivityOptionId, BuildDigest, LoadoutLockScope, OpaqueParticipantBuild, ParticipantId,
    ParticipantLock, ParticipantLockEntry, ParticipantPolicy, ParticipantSourceKind,
    ParticipantUniquenessScope, TechniqueEngagement,
};
use starclock_combat::{
    AbilityId, Battle, BattleEventKind, BattleSeed, BattleSpec, BattleSpecDigest,
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
        if let Some((technique_form, technique_ability)) = technique
            && technique_form == form_raw
        {
            abilities.push(AbilityId::new(technique_ability).unwrap());
            abilities.sort_unstable();
        }
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
        .with_energy(Energy::ZERO, Energy::from_scaled(100_000_000).unwrap())
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
    let path_definition = catalog
        .paths()
        .iter()
        .find(|path| path.stable_key() == path_key)
        .unwrap();
    let required = required_blessing.map(|(key, level)| {
        (
            catalog
                .blessings()
                .iter()
                .find(|blessing| blessing.stable_key() == key)
                .unwrap()
                .id(),
            level,
        )
    });
    let mut owned = required.into_iter().collect::<Vec<_>>();
    for blessing in path_definition.blessings() {
        if owned.len() == 3 {
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
    let path = PathRuntimeCatalog::compile(catalog)
        .unwrap()
        .contributions(path_definition.id(), &blessings, &[])
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

fn durable_spec(
    materialization: &UniverseBattleMaterialization,
    marker: u8,
    charged_resonance: bool,
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
        Hp::new(2_000_000_000).unwrap(),
        base.speed(),
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
        BattleSpecDigest::new([marker.wrapping_add(1); 32]).unwrap(),
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
    assert_eq!(without.materialized_rule_binding_count(), 0);
    assert_eq!(with_blessing.materialized_rule_binding_count(), 1);
    assert_eq!(with_curio.materialized_rule_binding_count(), 1);

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
    assert_ne!(without_spec.digest(), with_spec.digest());
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
        selected.battle_spec().digest(),
        normal.battle_spec().digest()
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
    assert_eq!(contributions.materialized_rule_binding_count(), 1);
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
