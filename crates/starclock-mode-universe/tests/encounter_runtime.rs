use std::sync::{Arc, OnceLock};

use sha2::Digest;

use starclock_activity::{
    ActivityBattleResultContract, ActivityInstanceId, ActivityMasterSeed,
    ActivityParticipantCarryDefinition, ActivityPreparationBoundary, BattleBinding, BattleOutcome,
    BattleResult, BuildDigest, EnergyCarryPolicy, EventDigest, HpCarryPolicy, LifeCarryPolicy,
    LoadoutLockScope, OpaqueParticipantBuild, ParticipantBattleState, ParticipantId,
    ParticipantLock, ParticipantLockEntry, ParticipantPolicy, ParticipantSourceKind,
    PresenceCarryPolicy, ProjectedValue, ProjectionField, ProjectionId,
    TechniqueContributionDigest,
};
use starclock_combat::{
    AbilityId, AssemblyDigest, BattleSpec, BattleStateHash, CombatantSpecDigest, ConcedePolicy,
    EncounterId, EnemyDefinitionId, Energy, FormationIndex, Hp, LifeState, ParticipantSource,
    ParticipantSpec, PresenceState, ResolvedCombatantSpec, ResolvedDefinitionBindings, Speed,
    TeamResourceSpec, TeamSide, UnitDefinitionId, UnitLevel,
};
use starclock_mode_universe::{
    ability_runtime::{AbilityBoundary, AbilityExecutionContext, AbilityProjectionScope},
    baseline_runner::{
        StandardUniverseBaselinePolicy, StandardUniverseBaselineRunner,
        StandardUniverseBaselineStep,
    },
    battle_overlay::{UniverseEncounterBattleBinding, UniverseEncounterOverlay},
    catalog::UniverseCatalog,
    encounter_content_runtime::EncounterContentRuntimeCatalog,
    entry::{StandardUniverseEntry, StandardUniverseProfile},
    universe_replay::{
        StandardUniverseReplayError, encode_standard_universe_trace, record_baseline_run,
        replay_entry_for, verify_standard_universe_replay,
    },
};

const CORE_BUNDLE: &[u8] = include_bytes!("../../../config/generated/config.sora");
const UNIVERSE_BUNDLE: &[u8] = include_bytes!("../../../config/universe-generated/config.sora");

fn catalog() -> Arc<UniverseCatalog> {
    static CATALOG: OnceLock<Arc<UniverseCatalog>> = OnceLock::new();
    Arc::clone(CATALOG.get_or_init(|| {
        let core = starclock_data::catalog::load(CORE_BUNDLE).expect("core catalog");
        UniverseCatalog::load(UNIVERSE_BUNDLE, core).expect("Universe catalog")
    }))
}

fn participants() -> ParticipantLock {
    let policy = ParticipantPolicy::new(
        1,
        1,
        4,
        starclock_activity::ParticipantUniquenessScope::Activity,
        LoadoutLockScope::Activity,
    )
    .unwrap();
    let entries = (0_u8..4)
        .map(|index| {
            let byte = index + 1;
            ParticipantLockEntry::new(
                ParticipantId::new(u32::from(byte)).unwrap(),
                0,
                index,
                UnitDefinitionId::new(20_001 + u32::from(index)).unwrap(),
                OpaqueParticipantBuild::new(
                    CombatantSpecDigest::new([byte; 32]).unwrap(),
                    BuildDigest::new([byte + 32; 32]).unwrap(),
                    "universe-test-build-v1",
                    ParticipantSourceKind::CompiledBuild,
                )
                .unwrap(),
            )
            .unwrap()
        })
        .collect();
    ParticipantLock::seal(policy, entries).unwrap()
}

