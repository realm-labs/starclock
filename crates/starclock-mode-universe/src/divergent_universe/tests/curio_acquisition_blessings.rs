//! Acquisition rewards only; later wax effects remain independently pending.

use starclock_activity::{
    ActivityExpression, ActivityMasterSeed, ActivityOperation, ActivityOptionId,
    ActivityProgramDefinition, ActivityProgramId, ActivityTransactionEventKind, ActivityValue,
    GraphActivity,
};
use starclock_data::{
    divergent_universe_catalog::DivergentUniverseRunFamily,
    divergent_universe_curio_catalog::{
        DivergentUniverseCurioCategory, DivergentUniverseCurioStateId,
    },
    divergent_universe_decisions::CurioAcquisitionGrant,
};

use super::{entry, instance, reward_draws};
use crate::divergent_universe::{
    DivergentUniverseBaselineFixture, DivergentUniverseRuntimeFactory,
    blessing_runtime::DivergentUniverseBlessingRuntimeError,
    curio_runtime::DivergentUniverseCurioRuntimeError,
    economy::DivergentUniverseCurrencyKind,
    equation_progress::DivergentUniverseEquationProgressError,
    state::{BLESSINGS_SLOT, CURRENCIES_SLOT, EQUATION_PROGRESS_DIRTY_SLOT, ROOM_FINISHED_SLOT},
};

fn state(raw: &str) -> DivergentUniverseCurioStateId {
    DivergentUniverseCurioStateId::new(format!("divergent-universe.curio-state.{raw}")).unwrap()
}

pub(super) fn acquisition_draws(
    factory: &DivergentUniverseRuntimeFactory,
    states: &[DivergentUniverseCurioStateId],
) -> u64 {
    factory
        .decision_catalog()
        .curio_acquisitions()
        .iter()
        .filter(|effect| states.contains(&effect.state))
        .map(|effect| match &effect.grant {
            CurioAcquisitionGrant::PathBlessings { count, paths } => {
                u64::from(*count) * u64::try_from(paths.len()).unwrap()
            }
            CurioAcquisitionGrant::RarityBlessings { count, .. } => u64::from(*count),
            _ => 0,
        })
        .sum()
}

fn start(factory: &DivergentUniverseRuntimeFactory) -> GraphActivity {
    factory
        .compile(entry("401", "3011"))
        .unwrap()
        .start(instance(23651), ActivityMasterSeed::from_u64(23651))
        .unwrap()
        .into_activity()
}

#[test]
fn all_nine_waxes_grant_exact_paths_and_refresh_equations_once_per_acquisition() {
    let factory = DivergentUniverseRuntimeFactory::production().unwrap();
    let curios = factory.curio_runtime().unwrap();
    let blessings = factory.blessing_runtime().unwrap();
    for (raw, paths) in [
        ("9043", vec!["126"]),
        ("9044", vec!["124"]),
        ("9045", vec!["125"]),
        ("9046", vec!["121"]),
        ("9047", vec!["122"]),
        ("9048", vec!["127"]),
        ("9049", vec!["128"]),
        ("9147", vec!["129"]),
        (
            "9187",
            vec!["121", "122", "124", "125", "126", "127", "128", "129"],
        ),
    ] {
        let mut first = start(&factory);
        let mut fresh = start(&factory);
        let before_draws = reward_draws(&first);
        for activity in [&mut first, &mut fresh] {
            curios
                .acquire_accepted_state(activity, activity.state_hash(), &state(raw))
                .unwrap();
        }
        let owned = blessings.owned(&first).unwrap();
        let mut actual = owned
            .iter()
            .map(|held| {
                blessings
                    .blessings()
                    .iter()
                    .find(|definition| definition.id() == held.blessing())
                    .unwrap()
                    .path()
                    .as_str()
            })
            .collect::<Vec<_>>();
        actual.sort_unstable();
        assert_eq!(actual, paths, "{raw}");
        assert_eq!(
            reward_draws(&first) - before_draws,
            u64::try_from(paths.len()).unwrap()
        );
        assert_eq!(first.canonical_state_bytes(), fresh.canonical_state_bytes());
        assert_eq!(
            first
                .player_view()
                .slots()
                .iter()
                .find(|slot| slot.id() == EQUATION_PROGRESS_DIRTY_SLOT)
                .unwrap()
                .value(),
            &ActivityValue::Boolean(false)
        );
        let accepted = first.canonical_state_bytes();
        let draws = reward_draws(&first);
        let hash = first.state_hash();
        assert_eq!(
            curios.acquire_accepted_state(&mut first, hash, &state(raw)),
            Err(DivergentUniverseCurioRuntimeError::AlreadyOwned)
        );
        assert_eq!(first.canonical_state_bytes(), accepted);
        assert_eq!(reward_draws(&first), draws);
    }
}

