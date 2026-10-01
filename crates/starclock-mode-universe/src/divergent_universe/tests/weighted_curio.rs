use crate::divergent_universe::{
    DivergentUniverseBaselineFixture, DivergentUniverseBaselineRunner,
    DivergentUniverseBaselineStep,
    contribution_snapshot::DivergentUniverseContributionSnapshotError,
    state::{CURIO_STATES_SLOT, WEIGHTED_CURIO_REFERENCES_SLOT},
    tests::{complete, entry, instance, reward_draws, slot_value},
    weighted_curio::{WeightedCurioError, WeightedCurioSlotLimit},
};
use starclock_activity::{
    ActivityMasterSeed, ActivityOperation, ActivityProgramDefinition, ActivityProgramId,
    ActivityStateHash,
};
use starclock_data::{
    divergent_universe_catalog::DivergentUniverseRunFamily,
    divergent_universe_curio_catalog::DivergentUniverseWeightedCurioId,
};

const FAMILIES: [DivergentUniverseRunFamily; 2] = [
    DivergentUniverseRunFamily::Ordinary,
    DivergentUniverseRunFamily::Cyclical,
];

#[test]
fn weighted_curio_all_current_selections_replace_unequip_and_reconstruct_both_families() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let fresh = DivergentUniverseBaselineFixture::production().unwrap();
    let runtime = fixture.factory().weighted_curio_runtime().unwrap();
    let rebuilt_runtime = fresh.factory().weighted_curio_runtime().unwrap();
    assert_eq!(runtime.candidates().len(), 17);
    for family in FAMILIES {
        let flow = fixture.flow(family).unwrap();
        let rebuilt_flow = fresh.flow(family).unwrap();
        let mut activity = flow
            .start(instance(2560), ActivityMasterSeed::from_u64(2560))
            .unwrap()
            .into_activity();
        let mut rebuilt = rebuilt_flow
            .start(instance(2560), ActivityMasterSeed::from_u64(2560))
            .unwrap()
            .into_activity();
        let ordinary = slot_value(&activity.player_view(), CURIO_STATES_SLOT).clone();
        let draws = reward_draws(&activity);
        for id in runtime.candidates() {
            let before = activity.state_hash();
            let events = runtime
                .replace_accepted_loadout(
                    &flow,
                    &mut activity,
                    before,
                    WeightedCurioSlotLimit::new(1).unwrap(),
                    std::slice::from_ref(id),
                )
                .unwrap();
            let rebuilt_hash = rebuilt.state_hash();
            let rebuilt_events = rebuilt_runtime
                .replace_accepted_loadout(
                    &rebuilt_flow,
                    &mut rebuilt,
                    rebuilt_hash,
                    WeightedCurioSlotLimit::new(1).unwrap(),
                    std::slice::from_ref(id),
                )
                .unwrap();
            assert!(!events.is_empty());
            assert_ne!(activity.state_hash(), before);
            assert_eq!(events, rebuilt_events);
            assert_eq!(runtime.equipped(&activity).unwrap(), vec![id.clone()]);
            assert_eq!(
                activity.canonical_state_bytes(),
                rebuilt.canonical_state_bytes()
            );
            assert_eq!(activity.debug_view(), rebuilt.debug_view());
            assert_eq!(
                slot_value(&activity.player_view(), CURIO_STATES_SLOT),
                &ordinary
            );
            assert_eq!(reward_draws(&activity), draws);
        }
        let hash = activity.state_hash();
        runtime
            .replace_accepted_loadout(
                &flow,
                &mut activity,
                hash,
                WeightedCurioSlotLimit::new(1).unwrap(),
                &[],
            )
            .unwrap();
        assert!(runtime.equipped(&activity).unwrap().is_empty());
        let before = activity.canonical_state_bytes();
        let hash = activity.state_hash();
        assert!(matches!(
            runtime.replace_accepted_loadout(
                &flow,
                &mut activity,
                hash,
                WeightedCurioSlotLimit::new(1).unwrap(),
                &[]
            ),
            Err(WeightedCurioError::UnchangedLoadout)
        ));
        assert_eq!(activity.canonical_state_bytes(), before);
    }
}