fn overlay(catalog: &UniverseCatalog, lock: &ParticipantLock) -> UniverseEncounterOverlay {
    let contract = Arc::new(
        ActivityBattleResultContract::new(
            Arc::new(
                starclock_activity::BattleResultProjection::new(
                    ProjectionId::new(1).unwrap(),
                    vec![
                        ProjectionField::Outcome,
                        ProjectionField::FinalStateHash,
                        ProjectionField::EventDigest,
                        ProjectionField::TerminalFault,
                        ProjectionField::ParticipantState(ParticipantId::new(1).unwrap()),
                        ProjectionField::ParticipantState(ParticipantId::new(2).unwrap()),
                        ProjectionField::ParticipantState(ParticipantId::new(3).unwrap()),
                        ProjectionField::ParticipantState(ParticipantId::new(4).unwrap()),
                    ],
                )
                .unwrap(),
            ),
            (1..=4)
                .map(|raw| {
                    ActivityParticipantCarryDefinition::new(
                        ParticipantId::new(raw).unwrap(),
                        HpCarryPolicy::CarryExact,
                        EnergyCarryPolicy::CarryExact,
                        LifeCarryPolicy::CarryExact,
                        PresenceCarryPolicy::CarryExact,
                    )
                })
                .collect(),
            vec![],
        )
        .unwrap(),
    );
    let bindings = catalog
        .encounter_groups()
        .iter()
        .flat_map(|group| group.members())
        .map(|member| {
            let preparation = Arc::new(
                starclock_activity::EncounterPreparationDefinition::new(
                    starclock_activity::ActivityOptionId::new(10).unwrap(),
                    starclock_activity::EncounterInitiativePolicy::PlayerControlled,
                    lock.digest(),
                    0,
                    vec![],
                    vec![starclock_activity::PreparedBattleVariant::new(
                        vec![],
                        TechniqueContributionDigest::new([0x44; 32]).unwrap(),
                        BattleBinding::new(
                            battle_spec(member.id().get()),
                            "universe-encounter",
                            "universe-battle-spec-v1",
                            lock.digest(),
                        )
                        .unwrap(),
                    )],
                )
                .unwrap(),
            );
            UniverseEncounterBattleBinding::new(member.id(), preparation, Arc::clone(&contract))
        })
        .collect();
    UniverseEncounterOverlay::new(bindings).unwrap()
}

fn battle_spec(member: u32) -> BattleSpec {
    let mut participants = (0_u8..4)
        .map(|index| {
            ParticipantSpec::new(
                TeamSide::Player,
                FormationIndex::new(index).unwrap(),
                ParticipantSource::Player,
                combatant(20_001 + u32::from(index), index + 1),
            )
        })
        .collect::<Vec<_>>();
    let enemy = 30_000 + member;
    participants.push(ParticipantSpec::new(
        TeamSide::Enemy,
        FormationIndex::new(0).unwrap(),
        ParticipantSource::EncounterEnemy(EnemyDefinitionId::new(enemy).unwrap()),
        combatant(enemy, u8::try_from(member).unwrap()),
    ));
    BattleSpec::new(
        "universe-test-rules-v1",
        AssemblyDigest::new([u8::try_from(member).unwrap(); 32]).unwrap(),
        EncounterId::new(member).unwrap(),
        participants,
        TeamResourceSpec::new(3, 5).unwrap(),
        TeamResourceSpec::new(0, 0).unwrap(),
        ConcedePolicy::Allowed,
    )
    .unwrap()
}

fn combatant(form: u32, digest: u8) -> ResolvedCombatantSpec {
    ResolvedCombatantSpec::new(
        UnitDefinitionId::new(form).unwrap(),
        UnitLevel::new(80).unwrap(),
        Hp::new(1_000).unwrap(),
        Speed::from_scaled(100_000_000).unwrap(),
        ResolvedDefinitionBindings::new(vec![AbilityId::new(form).unwrap()], vec![], vec![])
            .unwrap(),
        CombatantSpecDigest::new([digest; 32]).unwrap(),
    )
    .unwrap()
    .with_energy(Energy::ZERO, Energy::from_scaled(100_000_000).unwrap())
    .unwrap()
}

