//! Atomic Phase 3 movement, dice, map, and Communing orchestration.

use starclock_activity::{
    ActivityExpression, ActivityOperation, ActivityProgramDefinition, ActivityProgramId,
    ActivityRngStreams, ActivitySlotId, ActivityTransactionState, ActivityValue, NodeId,
};

use crate::error::{UniverseCatalogLoadError, UniverseCatalogLoadErrorKind};

use super::{SwarmDisasterRuntimeInstance, map_overlay, state::DEFERRED};

pub(super) const RESOLUTION_TIER_BASE: u64 = 0x0800_0000;
const RESOLUTION_PROGRAM_ID: u32 = 0x5380_0001;

#[cfg(test)]
pub(super) const PHASE3_FIXTURE_IDS: [&str; 4] = [
    "swarm-disaster.fixture.dice-roll-reroll-cheat",
    "swarm-disaster.fixture.dice-face-targeting",
    "swarm-disaster.fixture.communing-choice",
    "swarm-disaster.fixture.communing-dimension-points",
];

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(super) struct SwarmSimultaneousResolution {
    movement_target: Option<NodeId>,
    countdown_adjustments: Box<[(u32, i64)]>,
    explicit_face_target: Option<NodeId>,
    map_replacement: Option<(NodeId, Box<str>, Option<Box<str>>)>,
    communing_choice: Option<(u16, Box<str>)>,
    cabinet_completion: Option<(Box<str>, Box<str>)>,
}

impl SwarmSimultaneousResolution {
    pub(super) fn new() -> Self {
        Self::default()
    }

    pub(super) fn with_movement(mut self, target: NodeId, adjustments: &[(u32, i64)]) -> Self {
        self.movement_target = Some(target);
        self.countdown_adjustments = adjustments.into();
        self
    }

    pub(super) const fn with_face_activation(mut self, explicit_target: Option<NodeId>) -> Self {
        self.explicit_face_target = explicit_target;
        self
    }

    pub(super) fn with_map_replacement(
        mut self,
        target: NodeId,
        domain: impl Into<Box<str>>,
        beacon: Option<impl Into<Box<str>>>,
    ) -> Self {
        self.map_replacement = Some((target, domain.into(), beacon.map(Into::into)));
        self
    }

    pub(super) fn with_communing_choice(
        mut self,
        story_stage: u16,
        choice: impl Into<Box<str>>,
    ) -> Self {
        self.communing_choice = Some((story_stage, choice.into()));
        self
    }

    pub(super) fn with_cabinet_completion(
        mut self,
        cabinet: impl Into<Box<str>>,
        objective: impl Into<Box<str>>,
    ) -> Self {
        self.cabinet_completion = Some((cabinet.into(), objective.into()));
        self
    }
}

pub(super) fn compile(
    instance: &SwarmDisasterRuntimeInstance,
    state: &ActivityTransactionState,
    request: SwarmSimultaneousResolution,
    rng: &mut ActivityRngStreams,
) -> Result<ActivityProgramDefinition, UniverseCatalogLoadError> {
    rng.transact(|working| compile_inner(instance, state, &request, working))
}

fn compile_inner(
    instance: &SwarmDisasterRuntimeInstance,
    state: &ActivityTransactionState,
    request: &SwarmSimultaneousResolution,
    rng: &mut ActivityRngStreams,
) -> Result<ActivityProgramDefinition, UniverseCatalogLoadError> {
    let mut tiers: [Vec<ActivityOperation>; 5] = core::array::from_fn(|_| Vec::new());

    if let Some(target) = request.movement_target {
        let edge = instance
            .graph_definition()
            .outgoing(state.current_node())
            .find(|edge| edge.to() == target)
            .ok_or_else(|| reference("Swarm simultaneous movement is not an authored route"))?;
        let source_section = instance
            .graph_definition()
            .node(state.current_node())
            .ok_or_else(|| invalid("current Swarm graph node is missing"))?
            .section();
        let target_section = instance
            .graph_definition()
            .node(target)
            .ok_or_else(|| reference("unknown Swarm simultaneous movement target"))?
            .section();
        if source_section != target_section || map_overlay::node_is_blanked(state, target) {
            return Err(reference("Swarm simultaneous movement target is not legal"));
        }
        let countdown = instance.compile_countdown_move(state, &request.countdown_adjustments)?;
        tiers[0].extend(countdown.operations().iter().cloned());
        tiers[0].push(ActivityOperation::Traverse(edge.id()));
    } else if !request.countdown_adjustments.is_empty() {
        return Err(invalid(
            "Swarm simultaneous Countdown adjustments require movement",
        ));
    }

    let face = instance.compile_dice_face_activation(state, request.explicit_face_target, rng)?;
    tiers[1].extend(face.operations().iter().cloned());

    if let Some((target, domain, beacon)) = &request.map_replacement {
        let map = instance.compile_node_replacement(*target, domain, beacon.as_deref())?;
        tiers[2].extend(map.operations().iter().cloned());
    }

    if let Some((story_stage, choice)) = &request.communing_choice {
        let choice = instance.compile_communing_choice(state, *story_stage, choice)?;
        tiers[3].extend(choice.operations().iter().cloned());
    }

    if let Some((cabinet, objective)) = &request.cabinet_completion {
        let cabinet = instance.compile_pathstrider_cabinet_completion(state, cabinet, objective)?;
        tiers[4].extend(cabinet.operations().iter().cloned());
    }

    let mut operations = Vec::new();
    for (index, tier) in tiers.into_iter().enumerate() {
        operations.push(ActivityOperation::AddCounter {
            slot: ActivitySlotId::new(DEFERRED).expect("static Swarm slot is non-zero"),
            key: RESOLUTION_TIER_BASE
                + u64::try_from(index + 1).map_err(|_| invalid("invalid Swarm resolution tier"))?,
            delta: ActivityExpression::Literal(ActivityValue::BoundedInteger(1)),
        });
        operations.extend(tier);
    }
    ActivityProgramDefinition::new(
        ActivityProgramId::new(RESOLUTION_PROGRAM_ID)
            .expect("static Swarm resolution program ID is non-zero"),
        operations,
    )
    .map_err(|_| invalid("invalid Swarm simultaneous resolution program"))
}

fn invalid(message: &'static str) -> UniverseCatalogLoadError {
    UniverseCatalogLoadError::new(UniverseCatalogLoadErrorKind::InvalidDefinition, message)
}

fn reference(message: &'static str) -> UniverseCatalogLoadError {
    UniverseCatalogLoadError::new(UniverseCatalogLoadErrorKind::InvalidReference, message)
}
