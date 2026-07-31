use starclock_activity::{
    ActivityCause, ActivityConfigDigest, ActivityDefinitionDigest, ActivityDefinitionId,
    ActivityDefinitionIdentity, ActivityInstanceId, ActivityMasterSeed, ActivityRngContext,
    ActivityRngLabel, ActivityRngStreams, ActivitySlotId, ActivityTransactionOutcome,
    ActivityTransactionState, ActivityValue,
};

use super::{
    GOLD_AND_GEARS_ADVENTURE_POLICY_ACCURACY, GOLD_AND_GEARS_ADVENTURE_POLICY_REVISION,
    GOLD_AND_GEARS_ADVENTURE_RUNTIME_REVISION, GOLD_AND_GEARS_OCCURRENCE_POLICY_ACCURACY,
    GOLD_AND_GEARS_OCCURRENCE_POLICY_REVISION, GOLD_AND_GEARS_OCCURRENCE_RUNTIME_REVISION,
    GOLD_AND_GEARS_SERVICE_RUNTIME_REVISION, GoldAndGearsAdventureExternalOutcome,
    GoldAndGearsAdventureType, GoldAndGearsEntryError, GoldAndGearsOccurrenceOperation,
    GoldAndGearsRuntimeFactory, GoldAndGearsRuntimeInstance, GoldAndGearsServiceKind,
    GoldAndGearsServiceOfferSelector,
    state_layout::{RESOURCE_COSMIC_FRAGMENTS_KEY, RUN_RESOURCES_SLOT},
    tests::compiled_fixture,
};

#[test]
fn occurrence_service_and_adventure_catalogs_are_complete_and_revisioned() {
    let factory = factory();
    assert_eq!(factory.occurrence_definitions().len(), 62);
    assert_eq!(factory.occurrence_variants().len(), 65);
    assert_eq!(factory.occurrence_choices().len(), 257);
    assert_eq!(factory.service_definitions().len(), 15);
    assert_eq!(factory.adventure_definitions().len(), 8);
    assert_eq!(
        factory
            .occurrence_choices()
            .iter()
            .filter(|choice| choice.outcome().uses_seeded_uniform_policy())
            .count(),
        43
    );
    assert_eq!(
        factory
            .occurrence_definitions()
            .iter()
            .map(|occurrence| occurrence.variants().len())
            .sum::<usize>(),
        71
    );
    assert_eq!(
        factory
            .occurrence_variants()
            .iter()
            .map(|variant| variant.choices().len())
            .sum::<usize>(),
        257
    );
    assert_eq!(
        [
            GOLD_AND_GEARS_OCCURRENCE_RUNTIME_REVISION,
            GOLD_AND_GEARS_OCCURRENCE_POLICY_REVISION,
            GOLD_AND_GEARS_SERVICE_RUNTIME_REVISION,
            GOLD_AND_GEARS_ADVENTURE_RUNTIME_REVISION,
            GOLD_AND_GEARS_ADVENTURE_POLICY_REVISION,
        ],
        [
            "gold-and-gears-occurrence-runtime-v1",
            "gold-and-gears-occurrence-random-outcome-policy-v1",
            "gold-and-gears-service-runtime-v1",
            "gold-and-gears-adventure-runtime-v1",
            "gold-and-gears-adventure-reward-policy-v1",
        ]
    );
    assert_eq!(
        (
            GOLD_AND_GEARS_OCCURRENCE_POLICY_ACCURACY,
            GOLD_AND_GEARS_ADVENTURE_POLICY_ACCURACY,
        ),
        (
            "DeterministicProjectPolicyNotObservedParity",
            "DeterministicProjectPolicyNotObservedParity",
        )
    );
    assert_eq!(
        digest_hex(factory.occurrence_runtime_digest()),
        "a96fa3dafbb386838519844bd2d1e91df9517912178d8687951717e91c1102ae"
    );
    assert_eq!(
        digest_hex(factory.service_runtime_digest()),
        "021e650649cf66066432e97e55baac50b4315a48aaa910779ec10fd104777252"
    );
    assert_eq!(
        digest_hex(factory.adventure_runtime_digest()),
        "a6f7ed5ad7d4b5750ac17694cd8db05e528252ecbf42dfd8ee02ea4fc0403dec"
    );
}

