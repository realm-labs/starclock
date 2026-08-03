use starclock_activity::{
    ActivityCause, ActivityConfigDigest, ActivityDefinitionDigest, ActivityDefinitionId,
    ActivityDefinitionIdentity, ActivityInstanceId, ActivityInventoryId, ActivityMasterSeed,
    ActivityRngContext, ActivityRngLabel, ActivityRngStreams, ActivitySlotId,
    ActivityTransactionOutcome, ActivityTransactionState, ActivityValue,
};

use super::{
    GoldAndGearsRuntimeFactory, GoldAndGearsRuntimeInstance,
    content_link_runtime::GOLD_AND_GEARS_SHARED_CONTENT_RUNTIME_REVISION,
    curio_runtime::{
        GOLD_AND_GEARS_CURIO_OFFER_POLICY_ACCURACY,
        GOLD_AND_GEARS_CURIO_OFFER_POLICY_REVISION, GOLD_AND_GEARS_CURIO_RUNTIME_REVISION,
    },
    curio_types::{
        GoldAndGearsCurioCategory, GoldAndGearsCurioDefinition, GoldAndGearsCurioId,
        GoldAndGearsCurioOfferContext, GoldAndGearsCurioOfferSource,
        GoldAndGearsCurioRuleKind, GoldAndGearsCurioRuleOwnership, GoldAndGearsCurioState,
    },
    state_layout::{
        BLESSING_INVENTORY, CONTENT_CURIO_CHARGE_BASE, CONTENT_CURIO_STATE_BASE,
        CONTENT_LIFECYCLE_SLOT, CURIO_INVENTORY,
    },
    tests::compiled_fixture,
};
use super::{tests};

#[test]
fn curio_partition_binds_exactly_160_project_policy_rules() {
    let factory = factory();
    let bindings = factory.curio_rule_bindings();
    assert_eq!(bindings.len(), 160);
    assert!(
        bindings
            .windows(2)
            .all(|pair| pair[0].rule_id() < pair[1].rule_id())
    );
    assert!(
        bindings
            .iter()
            .all(|binding| binding.accuracy() == "ProjectPolicy")
    );
    assert_eq!(
        bindings
            .iter()
            .filter(|binding| binding.kind() == GoldAndGearsCurioRuleKind::LifecycleState)
            .count(),
        80
    );
    assert_eq!(
        bindings
            .iter()
            .filter(|binding| binding.kind() == GoldAndGearsCurioRuleKind::Contribution)
            .count(),
        80
    );
    assert_eq!(
        bindings
            .iter()
            .filter(|binding| binding.ownership() == GoldAndGearsCurioRuleOwnership::Shared)
            .count(),
        61
    );
    assert_eq!(
        bindings
            .iter()
            .filter(|binding| binding.ownership() == GoldAndGearsCurioRuleOwnership::GoldAndGears)
            .count(),
        99
    );
    assert!(bindings.iter().all(|binding| {
        binding.operation()
            == if binding.kind() == GoldAndGearsCurioRuleKind::LifecycleState {
                "ExecuteCurioLifecycle"
            } else {
                "ProjectCurioContribution"
            }
    }));
    assert!(bindings.iter().all(|binding| {
        binding.executor()
            == if binding.ownership() == GoldAndGearsCurioRuleOwnership::Shared {
                "ReleasedSharedExecutor"
            } else {
                "ActivityAndCombatPrograms"
            }
    }));
}

