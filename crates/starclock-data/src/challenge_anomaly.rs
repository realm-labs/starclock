use starclock_combat::{ActionValue, EncounterId, RuleBundleId};
use starclock_mode_challenge::{
    AnomalyProfile, AnomalyQuadrant, AnomalyQuadrantId, AnomalyStage, AnomalyStageKind,
    AnomalyTarget, AnomalyTargetKind, ChallengeProfileId, ChallengeStageId, ObjectiveId,
    PolicyConfidence, ProjectPolicy, anomaly_clock,
};

use crate::{
    challenge::{ChallengeDataError, PRODUCTION_BUNDLE, message},
    challenge_generated::{
        SoraConfig, anomaly_stage_kind::AnomalyStageKind as GeneratedStageKind,
        anomaly_target_kind::AnomalyTargetKind as GeneratedTargetKind,
        challenge_policy_confidence::ChallengePolicyConfidence, runtime::SoraBundle,
    },
};

pub fn anomaly_arbitration() -> Result<AnomalyProfile, ChallengeDataError> {
    load_anomaly_arbitration(PRODUCTION_BUNDLE)
}

pub fn load_anomaly_arbitration(bytes: &[u8]) -> Result<AnomalyProfile, ChallengeDataError> {
    let bundle = SoraBundle::parse(bytes).map_err(|error| message(&error.to_string()))?;
    let config = SoraConfig::from_source(&bundle).map_err(|error| message(&error.to_string()))?;
    let profile = config
        .arb_runtime_profiles()
        .ordered_rows()
        .next()
        .ok_or_else(|| message("Anomaly Arbitration runtime profile is missing"))?;
    if config.arb_runtime_profiles().len() != 1 {
        return Err(message("Anomaly Arbitration profile denominator drift"));
    }
    let first_window = ActionValue::from_scaled(profile.first_window_scaled)
        .map_err(|_| message("Anomaly first cycle Action Value is invalid"))?;
    let later_window = ActionValue::from_scaled(profile.later_window_scaled)
        .map_err(|_| message("Anomaly later cycle Action Value is invalid"))?;
    let mut stages = Vec::new();
    for row in config.arb_runtime_stages().ordered_rows() {
        let stage_id = stage_id(row.source_stage_id)?;
        let targets = config
            .arb_runtime_targets()
            .ordered_rows()
            .filter(|target| target.stage_id == row.id)
            .map(|target| {
                let id = ObjectiveId::new(unsigned(target.id, "Anomaly target id")?)
                    .ok_or_else(|| message("Anomaly target id must be non-zero"))?;
                let threshold = u16::try_from(target.threshold)
                    .map_err(|_| message("Anomaly target threshold exceeds u16"))?;
                let kind = match target.kind {
                    GeneratedTargetKind::ConsumedCyclesAtMost => {
                        AnomalyTargetKind::ConsumedCyclesAtMost(threshold)
                    }
                    GeneratedTargetKind::NoDefeatedParticipants => {
                        AnomalyTargetKind::NoDefeatedParticipants
                    }
                };
                Ok(AnomalyTarget::new(id, kind))
            })
            .collect::<Result<Vec<_>, ChallengeDataError>>()?;
        let team_index =
            u8::try_from(row.team_index).map_err(|_| message("Anomaly team index exceeds u8"))?;
        let kind = match row.kind {
            GeneratedStageKind::Knight => AnomalyStageKind::Knight { slot: team_index },
            GeneratedStageKind::KingNormal => AnomalyStageKind::KingNormal,
            GeneratedStageKind::KingPlight => AnomalyStageKind::KingPlight,
        };
        stages.push(AnomalyStage {
            id: stage_id,
            kind,
            encounter: EncounterId::new(unsigned(row.encounter_id, "Anomaly encounter id")?)
                .ok_or_else(|| message("Anomaly encounter id must be non-zero"))?,
            team_index,
            clock: anomaly_clock(
                u16::try_from(row.cycle_limit)
                    .map_err(|_| message("Anomaly cycle limit exceeds u16"))?,
                first_window,
                later_window,
            )
            .ok_or_else(|| message("Anomaly cycle clock is invalid"))?,
            targets: targets.into_boxed_slice(),
        });
    }
    let quadrants = config
        .arb_runtime_quadrants()
        .ordered_rows()
        .map(|row| {
            Ok(AnomalyQuadrant {
                id: AnomalyQuadrantId::new(unsigned(row.upstream_buff_id, "Quadrant id")?)
                    .ok_or_else(|| message("Quadrant id must be non-zero"))?,
                upstream_buff_id: unsigned(row.upstream_buff_id, "Quadrant buff id")?,
                rule_bundle: RuleBundleId::new(unsigned(row.rule_bundle_id, "Quadrant bundle")?)
                    .ok_or_else(|| message("Quadrant bundle id must be non-zero"))?,
                behavior_exact: row.behavior_exact,
            })
        })
        .collect::<Result<Vec<_>, ChallengeDataError>>()?;
    let policies = config
        .arb_runtime_policies()
        .ordered_rows()
        .map(|row| ProjectPolicy {
            id: row.stable_key.clone().into_boxed_str(),
            known_facts: row.known_facts.clone().into_boxed_str(),
            selected_behavior: row.selected_behavior.clone().into_boxed_str(),
            rejected_alternatives: row
                .rejected_alternatives
                .iter()
                .cloned()
                .map(String::into_boxed_str)
                .collect(),
            rationale: row.rationale.clone().into_boxed_str(),
            affected_tests: row
                .affected_tests
                .iter()
                .cloned()
                .map(String::into_boxed_str)
                .collect(),
            confidence: match row.confidence {
                ChallengePolicyConfidence::Low => PolicyConfidence::Low,
                ChallengePolicyConfidence::Medium => PolicyConfidence::Medium,
                ChallengePolicyConfidence::High => PolicyConfidence::High,
            },
            replacement_condition: row.replacement_condition.clone().into_boxed_str(),
        })
        .collect::<Vec<_>>();
    if stages.len() != 5 || quadrants.len() != 3 || policies.len() != 3 {
        return Err(message("Anomaly Arbitration runtime denominator drift"));
    }
    AnomalyProfile::new(
        ChallengeProfileId::new(unsigned(profile.id, "Anomaly profile id")?)
            .ok_or_else(|| message("Anomaly profile id must be non-zero"))?,
        stages,
        quadrants,
        policies,
    )
    .ok_or_else(|| message("Anomaly Arbitration profile invariants failed"))
}

