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
    AbilityId, BattleSpec, BattleSpecDigest, BattleStateHash, CombatantSpecDigest, ConcedePolicy,
    EncounterId, EnemyDefinitionId, Energy, FormationIndex, Hp, LifeState, ParticipantSource,
    ParticipantSpec, PresenceState, ResolvedCombatantSpec, ResolvedDefinitionBindings, Speed,
    TeamResourceSpec, TeamSide, UnitDefinitionId, UnitLevel,
};
use starclock_mode_universe::{
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
        BattleSpecDigest::new([u8::try_from(member).unwrap(); 32]).unwrap(),
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
    let overlay = overlay(&catalog, &lock);
    EncounterContentRuntimeCatalog::compile(&catalog)
        .unwrap()
        .validate_overlay(&overlay)
        .unwrap();
    assert_eq!(overlay.bindings().len(), 173);
    assert_eq!(
        overlay.digest().bytes(),
        [
            160, 152, 162, 41, 78, 135, 46, 152, 254, 121, 51, 229, 237, 80, 77, 170, 18, 136, 31,
            95, 12, 137, 4, 109, 97, 161, 155, 200, 110, 40, 143, 12,
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
            ActivityInstanceId::new(77).unwrap(),
            ActivityMasterSeed::from_u64(8),
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

    let encounter = loop {
        let view = activity.view();
        let decision = view.decision().expect("nonterminal domain decision");
        match decision.kind() {
            starclock_activity::ActivityDecisionKind::Encounter => {
                break (view.state_hash(), decision.id(), decision.options()[0].id());
            }
            starclock_activity::ActivityDecisionKind::Choice
            | starclock_activity::ActivityDecisionKind::ExternalOutcome
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
            124, 3, 202, 129, 216, 105, 174, 104, 113, 27, 25, 17, 23, 186, 57, 162, 152, 158, 209,
            204, 87, 90, 109, 174, 248, 161, 29, 121, 138, 245, 162, 77,
        ]
    );
    let reward = activity.view();
    let reward_decision = reward.decision().expect("post-battle reward");
    assert_eq!(
        reward_decision.kind(),
        starclock_activity::ActivityDecisionKind::Reward
    );
    assert_eq!(reward_decision.options().len(), 3);
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
    activity
        .reroll_blessing_offer(reward.state_hash())
        .expect("one deterministic Blessing reset");
    let before_exhausted_reroll = activity.graph().canonical_state_bytes();
    assert!(
        activity
            .reroll_blessing_offer(activity.view().state_hash())
            .is_err()
    );
    assert_eq!(
        activity.graph().canonical_state_bytes(),
        before_exhausted_reroll
    );
    let reward = activity.view();
    let reward_decision = reward.decision().expect("rerolled reward");
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
            36, 90, 72, 16, 218, 20, 219, 10, 135, 178, 227, 132, 177, 131, 41, 72, 207, 10, 162,
            27, 45, 139, 29, 112, 133, 32, 243, 154, 29, 106, 40, 207,
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
        |handoff: &starclock_activity::ActivityBattleHandoff| won_result(handoff.identity());
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
    assert_eq!(report.steps().len(), 30);
    assert_eq!(
        report.final_state_hash().bytes(),
        [
            224, 192, 76, 134, 150, 227, 159, 127, 173, 107, 8, 65, 204, 189, 237, 61, 108, 173,
            67, 2, 178, 49, 70, 29, 23, 9, 236, 74, 229, 98, 251, 215,
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
        2
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
    let seed = ActivityMasterSeed::from_u64(10);
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
        10,
        replay_entry_for(&activity, "standard-universe-v1"),
        0,
    )
    .unwrap();
    let mut executor =
        |handoff: &starclock_activity::ActivityBattleHandoff| won_result(handoff.identity());
    let recorded = record_baseline_run(
        &mut activity,
        &StandardUniverseBaselinePolicy::default(),
        &mut executor,
    )
    .unwrap();
    let bytes = encode_standard_universe_trace(&header, recorded.trace()).unwrap();
    assert_eq!(bytes.len(), 11_011);
    // SHA-256: f4ca5a3e84df8f123dc01b313cb0aeaa703370e07295fc4870cda28ac3da0b31
    assert_eq!(
        sha2::Sha256::digest(&bytes).as_slice(),
        [
            244, 202, 90, 62, 132, 223, 143, 18, 61, 192, 27, 49, 60, 176, 174, 170, 112, 51, 112,
            224, 114, 149, 252, 72, 112, 205, 162, 138, 195, 218, 11, 49,
        ]
    );
    let fresh = compiled
        .start_standard(instance, seed)
        .unwrap()
        .into_activity();
    let verified = verify_standard_universe_replay(&bytes, fresh, "standard-universe-v1").unwrap();
    assert_eq!(verified.action_count(), 60);
    assert_eq!(verified.nested_battle_count(), 5);
    assert_eq!(verified.diagnostic_count(), 50);
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
