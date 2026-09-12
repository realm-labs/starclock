//! Current gain components, not full Curio expiry or battle-reward acceptance.

use starclock_activity::{
    ActivityExpression, ActivityMasterSeed, ActivityOperation, ActivityOptionId,
    ActivityProgramDefinition, ActivityProgramId, ActivityValue, GraphActivity,
};
use starclock_data::{
    divergent_universe_catalog::DivergentUniverseRunFamily,
    divergent_universe_curio_catalog::DivergentUniverseCurioStateId,
    divergent_universe_service_catalog::DivergentUniverseGambleUnitId,
};

use super::{currency_balance, entry, instance, reward_draws};
use crate::divergent_universe::{
    DivergentUniverseBaselineFixture, DivergentUniverseFlowInstance,
    DivergentUniverseRuntimeFactory,
    economy::{
        DivergentUniverseCurrencyCommand, DivergentUniverseCurrencyCommandError,
        DivergentUniverseCurrencyGainRule, DivergentUniverseCurrencyKind,
    },
    state::{CURRENCIES_SLOT, FRAGMENT_GAIN_BASE_SLOT},
    workbench_curse_runtime::DivergentUniverseCurseChestOperation,
};

fn state(raw: &str) -> DivergentUniverseCurioStateId {
    DivergentUniverseCurioStateId::new(format!("divergent-universe.curio-state.{raw}")).unwrap()
}
fn start(
    factory: &DivergentUniverseRuntimeFactory,
) -> (DivergentUniverseFlowInstance, GraphActivity) {
    let flow = factory.compile(entry("401", "3011")).unwrap();
    let activity = flow
        .start(instance(23660), ActivityMasterSeed::from_u64(23660))
        .unwrap()
        .into_activity();
    (flow, activity)
}
fn key(flow: &DivergentUniverseFlowInstance) -> u64 {
    flow.economy()
        .currency(DivergentUniverseCurrencyKind::CosmicFragment)
        .key()
}
fn credit(
    flow: &DivergentUniverseFlowInstance,
    activity: &mut GraphActivity,
    amount: u64,
) -> Result<(), DivergentUniverseCurrencyCommandError> {
    flow.apply_currency_command(
        activity,
        activity.state_hash(),
        DivergentUniverseCurrencyCommand::Credit {
            currency: DivergentUniverseCurrencyKind::CosmicFragment,
            rule: DivergentUniverseCurrencyGainRule::CurseChest,
            amount,
        },
    )
    .map(|_| ())
}
fn set_balance(activity: &mut GraphActivity, key: u64, value: i64) {
    let program = ActivityProgramDefinition::new(
        ActivityProgramId::new(23660).unwrap(),
        vec![ActivityOperation::SetCounter {
            slot: CURRENCIES_SLOT,
            key,
            value: ActivityExpression::Literal(ActivityValue::BoundedInteger(value)),
        }],
    )
    .unwrap();
    activity
        .apply_boundary_program(activity.state_hash(), &program)
        .unwrap();
}

#[test]
fn exact_active_fragment_rates_floor_without_recursion_and_large_values_fit() {
    let factory = DivergentUniverseRuntimeFactory::production().unwrap();
    let curios = factory.curio_runtime().unwrap();
    for (raw, amount, expected) in [
        ("9055", 1, 1),
        ("9055", 3, 4),
        ("9055", 101, 151),
        ("9070", 10, 15),
        ("9079", 3, 3),
        ("9079", 10, 13),
        ("9159", 101, 131),
        ("9055", 6_000_000_000_000_000_001, 9_000_000_000_000_000_001),
    ] {
        let (flow, mut activity) = start(&factory);
        let hash = activity.state_hash();
        curios
            .acquire_accepted_state(&mut activity, hash, &state(raw))
            .unwrap();
        let draws = reward_draws(&activity);
        credit(&flow, &mut activity, amount).unwrap();
        assert_eq!(currency_balance(&activity, key(&flow)), expected, "{raw}");
        assert_eq!(reward_draws(&activity), draws);
        assert_eq!(
            activity
                .player_view()
                .slots()
                .iter()
                .find(|slot| slot.id() == FRAGMENT_GAIN_BASE_SLOT)
                .unwrap()
                .value(),
            &ActivityValue::BoundedInteger(0)
        );
    }
}

#[test]
fn gain_components_stack_from_original_base_and_follow_live_lifecycle() {
    let factory = DivergentUniverseRuntimeFactory::production().unwrap();
    let curios = factory.curio_runtime().unwrap();
    let (flow, mut activity) = start(&factory);
    let (_, mut fresh) = start(&factory);
    // 9070 and 9079 are mutually exclusive copies of handbook 9068.
    let selected = [state("9055"), state("9070"), state("9159")];
    let mut reversed = selected.clone();
    reversed.reverse();
    let hash = activity.state_hash();
    curios
        .acquire_accepted_states(&mut activity, hash, &selected)
        .unwrap();
    let hash = fresh.state_hash();
    curios
        .acquire_accepted_states(&mut fresh, hash, &reversed)
        .unwrap();
    for target in [&mut activity, &mut fresh] {
        credit(&flow, target, 3).unwrap();
    }
    assert_eq!(currency_balance(&activity, key(&flow)), 5); // 3 + floor(1.5) twice, not floor(3*2.3)
    assert_eq!(
        activity.canonical_state_bytes(),
        fresh.canonical_state_bytes()
    );
    let hash = activity.state_hash();
    curios
        .destroy_accepted(&mut activity, hash, &state("9055"))
        .unwrap();
    credit(&flow, &mut activity, 100).unwrap();
    assert_eq!(currency_balance(&activity, key(&flow)), 185);
    let hash = activity.state_hash();
    curios
        .repair_accepted(&mut activity, hash, &state("9055"))
        .unwrap();
    credit(&flow, &mut activity, 100).unwrap();
    assert_eq!(currency_balance(&activity, key(&flow)), 415);
    let hash = activity.state_hash();
    curios
        .replace_accepted(&mut activity, hash, &state("9055"), &state("9001"))
        .unwrap();
    credit(&flow, &mut activity, 100).unwrap();
    assert_eq!(currency_balance(&activity, key(&flow)), 595);
    let hash = activity.state_hash();
    curios
        .replace_accepted(&mut activity, hash, &state("9070"), &state("9079"))
        .unwrap();
    credit(&flow, &mut activity, 100).unwrap();
    assert_eq!(currency_balance(&activity, key(&flow)), 755); // Two 30% states, not both handbook copies.
}

