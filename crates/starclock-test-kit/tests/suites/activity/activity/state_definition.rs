use starclock_activity::{
    ActivityAccumulationPolicy, ActivityInstanceId, ActivityInventoryDefinition,
    ActivityInventoryId, ActivityModifierDefinition, ActivityModifierId, ActivityModifierOwner,
    ActivityScope, ActivityScopeIdentity, ActivityScopePath, ActivityScopePathError,
    ActivitySlotDefinition, ActivitySlotId, ActivitySnapshotBoundary, ActivityStateDefinition,
    ActivityStateDefinitionError, ActivityStateSource, ActivityStateVisibility, ActivityValue,
    AttemptId, NodeId, SectionId, SlotCarryPolicy, SlotDefinitionError, SlotResetPoint,
};

#[test]
fn scope_path_enters_and_leaves_each_generic_lifetime_in_order() {
    let activity = ActivityInstanceId::new(7).unwrap();
    let root = ActivityScopePath::new(activity);
    assert_eq!(root.active_scope(), ActivityScope::Activity);
    assert_eq!(
        root.enter_node(node(2)).unwrap_err(),
        ActivityScopePathError::MissingSection
    );

    let section_path = root.enter_section(section(1)).unwrap();
    let node_path = section_path.enter_node(node(2)).unwrap();
    let attempt_path = node_path.enter_attempt(attempt(3)).unwrap();
    assert_eq!(attempt_path.active_scope(), ActivityScope::Attempt);
    assert_eq!(
        attempt_path.identity(ActivityScope::Node).unwrap(),
        ActivityScopeIdentity::Node {
            activity,
            section: section(1),
            node: node(2),
        }
    );
    assert_eq!(
        attempt_path.leave_node().unwrap_err(),
        ActivityScopePathError::AttemptStillEntered
    );
    assert_eq!(
        attempt_path
            .leave_attempt()
            .unwrap()
            .leave_node()
            .unwrap()
            .leave_section()
            .unwrap(),
        root
    );
}

#[test]
fn counter_maps_are_sorted_bounded_and_require_explicit_modern_policy() {
    let source = ActivityStateSource::new(9001).unwrap();
    let definition = ActivitySlotDefinition::new_with_policy(
        slot(1),
        ActivityScope::Node,
        ActivityValue::BoundedCounterMap(vec![(10, 2), (20, 4)].into_boxed_slice()),
        Some((0, 9)),
        Some(8),
        vec![SlotResetPoint::NodeStart],
        SlotCarryPolicy::Snapshot(ActivitySnapshotBoundary::NodeExit),
        ActivityStateVisibility::Player,
        source,
    )
    .unwrap();
    assert_eq!(definition.maximum_entries(), Some(8));
    assert_eq!(definition.source(), Some(source));

    let error = ActivitySlotDefinition::new_with_policy(
        slot(2),
        ActivityScope::Node,
        ActivityValue::BoundedCounterMap(vec![(20, 2), (10, 4)].into_boxed_slice()),
        Some((0, 9)),
        Some(8),
        vec![],
        SlotCarryPolicy::CarryExact,
        ActivityStateVisibility::Private,
        source,
    )
    .unwrap_err();
    assert_eq!(error, SlotDefinitionError::InvalidInitialValue);

    let error = ActivitySlotDefinition::new_with_policy(
        slot(3),
        ActivityScope::Attempt,
        ActivityValue::BoundedCounterMap(Vec::new().into_boxed_slice()),
        Some((0, 9)),
        Some(8),
        vec![],
        SlotCarryPolicy::Snapshot(ActivitySnapshotBoundary::SectionExit),
        ActivityStateVisibility::DebugOnly,
        source,
    )
    .unwrap_err();
    assert_eq!(error, SlotDefinitionError::SnapshotBeforeOwnerExit);
}