#[test]
fn encounter_resolution_preparation_handoff_and_reward_return_are_one_deterministic_chain() {
    let catalog = catalog();
    let lock = participants();
    let lock_digest = lock.digest();
    let overlay = overlay(&catalog, &lock);
    EncounterContentRuntimeCatalog::compile(&catalog)
        .unwrap()
        .validate_overlay(&overlay)
        .unwrap();
    assert_eq!(overlay.bindings().len(), 173);
    assert_eq!(
        overlay.digest().bytes(),
        [
            52, 64, 187, 44, 149, 171, 26, 107, 61, 140, 10, 78, 1, 209, 19, 97, 220, 59, 39, 231,
            108, 132, 212, 200, 93, 227, 74, 95, 217, 172, 145, 71,
        ]
    );
    let world = &catalog.worlds()[0];
    let compiled = StandardUniverseProfile::new(Arc::clone(&catalog))
        .compile(
            StandardUniverseEntry::new(world.id(), world.difficulties()[0], lock, vec![])
                .with_encounter_overlay(overlay),
        )
        .unwrap();
    let mut activity = compiled
        .start_standard(
            ActivityInstanceId::new(88).unwrap(),
            ActivityMasterSeed::from_u64(9),
        )
        .unwrap()
        .into_activity();
    assert!(
        activity
            .curio_contributions()
            .expect("empty initial Curio contributions")
            .entries()
            .is_empty()
    );
    choose_first(&mut activity);
    let initial_context = AbilityExecutionContext::new(
        AbilityProjectionScope::Battle,
        AbilityBoundary::BattleStart,
        0,
        false,
    );
    let snapshot = activity
        .battle_start_snapshot()
        .expect("current Activity projects one immutable battle snapshot");
    assert_eq!(snapshot.source_state_hash(), activity.view().state_hash());
    assert_eq!(snapshot.participant_lock(), lock_digest);
    assert_eq!(snapshot.path(), &activity.path_contributions().unwrap());
    assert_eq!(
        snapshot.blessings(),
        &activity.blessing_contributions().unwrap()
    );
    assert_eq!(snapshot.curios(), &activity.curio_contributions().unwrap());
    assert_eq!(
        snapshot.ability_tree(),
        &activity.ability_tree_contributions().unwrap()
    );
    assert!(snapshot.participant_carry().is_empty());
    assert!(snapshot.digest().iter().any(|byte| *byte != 0));
    let contributions = activity
        .battle_contributions(initial_context)
        .expect("compatibility contribution API delegates to the snapshot");
    assert_eq!(snapshot.contributions(), &contributions);
    assert_eq!(contributions.selected_path_blessings(), 0);
    assert!(contributions.rules().is_empty());
    assert!(contributions.modifiers().is_empty());
    assert!(matches!(
        activity.battle_contribution_snapshot(AbilityExecutionContext::new(
            AbilityProjectionScope::Battle,
            AbilityBoundary::BattleStart,
            1,
            false
        )),
        Err(
            starclock_mode_universe::runtime::StandardUniverseBattleContributionError::ContextMismatch
        )
    ));

    let encounter = loop {
        let view = activity.view();
        let decision = view.decision().expect("nonterminal domain decision");
        match decision.kind() {
            starclock_activity::ActivityDecisionKind::Encounter => {
                break (view.state_hash(), decision.id(), decision.options()[0].id());
            }
            starclock_activity::ActivityDecisionKind::ExternalOutcome => {
                activity
                    .submit_external_outcome(
                        view.state_hash(),
                        decision.id(),
                        starclock_activity::ActivityExternalOutcomeId::new(
                            decision.options()[0].id().get(),
                        )
                        .unwrap(),
                    )
                    .unwrap();
            }
            starclock_activity::ActivityDecisionKind::Choice
            | starclock_activity::ActivityDecisionKind::Reward
            | starclock_activity::ActivityDecisionKind::Route => {
                activity
                    .choose_option(view.state_hash(), decision.id(), decision.options()[0].id())
                    .unwrap();
            }
            other => panic!("unexpected domain decision: {other:?}"),
        }
    };
    let member = compiled
        .encounter_options()
        .iter()
        .find(|binding| binding.option() == encounter.2)
        .expect("offered encounter binding")
        .member();
    let authored = catalog
        .encounter_groups()
        .iter()
        .flat_map(|group| group.members())
        .find(|candidate| candidate.id() == member)
        .expect("authored encounter member");
    assert!(!authored.waves().is_empty());
    assert!(
        authored
            .waves()
            .iter()
            .all(|wave| !wave.enemies().is_empty())
    );
    let before = activity.graph().canonical_state_bytes();
    assert!(
        activity
            .engage_encounter(
                starclock_activity::ActivityStateHash::new([0; 32]).unwrap(),
                encounter.1,
                encounter.2,
                5,
            )
            .is_err()
    );
    assert_eq!(activity.graph().canonical_state_bytes(), before);
    let prepared = activity
        .engage_encounter(encounter.0, encounter.1, encounter.2, 5)
        .unwrap();
    assert_eq!(prepared.boundary(), ActivityPreparationBoundary::Decision);
    let preparation = activity.preparation_view().expect("preparation decision");
    assert_eq!(preparation.options().len(), 1);
    assert_eq!(
        activity
            .choose_preparation_option(activity.view().state_hash(), preparation.options()[0].id(),)
            .unwrap(),
        ActivityPreparationBoundary::BattleReady
    );
    let handoff = activity
        .start_pending_battle(activity.view().state_hash())
        .unwrap();
    assert_eq!(handoff.battle_spec().participants().len(), 5);
    let result = won_result(handoff.identity());
    let settled = activity
        .submit_pending_battle_result(activity.view().state_hash(), result)
        .unwrap();
    assert_eq!(settled.settlement().outcome(), BattleOutcome::Won);
    assert_eq!(
        settled.state_hash().bytes(),
        [
            63, 7, 2, 40, 69, 103, 194, 73, 131, 4, 60, 68, 255, 147, 73, 142, 120, 203, 42, 139,
            96, 176, 253, 211, 220, 67, 233, 87, 154, 28, 126, 212,
        ]
    );
    let reward = activity.view();
    let reward_decision = reward.decision().expect("post-battle reward");
    assert_eq!(
        reward_decision.kind(),
        starclock_activity::ActivityDecisionKind::Reward
    );
    assert_eq!(reward_decision.options().len(), 3);
    let before_reroll_snapshot = activity.battle_start_snapshot().unwrap();
    assert_eq!(before_reroll_snapshot.participant_carry().len(), 4);
    assert_eq!(
        before_reroll_snapshot.participant_carry(),
        activity.view().participant_carry()
    );
    let before_stale_reroll = activity.graph().canonical_state_bytes();
    assert!(
        activity
            .reroll_blessing_offer(starclock_activity::ActivityStateHash::new([0; 32]).unwrap())
            .is_err()
    );
    assert_eq!(
        activity.graph().canonical_state_bytes(),
        before_stale_reroll
    );
    assert_eq!(
        activity.reroll_blessing_offer(reward.state_hash()),
        Err(starclock_activity::GraphActivityRandomOfferError::RerollDisabled)
    );
    assert_eq!(
        before_reroll_snapshot.source_state_hash(),
        activity
            .battle_start_snapshot()
            .unwrap()
            .source_state_hash()
    );
    let reward = activity.view();
    let reward_decision = reward.decision().expect("reward");
    assert_eq!(reward_decision.options().len(), 3);
    activity
        .choose_option(
            reward.state_hash(),
            reward_decision.id(),
            reward_decision.options()[0].id(),
        )
        .unwrap();
    let contributions = activity
        .blessing_contributions()
        .expect("typed Blessing contribution set");
    assert!(!contributions.entries().is_empty());
    assert!(contributions.entries().iter().all(|entry| {
        entry.level().level() == 1
            && !entry.level().rule_key().is_empty()
            && !entry.level().source_binding_key().is_empty()
    }));
    let path_contributions = activity
        .path_contributions()
        .expect("selected Path contribution set");
    assert_eq!(
        path_contributions.passive().path(),
        compiled.path_options()[0]
    );
    assert_eq!(
        path_contributions.selected_path_blessings(),
        u8::from(contributions.entries()[0].path() == compiled.path_options()[0])
    );
    assert_eq!(
        contributions.digest(),
        [
            186, 84, 149, 116, 186, 156, 240, 118, 241, 204, 215, 125, 113, 92, 226, 90, 196, 168,
            41, 52, 68, 132, 131, 124, 27, 60, 253, 78, 189, 125, 19, 156,
        ]
    );
    let formation = activity.view();
    assert_eq!(
        formation.decision().expect("Formation gate").kind(),
        starclock_activity::ActivityDecisionKind::Choice
    );
    assert_eq!(formation.decision().unwrap().options().len(), 1);
    activity
        .choose_option(
            formation.state_hash(),
            formation.decision().unwrap().id(),
            formation.decision().unwrap().options()[0].id(),
        )
        .unwrap();
    assert_eq!(
        activity
            .view()
            .decision()
            .expect("routes after reward")
            .kind(),
        starclock_activity::ActivityDecisionKind::Route
    );
}

