//! Production-data reward composition on a test-only choice graph. The baseline
//! binding is tested separately; these are not complete event/room fixtures.

use std::sync::Arc;

use starclock_activity::{
    ActivityCondition, ActivityDefinitionDigest, ActivityDefinitionId, ActivityDefinitionIdentity,
    ActivityExpression, ActivityGeneratedBoundaryResolution, ActivityMasterSeed, ActivityOperation,
    ActivityOptionDefinition, ActivityOptionId, ActivityProgramDefinition, ActivityRandomPolicies,
    ActivityValue, GraphActivity, GraphActivityCommandError, GraphActivityDefinition,
    GraphActivityNodeProgram, GraphActivityRuntimeError,
};
use starclock_data::{
    divergent_universe_blessing_catalog::DivergentUniverseBlessingCategory,
    divergent_universe_curio_catalog::{
        DivergentUniverseCurioCategory, DivergentUniverseCurioStateId,
    },
    divergent_universe_decisions::DecisionChoice,
};

use super::curio_acquisition_blessings::acquisition_draws;
use super::{entry, instance, reward_draws};
use crate::divergent_universe::{
    DivergentUniverseRuntimeFactory,
    decision_rewards::{DecisionRewardError, DecisionRewardGrant, DecisionRewardRuntime},
    state::CURRENCIES_SLOT,
};

fn choice(factory: &DivergentUniverseRuntimeFactory, ordinal: u16) -> &DecisionChoice {
    factory
        .decision_catalog()
        .occurrences()
        .iter()
        .find(|event| event.variant.as_str() == "divergent-universe.occurrence-variant.722601")
        .unwrap()
        .choices
        .iter()
        .find(|choice| choice.ordinal == ordinal)
        .unwrap()
}

fn start(factory: &DivergentUniverseRuntimeFactory, reject_after_reward: bool) -> GraphActivity {
    let flow = factory.compile(entry("401", "3011")).unwrap();
    let base = flow.definition();
    let first = &base.programs()[0];
    let mut operations = first.program().operations().to_vec();
    let offer = operations
        .iter_mut()
        .find(|operation| matches!(operation, ActivityOperation::Offer { .. }))
        .unwrap();
    let ActivityOperation::Offer { options, .. } = offer else {
        unreachable!()
    };
    let original = options[0].operations();
    *options = (1..=3)
        .map(|ordinal| {
            let authored = choice(factory, ordinal);
            let mut operations = original.to_vec();
            if reject_after_reward {
                operations.insert(
                    0,
                    ActivityOperation::Require(ActivityCondition::Boolean(
                        ActivityExpression::Literal(ActivityValue::Boolean(false)),
                    )),
                );
            }
            ActivityOptionDefinition::new(
                ActivityOptionId::new(u64::from(authored.ordinal)).unwrap(),
                i32::from(authored.ordinal),
                ActivityCondition::Boolean(ActivityExpression::Literal(ActivityValue::Boolean(
                    true,
                ))),
                operations,
            )
        })
        .collect();
    let mut programs = base.programs().to_vec();
    programs[0] = GraphActivityNodeProgram::new(
        first.node(),
        ActivityProgramDefinition::new(first.program().id(), operations).unwrap(),
    );
    // Deliberate fixture identity; do not call this the production entry graph.
    let identity = ActivityDefinitionIdentity::new(
        ActivityDefinitionId::new(23_620).unwrap(),
        ActivityDefinitionDigest::new([0x62; 32]).unwrap(),
        base.identity().config_digest(),
    );
    let definition = Arc::new(
        GraphActivityDefinition::new(
            identity,
            base.graph().clone(),
            base.state_definition().clone(),
            Arc::clone(base.participants()),
            programs,
            base.bootstrap().cloned(),
            ActivityRandomPolicies::new(
                base.random_checkpoints().to_vec(),
                base.random_offers().to_vec(),
            ),
        )
        .unwrap(),
    );
    GraphActivity::start(
        definition,
        instance(23_620),
        ActivityMasterSeed::from_u64(23_620),
    )
    .unwrap()
    .into_activity()
}