#[test]
fn inventories_and_modifier_ownership_are_bounded_and_reference_closed() {
    let inventory = inventory(2);
    let modifier = ActivityModifierDefinition::new(
        modifier_id(3),
        ActivityModifierOwner::Inventory(inventory.id()),
        77,
        99,
        SlotCarryPolicy::Accumulate(ActivityAccumulationPolicy::Maximum),
        source(3),
    )
    .unwrap();
    let state = ActivityStateDefinition::new(vec![], vec![inventory], vec![modifier]).unwrap();
    assert_eq!(state.inventories()[0].maximum_entries(), 32);
    assert_eq!(
        state.modifiers()[0].owner(),
        ActivityModifierOwner::Inventory(inventory.id())
    );

    let error = ActivityStateDefinition::new(vec![], vec![], vec![modifier]).unwrap_err();
    assert_eq!(
        error,
        ActivityStateDefinitionError::MissingModifierInventory(modifier.id())
    );

    let error = ActivityInventoryDefinition::new(
        inventory_id(9),
        ActivityScope::Activity,
        0,
        1,
        SlotCarryPolicy::CarryExact,
        ActivityStateVisibility::Private,
        source(9),
    )
    .unwrap_err();
    assert_eq!(
        error,
        ActivityStateDefinitionError::InvalidInventoryEntryLimit(inventory_id(9))
    );
}

#[test]
fn state_definition_identity_order_is_canonical_and_duplicate_ids_fail_closed() {
    let left = ActivityStateDefinition::new(
        vec![boolean_slot(2), boolean_slot(1)],
        vec![inventory(2), inventory(1)],
        vec![scope_modifier(2), scope_modifier(1)],
    )
    .unwrap();
    let right = ActivityStateDefinition::new(
        vec![boolean_slot(1), boolean_slot(2)],
        vec![inventory(1), inventory(2)],
        vec![scope_modifier(1), scope_modifier(2)],
    )
    .unwrap();
    assert_eq!(left, right);

    let error =
        ActivityStateDefinition::new(vec![boolean_slot(1), boolean_slot(1)], vec![], vec![])
            .unwrap_err();
    assert_eq!(error, ActivityStateDefinitionError::DuplicateSlot(slot(1)));
}

fn boolean_slot(value: u32) -> ActivitySlotDefinition {
    ActivitySlotDefinition::new_with_policy(
        slot(value),
        ActivityScope::Activity,
        ActivityValue::Boolean(false),
        None,
        None,
        vec![SlotResetPoint::ActivityStart],
        SlotCarryPolicy::CarryExact,
        ActivityStateVisibility::Private,
        source(u64::from(value)),
    )
    .unwrap()
}

fn inventory(value: u32) -> ActivityInventoryDefinition {
    ActivityInventoryDefinition::new(
        inventory_id(value),
        ActivityScope::Activity,
        32,
        999,
        SlotCarryPolicy::CarryExact,
        ActivityStateVisibility::Player,
        source(u64::from(value)),
    )
    .unwrap()
}

fn scope_modifier(value: u32) -> ActivityModifierDefinition {
    ActivityModifierDefinition::new(
        modifier_id(value),
        ActivityModifierOwner::Scope(ActivityScope::Activity),
        u64::from(value),
        8,
        SlotCarryPolicy::CarryExact,
        source(u64::from(value)),
    )
    .unwrap()
}

fn slot(value: u32) -> ActivitySlotId {
    ActivitySlotId::new(value).unwrap()
}
fn inventory_id(value: u32) -> ActivityInventoryId {
    ActivityInventoryId::new(value).unwrap()
}
fn modifier_id(value: u32) -> ActivityModifierId {
    ActivityModifierId::new(value).unwrap()
}
fn source(value: u64) -> ActivityStateSource {
    ActivityStateSource::new(value).unwrap()
}
fn section(value: u32) -> SectionId {
    SectionId::new(value).unwrap()
}
fn node(value: u32) -> NodeId {
    NodeId::new(value).unwrap()
}
fn attempt(value: u32) -> AttemptId {
    AttemptId::new(value).unwrap()
}
