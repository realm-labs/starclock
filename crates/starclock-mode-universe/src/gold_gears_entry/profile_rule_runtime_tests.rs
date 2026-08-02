use starclock_activity::{
    ActivityCause, ActivityConfigDigest, ActivityDefinitionDigest, ActivityDefinitionId,
    ActivityDefinitionIdentity, ActivityInstanceId, ActivityInventoryId, ActivityMasterSeed,
    ActivityRngContext, ActivityRngLabel, ActivityRngStreams, ActivitySlotId,
    ActivityTransactionOutcome, ActivityTransactionState, ActivityValue,
};

use super::{
    GoldAndGearsEntryError, GoldAndGearsRuntimeFactory, GoldAndGearsRuntimeInstance,
    curio_types::GoldAndGearsCurioCategory,
    profile_rule_runtime::{
        GOLD_AND_GEARS_PROFILE_RULE_RUNTIME_REVISION, GoldAndGearsProfileRuleExecution,
    },
    state_layout::{
        BLESSING_INVENTORY, CURIO_INVENTORY, RESOURCE_COSMIC_FRAGMENTS_KEY,
        RESOURCE_DICE_CHEATS_KEY, RUN_RESOURCES_SLOT,
    },
    tests::entry,
};

#[test]
fn profile_partition_binds_exactly_five_exact_public_activity_rules() {
    let factory = factory();
    assert_eq!(
        GOLD_AND_GEARS_PROFILE_RULE_RUNTIME_REVISION,
        "gold-and-gears-profile-entry-rule-runtime-v1"
    );
    let expected = [
        (
            "gold-gears.trailblaze-bonus.201",
            "gold-gears.rule.trailblaze-bonus.201",
            3010,
        ),
        (
            "gold-gears.trailblaze-bonus.202",
            "gold-gears.rule.trailblaze-bonus.202",
            3020,
        ),
        (
            "gold-gears.trailblaze-bonus.203",
            "gold-gears.rule.trailblaze-bonus.203",
            3030,
        ),
        (
            "gold-gears.trailblaze-bonus.204",
            "gold-gears.rule.trailblaze-bonus.204",
            3040,
        ),
        (
            "gold-gears.trailblaze-bonus.205",
            "gold-gears.rule.trailblaze-bonus.205",
            3050,
        ),
    ];
    for (bonus, rule, event) in expected {
        let instance = compile(factory, bonus);
        let plan = instance.trailblaze_bonus_plan().unwrap();
        assert_eq!(
            (plan.source_bonus(), plan.source_rule(), plan.event_id()),
            (bonus, rule, event)
        );
    }
}

#[test]
fn profile_entry_fixture_executes_all_five_rules_against_production_state() {
    let factory = factory();
    let mut observed = Vec::new();
    for source in 201..=205 {
        let instance = compile(factory, &format!("gold-gears.trailblaze-bonus.{source}"));
        let mut state = ActivityTransactionState::new(
            instance.state_definition().clone(),
            instance.graph_definition().entry(),
        );
        let mut rng = activity_rng(&instance, 0);
        let before_rng = rng.snapshots();
        let execution = execute_profile_rule(&instance, &mut state, &[], &[], &mut rng).unwrap();
        assert_eq!(
            execution.source_rule(),
            format!("gold-gears.rule.trailblaze-bonus.{source}")
        );
        assert!(!execution.program().operations().is_empty());
        let expected_draws = match source {
            201 | 204 => 0,
            202 | 203 => 1,
            205 => 2,
            _ => unreachable!(),
        };
        assert_only_reward_advanced(&before_rng, &rng.snapshots(), expected_draws);
        assert_rule_effect(&instance, &state, &execution, source);
        observed.push((
            source,
            execution
                .selected_blessings()
                .iter()
                .map(|id| id.get())
                .collect::<Vec<_>>(),
            execution
                .selected_curios()
                .iter()
                .map(|id| {
                    instance
                        .curio_definitions()
                        .iter()
                        .find(|definition| definition.id() == *id)
                        .unwrap()
                        .source_id()
                })
                .collect::<Vec<_>>(),
            state_hash(&instance, &state, &rng),
        ));
    }
    assert_eq!(
        observed,
        vec![
            (
                201,
                vec![],
                vec![],
                "067b7303789e7aa63ca87e94d94893d8715be66d0caaa38381b66c3d80653ac8".to_owned(),
            ),
            (
                202,
                vec![28],
                vec![],
                "6676ef381d7684891b843c2471232cac37c9bde11cc094bfbb2226d848d64d51".to_owned(),
            ),
            (
                203,
                vec![],
                vec![209],
                "80760e9ef4bc5e92dfca966808b62270fe0723606a527b518b9ffe2f02480fe1".to_owned(),
            ),
            (
                204,
                vec![],
                vec![],
                "88984812bb6a13a9ace8a27faf6fc7c8059fc2b492ea336d48854d77b5a690ed".to_owned(),
            ),
            (
                205,
                vec![],
                vec![214, 53],
                "5f249c83022dd9f49b848d4effecd60bb9fc0795cd51996720ab72c7838eb6f3".to_owned(),
            ),
        ]
    );
}