#[test]
fn goal07_ability_tree_unlocks_reroll_and_consumes_one_first_battle_bonus_choice() {
    let catalog = catalog();
    let lock = participants();
    let overlay = overlay(&catalog, &lock);
    let ability_tree = catalog
        .ability_tree_nodes()
        .iter()
        .map(|node| node.id())
        .collect::<Vec<_>>();
    let world = &catalog.worlds()[0];
    let compiled = StandardUniverseProfile::new(Arc::clone(&catalog))
        .compile(
            StandardUniverseEntry::new(world.id(), world.difficulties()[0], lock, ability_tree)
                .with_encounter_overlay(overlay),
        )
        .unwrap();
    let mut activity = compiled
        .start_standard(
            ActivityInstanceId::new(78).unwrap(),
            ActivityMasterSeed::from_u64(8),
        )
        .unwrap()
        .into_activity();
    choose_first(&mut activity);

    let encounter = loop {
        let view = activity.view();
        let decision = view.decision().expect("nonterminal domain decision");
        match decision.kind() {
            starclock_activity::ActivityDecisionKind::Encounter => {
                break (view.state_hash(), decision.id(), decision.options()[0].id());
            }
            starclock_activity::ActivityDecisionKind::ExternalOutcome => {
                activity
                    .submit_external_outcome(
                        view.state_hash(),
                        decision.id(),
                        starclock_activity::ActivityExternalOutcomeId::new(
                            decision.options()[0].id().get(),
                        )
                        .unwrap(),
                    )
                    .unwrap();
            }
            starclock_activity::ActivityDecisionKind::Choice
            | starclock_activity::ActivityDecisionKind::Reward
            | starclock_activity::ActivityDecisionKind::Route => {
                activity
                    .choose_option(view.state_hash(), decision.id(), decision.options()[0].id())
                    .unwrap();
            }
            other => panic!("unexpected domain decision: {other:?}"),
        }
    };
    activity
        .engage_encounter(encounter.0, encounter.1, encounter.2, 5)
        .unwrap();
    let preparation = activity.preparation_view().expect("preparation decision");
    activity
        .choose_preparation_option(activity.view().state_hash(), preparation.options()[0].id())
        .unwrap();
    let handoff = activity
        .start_pending_battle(activity.view().state_hash())
        .unwrap();
    let valid_result = won_result(handoff.identity());
    let before_forged = activity.graph().canonical_state_bytes();
    let forged = BattleResult::new(
        handoff.identity(),
        valid_result.values().to_vec(),
        starclock_activity::BattleResultDigest::new([0xee; 32]).unwrap(),
    );
    assert!(
        activity
            .submit_pending_battle_result(activity.view().state_hash(), forged)
            .is_err()
    );
    assert_eq!(activity.graph().canonical_state_bytes(), before_forged);
    activity
        .submit_pending_battle_result(activity.view().state_hash(), valid_result)
        .unwrap();

    let reward = activity.view();
    assert_eq!(
        reward.decision().expect("first bonus reward").kind(),
        starclock_activity::ActivityDecisionKind::Reward
    );
    activity
        .reroll_blessing_offer(reward.state_hash())
        .expect("Ability Tree node 11 unlocks one reroll");
    assert_eq!(
        activity.reroll_blessing_offer(activity.view().state_hash()),
        Err(starclock_activity::GraphActivityRandomOfferError::RerollLimitReached)
    );

    let bonus = activity.view();
    activity
        .choose_option(
            bonus.state_hash(),
            bonus.decision().unwrap().id(),
            bonus.decision().unwrap().options()[0].id(),
        )
        .unwrap();
    assert_eq!(
        activity.blessing_contributions().unwrap().entries().len(),
        1
    );
    let ordinary = activity.view();
    assert_eq!(
        ordinary
            .decision()
            .expect("ordinary reward follows bonus")
            .kind(),
        starclock_activity::ActivityDecisionKind::Reward
    );
    activity
        .choose_option(
            ordinary.state_hash(),
            ordinary.decision().unwrap().id(),
            ordinary.decision().unwrap().options()[0].id(),
        )
        .unwrap();
    assert_eq!(
        activity.blessing_contributions().unwrap().entries().len(),
        2
    );
    assert_eq!(
        activity.view().decision().expect("formation gate").kind(),
        starclock_activity::ActivityDecisionKind::Choice
    );
}

