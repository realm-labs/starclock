use std::collections::BTreeMap;

use starclock_activity::{
    ActivityCause, ActivityProgramDefinition, ActivitySlotId, ActivityTransactionOutcome,
    ActivityTransactionRejection, ActivityTransactionState, ActivityValue,
};

use super::{
    GOLD_AND_GEARS_NEURAL_RUNTIME_REVISION, GoldAndGearsEntryError,
    GoldAndGearsNeuralBattleEntryContext, GoldAndGearsNeuralBattleStat, GoldAndGearsRuntimeFactory,
    GoldAndGearsRuntimeInstance,
    state_layout::{
        PLANE_ACTION_POINTS_KEY, PLANE_STATE_SLOT, PROGRESSION_NEURAL_REBOOT_BATTLES_KEY,
        PROGRESSION_SLOT, RESOURCE_DICE_REROLLS_KEY, RUN_RESOURCES_SLOT,
    },
    tests::entry,
};

const BUNDLE: &[u8] = include_bytes!("../../../../config/gold-and-gears-generated/config.sora");
const AREA: &str = "gold-gears.area.401";
const PATH: &str = "universe.path.preservation";

#[test]
fn all_forty_nodes_compile_exact_costs_and_immutable_battle_contributions() {
    let factory = GoldAndGearsRuntimeFactory::load_candidate(BUNDLE).unwrap();
    assert_eq!(factory.neural.denominators(), (40, 30, 31_250));
    assert_eq!(
        GOLD_AND_GEARS_NEURAL_RUNTIME_REVISION,
        "gold-and-gears-neural-runtime-v1"
    );

    let mut selected = all_neural(&factory);
    selected.reverse();
    let instance = compile(&factory, selected);
    let contributions = instance.neural_battle_stat_contributions();
    assert_eq!(contributions.len(), 30);
    assert!(
        contributions
            .windows(2)
            .all(|pair| source(&factory, pair[0].source_node())
                < source(&factory, pair[1].source_node()))
    );
    assert!(
        contributions
            .iter()
            .all(|contribution| contribution.ratio_scaled() > 0)
    );
    let totals = contributions
        .iter()
        .fold(BTreeMap::new(), |mut totals, contribution| {
            *totals.entry(contribution.stat()).or_insert(0_i64) += contribution.ratio_scaled();
            totals
        });
    assert_eq!(
        totals[&GoldAndGearsNeuralBattleStat::PartyAttackRatio],
        550_000
    );
    assert_eq!(
        totals[&GoldAndGearsNeuralBattleStat::PathResonanceDamageRatio],
        800_000
    );
    assert_eq!(
        totals[&GoldAndGearsNeuralBattleStat::PartyMaximumHpRatio],
        400_000
    );
    assert_eq!(totals.len(), 11);
    assert_eq!(
        instance.neural_contribution_digest(),
        [
            0, 121, 69, 77, 175, 139, 178, 229, 26, 2, 221, 86, 218, 169, 41, 3, 155, 176, 156, 99,
            33, 167, 228, 135, 47, 167, 50, 16, 95, 75, 0, 40,
        ]
    );

    let duplicate = compile(&factory, all_neural(&factory));
    assert_eq!(
        instance.neural_contribution_digest(),
        duplicate.neural_contribution_digest(),
        "caller selection order must not affect immutable contributions"
    );
}

#[test]
fn acquisition_plan_enforces_currency_prerequisites_closure_and_exact_cost() {
    let factory = GoldAndGearsRuntimeFactory::load_candidate(BUNDLE).unwrap();
    let root = key(&factory, "101");
    let root_plan = factory.compile_neural_acquisition(&[], &root, 500).unwrap();
    assert_eq!(root_plan.node(), root);
    assert_eq!(root_plan.source_item_id(), 281_013);
    assert_eq!(root_plan.cost(), 250);
    assert_eq!(root_plan.remaining(), 250);
    assert_eq!(
        factory
            .compile_neural_acquisition(&[], &root, 249)
            .unwrap_err(),
        GoldAndGearsEntryError::InsufficientNeuralCurrency {
            required: 250,
            available: 249
        }
    );

    let reboot = key(&factory, "201");
    assert!(matches!(
        factory
            .compile_neural_acquisition(&[], &reboot, 600)
            .unwrap_err(),
        GoldAndGearsEntryError::MissingNeuralPrerequisite { .. }
    ));
    let prerequisites = ["101", "102", "103"]
        .map(|source| key(&factory, source))
        .to_vec();
    let reboot_plan = factory
        .compile_neural_acquisition(&prerequisites, &reboot, 1_000)
        .unwrap();
    assert_eq!(reboot_plan.cost(), 600);
    assert_eq!(reboot_plan.remaining(), 400);

    let duplicate = vec![root.clone(), root.clone()];
    assert_eq!(
        factory
            .compile_neural_acquisition(&duplicate, &key(&factory, "102"), 250)
            .unwrap_err(),
        GoldAndGearsEntryError::DuplicateNeuralNode(root.clone().into())
    );
    assert_eq!(
        factory
            .compile_neural_acquisition(std::slice::from_ref(&root), &root, 250)
            .unwrap_err(),
        GoldAndGearsEntryError::NeuralAlreadyAcquired(root.into())
    );
}