#[test]
fn wax_batch_exhaustion_and_overlapping_paths_reject_before_rng() {
    let factory = DivergentUniverseRuntimeFactory::production().unwrap();
    let curios = factory.curio_runtime().unwrap();
    let blessings = factory.blessing_runtime().unwrap();
    let path_ids = blessings
        .blessings()
        .iter()
        .filter(|blessing| blessing.path().as_str() == "126")
        .map(|blessing| blessing.id().clone())
        .collect::<Vec<_>>();
    let mut activity = start(&factory);
    let hash = activity.state_hash();
    blessings
        .acquire_accepted_identities(&mut activity, hash, &path_ids[1..])
        .unwrap();
    let before = activity.canonical_state_bytes();
    let draws = reward_draws(&activity);
    let hash = activity.state_hash();
    assert_eq!(
        curios.acquire_accepted_states(&mut activity, hash, &[state("9043"), state("9187")]),
        Err(DivergentUniverseCurioRuntimeError::NoLegalCandidate)
    );
    assert_eq!(activity.canonical_state_bytes(), before);
    assert_eq!(reward_draws(&activity), draws);
    let hash = activity.state_hash();
    curios
        .acquire_accepted_state(&mut activity, hash, &state("9043"))
        .unwrap();
    assert!(
        blessings
            .owned(&activity)
            .unwrap()
            .iter()
            .any(|held| held.blessing() == &path_ids[0])
    );
    let before = activity.canonical_state_bytes();
    let draws = reward_draws(&activity);
    let hash = activity.state_hash();
    assert_eq!(
        curios.acquire_accepted_state(&mut activity, hash, &state("9187")),
        Err(DivergentUniverseCurioRuntimeError::NoLegalCandidate)
    );
    assert_eq!(activity.canonical_state_bytes(), before);
    assert_eq!(reward_draws(&activity), draws);
}

#[test]
fn wax_pending_offer_or_dirty_progress_rejects_without_blocking_fragment_curios() {
    let factory = DivergentUniverseRuntimeFactory::production().unwrap();
    let curios = factory.curio_runtime().unwrap();
    let blessings = factory.blessing_runtime().unwrap();
    for dirty in [false, true] {
        let mut activity = start(&factory);
        let hash = activity.state_hash();
        let expected = if dirty {
            let program = ActivityProgramDefinition::new(
                ActivityProgramId::new(23651).unwrap(),
                vec![ActivityOperation::SetSlot {
                    slot: EQUATION_PROGRESS_DIRTY_SLOT,
                    value: ActivityExpression::Literal(ActivityValue::Boolean(true)),
                }],
            )
            .unwrap();
            activity.apply_boundary_program(hash, &program).unwrap();
            DivergentUniverseBlessingRuntimeError::Progress(
                DivergentUniverseEquationProgressError::ProgressDirty,
            )
        } else {
            let group = blessings
                .groups()
                .iter()
                .find(|group| !group.candidates().is_empty())
                .unwrap();
            blessings
                .begin_group_offer(&mut activity, hash, group.id())
                .unwrap();
            DivergentUniverseBlessingRuntimeError::OfferAlreadyActive
        };
        let before = activity.canonical_state_bytes();
        let draws = reward_draws(&activity);
        let hash = activity.state_hash();
        assert_eq!(
            curios.acquire_accepted_state(&mut activity, hash, &state("9043")),
            Err(DivergentUniverseCurioRuntimeError::Blessing(expected))
        );
        assert_eq!(activity.canonical_state_bytes(), before);
        assert_eq!(reward_draws(&activity), draws);
        curios
            .acquire_accepted_state(&mut activity, hash, &state("9028"))
            .unwrap();
        assert_eq!(reward_draws(&activity), draws);
    }
}