#[test]
fn baseline_runner_uses_offered_options_and_executes_nested_battles_to_terminal() {
    let catalog = catalog();
    let lock = participants();
    let overlay = overlay(&catalog, &lock);
    let world = &catalog.worlds()[0];
    let compiled = StandardUniverseProfile::new(Arc::clone(&catalog))
        .compile(
            StandardUniverseEntry::new(world.id(), world.difficulties()[0], lock, vec![])
                .with_encounter_overlay(overlay),
        )
        .unwrap();
    let mut activity = compiled
        .start_standard(
            ActivityInstanceId::new(88).unwrap(),
            ActivityMasterSeed::from_u64(9),
        )
        .unwrap()
        .into_activity();
    let runner = StandardUniverseBaselineRunner::default();
    let mut executor =
        |handoff: &starclock_activity::ActivityBattleHandoff| Ok(won_result(handoff.identity()));
    let report = runner
        .run_to_terminal(
            &mut activity,
            &StandardUniverseBaselinePolicy::default(),
            &mut executor,
        )
        .unwrap();
    assert_eq!(
        report.terminal(),
        starclock_activity::ActivityTerminalOutcome::Completed
    );
    assert_eq!(report.steps().len(), 69);
    assert_eq!(
        report.final_state_hash().bytes(),
        [
            5, 135, 45, 65, 196, 202, 178, 116, 209, 189, 51, 79, 124, 26, 239, 155, 98, 98, 153,
            124, 20, 28, 215, 155, 250, 210, 229, 56, 193, 68, 132, 158,
        ]
    );
    assert_eq!(report.final_state_hash(), activity.view().state_hash());
    assert!(report.steps().iter().any(|step| matches!(
        step,
        StandardUniverseBaselineStep::Decision { decision, .. }
            if decision.kind() == starclock_activity::ActivityDecisionKind::Encounter
    )));
    assert!(report.steps().iter().any(|step| matches!(
        step,
        StandardUniverseBaselineStep::Battle {
            outcome: BattleOutcome::Won,
            ..
        }
    )));
    assert_eq!(
        report
            .steps()
            .iter()
            .filter(|step| matches!(step, StandardUniverseBaselineStep::Battle { .. }))
            .count(),
        7
    );
}

