//! Production-owned native composition: no room callback, probes or fake credit.
use crate::baseline_controller::{
    ActivityBaselineHints, ActivityOptionHint, ActivityScoreComponents,
};
use crate::divergent_universe::{
    DivergentUniverseBaselineFixture, DivergentUniverseBaselinePolicy,
    DivergentUniverseBaselineRunner, DivergentUniverseBaselineStep,
    DivergentUniverseCyclicalRefresh, DivergentUniverseEntry, DivergentUniverseOfferedSelection,
    battle_room::{BattleRoomSelection, BattleRoomSequenceLength},
    domain_route::profile::{PositionProfileError, PositionProfileRecipe},
    encode_divergent_universe_replay,
    respite_room::RespiteEnhancementPolicy,
    tests::{
        battle_room::base,
        weighted_curio_deflagration_native::{applications, fixture as fire_fixture},
    },
    weighted_curio::{WeightedCurioSlotLimit, room::CLEAR_EQUIPMENT},
};
use starclock_activity::{ActivityInstanceId, ActivityMasterSeed, ActivityOptionId};
use starclock_data::{
    divergent_universe_catalog::{DivergentUniverseAreaId, DivergentUniverseRunFamily},
    divergent_universe_decisions::{
        BattleRewardDomain, reward_occurrences::RewardOccurrenceId, shop::ShopStockId,
    },
    divergent_universe_domain_decks::DomainCardKind,
    divergent_universe_service_catalog::DivergentUniverseWorkbenchId,
};
use std::sync::Arc;
const FAMILIES: [DivergentUniverseRunFamily; 2] = [
    DivergentUniverseRunFamily::Ordinary,
    DivergentUniverseRunFamily::Cyclical,
];

fn recipe(source: &DivergentUniverseBaselineFixture, deck: usize) -> PositionProfileRecipe {
    let factory = source.factory();
    let pool = factory.decision_catalog().encounter_pool();
    let battle = |domain| BattleRoomSelection {
        group: pool.encounter_group.clone(),
        stage: pool.candidate_stages[0].clone(),
        domain,
    };
    PositionProfileRecipe {
        deck: factory.decision_catalog().domain_decks()[deck].key.clone(),
        hand_width: 3,
        combat: battle(BattleRewardDomain::Combat),
        encounter: battle(BattleRewardDomain::Aberration),
        elite: battle(BattleRewardDomain::Elite),
        boss: battle(BattleRewardDomain::Boss),
        battle_sequence: BattleRoomSequenceLength::SINGLE,
        conversion_sequence: BattleRoomSequenceLength::new(2).unwrap(),
        event: factory.decision_catalog().occurrences()[0].variant.clone(),
        following_events: Vec::new(),
        shop: ShopStockId::new("du.shop-stock.acquisition-policy").unwrap(),
        reward: RewardOccurrenceId::new("du.reward-room.level-one-substitute").unwrap(),
        workbench: DivergentUniverseWorkbenchId::new("divergent-universe.workbench.101").unwrap(),
        respite: RespiteEnhancementPolicy::new(0, 1).unwrap(),
        equipment_capacity: WeightedCurioSlotLimit::new(1).unwrap(),
    }
}

#[test]
fn native_position_profile_accounts_all_nine_decks_without_missing_program_fallback() {
    let source = DivergentUniverseBaselineFixture::production().unwrap();
    let fresh = DivergentUniverseBaselineFixture::production().unwrap();
    for family in FAMILIES {
        let mut accepted = Vec::new();
        for deck in 0..source.factory().decision_catalog().domain_decks().len() {
            let inputs = recipe(&source, deck);
            match source
                .factory()
                .compile_position_profile(base(&source, family), &inputs)
            {
                Ok(flow) => {
                    let rebuilt = fresh
                        .factory()
                        .compile_position_profile(base(&fresh, family), &inputs)
                        .unwrap();
                    assert_eq!(
                        flow.definition().identity(),
                        rebuilt.definition().identity()
                    );
                    let instance = ActivityInstanceId::new(1).unwrap();
                    let seed = ActivityMasterSeed::from_u64(27_001);
                    let a = flow.start(instance, seed).unwrap();
                    let b = rebuilt.start(instance, seed).unwrap();
                    assert_eq!(a.events(), b.events());
                    assert_eq!(
                        a.into_activity().canonical_state_bytes(),
                        b.into_activity().canonical_state_bytes()
                    );
                    accepted.push(deck);
                }
                other => panic!("unexpected native composition for deck {deck}: {other:?}"),
            }
        }
        assert_eq!(accepted, vec![0, 1, 2, 3, 4, 5, 6, 7, 8]);
    }
}

