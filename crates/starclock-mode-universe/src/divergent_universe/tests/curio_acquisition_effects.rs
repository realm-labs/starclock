//! Reviewed immediate effects; full Curio lifecycle acceptance remains separate.

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
};

use super::{entry, instance, reward_draws};
use crate::divergent_universe::{
    DivergentUniverseBaselineFixture, DivergentUniverseRuntimeFactory,
    curio_runtime::DivergentUniverseCurioRuntimeError,
    economy::DivergentUniverseCurrencyKind,
    state::{CURRENCIES_SLOT, ROOM_FINISHED_SLOT},
};

fn state(raw: &str) -> DivergentUniverseCurioStateId {
    DivergentUniverseCurioStateId::new(format!("divergent-universe.curio-state.{raw}")).unwrap()
}

fn start(factory: &DivergentUniverseRuntimeFactory) -> GraphActivity {
    factory
        .compile(entry("401", "3011"))
        .unwrap()
        .start(instance(23640), ActivityMasterSeed::from_u64(23640))
        .unwrap()
        .into_activity()
}

fn set_balance(activity: &mut GraphActivity, key: u64, amount: i64) {
    let program = ActivityProgramDefinition::new(
        ActivityProgramId::new(23640).unwrap(),
        vec![ActivityOperation::SetCounter {
            slot: CURRENCIES_SLOT,
            key,
            value: ActivityExpression::Literal(ActivityValue::BoundedInteger(amount)),
        }],
    )
    .unwrap();
    activity
        .apply_boundary_program(activity.state_hash(), &program)
        .unwrap();
}

fn balance(activity: &GraphActivity, key: u64) -> i64 {
    let view = activity.player_view();
    let ActivityValue::BoundedCounterMap(values) = view
        .slots()
        .iter()
        .find(|slot| slot.id() == CURRENCIES_SLOT)
        .unwrap()
        .value()
    else {
        panic!("currency counter")
    };
    values
        .iter()
        .find(|(candidate, _)| *candidate == key)
        .map_or(0, |(_, value)| *value)
}

#[test]
fn reviewed_curio_acquisition_amounts_floor_and_large_balance_are_exact() {
    let factory = DivergentUniverseRuntimeFactory::production().unwrap();
    let runtime = factory.curio_runtime().unwrap();
    let key = factory
        .compile(entry("401", "3011"))
        .unwrap()
        .economy()
        .currency(DivergentUniverseCurrencyKind::CosmicFragment)
        .key();
    for (id, before, after) in [
        ("9028", 101, 401),
        ("9167", 101, 601),
        ("9195", 101, 251),
        ("9053", 0, 0),
        ("9053", 1, 1),
        ("9053", 2, 2),
        ("9053", 3, 4),
        ("9053", 7, 9),
        ("9053", 101, 141),
        ("9053", 6_000_000_000_000_000_001, 8_400_000_000_000_000_001),
    ] {
        let mut activity = start(&factory);
        set_balance(&mut activity, key, before);
        let draws = reward_draws(&activity);
        let hash = activity.state_hash();
        runtime
            .acquire_accepted_state(&mut activity, hash, &state(id))
            .unwrap();
        assert_eq!(balance(&activity, key), after, "{id}, before={before}");
        assert_eq!(reward_draws(&activity), draws);
        let accepted = activity.canonical_state_bytes();
        let hash = activity.state_hash();
        assert_eq!(
            runtime.acquire_accepted_state(&mut activity, hash, &state(id)),
            Err(DivergentUniverseCurioRuntimeError::AlreadyOwned)
        );
        assert_eq!(activity.canonical_state_bytes(), accepted);
    }
}

