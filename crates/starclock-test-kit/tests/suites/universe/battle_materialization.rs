use std::{
    num::NonZeroUsize,
    sync::{Arc, OnceLock},
};

use starclock_activity::{
    ActivityInstanceId, ActivityMasterSeed, BuildDigest, LoadoutLockScope, OpaqueParticipantBuild,
    ParticipantId, ParticipantLock, ParticipantLockEntry, ParticipantPolicy, ParticipantSourceKind,
    ParticipantUniquenessScope,
};
use starclock_combat::{
    CombatantSpecDigest, Energy, Hp, ResolvedCombatantSpec, ResolvedDefinitionBindings, Speed,
    StatValue, TeamSide, UnitDefinitionId, UnitLevel, catalog::action::AbilityKind,
    formula::model::CombatElement,
};
use starclock_mode_universe::{
    ability_runtime::{
        AbilityBoundary, AbilityExecutionContext, AbilityProjectionScope, AbilityRuntimeCatalog,
    },
    baseline_runner::{
        NestedBattleExecutionError, StandardUniverseBaselinePolicy, StandardUniverseBaselineRunner,
    },
    battle_assembly::BattleAssemblyCache,
    battle_contribution::{UniverseBattleContributionCompiler, UniverseBattleContributionSet},
    battle_materialization::{
        EnemyDefinitionMatch, UNIVERSE_ENEMY_RUNTIME_STAT_POLICY, UniverseBattleMaterializer,
        UniverseBattleRoster, catalog_composition::UniverseBattleCatalogComposition,
    },
    blessing_runtime::BlessingRuntimeCatalog,
    catalog::UniverseCatalog,
    curio_runtime::CurioRuntimeCatalog,
    entry::{StandardUniverseEntry, StandardUniverseProfile},
    nested_battle_executor::{NestedBattleController, UniverseNestedBattleExecutor},
    path_runtime::PathRuntimeCatalog,
    run_runtime::RunRuntimeCatalog,
};

#[path = "battle_materialization/blaze_s04.rs"]
mod blaze_s04;
#[path = "battle_materialization/cocolia_s06.rs"]
mod cocolia_s06;
#[path = "battle_materialization/direwolf_s02.rs"]
mod direwolf_s02;
#[path = "battle_materialization/gepard_s07.rs"]
mod gepard_s07;
#[path = "battle_materialization/grizzly_s03.rs"]
mod grizzly_s03;
#[path = "battle_materialization/ice_out_of_space_s08.rs"]
mod ice_out_of_space_s08;
#[path = "battle_materialization/ordinary_enemies_s12.rs"]
mod ordinary_enemies_s12;
#[path = "battle_materialization/ordinary_enemies_s13.rs"]
mod ordinary_enemies_s13;
#[path = "battle_materialization/ordinary_enemies_s14.rs"]
mod ordinary_enemies_s14;
#[path = "battle_materialization/ordinary_enemies_s15.rs"]
mod ordinary_enemies_s15;
#[path = "battle_materialization/ordinary_enemies_s16.rs"]
mod ordinary_enemies_s16;
#[path = "battle_materialization/ordinary_enemies_s17.rs"]
mod ordinary_enemies_s17;
#[path = "battle_materialization/ordinary_enemies_s18.rs"]
mod ordinary_enemies_s18;
#[path = "battle_materialization/something_unto_death_s09.rs"]
mod something_unto_death_s09;
#[path = "battle_materialization/stellaron_hunter_kafka_s10.rs"]
mod stellaron_hunter_kafka_s10;
#[path = "battle_materialization/svarog_s11.rs"]
mod svarog_s11;
#[path = "battle_materialization/yanqing_s05.rs"]
mod yanqing_s05;

const CORE_BUNDLE: &[u8] = include_bytes!("../../../../../config/generated/config.sora");
const UNIVERSE_BUNDLE: &[u8] =
    include_bytes!("../../../../../config/universe-generated/config.sora");

fn catalog() -> Arc<UniverseCatalog> {
    static CATALOG: OnceLock<Arc<UniverseCatalog>> = OnceLock::new();
    Arc::clone(CATALOG.get_or_init(|| {
        let core = starclock_data::catalog::load(CORE_BUNDLE).expect("core catalog");
        UniverseCatalog::load(UNIVERSE_BUNDLE, core).expect("Universe catalog")
    }))
}

