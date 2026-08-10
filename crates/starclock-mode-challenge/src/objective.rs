use crate::ObjectiveId;

/// Closed objective vocabulary shared by the three challenge profiles.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ObjectiveKind {
    Complete,
    NoDefeatedParticipants,
    RemainingCyclesAtLeast(u16),
    ScoreAtLeast(i64),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Objective {
    id: ObjectiveId,
    kind: ObjectiveKind,
}

impl Objective {
    #[must_use]
    pub const fn new(id: ObjectiveId, kind: ObjectiveKind) -> Self {
        Self { id, kind }
    }
    #[must_use]
    pub const fn id(self) -> ObjectiveId {
        self.id
    }
    #[must_use]
    pub const fn kind(self) -> ObjectiveKind {
        self.kind
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ObjectiveInput {
    pub completed: bool,
    pub any_participant_defeated: bool,
    pub remaining_cycles: Option<u16>,
    pub score: Option<i64>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObjectiveEvaluation {
    awarded: Box<[ObjectiveId]>,
}

impl ObjectiveEvaluation {
    #[must_use]
    pub fn evaluate(objectives: &[Objective], input: ObjectiveInput) -> Self {
        let awarded = objectives
            .iter()
            .copied()
            .filter(|objective| match objective.kind {
                ObjectiveKind::Complete => input.completed,
                ObjectiveKind::NoDefeatedParticipants => {
                    input.completed && !input.any_participant_defeated
                }
                ObjectiveKind::RemainingCyclesAtLeast(threshold) => {
                    input.completed
                        && input
                            .remaining_cycles
                            .is_some_and(|remaining| remaining >= threshold)
                }
                ObjectiveKind::ScoreAtLeast(threshold) => {
                    input.score.is_some_and(|score| score >= threshold)
                }
            })
            .map(Objective::id)
            .collect();
        Self { awarded }
    }

    #[must_use]
    pub fn awarded(&self) -> &[ObjectiveId] {
        &self.awarded
    }

    #[must_use]
    pub fn stars(&self) -> u8 {
        self.awarded.len().try_into().unwrap_or(u8::MAX)
    }
}
