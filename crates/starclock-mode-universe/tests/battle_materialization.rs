use std::sync::{Arc, OnceLock};

use starclock_activity::{
    ActivityInstanceId, ActivityMasterSeed, BuildDigest, LoadoutLockScope, OpaqueParticipantBuild,
    ParticipantId, ParticipantLock, ParticipantLockEntry, ParticipantPolicy, ParticipantSourceKind,
    ParticipantUniquenessScope,
};
use starclock_combat::{
    CombatantSpecDigest, Energy, Hp, ResolvedCombatantSpec, ResolvedDefinitionBindings, Speed,
    StatValue, UnitDefinitionId, UnitLevel, catalog::action::AbilityKind,
};
use starclock_mode_universe::{
    ability_runtime::{
        AbilityBoundary, AbilityExecutionContext, AbilityProjectionScope, AbilityRuntimeCatalog,
    },
    baseline_runner::{
        NestedBattleExecutionError, StandardUniverseBaselinePolicy, StandardUniverseBaselineRunner,
    },
    battle_contribution::{UniverseBattleContributionCompiler, UniverseBattleContributionSet},
    battle_materialization::{
        EnemyDefinitionMatch, UNIVERSE_ENEMY_RUNTIME_STAT_POLICY, UniverseBattleMaterializer,
        UniverseBattleRoster,
    },
    blessing_runtime::BlessingRuntimeCatalog,
    catalog::UniverseCatalog,
    curio_runtime::CurioRuntimeCatalog,
    entry::{StandardUniverseEntry, StandardUniverseProfile},
    nested_battle_executor::{NestedBattleController, UniverseNestedBattleExecutor},
    path_runtime::PathRuntimeCatalog,
    run_runtime::RunRuntimeCatalog,
    universe_replay_v2::{
        StandardUniverseReplayV2Error, encode_standard_universe_trace_v2, record_baseline_run_v2,
        standard_universe_component_set, standard_universe_header_v2,
        verify_standard_universe_replay_v2,
    },
};