fn roster_and_lock(catalog: &UniverseCatalog) -> (UniverseBattleRoster, ParticipantLock) {
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
    for index in 0_u8..4 {
        let form = UnitDefinitionId::new(u32::from(index) + 1).unwrap();
        let unit = catalog
            .simulation_catalog()
            .combat_catalog()
            .unit(form)
            .expect("production character unit");
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
            .expect("production character has a Basic action");
        let combatant = ResolvedCombatantSpec::new(
            form,
            UnitLevel::new(80).unwrap(),
            Hp::new(100_000).unwrap(),
            Speed::from_scaled(200_000_000).unwrap(),
            ResolvedDefinitionBindings::new(vec![basic], Vec::new(), Vec::new()).unwrap(),
            CombatantSpecDigest::new([index + 1; 32]).unwrap(),
        )
        .unwrap()
        .with_base_attack_defense(
            StatValue::from_scaled(1_000_000_000).unwrap(),
            StatValue::from_scaled(1_000_000_000).unwrap(),
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
                    ParticipantSourceKind::FixedResolved,
                )
                .unwrap(),
            )
            .unwrap(),
        );
        combatants.push((participant, combatant));
    }
    let lock = ParticipantLock::seal(policy, lock_entries).unwrap();
    (UniverseBattleRoster::new(&lock, combatants).unwrap(), lock)
}

fn roster(catalog: &UniverseCatalog) -> UniverseBattleRoster {
    roster_and_lock(catalog).0
}

fn contributions(catalog: &Arc<UniverseCatalog>) -> UniverseBattleContributionSet {
    contributions_with_ability_limit(catalog, usize::MAX)
}

fn contributions_with_ability_limit(
    catalog: &Arc<UniverseCatalog>,
    ability_limit: usize,
) -> UniverseBattleContributionSet {
    let path_definition = &catalog.paths()[0];
    let selected_path = path_definition.id();
    let mut owned_blessings = path_definition
        .blessings()
        .iter()
        .take(14)
        .map(|id| (*id, 1))
        .collect::<Vec<_>>();
    owned_blessings.sort_unstable_by_key(|entry| entry.0);
    let blessings = BlessingRuntimeCatalog::compile(catalog)
        .unwrap()
        .contributions_from_owned(&owned_blessings)
        .unwrap();
    let formations = path_definition
        .formations()
        .iter()
        .map(|id| (*id, 1))
        .collect::<Vec<_>>();
    let path = PathRuntimeCatalog::compile(catalog)
        .unwrap()
        .contributions_with_formation_slots(selected_path, &blessings, &formations, 3)
        .unwrap();
    let curio_runtime = CurioRuntimeCatalog::compile(catalog).unwrap();
    let selected_curios = curio_runtime
        .definitions()
        .iter()
        .take(2)
        .collect::<Vec<_>>();
    let inventory = selected_curios
        .iter()
        .map(|definition| (definition.curio(), 1))
        .collect::<Vec<_>>();
    let states = selected_curios
        .iter()
        .map(|definition| (definition.curio(), definition.initial_state()))
        .collect::<Vec<_>>();
    let charges = selected_curios
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
    let selected_abilities = catalog
        .ability_tree_nodes()
        .iter()
        .take(ability_limit)
        .map(|node| node.id())
        .collect::<Vec<_>>();
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
                14,
                false,
            ),
        )
        .unwrap();
    UniverseBattleContributionCompiler::compile(Arc::clone(catalog))
        .unwrap()
        .compile_snapshot(&path, &blessings, &curios, &abilities, &projection)
        .unwrap()
}

