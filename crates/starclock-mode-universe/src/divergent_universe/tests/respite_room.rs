//! Source Respite enhancement between real proxy battles, not full room parity.

use super::{SLOTS, base, probe};
#[path = "respite_equation_reforge.rs"]
mod equation_reforge;
#[path = "respite_profile.rs"]
mod profile;
#[path = "respite_reforge.rs"]
mod reforge;
use crate::divergent_universe::{
    DivergentUniverseBaselineFixture, DivergentUniverseBaselinePolicy,
    DivergentUniverseBaselineRunner, DivergentUniverseFlowInstance,
    domain_route::{CompiledDomainRoute, DomainRoomComposition},
    economy::DivergentUniverseCurrencyKind,
    respite_room::reforge::RespiteReforgeSlots,
    respite_room::{CompiledRespiteRoom, RespiteEnhancementPolicy, RespiteRoomError},
    state::{
        BLESSING_OFFER_SOURCE_SLOT, BLESSING_OFFERS_SLOT, CURIO_CHARGES_SLOT, CURRENCIES_SLOT,
        EQUATION_PROGRESS_DIRTY_SLOT, SERVICE_RECEIPTS_SLOT, WORKBENCH_SLOT,
    },
    tests::{acquire_target_equation, currency_balance, instance, reward_draws},
};
use profile::{Services, compile_services};
use starclock_activity::{
    ActivityDecisionId, ActivityExpression, ActivityMasterSeed, ActivityOperation,
    ActivityOptionId, ActivityProgramDefinition, ActivityProgramId, ActivityRandomPolicies,
    ActivitySlotId, ActivityTerminalOutcome, ActivityValue, GraphActivity,
    GraphActivityCommandError, GraphActivityDefinition, GraphActivityNodeProgram,
};
use starclock_data::{
    divergent_universe_blessing_catalog::DivergentUniverseBlessingGroupId,
    divergent_universe_catalog::DivergentUniverseRunFamily,
    divergent_universe_curio_catalog::DivergentUniverseCurioStateId,
    divergent_universe_domain_layout::FixedDomainKind,
    divergent_universe_service_catalog::DivergentUniverseWorkbenchId,
};
use std::sync::Arc;

const LEAVE: u64 = u64::MAX;
const REFORGE_SLOTS: RespiteReforgeSlots = RespiteReforgeSlots {
    selected: ActivitySlotId::new(70).unwrap(),
    offers: ActivitySlotId::new(71).unwrap(),
    completed: ActivitySlotId::new(72).unwrap(),
    accepted: ActivitySlotId::new(73).unwrap(),
};
struct Profile {
    flow: DivergentUniverseFlowInstance,
    unbound: DivergentUniverseFlowInstance,
    route: CompiledDomainRoute,
    respite: CompiledRespiteRoom,
}
fn compile(
    fixture: &DivergentUniverseBaselineFixture,
    family: DivergentUniverseRunFamily,
    budget: u64,
    price: u64,
) -> Profile {
    compile_config(fixture, family, budget, price, None)
}
fn compile_config(
    fixture: &DivergentUniverseBaselineFixture,
    family: DivergentUniverseRunFamily,
    budget: u64,
    price: u64,
    reforge_group: Option<&DivergentUniverseBlessingGroupId>,
) -> Profile {
    compile_services(
        fixture,
        family,
        budget,
        price,
        Services {
            blessings: reforge_group,
            equations: None,
            equation_first: false,
        },
    )
}
fn policy(fixture: &DivergentUniverseBaselineFixture) -> DivergentUniverseBaselinePolicy {
    let original = fixture.policy().unwrap();
    DivergentUniverseBaselinePolicy::new(
        original.hints().clone(),
        original.encounter_group().clone(),
        original.encounter_stage(),
        128,
    )
    .unwrap()
}
fn start(profile: &Profile) -> GraphActivity {
    profile
        .flow
        .start(instance(24500), ActivityMasterSeed::from_u64(24500))
        .unwrap()
        .into_activity()
}
fn advance(
    fixture: &DivergentUniverseBaselineFixture,
    profile: &Profile,
    activity: &mut GraphActivity,
) {
    DivergentUniverseBaselineRunner::default()
        .advance(
            fixture.factory(),
            &profile.flow,
            activity,
            fixture.core(),
            &policy(fixture),
        )
        .unwrap();
}
fn ready(fixture: &DivergentUniverseBaselineFixture, profile: &Profile) -> GraphActivity {
    let mut activity = start(profile);
    for _ in 0..128 {
        if profile
            .respite
            .menu_nodes()
            .contains(&activity.current_node())
        {
            let offer = activity.player_view().decision().unwrap().clone();
            if offer
                .options()
                .iter()
                .any(|option| option.id().get() < LEAVE - 4)
                || offer.options().len() == 1
            {
                return activity;
            }
            choose(
                profile,
                &mut activity,
                offer
                    .options()
                    .iter()
                    .find(|option| option.id().get() != LEAVE)
                    .unwrap()
                    .id()
                    .get(),
            );
            continue;
        }
        advance(fixture, profile, &mut activity);
    }
    panic!("bounded controlled run must reach the fixed Respite");
}
fn try_choose(
    profile: &Profile,
    activity: &mut GraphActivity,
    decision: ActivityDecisionId,
    option: ActivityOptionId,
) -> Result<(), GraphActivityCommandError> {
    let hash = activity.state_hash();
    profile
        .flow
        .choose_respite_service_option(activity, hash, decision, option)
}
fn choose(profile: &Profile, activity: &mut GraphActivity, option: u64) {
    let offer = activity.player_view().decision().unwrap().clone();
    try_choose(
        profile,
        activity,
        offer.id(),
        ActivityOptionId::new(option).unwrap(),
    )
    .unwrap();
}
fn mutate(activity: &mut GraphActivity, operations: Vec<ActivityOperation>) {
    let program =
        ActivityProgramDefinition::new(ActivityProgramId::new(24501).unwrap(), operations).unwrap();
    activity
        .apply_boundary_program(activity.state_hash(), &program)
        .unwrap();
}
fn assign(slot: ActivitySlotId, value: ActivityValue) -> ActivityOperation {
    match value {
        ActivityValue::BoundedCounterMap(values) => {
            ActivityOperation::SetCounterMap { slot, values }
        }
        ActivityValue::OrderedIdSet(values) => ActivityOperation::SetOrderedIdSet { slot, values },
        value => ActivityOperation::SetSlot {
            slot,
            value: ActivityExpression::Literal(value),
        },
    }
}