#[test]
fn weighted_curio_capacity_canonical_order_and_invalid_requests_are_atomic() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let runtime = fixture.factory().weighted_curio_runtime().unwrap();
    assert!(WeightedCurioSlotLimit::new(0).is_err());
    assert!(WeightedCurioSlotLimit::new(4).is_err());
    let flow = fixture.flow(FAMILIES[0]).unwrap();
    let mut activity = flow
        .start(instance(2561), ActivityMasterSeed::from_u64(2561))
        .unwrap()
        .into_activity();
    let selected = runtime.candidates()[..3]
        .iter()
        .rev()
        .cloned()
        .collect::<Vec<_>>();
    let hash = activity.state_hash();
    runtime
        .replace_accepted_loadout(
            &flow,
            &mut activity,
            hash,
            WeightedCurioSlotLimit::new(3).unwrap(),
            &selected,
        )
        .unwrap();
    assert_eq!(
        runtime.equipped(&activity).unwrap(),
        runtime.candidates()[..3]
    );
    let duplicate = [selected[0].clone(), selected[0].clone()];
    let unknown = vec![
        DivergentUniverseWeightedCurioId::new("divergent-universe.weighted-curio.unknown").unwrap(),
    ];
    for invalid in [
        &duplicate[..],
        &unknown[..],
        &runtime.candidates()[..4],
        &selected[..],
    ] {
        let before = activity.canonical_state_bytes();
        let debug = activity.debug_view();
        let hash = activity.state_hash();
        assert!(
            runtime
                .replace_accepted_loadout(
                    &flow,
                    &mut activity,
                    hash,
                    WeightedCurioSlotLimit::new(3).unwrap(),
                    invalid
                )
                .is_err()
        );
        assert_eq!(activity.canonical_state_bytes(), before);
        assert_eq!(activity.debug_view(), debug);
    }
    let before = activity.canonical_state_bytes();
    assert!(matches!(
        runtime.replace_accepted_loadout(
            &flow,
            &mut activity,
            ActivityStateHash::new([1; 32]).unwrap(),
            WeightedCurioSlotLimit::new(1).unwrap(),
            &unknown
        ),
        Err(WeightedCurioError::Command(_))
    ));
    assert_eq!(activity.canonical_state_bytes(), before);
    let hash = activity.state_hash();
    assert!(matches!(
        runtime.replace_accepted_loadout(
            &flow,
            &mut activity,
            hash,
            WeightedCurioSlotLimit::new(1).unwrap(),
            &selected
        ),
        Err(WeightedCurioError::CapacityExceeded)
    ));
    assert_eq!(activity.canonical_state_bytes(), before);
    let foreign = fixture.flow(FAMILIES[1]).unwrap();
    let hash = activity.state_hash();
    assert!(matches!(
        runtime.replace_accepted_loadout(
            &foreign,
            &mut activity,
            hash,
            WeightedCurioSlotLimit::new(1).unwrap(),
            &selected[..1]
        ),
        Err(WeightedCurioError::DefinitionMismatch)
    ));
    assert_eq!(activity.canonical_state_bytes(), before);
}

