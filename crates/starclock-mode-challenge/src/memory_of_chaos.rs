use starclock_combat::{ActionValue, BattleClockExpiry, BattleSpec, EncounterId, RuleBundleId};

use crate::{
    ChallengeNodeId, ChallengeProfileId, ChallengeStageId, CycleClockRule, Objective,
    ObjectiveEvaluation, ObjectiveInput, ProjectPolicy,
};

/// One ordered node and its mode-owned rule contribution bindings.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemoryNode {
    id: ChallengeNodeId,
    encounter: EncounterId,
    team_index: u8,
    rule_bundles: Box<[RuleBundleId]>,
}

impl MemoryNode {
    pub fn new(
        id: ChallengeNodeId,
        encounter: EncounterId,
        team_index: u8,
        mut rule_bundles: Vec<RuleBundleId>,
    ) -> Option<Self> {
        if team_index > 2 {
            return None;
        }
        rule_bundles.sort_unstable();
        if rule_bundles.windows(2).any(|pair| pair[0] == pair[1]) {
            return None;
        }
        Some(Self {
            id,
            encounter,
            team_index,
            rule_bundles: rule_bundles.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn id(&self) -> ChallengeNodeId {
        self.id
    }
    #[must_use]
    pub const fn encounter(&self) -> EncounterId {
        self.encounter
    }
    #[must_use]
    pub const fn team_index(&self) -> u8 {
        self.team_index
    }
    #[must_use]
    pub fn rule_bundles(&self) -> &[RuleBundleId] {
        &self.rule_bundles
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemoryStage {
    id: ChallengeStageId,
    clock: CycleClockRule,
    nodes: Box<[MemoryNode]>,
    objectives: Box<[Objective]>,
}

impl MemoryStage {
    pub fn new(
        id: ChallengeStageId,
        clock: CycleClockRule,
        mut nodes: Vec<MemoryNode>,
        objectives: Vec<Objective>,
    ) -> Option<Self> {
        nodes.sort_by_key(MemoryNode::team_index);
        if !(2..=3).contains(&nodes.len())
            || nodes
                .iter()
                .enumerate()
                .any(|(index, node)| usize::from(node.team_index) != index)
            || nodes
                .windows(2)
                .any(|pair| pair[0].id == pair[1].id || pair[0].encounter == pair[1].encounter)
            || objectives.len() != 3
        {
            return None;
        }
        Some(Self {
            id,
            clock,
            nodes: nodes.into_boxed_slice(),
            objectives: objectives.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn id(&self) -> ChallengeStageId {
        self.id
    }
    #[must_use]
    pub fn nodes(&self) -> &[MemoryNode] {
        &self.nodes
    }
    #[must_use]
    pub const fn initial_cycles(&self) -> u16 {
        self.clock.initial_cycles()
    }
    #[must_use]
    pub fn evaluate(&self, input: ObjectiveInput) -> ObjectiveEvaluation {
        ObjectiveEvaluation::evaluate(&self.objectives, input)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemoryProfile {
    id: ChallengeProfileId,
    clock: CycleClockRule,
    stages: Box<[MemoryStage]>,
    policies: Box<[ProjectPolicy]>,
}

impl MemoryProfile {
    pub fn new(
        id: ChallengeProfileId,
        clock: CycleClockRule,
        mut stages: Vec<MemoryStage>,
        policies: Vec<ProjectPolicy>,
    ) -> Option<Self> {
        if stages.is_empty() {
            return None;
        }
        stages.sort_by_key(MemoryStage::id);
        if stages.windows(2).any(|pair| pair[0].id == pair[1].id) {
            return None;
        }
        Some(Self {
            id,
            clock,
            stages: stages.into_boxed_slice(),
            policies: policies.into_boxed_slice(),
        })
    }

    #[must_use]
    pub fn version_4_4_clock() -> CycleClockRule {
        CycleClockRule::new(
            30,
            ActionValue::from_scaled(150_000_000).expect("150 AV is non-negative"),
            ActionValue::from_scaled(100_000_000).expect("100 AV is non-negative"),
            true,
            BattleClockExpiry::Lose,
        )
        .expect("released Memory clock values are non-zero")
    }

    #[must_use]
    pub const fn id(&self) -> ChallengeProfileId {
        self.id
    }
    #[must_use]
    pub fn stages(&self) -> &[MemoryStage] {
        &self.stages
    }
    #[must_use]
    pub fn policies(&self) -> &[ProjectPolicy] {
        &self.policies
    }
    #[must_use]
    pub fn initial_cycles(&self, stage_index: usize) -> Option<u16> {
        self.stages
            .get(stage_index)
            .map(MemoryStage::initial_cycles)
    }

    /// Compiles the Activity-owned carried cycle value into a fresh battle.
    #[must_use]
    pub fn compile_battle(
        &self,
        stage_index: usize,
        base: BattleSpec,
        remaining_cycles: u16,
    ) -> Option<BattleSpec> {
        self.stages
            .get(stage_index)?
            .clock
            .compile(remaining_cycles)
            .map(|clock| base.with_clock(clock))
    }
}
