use std::sync::{Arc, OnceLock};

use starclock_activity::{
    ActivityDecisionKind, ActivityOperation, BuildDigest, LoadoutLockScope, OpaqueParticipantBuild,
    ParticipantId, ParticipantLock, ParticipantLockEntry, ParticipantPolicy, ParticipantSourceKind,
    ParticipantUniquenessScope,
};
use starclock_combat::{CombatantSpecDigest, UnitDefinitionId};
use starclock_mode_universe::{
    blessing_runtime::BlessingRuntimeCatalog,
    catalog::UniverseCatalog,
    entry::{StandardUniverseEntry, StandardUniverseProfile},
    path_runtime::{
        FORMATION_SELECTION_THRESHOLDS, PATH_RUNTIME_REVISION, PathRuntimeCatalog,
        PathRuntimeError, ResonanceEnergy,
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

fn participants() -> ParticipantLock {
    let policy = ParticipantPolicy::new(
        1,
        1,
        4,
        ParticipantUniquenessScope::Activity,
        LoadoutLockScope::Activity,
    )
    .expect("standard policy");
    let entries = (0_u8..4)
        .map(|index| {
            let byte = index + 1;
            ParticipantLockEntry::new(
                ParticipantId::new(u32::from(byte)).expect("participant"),
                0,
                index,
                UnitDefinitionId::new(20_001 + u32::from(index)).expect("unit"),
                OpaqueParticipantBuild::new(
                    CombatantSpecDigest::new([byte; 32]).expect("spec digest"),
                    BuildDigest::new([byte + 32; 32]).expect("build digest"),
                    "test-build-catalog-v1",
                    ParticipantSourceKind::CompiledBuild,
                )
                .expect("build"),
            )
            .expect("entry")
        })
        .collect();
    ParticipantLock::seal(policy, entries).expect("lock")
}

#[test]
fn all_paths_compile_resonance_thresholds_formations_and_exact_contributions() {
    let catalog = catalog();
    let blessing_runtime = BlessingRuntimeCatalog::compile(&catalog).expect("Blessing runtime");
    let path_runtime = PathRuntimeCatalog::compile(&catalog).expect("Path runtime");
    assert_eq!(path_runtime.len(), 9);
    assert_eq!(
        path_runtime.digest(),
        [
            149, 189, 227, 54, 222, 77, 30, 87, 252, 93, 95, 84, 54, 107, 171, 181, 169, 76, 72,
            212, 184, 212, 216, 15, 206, 9, 84, 227, 17, 179, 253, 216,
        ]
    );
    assert_eq!(FORMATION_SELECTION_THRESHOLDS, [6, 10, 14]);
    assert_eq!(PATH_RUNTIME_REVISION, "standard-universe-path-runtime-v1");

    for path in catalog.paths() {
        let owned = path
            .blessings()
            .iter()
            .map(|id| (*id, 1))
            .collect::<Vec<_>>();
        let none = blessing_runtime
            .contributions_from_owned(&[])
            .expect("empty contribution set");
        let locked = path_runtime
            .contributions(path.id(), &none, &[])
            .expect("selected Path passive");
        assert_eq!(locked.passive().path(), path.id());
        assert_eq!(locked.passive().buff_type(), path.buff_type());
        assert!(locked.resonance().is_none());
        assert_eq!(locked.next_formation_threshold(), Some(6));

        let three = blessing_runtime
            .contributions_from_owned(&owned[..3])
            .expect("three Blessings");
        let unlocked = path_runtime
            .contributions(path.id(), &three, &[])
            .expect("Resonance unlocked");
        let resonance = unlocked.resonance().expect("threshold three Resonance");
        assert_eq!(resonance.id(), path.resonance());
        assert_eq!(
            resonance.rule_key(),
            catalog.resonance(path.resonance()).unwrap().rule_key()
        );

        let all = blessing_runtime
            .contributions_from_owned(&owned)
            .expect("all Path Blessings");
        let formations = path
            .formations()
            .iter()
            .map(|id| (*id, 1))
            .collect::<Vec<_>>();
        let complete = path_runtime
            .contributions(path.id(), &all, &formations)
            .expect("three selected Formations");
        assert_eq!(complete.selected_path_blessings(), 18);
        assert_eq!(complete.formations().len(), 3);
        assert_eq!(complete.next_formation_threshold(), None);
        if path.id() == catalog.paths()[0].id() {
            assert_eq!(
                complete.digest(),
                [
                    60, 23, 225, 149, 43, 63, 107, 183, 102, 98, 33, 160, 51, 209, 218, 78, 9, 198,
                    117, 173, 36, 198, 13, 128, 132, 145, 75, 171, 141, 122, 95, 181,
                ]
            );
        }

        assert_eq!(
            path_runtime
                .contributions(path.id(), &three, &formations[..1])
                .expect_err("Formation is locked below six Blessings"),
            PathRuntimeError::InvalidFormationSelection
        );
    }
}

#[test]
fn resonance_energy_is_checked_clamped_available_and_consumed_without_float() {
    let catalog = catalog();
    let blessing_runtime = BlessingRuntimeCatalog::compile(&catalog).expect("Blessing runtime");
    let path_runtime = PathRuntimeCatalog::compile(&catalog).expect("Path runtime");
    let path = &catalog.paths()[0];
    let owned = path
        .blessings()
        .iter()
        .take(3)
        .map(|id| (*id, 1))
        .collect::<Vec<_>>();
    let blessings = blessing_runtime
        .contributions_from_owned(&owned)
        .expect("three Blessings");
    let contribution = path_runtime
        .contributions(path.id(), &blessings, &[])
        .expect("unlocked Resonance");
    let mut action = contribution
        .resonance()
        .expect("Resonance")
        .initial_action_state()
        .expect("energy state");
    assert_eq!(action.energy().raw_six_decimal(), 0);
    assert_eq!(action.maximum().raw_six_decimal(), 100_000_000);
    assert!(!action.can_activate());
    assert_eq!(
        action
            .gain(ResonanceEnergy::from_scaled(40, 0).unwrap())
            .unwrap()
            .raw_six_decimal(),
        40_000_000
    );
    assert_eq!(
        action
            .gain(ResonanceEnergy::from_scaled(70, 0).unwrap())
            .unwrap()
            .raw_six_decimal(),
        60_000_000
    );
    assert!(action.can_activate());
    assert_eq!(action.activate().unwrap().raw_six_decimal(), 100_000_000);
    assert_eq!(action.energy().raw_six_decimal(), 0);
    assert_eq!(
        action.activate().unwrap_err(),
        PathRuntimeError::ResonanceNotReady
    );
}

#[test]
fn every_reward_routes_through_a_generic_formation_gate() {
    let catalog = catalog();
    let world = &catalog.worlds()[0];
    let compiled = StandardUniverseProfile::new(Arc::clone(&catalog))
        .compile(StandardUniverseEntry::new(
            world.id(),
            world.difficulties()[0],
            participants(),
            vec![],
        ))
        .expect("compiled profile");
    assert_eq!(compiled.domain_hubs().len(), 579);
    for hub in compiled.domain_hubs() {
        let formation = compiled
            .runtime_definition()
            .programs()
            .iter()
            .find(|program| program.node() == hub.formation_node())
            .expect("Formation gate program");
        let [starclock_activity::ActivityOperation::Offer { kind, options }] =
            formation.program().operations()
        else {
            panic!("Formation gate must be one generic offer");
        };
        assert_eq!(*kind, ActivityDecisionKind::Choice);
        assert_eq!(options.len(), 28);
        assert_eq!(
            options
                .iter()
                .filter(|option| matches!(option.operations().first(), Some(ActivityOperation::AddInventory { inventory, .. }) if *inventory == compiled.formation_inventory()))
                .count(),
            27
        );

        let reward = compiled
            .runtime_definition()
            .programs()
            .iter()
            .find(|program| program.node() == hub.reward_node())
            .expect("reward program");
        let [ActivityOperation::Offer { options, .. }] = reward.program().operations() else {
            panic!("reward must be one offer");
        };
        assert!(options.iter().all(|option| option.operations().iter().any(
            |operation| matches!(operation, ActivityOperation::AddCounter { slot, .. } if *slot == compiled.path_blessing_count_slot())
        )));
    }
}
