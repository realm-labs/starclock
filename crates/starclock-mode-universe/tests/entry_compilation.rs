use std::sync::{Arc, OnceLock};

use starclock_activity::{
    ActivityValue, BuildDigest, LoadoutLockScope, OpaqueParticipantBuild, ParticipantId,
    ParticipantLock, ParticipantLockEntry, ParticipantPolicy, ParticipantSourceKind,
    ParticipantUniquenessScope, SlotValueKind,
};
use starclock_combat::{CombatantSpecDigest, UnitDefinitionId};
use starclock_mode_universe::{
    catalog::UniverseCatalog,
    entry::{
        STANDARD_UNIVERSE_ENTRY_REVISION, StandardUniverseCompileError, StandardUniverseEntry,
        StandardUniverseProfile,
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

fn participants(seed: u8, policy: ParticipantPolicy) -> ParticipantLock {
    let entries = (0_u8..4)
        .map(|index| {
            let byte = seed.checked_add(index).expect("test digest byte");
            let build = OpaqueParticipantBuild::new(
                CombatantSpecDigest::new([byte; 32]).expect("non-zero digest"),
                BuildDigest::new([byte.wrapping_add(32); 32]).expect("non-zero digest"),
                "test-build-catalog-v1",
                ParticipantSourceKind::CompiledBuild,
            )
            .expect("opaque build");
            ParticipantLockEntry::new(
                ParticipantId::new(u32::from(index) + 1).expect("participant ID"),
                0,
                index,
                UnitDefinitionId::new(20_001 + u32::from(index)).expect("unit ID"),
                build,
            )
            .expect("participant")
        })
        .collect();
    ParticipantLock::seal(policy, entries).expect("participant lock")
}

fn standard_policy() -> ParticipantPolicy {
    ParticipantPolicy::new(
        1,
        1,
        4,
        ParticipantUniquenessScope::Activity,
        LoadoutLockScope::Activity,
    )
    .expect("standard policy")
}

#[test]
fn every_world_and_difficulty_compiles_the_same_generic_entry_contract() {
    let catalog = catalog();
    let profile = StandardUniverseProfile::new(Arc::clone(&catalog));
    let lock = participants(1, standard_policy());
    let mut compiled = 0;

    for world in catalog.worlds() {
        for difficulty in world.difficulties() {
            let activity = profile
                .compile(StandardUniverseEntry::new(
                    world.id(),
                    *difficulty,
                    lock.clone(),
                    vec![],
                ))
                .expect("valid World/difficulty entry");
            assert_eq!(activity.world(), world.id());
            assert_eq!(activity.difficulty(), *difficulty);
            assert_eq!(activity.path_options().len(), 9);
            assert_eq!(activity.state_definition().slots().len(), 14);
            assert_eq!(activity.state_definition().inventories().len(), 3);
            assert_eq!(activity.blessing_runtime().definitions().len(), 162);
            assert_eq!(activity.path_runtime().len(), 9);
            assert_eq!(activity.preservation_runtime().content_count(), 59);
            assert_eq!(activity.remembrance_runtime().content_count(), 59);
            assert_eq!(activity.nihility_runtime().content_count(), 59);
            assert_eq!(activity.abundance_runtime().content_count(), 59);
            assert_eq!(activity.hunt_runtime().content_count(), 59);
            assert_eq!(activity.destruction_runtime().content_count(), 59);
            assert_eq!(activity.elation_runtime().content_count(), 59);
            assert_eq!(activity.curio_runtime().definitions().len(), 61);
            assert_eq!(activity.run_runtime().occurrence_choices().len(), 321);
            assert_eq!(activity.run_runtime().services().len(), 94);
            assert_eq!(
                activity
                    .state_definition()
                    .slots()
                    .iter()
                    .find(|slot| slot.id() == activity.selected_path_slot())
                    .expect("Path slot")
                    .kind(),
                SlotValueKind::OptionalId
            );
            compiled += 1;
        }
    }

    assert_eq!(compiled, 33);
    assert_eq!(
        STANDARD_UNIVERSE_ENTRY_REVISION,
        "standard-universe-entry-v1"
    );
}

#[test]
fn ability_tree_input_is_canonical_and_prerequisite_closed() {
    let catalog = catalog();
    let profile = StandardUniverseProfile::new(Arc::clone(&catalog));
    let world = &catalog.worlds()[0];
    let difficulty = world.difficulties()[0];
    let lock = participants(16, standard_policy());
    let all = catalog
        .ability_tree_nodes()
        .iter()
        .map(|node| node.id())
        .collect::<Vec<_>>();
    let mut reversed = all.clone();
    reversed.reverse();

    let forward = profile
        .compile(StandardUniverseEntry::new(
            world.id(),
            difficulty,
            lock.clone(),
            all,
        ))
        .expect("full prerequisite-closed tree");
    let reverse = profile
        .compile(StandardUniverseEntry::new(
            world.id(),
            difficulty,
            lock.clone(),
            reversed,
        ))
        .expect("input order is not semantic");
    assert_eq!(forward.ability_tree(), reverse.ability_tree());
    assert_eq!(forward.identity(), reverse.identity());

    let child = catalog
        .ability_tree_nodes()
        .iter()
        .find(|node| !node.prerequisites().is_empty())
        .expect("catalog has a prerequisite edge");
    assert_eq!(
        profile
            .compile(StandardUniverseEntry::new(
                world.id(),
                difficulty,
                lock,
                vec![child.id()],
            ))
            .expect_err("missing prerequisite must fail"),
        StandardUniverseCompileError::MissingAbilityTreePrerequisite {
            node: child.id(),
            prerequisite: child.prerequisites()[0],
        }
    );
}

#[test]
fn cross_world_difficulty_and_nonstandard_roster_policy_fail_closed() {
    let catalog = catalog();
    let profile = StandardUniverseProfile::new(Arc::clone(&catalog));
    let first = &catalog.worlds()[0];
    let other = catalog
        .worlds()
        .iter()
        .find(|world| world.id() != first.id())
        .expect("second World");
    let other_difficulty = other.difficulties()[0];
    assert_eq!(
        profile
            .compile(StandardUniverseEntry::new(
                first.id(),
                other_difficulty,
                participants(32, standard_policy()),
                vec![],
            ))
            .expect_err("cross-World difficulty must fail"),
        StandardUniverseCompileError::DifficultyWorldMismatch {
            world: first.id(),
            difficulty: other_difficulty,
        }
    );

    let team_scoped = ParticipantPolicy::new(
        1,
        1,
        4,
        ParticipantUniquenessScope::Team,
        LoadoutLockScope::Activity,
    )
    .expect("test policy");
    assert_eq!(
        profile
            .compile(StandardUniverseEntry::new(
                first.id(),
                first.difficulties()[0],
                participants(48, team_scoped),
                vec![],
            ))
            .expect_err("nonstandard roster policy must fail"),
        StandardUniverseCompileError::ParticipantPolicyMismatch
    );
}

#[test]
fn world_difficulty_roster_and_ability_input_are_definition_identity() {
    let catalog = catalog();
    let profile = StandardUniverseProfile::new(Arc::clone(&catalog));
    let world = &catalog.worlds()[0];
    let difficulty = world.difficulties()[0];
    let base = profile
        .compile(StandardUniverseEntry::new(
            world.id(),
            difficulty,
            participants(64, standard_policy()),
            vec![],
        ))
        .expect("base entry");
    let different_roster = profile
        .compile(StandardUniverseEntry::new(
            world.id(),
            difficulty,
            participants(80, standard_policy()),
            vec![],
        ))
        .expect("different roster");
    let root = catalog
        .ability_tree_nodes()
        .iter()
        .find(|node| node.prerequisites().is_empty())
        .expect("root Ability Tree node");
    let different_tree = profile
        .compile(StandardUniverseEntry::new(
            world.id(),
            difficulty,
            participants(64, standard_policy()),
            vec![root.id()],
        ))
        .expect("one root selected");

    assert_ne!(base.identity(), different_roster.identity());
    assert_ne!(base.identity(), different_tree.identity());
    assert_eq!(
        base.identity().definition_digest().bytes(),
        [
            100, 220, 149, 202, 61, 82, 191, 151, 40, 122, 245, 159, 191, 117, 28, 203, 225, 90,
            128, 203, 129, 218, 211, 241, 56, 214, 208, 197, 75, 250, 255, 167,
        ]
    );
    assert_eq!(
        base.identity().config_digest().bytes(),
        catalog.identity().configuration_digest().bytes()
    );
}

#[test]
fn ability_tree_run_start_currency_is_materialized_into_activity_state() {
    let catalog = catalog();
    let world = &catalog.worlds()[0];
    let all_nodes = catalog
        .ability_tree_nodes()
        .iter()
        .map(|node| node.id())
        .collect::<Vec<_>>();
    let compiled = StandardUniverseProfile::new(Arc::clone(&catalog))
        .compile(StandardUniverseEntry::new(
            world.id(),
            world.difficulties()[0],
            participants(96, standard_policy()),
            all_nodes,
        ))
        .expect("complete Ability Tree entry");
    let fragments = compiled
        .state_definition()
        .slots()
        .iter()
        .find(|slot| slot.id() == compiled.cosmic_fragments_slot())
        .expect("Cosmic Fragment slot");
    assert_eq!(fragments.initial(), &ActivityValue::BoundedInteger(50));
}