#[test]
fn acquisition_order_is_stable_and_overflow_or_unbound_identity_rolls_back() {
    let factory = DivergentUniverseRuntimeFactory::production().unwrap();
    let runtime = factory.curio_runtime().unwrap();
    let key = factory
        .compile(entry("401", "3011"))
        .unwrap()
        .economy()
        .currency(DivergentUniverseCurrencyKind::CosmicFragment)
        .key();
    let mut first = start(&factory);
    let mut second = start(&factory);
    for activity in [&mut first, &mut second] {
        set_balance(activity, key, 101);
    }
    let hash = first.state_hash();
    let left = runtime
        .acquire_accepted_states(&mut first, hash, &[state("9053"), state("9028")])
        .unwrap();
    let hash = second.state_hash();
    let right = runtime
        .acquire_accepted_states(&mut second, hash, &[state("9028"), state("9053")])
        .unwrap();
    assert_eq!(balance(&first, key), 561); // (101+300) + floor(401*0.4)
    assert_eq!(left, right);
    assert_eq!(
        first.canonical_state_bytes(),
        second.canonical_state_bytes()
    );
    for selected in [
        vec![state("9028")],
        vec![state("9053")],
        vec![state("9028"), state("9196")],
        vec![state("9197")],
    ] {
        let mut activity = start(&factory);
        set_balance(&mut activity, key, i64::MAX);
        let before = activity.canonical_state_bytes();
        let hash = activity.state_hash();
        assert!(
            runtime
                .acquire_accepted_states(&mut activity, hash, &selected)
                .is_err()
        );
        assert_eq!(activity.canonical_state_bytes(), before);
    }
}

#[test]
fn production_occurrence_rewards_execute_acquisition_before_room_completion() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let runtime = fixture.factory().curio_runtime().unwrap();
    let targets = [state("9028"), state("9053")];
    let held = runtime
        .curios()
        .iter()
        .filter(|curio| {
            matches!(
                curio.category(),
                DivergentUniverseCurioCategory::Common | DivergentUniverseCurioCategory::Rare
            ) && !curio.states().iter().any(|state| targets.contains(state))
        })
        .map(|curio| {
            curio
                .states()
                .iter()
                .min_by_key(|state| state.as_str())
                .unwrap()
                .clone()
        })
        .collect::<Vec<_>>();
    for family in [
        DivergentUniverseRunFamily::Ordinary,
        DivergentUniverseRunFamily::Cyclical,
    ] {
        let flow = fixture.flow(family).unwrap();
        let key = flow
            .economy()
            .currency(DivergentUniverseCurrencyKind::CosmicFragment)
            .key();
        let mut activity = flow
            .start(instance(1), ActivityMasterSeed::from_u64(23641))
            .unwrap()
            .into_activity();
        super::initial_equations::accept_initial(&flow, &mut activity);
        let hash = activity.state_hash();
        runtime
            .acquire_accepted_states(&mut activity, hash, &held)
            .unwrap();
        set_balance(&mut activity, key, i64::MAX);
        let before = activity.canonical_state_bytes();
        let hash = activity.state_hash();
        let decision = activity.player_view().decision().unwrap().id();
        let draws = reward_draws(&activity);
        assert!(
            flow.choose_occurrence_option(
                fixture.factory(),
                &mut activity,
                hash,
                decision,
                ActivityOptionId::new(2).unwrap()
            )
            .is_err()
        );
        assert_eq!(activity.canonical_state_bytes(), before);
        assert_eq!(reward_draws(&activity), draws);
        set_balance(&mut activity, key, 101);
        let hash = activity.state_hash();
        let decision = activity.player_view().decision().unwrap().id();
        let draws = reward_draws(&activity);
        let result = flow
            .choose_occurrence_option(
                fixture.factory(),
                &mut activity,
                hash,
                decision,
                ActivityOptionId::new(2).unwrap(),
            )
            .unwrap();
        // Canonical preloads include 50% Gossip and 30% state 9159, not
        // alternate copies 9070/9079 of handbook 9068.
        // 101+(300+150+90)=641; floor(641*0.4)=256, plus 128+76.
        assert_eq!(balance(&activity, key), 1101);
        assert_eq!(reward_draws(&activity) - draws, 2);
        let last_gain = result.events().iter().rposition(|event| matches!(event.kind(), ActivityTransactionEventKind::CounterChanged { slot, .. } if *slot == CURRENCIES_SLOT)).unwrap();
        let completed = result
            .events()
            .iter()
            .position(|event| {
                event.kind() == &ActivityTransactionEventKind::SlotChanged(ROOM_FINISHED_SLOT)
            })
            .unwrap();
        assert!(last_gain < completed);
    }
}