#[test]
fn shared_content_denominators_revisions_and_inventories_are_bound() {
    let factory = factory();
    assert_eq!(
        factory.content_runtime.denominators(),
        (162, 324, 9, 61, 80)
    );
    assert_eq!(
        GOLD_AND_GEARS_SHARED_CONTENT_RUNTIME_REVISION,
        "gold-and-gears-shared-content-runtime-v1"
    );
    assert_eq!(
        GOLD_AND_GEARS_CURIO_RUNTIME_REVISION,
        "gold-and-gears-curio-runtime-v1"
    );
    assert_eq!(
        GOLD_AND_GEARS_CURIO_OFFER_POLICY_REVISION,
        "gold-and-gears-curio-offer-policy-v1"
    );
    assert_eq!(
        GOLD_AND_GEARS_CURIO_OFFER_POLICY_ACCURACY,
        "DeterministicProjectPolicyNotObservedParity"
    );

    let instance = compiled_fixture(factory);
    let blessing = inventory_definition(&instance, BLESSING_INVENTORY);
    assert_eq!(
        (blessing.maximum_entries(), blessing.maximum_stack()),
        (162, 2)
    );
    let curio = inventory_definition(&instance, CURIO_INVENTORY);
    assert_eq!((curio.maximum_entries(), curio.maximum_stack()), (80, 1));
    assert_ne!(
        factory.shared_content_digests().blessing(),
        factory.shared_content_digests().curio()
    );
    assert_eq!(
        factory.shared_content_digests(),
        instance.shared_content_digests()
    );
    assert_eq!(
        digest_hex(factory.shared_content_digests().blessing()),
        "5b0e55ff87403533edab808784b3f4234c6ef553e5d77c61a1c7ad4b6d873e8b"
    );
    assert_eq!(
        digest_hex(factory.shared_content_digests().path()),
        "0e7f6d8182250caf80b0110c5254ba3647d67ca4e842231cd92287ba51c3f9f5"
    );
    assert_eq!(
        digest_hex(factory.shared_content_digests().curio()),
        "7f7326ca2ea4a58c312feb9c551abe6a12703b7e647d6834f8e328c5cc1f6ca9"
    );
}

#[test]
fn all_curio_copies_categories_and_lifecycle_denominators_are_exact() {
    let factory = factory();
    let definitions = factory.curio_definitions();
    assert_eq!(definitions.len(), 80);
    assert_eq!(
        definitions
            .iter()
            .filter(|definition| definition.shared_curio().is_some())
            .count(),
        61
    );
    assert_eq!(
        definitions
            .iter()
            .filter(|definition| definition.shared_curio().is_none())
            .count(),
        19
    );
    for (category, expected) in [
        (GoldAndGearsCurioCategory::Normal, 60),
        (GoldAndGearsCurioCategory::Negative, 14),
        (GoldAndGearsCurioCategory::ErrorCode, 6),
    ] {
        assert_eq!(
            definitions
                .iter()
                .filter(|definition| definition.category() == category)
                .count(),
            expected
        );
    }
    assert_eq!(
        definitions
            .iter()
            .filter(|definition| definition.initial_state() == GoldAndGearsCurioState::Repairing)
            .count(),
        6
    );
    assert_eq!(
        definitions
            .iter()
            .filter(|definition| definition.maximum_charges().is_some())
            .count(),
        12
    );
    assert_eq!(
        definitions
            .iter()
            .filter(|definition| {
                definition.decrement_event() == "SourceConditionWithoutNumericCharges"
            })
            .count(),
        4
    );
    assert!(definitions.windows(2).all(|pair| {
        (pair[0].handbook_order(), pair[0].source_id())
            < (pair[1].handbook_order(), pair[1].source_id())
    }));
    assert_eq!(
        digest_hex(factory.curio_runtime_digest()),
        "3c058dd675fac30ac548f62d59d713ee1f056adb791fa735230ea9a9e35e6049"
    );
}

