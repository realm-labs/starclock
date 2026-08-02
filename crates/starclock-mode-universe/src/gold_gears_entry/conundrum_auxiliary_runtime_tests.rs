use starclock_activity::{
    ActivityCause, ActivityConfigDigest, ActivityDefinitionDigest, ActivityDefinitionId,
    ActivityDefinitionIdentity, ActivityInstanceId, ActivityMasterSeed, ActivityRngContext,
    ActivityRngLabel, ActivityRngStreams, ActivitySlotId, ActivityTransactionOutcome,
    ActivityTransactionState, ActivityValue,
};

use super::{
    CONUNDRUM_AREA_KEY, GoldAndGearsEntryError, GoldAndGearsRuntimeFactory,
    GoldAndGearsRuntimeInstance,
    conundrum_auxiliary_runtime::{
        GOLD_AND_GEARS_AUXILIARY_CONUNDRUM_RULE_REVISION,
        GoldAndGearsAuxiliaryBattleContribution, GoldAndGearsAuxiliaryPlaneEntryExecution,
    },
    state_layout::{
        DEFERRED_CONUNDRUM_PLANE_APPLIED_BASE, DEFERRED_CONUNDRUM_RULE_APPLIED_BASE,
        DEFERRED_CONUNDRUM_RULE_VALUE_BASE, DEFERRED_EFFECTS_SLOT, RESOURCE_COSMIC_FRAGMENTS_KEY,
        RESOURCE_DICE_REROLLS_KEY, RUN_RESOURCES_SLOT,
    },
    tests::entry,
};

const PATH: &str = "universe.path.preservation";

#[test]
fn auxiliary_partition_binds_exactly_six_cumulative_exact_public_rules() {
    let factory = super::tests::shared_factory();
    assert_eq!(
        GOLD_AND_GEARS_AUXILIARY_CONUNDRUM_RULE_REVISION,
        "gold-and-gears-auxiliary-conundrum-rule-runtime-v1"
    );
    assert!(
        compile(factory, 0)
            .compile_auxiliary_conundrum_rules(&new_state(&compile(factory, 0)))
            .unwrap()
            .is_none()
    );
    for level in 1..=6 {
        let instance = compile(factory, level);
        let state = new_state(&instance);
        let execution = instance
            .compile_auxiliary_conundrum_rules(&state)
            .unwrap()
            .unwrap();
        assert_eq!(execution.source_rules().len(), usize::from(level));
        assert_eq!(
            execution
                .source_rules()
                .iter()
                .map(Box::as_ref)
                .collect::<Vec<_>>(),
            (1..=level)
                .map(|value| format!("gold-gears.rule.conundrum.auxiliary.{value}"))
                .collect::<Vec<_>>()
        );
        assert!(!execution.program().operations().is_empty());
        let expected_battle =
            usize::from(level >= 1) + usize::from(level >= 2) + usize::from(level >= 6);
        assert_eq!(execution.battle_contributions().len(), expected_battle);
    }
}

#[test]
fn cumulative_start_program_executes_all_six_rule_payloads_without_rng() {
    let factory = super::tests::shared_factory();
    let instance = compile(factory, 6);
    let mut state = new_state(&instance);
    let rng = activity_rng(&instance, 0);
    let before_rng = rng.snapshots();
    let execution = instance
        .compile_auxiliary_conundrum_rules(&state)
        .unwrap()
        .unwrap();
    commit(&instance, &mut state, execution.program());
    assert_eq!(rng.snapshots(), before_rng);

    for level in 1..=6 {
        assert_eq!(
            counter(
                &state,
                DEFERRED_EFFECTS_SLOT,
                DEFERRED_CONUNDRUM_RULE_APPLIED_BASE + level
            ),
            1
        );
    }
    assert_eq!(rule_value(&state, 1, 0), 1);
    assert_eq!(rule_value(&state, 2, 0), 12);
    assert_eq!(rule_value(&state, 3, 0), 20);
    assert_eq!(
        [
            rule_value(&state, 4, 0),
            rule_value(&state, 4, 1),
            rule_value(&state, 4, 2)
        ],
        [1, 1, 100]
    );
    assert_eq!(rule_value(&state, 5, 0), 1);
    assert_eq!([rule_value(&state, 6, 0), rule_value(&state, 6, 1)], [1, 0]);
    assert_eq!(
        counter(&state, RUN_RESOURCES_SLOT, RESOURCE_COSMIC_FRAGMENTS_KEY),
        0
    );
    assert_eq!(
        counter(&state, RUN_RESOURCES_SLOT, RESOURCE_DICE_REROLLS_KEY),
        0
    );

    let battle = execution.battle_contributions();
    assert!(matches!(
        &battle[0],
        GoldAndGearsAuxiliaryBattleContribution::ThirdPlaneFormationExtrapolation { count: 1, .. }
    ));
    assert!(matches!(
        &battle[1],
        GoldAndGearsAuxiliaryBattleContribution::SecondPlaneBossPhaseThree {
            encounter_groups,
            ..
        } if encounter_groups.len() == 12
    ));
    assert!(matches!(
        &battle[2],
        GoldAndGearsAuxiliaryBattleContribution::EffectiveBlessingsPerPath {
            delta: -1,
            minimum: 0,
            ..
        }
    ));
    assert_eq!(
        state_hash(&instance, &state, &rng),
        "4581fc1940dcb845a5cc1cd52ca162fe1d74c27f496079bfab9b192df1bb3176"
    );
}