#[test]
fn acquisition_grants_share_gain_pipeline_and_bonus_overflow_rolls_back_everything() {
    let factory = DivergentUniverseRuntimeFactory::production().unwrap();
    let curios = factory.curio_runtime().unwrap();
    let (flow, mut activity) = start(&factory);
    let hash = activity.state_hash();
    curios
        .acquire_accepted_states(&mut activity, hash, &[state("9028"), state("9055")])
        .unwrap();
    assert_eq!(currency_balance(&activity, key(&flow)), 450);
    set_balance(&mut activity, key(&flow), 101);
    let hash = activity.state_hash();
    curios
        .acquire_accepted_state(&mut activity, hash, &state("9053"))
        .unwrap();
    assert_eq!(currency_balance(&activity, key(&flow)), 161); // floor(101*0.4)=40, plus 20 once
    set_balance(&mut activity, key(&flow), 0);
    let before = activity.canonical_state_bytes();
    let draws = reward_draws(&activity);
    assert!(credit(&flow, &mut activity, i64::MAX as u64).is_err());
    assert_eq!(activity.canonical_state_bytes(), before);
    assert_eq!(reward_draws(&activity), draws);
    set_balance(&mut activity, key(&flow), i64::MAX - 150);
    let before = activity.canonical_state_bytes();
    let draws = reward_draws(&activity);
    let hash = activity.state_hash();
    // Base 150 fits, but its 75 bonus fails after the Wax has already drawn.
    assert!(
        curios
            .acquire_accepted_states(&mut activity, hash, &[state("9043"), state("9195")])
            .is_err()
    );
    assert_eq!(activity.canonical_state_bytes(), before);
    assert_eq!(reward_draws(&activity), draws);
}

#[test]
fn production_event_and_curse_chest_use_same_gain_policy_without_multiplying_spends() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let curios = fixture.factory().curio_runtime().unwrap();
    for family in [
        DivergentUniverseRunFamily::Ordinary,
        DivergentUniverseRunFamily::Cyclical,
    ] {
        let flow = fixture.flow(family).unwrap();
        let mut activity = flow
            .start(instance(23660), ActivityMasterSeed::from_u64(23660))
            .unwrap()
            .into_activity();
        super::initial_equations::accept_initial(&flow, &mut activity);
        let hash = activity.state_hash();
        curios
            .acquire_accepted_state(&mut activity, hash, &state("9055"))
            .unwrap();
        let hash = activity.state_hash();
        let decision = activity.player_view().decision().unwrap().id();
        flow.choose_occurrence_option(
            fixture.factory(),
            &mut activity,
            hash,
            decision,
            ActivityOptionId::new(1).unwrap(),
        )
        .unwrap();
        assert_eq!(currency_balance(&activity, key(&flow)), 300);
        let chests = fixture.factory().workbench_curse_runtime().unwrap();
        for gain in [true, false] {
            let (chest, index) = chests
                .curse_chests()
                .iter()
                .find_map(|chest| {
                    chest
                        .operations()
                        .iter()
                        .enumerate()
                        .find_map(|(index, operation)| {
                            let matches = if gain {
                                matches!(
                                    operation,
                                    DivergentUniverseCurseChestOperation::GainCosmicFragments { .. }
                                )
                            } else {
                                matches!(
                                    operation,
                                    DivergentUniverseCurseChestOperation::LoseCosmicFragments { .. }
                                )
                            };
                            matches.then_some((chest, index))
                        })
                })
                .unwrap();
            let hash = activity.state_hash();
            chests
                .execute_curse_chest_choice_policy_accepted(
                    &mut activity,
                    hash,
                    chest.id(),
                    index,
                    Some(100),
                )
                .unwrap();
            assert_eq!(
                currency_balance(&activity, key(&flow)),
                if gain { 450 } else { 350 }
            );
        }
        let gamble = fixture.factory().gamble_runtime().unwrap();
        for raw in ["301", "302"] {
            let unit =
                DivergentUniverseGambleUnitId::new(format!("divergent-universe.gamble-unit.{raw}"))
                    .unwrap();
            let hash = activity.state_hash();
            gamble
                .execute_accepted_unit(&flow, &mut activity, hash, &unit)
                .unwrap();
        }
        assert_eq!(currency_balance(&activity, key(&flow)), 440); // 350 + (20+10) + (40+20)
    }
}