#[test]
fn blessing_selection_and_inventory_programs_use_only_reward_rng() {
    let factory = factory();
    let instance = compiled_fixture(factory);
    let mut rng = activity_rng(&instance, 14);
    let before = rng.snapshots();
    let first = instance
        .select_trailblaze_blessing(&[], &mut rng)
        .unwrap()
        .unwrap();
    assert_only_label_advanced(&before, &rng.snapshots(), ActivityRngLabel::Reward, 1);
    let second = instance
        .select_trailblaze_blessing(&[first], &mut rng)
        .unwrap()
        .unwrap();
    assert_ne!(first, second);

    let mut state = new_state(&instance);
    commit(
        &instance,
        &mut state,
        instance.compile_blessing_acquisition(first).unwrap(),
    );
    assert_eq!(
        inventory_count(&instance, &state, BLESSING_INVENTORY, first.get()),
        1
    );
    commit(
        &instance,
        &mut state,
        instance.compile_blessing_enhancement(first).unwrap(),
    );
    assert_eq!(
        inventory_count(&instance, &state, BLESSING_INVENTORY, first.get()),
        2
    );
    let contributions = instance.blessing_contributions(&[(first, 2)]).unwrap();
    assert_eq!(contributions.entries().len(), 1);
    assert_eq!(contributions.entries()[0].blessing(), first);
    assert_eq!(contributions.entries()[0].level().level(), 2);

    commit(
        &instance,
        &mut state,
        instance
            .compile_blessing_replacement(first, second)
            .unwrap(),
    );
    assert_eq!(
        inventory_count(&instance, &state, BLESSING_INVENTORY, first.get()),
        0
    );
    assert_eq!(
        inventory_count(&instance, &state, BLESSING_INVENTORY, second.get()),
        1
    );
    assert!(
        instance
            .compile_blessing_replacement(second, second)
            .is_err()
    );
}

#[test]
fn offer_policy_is_fail_closed_canonical_and_excludes_owned_curios() {
    let instance = compiled_fixture(factory());
    let normal = GoldAndGearsCurioOfferContext::full_category(
        GoldAndGearsCurioOfferSource::TrailblazeBonus,
        GoldAndGearsCurioCategory::Normal,
    )
    .unwrap();
    let candidates = instance.curio_candidates(&normal, &[]).unwrap();
    assert_eq!(candidates.len(), 60);
    assert!(candidates.windows(2).all(|pair| {
        let left = definition(&instance, pair[0].id());
        let right = definition(&instance, pair[1].id());
        (left.handbook_order(), left.source_id()) < (right.handbook_order(), right.source_id())
    }));
    assert_eq!(
        instance
            .curio_candidates(&normal, &[candidates[0].id()])
            .unwrap()
            .len(),
        59
    );

    let explicit = GoldAndGearsCurioOfferContext::explicit(
        GoldAndGearsCurioOfferSource::Occurrence,
        GoldAndGearsCurioCategory::Normal,
        vec![
            candidates[1].stable_key().into(),
            candidates[0].stable_key().into(),
        ],
    )
    .unwrap();
    let selected = instance.curio_candidates(&explicit, &[]).unwrap();
    assert_eq!(
        selected
            .iter()
            .map(|candidate| candidate.id())
            .collect::<Vec<_>>(),
        vec![candidates[0].id(), candidates[1].id()]
    );
    let unknown = GoldAndGearsCurioOfferContext::explicit(
        GoldAndGearsCurioOfferSource::Service,
        GoldAndGearsCurioCategory::Normal,
        vec!["gold-gears.curio.not-released".into()],
    )
    .unwrap();
    assert!(instance.curio_candidates(&unknown, &[]).is_err());
    assert!(
        GoldAndGearsCurioOfferContext::full_category(
            GoldAndGearsCurioOfferSource::Occurrence,
            GoldAndGearsCurioCategory::Normal,
        )
        .is_none()
    );
}