#[test]
fn respite_room_controller_enhances_between_actual_battles_and_reconstructs_both_families() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let blessings = fixture.factory().blessing_runtime().unwrap();
    for family in [
        DivergentUniverseRunFamily::Ordinary,
        DivergentUniverseRunFamily::Cyclical,
    ] {
        let profile = compile(&fixture, family, 7, 3);
        let fresh = compile(&fixture, family, 7, 3);
        let mut activity = start(&profile);
        let mut reconstructed = start(&fresh);
        let heat = profile
            .flow
            .economy()
            .currency(DivergentUniverseCurrencyKind::WorkbenchHeat)
            .key();
        let mut visits = 0;
        for _ in 0..128 {
            assert_eq!(
                activity.canonical_state_bytes(),
                reconstructed.canonical_state_bytes()
            );
            if activity.player_view().terminal().is_some() {
                break;
            }
            if profile
                .respite
                .menu_nodes()
                .contains(&activity.current_node())
            {
                let before_draws = reward_draws(&activity);
                let before = blessings.owned(&activity).unwrap();
                advance(&fixture, &profile, &mut activity);
                advance(&fixture, &fresh, &mut reconstructed);
                assert_eq!(reward_draws(&activity), before_draws);
                let after = blessings.owned(&activity).unwrap();
                if after.iter().filter(|owned| owned.level() == 2).count()
                    > before.iter().filter(|owned| owned.level() == 2).count()
                {
                    visits += 1;
                    assert_eq!(blessings.owned(&activity).unwrap().len(), before.len());
                    assert_eq!(currency_balance(&activity, heat), 7 - visits * 3);
                    fixture
                        .factory()
                        .equation_progress_runtime()
                        .unwrap()
                        .observations(&activity)
                        .unwrap();
                }
            } else {
                advance(&fixture, &profile, &mut activity);
                advance(&fixture, &fresh, &mut reconstructed);
            }
        }
        assert_eq!(
            activity.player_view().terminal(),
            Some(ActivityTerminalOutcome::Completed)
        );
        assert_eq!(activity.player_view().completed_battle_count(), 3);
        assert_eq!(visits, 2);
        assert_eq!(currency_balance(&activity, heat), 0);
        assert_eq!(
            activity
                .player_view()
                .slots()
                .iter()
                .find(|slot| slot.id() == WORKBENCH_SLOT)
                .unwrap()
                .value(),
            &ActivityValue::OptionalId(None)
        );
        assert_eq!(
            blessings
                .owned(&activity)
                .unwrap()
                .iter()
                .filter(|owned| owned.level() == 2)
                .count(),
            2
        );
        assert_eq!(
            activity.canonical_state_bytes(),
            reconstructed.canonical_state_bytes()
        );
    }
}