#[test]
fn occurrence_choices_preserve_authored_costs_operations_and_parameter_indices() {
    let factory = factory();
    let choices = factory.occurrence_choices();
    assert_eq!(
        choices
            .iter()
            .map(|choice| choice.costs().len())
            .sum::<usize>(),
        55
    );
    assert_eq!(
        choices
            .iter()
            .flat_map(|choice| choice.outcome().parameter_refs())
            .count(),
        4
    );
    assert!(choices.iter().all(|choice| {
        choice.node_index() > 0 && choice.choice_index() > 0 && choice.option_index() > 0
    }));
    let obtain = choices
        .iter()
        .flat_map(|choice| choice.outcome().operations())
        .filter(|operation| **operation == GoldAndGearsOccurrenceOperation::Obtain)
        .count();
    assert_eq!(obtain, 118);
    assert!(choices.windows(2).all(|pair| pair[0].id() < pair[1].id()));
}

#[test]
fn occurrence_random_selection_is_labeled_canonical_and_fail_closed() {
    let instance = compiled_fixture(factory());
    let random = instance
        .occurrence_choices()
        .iter()
        .find(|choice| choice.outcome().uses_seeded_uniform_policy())
        .unwrap()
        .id();
    let deterministic = instance
        .occurrence_choices()
        .iter()
        .find(|choice| !choice.outcome().uses_seeded_uniform_policy())
        .unwrap()
        .id();
    let mut rng = activity_rng(&instance, 0);
    let before = rng.snapshots();
    let selection = instance
        .select_occurrence_candidates(random, &[50, 10, 30], 2, &mut rng)
        .unwrap();
    assert_eq!(selection.selected(), [10, 30]);
    assert!(
        selection
            .selected()
            .windows(2)
            .all(|pair| pair[0] < pair[1])
    );
    assert_only_label_advanced(&before, &rng.snapshots(), ActivityRngLabel::Occurrence, 2);

    let unchanged = rng.snapshots();
    assert_eq!(
        instance.select_occurrence_candidates(deterministic, &[1], 1, &mut rng),
        Err(GoldAndGearsEntryError::OccurrenceChoiceIsNotRandom(
            deterministic
        ))
    );
    assert_eq!(
        instance.select_occurrence_candidates(random, &[1, 1], 1, &mut rng),
        Err(GoldAndGearsEntryError::InvalidOccurrenceCandidates)
    );
    assert_eq!(rng.snapshots(), unchanged);
}