#[test]
fn curio_selection_uses_the_causal_stream_and_empty_offers_draw_nothing() {
    let instance = compiled_fixture(factory());
    for (source, label) in [
        (
            GoldAndGearsCurioOfferSource::TrailblazeBonus,
            ActivityRngLabel::Reward,
        ),
        (
            GoldAndGearsCurioOfferSource::AuxiliaryConundrum,
            ActivityRngLabel::Reward,
        ),
    ] {
        let context =
            GoldAndGearsCurioOfferContext::full_category(source, GoldAndGearsCurioCategory::Normal)
                .unwrap();
        assert_selection_label(&instance, &context, label);
    }
    let definitions = instance.curio_definitions();
    for (source, label) in [
        (
            GoldAndGearsCurioOfferSource::Occurrence,
            ActivityRngLabel::Occurrence,
        ),
        (
            GoldAndGearsCurioOfferSource::Service,
            ActivityRngLabel::Shop,
        ),
        (
            GoldAndGearsCurioOfferSource::Replacement,
            ActivityRngLabel::Reward,
        ),
    ] {
        let context = GoldAndGearsCurioOfferContext::explicit(
            source,
            GoldAndGearsCurioCategory::Normal,
            definitions
                .iter()
                .filter(|definition| definition.category() == GoldAndGearsCurioCategory::Normal)
                .take(3)
                .map(|definition| definition.stable_key().into())
                .collect(),
        )
        .unwrap();
        assert_selection_label(&instance, &context, label);
    }

    let context = GoldAndGearsCurioOfferContext::full_category(
        GoldAndGearsCurioOfferSource::TrailblazeBonus,
        GoldAndGearsCurioCategory::Normal,
    )
    .unwrap();
    let mut golden_rng = activity_rng(&instance, 0);
    let golden = instance
        .select_curios(&context, &[], 3, &mut golden_rng)
        .unwrap();
    assert_eq!(
        golden
            .iter()
            .map(|candidate| candidate.id().get())
            .collect::<Vec<_>>(),
        vec![3205, 3209, 122]
    );
    let mut rng = activity_rng(&instance, 15);
    let before = rng.snapshots();
    assert!(
        instance
            .select_curios(&context, &[], 0, &mut rng)
            .unwrap()
            .is_empty()
    );
    assert_eq!(before, rng.snapshots());
}

#[test]
fn charged_and_source_condition_curios_transition_atomically() {
    let instance = compiled_fixture(factory());
    let charged = by_source(&instance, 201);
    assert_eq!(definition(&instance, charged).maximum_charges(), Some(2));
    let source_destroyed = by_source(&instance, 203);
    assert_eq!(
        definition(&instance, source_destroyed).decrement_event(),
        "SourceConditionWithoutNumericCharges"
    );
    let mut state = new_state(&instance);
    commit(
        &instance,
        &mut state,
        instance.compile_curio_acquisition(charged).unwrap(),
    );
    assert_curio_state(&state, charged, GoldAndGearsCurioState::Active, 2);
    commit(
        &instance,
        &mut state,
        instance.compile_curio_charge_use(charged, 2).unwrap(),
    );
    assert_curio_state(&state, charged, GoldAndGearsCurioState::Active, 1);
    commit(
        &instance,
        &mut state,
        instance.compile_curio_charge_use(charged, 1).unwrap(),
    );
    assert_curio_state(&state, charged, GoldAndGearsCurioState::Destroyed, 0);

    commit(
        &instance,
        &mut state,
        instance
            .compile_curio_acquisition(source_destroyed)
            .unwrap(),
    );
    commit(
        &instance,
        &mut state,
        instance
            .compile_curio_source_destruction(source_destroyed)
            .unwrap(),
    );
    assert_curio_state(
        &state,
        source_destroyed,
        GoldAndGearsCurioState::Destroyed,
        0,
    );
}

