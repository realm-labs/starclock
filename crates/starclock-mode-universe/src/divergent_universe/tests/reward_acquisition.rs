//! Production-catalog inventory fixtures, not event/reward-pool gameplay claims.

use std::slice::from_ref;

use starclock_activity::{
    ActivityMasterSeed, ActivityStateHash, ActivityTransactionEventKind, GraphActivity,
    GraphActivityCommandError,
};
use starclock_data::divergent_universe_blessing_catalog::DivergentUniverseBlessingId;
use starclock_data::divergent_universe_curio_catalog::DivergentUniverseCurioStateId;
use starclock_data::divergent_universe_equation_catalog::DivergentUniverseEquationId;

use crate::divergent_universe::{
    DivergentUniverseRuntimeFactory,
    blessing_runtime::DivergentUniverseBlessingRuntimeError as BlessingError,
    curio_runtime::{
        DivergentUniverseCurioLifecycleState, DivergentUniverseCurioRuntimeError as CurioError,
    },
    equation_progress::DivergentUniverseEquationExpansionState,
    state::{BLESSINGS_SLOT, CURIO_ACTIVATIONS_SLOT, CURIO_CHARGES_SLOT, CURIO_STATES_SLOT},
};

use super::{acquire_target_equation, contribution_keys, entry, instance, reward_draws};

fn start(factory: &DivergentUniverseRuntimeFactory) -> GraphActivity {
    factory
        .compile(entry("401", "3011"))
        .expect("production Ordinary flow")
        .start(instance(23_500), ActivityMasterSeed::from_u64(23_500))
        .expect("start flow")
        .into_activity()
}