#[test]
fn respite_room_all_414_owned_identities_enhance_once_without_changing_path_counts() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let profile = compile(&fixture, DivergentUniverseRunFamily::Ordinary, 414, 1);
    let mut activity = ready(&fixture, &profile);
    let blessings = fixture.factory().blessing_runtime().unwrap();
    let equation = fixture.factory().bundle.equation_catalog().equations()[0]
        .id
        .clone();
    acquire_target_equation(
        &fixture.factory().equation_offer_runtime().unwrap(),
        &mut activity,
        &equation,
    );
    let owned = blessings.owned(&activity).unwrap();
    let remaining = blessings
        .blessings()
        .iter()
        .filter(|definition| {
            owned
                .iter()
                .all(|owned| owned.blessing() != definition.id())
        })
        .map(|definition| definition.id().clone())
        .collect::<Vec<_>>();
    // Trusted accepted reward boundary supplies every shape for this inventory
    // test; this does NOT assert that a Respite grants missing Blessings.
    let hash = activity.state_hash();
    blessings
        .acquire_accepted_identities(&mut activity, hash, &remaining)
        .unwrap();
    let progress = fixture.factory().equation_progress_runtime().unwrap();
    let snapshot = progress.observations(&activity).unwrap();
    assert!(!snapshot.is_empty());
    let draws = reward_draws(&activity);
    let mut upgrades = 0;
    for _ in 0..1000 {
        if blessings
            .owned(&activity)
            .unwrap()
            .iter()
            .all(|owned| owned.level() == 2)
        {
            break;
        }
        let option = activity
            .player_view()
            .decision()
            .unwrap()
            .options()
            .iter()
            .find(|option| option.id().get() != LEAVE)
            .unwrap()
            .id();
        choose(&profile, &mut activity, option.get());
        if option.get() < LEAVE - 4 {
            upgrades += 1;
        }
        assert_eq!(progress.observations(&activity).unwrap(), snapshot);
    }
    assert_eq!(upgrades, 414);
    assert!(
        blessings
            .owned(&activity)
            .unwrap()
            .iter()
            .all(|owned| owned.level() == 2)
    );
    assert_eq!(reward_draws(&activity), draws);
    assert_eq!(
        activity.player_view().decision().unwrap().options().len(),
        1
    );
    choose(&profile, &mut activity, LEAVE);
}

