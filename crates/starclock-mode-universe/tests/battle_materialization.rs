use std::sync::{Arc, OnceLock};

use starclock_activity::{
    BuildDigest, LoadoutLockScope, OpaqueParticipantBuild, ParticipantId, ParticipantLock,
    ParticipantLockEntry, ParticipantPolicy, ParticipantSourceKind, ParticipantUniquenessScope,
};
use starclock_combat::{
    CombatantSpecDigest, Energy, Hp, ResolvedCombatantSpec, ResolvedDefinitionBindings, Speed,
    UnitDefinitionId, UnitLevel,
};
use starclock_mode_universe::{
    ability_runtime::{
        AbilityBoundary, AbilityExecutionContext, AbilityProjectionScope, AbilityRuntimeCatalog,
    },
    battle_contribution::{UniverseBattleContributionCompiler, UniverseBattleContributionSet},
    battle_materialization::{
        EnemyDefinitionMatch, UNIVERSE_ENEMY_RUNTIME_STAT_POLICY, UniverseBattleMaterializer,
        UniverseBattleRoster,
    },
    blessing_runtime::BlessingRuntimeCatalog,
    catalog::UniverseCatalog,
    curio_runtime::CurioRuntimeCatalog,
    path_runtime::PathRuntimeCatalog,
    run_runtime::RunRuntimeCatalog,
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

fn roster(catalog: &UniverseCatalog) -> UniverseBattleRoster {
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
        let combatant = ResolvedCombatantSpec::new(
            form,
            UnitLevel::new(80).unwrap(),
            Hp::new(100_000).unwrap(),
            Speed::from_scaled(200_000_000).unwrap(),
            ResolvedDefinitionBindings::new(unit.abilities().to_vec(), Vec::new(), Vec::new())
                .unwrap(),
            CombatantSpecDigest::new([index + 1; 32]).unwrap(),
        )
        .unwrap()
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
    UniverseBattleRoster::new(&lock, combatants).unwrap()
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