#[test]
fn curio_reward_batch_commits_both_inventories_and_reconstructs() {
    let factory = DivergentUniverseRuntimeFactory::production().expect("production bundle");
    let runtime = factory.curio_runtime().expect("Curio runtime");
    let selected = runtime
        .curios()
        .iter()
        .take(2)
        .map(|curio| curio.states()[0].clone())
        .collect::<Vec<_>>();
    let mut first = start(&factory);
    let mut replay = start(&factory);
    let before_draws = reward_draws(&first);
    let hash = first.state_hash();
    let result = runtime
        .acquire_accepted_states(&mut first, hash, &selected)
        .expect("two accepted Curios");
    let hash = replay.state_hash();
    let reconstructed = runtime
        .acquire_accepted_states(&mut replay, hash, &selected)
        .expect("fresh reconstruction");
    assert_eq!(result, reconstructed);
    assert_eq!(
        first.canonical_state_bytes(),
        replay.canonical_state_bytes()
    );
    assert_eq!(reward_draws(&first), before_draws);
    assert_eq!(result.owned().len(), 2);
    for owned in result.owned() {
        let definition = runtime
            .states()
            .iter()
            .find(|row| row.id() == owned.state())
            .unwrap();
        assert_eq!(
            owned.lifecycle(),
            DivergentUniverseCurioLifecycleState::Active
        );
        assert_eq!(owned.charges(), definition.declared_charges().unwrap_or(0));
        assert_eq!(owned.activations(), 0);
    }
    let changed_slots = result
        .events()
        .iter()
        .filter_map(|event| match event.kind() {
            ActivityTransactionEventKind::SlotChanged(slot) => Some(*slot),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        changed_slots,
        [
            CURIO_STATES_SLOT,
            CURIO_CHARGES_SLOT,
            CURIO_ACTIVATIONS_SLOT
        ]
    );

    let mut reversed = start(&factory);
    let hash = reversed.state_hash();
    let reversed_selection = selected.into_iter().rev().collect::<Vec<_>>();
    runtime
        .acquire_accepted_states(&mut reversed, hash, &reversed_selection)
        .unwrap();
    assert_eq!(
        first.canonical_state_bytes(),
        reversed.canonical_state_bytes()
    );
}

#[test]
fn curio_reward_batch_rejects_invalid_second_item_without_partial_reward() {
    let factory = DivergentUniverseRuntimeFactory::production().unwrap();
    let runtime = factory.curio_runtime().unwrap();
    let good = runtime.curios()[0].states()[0].clone();
    let unbound = runtime
        .states()
        .iter()
        .find(|row| row.curio().is_none())
        .unwrap()
        .id()
        .clone();
    let unknown =
        DivergentUniverseCurioStateId::new("divergent-universe.curio-state.99999999").unwrap();
    let multi = runtime
        .curios()
        .iter()
        .find(|row| row.states().len() > 1)
        .unwrap();
    let invalid = [
        (Vec::new(), CurioError::InvalidAcquisitionCount),
        (
            vec![good.clone(); runtime.curios().len() + 1],
            CurioError::InvalidAcquisitionCount,
        ),
        (vec![good.clone(), good.clone()], CurioError::AlreadyOwned),
        (vec![good.clone(), unknown], CurioError::UnknownState),
        (
            vec![good.clone(), unbound],
            CurioError::MissingCurioIdentity,
        ),
        (
            vec![multi.states()[0].clone(), multi.states()[1].clone()],
            CurioError::AlreadyOwned,
        ),
    ];
    let mut activity = start(&factory);
    for (selected, error) in invalid {
        let before = activity.canonical_state_bytes();
        let hash = activity.state_hash();
        let draws = reward_draws(&activity);
        assert_eq!(
            runtime.acquire_accepted_states(&mut activity, hash, &selected),
            Err(error)
        );
        assert_eq!(activity.canonical_state_bytes(), before);
        assert_eq!(reward_draws(&activity), draws);
    }
    let hash = activity.state_hash();
    runtime
        .acquire_accepted_state(&mut activity, hash, &good)
        .unwrap();
    let before = activity.canonical_state_bytes();
    let other = runtime.curios()[1].states()[0].clone();
    let hash = activity.state_hash();
    assert_eq!(
        runtime.acquire_accepted_states(&mut activity, hash, &[other, good.clone()]),
        Err(CurioError::AlreadyOwned)
    );
    let stale = ActivityStateHash::new([0x91; 32]).unwrap();
    assert_eq!(
        runtime.acquire_accepted_states(&mut activity, stale, &[good]),
        Err(CurioError::Activity(
            GraphActivityCommandError::StaleStateHash
        ))
    );
    assert_eq!(activity.canonical_state_bytes(), before);
}

#[test]
fn blessing_reward_batch_updates_equation_progress_in_same_commit() {
    let factory = DivergentUniverseRuntimeFactory::production().unwrap();
    let runtime = factory.blessing_runtime().unwrap();
    let progress = factory.equation_progress_runtime().unwrap();
    let offers = factory.equation_offer_runtime().unwrap();
    let equation = DivergentUniverseEquationId::new("divergent-universe.equation.3102001").unwrap();
    let recipe = progress
        .recipes()
        .iter()
        .find(|row| row.equation() == &equation)
        .unwrap();
    let mut keys = contribution_keys(&factory, &equation, recipe.main_path())
        .into_iter()
        .take(usize::from(recipe.main_required()))
        .collect::<Vec<_>>();
    keys.extend(
        contribution_keys(&factory, &equation, recipe.sub_path().unwrap())
            .into_iter()
            .take(usize::from(recipe.sub_required())),
    );
    let selected = keys
        .iter()
        .map(|key| {
            runtime
                .blessings()
                .iter()
                .find(|row| row.state_key() == *key)
                .unwrap()
                .id()
                .clone()
        })
        .collect::<Vec<_>>();
    assert!(selected.len() >= 2);
    let mut first = start(&factory);
    let mut replay = start(&factory);
    for activity in [&mut first, &mut replay] {
        acquire_target_equation(&offers, activity, &equation);
        let observations = progress
            .observations(activity)
            .expect("acquisition refreshed progress");
        assert!(
            observations
                .iter()
                .all(|row| row.main_count() == 0 && row.sub_count() == 0)
        );
    }
    let hash = first.state_hash();
    let before_draws = reward_draws(&first);
    let result = runtime
        .acquire_accepted_identities(&mut first, hash, &selected)
        .unwrap();
    let hash = replay.state_hash();
    let reconstructed = runtime
        .acquire_accepted_identities(&mut replay, hash, &selected)
        .unwrap();
    assert_eq!(result, reconstructed);
    assert_eq!(
        first.canonical_state_bytes(),
        replay.canonical_state_bytes()
    );
    assert_eq!(reward_draws(&first), before_draws);
    assert_eq!(result.owned().len(), selected.len());
    assert!(result.owned().iter().all(|owned| owned.level() == 1));
    assert_eq!(
        result
            .events()
            .iter()
            .filter(|event| matches!(event.kind(),
        ActivityTransactionEventKind::SlotChanged(slot) if *slot == BLESSINGS_SLOT))
            .count(),
        1
    );
    let observations = progress
        .observations(&first)
        .expect("no intermediate dirty progress");
    let actual = observations
        .iter()
        .find(|row| row.equation() == &equation)
        .unwrap();
    assert_eq!(actual.main_count(), recipe.main_required());
    assert_eq!(actual.sub_count(), recipe.sub_required());
    assert_eq!(
        actual.state(),
        DivergentUniverseEquationExpansionState::Expanded
    );
}

#[test]
fn blessing_reward_batch_rejects_duplicates_unknowns_existing_and_pending_offer() {
    let factory = DivergentUniverseRuntimeFactory::production().unwrap();
    let runtime = factory.blessing_runtime().unwrap();
    let good = runtime.blessings()[0].id().clone();
    let other = runtime.blessings()[1].id().clone();
    let unknown = DivergentUniverseBlessingId::new("divergent-universe.blessing.99999999").unwrap();
    let invalid = [
        (Vec::new(), BlessingError::InvalidAcquisitionCount),
        (
            vec![good.clone(); runtime.blessings().len() + 1],
            BlessingError::InvalidAcquisitionCount,
        ),
        (
            vec![good.clone(), good.clone()],
            BlessingError::AlreadyOwned,
        ),
        (vec![good.clone(), unknown], BlessingError::UnknownBlessing),
    ];
    let mut activity = start(&factory);
    for (selected, error) in invalid {
        let before = activity.canonical_state_bytes();
        let hash = activity.state_hash();
        let draws = reward_draws(&activity);
        assert_eq!(
            runtime.acquire_accepted_identities(&mut activity, hash, &selected),
            Err(error)
        );
        assert_eq!(activity.canonical_state_bytes(), before);
        assert_eq!(reward_draws(&activity), draws);
    }
    let hash = activity.state_hash();
    runtime
        .acquire_accepted_identity(&mut activity, hash, &good)
        .unwrap();
    let before = activity.canonical_state_bytes();
    let hash = activity.state_hash();
    assert_eq!(
        runtime.acquire_accepted_identities(&mut activity, hash, &[other.clone(), good]),
        Err(BlessingError::AlreadyOwned)
    );
    let stale = ActivityStateHash::new([0x91; 32]).unwrap();
    assert_eq!(
        runtime.acquire_accepted_identities(&mut activity, stale, from_ref(&other)),
        Err(BlessingError::Activity(
            GraphActivityCommandError::StaleStateHash
        ))
    );
    assert_eq!(activity.canonical_state_bytes(), before);
    let group = runtime
        .groups()
        .iter()
        .find(|group| !group.candidates().is_empty())
        .unwrap();
    let hash = activity.state_hash();
    runtime
        .begin_group_offer(&mut activity, hash, group.id())
        .unwrap();
    let before = activity.canonical_state_bytes();
    let hash = activity.state_hash();
    assert_eq!(
        runtime.acquire_accepted_identities(&mut activity, hash, &[other]),
        Err(BlessingError::OfferAlreadyActive)
    );
    assert_eq!(activity.canonical_state_bytes(), before);
}
