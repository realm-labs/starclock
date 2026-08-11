use starclock_combat::{BattleClockExpiry, BattleSpec, EncounterId, RuleBundleId};

use crate::{
    AnomalyQuadrantId, ChallengeProfileId, ChallengeStageId, CycleClockRule, ObjectiveEvaluation,
    ObjectiveId, ProjectPolicy,
};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum AnomalyStageKind {
    Knight { slot: u8 },
    KingNormal,
    KingPlight,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum AnomalyTargetKind {
    ConsumedCyclesAtMost(u16),
    NoDefeatedParticipants,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AnomalyTarget {
    id: ObjectiveId,
    kind: AnomalyTargetKind,
}

impl AnomalyTarget {
    #[must_use]
    pub const fn new(id: ObjectiveId, kind: AnomalyTargetKind) -> Self {
        Self { id, kind }
    }

    #[must_use]
    pub const fn id(self) -> ObjectiveId {
        self.id
    }

    #[must_use]
    pub const fn kind(self) -> AnomalyTargetKind {
        self.kind
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AnomalyStage {
    pub id: ChallengeStageId,
    pub kind: AnomalyStageKind,
    pub encounter: EncounterId,
    pub team_index: u8,
    pub clock: CycleClockRule,
    pub targets: Box<[AnomalyTarget]>,
}

impl AnomalyStage {
    #[must_use]
    pub fn compile_battle(&self, base: BattleSpec) -> Option<BattleSpec> {
        if base.encounter() != self.encounter {
            return None;
        }
        Some(base.with_clock(self.clock.compile(self.clock.initial_cycles())?))
    }

    #[must_use]
    pub fn evaluate(
        &self,
        completed: bool,
        consumed_cycles: u16,
        defeated_participants: u8,
    ) -> ObjectiveEvaluation {
        let awarded = self
            .targets
            .iter()
            .filter(|target| {
                completed
                    && match target.kind() {
                        AnomalyTargetKind::ConsumedCyclesAtMost(limit) => consumed_cycles <= limit,
                        AnomalyTargetKind::NoDefeatedParticipants => defeated_participants == 0,
                    }
            })
            .map(|target| target.id())
            .collect();
        ObjectiveEvaluation::from_awarded(awarded)
    }

    #[must_use]
    pub const fn protection_contributions(self_kind: AnomalyStageKind) -> u8 {
        match self_kind {
            AnomalyStageKind::KingPlight => 3,
            AnomalyStageKind::KingNormal | AnomalyStageKind::Knight { .. } => 0,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AnomalyQuadrant {
    pub id: AnomalyQuadrantId,
    pub upstream_buff_id: u32,
    pub rule_bundle: RuleBundleId,
    pub behavior_exact: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AnomalyProfile {
    pub id: ChallengeProfileId,
    pub stages: Box<[AnomalyStage]>,
    pub quadrants: Box<[AnomalyQuadrant]>,
    pub policies: Box<[ProjectPolicy]>,
}

impl AnomalyProfile {
    #[must_use]
    pub fn new(
        id: ChallengeProfileId,
        mut stages: Vec<AnomalyStage>,
        mut quadrants: Vec<AnomalyQuadrant>,
        policies: Vec<ProjectPolicy>,
    ) -> Option<Self> {
        stages.sort_by_key(|stage| stage.id);
        quadrants.sort_by_key(|quadrant| quadrant.id);
        let valid_stages = stages.len() == 5
            && stages.windows(2).all(|pair| pair[0].id != pair[1].id)
            && stages
                .iter()
                .filter(|stage| matches!(stage.kind, AnomalyStageKind::Knight { .. }))
                .count()
                == 3
            && stages
                .iter()
                .filter(|stage| matches!(stage.kind, AnomalyStageKind::KingNormal))
                .count()
                == 1
            && stages
                .iter()
                .filter(|stage| matches!(stage.kind, AnomalyStageKind::KingPlight))
                .count()
                == 1;
        let valid_quadrants = quadrants.len() == 3
            && quadrants.windows(2).all(|pair| pair[0].id != pair[1].id)
            && quadrants
                .iter()
                .map(|quadrant| quadrant.rule_bundle)
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                == quadrants.len();
        if !valid_stages || !valid_quadrants {
            return None;
        }
        Some(Self {
            id,
            stages: stages.into_boxed_slice(),
            quadrants: quadrants.into_boxed_slice(),
            policies: policies.into_boxed_slice(),
        })
    }

    #[must_use]
    pub fn stage(&self, id: ChallengeStageId) -> Option<&AnomalyStage> {
        self.stages
            .binary_search_by_key(&id, |stage| stage.id)
            .ok()
            .map(|index| &self.stages[index])
    }

    #[must_use]
    pub fn quadrant(&self, id: AnomalyQuadrantId) -> Option<&AnomalyQuadrant> {
        self.quadrants
            .binary_search_by_key(&id, |quadrant| quadrant.id)
            .ok()
            .map(|index| &self.quadrants[index])
    }
}

#[must_use]
pub fn anomaly_clock(
    cycles: u16,
    first_window: starclock_combat::ActionValue,
    later_window: starclock_combat::ActionValue,
) -> Option<CycleClockRule> {
    CycleClockRule::new(
        cycles,
        first_window,
        later_window,
        false,
        BattleClockExpiry::Lose,
    )
}
