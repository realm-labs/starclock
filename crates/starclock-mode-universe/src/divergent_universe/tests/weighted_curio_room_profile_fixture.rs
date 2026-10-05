//! Actual source-hand equipment admission and real Boss proxy consumers.
//! Other room payloads are probes, not a released complete-run claim.
use crate::baseline_controller::{
    ActivityBaselineHints, ActivityOptionHint, ActivityScoreComponents,
};
use crate::digest::CanonicalDigestBuilder;
use crate::divergent_universe::{
    DivergentUniverseBaselineFixture, DivergentUniverseBaselinePolicy,
    DivergentUniverseBaselineRunner, DivergentUniverseBaselineStep, DivergentUniverseFlowInstance,
    DivergentUniverseOfferedSelection,
    battle_room::BattleRoomSelection,
    domain_deck::DomainDeckSlots,
    domain_route::DomainRoomComposition,
    tests::{
        battle_room::{base, probe},
        instance,
    },
    weighted_curio::{
        WeightedCurioSlotLimit,
        room::{CompiledWeightedCurioRoom, WeightedCurioRoomSlots},
    },
};
use starclock_activity::{
    ActivityConfigDigest, ActivityMasterSeed, ActivityOptionId, ActivityProgramDefinition,
    ActivityProgramId, ActivityRandomPolicies, ActivitySlotId, ActivityStateDefinition,
    GraphActivity, GraphActivityDefinition, GraphActivityNodeProgram,
};
use starclock_data::{
    divergent_universe_catalog::DivergentUniverseRunFamily,
    divergent_universe_decisions::BattleRewardDomain,
    divergent_universe_domain_decks::DomainCardKind,
    divergent_universe_domain_layout::FixedDomainKind,
};
use std::sync::Arc;

pub(super) const FAMILIES: [DivergentUniverseRunFamily; 2] = [
    DivergentUniverseRunFamily::Ordinary,
    DivergentUniverseRunFamily::Cyclical,
];
const DECK: DomainDeckSlots = DomainDeckSlots {
    draw: slot(80),
    discard: slot(81),
    selected: slot(82),
    accepted: slot(83),
};
const HOST: WeightedCurioRoomSlots = WeightedCurioRoomSlots {
    changes: slot(90),
    accepted: slot(91),
};
const fn slot(raw: u32) -> ActivitySlotId {
    ActivitySlotId::new(raw).unwrap()
}

pub(super) struct Profile {
    pub flow: DivergentUniverseFlowInstance,
    pub unbound: DivergentUniverseFlowInstance,
    pub rooms: Vec<CompiledWeightedCurioRoom>,
    pub policy: DivergentUniverseBaselinePolicy,
}
pub(super) fn compile(
    source: &DivergentUniverseBaselineFixture,
    family: DivergentUniverseRunFamily,
    deck: usize,
    capacity: u16,
    changed: bool,
) -> Profile {
    let factory = source.factory();
    let base = base(source, family);
    let pool = factory.decision_catalog().encounter_pool();
    let battle = factory
        .battle_room_compiler(BattleRoomSelection {
            group: pool.encounter_group.clone(),
            stage: pool.candidate_stages[0].clone(),
            domain: BattleRewardDomain::Boss,
        })
        .unwrap();
    let compiler = factory
        .weighted_curio_room_compiler(WeightedCurioSlotLimit::new(capacity).unwrap(), HOST)
        .unwrap();
    let mut rooms = Vec::new();
    let mut battles = Vec::new();
    let key = &factory.decision_catalog().domain_decks()[deck].key;
    let mut route = factory
        .compile_curio_domain_route(base.area(), key, 3, DECK, |context| {
            match context.composition {
                DomainRoomComposition::Card(DomainCardKind::Reforge) => {
                    let room = compiler.compile(context).unwrap();
                    let fragment = room.fragment().clone();
                    rooms.push(room);
                    Ok(fragment)
                }
                DomainRoomComposition::Fixed(FixedDomainKind::Boss) => {
                    let room = battle.compile(context).unwrap();
                    let fragment = room.fragment().clone();
                    battles.push(room);
                    Ok(fragment)
                }
                _ => probe(context),
            }
        })
        .unwrap();
    assert!(!rooms.is_empty());
    assert_eq!(battles.len(), 3);
    if changed {
        let node = rooms[0].menu_node();
        *route
            .programs
            .iter_mut()
            .find(|program| program.node() == node)
            .unwrap() = GraphActivityNodeProgram::new(
            node,
            ActivityProgramDefinition::new(ActivityProgramId::new(node.get()).unwrap(), Vec::new())
                .unwrap(),
        );
    }
    let mut slots = base.definition().state_definition().slots().to_vec();
    // Existing equipment declaration is identical; add only isolated service slots.
    for declaration in rooms[0].slot_definitions() {
        if let Some(existing) = slots.iter().find(|slot| slot.id() == declaration.id()) {
            assert_eq!(existing, declaration);
        } else {
            slots.push(declaration.clone());
        }
    }
    slots.extend(route.deck.slot_definitions().unwrap());
    let mut owner = CanonicalDigestBuilder::new();
    owner.update(
        b"du.test.source-reforge-equipment.three-boss-proxies.other-payloads-probes.width-three",
    );
    owner.update(factory.decision_catalog().digest());
    owner.update(key.as_bytes());
    for room in &rooms {
        owner.update(room.configuration_digest());
    }
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
    let deck_bound = factory.bind_position_domain_route(unbound, &route).unwrap();
    let flow = if changed {
        deck_bound.clone()
    } else {
        factory
            .bind_position_weighted_curio_rooms(deck_bound.clone(), &rooms)
            .unwrap()
    };
    let original = source.policy().unwrap();
    let hints = factory.decision_catalog().domain_decks()[deck]
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
        256,
    )
    .unwrap();
    Profile {
        flow,
        unbound: deck_bound,
        rooms,
        policy,
    }
}
pub(super) fn start(profile: &Profile) -> GraphActivity {
    profile
        .flow
        .start(instance(26_801), ActivityMasterSeed::from_u64(26_801))
        .unwrap()
        .into_activity()
}
pub(super) fn advance(
    source: &DivergentUniverseBaselineFixture,
    profile: &Profile,
    activity: &mut GraphActivity,
) -> DivergentUniverseBaselineStep {
    DivergentUniverseBaselineRunner::default()
        .advance(
            source.factory(),
            &profile.flow,
            activity,
            source.core(),
            &profile.policy,
        )
        .unwrap()
}
pub(super) fn select(
    source: &DivergentUniverseBaselineFixture,
    profile: &Profile,
    activity: &mut GraphActivity,
    option: u64,
) -> DivergentUniverseBaselineStep {
    let decision = activity.player_view().decision().unwrap().id();
    DivergentUniverseBaselineRunner::default()
        .advance_selected(
            source.factory(),
            &profile.flow,
            activity,
            source.core(),
            &profile.policy,
            DivergentUniverseOfferedSelection::new(
                decision,
                ActivityOptionId::new(option).unwrap(),
            ),
        )
        .unwrap()
}
pub(super) fn ready(source: &DivergentUniverseBaselineFixture, profile: &Profile) -> GraphActivity {
    let mut activity = start(profile);
    for _ in 0..256 {
        if profile.flow.offered_weighted_curio_equipment(&activity) {
            return activity;
        }
        assert!(
            activity.player_view().terminal().is_none(),
            "pinned trace must select a Reforge card"
        );
        advance(source, profile, &mut activity);
    }
    panic!("bounded source-hand trace did not enter equipment room");
}