#[test]
fn error_code_repair_and_fixed_contribution_are_deterministic() {
    let instance = compiled_fixture(factory());
    let error_code = by_source(&instance, 45);
    assert_eq!(
        definition(&instance, error_code).initial_state(),
        GoldAndGearsCurioState::Repairing
    );
    let mut state = new_state(&instance);
    commit(
        &instance,
        &mut state,
        instance.compile_curio_acquisition(error_code).unwrap(),
    );
    for progress in 0..3 {
        commit(
            &instance,
            &mut state,
            instance
                .compile_curio_repair_progress(error_code, progress)
                .unwrap(),
        );
    }
    assert_curio_state(&state, error_code, GoldAndGearsCurioState::Fixed, 0);
    let contribution = instance
        .curio_contributions(
            &[(error_code, 1)],
            &[(error_code, GoldAndGearsCurioState::Fixed)],
            &[],
        )
        .unwrap();
    assert_eq!(
        digest_hex(contribution.digest()),
        "88a396ab3754c643fa5a8ec5666c56023a0b65ea6ab2067ce602897c9935c695"
    );
    assert_eq!(contribution.entries().len(), 1);
    assert_eq!(
        contribution.entries()[0].state(),
        GoldAndGearsCurioState::Fixed
    );
    assert!(!contribution.entries()[0].source_effect_id().is_empty());
}

#[test]
fn replacement_teardown_and_contribution_validation_preserve_invariants() {
    let instance = compiled_fixture(factory());
    let removed = by_source(&instance, 201);
    let acquired = by_source(&instance, 203);
    let mut state = new_state(&instance);
    commit(
        &instance,
        &mut state,
        instance.compile_curio_acquisition(removed).unwrap(),
    );
    commit(
        &instance,
        &mut state,
        instance
            .compile_curio_replacement(removed, acquired)
            .unwrap(),
    );
    assert_eq!(
        inventory_count(&instance, &state, CURIO_INVENTORY, removed.get()),
        0
    );
    assert_curio_state(&state, acquired, GoldAndGearsCurioState::Active, 0);
    commit(
        &instance,
        &mut state,
        instance.compile_curio_teardown(acquired).unwrap(),
    );
    assert_eq!(
        inventory_count(&instance, &state, CURIO_INVENTORY, acquired.get()),
        0
    );
    assert_eq!(
        counter(&state, CONTENT_CURIO_STATE_BASE + u64::from(acquired.get())),
        0
    );

    assert!(
        instance
            .curio_contributions(
                &[(acquired, 1), (acquired, 1)],
                &[(acquired, GoldAndGearsCurioState::Active)],
                &[],
            )
            .is_err()
    );
    assert!(
        instance
            .curio_contributions(
                &[(acquired, 1)],
                &[(acquired, GoldAndGearsCurioState::Active)],
                &[(acquired, 1)],
            )
            .is_err()
    );
}

#[test]
fn all_160_curio_rules_execute_through_the_production_fixture() {
    let instance = compiled_fixture(factory());
    let definitions = instance.curio_definitions();
    let mut state = new_state(&instance);
    for definition in definitions {
        commit(
            &instance,
            &mut state,
            instance.compile_curio_acquisition(definition.id()).unwrap(),
        );
    }
    assert_eq!(state.command_sequence(), 80);

    let owned = definitions
        .iter()
        .map(|definition| (definition.id(), 1))
        .collect::<Vec<_>>();
    let states = definitions
        .iter()
        .map(|definition| (definition.id(), definition.initial_state()))
        .collect::<Vec<_>>();
    let counters = definitions
        .iter()
        .filter_map(|definition| {
            definition
                .maximum_charges()
                .map(|charges| (definition.id(), charges))
        })
        .collect::<Vec<_>>();
    let contributions = instance
        .curio_contributions(&owned, &states, &counters)
        .unwrap();
    assert_eq!(contributions.entries().len(), 80);
    assert_eq!(
        contributions
            .entries()
            .iter()
            .filter(|contribution| contribution.shared_curio().is_some())
            .count(),
        61
    );
    assert_eq!(
        digest_hex(contributions.digest()),
        "a7fbde6e31ec7a7037dd3168fe4d99c71fa8ee593409c36c3fd333a9cd4b0934"
    );
    let rng = activity_rng(&instance, 14_505);
    assert_eq!(
        state_hash(&instance, &state, &rng),
        "0fd7bbf9c4c132ce346b04781ec64648f824d496adc921773bbe0b710192db32"
    );
}

