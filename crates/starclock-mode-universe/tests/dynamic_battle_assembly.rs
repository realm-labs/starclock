use std::{num::NonZeroUsize, sync::Arc};

use starclock_activity::{
    ActivityDecisionKind, ActivityExternalOutcomeId, ActivityPreparationBoundary,
};
use starclock_combat::Battle;
use starclock_mode_universe::{
    battle_technique::UniverseBattleTechniqueDefinition,
    dynamic_battle_assembler::{
        BattleAssemblyBudget, StandardUniverseBattleAssembler, StandardUniverseDynamicBattleError,
    },
    production_runtime::{StandardUniverseControllerIdentity, StandardUniverseRuntimeFactory},
};

const CORE_BUNDLE: &[u8] = include_bytes!("../../../config/generated/config.sora");
const UNIVERSE_BUNDLE: &[u8] = include_bytes!("../../../config/universe-generated/config.sora");

fn activity_and_assembler(
    seed: u64,
) -> (
    starclock_mode_universe::runtime::StandardUniverseActivity,
    Arc<StandardUniverseBattleAssembler>,
) {
    let factory = StandardUniverseRuntimeFactory::load(CORE_BUNDLE, UNIVERSE_BUNDLE).unwrap();
    let instance = factory
        .start(
            1,
            0,
            seed,
            StandardUniverseControllerIdentity {
                id: "dynamic-assembly-test",
                revision: "dynamic-assembly-test-v1",
                digest: [0x62; 32],
            },
        )
        .unwrap();
    let assembler = Arc::clone(instance.battle_assembler());
    let (_, activity, _, _, _) = instance.into_parts();
    (activity, assembler)
}

fn drive_to_pending(activity: &mut starclock_mode_universe::runtime::StandardUniverseActivity) {
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
                return;
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
    panic!("first encounter was not reached within the test budget");
}

#[test]
fn pending_encounter_is_assembled_and_sealed_from_one_current_snapshot() {
    let (mut activity, assembler) = activity_and_assembler(0x6023);
    let initial_bytes = activity.graph().canonical_state_bytes();
    assert!(matches!(
        assembler.start_pending_battle(&mut activity),
        Err(StandardUniverseDynamicBattleError::MissingPendingBattle)
    ));
    assert_eq!(activity.graph().canonical_state_bytes(), initial_bytes);

    drive_to_pending(&mut activity);
    let pending = activity
        .view()
        .pending_battle()
        .expect("encounter preparation produced a pending placeholder")
        .clone();
    let snapshot = activity.battle_start_snapshot().unwrap();
    assert!(
        !assembler
            .resolve_snapshot(&snapshot, None)
            .unwrap()
            .cache_hit()
    );
    assert!(
        assembler
            .resolve_snapshot(&snapshot, None)
            .unwrap()
            .cache_hit()
    );
    let before_start_hash = activity.view().state_hash();
    let started = assembler.start_pending_battle(&mut activity).unwrap();
    let handoff = started.handoff();

    assert!(started.cache_hit());
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

#[test]
fn stale_invalid_and_budget_failures_preserve_state_and_retry_cleanly() {
    let (mut activity, assembler) = activity_and_assembler(0x6024);
    let initial = activity.view();
    activity
        .choose_option(
            initial.state_hash(),
            initial.decision().unwrap().id(),
            initial.decision().unwrap().options()[0].id(),
        )
        .unwrap();
    let stale_snapshot = activity.battle_start_snapshot().unwrap();
    drive_to_pending(&mut activity);
    let pending_bytes = activity.graph().canonical_state_bytes();

    assert!(matches!(
        assembler.start_pending_battle_from_snapshot(&mut activity, stale_snapshot),
        Err(StandardUniverseDynamicBattleError::StaleSnapshot)
    ));
    assert_eq!(activity.graph().canonical_state_bytes(), pending_bytes);

    let current = activity.battle_start_snapshot().unwrap();
    let invalid_technique = UniverseBattleTechniqueDefinition::new(
        starclock_activity::ActivityOptionId::new(0x6024).unwrap(),
        starclock_activity::ParticipantId::new(1).unwrap(),
        starclock_combat::AbilityId::new(1).unwrap(),
        1,
        starclock_activity::TechniqueEngagement::Engage,
    )
    .unwrap();
    let entries_before_invalid = assembler.cache_entry_count();
    assert!(matches!(
        assembler.resolve_snapshot(&current, Some(invalid_technique)),
        Err(StandardUniverseDynamicBattleError::MissingTechnique)
    ));
    assert_eq!(assembler.cache_entry_count(), entries_before_invalid);
    assert_eq!(activity.graph().canonical_state_bytes(), pending_bytes);

    let constrained = assembler
        .fork_with_policy(
            NonZeroUsize::new(2).unwrap(),
            BattleAssemblyBudget::new(1_024, 64, 8, 0),
        )
        .unwrap();
    assert!(matches!(
        constrained.start_pending_battle(&mut activity),
        Err(StandardUniverseDynamicBattleError::BudgetExceeded)
    ));
    assert_eq!(activity.graph().canonical_state_bytes(), pending_bytes);

    let started = assembler.start_pending_battle(&mut activity).unwrap();
    assert_eq!(
        started.handoff().identity().combat_input_digest(),
        started.handoff().battle_spec().combat_input_digest()
    );
}

#[test]
fn bounded_dynamic_cache_hits_and_evicts_exact_activity_snapshots() {
    let (mut activity, assembler) = activity_and_assembler(0x6025);
    let bounded = assembler
        .fork_with_policy(
            NonZeroUsize::new(2).unwrap(),
            BattleAssemblyBudget::default(),
        )
        .unwrap();
    let mut resolved_keys = Vec::new();
    for _ in 0..128 {
        if let Ok(snapshot) = activity.battle_start_snapshot() {
            let resolved = bounded.resolve_snapshot(&snapshot, None).unwrap();
            resolved_keys.push(resolved.assembly_key());
            if resolved_keys.len() == 3 {
                break;
            }
        }
        let view = activity.view();
        let decision = view.decision().unwrap();
        if decision.kind() == ActivityDecisionKind::Encounter {
            let option = decision.options()[0].id();
            activity
                .engage_encounter(view.state_hash(), decision.id(), option, 5)
                .unwrap();
        } else if decision.kind() == ActivityDecisionKind::ExternalOutcome {
            activity
                .submit_external_outcome(
                    view.state_hash(),
                    decision.id(),
                    ActivityExternalOutcomeId::new(decision.options()[0].id().get()).unwrap(),
                )
                .unwrap();
        } else {
            activity
                .choose_option(view.state_hash(), decision.id(), decision.options()[0].id())
                .unwrap();
        }
    }
    assert_eq!(resolved_keys.len(), 3);
    assert!(resolved_keys.windows(2).all(|pair| pair[0] != pair[1]));
    assert_eq!(bounded.cache_entry_count(), 2);
    assert!(bounded.cache_metrics().evictions() >= 2);
    assert!(
        bounded
            .resolve_snapshot(&activity.battle_start_snapshot().unwrap(), None)
            .unwrap()
            .cache_hit()
    );
}