#[test]
fn duplicate_and_stale_profile_rule_execution_preserve_state_and_rng() {
    let factory = factory();
    let instance = compile(factory, "gold-gears.trailblaze-bonus.202");
    let mut state = ActivityTransactionState::new(
        instance.state_definition().clone(),
        instance.graph_definition().entry(),
    );
    let mut rng = activity_rng(&instance, 17);
    let execution = execute_profile_rule(&instance, &mut state, &[], &[], &mut rng).unwrap();
    let owned = [(execution.selected_blessings()[0], 1)];
    let before = state_bytes(&instance, &state, &rng);
    assert_eq!(
        execute_profile_rule(&instance, &mut state, &owned, &[], &mut rng),
        Err(GoldAndGearsEntryError::ProfileEntryRuleAlreadyApplied)
    );
    assert_eq!(state_bytes(&instance, &state, &rng), before);

    let stale_instance = compile(factory, "gold-gears.trailblaze-bonus.202");
    let mut stale_state = ActivityTransactionState::new(
        stale_instance.state_definition().clone(),
        stale_instance.graph_definition().entry(),
    );
    let already_owned = stale_instance
        .select_trailblaze_blessing(&[], &mut activity_rng(&stale_instance, 99))
        .unwrap()
        .unwrap();
    commit(
        &stale_instance,
        &mut stale_state,
        stale_instance
            .compile_blessing_acquisition(already_owned)
            .unwrap(),
    );
    let mut stale_rng = activity_rng(&stale_instance, 23);
    let before_stale = state_bytes(&stale_instance, &stale_state, &stale_rng);
    assert_eq!(
        execute_profile_rule(&stale_instance, &mut stale_state, &[], &[], &mut stale_rng),
        Err(GoldAndGearsEntryError::ProfileEntryStateMismatch)
    );
    assert_eq!(
        state_bytes(&stale_instance, &stale_state, &stale_rng),
        before_stale
    );
}

fn assert_rule_effect(
    instance: &GoldAndGearsRuntimeInstance,
    state: &ActivityTransactionState,
    execution: &GoldAndGearsProfileRuleExecution,
    source: u32,
) {
    match source {
        201 => {
            assert_eq!(
                counter(state, RUN_RESOURCES_SLOT, RESOURCE_COSMIC_FRAGMENTS_KEY),
                250
            );
            assert!(execution.selected_blessings().is_empty());
            assert!(execution.selected_curios().is_empty());
        }
        202 => {
            assert_eq!(execution.selected_blessings().len(), 1);
            assert_eq!(
                inventory_count(
                    instance,
                    state,
                    BLESSING_INVENTORY,
                    execution.selected_blessings()[0].get()
                ),
                1
            );
        }
        203 => {
            assert_eq!(execution.selected_curios().len(), 1);
            assert_eq!(
                curio_category(instance, execution.selected_curios()[0]),
                GoldAndGearsCurioCategory::Normal
            );
        }
        204 => assert_eq!(
            counter(state, RUN_RESOURCES_SLOT, RESOURCE_DICE_CHEATS_KEY),
            1
        ),
        205 => {
            assert_eq!(execution.selected_curios().len(), 2);
            assert_eq!(
                execution
                    .selected_curios()
                    .iter()
                    .map(|id| curio_category(instance, *id))
                    .collect::<Vec<_>>(),
                [
                    GoldAndGearsCurioCategory::Negative,
                    GoldAndGearsCurioCategory::ErrorCode
                ]
            );
            for id in execution.selected_curios() {
                assert_eq!(
                    inventory_count(instance, state, CURIO_INVENTORY, id.get()),
                    1
                );
            }
        }
        _ => unreachable!(),
    }
}

fn factory() -> &'static GoldAndGearsRuntimeFactory {
    super::tests::shared_factory()
}

fn compile(factory: &GoldAndGearsRuntimeFactory, bonus: &str) -> GoldAndGearsRuntimeInstance {
    let selected = entry(
        factory,
        "gold-gears.area.405",
        &factory.unique.paths[0].identity.stable_key,
        &factory.unique.dice[0],
    )
    .with_neural_network(
        factory
            .unique
            .neural_nodes
            .iter()
            .map(|node| node.identity.stable_key.to_string())
            .collect(),
    )
    .with_trailblaze_bonus(bonus);
    factory.compile_entry(selected).unwrap()
}