#[test]
fn respite_room_empty_budget_stale_hidden_and_changed_availability_preserve_bytes() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let family = DivergentUniverseRunFamily::Ordinary;
    let empty = compile(&fixture, family, 0, 3);
    let mut activity = ready(&fixture, &empty);
    assert_eq!(
        activity.player_view().decision().unwrap().options().len(),
        1
    );
    choose(&empty, &mut activity, LEAVE);

    let profile = compile(&fixture, family, 7, 3);
    let mut activity = ready(&fixture, &profile);
    let offer = activity.player_view().decision().unwrap().clone();
    let first = offer
        .options()
        .iter()
        .find(|option| option.id().get() != LEAVE)
        .unwrap()
        .id();
    let old = activity.state_hash();
    choose(&profile, &mut activity, first.get());
    let before = activity.canonical_state_bytes();
    assert!(activity.choose_option(old, offer.id(), first).is_err());
    let current = activity.player_view().decision().unwrap().id();
    assert!(
        activity
            .choose_option(activity.state_hash(), current, first)
            .is_err()
    );
    assert!(
        activity
            .choose_option(
                activity.state_hash(),
                current,
                ActivityOptionId::new(0x123456).unwrap()
            )
            .is_err()
    );
    assert_eq!(activity.canonical_state_bytes(), before);

    let available = activity
        .player_view()
        .decision()
        .unwrap()
        .options()
        .iter()
        .find(|option| option.id().get() != LEAVE)
        .unwrap()
        .id();
    for (slot, blocked) in [
        (
            BLESSING_OFFER_SOURCE_SLOT,
            ActivityValue::OptionalId(Some(1)),
        ),
        (
            BLESSING_OFFERS_SLOT,
            ActivityValue::BoundedCounterMap(vec![(1, 1)].into()),
        ),
        (EQUATION_PROGRESS_DIRTY_SLOT, ActivityValue::Boolean(true)),
        (WORKBENCH_SLOT, ActivityValue::OptionalId(None)),
    ] {
        let original = activity
            .player_view()
            .slots()
            .iter()
            .find(|value| value.id() == slot)
            .unwrap()
            .value()
            .clone();
        mutate(&mut activity, vec![assign(slot, blocked)]);
        let before = activity.canonical_state_bytes();
        assert!(try_choose(&profile, &mut activity, current, available).is_err());
        assert_eq!(activity.canonical_state_bytes(), before);
        mutate(&mut activity, vec![assign(slot, original)]);
    }
    let heat = profile
        .flow
        .economy()
        .currency(DivergentUniverseCurrencyKind::WorkbenchHeat)
        .key();
    mutate(
        &mut activity,
        vec![ActivityOperation::SetCounter {
            slot: CURRENCIES_SLOT,
            key: heat,
            value: ActivityExpression::Literal(ActivityValue::BoundedInteger(0)),
        }],
    );
    let before = activity.canonical_state_bytes();
    assert!(try_choose(&profile, &mut activity, current, available).is_err());
    assert_eq!(activity.canonical_state_bytes(), before);
    choose(&profile, &mut activity, LEAVE);
}

#[test]
fn respite_room_late_receipt_overflow_rolls_back_level_heat_menu_and_rng() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let profile = compile(&fixture, DivergentUniverseRunFamily::Ordinary, 7, 3);
    let mut activity = ready(&fixture, &profile);
    let runtime = fixture.factory().workbench_curse_runtime().unwrap();
    let id = DivergentUniverseWorkbenchId::new("divergent-universe.workbench.101").unwrap();
    let (_, _, receipt) = runtime.enhancement_service_keys(&id).unwrap();
    mutate(
        &mut activity,
        vec![ActivityOperation::SetCounter {
            slot: SERVICE_RECEIPTS_SLOT,
            key: receipt,
            value: ActivityExpression::Literal(ActivityValue::BoundedInteger(i64::MAX)),
        }],
    );
    let offer = activity.player_view().decision().unwrap().clone();
    let option = offer
        .options()
        .iter()
        .find(|option| option.id().get() != LEAVE)
        .unwrap()
        .id();
    let before = activity.canonical_state_bytes();
    for _ in 0..2 {
        assert!(try_choose(&profile, &mut activity, offer.id(), option).is_err());
        assert_eq!(activity.canonical_state_bytes(), before);
    }
    choose(&profile, &mut activity, LEAVE);
}

#[test]
fn respite_room_failed_next_entry_restores_leave_but_keeps_committed_enhancement() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    for family in [
        DivergentUniverseRunFamily::Ordinary,
        DivergentUniverseRunFamily::Cyclical,
    ] {
        let profile = compile(&fixture, family, 7, 3);
        let mut activity = ready(&fixture, &profile);
        let option = activity
            .player_view()
            .decision()
            .unwrap()
            .options()
            .iter()
            .find(|option| option.id().get() < LEAVE - 4)
            .unwrap()
            .id();
        choose(&profile, &mut activity, option.get());
        let curios = fixture.factory().curio_runtime().unwrap();
        let states =
            [DivergentUniverseCurioStateId::new("divergent-universe.curio-state.9071").unwrap()];
        let hash = activity.state_hash();
        curios
            .acquire_accepted_states(&mut activity, hash, &states)
            .unwrap();
        let key = curios
            .states()
            .iter()
            .find(|state| state.id() == &states[0])
            .unwrap()
            .state_key();
        // Trusted malformed counter triggers the standard late next-domain
        // guard, after its intrinsic income. Not a reachable original charge.
        mutate(
            &mut activity,
            vec![ActivityOperation::SetCounter {
                slot: CURIO_CHARGES_SLOT,
                key,
                value: ActivityExpression::Literal(ActivityValue::BoundedInteger(4)),
            }],
        );
        let before = activity.canonical_state_bytes();
        let offer = activity.player_view().decision().unwrap().id();
        for _ in 0..2 {
            assert!(
                try_choose(
                    &profile,
                    &mut activity,
                    offer,
                    ActivityOptionId::new(LEAVE).unwrap()
                )
                .is_err()
            );
            assert_eq!(activity.canonical_state_bytes(), before);
        }
        let heat = profile
            .flow
            .economy()
            .currency(DivergentUniverseCurrencyKind::WorkbenchHeat)
            .key();
        assert_eq!(currency_balance(&activity, heat), 4);
        assert!(
            fixture
                .factory()
                .blessing_runtime()
                .unwrap()
                .owned(&activity)
                .unwrap()
                .iter()
                .any(|owned| owned.level() == 2)
        );
    }
}