#[test]
fn immutable_catalog_composition_and_bounded_exact_key_cache_are_separate() {
    let catalog = catalog();
    let roster = roster(&catalog);
    let composition = UniverseBattleCatalogComposition::compile(&catalog).unwrap();
    let contribution_sets =
        [0, 1, 2].map(|limit| contributions_with_ability_limit(&catalog, limit));
    let assemblies = contribution_sets
        .iter()
        .map(|contributions| {
            Arc::new(
                UniverseBattleMaterializer
                    .compile_from_composition(&catalog, &composition, &roster, contributions)
                    .unwrap(),
            )
        })
        .collect::<Vec<_>>();

    assert!(assemblies.windows(2).all(|pair| {
        pair[0].assembly_key() != pair[1].assembly_key()
            && pair[0].assembly_key().catalog_composition()
                == pair[1].assembly_key().catalog_composition()
    }));
    assert!(assemblies.iter().all(|assembly| {
        assembly.assembly_key().catalog_composition() == composition.digest()
            && assembly.assembly_key().participant_lock() == roster.participant_lock()
            && assembly.assembly_key().encounter() == composition.content().digest()
    }));

    let mut cache = BattleAssemblyCache::new(NonZeroUsize::new(2).unwrap());
    cache
        .insert(assemblies[0].assembly_key(), Arc::clone(&assemblies[0]))
        .unwrap();
    cache
        .insert(assemblies[1].assembly_key(), Arc::clone(&assemblies[1]))
        .unwrap();
    assert!(
        cache
            .get(assemblies[0].assembly_key())
            .is_some_and(|value| Arc::ptr_eq(&value, &assemblies[0]))
    );
    cache
        .insert(assemblies[2].assembly_key(), Arc::clone(&assemblies[2]))
        .unwrap();

    assert_eq!(cache.len(), 2);
    assert!(cache.get(assemblies[0].assembly_key()).is_none());
    assert!(cache.get(assemblies[1].assembly_key()).is_some());
    assert!(cache.get(assemblies[2].assembly_key()).is_some());
    assert_eq!(cache.metrics().hits(), 3);
    assert_eq!(cache.metrics().misses(), 1);
    assert_eq!(cache.metrics().insertions(), 3);
    assert_eq!(cache.metrics().evictions(), 1);

    let identities = assemblies
        .iter()
        .map(|assembly| {
            let spec = assembly.overlay().bindings()[0].preparation().variants()[0].battle_spec();
            (
                spec.combat_input_digest(),
                spec.assembly_digest(),
                assembly.digest(),
            )
        })
        .collect::<Vec<_>>();
    cache.clear();
    assert!(cache.is_empty());
    assert_eq!(
        identities,
        assemblies
            .iter()
            .map(|assembly| {
                let spec =
                    assembly.overlay().bindings()[0].preparation().variants()[0].battle_spec();
                (
                    spec.combat_input_digest(),
                    spec.assembly_digest(),
                    assembly.digest(),
                )
            })
            .collect::<Vec<_>>()
    );
}

