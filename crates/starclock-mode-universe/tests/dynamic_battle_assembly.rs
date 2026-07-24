use std::sync::Arc;

use starclock_activity::{
    ActivityDecisionKind, ActivityExternalOutcomeId, ActivityPreparationBoundary,
};
use starclock_combat::Battle;
use starclock_mode_universe::{
    dynamic_battle_assembler::StandardUniverseDynamicBattleError,
    production_runtime::{StandardUniverseControllerIdentity, StandardUniverseRuntimeFactory},
};

const CORE_BUNDLE: &[u8] = include_bytes!("../../../config/generated/config.sora");
const UNIVERSE_BUNDLE: &[u8] = include_bytes!("../../../config/universe-generated/config.sora");

#[test]
fn pending_encounter_is_assembled_and_sealed_from_one_current_snapshot() {
    let factory = StandardUniverseRuntimeFactory::load(CORE_BUNDLE, UNIVERSE_BUNDLE).unwrap();
    let instance = factory
        .start(
            1,
            0,
            0x6023,
            StandardUniverseControllerIdentity {
                id: "dynamic-assembly-test",
                revision: "dynamic-assembly-test-v1",
                digest: [0x62; 32],
            },
        )
        .unwrap();
    let assembler = Arc::clone(instance.battle_assembler());
    let (_, mut activity, _, _, _) = instance.into_parts();

    let initial_bytes = activity.graph().canonical_state_bytes();
    assert!(matches!(
        assembler.start_pending_battle(&mut activity),
        Err(StandardUniverseDynamicBattleError::MissingPendingBattle)
    ));
    assert_eq!(activity.graph().canonical_state_bytes(), initial_bytes);

    for _ in 0..128 {
        let view = activity.view();
        let decision = view.decision().expect("nonterminal Activity decision");
        match decision.kind() {
            ActivityDecisionKind::Encounter => {
                let option = decision.options()[0].id();
                let prepared = activity
                    .engage_encounter(view.state_hash(), decision.id(), option, 5)
                    .unwrap();
                if prepared.boundary() == ActivityPreparationBoundary::Decision {
                    let preparation = activity.preparation_view().unwrap();
                    let normal = preparation.options()[0].id();
                    assert_eq!(
                        activity
                            .choose_preparation_option(activity.view().state_hash(), normal)
                            .unwrap(),
                        ActivityPreparationBoundary::BattleReady
                    );
                }
                break;
            }
            ActivityDecisionKind::ExternalOutcome => {
                activity
                    .submit_external_outcome(
                        view.state_hash(),
                        decision.id(),
                        ActivityExternalOutcomeId::new(decision.options()[0].id().get()).unwrap(),
                    )
                    .unwrap();
            }
            ActivityDecisionKind::Choice
            | ActivityDecisionKind::Reward
            | ActivityDecisionKind::Route => {
                activity
                    .choose_option(view.state_hash(), decision.id(), decision.options()[0].id())
                    .unwrap();
            }
            other => panic!("unexpected decision before first encounter: {other:?}"),
        }
    }

    let pending = activity
        .view()
        .pending_battle()
        .expect("encounter preparation produced a pending placeholder")
        .clone();
    let snapshot = activity.battle_start_snapshot().unwrap();
    let before_start_hash = activity.view().state_hash();
    let started = assembler.start_pending_battle(&mut activity).unwrap();
    let handoff = started.handoff();

    assert!(!started.cache_hit());
    assert_eq!(started.assembly_key().contributions(), snapshot.digest());
    assert_eq!(started.assembly_key().carry(), snapshot.carry_digest());
    assert_ne!(
        handoff.identity().assembly_digest(),
        pending.assembly_digest()
    );
    assert_eq!(
        handoff.identity().combat_input_digest(),
        handoff.battle_spec().combat_input_digest()
    );
    assert_eq!(
        handoff.identity().assembly_digest(),
        handoff.battle_spec().assembly_digest()
    );
    assert_ne!(activity.view().state_hash(), before_start_hash);
    assert!(
        handoff
            .contract_digest()
            .bytes()
            .iter()
            .any(|byte| *byte != 0)
    );
    assert!(assembler.cache_metrics().misses() >= 1);
    assert!(assembler.cache_entry_count() >= 2);

    let battle = Battle::create(
        Arc::clone(started.combat_catalog()),
        handoff.battle_spec().clone(),
        handoff.identity().seed(),
    )
    .expect("dynamically assembled handoff is executable");
    assert_eq!(
        battle.view().identity().combat_input_digest(),
        handoff.identity().combat_input_digest()
    );
}