#[test]
fn native_position_profile_rejects_changed_unknown_and_already_attached_inputs() {
    let source = DivergentUniverseBaselineFixture::production().unwrap();
    let plain = base(&source, FAMILIES[0]);
    let original = recipe(&source, 0);
    let factory = source.factory();
    let flow = factory
        .compile_position_profile(plain.clone(), &original)
        .unwrap();
    assert!(factory.compile_position_profile(flow, &original).is_err());
    assert!(
        factory
            .compile_position_profile(source.flow(FAMILIES[0]).unwrap(), &original)
            .is_err()
    );
    let before = plain.definition().identity();
    let mut unknown = original.clone();
    unknown.deck = "du.domain-deck.unknown".into();
    assert!(
        factory
            .compile_position_profile(plain.clone(), &unknown)
            .is_err()
    );
    unknown = original.clone();
    unknown.hand_width = 0;
    assert!(
        factory
            .compile_position_profile(plain.clone(), &unknown)
            .is_err()
    );
    unknown = original.clone();
    unknown.shop = ShopStockId::new("du.shop-stock.unknown").unwrap();
    assert!(matches!(
        factory.compile_position_profile(plain.clone(), &unknown),
        Err(PositionProfileError::Shop(_))
    ));
    assert_eq!(plain.definition().identity(), before);
    let mut changed = original.clone();
    changed.equipment_capacity = WeightedCurioSlotLimit::new(2).unwrap();
    assert_ne!(
        factory
            .compile_position_profile(plain.clone(), &original)
            .unwrap()
            .definition()
            .identity(),
        factory
            .compile_position_profile(plain.clone(), &changed)
            .unwrap()
            .definition()
            .identity()
    );
    changed = original.clone();
    changed.battle_sequence = BattleRoomSequenceLength::new(2).unwrap();
    assert_ne!(
        factory
            .compile_position_profile(plain.clone(), &original)
            .unwrap()
            .definition()
            .identity(),
        factory
            .compile_position_profile(plain, &changed)
            .unwrap()
            .definition()
            .identity()
    );
}

#[test]
fn native_position_profile_compiles_guide_blank_coin_and_both_conversion_layouts() {
    let source = DivergentUniverseBaselineFixture::production().unwrap();
    let factory = source.factory();
    let catalog = factory.bundle.catalog();
    let inputs = recipe(&source, 0);
    for raw in ["103", "104", "406", "20406", "409", "20409"] {
        let area = DivergentUniverseAreaId::new(format!("divergent-universe.area.{raw}")).unwrap();
        let definition = catalog.area(&area).unwrap();
        let original = source.flow(definition.run_family()).unwrap();
        let mut entry = DivergentUniverseEntry::new(
            area.clone(),
            definition.difficulties[0].clone(),
            Arc::clone(source.participants()),
            original.input_snapshot().clone(),
            Vec::new(),
        )
        .unwrap()
        .with_mapping_snapshot(Arc::clone(source.mapping()))
        .with_runtime_battle_route();
        if let Some(challenge) = catalog
            .cyclical_challenges()
            .iter()
            .find(|challenge| challenge.area == area)
        {
            entry = entry.with_cyclical_refresh(
                DivergentUniverseCyclicalRefresh::new(1, challenge.id.clone()).unwrap(),
            );
        }
        let plain = factory.compile(entry).unwrap();
        let flow = factory.compile_position_profile(plain, &inputs).unwrap();
        let activity = flow
            .start(
                ActivityInstanceId::new(1).unwrap(),
                ActivityMasterSeed::from_u64(27_003),
            )
            .unwrap()
            .into_activity();
        assert!(activity.player_view().terminal().is_none());
        assert_eq!(flow.area(), &area);
    }
}