#[test]
fn activity_service_and_dice_effects_execute_at_their_declared_boundaries() {
    let factory = GoldAndGearsRuntimeFactory::load_candidate(BUNDLE).unwrap();
    let instance = compile(&factory, all_neural(&factory));
    assert_eq!(instance.neural_blessing_store_offer_count(), 3);
    assert_eq!(
        instance
            .neural_trailblaze_bonus_unlocks()
            .collect::<Vec<_>>(),
        [
            "gold-gears.trailblaze-bonus.204",
            "gold-gears.trailblaze-bonus.205"
        ]
    );
    assert_eq!(
        instance.dice_slot_max_rarities().collect::<Vec<_>>(),
        [3, 3, 3, 2, 2, 2]
    );

    let mut first = new_state(&instance);
    let first_program = instance.compile_neural_plane_start(1).unwrap().unwrap();
    commit(&instance, &mut first, first_program);
    assert_eq!(
        counter(&first, PLANE_STATE_SLOT, PLANE_ACTION_POINTS_KEY),
        1
    );
    assert_eq!(
        counter(&first, RUN_RESOURCES_SLOT, RESOURCE_DICE_REROLLS_KEY),
        1
    );

    let mut second = new_state(&instance);
    let second_program = instance.compile_neural_plane_start(2).unwrap().unwrap();
    commit(&instance, &mut second, second_program);
    assert_eq!(
        counter(&second, PLANE_STATE_SLOT, PLANE_ACTION_POINTS_KEY),
        1
    );
    assert_eq!(
        counter(&second, RUN_RESOURCES_SLOT, RESOURCE_DICE_REROLLS_KEY),
        2
    );
    assert_eq!(
        instance.compile_neural_plane_start(0).unwrap_err(),
        GoldAndGearsEntryError::InvalidPlaneLayer
    );

    let baseline_bonus = bonus(&factory, "201");
    factory
        .compile_entry(
            entry(&factory, AREA, PATH, &factory.unique.dice[0])
                .with_trailblaze_bonus(baseline_bonus),
        )
        .expect("three baseline bonuses require no Neural unlock");
    let locked = bonus(&factory, "204");
    assert_eq!(
        factory
            .compile_entry(
                entry(&factory, AREA, PATH, &factory.unique.dice[0],)
                    .with_trailblaze_bonus(locked.clone()),
            )
            .unwrap_err(),
        GoldAndGearsEntryError::LockedTrailblazeBonus(locked.into())
    );
}

#[test]
fn reboot_plane_projects_four_non_boss_entries_and_rejects_stale_accounting() {
    let factory = GoldAndGearsRuntimeFactory::load_candidate(BUNDLE).unwrap();
    let selected = ["101", "102", "103", "201"]
        .map(|source| key(&factory, source))
        .to_vec();
    let instance = compile(&factory, selected);
    let eligible = GoldAndGearsNeuralBattleEntryContext::new(1, false, true);
    let mut state = new_state(&instance);
    assert!(
        instance
            .compile_neural_battle_entry(
                &state,
                GoldAndGearsNeuralBattleEntryContext::new(1, false, false)
            )
            .unwrap()
            .is_none()
    );
    assert!(
        instance
            .compile_neural_battle_entry(
                &state,
                GoldAndGearsNeuralBattleEntryContext::new(1, true, true)
            )
            .unwrap()
            .is_none()
    );
    assert!(
        instance
            .compile_neural_battle_entry(
                &state,
                GoldAndGearsNeuralBattleEntryContext::new(2, false, true)
            )
            .unwrap()
            .is_none()
    );

    for expected in 1..=3 {
        let effect = instance
            .compile_neural_battle_entry(&state, eligible)
            .unwrap()
            .unwrap();
        assert_eq!(effect.source_node(), key(&factory, "201"));
        assert_eq!(effect.target_max_hp_ratio_scaled(), 990_000);
        commit(&instance, &mut state, effect.accounting_program().clone());
        assert_eq!(
            counter(
                &state,
                PROGRESSION_SLOT,
                PROGRESSION_NEURAL_REBOOT_BATTLES_KEY
            ),
            expected
        );
    }

    let first = instance
        .compile_neural_battle_entry(&state, eligible)
        .unwrap()
        .unwrap();
    let stale = first.accounting_program().clone();
    commit(&instance, &mut state, first.accounting_program().clone());
    let sequence = state.command_sequence();
    let before = format!("{state:?}");
    let cause = ActivityCause::new(sequence + 1, stale.id(), state.current_node()).unwrap();
    assert_eq!(
        state.apply_program(&stale, cause, instance.graph_definition()),
        ActivityTransactionOutcome::Rejected(ActivityTransactionRejection::ConditionNotSatisfied)
    );
    assert_eq!(state.command_sequence(), sequence);
    assert_eq!(format!("{state:?}"), before);
    assert!(
        instance
            .compile_neural_battle_entry(&state, eligible)
            .unwrap()
            .is_none()
    );
}