fn stage_id(value: i32) -> Result<ChallengeStageId, ChallengeDataError> {
    ChallengeStageId::new(unsigned(value, "Anomaly stage id")?)
        .ok_or_else(|| message("Anomaly stage id must be non-zero"))
}

fn unsigned(value: i32, field: &str) -> Result<u32, ChallengeDataError> {
    u32::try_from(value).map_err(|_| message(&format!("{field} must be non-negative")))
}

#[cfg(test)]
mod tests {
    use starclock_mode_challenge::{AnomalyStageKind, AnomalyTargetKind};

    use super::anomaly_arbitration;

    #[test]
    fn production_profile_preserves_released_topology() {
        let profile = anomaly_arbitration().expect("Anomaly runtime profile lowers");
        assert_eq!(profile.stages.len(), 5);
        assert_eq!(profile.quadrants.len(), 3);
        assert_eq!(profile.policies.len(), 3);
        assert_eq!(profile.stages[0].id.get(), 30_508_011);
        assert_eq!(profile.stages[0].clock.initial_cycles(), 6);
        assert_eq!(profile.stages[4].clock.initial_cycles(), 2);
        assert!(matches!(
            profile.stages[0].kind,
            AnomalyStageKind::Knight { slot: 0 }
        ));
        assert!(matches!(
            profile.stages[3].kind,
            AnomalyStageKind::KingNormal
        ));
        assert!(matches!(
            profile.stages[4].kind,
            AnomalyStageKind::KingPlight
        ));
        assert!(
            profile.stages[0]
                .targets
                .iter()
                .any(|target| { target.kind() == AnomalyTargetKind::NoDefeatedParticipants })
        );
        assert_eq!(
            profile
                .quadrants
                .iter()
                .filter(|item| item.behavior_exact)
                .count(),
            1
        );
    }
}
