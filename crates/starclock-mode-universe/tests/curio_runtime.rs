use std::sync::{Arc, OnceLock};

use starclock_activity::{
    ActivityCause, ActivityCondition, ActivityDecisionKind, ActivityEdgeCondition,
    ActivityEdgeDefinition, ActivityEdgeId, ActivityGraphDefinition, ActivityInventoryDefinition,
    ActivityInventoryId, ActivityNodeDefinition, ActivityNodeKind, ActivityOperation,
    ActivityOptionDefinition, ActivityOptionId, ActivityProgramDefinition, ActivityProgramId,
    ActivityScope, ActivitySlotDefinition, ActivitySlotId, ActivityStateDefinition,
    ActivityStateSource, ActivityStateVisibility, ActivityTransactionOutcome,
    ActivityTransactionRejection, ActivityTransactionState, ActivityValue, NodeId, SectionId,
    SlotCarryPolicy, SlotResetPoint,
};
use starclock_mode_universe::{
    catalog::UniverseCatalog,
    curio::CurioStateKind,
    curio_runtime::{
        CURIO_RUNTIME_REVISION, CurioRuntimeBindings, CurioRuntimeCatalog, CurioRuntimeDefinition,
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

#[test]
fn all_curios_compile_with_exact_lifecycle_denominators() {
    let runtime = CurioRuntimeCatalog::compile(&catalog()).expect("Curio runtime");
    assert_eq!(CURIO_RUNTIME_REVISION, "standard-universe-curio-runtime-v1");
    assert_eq!(runtime.definitions().len(), 61);
    assert_eq!(
        runtime
            .definitions()
            .iter()
            .map(|definition| definition.states().len())
            .sum::<usize>(),
        67
    );
    assert_eq!(states(&runtime, CurioStateKind::Active), 55);
    assert_eq!(states(&runtime, CurioStateKind::Repairing), 6);
    assert_eq!(states(&runtime, CurioStateKind::Fixed), 6);
    assert_eq!(
        runtime
            .definitions()
            .iter()
            .flat_map(CurioRuntimeDefinition::states)
            .filter(|state| state.maximum_charges().is_some())
            .count(),
        9
    );
    assert_eq!(
        runtime
            .definitions()
            .iter()
            .filter(|definition| {
                definition
                    .pool_tags()
                    .iter()
                    .any(|tag| tag.as_ref() == "polarity:negative")
            })
            .count(),
        15
    );
    assert_eq!(
        runtime
            .definitions()
            .iter()
            .flat_map(CurioRuntimeDefinition::states)
            .filter(|state| state.replacement_curio().is_some())
            .count(),
        0
    );
    assert_eq!(
        runtime.digest(),
        [
            11, 244, 17, 218, 8, 39, 22, 106, 235, 230, 30, 47, 193, 243, 144, 68, 231, 235, 103,
            153, 72, 20, 42, 123, 92, 148, 207, 65, 124, 0, 82, 120,
        ]
    );
}

#[test]
fn acquisition_is_unique_and_charge_exhaustion_repairs_error_code() {
    let runtime = CurioRuntimeCatalog::compile(&catalog()).expect("Curio runtime");
    let repairing = runtime
        .definitions()
        .iter()
        .find(|definition| {
            definition
                .states()
                .iter()
                .any(|state| state.kind() == CurioStateKind::Repairing)
        })
        .expect("repairing Curio");
    let initial = repairing
        .states()
        .iter()
        .find(|state| state.id() == repairing.initial_state())
        .expect("initial state");
    let fixed = initial.next_state().expect("fixed transition");
    let initial_contribution = runtime
        .contributions_from_owned(
            &[(repairing.curio(), 1)],
            &[(repairing.curio(), initial.id())],
            &[(repairing.curio(), 3)],
        )
        .expect("repairing contribution");
    assert_eq!(
        initial_contribution.digest(),
        [
            76, 77, 201, 216, 220, 14, 103, 162, 185, 127, 160, 16, 216, 190, 217, 109, 219, 188,
            77, 31, 247, 232, 36, 198, 6, 187, 144, 62, 125, 148, 61, 62,
        ]
    );
    let bindings = bindings();
    let definition = state_definition(bindings);
    let graph = graph();
    let mut state = ActivityTransactionState::new(definition.clone(), node(1));

    let acquire = offer_program(
        1,
        runtime
            .acquisition_option(repairing.curio(), option(1), 0, bindings, vec![])
            .expect("acquisition option"),
    );
    acquire.validate_against(&definition, &graph).unwrap();
    commit_program(&mut state, &acquire, 1, &graph);
    commit_option(&mut state, option(1), 2, &graph);
    assert_counter(
        &state,
        bindings.state_slot,
        repairing.curio().get(),
        initial.id().get(),
    );
    assert_counter(&state, bindings.charge_slot, repairing.curio().get(), 3);

    for (sequence, expected) in [(3, 3), (4, 2), (5, 1)] {
        let operations = runtime
            .consume_charge_operations(repairing.curio(), expected, bindings)
            .expect("consume charge");
        let program = operation_program(sequence, operations.into_vec());
        program.validate_against(&definition, &graph).unwrap();
        commit_program(&mut state, &program, u64::from(sequence), &graph);
    }
    assert_counter(
        &state,
        bindings.state_slot,
        repairing.curio().get(),
        fixed.get(),
    );
    assert_counter(&state, bindings.charge_slot, repairing.curio().get(), 0);

    let rejected = runtime
        .consume_charge_operations(repairing.curio(), 1, bindings)
        .expect("typed stale command");
    let rejected = operation_program(6, rejected.into_vec());
    assert!(matches!(
        state.apply_program(&rejected, cause(6, 6), &graph),
        ActivityTransactionOutcome::Rejected(ActivityTransactionRejection::ConditionNotSatisfied)
    ));
    assert_counter(
        &state,
        bindings.state_slot,
        repairing.curio().get(),
        fixed.get(),
    );

    let duplicate_option = runtime
        .acquisition_option(repairing.curio(), option(7), 0, bindings, vec![])
        .expect("duplicate acquisition option");
    let duplicate = offer_program_many(
        7,
        vec![
            duplicate_option,
            ActivityOptionDefinition::new(option(8), 1, always(), vec![]),
        ],
    );
    duplicate.validate_against(&definition, &graph).unwrap();
    commit_program(&mut state, &duplicate, 6, &graph);
    assert_eq!(
        state.apply_option(option(7), cause(7, 7), &graph),
        ActivityTransactionOutcome::Rejected(ActivityTransactionRejection::UnknownOption)
    );
}

#[test]
fn immediate_repair_replacement_and_teardown_are_atomic_and_scoped() {
    let runtime = CurioRuntimeCatalog::compile(&catalog()).expect("Curio runtime");
    let repairing = runtime
        .definitions()
        .iter()
        .find(|definition| {
            definition
                .states()
                .iter()
                .any(|state| state.kind() == CurioStateKind::Repairing)
        })
        .expect("repairing Curio");
    let replacement = runtime
        .definitions()
        .iter()
        .find(|definition| definition.curio() != repairing.curio())
        .expect("replacement Curio");
    let bindings = bindings();
    let definition = state_definition(bindings);
    let graph = graph();
    let mut state = ActivityTransactionState::new(definition.clone(), node(1));
    acquire(
        &runtime,
        repairing,
        bindings,
        &definition,
        &graph,
        &mut state,
        1,
    );

    let repair = operation_program(
        2,
        runtime
            .repair_operations(repairing.curio(), bindings)
            .unwrap()
            .into_vec(),
    );
    repair.validate_against(&definition, &graph).unwrap();
    commit_program(&mut state, &repair, 3, &graph);
    let fixed = repairing
        .states()
        .iter()
        .find(|state| state.kind() == CurioStateKind::Fixed)
        .unwrap();
    assert_counter(
        &state,
        bindings.state_slot,
        repairing.curio().get(),
        fixed.id().get(),
    );
    assert_counter(&state, bindings.charge_slot, repairing.curio().get(), 0);

    let replace = operation_program(
        3,
        runtime
            .replacement_operations(repairing.curio(), replacement.curio(), bindings)
            .unwrap()
            .into_vec(),
    );
    replace.validate_against(&definition, &graph).unwrap();
    commit_program(&mut state, &replace, 4, &graph);
    assert_counter(&state, bindings.state_slot, repairing.curio().get(), 0);
    assert_counter(
        &state,
        bindings.state_slot,
        replacement.curio().get(),
        replacement.initial_state().get(),
    );

    let contribution = runtime
        .contributions_from_owned(
            &[(replacement.curio(), 1)],
            &[(replacement.curio(), replacement.initial_state())],
            &[],
        )
        .expect("replacement contribution");
    assert_eq!(contribution.entries().len(), 1);
    assert_eq!(contribution.entries()[0].curio(), replacement.curio());
    assert_ne!(contribution.digest(), [0; 32]);

    let teardown = operation_program(
        4,
        runtime
            .teardown_operations(replacement.curio(), bindings)
            .unwrap()
            .into_vec(),
    );
    teardown.validate_against(&definition, &graph).unwrap();
    commit_program(&mut state, &teardown, 5, &graph);
    assert_counter(&state, bindings.state_slot, replacement.curio().get(), 0);
    assert_counter(&state, bindings.charge_slot, replacement.curio().get(), 0);
    let empty = runtime
        .contributions_from_owned(&[], &[], &[])
        .expect("empty scoped contribution");
    assert!(empty.entries().is_empty());
    assert!(
        runtime
            .contributions_from_owned(
                &[],
                &[(replacement.curio(), replacement.initial_state())],
                &[],
            )
            .is_err()
    );
}

fn states(runtime: &CurioRuntimeCatalog, kind: CurioStateKind) -> usize {
    runtime
        .definitions()
        .iter()
        .flat_map(CurioRuntimeDefinition::states)
        .filter(|state| state.kind() == kind)
        .count()
}

fn acquire(
    runtime: &CurioRuntimeCatalog,
    definition: &CurioRuntimeDefinition,
    bindings: CurioRuntimeBindings,
    state_definition: &ActivityStateDefinition,
    graph: &ActivityGraphDefinition,
    state: &mut ActivityTransactionState,
    sequence: u32,
) {
    let program = offer_program(
        sequence,
        runtime
            .acquisition_option(
                definition.curio(),
                option(u64::from(sequence)),
                0,
                bindings,
                vec![],
            )
            .unwrap(),
    );
    program.validate_against(state_definition, graph).unwrap();
    commit_program(state, &program, u64::from(sequence), graph);
    commit_option(
        state,
        option(u64::from(sequence)),
        u64::from(sequence) + 1,
        graph,
    );
}

fn state_definition(bindings: CurioRuntimeBindings) -> ActivityStateDefinition {
    ActivityStateDefinition::new(
        vec![
            counter_slot(bindings.state_slot, i64::from(u32::MAX), 1),
            counter_slot(bindings.charge_slot, 3, 2),
        ],
        vec![
            ActivityInventoryDefinition::new(
                bindings.inventory,
                ActivityScope::Activity,
                61,
                1,
                SlotCarryPolicy::CarryExact,
                ActivityStateVisibility::Player,
                ActivityStateSource::new(3).unwrap(),
            )
            .unwrap(),
        ],
        vec![],
    )
    .unwrap()
}

fn counter_slot(id: ActivitySlotId, maximum: i64, source: u64) -> ActivitySlotDefinition {
    ActivitySlotDefinition::new_with_policy(
        id,
        ActivityScope::Activity,
        ActivityValue::BoundedCounterMap(Box::new([])),
        Some((0, maximum)),
        Some(61),
        vec![SlotResetPoint::ActivityStart],
        SlotCarryPolicy::CarryExact,
        ActivityStateVisibility::Player,
        ActivityStateSource::new(source).unwrap(),
    )
    .unwrap()
}

fn offer_program(raw: u32, option: ActivityOptionDefinition) -> ActivityProgramDefinition {
    offer_program_many(raw, vec![option])
}

fn offer_program_many(
    raw: u32,
    options: Vec<ActivityOptionDefinition>,
) -> ActivityProgramDefinition {
    operation_program(
        raw,
        vec![ActivityOperation::Offer {
            kind: ActivityDecisionKind::Service,
            options: options.into_boxed_slice(),
        }],
    )
}

fn operation_program(raw: u32, operations: Vec<ActivityOperation>) -> ActivityProgramDefinition {
    ActivityProgramDefinition::new(program(raw), operations).unwrap()
}

fn commit_program(
    state: &mut ActivityTransactionState,
    program: &ActivityProgramDefinition,
    sequence: u64,
    graph: &ActivityGraphDefinition,
) {
    let outcome = state.apply_program(program, cause(sequence, program.id().get()), graph);
    assert!(
        matches!(outcome, ActivityTransactionOutcome::Committed(_)),
        "program {} sequence {sequence}: {outcome:?}",
        program.id().get()
    );
}

fn commit_option(
    state: &mut ActivityTransactionState,
    option: ActivityOptionId,
    sequence: u64,
    graph: &ActivityGraphDefinition,
) {
    assert!(matches!(
        state.apply_option(option, cause(sequence, 1), graph),
        ActivityTransactionOutcome::Committed(_)
    ));
}

fn assert_counter(state: &ActivityTransactionState, slot: ActivitySlotId, key: u32, expected: u32) {
    let ActivityValue::BoundedCounterMap(entries) = state.slot(slot).unwrap() else {
        panic!("counter slot");
    };
    let actual = entries
        .binary_search_by_key(&u64::from(key), |entry| entry.0)
        .ok()
        .map_or(0, |index| entries[index].1);
    assert_eq!(actual, i64::from(expected));
}

fn graph() -> ActivityGraphDefinition {
    ActivityGraphDefinition::new(
        node(1),
        vec![
            ActivityNodeDefinition::new(node(1), section(1), ActivityNodeKind::Choice, 128)
                .unwrap(),
            ActivityNodeDefinition::new(
                node(2),
                section(1),
                ActivityNodeKind::Terminal(starclock_activity::ActivityTerminalOutcome::Completed),
                1,
            )
            .unwrap(),
        ],
        vec![
            ActivityEdgeDefinition::new(
                ActivityEdgeId::new(1).unwrap(),
                node(1),
                node(2),
                ActivityEdgeCondition::Always,
                0,
                1,
            )
            .unwrap(),
        ],
        128,
    )
    .unwrap()
}

fn bindings() -> CurioRuntimeBindings {
    CurioRuntimeBindings {
        inventory: ActivityInventoryId::new(1).unwrap(),
        state_slot: ActivitySlotId::new(1).unwrap(),
        charge_slot: ActivitySlotId::new(2).unwrap(),
    }
}

fn cause(sequence: u64, program: u32) -> ActivityCause {
    ActivityCause::new(sequence, self::program(program), node(1)).unwrap()
}
fn node(raw: u32) -> NodeId {
    NodeId::new(raw).unwrap()
}
fn section(raw: u32) -> SectionId {
    SectionId::new(raw).unwrap()
}
fn option(raw: u64) -> ActivityOptionId {
    ActivityOptionId::new(raw).unwrap()
}
fn program(raw: u32) -> ActivityProgramId {
    ActivityProgramId::new(raw).unwrap()
}

fn always() -> ActivityCondition {
    ActivityCondition::Boolean(starclock_activity::ActivityExpression::Literal(
        ActivityValue::Boolean(true),
    ))
}