#[test]
fn wax_batch_order_is_stable_and_late_fragment_failure_restores_rng_and_blessings() {
    let factory = DivergentUniverseRuntimeFactory::production().unwrap();
    let curios = factory.curio_runtime().unwrap();
    let mut first = start(&factory);
    let mut fresh = start(&factory);
    let ids = [state("9043"), state("9187"), state("9195")];
    let hash = first.state_hash();
    curios
        .acquire_accepted_states(&mut first, hash, &ids)
        .unwrap();
    let hash = fresh.state_hash();
    let mut reversed = ids.clone();
    reversed.reverse();
    curios
        .acquire_accepted_states(&mut fresh, hash, &reversed)
        .unwrap();
    assert_eq!(first.canonical_state_bytes(), fresh.canonical_state_bytes());
    assert_eq!(
        factory
            .blessing_runtime()
            .unwrap()
            .owned(&first)
            .unwrap()
            .len(),
        9
    );
    let mut activity = start(&factory);
    let key = factory
        .compile(entry("401", "3011"))
        .unwrap()
        .economy()
        .currency(DivergentUniverseCurrencyKind::CosmicFragment)
        .key();
    let program = ActivityProgramDefinition::new(
        ActivityProgramId::new(23651).unwrap(),
        vec![ActivityOperation::SetCounter {
            slot: CURRENCIES_SLOT,
            key,
            value: ActivityExpression::Literal(ActivityValue::BoundedInteger(i64::MAX)),
        }],
    )
    .unwrap();
    activity
        .apply_boundary_program(activity.state_hash(), &program)
        .unwrap();
    let before = activity.canonical_state_bytes();
    let draws = reward_draws(&activity);
    let hash = activity.state_hash();
    assert!(
        curios
            .acquire_accepted_states(&mut activity, hash, &ids)
            .is_err()
    );
    assert_eq!(activity.canonical_state_bytes(), before);
    assert_eq!(reward_draws(&activity), draws);
}

#[test]
fn production_occurrence_commits_wax_blessings_before_room_completion() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let curios = fixture.factory().curio_runtime().unwrap();
    let blessings = fixture.factory().blessing_runtime().unwrap();
    let selected = [state("9043"), state("9044")];
    let held = curios
        .curios()
        .iter()
        .filter(|curio| {
            matches!(
                curio.category(),
                DivergentUniverseCurioCategory::Common | DivergentUniverseCurioCategory::Rare
            )
        })
        .map(|curio| {
            curios
                .states()
                .iter()
                .filter(|copy| copy.curio() == Some(curio.id()))
                .min_by_key(|copy| copy.id())
                .unwrap()
                .id()
                .clone()
        })
        .filter(|id| !selected.contains(id))
        .collect::<Vec<_>>();
    for family in [
        DivergentUniverseRunFamily::Ordinary,
        DivergentUniverseRunFamily::Cyclical,
    ] {
        let flow = fixture.flow(family).unwrap();
        let mut activity = flow
            .start(instance(23651), ActivityMasterSeed::from_u64(23651))
            .unwrap()
            .into_activity();
        super::initial_equations::accept_initial(&flow, &mut activity);
        let hash = activity.state_hash();
        curios
            .acquire_accepted_states(&mut activity, hash, &held)
            .unwrap();
        let before = blessings.owned(&activity).unwrap().len();
        let draws = reward_draws(&activity);
        let hash = activity.state_hash();
        let decision = activity.player_view().decision().unwrap().id();
        let result = flow
            .choose_occurrence_option(
                fixture.factory(),
                &mut activity,
                hash,
                decision,
                ActivityOptionId::new(2).unwrap(),
            )
            .unwrap();
        assert_eq!(blessings.owned(&activity).unwrap().len(), before + 2);
        assert_eq!(reward_draws(&activity) - draws, 4);
        let last_blessing = result
            .events()
            .iter()
            .rposition(|event| {
                event.kind() == &ActivityTransactionEventKind::SlotChanged(BLESSINGS_SLOT)
            })
            .unwrap();
        let finished = result
            .events()
            .iter()
            .position(|event| {
                event.kind() == &ActivityTransactionEventKind::SlotChanged(ROOM_FINISHED_SLOT)
            })
            .unwrap();
        assert!(last_blessing < finished);
    }
}