fn select(
    factory: &DivergentUniverseRuntimeFactory,
    runtime: &DecisionRewardRuntime,
    activity: &mut GraphActivity,
    ordinal: u16,
) -> Result<ActivityGeneratedBoundaryResolution<DecisionRewardGrant>, GraphActivityCommandError> {
    let hash = activity.state_hash();
    let offered = activity.player_view().decision().unwrap().id();
    activity.choose_option_with_generated_prefix(
        hash,
        offered,
        ActivityOptionId::new(u64::from(ordinal)).unwrap(),
        |view, rng| {
            runtime
                .generate_choice(view, &choice(factory, ordinal).id, rng)
                .map(|program| program.into_parts())
                .map_err(|error| match error {
                    DecisionRewardError::Rng(error) => GraphActivityCommandError::Rng(error),
                    _ => GraphActivityCommandError::Runtime(
                        GraphActivityRuntimeError::InvalidBoundaryProgram,
                    ),
                })
        },
    )
}

#[test]
fn authored_three_choice_rewards_commit_real_inventory_and_reconstruct() {
    let factory = DivergentUniverseRuntimeFactory::production().unwrap();
    let runtime = factory.decision_reward_runtime().unwrap();
    assert_eq!(
        runtime.configuration_digest(),
        factory.decision_catalog().digest()
    );
    for ordinal in 1..=3 {
        let mut activity = start(&factory, false);
        let mut replay = start(&factory, false);
        runtime
            .check_choice(&activity.player_view(), &choice(&factory, ordinal).id)
            .unwrap();
        let before_draws = reward_draws(&activity);
        let before_node = activity.current_node();
        let result = select(&factory, &runtime, &mut activity, ordinal).unwrap();
        let reconstructed = select(&factory, &runtime, &mut replay, ordinal).unwrap();
        assert_eq!(result, reconstructed);
        assert_eq!(
            activity.canonical_state_bytes(),
            replay.canonical_state_bytes()
        );
        assert_ne!(before_node, activity.current_node());
        match result.value() {
            DecisionRewardGrant::Fragments(amount) => {
                assert_eq!(*amount, 200);
                assert_eq!(before_draws, reward_draws(&activity));
                let view = activity.player_view();
                let value = view
                    .slots()
                    .iter()
                    .find(|slot| slot.id() == CURRENCIES_SLOT)
                    .unwrap();
                let ActivityValue::BoundedCounterMap(values) = value.value() else {
                    panic!("currency map")
                };
                assert_eq!(values.len(), 1);
                assert_eq!(values[0].1, 200);
            }
            DecisionRewardGrant::Curios(states) => {
                assert_eq!(states.len(), 2);
                assert_ne!(states[0], states[1]);
                let curios = factory.curio_runtime().unwrap();
                let owned = curios.owned(&activity).unwrap();
                assert_eq!(owned.len(), 2);
                assert_ne!(owned[0].curio(), owned[1].curio());
                for held in &owned {
                    let definition = curios
                        .curios()
                        .iter()
                        .find(|curio| curio.id() == held.curio())
                        .unwrap();
                    assert!(matches!(
                        definition.category(),
                        DivergentUniverseCurioCategory::Common
                            | DivergentUniverseCurioCategory::Rare
                    ));
                    let canonical = definition
                        .states()
                        .iter()
                        .min_by_key(|state| state.as_str())
                        .unwrap();
                    assert_eq!(held.state(), canonical);
                }
                assert_eq!(
                    reward_draws(&activity) - before_draws,
                    2 + acquisition_draws(&factory, states)
                );
            }
            DecisionRewardGrant::Blessings(ids) => {
                assert_eq!(ids.len(), 2);
                assert_ne!(ids[0], ids[1]);
                let blessings = factory.blessing_runtime().unwrap();
                let owned = blessings.owned(&activity).unwrap();
                assert_eq!(owned.len(), 2);
                for held in &owned {
                    assert_eq!(held.level(), 1);
                    let definition = blessings
                        .blessings()
                        .iter()
                        .find(|blessing| blessing.id() == held.blessing())
                        .unwrap();
                    assert!(matches!(
                        definition.category(),
                        DivergentUniverseBlessingCategory::Common
                            | DivergentUniverseBlessingCategory::Rare
                    ));
                }
                factory
                    .equation_progress_runtime()
                    .unwrap()
                    .observations(&activity)
                    .unwrap();
                assert_eq!(reward_draws(&activity) - before_draws, 2);
            }
        }
    }
}

#[test]
fn late_choice_rejection_rolls_back_fragments_curios_blessings_and_rng() {
    let factory = DivergentUniverseRuntimeFactory::production().unwrap();
    let runtime = factory.decision_reward_runtime().unwrap();
    for ordinal in 1..=3 {
        let mut activity = start(&factory, true);
        runtime
            .check_choice(&activity.player_view(), &choice(&factory, ordinal).id)
            .unwrap();
        let before = activity.canonical_state_bytes();
        assert!(select(&factory, &runtime, &mut activity, ordinal).is_err());
        assert_eq!(before, activity.canonical_state_bytes());
    }
}