#[test]
fn service_stocks_and_shop_offers_use_exact_pools_and_shop_rng() {
    let instance = compiled_fixture(factory());
    let kinds = [
        GoldAndGearsServiceKind::BlessingShop,
        GoldAndGearsServiceKind::CurioShop,
        GoldAndGearsServiceKind::Currency,
        GoldAndGearsServiceKind::Downloader,
        GoldAndGearsServiceKind::EnhanceBlessing,
        GoldAndGearsServiceKind::ResetBlessing,
        GoldAndGearsServiceKind::RespiteOffers,
        GoldAndGearsServiceKind::Reviver,
    ];
    assert_eq!(
        kinds.map(|kind| {
            instance
                .service_definitions()
                .iter()
                .filter(|service| service.kind() == kind)
                .count()
        }),
        [5, 4, 1, 1, 1, 1, 1, 1]
    );
    let blessing_shop = instance
        .service_definitions()
        .iter()
        .find(|service| service.kind() == GoldAndGearsServiceKind::BlessingShop)
        .unwrap();
    assert_eq!(
        blessing_shop.offer_pool(),
        Some("gold-gears.blessing-pool.all")
    );
    assert_eq!(
        blessing_shop
            .stock()
            .iter()
            .map(|stock| (stock.selector(), stock.unit_cost(), stock.maximum_uses()))
            .collect::<Vec<_>>(),
        vec![
            (GoldAndGearsServiceOfferSelector::BlessingRarity(1), 100, 3,),
            (GoldAndGearsServiceOfferSelector::BlessingRarity(2), 200, 2,),
            (GoldAndGearsServiceOfferSelector::BlessingRarity(3), 300, 1,),
        ]
    );

    let mut rng = activity_rng(&instance, 7);
    let before = rng.snapshots();
    let blessings = instance
        .select_service_blessings(blessing_shop.stable_key(), 1, &[], 3, &mut rng)
        .unwrap();
    assert_eq!(blessings.len(), 3);
    let curio_shop = instance
        .service_definitions()
        .iter()
        .find(|service| service.kind() == GoldAndGearsServiceKind::CurioShop)
        .unwrap();
    let curios = instance
        .select_service_curios(curio_shop.stable_key(), &[], 3, &mut rng)
        .unwrap();
    assert_eq!(
        blessings.iter().map(|id| id.get()).collect::<Vec<_>>(),
        [68, 11, 53]
    );
    assert_eq!(
        curios
            .iter()
            .map(|candidate| candidate.source_id())
            .collect::<Vec<_>>(),
        [112, 122, 119]
    );
    assert_eq!(curios.len(), 3);
    assert!(curios.windows(2).all(|pair| pair[0].id() != pair[1].id()));
    assert_only_label_advanced(&before, &rng.snapshots(), ActivityRngLabel::Shop, 6);
}

#[test]
fn service_purchase_deducts_currency_and_stale_or_unfunded_use_is_atomic() {
    let instance = compiled_fixture(factory());
    let mut state = ActivityTransactionState::new(
        instance.state_definition().clone(),
        instance.graph_definition().entry(),
    );
    let mut rng = activity_rng(&instance, 11);
    let service = "universe.service.shop.100011";
    let selector = GoldAndGearsServiceOfferSelector::BlessingRarity(1);
    commit(
        &instance,
        &mut state,
        instance
            .compile_service_purchase(service, selector, 0)
            .unwrap(),
    );
    assert_eq!(
        counter(&state, RUN_RESOURCES_SLOT, RESOURCE_COSMIC_FRAGMENTS_KEY),
        0
    );

    let before = state_bytes(&instance, &state, &rng);
    let stale = instance
        .compile_service_purchase(service, selector, 0)
        .unwrap();
    assert!(matches!(
        apply(&instance, &mut state, stale),
        ActivityTransactionOutcome::Rejected(_)
    ));
    assert_eq!(state_bytes(&instance, &state, &rng), before);

    let unfunded = instance
        .compile_service_purchase(service, selector, 1)
        .unwrap();
    assert!(matches!(
        apply(&instance, &mut state, unfunded),
        ActivityTransactionOutcome::Rejected(_)
    ));
    assert_eq!(state_bytes(&instance, &state, &rng), before);
    let _ = &mut rng;
}