#[test]
fn every_structured_member_and_difficulty_binding_is_an_executable_battle_spec() {
    let catalog = catalog();
    let roster = roster(&catalog);
    let contributions = contributions(&catalog);
    let materialized = UniverseBattleMaterializer
        .compile(&catalog, &roster, &contributions)
        .unwrap();
    let coverage = materialized.coverage();

    assert_eq!(materialized.overlay().bindings().len(), 199);
    assert_eq!(materialized.difficulty_specs().len(), 182);
    assert_eq!(materialized.enemies().len(), 86);
    assert_eq!(
        materialized
            .enemies()
            .iter()
            .filter(|enemy| enemy.definition_match() == EnemyDefinitionMatch::Exact)
            .count(),
        86
    );
    assert_eq!(
        materialized
            .enemies()
            .iter()
            .filter(|enemy| enemy.definition_match() == EnemyDefinitionMatch::ApproximateProxy)
            .count(),
        0
    );
    assert!(
        materialized
            .enemies()
            .iter()
            .filter(|enemy| enemy.definition_match() == EnemyDefinitionMatch::ApproximateProxy)
            .all(|enemy| enemy.source_enemy().is_none() && enemy.proxy_stable_key().is_some())
    );

    assert_eq!(coverage.member_count(), 173);
    assert_eq!(coverage.member_wave_count(), 173);
    assert_eq!(coverage.member_enemy_slot_count(), 538);
    assert_eq!(coverage.difficulty_binding_count(), 182);
    assert_eq!(coverage.enemy_variant_count(), 86);
    assert_eq!(coverage.exact_enemy_variant_count(), 86);
    assert_eq!(coverage.approximate_enemy_variant_count(), 0);
    assert_eq!(
        coverage.declared_rule_binding_count(),
        u16::try_from(contributions.rules().len()).unwrap()
    );
    assert_eq!(coverage.materialized_rule_binding_count(), 18);
    assert_eq!(
        coverage.runtime_stat_policy(),
        UNIVERSE_ENEMY_RUNTIME_STAT_POLICY
    );
    let occurrence = materialized
        .overlay()
        .bindings()
        .iter()
        .find(|binding| {
            binding
                .contract()
                .metrics()
                .iter()
                .any(|metric| metric.key() == "enemy.defeated.count")
        })
        .unwrap();
    assert!(occurrence.member().get() >= 10_000);
    let occurrence_spec = occurrence.preparation().variants()[0].battle_spec();
    let occurrence_enemies = occurrence_spec
        .participants()
        .iter()
        .filter(|participant| participant.side() == TeamSide::Enemy)
        .collect::<Vec<_>>();
    assert_eq!(occurrence_enemies.len(), 3);
    assert!(
        occurrence_enemies
            .iter()
            .all(|participant| participant.combatant().level().get() == 48)
    );
    assert_eq!(occurrence.contract().metrics().len(), 1);
    assert_eq!(
        occurrence.contract().metrics()[0].key(),
        "enemy.defeated.count"
    );
    let occurrence_binding = |key: &str| {
        let choice = catalog
            .occurrence_choices()
            .iter()
            .find(|choice| choice.stable_key() == key)
            .unwrap();
        let member = starclock_mode_universe::id::EncounterMemberId::new(
            10_000_u32.checked_add(choice.id().get()).unwrap(),
        )
        .unwrap();
        materialized
            .overlay()
            .bindings()
            .iter()
            .find(|binding| binding.member() == member)
            .unwrap()
    };
    let rock = occurrence_binding("universe.occurrence.33.variant.13401.choice.01");
    let rock_spec = rock.preparation().variants()[0].battle_spec();
    assert_eq!(
        rock_spec
            .participants()
            .iter()
            .filter(|participant| participant.side() == TeamSide::Enemy && participant.wave() == 1)
            .count(),
        3
    );
    assert_eq!(
        rock_spec
            .participants()
            .iter()
            .filter(|participant| participant.side() == TeamSide::Enemy && participant.wave() == 2)
            .count(),
        3
    );
    assert_eq!(
        rock.contract().metrics()[0].key(),
        "occurrence.blessing-reward.fixed.2"
    );
    let periodic = occurrence_binding("universe.occurrence.35.variant.13701.choice.01");
    assert_eq!(
        periodic.contract().metrics()[0].key(),
        "occurrence.blessing-reward.within-cycles.4.base.1.bonus.1"
    );
    assert_eq!(
        periodic.preparation().variants()[0]
            .battle_spec()
            .participants()
            .iter()
            .filter(|participant| participant.side() == TeamSide::Enemy)
            .count(),
        1
    );
    assert_eq!(
        materialized
            .overlay()
            .bindings()
            .iter()
            .filter(|binding| {
                binding.contract().metrics().iter().any(|metric| {
                    metric
                        .key()
                        .starts_with("occurrence.blessing-reward.fixed.")
                })
            })
            .count(),
        20
    );
    assert_eq!(
        materialized.digest(),
        [
            37, 199, 209, 171, 119, 104, 31, 13, 38, 80, 221, 146, 182, 43, 191, 182, 254, 70, 55,
            102, 66, 91, 170, 241, 61, 37, 195, 55, 112, 33, 229, 31,
        ]
    );
    assert_eq!(
        coverage.digest(),
        [
            68, 4, 34, 41, 165, 225, 28, 223, 55, 102, 61, 228, 157, 102, 161, 152, 228, 182, 91,
            42, 45, 243, 252, 194, 2, 76, 226, 177, 107, 135, 146, 28,
        ]
    );

    assert!(materialized.overlay().bindings().iter().all(|binding| {
        let spec = binding.preparation().variants()[0].battle_spec();
        spec.participants()
            .iter()
            .filter(|participant| participant.side() == starclock_combat::TeamSide::Player)
            .count()
            == 4
    }));
    assert!(materialized.difficulty_specs().iter().all(|binding| {
        binding
            .battle_spec()
            .participants()
            .iter()
            .filter(|participant| participant.side() == starclock_combat::TeamSide::Enemy)
            .count()
            == 1
    }));
    for modifier in contributions.modifiers() {
        assert!(
            materialized
                .combat_catalog()
                .modifier(modifier.definition().id)
                .is_some()
        );
    }
}

