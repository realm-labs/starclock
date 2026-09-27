//! Actual fixed Blank flow dispatch; other guide payloads are explicit probes.

use super::{DECK, flag, probe};
use crate::digest::CanonicalDigestBuilder;
use crate::divergent_universe::{
    DivergentUniverseBaselineFixture, DivergentUniverseBaselineRunner,
    DivergentUniverseCurrencyKind, DivergentUniverseEntry, DivergentUniverseFlowInstance,
    battle_room::{BattleRoomSelection, CompiledBattleRoom},
    blank_room::{CompiledBlankRoom, LEAVE_BLANK},
    domain_route::{CompiledDomainRoute, DomainRoomComposition},
    state::{ROOM_DOORS_OPEN_SLOT, ROOM_FINISHED_SLOT},
    tests::{currency_balance, instance, reward_draws},
};
use starclock_activity::{
    ActivityConfigDigest, ActivityDecisionKind, ActivityMasterSeed, ActivityOptionId,
    ActivityProgramDefinition, ActivityProgramId, ActivityRandomPolicies, ActivityStateDefinition,
    GraphActivity, GraphActivityCommandError, GraphActivityDefinition, GraphActivityNodeProgram,
};
use starclock_data::{
    divergent_universe_catalog::{
        DivergentUniverseAreaId, DivergentUniverseDifficultyId, DivergentUniverseRunFamily,
    },
    divergent_universe_decisions::BattleRewardDomain,
    divergent_universe_domain_layout::FixedDomainKind,
};
use std::sync::Arc;

struct Profile {
    unbound: DivergentUniverseFlowInstance,
    flow: DivergentUniverseFlowInstance,
    blank: CompiledBlankRoom,
}

fn compile(fixture: &DivergentUniverseBaselineFixture, deck: usize, changed: bool) -> Profile {
    let factory = fixture.factory();
    let ordinary = fixture.flow(DivergentUniverseRunFamily::Ordinary).unwrap();
    let base = factory
        .compile(
            DivergentUniverseEntry::new(
                DivergentUniverseAreaId::new("divergent-universe.area.103").unwrap(),
                DivergentUniverseDifficultyId::new("divergent-universe.difficulty.1001").unwrap(),
                Arc::clone(fixture.participants()),
                ordinary.input_snapshot().clone(),
                Vec::new(),
            )
            .unwrap()
            .with_mapping_snapshot(Arc::clone(fixture.mapping()))
            .with_runtime_battle_route(),
        )
        .unwrap();
    let pool = factory.decision_catalog().encounter_pool();
    let battle = factory
        .battle_room_compiler(BattleRoomSelection {
            group: pool.encounter_group.clone(),
            stage: pool.candidate_stages[0].clone(),
            domain: BattleRewardDomain::Boss,
        })
        .unwrap();
    let mut blanks = Vec::new();
    let mut battles: Vec<CompiledBattleRoom> = Vec::new();
    let mut route: CompiledDomainRoute = factory
        .compile_curio_domain_route(
            base.area(),
            &factory.decision_catalog().domain_decks()[deck].key,
            3,
            DECK,
            |context| match context.composition {
                DomainRoomComposition::Fixed(FixedDomainKind::Blank) => {
                    let room = factory.blank_room_compiler().compile(context).unwrap();
                    let fragment = room.fragment().clone();
                    blanks.push(room);
                    Ok(fragment)
                }
                DomainRoomComposition::Fixed(FixedDomainKind::Boss) => {
                    let room = battle.compile(context).unwrap();
                    let fragment = room.fragment().clone();
                    battles.push(room);
                    Ok(fragment)
                }
                _ => probe(context),
            },
        )
        .unwrap();
    let blank = blanks.remove(0);
    assert!(blanks.is_empty());
    if changed {
        let record = route
            .programs
            .iter_mut()
            .find(|record| record.node() == blank.menu_node())
            .unwrap();
        *record = GraphActivityNodeProgram::new(
            blank.menu_node(),
            ActivityProgramDefinition::new(
                ActivityProgramId::new(blank.menu_node().get()).unwrap(),
                Vec::new(),
            )
            .unwrap(),
        );
    }
    let mut slots = base.definition().state_definition().slots().to_vec();
    slots.extend(route.deck.slot_definitions().unwrap());
    let mut owner = CanonicalDigestBuilder::new();
    owner.update(b"du.test.fixed-blank.actual-boss-proxy.other-guide-payloads-probes.width-three");
    owner.update(factory.decision_catalog().digest());
    owner.update(
        factory.decision_catalog().domain_decks()[deck]
            .key
            .as_bytes(),
    );
    owner.update(blank.configuration_digest());
    owner.update([u8::from(changed)]);
    let payload = ActivityConfigDigest::new(owner.finalize()).unwrap();
    let identity = factory
        .battle_room_identity(&base, &route.graph, &battles, payload)
        .unwrap();
    let definition = Arc::new(
        GraphActivityDefinition::new(
            identity,
            route.graph.clone(),
            ActivityStateDefinition::new(
                slots,
                base.definition().state_definition().inventories().to_vec(),
                base.definition().state_definition().modifiers().to_vec(),
            )
            .unwrap()
            .with_logical_scopes(route.logical_scopes.clone()),
            Arc::clone(base.definition().participants()),
            route.programs.clone(),
            None,
            ActivityRandomPolicies::new(Vec::new(), route.random_offers.clone()),
        )
        .unwrap(),
    );
    let unbound = factory
        .bind_battle_rooms(base, definition, &battles, payload)
        .unwrap();
    let flow = if changed {
        unbound.clone()
    } else {
        let flow = factory
            .bind_position_domain_route(unbound.clone(), &route)
            .unwrap();
        factory
            .bind_position_blank_rooms(flow, std::slice::from_ref(&blank))
            .unwrap()
    };
    Profile {
        unbound,
        flow,
        blank,
    }
}