#[test]
fn plane_entry_rule_grants_one_negative_curio_per_plane_on_reward_stream() {
    let factory = super::tests::shared_factory();
    let instance = compile(factory, 6);
    let mut state = new_state(&instance);
    let mut rng = activity_rng(&instance, 0);
    let before = rng.snapshots();
    let mut owned = Vec::new();
    let mut selected_sources = Vec::new();
    for plane in 1..=3 {
        let execution = execute_plane(&instance, &mut state, plane, &owned, &mut rng).unwrap();
        assert_eq!(
            execution.source_rule(),
            "gold-gears.rule.conundrum.auxiliary.5"
        );
        assert_eq!(execution.plane_layer(), plane);
        let id = execution.selected_curio();
        let definition = instance
            .curio_definitions()
            .iter()
            .find(|definition| definition.id() == id)
            .unwrap();
        assert_eq!(
            definition.category(),
            super::GoldAndGearsCurioCategory::Negative
        );
        selected_sources.push(definition.source_id());
        owned.push((id, 1));
        owned.sort_unstable_by_key(|entry| entry.0);
        assert_eq!(
            counter(
                &state,
                DEFERRED_EFFECTS_SLOT,
                DEFERRED_CONUNDRUM_PLANE_APPLIED_BASE + u64::from(plane)
            ),
            1
        );
    }
    assert_eq!(selected_sources, [214, 215, 70]);
    assert_only_reward_advanced(&before, &rng.snapshots(), 3);
    assert_eq!(
        state_hash(&instance, &state, &rng),
        "54518db2db46b29e3e7a16c4604cd4d393d6e3c346a8e9d2d632dfc9319daa63"
    );
}

#[test]
fn duplicate_and_stale_auxiliary_execution_preserve_state_and_rng() {
    let factory = super::tests::shared_factory();
    let instance = compile(factory, 6);
    let mut state = new_state(&instance);
    let mut rng = activity_rng(&instance, 23);
    let first = execute_plane(&instance, &mut state, 1, &[], &mut rng).unwrap();
    let owned = [(first.selected_curio(), 1)];
    let before_duplicate = state_bytes(&instance, &state, &rng);
    assert_eq!(
        execute_plane(&instance, &mut state, 1, &owned, &mut rng),
        Err(GoldAndGearsEntryError::AuxiliaryConundrumRuleAlreadyApplied)
    );
    assert_eq!(state_bytes(&instance, &state, &rng), before_duplicate);

    let stale_instance = compile(factory, 6);
    let mut stale_state = new_state(&stale_instance);
    let mut selection_rng = activity_rng(&stale_instance, 7);
    let stale = stale_instance
        .compile_auxiliary_conundrum_plane_entry(&stale_state, 1, &[], &mut selection_rng)
        .unwrap();
    commit(
        &stale_instance,
        &mut stale_state,
        &stale_instance
            .compile_curio_acquisition(stale.selected_curio())
            .unwrap(),
    );
    let mut stale_rng = activity_rng(&stale_instance, 7);
    let before_stale = state_bytes(&stale_instance, &stale_state, &stale_rng);
    assert_eq!(
        execute_plane(&stale_instance, &mut stale_state, 1, &[], &mut stale_rng),
        Err(GoldAndGearsEntryError::AuxiliaryConundrumStateMismatch)
    );
    assert_eq!(
        state_bytes(&stale_instance, &stale_state, &stale_rng),
        before_stale
    );
}

fn compile(factory: &GoldAndGearsRuntimeFactory, auxiliary: u8) -> GoldAndGearsRuntimeInstance {
    let dice = &factory.unique.dice[0];
    factory
        .compile_entry(
            entry(factory, CONUNDRUM_AREA_KEY, PATH, dice).with_conundrum(
                0,
                auxiliary,
                vec![CONUNDRUM_AREA_KEY.to_owned()],
            ),
        )
        .unwrap()
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
    program: &starclock_activity::ActivityProgramDefinition,
) {
    let cause = ActivityCause::new(
        state.command_sequence() + 1,
        program.id(),
        state.current_node(),
    )
    .unwrap();
    assert!(matches!(
        state.apply_program(program, cause, instance.graph_definition()),
        ActivityTransactionOutcome::Committed(events) if !events.is_empty()
    ));
}

fn execute_plane(
    instance: &GoldAndGearsRuntimeInstance,
    state: &mut ActivityTransactionState,
    plane: u8,
    owned: &[(super::GoldAndGearsCurioId, u32)],
    rng: &mut ActivityRngStreams,
) -> Result<GoldAndGearsAuxiliaryPlaneEntryExecution, GoldAndGearsEntryError> {
    rng.transact(|working| {
        let execution =
            instance.compile_auxiliary_conundrum_plane_entry(state, plane, owned, working)?;
        let cause = ActivityCause::new(
            state.command_sequence() + 1,
            execution.program().id(),
            state.current_node(),
        )
        .ok_or(GoldAndGearsEntryError::InvalidActivityState)?;
        match state.apply_program(execution.program(), cause, instance.graph_definition()) {
            ActivityTransactionOutcome::Committed(events) if !events.is_empty() => Ok(execution),
            ActivityTransactionOutcome::Rejected(_) => {
                Err(GoldAndGearsEntryError::AuxiliaryConundrumStateMismatch)
            }
            _ => Err(GoldAndGearsEntryError::InvalidActivityState),
        }
    })
}

fn rule_value(state: &ActivityTransactionState, level: u64, index: u64) -> i64 {
    counter(
        state,
        DEFERRED_EFFECTS_SLOT,
        DEFERRED_CONUNDRUM_RULE_VALUE_BASE + level * 16 + index,
    )
}

fn counter(state: &ActivityTransactionState, slot: u32, key: u64) -> i64 {
    let Some(ActivityValue::BoundedCounterMap(values)) =
        state.slot(ActivitySlotId::new(slot).unwrap())
    else {
        panic!("expected counter map");
    };
    values
        .binary_search_by_key(&key, |entry| entry.0)
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