#[test]
fn abundant_ebon_deer_uses_reviewed_universe_stats_and_authored_phase() {
    let catalog = catalog();
    let materialized = UniverseBattleMaterializer
        .compile(&catalog, &roster(&catalog), &contributions(&catalog))
        .unwrap();
    let variant_key = "enemy.abundant-ebon-deer-complete.littleboss.variant.01";
    let enemy = materialized
        .enemies()
        .iter()
        .find(|enemy| enemy.stable_key() == variant_key)
        .expect("S01 enemy materialization");
    assert_eq!(enemy.definition_match(), EnemyDefinitionMatch::Exact);

    let expected = [
        (
            56,
            56_487,
            447_000_000,
            760_000_000,
            144_000_000,
            48_000,
            324_000,
        ),
        (
            72,
            88_816,
            561_000_000,
            920_000_000,
            158_400_000,
            176_000,
            388_000,
        ),
        (
            81,
            267_655,
            881_000_000,
            1_010_000_000,
            172_800_000,
            248_000,
            400_000,
        ),
        (
            90,
            514_058,
            876_000_000,
            1_100_000_000,
            190_080_000,
            320_000,
            400_000,
        ),
    ];
    for (level, hp, attack, defense, speed, effect_hit_rate, effect_resistance) in expected {
        let spec = materialized
            .difficulty_specs()
            .iter()
            .find(|spec| spec.enemy_variant_key() == variant_key && spec.level().get() == level)
            .expect("reviewed difficulty binding");
        let participant = spec
            .battle_spec()
            .participants()
            .iter()
            .find(|participant| participant.side() == TeamSide::Enemy)
            .expect("enemy participant");
        let combatant = participant.combatant();
        assert_eq!(combatant.maximum_hp().get(), hp);
        assert_eq!(combatant.base_attack().scaled(), attack);
        assert_eq!(combatant.base_defense().scaled(), defense);
        assert_eq!(combatant.speed().scaled(), speed);
        assert_eq!(combatant.base_effect_hit_rate().scaled(), effect_hit_rate);
        assert_eq!(
            combatant.base_effect_resistance().scaled(),
            effect_resistance
        );
        assert_eq!(
            combatant.weaknesses(),
            &[
                CombatElement::Fire,
                CombatElement::Ice,
                CombatElement::Quantum,
            ]
        );
        assert_eq!(combatant.toughness_layers().len(), 1);
        assert_eq!(combatant.toughness_layers()[0].maximum().get(), 420);

        let encounter = materialized
            .combat_catalog()
            .encounter(spec.battle_spec().encounter())
            .expect("materialized difficulty encounter");
        let initial_phase = encounter.waves()[0].slots()[0]
            .initial_phase()
            .expect("authored initial phase");
        assert_eq!(
            initial_phase,
            materialized
                .combat_catalog()
                .enemy(enemy.combat_enemy())
                .expect("authored enemy")
                .phases()[0]
                .id()
        );
    }

    let combat_catalog = materialized.combat_catalog();
    let deer = combat_catalog
        .enemy(enemy.combat_enemy())
        .expect("authored enemy");
    let phase_three_program = deer.phases()[2]
        .entry_program()
        .and_then(|program| combat_catalog.program(program))
        .expect("phase-three entry program");
    assert_eq!(
        phase_three_program
            .steps()
            .iter()
            .filter(|step| matches!(
                step,
                starclock_combat::rule::model::ProgramStep::Operation(
                    starclock_combat::rule::model::RuleOperationTemplate::GrantExtraTurn { .. }
                )
            ))
            .count(),
        2
    );
    let hardy = combat_catalog
        .effect(starclock_combat::EffectDefinitionId::new(980_504).unwrap())
        .and_then(|effect| effect.runtime_template())
        .expect("Hardy Leaf runtime");
    assert!(hardy.prevents_toughness_reduction());
    let outrage = combat_catalog
        .effect(starclock_combat::EffectDefinitionId::new(980_508).unwrap())
        .and_then(|effect| effect.runtime_template())
        .expect("Outrage runtime");
    assert_eq!(
        outrage.forced_normal_action(),
        Some(starclock_combat::ForcedNormalAction::BasicAttackRandomAlly)
    );
    let vigor = combat_catalog
        .effect(starclock_combat::EffectDefinitionId::new(980_506).unwrap())
        .expect("Vigor Overflow runtime");
    assert_eq!(vigor.runtime_template().unwrap().stack_limit(), 100);
    assert_eq!(vigor.modifiers().len(), 1);
}