#[test]
fn adventure_accepts_external_results_and_resolves_cumulative_rewards_atomically() {
    let instance = compiled_fixture(factory());
    let kinds = [
        GoldAndGearsAdventureType::CaptureMonster,
        GoldAndGearsAdventureType::DestroyProp,
        GoldAndGearsAdventureType::EscapeLaser,
        GoldAndGearsAdventureType::Turntable,
    ];
    assert_eq!(
        kinds.map(|kind| {
            instance
                .adventure_definitions()
                .iter()
                .filter(|definition| definition.adventure_type() == kind)
                .count()
        }),
        [3, 3, 1, 1]
    );
    assert!(instance.adventure_definitions().iter().all(|definition| {
        definition.thresholds().len() == 2
            && definition.thresholds()[0].objective() == 1
            && definition.thresholds()[1].objective() == 2
            && definition.thresholds()[0].minimum_value()
                < definition.thresholds()[1].minimum_value()
    }));

    let mut rng = activity_rng(&instance, 0);
    let before = rng.snapshots();
    let plan = instance
        .resolve_adventure_outcome(
            GoldAndGearsAdventureExternalOutcome::new(1_210_601, 3_600).unwrap(),
            &[],
            &[],
            &mut rng,
        )
        .unwrap();
    assert_eq!(plan.completed_objectives(), 2);
    assert!((100..=150).contains(&plan.cosmic_fragments()));
    assert_eq!(plan.blessing_rarity(), Some(2));
    assert!(plan.blessing_offer().is_some());
    assert!(plan.offers_curio());
    assert!(plan.curio_offer().is_some());
    assert_only_label_advanced(&before, &rng.snapshots(), ActivityRngLabel::Reward, 3);
    assert_eq!(
        (
            plan.cosmic_fragments(),
            plan.blessing_offer().unwrap().get(),
            plan.curio_offer().unwrap().source_id(),
        ),
        (121, 130, 205)
    );

    let unchanged = rng.snapshots();
    assert_eq!(
        instance.resolve_adventure_outcome(
            GoldAndGearsAdventureExternalOutcome::new(1_210_601, 4_401).unwrap(),
            &[],
            &[],
            &mut rng,
        ),
        Err(GoldAndGearsEntryError::InvalidAdventureOutcome)
    );
    assert_eq!(rng.snapshots(), unchanged);

    let mut state = ActivityTransactionState::new(
        instance.state_definition().clone(),
        instance.graph_definition().entry(),
    );
    let initial = counter(&state, RUN_RESOURCES_SLOT, RESOURCE_COSMIC_FRAGMENTS_KEY);
    commit(
        &instance,
        &mut state,
        instance.compile_adventure_settlement(plan.clone()).unwrap(),
    );
    assert_eq!(
        counter(&state, RUN_RESOURCES_SLOT, RESOURCE_COSMIC_FRAGMENTS_KEY),
        initial + i64::from(plan.cosmic_fragments())
    );
    let before_repeat = state_bytes(&instance, &state, &rng);
    assert!(matches!(
        apply(
            &instance,
            &mut state,
            instance.compile_adventure_settlement(plan).unwrap()
        ),
        ActivityTransactionOutcome::Rejected(_)
    ));
    assert_eq!(state_bytes(&instance, &state, &rng), before_repeat);
}

fn factory() -> &'static GoldAndGearsRuntimeFactory {
    super::tests::shared_factory()
}

fn commit(
    instance: &GoldAndGearsRuntimeInstance,
    state: &mut ActivityTransactionState,
    program: starclock_activity::ActivityProgramDefinition,
) {
    assert!(matches!(
        apply(instance, state, program),
        ActivityTransactionOutcome::Committed(_)
    ));
}

fn apply(
    instance: &GoldAndGearsRuntimeInstance,
    state: &mut ActivityTransactionState,
    program: starclock_activity::ActivityProgramDefinition,
) -> ActivityTransactionOutcome {
    program
        .validate_against(instance.state_definition(), instance.graph_definition())
        .unwrap();
    let cause = ActivityCause::new(
        state.command_sequence() + 1,
        program.id(),
        state.current_node(),
    )
    .unwrap();
    state.apply_program(&program, cause, instance.graph_definition())
}

fn counter(state: &ActivityTransactionState, slot: u32, key: u64) -> i64 {
    match state.slot(ActivitySlotId::new(slot).unwrap()) {
        Some(ActivityValue::BoundedCounterMap(values)) => values
            .binary_search_by_key(&key, |(candidate, _)| *candidate)
            .ok()
            .map_or(0, |index| values[index].1),
        value => panic!("unexpected counter slot: {value:?}"),
    }
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

fn identity() -> ActivityDefinitionIdentity {
    ActivityDefinitionIdentity::new(
        ActivityDefinitionId::new(14).unwrap(),
        ActivityDefinitionDigest::new([0x14; 32]).unwrap(),
        ActivityConfigDigest::new([0x47; 32]).unwrap(),
    )
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

fn digest_hex(digest: [u8; 32]) -> String {
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}
