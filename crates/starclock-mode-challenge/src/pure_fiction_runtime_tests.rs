#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use starclock_activity::{
        ActivityConfigDigest, ActivityDefinitionDigest, ActivityDefinitionId,
        ActivityDefinitionIdentity, ActivityInstanceId, ActivityMasterSeed,
        ActivityTerminalOutcome, BattleOutcome, BattleResult, BuildDigest, EventDigest,
        LoadoutLockScope, MetricValue, OpaqueParticipantBuild, ParticipantBattleState,
        ParticipantId, ParticipantLock, ParticipantLockEntry, ParticipantPolicy,
        ParticipantSourceKind, ParticipantUniquenessScope, ProjectedValue,
    };
    use starclock_combat::{
        AbilityId, ActionValue, AssemblyDigest, BattleClockExpiry, BattleStateHash,
        CombatantSpecDigest, ConcedePolicy, EncounterId, FormationIndex, Hp, LifeState,
        ParticipantSource, ParticipantSpec, PresenceState, ResolvedCombatantSpec,
        ResolvedDefinitionBindings, RuleBundleId, Speed, TeamResourceSpec, TeamSide,
        UnitDefinitionId, UnitLevel,
    };

    use crate::{
        ChallengeNodeId, ChallengeProfileId, ChallengeStageId, CycleClockRule, Objective,
        ObjectiveId, ObjectiveKind, PureFictionAttempt, PureFictionAttemptDefinition,
        PureFictionNode, PureFictionProfile, PureFictionStage,
    };

    #[test]
    fn independent_node_scores_aggregate_and_timeout_advances() {
        let definition = Arc::new(
            PureFictionAttemptDefinition::new(
                identity(),
                Arc::new(profile()),
                0,
                participants(),
                vec![battle(10, 101, 201, 0x11), battle(20, 102, 202, 0x12)],
                vec![cacophony(), cacophony()],
            )
            .unwrap(),
        );
        let mut attempt = PureFictionAttempt::start(
            definition,
            ActivityInstanceId::new(7).unwrap(),
            ActivityMasterSeed::from_u64(9),
        )
        .unwrap();
        let first = start_node(&mut attempt, 1);
        assert_eq!(
            first.battle_spec().clock().and_then(|clock| match clock {
                starclock_combat::BattleClockSpec::Cycles(clock) => {
                    Some(clock.remaining_cycles())
                }
                starclock_combat::BattleClockSpec::ActionValue(_) => None,
            }),
            Some(5)
        );
        attempt
            .submit_battle_result(result(
                &first,
                BattleOutcome::Finalized,
                25_000,
                [8_000, 12_000, 5_000],
            ))
            .unwrap();
        let second = start_node(&mut attempt, 2);
        attempt
            .submit_battle_result(result(
                &second,
                BattleOutcome::Won,
                35_000,
                [8_000, 16_000, 11_000],
            ))
            .unwrap();
        assert_eq!(attempt.total_score(), 60_000);
        assert_eq!(attempt.node_score(0), 25_000);
        assert_eq!(attempt.wave_three_score(1), 11_000);
        assert!(attempt.cleared());
        assert_eq!(attempt.objectives().stars(), 1);
        assert_eq!(
            attempt.debug_view().player().state_hash(),
            attempt.state_hash()
        );
        assert_eq!(
            attempt.player_view().terminal(),
            Some(ActivityTerminalOutcome::Completed)
        );
    }

    #[test]
    fn starward_runs_three_independent_nodes() {
        let definition = Arc::new(
            PureFictionAttemptDefinition::new(
                identity(),
                Arc::new(starward_profile()),
                0,
                starward_participants(),
                vec![
                    battle(10, 101, 201, 0x11),
                    battle(20, 102, 202, 0x12),
                    battle(30, 103, 203, 0x13),
                ],
                vec![cacophony(), cacophony(), cacophony()],
            )
            .unwrap(),
        );
        let mut attempt = PureFictionAttempt::start(
            definition,
            ActivityInstanceId::new(8).unwrap(),
            ActivityMasterSeed::from_u64(10),
        )
        .unwrap();
        for node_index in 0..3 {
            let handoff = start_node(&mut attempt, u32::try_from(node_index + 1).unwrap());
            attempt
                .submit_battle_result(result(
                    &handoff,
                    BattleOutcome::Won,
                    30_000,
                    [8_000, 11_000, 11_000],
                ))
                .unwrap();
        }
        assert_eq!(attempt.total_score(), 90_000);
        assert_eq!(attempt.node_score(2), 30_000);
        assert!(attempt.cleared());
        assert_eq!(attempt.objectives().stars(), 1);
        assert_eq!(
            attempt.player_view().terminal(),
            Some(ActivityTerminalOutcome::Completed)
        );
    }

    fn start_node(
        attempt: &mut PureFictionAttempt,
        raw_attempt: u32,
    ) -> starclock_activity::ActivityBattleHandoff {
        attempt
            .engage_current_node(starclock_activity::AttemptId::new(raw_attempt).unwrap())
            .unwrap();
        attempt.choose_normal_engagement().unwrap();
        attempt.start_pending_battle().unwrap()
    }

    fn profile() -> PureFictionProfile {
        PureFictionProfile {
            id: ChallengeProfileId::new(1).unwrap(),
            stages: vec![PureFictionStage {
                id: ChallengeStageId::new(30_191).unwrap(),
                clock: CycleClockRule::new(
                    5,
                    ActionValue::from_scaled(150_000_000).unwrap(),
                    ActionValue::from_scaled(100_000_000).unwrap(),
                    false,
                    BattleClockExpiry::Finalize,
                )
                .unwrap(),
                clear_score: 30_000,
                nodes: vec![node(1, 10, 0), node(2, 20, 1)].into_boxed_slice(),
                objectives: vec![Objective::new(
                    ObjectiveId::new(3_001).unwrap(),
                    ObjectiveKind::ScoreAtLeast(60_000),
                )]
                .into_boxed_slice(),
            }]
            .into_boxed_slice(),
            policies: Vec::new().into_boxed_slice(),
        }
    }

    fn starward_profile() -> PureFictionProfile {
        PureFictionProfile {
            id: ChallengeProfileId::new(2).unwrap(),
            stages: vec![PureFictionStage {
                id: ChallengeStageId::new(30_322_043).unwrap(),
                clock: CycleClockRule::new(
                    4,
                    ActionValue::from_scaled(150_000_000).unwrap(),
                    ActionValue::from_scaled(100_000_000).unwrap(),
                    false,
                    BattleClockExpiry::Finalize,
                )
                .unwrap(),
                clear_score: 45_000,
                nodes: vec![node(1, 10, 0), node(2, 20, 1), node(3, 30, 2)].into_boxed_slice(),
                objectives: vec![Objective::new(
                    ObjectiveId::new(4_003).unwrap(),
                    ObjectiveKind::ScoreAtLeast(90_000),
                )]
                .into_boxed_slice(),
            }]
            .into_boxed_slice(),
            policies: Vec::new().into_boxed_slice(),
        }
    }

    fn node(id: u32, encounter: u32, team: u8) -> PureFictionNode {
        PureFictionNode {
            id: ChallengeNodeId::new(id).unwrap(),
            encounter: EncounterId::new(encounter).unwrap(),
            team_index: team,
            score_cap: 40_000,
            cacophony_bundles: vec![cacophony()].into_boxed_slice(),
        }
    }

    fn identity() -> ActivityDefinitionIdentity {
        ActivityDefinitionIdentity::new(
            ActivityDefinitionId::new(60).unwrap(),
            ActivityDefinitionDigest::new([0x51; 32]).unwrap(),
            ActivityConfigDigest::new([0x52; 32]).unwrap(),
        )
    }

    fn participants() -> ParticipantLock {
        ParticipantLock::seal(
            ParticipantPolicy::new(
                2,
                1,
                4,
                ParticipantUniquenessScope::Section,
                LoadoutLockScope::Section,
            )
            .unwrap(),
            vec![participant(1, 0, 101, 0x11), participant(2, 1, 102, 0x12)],
        )
        .unwrap()
    }

    fn starward_participants() -> ParticipantLock {
        ParticipantLock::seal(
            ParticipantPolicy::new(
                3,
                1,
                4,
                ParticipantUniquenessScope::Section,
                LoadoutLockScope::Section,
            )
            .unwrap(),
            vec![
                participant(1, 0, 101, 0x11),
                participant(2, 1, 102, 0x12),
                participant(3, 2, 103, 0x13),
            ],
        )
        .unwrap()
    }

    fn participant(id: u32, team: u8, character: u32, digest: u8) -> ParticipantLockEntry {
        ParticipantLockEntry::new(
            ParticipantId::new(id).unwrap(),
            team,
            0,
            UnitDefinitionId::new(character).unwrap(),
            OpaqueParticipantBuild::new(
                CombatantSpecDigest::new([digest; 32]).unwrap(),
                BuildDigest::new([digest.wrapping_add(1); 32]).unwrap(),
                ParticipantSourceKind::CompiledBuild,
            )
            .unwrap(),
        )
        .unwrap()
    }

    fn battle(encounter: u32, player: u32, enemy: u32, digest: u8) -> starclock_combat::BattleSpec {
        starclock_combat::BattleSpec::new(
            AssemblyDigest::new([u8::try_from(encounter).unwrap(); 32]).unwrap(),
            EncounterId::new(encounter).unwrap(),
            vec![
                ParticipantSpec::new(
                    TeamSide::Player,
                    FormationIndex::new(0).unwrap(),
                    ParticipantSource::Player,
                    combatant(player, digest, vec![cacophony()]),
                ),
                ParticipantSpec::new(
                    TeamSide::Enemy,
                    FormationIndex::new(0).unwrap(),
                    ParticipantSource::Player,
                    combatant(enemy, digest.wrapping_add(0x20), Vec::new()),
                ),
            ],
            TeamResourceSpec::new(3, 5).unwrap(),
            TeamResourceSpec::new(0, 0).unwrap(),
            ConcedePolicy::Allowed,
        )
        .unwrap()
    }

    fn combatant(form: u32, digest: u8, bundles: Vec<RuleBundleId>) -> ResolvedCombatantSpec {
        ResolvedCombatantSpec::new(
            UnitDefinitionId::new(form).unwrap(),
            UnitLevel::new(80).unwrap(),
            Hp::new(1_000).unwrap(),
            Speed::from_scaled(100_000_000).unwrap(),
            ResolvedDefinitionBindings::new(vec![AbilityId::new(form).unwrap()], bundles, vec![])
                .unwrap(),
            CombatantSpecDigest::new([digest; 32]).unwrap(),
        )
        .unwrap()
    }

    fn cacophony() -> RuleBundleId {
        RuleBundleId::new(3_031_359).unwrap()
    }

    fn result(
        handoff: &starclock_activity::ActivityBattleHandoff,
        outcome: BattleOutcome,
        total: i64,
        waves: [i64; 3],
    ) -> BattleResult {
        let mut values = vec![
            ProjectedValue::Outcome(outcome),
            ProjectedValue::FinalStateHash(BattleStateHash::from_bytes([0x61; 32])),
            ProjectedValue::EventDigest(EventDigest::new([0x62; 32]).unwrap()),
            ProjectedValue::TerminalFault(None),
        ];
        values.extend(handoff.participant_carry().iter().map(|carry| {
            ProjectedValue::ParticipantState(
                ParticipantBattleState::new(
                    carry.participant(),
                    carry.current_hp(),
                    carry.maximum_hp(),
                    carry.current_energy(),
                    carry.maximum_energy(),
                    LifeState::Alive,
                    PresenceState::Present,
                )
                .unwrap(),
            )
        }));
        for (key, value) in [
            ("node_score", total),
            ("wave_one_score", waves[0]),
            ("wave_two_score", waves[1]),
            ("wave_three_score", waves[2]),
        ] {
            values.push(ProjectedValue::Metric {
                key: key.into(),
                value: MetricValue::BoundedInteger(value),
            });
        }
        BattleResult::seal(handoff.identity(), values)
    }
}