#[test]
fn complete_run_replay_verifies_and_reports_the_first_divergence() {
    let catalog = catalog();
    let lock = participants();
    let overlay = overlay(&catalog, &lock);
    let world = &catalog.worlds()[0];
    let compiled = StandardUniverseProfile::new(Arc::clone(&catalog))
        .compile(
            StandardUniverseEntry::new(world.id(), world.difficulties()[0], lock, vec![])
                .with_encounter_overlay(overlay),
        )
        .unwrap();
    let instance = ActivityInstanceId::new(89).unwrap();
    let seed = ActivityMasterSeed::from_u64(1);
    let mut activity = compiled
        .start_standard(instance, seed)
        .unwrap()
        .into_activity();
    let header = starclock_replay::format::ReplayHeader::new(
        starclock_replay::format::ReplayIdentity::new(
            "4.4",
            "standard-universe-rules-v1",
            "standard-universe-data-v4.4",
            starclock_replay::digest::ConfigBundleDigest::new([0x41; 32]),
            "fixed-i64-6dp-v1",
            "chacha8-rand-0.10.2-intmap-v1",
            starclock_activity::ACTIVITY_STATE_HASH_REVISION,
        )
        .unwrap(),
        starclock_replay::format::ControllerIdentity::new(
            StandardUniverseBaselineRunner::REVISION,
            starclock_replay::digest::ControllerDigest::new([0x42; 32]),
        )
        .unwrap(),
        0,
        replay_entry_for(&activity, "standard-universe-v1"),
        0,
    )
    .unwrap();
    let mut executor =
        |handoff: &starclock_activity::ActivityBattleHandoff| Ok(won_result(handoff.identity()));
    let recorded = record_baseline_run(
        &mut activity,
        &StandardUniverseBaselinePolicy::default(),
        &mut executor,
    )
    .unwrap();
    let bytes = encode_standard_universe_trace(&header, recorded.trace()).unwrap();
    assert_eq!(bytes.len(), 7_982);
    assert_eq!(
        sha2::Sha256::digest(&bytes).as_slice(),
        [
            81, 77, 213, 207, 78, 19, 171, 121, 127, 212, 90, 57, 44, 211, 208, 184, 20, 237, 119,
            255, 96, 72, 153, 76, 20, 171, 19, 80, 152, 170, 60, 152,
        ]
    );
    let fresh = compiled
        .start_standard(instance, seed)
        .unwrap()
        .into_activity();
    let verified = verify_standard_universe_replay(&bytes, fresh, "standard-universe-v1").unwrap();
    assert_eq!(verified.action_count(), 49);
    assert_eq!(verified.nested_battle_count(), 2);
    assert_eq!(verified.diagnostic_count(), 45);
    assert_eq!(verified.terminal(), recorded.report().terminal());
    assert_eq!(
        verified.final_state_hash().bytes(),
        recorded.report().final_state_hash().bytes()
    );

    let mut state_corrupt = bytes.clone();
    let state_offset = replay_payload_offset(
        &state_corrupt,
        starclock_replay::record::RecordKind::ExpectedActivityState,
        0,
    );
    state_corrupt[state_offset] ^= 0x80;
    let fresh = compiled
        .start_standard(instance, seed)
        .unwrap()
        .into_activity();
    assert!(matches!(
        verify_standard_universe_replay(&state_corrupt, fresh, "standard-universe-v1"),
        Err(StandardUniverseReplayError::StateDivergence {
            action_index: 0,
            ..
        })
    ));

    let mut nested_corrupt = bytes.clone();
    let nested_offset = replay_payload_offset(
        &nested_corrupt,
        starclock_replay::record::RecordKind::NestedBattleStart,
        0,
    );
    nested_corrupt[nested_offset + 2] ^= 1;
    let fresh = compiled
        .start_standard(instance, seed)
        .unwrap()
        .into_activity();
    assert!(matches!(
        verify_standard_universe_replay(&nested_corrupt, fresh, "standard-universe-v1"),
        Err(StandardUniverseReplayError::NestedStartDivergence { .. })
    ));

    let mut action_corrupt = bytes.clone();
    let action_offset = replay_payload_offset(
        &action_corrupt,
        starclock_replay::record::RecordKind::AcceptedActivityCommand,
        0,
    );
    action_corrupt[action_offset + 12] ^= 0x40;
    let fresh = compiled
        .start_standard(instance, seed)
        .unwrap()
        .into_activity();
    assert!(matches!(
        verify_standard_universe_replay(&action_corrupt, fresh, "standard-universe-v1"),
        Err(StandardUniverseReplayError::DecisionDivergence { action_index: 0 })
    ));
}