fn curio_pool(factory: &DivergentUniverseRuntimeFactory) -> Vec<DivergentUniverseCurioStateId> {
    factory
        .curio_runtime()
        .unwrap()
        .curios()
        .iter()
        .filter(|curio| {
            matches!(
                curio.category(),
                DivergentUniverseCurioCategory::Common | DivergentUniverseCurioCategory::Rare
            )
        })
        .map(|curio| {
            curio
                .states()
                .iter()
                .min_by_key(|state| state.as_str())
                .unwrap()
                .clone()
        })
        .collect()
}

#[test]
fn exhausted_curio_or_blessing_pool_reports_unavailability_without_rng() {
    let factory = DivergentUniverseRuntimeFactory::production().unwrap();
    let runtime = factory.decision_reward_runtime().unwrap();
    for ordinal in [2, 3] {
        let mut activity = start(&factory, false);
        let hash = activity.state_hash();
        if ordinal == 2 {
            let mut held = curio_pool(&factory);
            held.pop();
            factory
                .curio_runtime()
                .unwrap()
                .acquire_accepted_states(&mut activity, hash, &held)
                .unwrap();
        } else {
            let blessings = factory.blessing_runtime().unwrap();
            let mut held = blessings
                .blessings()
                .iter()
                .filter(|blessing| {
                    matches!(
                        blessing.category(),
                        DivergentUniverseBlessingCategory::Common
                            | DivergentUniverseBlessingCategory::Rare
                    )
                })
                .map(|blessing| blessing.id().clone())
                .collect::<Vec<_>>();
            held.pop();
            blessings
                .acquire_accepted_identities(&mut activity, hash, &held)
                .unwrap();
        }
        let before = activity.canonical_state_bytes();
        assert_eq!(
            runtime.check_choice(&activity.player_view(), &choice(&factory, ordinal).id),
            Err(DecisionRewardError::InsufficientCandidates {
                required: 2,
                available: 1
            })
        );
        assert!(select(&factory, &runtime, &mut activity, ordinal).is_err());
        assert_eq!(before, activity.canonical_state_bytes());
        runtime
            .check_choice(&activity.player_view(), &choice(&factory, 1).id)
            .unwrap();
        assert_eq!(
            select(&factory, &runtime, &mut activity, 1)
                .unwrap()
                .value(),
            &DecisionRewardGrant::Fragments(200)
        );
    }
}

#[test]
fn only_the_two_unowned_identities_can_be_selected() {
    let factory = DivergentUniverseRuntimeFactory::production().unwrap();
    let runtime = factory.decision_reward_runtime().unwrap();
    for ordinal in [2, 3] {
        let mut activity = start(&factory, false);
        let hash = activity.state_hash();
        let mut expected = if ordinal == 2 {
            let mut held = curio_pool(&factory);
            let missing = held.split_off(held.len() - 2);
            factory
                .curio_runtime()
                .unwrap()
                .acquire_accepted_states(&mut activity, hash, &held)
                .unwrap();
            missing
                .iter()
                .map(|id| id.as_str().to_owned())
                .collect::<Vec<_>>()
        } else {
            let blessings = factory.blessing_runtime().unwrap();
            let mut held = blessings
                .blessings()
                .iter()
                .filter(|blessing| {
                    matches!(
                        blessing.category(),
                        DivergentUniverseBlessingCategory::Common
                            | DivergentUniverseBlessingCategory::Rare
                    )
                })
                .map(|blessing| blessing.id().clone())
                .collect::<Vec<_>>();
            let missing = held.split_off(held.len() - 2);
            blessings
                .acquire_accepted_identities(&mut activity, hash, &held)
                .unwrap();
            missing
                .iter()
                .map(|id| id.as_str().to_owned())
                .collect::<Vec<_>>()
        };
        runtime
            .check_choice(&activity.player_view(), &choice(&factory, ordinal).id)
            .unwrap();
        let grant = select(&factory, &runtime, &mut activity, ordinal).unwrap();
        let mut actual = match grant.value() {
            DecisionRewardGrant::Curios(ids) => ids
                .iter()
                .map(|id| id.as_str().to_owned())
                .collect::<Vec<_>>(),
            DecisionRewardGrant::Blessings(ids) => ids
                .iter()
                .map(|id| id.as_str().to_owned())
                .collect::<Vec<_>>(),
            DecisionRewardGrant::Fragments(_) => panic!("inventory choice"),
        };
        actual.sort();
        expected.sort();
        assert_eq!(actual, expected);
    }
}
