//! Shared controlled source-position profile for coexisting Respite services.
//! Three real Boss proxies; other payloads remain probes, NOT complete runs.

use super::{Profile, REFORGE_SLOTS, SLOTS, base, probe};
use crate::digest::CanonicalDigestBuilder;
use crate::divergent_universe::{
    DivergentUniverseBaselineFixture, DivergentUniverseWorkbenchBlessingReforgePolicy,
    battle_room::BattleRoomSelection,
    domain_route::DomainRoomComposition,
    respite_room::equation_reforge::RespiteEquationReforgePolicy,
    respite_room::reforge::RespiteReforgeSlots,
    respite_room::{CompiledRespiteRoom, RespiteEnhancementPolicy},
};
use starclock_activity::{
    ActivityConfigDigest, ActivityRandomPolicies, ActivitySlotId, ActivityStateDefinition,
    GraphActivityDefinition,
};
use starclock_data::{
    divergent_universe_blessing_catalog::DivergentUniverseBlessingGroupId,
    divergent_universe_catalog::DivergentUniverseRunFamily,
    divergent_universe_decisions::BattleRewardDomain,
    divergent_universe_domain_layout::FixedDomainKind,
    divergent_universe_service_catalog::DivergentUniverseWorkbenchId,
};
use std::sync::Arc;

pub(super) const EQUATION_SLOTS: RespiteReforgeSlots = RespiteReforgeSlots {
    selected: ActivitySlotId::new(74).unwrap(),
    offers: ActivitySlotId::new(75).unwrap(),
    completed: ActivitySlotId::new(76).unwrap(),
    accepted: ActivitySlotId::new(77).unwrap(),
};
pub(super) struct Services<'a> {
    pub(super) blessings: Option<&'a DivergentUniverseBlessingGroupId>,
    pub(super) equations: Option<RespiteEquationReforgePolicy>,
    pub(super) equation_first: bool,
}
pub(super) fn compile_services(
    fixture: &DivergentUniverseBaselineFixture,
    family: DivergentUniverseRunFamily,
    budget: u64,
    price: u64,
    services: Services<'_>,
) -> Profile {
    let factory = fixture.factory();
    let base = base(fixture, family);
    let pool = factory.decision_catalog().encounter_pool();
    let battle = factory
        .battle_room_compiler(BattleRoomSelection {
            group: pool.encounter_group.clone(),
            stage: pool.candidate_stages[0].clone(),
            domain: BattleRewardDomain::Boss,
        })
        .unwrap();
    let workbench = if services.equations.is_some() {
        "divergent-universe.workbench.102"
    } else {
        "divergent-universe.workbench.101"
    };
    let respite = factory
        .respite_room_compiler(
            &DivergentUniverseWorkbenchId::new(workbench).unwrap(),
            RespiteEnhancementPolicy::new(budget, price).unwrap(),
        )
        .unwrap();
    let mut battles = Vec::new();
    let mut rest = None;
    let equation = |room: CompiledRespiteRoom| match services.equations {
        Some(policy) => room.with_equation_reforge(policy, EQUATION_SLOTS).unwrap(),
        None => room,
    };
    let route = factory
        .compile_curio_domain_route(
            base.area(),
            &factory.decision_catalog().domain_decks()[0].key,
            3,
            SLOTS,
            |context| match context.composition {
                DomainRoomComposition::Fixed(FixedDomainKind::Boss) => {
                    let room = battle.compile(context).unwrap();
                    let fragment = room.fragment().clone();
                    battles.push(room);
                    Ok(fragment)
                }
                DomainRoomComposition::Fixed(FixedDomainKind::Respite) => {
                    let room = respite.compile(context).unwrap();
                    let room = if services.equation_first {
                        equation(room)
                    } else {
                        room
                    };
                    let room = match services.blessings {
                        Some(group) => room
                            .with_blessing_reforge(
                                group,
                                DivergentUniverseWorkbenchBlessingReforgePolicy::new(7, 3).unwrap(),
                                REFORGE_SLOTS,
                            )
                            .unwrap(),
                        None => room,
                    };
                    let room = if services.equation_first {
                        room
                    } else {
                        equation(room)
                    };
                    let fragment = room.fragment().clone();
                    rest = Some(room);
                    Ok(fragment)
                }
                _ => probe(context),
            },
        )
        .unwrap();
    let respite = rest.unwrap();
    let mut slots = base.definition().state_definition().slots().to_vec();
    slots.extend(route.deck.slot_definitions().unwrap());
    slots.extend_from_slice(respite.slot_definitions());
    let mut owner = CanonicalDigestBuilder::new();
    owner
        .update(b"du.test.respite.three-boss-proxies.first-deck.width-three.other-payloads-probes");
    owner.update(factory.decision_catalog().domain_decks()[0].key.as_bytes());
    owner.update(respite.configuration_digest());
    let payload = ActivityConfigDigest::new(owner.finalize()).unwrap();
    let identity = factory
        .battle_room_identity(&base, &route.graph, &battles, payload)
        .unwrap();
    let definition = Arc::new(
        GraphActivityDefinition::new(
            identity,
            route.graph.clone(),
            ActivityStateDefinition::new(slots, Vec::new(), Vec::new())
                .unwrap()
                .with_logical_scopes(route.logical_scopes.clone()),
            Arc::clone(base.definition().participants()),
            route.programs.clone(),
            None,
            ActivityRandomPolicies::new(Vec::new(), route.random_offers.clone()),
        )
        .unwrap(),
    );
    respite.validate_definition(&definition).unwrap();
    let flow = factory
        .bind_battle_rooms(base, definition, &battles, payload)
        .unwrap();
    let flow = factory.bind_position_domain_route(flow, &route).unwrap();
    let unbound = flow.clone();
    let flow = factory
        .bind_position_respite_rooms(flow, std::slice::from_ref(&respite))
        .unwrap();
    Profile {
        flow,
        unbound,
        route,
        respite,
    }
}