fn factory() -> &'static GoldAndGearsRuntimeFactory {
    tests::shared_factory()
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

fn inventory_definition(
    instance: &GoldAndGearsRuntimeInstance,
    id: u32,
) -> starclock_activity::ActivityInventoryDefinition {
    instance
        .state_definition()
        .inventories()
        .iter()
        .copied()
        .find(|definition| definition.id() == ActivityInventoryId::new(id).unwrap())
        .unwrap()
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

fn counter(state: &ActivityTransactionState, key: u64) -> i64 {
    match state.slot(ActivitySlotId::new(CONTENT_LIFECYCLE_SLOT).unwrap()) {
        Some(ActivityValue::BoundedCounterMap(values)) => values
            .binary_search_by_key(&key, |(candidate, _)| *candidate)
            .ok()
            .map_or(0, |index| values[index].1),
        value => panic!("unexpected lifecycle slot: {value:?}"),
    }
}

fn assert_curio_state(
    state: &ActivityTransactionState,
    id: GoldAndGearsCurioId,
    expected_state: GoldAndGearsCurioState,
    expected_counter: i64,
) {
    assert_eq!(
        counter(state, CONTENT_CURIO_STATE_BASE + u64::from(id.get())),
        expected_state as i64
    );
    assert_eq!(
        counter(state, CONTENT_CURIO_CHARGE_BASE + u64::from(id.get())),
        expected_counter
    );
}

fn by_source(instance: &GoldAndGearsRuntimeInstance, source: u32) -> GoldAndGearsCurioId {
    instance
        .curio_definitions()
        .iter()
        .find(|definition| definition.source_id() == source)
        .unwrap()
        .id()
}

fn definition(
    instance: &GoldAndGearsRuntimeInstance,
    id: GoldAndGearsCurioId,
) -> &GoldAndGearsCurioDefinition {
    instance
        .curio_definitions()
        .iter()
        .find(|definition| definition.id() == id)
        .unwrap()
}

fn assert_selection_label(
    instance: &GoldAndGearsRuntimeInstance,
    context: &GoldAndGearsCurioOfferContext,
    label: ActivityRngLabel,
) {
    let mut rng = activity_rng(instance, 99);
    let before = rng.snapshots();
    assert_eq!(
        instance
            .select_curios(context, &[], 2, &mut rng)
            .unwrap()
            .len(),
        2
    );
    assert_only_label_advanced(&before, &rng.snapshots(), label, 2);
}

fn assert_only_label_advanced(
    before: &[starclock_activity::ActivityRngStreamSnapshot],
    after: &[starclock_activity::ActivityRngStreamSnapshot],
    label: ActivityRngLabel,
    draws: u64,
) {
    for (old, new) in before.iter().zip(after) {
        assert_eq!(new.seed(), old.seed());
        assert_eq!(
            new.draw_count(),
            old.draw_count() + if old.label() == label { draws } else { 0 }
        );
    }
}

fn identity() -> ActivityDefinitionIdentity {
    ActivityDefinitionIdentity::new(
        ActivityDefinitionId::new(14).unwrap(),
        ActivityDefinitionDigest::new([0x14; 32]).unwrap(),
        ActivityConfigDigest::new([0x47; 32]).unwrap(),
    )
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

fn digest_hex(digest: [u8; 32]) -> String {
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn state_hash(
    instance: &GoldAndGearsRuntimeInstance,
    state: &ActivityTransactionState,
    rng: &ActivityRngStreams,
) -> String {
    digest_hex(
        state
            .state_hash(
                identity(),
                instance.graph_definition(),
                ActivityInstanceId::new(1).unwrap(),
                rng,
            )
            .bytes(),
    )
}
