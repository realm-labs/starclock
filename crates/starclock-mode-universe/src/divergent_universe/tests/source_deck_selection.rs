use super::{domain_choices::advance, instance};
use crate::divergent_universe::{
    DivergentUniverseBaselineFixture, DivergentUniverseBaselineStep,
    encode_divergent_universe_replay, record_divergent_universe_transcript, state::EQUATIONS_SLOT,
    verify_divergent_universe_replay,
};
use starclock_activity::{
    ActivityMasterSeed, ActivityOperation, ActivityOptionId, ActivityProgramDefinition,
    ActivityProgramId, ActivityTerminalOutcome,
};
use starclock_data::divergent_universe_catalog::DivergentUniverseRunFamily;

const FAMILIES: [DivergentUniverseRunFamily; 2] = [
    DivergentUniverseRunFamily::Ordinary,
    DivergentUniverseRunFamily::Cyclical,
];

#[test]
fn source_deck_selection_initializes_each_authored_deck_and_rejects_invalid_commands() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    for family in FAMILIES {
        let flow = fixture.flow_with_source_deck_selection(family).unwrap();
        let foreign = fixture.flow(family).unwrap();
        assert_ne!(
            flow.definition().identity(),
            foreign.definition().identity()
        );
        for index in 0..9 {
            let mut activity = flow
                .start(instance(1), ActivityMasterSeed::from_u64(24201))
                .unwrap()
                .into_activity();
            assert!(flow.selected_source_deck(&activity).unwrap().is_none());
            let before = activity.canonical_state_bytes();
            let options = flow.source_deck_options(&activity).unwrap();
            let (selected, expected) = &options[index];
            let expected_ids = expected
                .cards
                .iter()
                .map(|card| card.instance.get())
                .collect::<Vec<_>>();
            let expected_key = expected.key.clone();
            let selected = *selected;
            let offered = activity.player_view().decision().unwrap().clone();
            let hash = activity.state_hash();
            assert!(
                activity
                    .choose_option(hash, offered.id(), selected)
                    .is_err()
            );
            assert_eq!(activity.canonical_state_bytes(), before);
            assert!(
                foreign
                    .choose_source_deck(&mut activity, hash, offered.id(), selected)
                    .is_err()
            );
            assert!(
                flow.choose_source_deck(
                    &mut activity,
                    hash,
                    offered.id(),
                    ActivityOptionId::new(999).unwrap()
                )
                .is_err()
            );
            assert_eq!(activity.canonical_state_bytes(), before);
            assert!(
                flow.choose_initial_equation(&mut activity, hash, offered.id(), selected)
                    .is_err()
            );
            assert_eq!(activity.canonical_state_bytes(), before);
            flow.choose_source_deck(&mut activity, hash, offered.id(), selected)
                .unwrap();
            let (definition, piles) = flow.selected_source_deck(&activity).unwrap().unwrap();
            assert_eq!(definition.key, expected_key);
            assert_eq!(
                piles.draw.iter().map(|card| card.get()).collect::<Vec<_>>(),
                expected_ids
            );
            assert!(piles.hand.is_empty() && piles.discard.is_empty() && piles.selected.is_none());
            let committed = activity.canonical_state_bytes();
            assert!(flow.source_deck_options(&activity).is_err());
            assert!(
                flow.choose_source_deck(&mut activity, hash, offered.id(), selected)
                    .is_err()
            );
            assert_eq!(activity.canonical_state_bytes(), committed);
            let fresh = flow
                .start(instance(2), ActivityMasterSeed::from_u64(24201))
                .unwrap()
                .into_activity();
            assert!(flow.selected_source_deck(&fresh).unwrap().is_none());
        }
    }
}

#[test]
fn source_deck_selection_late_offer_failure_restores_the_entire_choice() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let flow = fixture
        .flow_with_source_deck_selection(FAMILIES[0])
        .unwrap();
    let mut activity = flow
        .start(instance(1), ActivityMasterSeed::from_u64(24201))
        .unwrap()
        .into_activity();
    let count = fixture
        .factory()
        .bundle
        .equation_catalog()
        .equations()
        .len();
    let program = ActivityProgramDefinition::new(
        ActivityProgramId::new(24202).unwrap(),
        vec![ActivityOperation::SetOrderedIdSet {
            slot: EQUATIONS_SLOT,
            values: (1..=u64::try_from(count).unwrap()).collect(),
        }],
    )
    .unwrap();
    activity
        .apply_boundary_program(activity.state_hash(), &program)
        .unwrap();
    let before = activity.canonical_state_bytes();
    let offered = activity.player_view().decision().unwrap().clone();
    let hash = activity.state_hash();
    assert!(
        flow.choose_source_deck(&mut activity, hash, offered.id(), offered.options()[0].id())
            .is_err()
    );
    assert_eq!(activity.canonical_state_bytes(), before);
    assert!(flow.selected_source_deck(&activity).unwrap().is_none());
}

#[test]
fn source_deck_selection_persists_through_real_battles_and_encoded_replay() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let fresh = DivergentUniverseBaselineFixture::production().unwrap();
    for family in FAMILIES {
        let flow = fixture.flow_with_source_deck_selection(family).unwrap();
        for index in [0, 4, 8] {
            let seed = 24201;
            let mut activity = flow
                .start(instance(1), ActivityMasterSeed::from_u64(seed))
                .unwrap()
                .into_activity();
            let (option, definition) = flow.source_deck_options(&activity).unwrap()[index];
            let key = definition.key.clone();
            let count = definition.cards.len();
            let mut steps = vec![advance(&fixture, &flow, &mut activity, Some(option.get()))];
            let mut battles = 0;
            while activity.player_view().terminal().is_none() {
                assert!(steps.len() < 100);
                let step = advance(&fixture, &flow, &mut activity, None);
                if matches!(step, DivergentUniverseBaselineStep::Battle { .. }) {
                    battles += 1;
                }
                steps.push(step);
                let (definition, piles) = flow.selected_source_deck(&activity).unwrap().unwrap();
                assert_eq!(definition.key, key);
                assert_eq!(piles.draw.len(), count);
                assert!(piles.hand.is_empty() && piles.discard.is_empty());
            }
            assert_eq!(battles, 3);
            assert_eq!(
                activity.player_view().terminal(),
                Some(ActivityTerminalOutcome::Completed)
            );
            let recorded =
                record_divergent_universe_transcript(&fixture, &flow, &activity, seed, steps)
                    .unwrap();
            let bytes = encode_divergent_universe_replay(&recorded).unwrap();
            let verified = verify_divergent_universe_replay(&bytes, &fresh).unwrap();
            assert_eq!(verified.battle_count(), 3);
            assert_eq!(
                verified.final_state_hash().bytes(),
                activity.state_hash().bytes()
            );
        }
    }
}