fn start(profile: &Profile) -> GraphActivity {
    profile
        .flow
        .start(instance(24800), ActivityMasterSeed::from_u64(24800))
        .unwrap()
        .into_activity()
}

#[test]
fn blank_profile_controller_leaves_actual_fixed_room_and_reconstructs_all_nine_decks() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let policy = fixture.policy().unwrap();
    for deck in 0..fixture.factory().decision_catalog().domain_decks().len() {
        let profile = compile(&fixture, deck, false);
        let fresh = compile(&fixture, deck, false);
        let mut activity = start(&profile);
        let mut rebuilt = start(&fresh);
        let before = activity.canonical_state_bytes();
        assert!(profile.flow.offered_blank_exit(&activity));
        assert!(!profile.unbound.offered_blank_exit(&activity));
        assert!(!profile.flow.has_position_domain_hand(&activity).unwrap());
        assert_eq!(activity.current_node(), profile.blank.menu_node());
        assert!(flag(&activity, ROOM_FINISHED_SLOT));
        assert!(flag(&activity, ROOM_DOORS_OPEN_SLOT));
        assert_eq!(reward_draws(&activity), 0);
        let currency = profile
            .flow
            .economy()
            .currency(DivergentUniverseCurrencyKind::CosmicFragment)
            .key();
        assert_eq!(currency_balance(&activity, currency), 0);
        assert_eq!(activity.canonical_state_bytes(), before);
        let old = activity.player_view().decision().unwrap().clone();
        let entry_hash = activity.state_hash();
        assert_eq!(old.options()[0].id().get(), LEAVE_BLANK);
        let controller = DivergentUniverseBaselineRunner::default();
        let step = controller
            .advance(
                fixture.factory(),
                &profile.flow,
                &mut activity,
                fixture.core(),
                &policy,
            )
            .unwrap();
        let fresh_step = controller
            .advance(
                fixture.factory(),
                &fresh.flow,
                &mut rebuilt,
                fixture.core(),
                &policy,
            )
            .unwrap();
        assert_eq!(step, fresh_step);
        assert_eq!(
            activity.canonical_state_bytes(),
            rebuilt.canonical_state_bytes()
        );
        assert_eq!(activity.debug_view(), rebuilt.debug_view());
        assert!(!profile.flow.offered_blank_exit(&activity));
        assert!(profile.flow.has_position_domain_hand(&activity).unwrap());
        assert_eq!(
            activity.player_view().decision().unwrap().kind(),
            ActivityDecisionKind::Route
        );
        assert_eq!(reward_draws(&activity), 0);
        assert_eq!(currency_balance(&activity, currency), 0);
        let after = activity.canonical_state_bytes();
        let hash = activity.state_hash();
        assert!(matches!(
            profile
                .flow
                .leave_blank_room(&mut activity, hash, old.id(), old.options()[0].id()),
            Err(GraphActivityCommandError::DecisionNotOffered)
        ));
        assert_eq!(activity.canonical_state_bytes(), after);
        assert!(matches!(
            profile.flow.leave_blank_room(
                &mut activity,
                entry_hash,
                old.id(),
                old.options()[0].id()
            ),
            Err(GraphActivityCommandError::StaleStateHash)
        ));
        assert_eq!(activity.canonical_state_bytes(), after);
    }
}

#[test]
fn blank_profile_rejects_missing_duplicate_changed_and_repeated_capabilities() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let profile = compile(&fixture, 0, false);
    let factory = fixture.factory();
    assert!(
        factory
            .bind_position_blank_rooms(profile.unbound.clone(), &[])
            .is_err()
    );
    assert!(
        factory
            .bind_position_blank_rooms(
                profile.unbound.clone(),
                &[profile.blank.clone(), profile.blank.clone()]
            )
            .is_err()
    );
    assert!(
        factory
            .bind_position_blank_rooms(profile.flow.clone(), std::slice::from_ref(&profile.blank))
            .is_err()
    );
    let changed = compile(&fixture, 0, true);
    assert!(
        factory
            .bind_position_blank_rooms(changed.unbound, std::slice::from_ref(&changed.blank))
            .is_err()
    );
    assert!(
        factory
            .bind_position_blank_rooms(
                fixture.flow(DivergentUniverseRunFamily::Ordinary).unwrap(),
                std::slice::from_ref(&profile.blank)
            )
            .is_err()
    );
}

#[test]
fn blank_profile_rejected_leave_and_foreign_observation_are_state_and_rng_inert() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let profile = compile(&fixture, 0, false);
    let foreign = compile(&fixture, 1, false);
    let mut activity = start(&profile);
    let other = start(&foreign);
    let before = activity.canonical_state_bytes();
    let debug = activity.debug_view();
    assert!(!profile.flow.offered_blank_exit(&other));
    let offer = activity.player_view().decision().unwrap().clone();
    let hash = activity.state_hash();
    assert!(
        profile
            .flow
            .leave_blank_room(
                &mut activity,
                hash,
                offer.id(),
                ActivityOptionId::new(1).unwrap()
            )
            .is_err()
    );
    assert_eq!(activity.canonical_state_bytes(), before);
    assert_eq!(activity.debug_view(), debug);
    assert!(
        foreign
            .flow
            .leave_blank_room(&mut activity, hash, offer.id(), offer.options()[0].id())
            .is_err()
    );
    assert_eq!(activity.canonical_state_bytes(), before);
    assert_eq!(activity.debug_view(), debug);
}