#[test]
fn native_position_profile_actual_source_rooms_menu_effects_and_encoded_replay_reconstruct() {
    let source = fire_fixture(1);
    let fresh = fire_fixture(1);
    let runtime = source.factory().weighted_curio_runtime().unwrap();
    let id = &source
        .factory()
        .decision_catalog()
        .weighted_curio_deflagrations()[0]
        .weighted_curio;
    let key = u64::try_from(
        runtime
            .candidates()
            .iter()
            .position(|candidate| candidate == id)
            .unwrap(),
    )
    .unwrap()
        + 1;
    for family in FAMILIES {
        let inputs = recipe(&source, 0);
        let flow = source
            .factory()
            .compile_position_profile(base(&source, family), &inputs)
            .unwrap();
        let rebuilt = fresh
            .factory()
            .compile_position_profile(base(&fresh, family), &inputs)
            .unwrap();
        let original = source.policy().unwrap();
        let hints = source.factory().decision_catalog().domain_decks()[0]
            .cards
            .iter()
            .filter(|card| card.kind == DomainCardKind::Reforge)
            .map(|card| {
                ActivityOptionHint::new(
                    ActivityOptionId::new(card.instance.get()).unwrap(),
                    ActivityScoreComponents::new(10_000, 0, 0, 0, 0).unwrap(),
                )
            })
            .collect();
        let policy = DivergentUniverseBaselinePolicy::new(
            ActivityBaselineHints::new(hints).unwrap(),
            original.encounter_group().clone(),
            original.encounter_stage(),
            512,
        )
        .unwrap();
        let seed = 26_801;
        let mut activity = flow
            .start(
                ActivityInstanceId::new(1).unwrap(),
                ActivityMasterSeed::from_u64(seed),
            )
            .unwrap()
            .into_activity();
        let mut changes = [key, CLEAR_EQUIPMENT, key].into_iter();
        let mut steps = Vec::new();
        let mut equipment = 0;
        let mut respite = 0;
        for _ in 0..policy.max_steps() {
            if activity.player_view().terminal().is_some() {
                break;
            }
            if rebuilt.offered_respite_service(&activity).is_some() {
                respite += 1;
            }
            let runner = DivergentUniverseBaselineRunner::default();
            let step = if flow.offered_weighted_curio_equipment(&activity)
                && let Some(option) = changes.next()
            {
                equipment += 1;
                let decision = activity.player_view().decision().unwrap().id();
                runner
                    .advance_selected(
                        source.factory(),
                        &rebuilt,
                        &mut activity,
                        source.core(),
                        &policy,
                        DivergentUniverseOfferedSelection::new(
                            decision,
                            ActivityOptionId::new(option).unwrap(),
                        ),
                    )
                    .unwrap()
            } else {
                runner
                    .advance(
                        source.factory(),
                        &rebuilt,
                        &mut activity,
                        source.core(),
                        &policy,
                    )
                    .unwrap()
            };
            steps.push(step);
        }
        assert!(activity.player_view().terminal().is_some());
        assert_eq!(equipment, 3);
        // The fresh immutable profile is not pointer-identical. Its Respite
        // capability must authenticate identical Adventure interaction bindings.
        assert_eq!(respite, 1);
        assert!(
            activity.player_view().completed_battle_count() >= 4,
            "initial real Battle plus three Boss rooms are required"
        );
        let events = steps
            .iter()
            .filter_map(|step| match step {
                DivergentUniverseBaselineStep::Battle { execution, .. } => Some(execution),
                _ => None,
            })
            .flat_map(|execution| execution.trace())
            .flat_map(|entry| entry.events().iter().cloned())
            .collect::<Vec<_>>();
        assert!(applications(&events) > 0);
        let recorded = flow
            .record_bound_transcript(&source, &activity, seed, steps, &policy)
            .unwrap();
        let bytes = encode_divergent_universe_replay(&recorded).unwrap();
        let verified = rebuilt
            .verify_bound_replay(&bytes, &fresh, &policy)
            .unwrap();
        assert_eq!(
            verified.final_state_hash().bytes(),
            recorded.report().final_state_hash().bytes()
        );
        assert_eq!(
            verified.battle_count(),
            activity.player_view().completed_battle_count()
        );
    }
}
