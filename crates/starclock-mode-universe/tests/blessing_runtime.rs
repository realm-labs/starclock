use std::sync::{Arc, OnceLock};

use starclock_activity::{
    ActivityCause, ActivityCondition, ActivityDecisionKind, ActivityEdgeCondition,
    ActivityEdgeDefinition, ActivityEdgeId, ActivityGraphDefinition, ActivityInventoryDefinition,
    ActivityInventoryId, ActivityNodeDefinition, ActivityNodeKind, ActivityOperation,
    ActivityOptionDefinition, ActivityOptionId, ActivityProgramDefinition, ActivityProgramId,
    ActivityScope, ActivityStateDefinition, ActivityStateSource, ActivityStateVisibility,
    ActivityTransactionOutcome, ActivityTransactionRejection, ActivityTransactionState, NodeId,
    SectionId, SlotCarryPolicy,
};
use starclock_mode_universe::{
    blessing_runtime::{
        BLESSING_RUNTIME_REVISION, BlessingOfferEligibility, BlessingRuntimeCatalog,
    },
    catalog::UniverseCatalog,
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
fn all_blessings_compile_to_two_exact_typed_contributions() {
    let runtime = BlessingRuntimeCatalog::compile(&catalog()).expect("Blessing runtime");
    assert_eq!(runtime.definitions().len(), 162);
    assert_eq!(
        BLESSING_RUNTIME_REVISION,
        "standard-universe-blessing-runtime-v1"
    );
    assert!(runtime.definitions().iter().all(|definition| {
        (1..=3).contains(&definition.rarity())
            && definition.level(1).is_some()
            && definition.level(2).is_some()
            && definition.level(3).is_none()
            && !definition.level(1).unwrap().rule_key().is_empty()
            && !definition.level(2).unwrap().source_binding_key().is_empty()
    }));
    assert_eq!(
        runtime.digest(),
        [
            230, 112, 214, 65, 157, 255, 188, 68, 26, 24, 170, 148, 101, 22, 70, 107, 202, 98, 242,
            133, 5, 146, 27, 191, 150, 35, 63, 101, 131, 61, 54, 145,
        ]
    );

    let locked = BlessingOfferEligibility::explicit(vec![3], vec![]).unwrap();
    let locked_candidates = runtime.eligible(&locked).collect::<Vec<_>>();
    assert!(!locked_candidates.is_empty());
    assert!(locked_candidates
        .iter()
        .all(|definition| definition.rarity() == 3 && definition.prerequisite_keys().is_empty()));
    let gated = runtime
        .definitions()
        .iter()
        .find(|definition| !definition.prerequisite_keys().is_empty())
        .unwrap();
    let unlocked = BlessingOfferEligibility::explicit(
        vec![gated.rarity()],
        gated.prerequisite_keys().to_vec(),
    )
    .unwrap();
    assert!(unlocked.allows(gated));
}

#[test]
fn acquire_enhance_and_replace_are_generic_atomic_inventory_programs() {
    let runtime = BlessingRuntimeCatalog::compile(&catalog()).expect("Blessing runtime");
    let removed = runtime.definitions()[0].blessing();
    let acquired = runtime.definitions()[1].blessing();
    let inventory = inventory(1);
    let graph = graph();
    let definition = ActivityStateDefinition::new(
        vec![],
        vec![
            ActivityInventoryDefinition::new(
                inventory,
                ActivityScope::Activity,
                162,
                2,
                SlotCarryPolicy::CarryExact,
                ActivityStateVisibility::Player,
                ActivityStateSource::new(1).unwrap(),
            )
            .unwrap(),
        ],
        vec![],
    )
    .unwrap();
    let mut state = ActivityTransactionState::new(definition.clone(), node(1));

    let acquire = offer_program(
        1,
        runtime
            .acquisition_option(removed, option(1), 0, inventory, vec![])
            .unwrap(),
    );
    acquire.validate_against(&definition, &graph).unwrap();
    assert!(matches!(
        state.apply_program(&acquire, cause(1, 1), &graph),
        ActivityTransactionOutcome::Committed(_)
    ));
    assert!(matches!(
        state.apply_option(option(1), cause(2, 1), &graph),
        ActivityTransactionOutcome::Committed(_)
    ));

    let enhance = offer_program(
        2,
        ActivityOptionDefinition::new(
            option(2),
            0,
            always(),
            runtime
                .enhancement_operations(inventory, removed)
                .unwrap()
                .into_vec(),
        ),
    );
    enhance.validate_against(&definition, &graph).unwrap();
    assert!(matches!(
        state.apply_program(&enhance, cause(3, 2), &graph),
        ActivityTransactionOutcome::Committed(_)
    ));
    assert!(matches!(
        state.apply_option(option(2), cause(4, 2), &graph),
        ActivityTransactionOutcome::Committed(_)
    ));

    let replace = offer_program(
        3,
        ActivityOptionDefinition::new(
            option(3),
            0,
            always(),
            runtime
                .replacement_operations(inventory, removed, acquired)
                .unwrap()
                .into_vec(),
        ),
    );
    replace.validate_against(&definition, &graph).unwrap();
    assert!(matches!(
        state.apply_program(&replace, cause(5, 3), &graph),
        ActivityTransactionOutcome::Committed(_)
    ));
    assert!(matches!(
        state.apply_option(option(3), cause(6, 3), &graph),
        ActivityTransactionOutcome::Committed(_)
    ));

    assert!(matches!(
        state.apply_program(&enhance, cause(7, 2), &graph),
        ActivityTransactionOutcome::Committed(_)
    ));
    assert_eq!(
        state.apply_option(option(2), cause(8, 2), &graph),
        ActivityTransactionOutcome::Rejected(ActivityTransactionRejection::ConditionNotSatisfied)
    );
}

fn offer_program(raw: u32, option: ActivityOptionDefinition) -> ActivityProgramDefinition {
    ActivityProgramDefinition::new(
        program(raw),
        vec![ActivityOperation::Offer {
            kind: ActivityDecisionKind::Service,
            options: vec![option].into_boxed_slice(),
        }],
    )
    .unwrap()
}

fn graph() -> ActivityGraphDefinition {
    ActivityGraphDefinition::new(
        node(1),
        vec![
            ActivityNodeDefinition::new(node(1), section(1), ActivityNodeKind::Choice, 32).unwrap(),
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
        32,
    )
    .unwrap()
}

fn always() -> ActivityCondition {
    ActivityCondition::Boolean(starclock_activity::ActivityExpression::Literal(
        starclock_activity::ActivityValue::Boolean(true),
    ))
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
fn inventory(raw: u32) -> ActivityInventoryId {
    ActivityInventoryId::new(raw).unwrap()
}
fn option(raw: u64) -> ActivityOptionId {
    ActivityOptionId::new(raw).unwrap()
}
fn program(raw: u32) -> ActivityProgramId {
    ActivityProgramId::new(raw).unwrap()
}