#[test]
fn respite_room_policy_context_and_exact_definition_checks_reject_substitutions() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    assert!(RespiteEnhancementPolicy::new(1, 0).is_err());
    assert!(RespiteEnhancementPolicy::new(u64::MAX, 1).is_err());
    assert!(RespiteEnhancementPolicy::new(1, u64::MAX).is_err());
    let profile = compile(&fixture, DivergentUniverseRunFamily::Ordinary, 7, 3);
    let different = compile(&fixture, DivergentUniverseRunFamily::Ordinary, 7, 4);
    assert!(
        fixture
            .factory()
            .bind_position_respite_rooms(
                profile.flow.clone(),
                std::slice::from_ref(&profile.respite)
            )
            .is_err()
    );
    assert!(
        fixture
            .factory()
            .bind_position_respite_rooms(
                profile.unbound.clone(),
                std::slice::from_ref(&different.respite)
            )
            .is_err()
    );
    assert!(
        fixture
            .factory()
            .bind_position_respite_rooms(
                profile.unbound.clone(),
                &[profile.respite.clone(), profile.respite.clone()]
            )
            .is_err()
    );
    let mut activity = ready(&fixture, &profile);
    let before = activity.canonical_state_bytes();
    let offer = activity.player_view().decision().unwrap().clone();
    assert!(profile.unbound.offered_respite_service(&activity).is_none());
    let hash = activity.state_hash();
    assert!(
        profile
            .unbound
            .choose_respite_service_option(&mut activity, hash, offer.id(), offer.options()[0].id())
            .is_err()
    );
    assert_eq!(activity.canonical_state_bytes(), before);
    assert_ne!(
        profile.flow.definition().identity(),
        different.flow.definition().identity()
    );
    assert!(
        profile
            .respite
            .validate_definition(different.flow.definition())
            .is_err()
    );
    let compiler = fixture
        .factory()
        .respite_room_compiler(
            &DivergentUniverseWorkbenchId::new("divergent-universe.workbench.101").unwrap(),
            RespiteEnhancementPolicy::new(7, 3).unwrap(),
        )
        .unwrap();
    for mutate_context in 0..3 {
        let mut context = profile.respite.context().clone();
        match mutate_context {
            0 => context.level = 2,
            1 => context.composition = DomainRoomComposition::Fixed(FixedDomainKind::Boss),
            _ => context.position_ordinal = 1,
        }
        assert!(matches!(
            compiler.compile(&context),
            Err(RespiteRoomError::InvalidContext)
        ));
    }
    assert!(
        fixture
            .factory()
            .respite_room_compiler(
                &DivergentUniverseWorkbenchId::new("divergent-universe.workbench.106").unwrap(),
                RespiteEnhancementPolicy::new(7, 3).unwrap(),
            )
            .is_err()
    );
    let definition = profile.flow.definition();
    let mut programs = definition.programs().to_vec();
    let node = profile.respite.context().entry_node();
    let target = programs
        .iter_mut()
        .find(|program| program.node() == node)
        .unwrap();
    *target = GraphActivityNodeProgram::new(
        node,
        profile.respite.fragment().programs[0].program().clone(),
    );
    let changed = GraphActivityDefinition::new(
        definition.identity(),
        definition.graph().clone(),
        definition.state_definition().clone(),
        Arc::clone(definition.participants()),
        programs,
        None,
        ActivityRandomPolicies::new(Vec::new(), definition.random_offers().to_vec()),
    )
    .unwrap();
    assert!(profile.respite.validate_definition(&changed).is_err());
    assert_eq!(
        profile
            .route
            .rooms
            .iter()
            .filter(|context| context.composition
                == DomainRoomComposition::Fixed(FixedDomainKind::Respite))
            .count(),
        1
    );
}