#[test]
fn roster_mismatch_fails_before_any_catalog_or_spec_is_emitted() {
    let catalog = catalog();
    let roster = roster(&catalog);
    let mut combatants = roster
        .entries()
        .iter()
        .map(|entry| (entry.participant(), entry.combatant().clone()))
        .collect::<Vec<_>>();
    combatants.pop();
    let policy = ParticipantPolicy::new(
        1,
        1,
        4,
        ParticipantUniquenessScope::Activity,
        LoadoutLockScope::Activity,
    )
    .unwrap();
    let lock = ParticipantLock::seal(
        policy,
        roster
            .entries()
            .iter()
            .map(|entry| {
                ParticipantLockEntry::new(
                    entry.participant(),
                    0,
                    entry.formation().get(),
                    entry.combatant().form(),
                    OpaqueParticipantBuild::new(
                        entry.combatant().digest(),
                        BuildDigest::new([entry.formation().get() + 17; 32]).unwrap(),
                        ParticipantSourceKind::FixedResolved,
                    )
                    .unwrap(),
                )
                .unwrap()
            })
            .collect(),
    )
    .unwrap();
    assert!(UniverseBattleRoster::new(&lock, combatants).is_err());
}

#[test]
fn production_executor_runs_real_nested_battles_and_settles_activity_carry() {
    let catalog = catalog();
    let (roster, lock) = roster_and_lock(&catalog);
    let contributions = contributions(&catalog);
    let materialized = UniverseBattleMaterializer
        .compile(&catalog, &roster, &contributions)
        .unwrap();
    let world = &catalog.worlds()[0];
    let compiled = StandardUniverseProfile::new(Arc::clone(&catalog))
        .compile(
            StandardUniverseEntry::new(world.id(), world.difficulties()[0], lock, vec![])
                .with_encounter_overlay(materialized.overlay().clone()),
        )
        .unwrap();
    let mut activity = compiled
        .start_standard(
            ActivityInstanceId::new(5_033).unwrap(),
            ActivityMasterSeed::from_u64(0x5033),
        )
        .unwrap()
        .into_activity();
    let runner = StandardUniverseBaselineRunner::default();
    let mut failing = |_: &starclock_activity::ActivityBattleHandoff| {
        Err(NestedBattleExecutionError::StepBudgetExceeded)
    };
    assert!(
        runner
            .run_to_terminal(
                &mut activity,
                &StandardUniverseBaselinePolicy::default(),
                &mut failing,
            )
            .is_err()
    );
    let retry_hash = activity.view().state_hash();
    assert!(
        runner
            .advance(
                &mut activity,
                &StandardUniverseBaselinePolicy::default(),
                &mut failing,
            )
            .is_err()
    );
    assert_eq!(activity.view().state_hash(), retry_hash);
    let mut bounded = UniverseNestedBattleExecutor::new(Arc::clone(materialized.combat_catalog()))
        .with_command_budget(1)
        .unwrap();
    assert!(matches!(
        bounded.execute_pending_activity_battle(&mut activity),
        Err(
            starclock_mode_universe::nested_battle_executor::ActivityNestedBattleExecutionError::Execution(
                NestedBattleExecutionError::StepBudgetExceeded
            )
        )
    ));
    assert_eq!(activity.view().state_hash(), retry_hash);
    assert!(bounded.reports().is_empty());
    let mut executor = UniverseNestedBattleExecutor::new(Arc::clone(materialized.combat_catalog()));
    let report = runner
        .run_to_terminal(
            &mut activity,
            &StandardUniverseBaselinePolicy::default(),
            &mut executor,
        )
        .unwrap();

    assert_eq!(
        report.terminal(),
        starclock_activity::ActivityTerminalOutcome::Completed
    );
    assert_eq!(executor.reports().len(), 6);
    assert_eq!(
        executor
            .reports()
            .iter()
            .map(|battle| battle.trace().len())
            .sum::<usize>(),
        36
    );
    assert_eq!(
        report.final_state_hash().bytes(),
        [
            157, 209, 200, 23, 157, 85, 218, 44, 75, 18, 45, 215, 255, 80, 76, 167, 199, 210, 0,
            226, 144, 59, 49, 46, 58, 238, 9, 216, 62, 48, 223, 21,
        ]
    );
    assert_eq!(
        executor.reports()[0].event_digest().bytes(),
        [
            79, 140, 235, 174, 196, 151, 195, 216, 51, 23, 41, 88, 168, 56, 11, 71, 129, 104, 2,
            79, 200, 24, 165, 109, 64, 94, 50, 79, 127, 58, 57, 42,
        ]
    );
    assert!(executor.reports().iter().all(|battle| {
        battle.outcome() == starclock_activity::BattleOutcome::Won
            && !battle.trace().is_empty()
            && battle
                .trace()
                .iter()
                .any(|entry| entry.controller() == NestedBattleController::BaselinePlayer)
    }));
    assert!(
        activity
            .view()
            .participant_carry()
            .iter()
            .all(|carry| carry.current_hp() == carry.maximum_hp())
    );
}