fn commit(
    instance: &GoldAndGearsRuntimeInstance,
    state: &mut ActivityTransactionState,
    program: starclock_activity::ActivityProgramDefinition,
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

fn execute_profile_rule(
    instance: &GoldAndGearsRuntimeInstance,
    state: &mut ActivityTransactionState,
    blessings: &[(crate::id::BlessingId, u32)],
    curios: &[(super::GoldAndGearsCurioId, u32)],
    rng: &mut ActivityRngStreams,
) -> Result<GoldAndGearsProfileRuleExecution, GoldAndGearsEntryError> {
    rng.transact(|working| {
        let execution = instance.compile_profile_entry_rule(state, blessings, curios, working)?;
        let cause = ActivityCause::new(
            state
                .command_sequence()
                .checked_add(1)
                .ok_or(GoldAndGearsEntryError::InvalidActivityState)?,
            execution.program().id(),
            state.current_node(),
        )
        .ok_or(GoldAndGearsEntryError::InvalidActivityState)?;
        match state.apply_program(execution.program(), cause, instance.graph_definition()) {
            ActivityTransactionOutcome::Committed(events) if !events.is_empty() => Ok(execution),
            ActivityTransactionOutcome::Rejected(_) => {
                Err(GoldAndGearsEntryError::ProfileEntryStateMismatch)
            }
            ActivityTransactionOutcome::Faulted(_, _) => {
                Err(GoldAndGearsEntryError::InvalidActivityState)
            }
            ActivityTransactionOutcome::Committed(_) => {
                Err(GoldAndGearsEntryError::InvalidProfileEntryRule)
            }
        }
    })
}

fn curio_category(
    instance: &GoldAndGearsRuntimeInstance,
    id: super::GoldAndGearsCurioId,
) -> GoldAndGearsCurioCategory {
    instance
        .curio_definitions()
        .iter()
        .find(|definition| definition.id() == id)
        .unwrap()
        .category()
}

fn inventory_count(
    instance: &GoldAndGearsRuntimeInstance,
    state: &ActivityTransactionState,
    inventory: u32,
    content: u32,
) -> u32 {
    let rng = activity_rng(instance, 0);
    state
        .player_view(
            identity(),
            instance.graph_definition(),
            ActivityInstanceId::new(1).unwrap(),
            &rng,
        )
        .inventories()
        .iter()
        .find(|view| view.id() == ActivityInventoryId::new(inventory).unwrap())
        .unwrap()
        .entries()
        .iter()
        .find(|(candidate, _)| *candidate == u64::from(content))
        .map_or(0, |(_, count)| *count)
}

fn counter(state: &ActivityTransactionState, slot: u32, key: u64) -> i64 {
    let Some(ActivityValue::BoundedCounterMap(values)) =
        state.slot(ActivitySlotId::new(slot).unwrap())
    else {
        panic!("expected counter map");
    };
    values
        .binary_search_by_key(&key, |(candidate, _)| *candidate)
        .ok()
        .map_or(0, |index| values[index].1)
}

fn activity_rng(instance: &GoldAndGearsRuntimeInstance, seed: u64) -> ActivityRngStreams {
    let identity = identity();
    ActivityRngStreams::new(ActivityRngContext::new(
        ActivityMasterSeed::from_u64(seed),
        identity.id(),
        identity.definition_digest(),
        identity.config_digest(),
        instance.graph_definition().digest(),
        ActivityInstanceId::new(1).unwrap(),
        None,
        Some(instance.graph_definition().entry()),
        None,
        0,
    ))
}

fn identity() -> ActivityDefinitionIdentity {
    ActivityDefinitionIdentity::new(
        ActivityDefinitionId::new(14).unwrap(),
        ActivityDefinitionDigest::new([0x14; 32]).unwrap(),
        ActivityConfigDigest::new([0x47; 32]).unwrap(),
    )
}

fn state_hash(
    instance: &GoldAndGearsRuntimeInstance,
    state: &ActivityTransactionState,
    rng: &ActivityRngStreams,
) -> String {
    state
        .state_hash(
            identity(),
            instance.graph_definition(),
            ActivityInstanceId::new(1).unwrap(),
            rng,
        )
        .bytes()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn state_bytes(
    instance: &GoldAndGearsRuntimeInstance,
    state: &ActivityTransactionState,
    rng: &ActivityRngStreams,
) -> Box<[u8]> {
    state.canonical_state_bytes(
        identity(),
        instance.graph_definition(),
        ActivityInstanceId::new(1).unwrap(),
        rng,
    )
}

fn assert_only_reward_advanced(
    before: &[starclock_activity::ActivityRngStreamSnapshot],
    after: &[starclock_activity::ActivityRngStreamSnapshot],
    draws: u64,
) {
    for (old, new) in before.iter().zip(after) {
        assert_eq!(new.seed(), old.seed());
        assert_eq!(
            new.draw_count(),
            old.draw_count()
                + if old.label() == ActivityRngLabel::Reward {
                    draws
                } else {
                    0
                }
        );
    }
}