const CORE_BUNDLE: &[u8] = include_bytes!("../../../config/generated/config.sora");
const UNIVERSE_BUNDLE: &[u8] = include_bytes!("../../../config/universe-generated/config.sora");

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
                    "battle-materialization-test-v1",
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
        .contributions(selected_path, &blessings, &formations)
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
fn every_structured_member_and_difficulty_binding_is_an_executable_battle_spec() {
    let catalog = catalog();
    let roster = roster(&catalog);
    let contributions = contributions(&catalog);
    let materialized = UniverseBattleMaterializer
        .compile(&catalog, &roster, &contributions)
        .unwrap();

    assert_eq!(materialized.overlay().bindings().len(), 173);
    assert_eq!(materialized.difficulty_specs().len(), 182);
    assert_eq!(materialized.enemies().len(), 86);
    assert_eq!(
        materialized
            .enemies()
            .iter()
            .filter(|enemy| enemy.definition_match() == EnemyDefinitionMatch::Exact)
            .count(),
        13
    );
    assert_eq!(
        materialized
            .enemies()
            .iter()
            .filter(|enemy| enemy.definition_match() == EnemyDefinitionMatch::ApproximateProxy)
            .count(),
        73
    );
    assert!(
        materialized
            .enemies()
            .iter()
            .filter(|enemy| enemy.definition_match() == EnemyDefinitionMatch::ApproximateProxy)
            .all(|enemy| enemy.source_enemy().is_none() && enemy.proxy_stable_key().is_some())
    );

    let coverage = materialized.coverage();
    assert_eq!(coverage.member_count(), 173);
    assert_eq!(coverage.member_wave_count(), 173);
    assert_eq!(coverage.member_enemy_slot_count(), 538);
    assert_eq!(coverage.difficulty_binding_count(), 182);
    assert_eq!(coverage.enemy_variant_count(), 86);
    assert_eq!(coverage.exact_enemy_variant_count(), 13);
    assert_eq!(coverage.approximate_enemy_variant_count(), 73);
    assert_eq!(
        coverage.declared_rule_binding_count(),
        u16::try_from(contributions.rules().len()).unwrap()
    );
    assert_eq!(coverage.materialized_rule_binding_count(), 0);
    assert_eq!(
        coverage.runtime_stat_policy(),
        UNIVERSE_ENEMY_RUNTIME_STAT_POLICY
    );
    assert_eq!(
        materialized.digest(),
        [
            175, 198, 160, 11, 42, 223, 13, 16, 106, 219, 1, 214, 78, 198, 27, 168, 177, 32, 44,
            95, 174, 139, 7, 165, 207, 81, 10, 146, 27, 158, 13, 196,
        ]
    );
    assert_eq!(
        coverage.digest(),
        [
            47, 160, 228, 103, 134, 128, 149, 68, 71, 143, 156, 34, 78, 164, 85, 57, 84, 15, 39,
            143, 245, 254, 211, 84, 138, 110, 92, 17, 154, 222, 217, 243,
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
                        "battle-materialization-test-v1",
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
    assert_eq!(executor.reports().len(), 3);
    assert_eq!(
        executor
            .reports()
            .iter()
            .map(|battle| battle.trace().len())
            .sum::<usize>(),
        15
    );
    assert_eq!(
        report.final_state_hash().bytes(),
        [
            140, 9, 218, 237, 192, 227, 89, 32, 245, 13, 11, 235, 214, 152, 65, 91, 44, 66, 226,
            129, 95, 73, 248, 188, 159, 192, 96, 39, 9, 56, 192, 36,
        ]
    );
    assert_eq!(
        executor.reports()[0].event_digest().bytes(),
        [
            48, 255, 232, 37, 197, 227, 80, 223, 81, 145, 152, 29, 225, 61, 56, 11, 103, 81, 7, 6,
            114, 173, 177, 197, 92, 110, 195, 190, 121, 244, 167, 81,
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

#[test]
fn component_replay_reexecutes_nested_commands_and_compares_event_payloads() {
    let catalog = catalog();
    let (roster, lock) = roster_and_lock(&catalog);
    let materialized = UniverseBattleMaterializer
        .compile(&catalog, &roster, &contributions(&catalog))
        .unwrap();
    let world = &catalog.worlds()[0];
    let compiled = StandardUniverseProfile::new(Arc::clone(&catalog))
        .compile(
            StandardUniverseEntry::new(world.id(), world.difficulties()[0], lock, vec![])
                .with_encounter_overlay(materialized.overlay().clone()),
        )
        .unwrap();
    let components = standard_universe_component_set(
        &catalog,
        &compiled,
        &materialized,
        "baseline-controller",
        StandardUniverseBaselineRunner::REVISION,
        [0x42; 32],
    )
    .unwrap();
    let compatibility = starclock_replay::format_v2::ReplayCompatibilityV2::new(
        "4.4",
        starclock_combat::NUMERIC_POLICY_REVISION,
        starclock_combat::rng::RNG_ALGORITHM_REVISION,
        starclock_activity::ACTIVITY_STATE_HASH_REVISION,
    )
    .unwrap();
    let instance = ActivityInstanceId::new(5_034).unwrap();
    let seed = ActivityMasterSeed::from_u64(0x5034);
    let mut activity = compiled
        .start_standard(instance, seed)
        .unwrap()
        .into_activity();
    let header = standard_universe_header_v2(
        compatibility.clone(),
        components.clone(),
        0x5034,
        &activity,
        "standard-universe-v1",
    )
    .unwrap();
    let mut executor = UniverseNestedBattleExecutor::new(Arc::clone(materialized.combat_catalog()));
    let recorded = record_baseline_run_v2(
        &mut activity,
        &StandardUniverseBaselinePolicy::default(),
        &mut executor,
    )
    .unwrap();
    let bytes = encode_standard_universe_trace_v2(&header, &recorded).unwrap();
    let fresh = compiled
        .start_standard(instance, seed)
        .unwrap()
        .into_activity();
    let verified = verify_standard_universe_replay_v2(
        &bytes,
        fresh,
        Arc::clone(materialized.combat_catalog()),
        &components,
        &compatibility,
        "standard-universe-v1",
    )
    .unwrap();
    assert_eq!(verified.battle_count() as usize, recorded.battles().len());
    assert_eq!(
        verified.battle_command_count() as usize,
        recorded
            .battles()
            .iter()
            .map(|battle| battle.trace().len())
            .sum::<usize>()
    );
    assert_eq!(
        verified.final_state_hash().bytes(),
        recorded.report().final_state_hash().bytes()
    );

    let mut event_corrupt = bytes.clone();
    let payload = v2_payload_offset(
        &event_corrupt,
        starclock_replay::record::RecordKind::ExpectedBattleState,
        0,
    );
    // state payload: version + hash + event count + first event byte length.
    event_corrupt[payload + 42] ^= 0x80;
    let fresh = compiled
        .start_standard(instance, seed)
        .unwrap()
        .into_activity();
    assert!(matches!(
        verify_standard_universe_replay_v2(
            &event_corrupt,
            fresh,
            Arc::clone(materialized.combat_catalog()),
            &components,
            &compatibility,
            "standard-universe-v1",
        ),
        Err(StandardUniverseReplayV2Error::BattleEventDivergence {
            battle_index: 0,
            command_index: 0,
            event_index: 0,
            ..
        })
    ));

    let divergent = replacement_controller_component_set(&components);
    let fresh = compiled
        .start_standard(instance, seed)
        .unwrap()
        .into_activity();
    assert!(matches!(
        verify_standard_universe_replay_v2(
            &bytes,
            fresh,
            Arc::clone(materialized.combat_catalog()),
            &divergent,
            &compatibility,
            "standard-universe-v1",
        ),
        Err(StandardUniverseReplayV2Error::ComponentDivergence(divergence))
            if divergence.expected.as_ref().unwrap().id() == "baseline-controller"
    ));
}

fn replacement_controller_component_set(
    source: &starclock_replay::component::ConfigurationComponentSet,
) -> starclock_replay::component::ConfigurationComponentSet {
    use starclock_replay::{
        component::{ConfigurationComponentIdentity, ConfigurationComponentKind},
        digest::ComponentDigest,
    };
    let mut values = source.components().to_vec();
    let controller = values
        .iter()
        .position(|value| value.kind() == ConfigurationComponentKind::Controller)
        .unwrap();
    values[controller] = ConfigurationComponentIdentity::new(
        ConfigurationComponentKind::Controller,
        "baseline-controller",
        StandardUniverseBaselineRunner::REVISION,
        ComponentDigest::new([0x43; 32]),
    )
    .unwrap();
    starclock_replay::component::ConfigurationComponentSet::new(values).unwrap()
}

fn v2_payload_offset(
    bytes: &[u8],
    kind: starclock_replay::record::RecordKind,
    ordinal: usize,
) -> usize {
    let decoded = starclock_replay::format_v2::decode_replay_v2(bytes).unwrap();
    let payload = decoded
        .records()
        .iter()
        .filter(|record| record.kind() == kind)
        .nth(ordinal)
        .unwrap()
        .payload();
    payload.as_ptr() as usize - bytes.as_ptr() as usize
}