#[test]
fn weighted_curio_unlowered_and_dirty_loadouts_reject_contribution_without_mutation_or_rng() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let runtime = fixture.factory().weighted_curio_runtime().unwrap();
    let contribution = fixture.factory().contribution_snapshot_runtime().unwrap();
    for family in FAMILIES {
        let flow = fixture.flow(family).unwrap();
        let mut activity = flow
            .start(instance(2562), ActivityMasterSeed::from_u64(2562))
            .unwrap()
            .into_activity();
        for id in runtime.candidates() {
            let hash = activity.state_hash();
            runtime
                .replace_accepted_loadout(
                    &flow,
                    &mut activity,
                    hash,
                    WeightedCurioSlotLimit::new(1).unwrap(),
                    std::slice::from_ref(id),
                )
                .unwrap();
            let before = activity.canonical_state_bytes();
            let debug = activity.debug_view();
            if fixture
                .factory()
                .decision_catalog()
                .weighted_curio_splashes()
                .iter()
                .any(|definition| &definition.weighted_curio == id)
                || fixture
                    .factory()
                    .decision_catalog()
                    .weighted_curio_shields()
                    .iter()
                    .any(|definition| &definition.weighted_curio == id)
                || fixture
                    .factory()
                    .decision_catalog()
                    .weighted_curio_attack_debuffs()
                    .iter()
                    .any(|definition| &definition.weighted_curio == id)
            {
                let snapshot = contribution.snapshot(&flow, &activity).unwrap();
                assert_eq!(
                    snapshot.weighted_curios().equipped(),
                    std::slice::from_ref(id)
                );
            } else {
                assert!(matches!(contribution.snapshot(&flow, &activity),
                    Err(DivergentUniverseContributionSnapshotError::WeightedCurio(
                        WeightedCurioError::UnsupportedBattleEffect(ref rejected))) if rejected == id));
            }
            assert_eq!(activity.canonical_state_bytes(), before);
            assert_eq!(activity.debug_view(), debug);
        }
        for values in [
            vec![(1, 2)],
            vec![(999, 1)],
            vec![(1, 1), (2, 1), (3, 1), (4, 1)],
        ] {
            let program = ActivityProgramDefinition::new(
                ActivityProgramId::new(25620).unwrap(),
                vec![ActivityOperation::SetCounterMap {
                    slot: WEIGHTED_CURIO_REFERENCES_SLOT,
                    values: values.into_boxed_slice(),
                }],
            )
            .unwrap();
            activity
                .apply_boundary_program(activity.state_hash(), &program)
                .unwrap();
            let before = activity.canonical_state_bytes();
            assert!(matches!(
                contribution.snapshot(&flow, &activity),
                Err(DivergentUniverseContributionSnapshotError::WeightedCurio(
                    WeightedCurioError::InvalidState
                ))
            ));
            assert_eq!(activity.canonical_state_bytes(), before);
        }
    }
}

#[test]
fn weighted_curio_completed_activity_rejects_equipment_without_events() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let runtime = fixture.factory().weighted_curio_runtime().unwrap();
    let flow = fixture.factory().compile(entry("401", "3011")).unwrap();
    let mut activity = flow
        .start(instance(2564), ActivityMasterSeed::from_u64(2564))
        .unwrap()
        .into_activity();
    complete(&mut activity);
    let before = activity.canonical_state_bytes();
    let debug = activity.debug_view();
    let hash = activity.state_hash();
    assert!(matches!(
        runtime.replace_accepted_loadout(
            &flow,
            &mut activity,
            hash,
            WeightedCurioSlotLimit::new(1).unwrap(),
            &runtime.candidates()[..1]
        ),
        Err(WeightedCurioError::ActivityCompleted)
    ));
    assert_eq!(activity.canonical_state_bytes(), before);
    assert_eq!(activity.debug_view(), debug);
}

#[test]
fn weighted_curio_unequip_restores_real_battle_execution_not_effect_credit() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let runtime = fixture.factory().weighted_curio_runtime().unwrap();
    for family in FAMILIES {
        let flow = fixture.flow(family).unwrap();
        let mut activity = flow
            .start(instance(2563), ActivityMasterSeed::from_u64(2563))
            .unwrap()
            .into_activity();
        let hash = activity.state_hash();
        runtime
            .replace_accepted_loadout(
                &flow,
                &mut activity,
                hash,
                WeightedCurioSlotLimit::new(1).unwrap(),
                &runtime.candidates()[..1],
            )
            .unwrap();
        let hash = activity.state_hash();
        runtime
            .replace_accepted_loadout(
                &flow,
                &mut activity,
                hash,
                WeightedCurioSlotLimit::new(1).unwrap(),
                &[],
            )
            .unwrap();
        let policy = fixture.policy().unwrap();
        let mut executed = false;
        for _ in 0..16 {
            let step = DivergentUniverseBaselineRunner::default()
                .advance(
                    fixture.factory(),
                    &flow,
                    &mut activity,
                    fixture.core(),
                    &policy,
                )
                .unwrap();
            if matches!(step, DivergentUniverseBaselineStep::Battle { .. }) {
                executed = true;
                break;
            }
        }
        assert!(executed);
        assert_eq!(activity.player_view().completed_battle_count(), 1);
    }
}