#[test]
fn production_program_matches_the_neural_network_effect_semantic_fixture() {
    let factory = GoldAndGearsRuntimeFactory::load_candidate(BUNDLE).unwrap();
    let selected = ["101", "102", "103", "201"]
        .map(|source| key(&factory, source))
        .to_vec();
    let instance = compile(&factory, selected);
    let state = new_state(&instance);
    let effect = instance
        .compile_neural_battle_entry(
            &state,
            GoldAndGearsNeuralBattleEntryContext::new(1, false, true),
        )
        .unwrap()
        .unwrap();
    let authored = factory
        .unique
        .neural_nodes
        .iter()
        .find(|node| node.identity.source_id.as_ref() == "201")
        .unwrap();
    assert_eq!(authored.disposition.as_ref(), "MechanicallyRelevant");
    assert_eq!(authored.effect_domain.as_ref(), "ActivityAndBattle");
    assert_eq!(effect.target_max_hp_ratio_scaled(), 990_000);
    assert_eq!(effect.accounting_program().operations().len(), 2);
}

fn compile(
    factory: &GoldAndGearsRuntimeFactory,
    selected: Vec<String>,
) -> GoldAndGearsRuntimeInstance {
    factory
        .compile_entry(
            entry(factory, AREA, PATH, &factory.unique.dice[0]).with_neural_network(selected),
        )
        .unwrap()
}

fn all_neural(factory: &GoldAndGearsRuntimeFactory) -> Vec<String> {
    factory
        .unique
        .neural_nodes
        .iter()
        .map(|node| node.identity.stable_key.to_string())
        .collect()
}

fn key(factory: &GoldAndGearsRuntimeFactory, source_id: &str) -> String {
    factory
        .unique
        .neural_nodes
        .iter()
        .find(|node| node.identity.source_id.as_ref() == source_id)
        .unwrap()
        .identity
        .stable_key
        .to_string()
}

fn source(factory: &GoldAndGearsRuntimeFactory, stable_key: &str) -> u32 {
    factory
        .unique
        .neural_nodes
        .iter()
        .find(|node| node.identity.stable_key.as_ref() == stable_key)
        .unwrap()
        .identity
        .source_id
        .parse()
        .unwrap()
}

fn bonus(factory: &GoldAndGearsRuntimeFactory, source_id: &str) -> String {
    factory
        .unique
        .trailblaze_bonuses
        .iter()
        .find(|bonus| bonus.identity.source_id.as_ref() == source_id)
        .unwrap()
        .identity
        .stable_key
        .to_string()
}

fn new_state(instance: &GoldAndGearsRuntimeInstance) -> ActivityTransactionState {
    ActivityTransactionState::new(
        instance.state_definition().clone(),
        instance.graph_definition().entry(),
    )
}

fn commit(
    instance: &GoldAndGearsRuntimeInstance,
    state: &mut ActivityTransactionState,
    program: ActivityProgramDefinition,
) {
    program
        .validate_against(instance.state_definition(), instance.graph_definition())
        .unwrap();
    let cause = ActivityCause::new(
        state.command_sequence() + 1,
        program.id(),
        state.current_node(),
    )
    .unwrap();
    assert!(matches!(
        state.apply_program(&program, cause, instance.graph_definition()),
        ActivityTransactionOutcome::Committed(_)
    ));
}

fn counter(state: &ActivityTransactionState, slot_id: u32, key: u64) -> i64 {
    match state.slot(ActivitySlotId::new(slot_id).unwrap()) {
        Some(ActivityValue::BoundedCounterMap(values)) => values
            .binary_search_by_key(&key, |(candidate, _)| *candidate)
            .ok()
            .map_or(0, |index| values[index].1),
        value => panic!("unexpected counter slot: {value:?}"),
    }
}