fn replay_payload_offset(
    bytes: &[u8],
    kind: starclock_replay::record::RecordKind,
    ordinal: usize,
) -> usize {
    let decoded = starclock_replay::format::decode_replay(bytes).unwrap();
    let payload = decoded
        .records()
        .iter()
        .filter(|record| record.kind() == kind)
        .nth(ordinal)
        .unwrap()
        .payload();
    payload.as_ptr() as usize - bytes.as_ptr() as usize
}

fn choose_first(activity: &mut starclock_mode_universe::runtime::StandardUniverseActivity) {
    let view = activity.view();
    let decision = view.decision().unwrap();
    activity
        .choose_option(view.state_hash(), decision.id(), decision.options()[0].id())
        .unwrap();
}

fn won_result(identity: starclock_activity::BattleResultIdentity) -> BattleResult {
    let mut values = vec![
        ProjectedValue::Outcome(BattleOutcome::Won),
        ProjectedValue::FinalStateHash(BattleStateHash::from_bytes([0x71; 32])),
        ProjectedValue::EventDigest(EventDigest::new([0x72; 32]).unwrap()),
        ProjectedValue::TerminalFault(None),
    ];
    values.extend((1_u32..=4).map(|raw| {
        ProjectedValue::ParticipantState(
            ParticipantBattleState::new(
                ParticipantId::new(raw).unwrap(),
                Hp::new(900).unwrap(),
                Hp::new(1_000).unwrap(),
                Energy::from_scaled(50_000_000).unwrap(),
                Energy::from_scaled(100_000_000).unwrap(),
                LifeState::Alive,
                PresenceState::Present,
            )
            .unwrap(),
        )
    }));
    BattleResult::seal(identity, values)
}
