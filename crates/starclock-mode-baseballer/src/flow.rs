use starclock_activity::{
    ActivityEdgeCondition, ActivityEdgeDefinition, ActivityEdgeId, ActivityGraphDefinition,
    ActivityNodeDefinition, ActivityNodeKind, ActivityTerminalOutcome, NodeId, SectionId,
    TerminalOutcome,
};

use crate::{BaseballerPeriodRank, BaseballerStagePeriod};

/// Shared-Activity topology for one Galactic Baseballer stage.
///
/// Released descriptions establish three combat phases with an equipment
/// choice between adjacent phases. Equipment candidates and their mutations
/// remain mode-owned option programs layered onto the two Reward nodes.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct BaseballerStageFlow;

impl BaseballerStageFlow {
    pub fn compile(
        section: SectionId,
        periods: &[BaseballerStagePeriod],
    ) -> Result<ActivityGraphDefinition, BaseballerFlowError> {
        let ranks = period_ranks(periods)?;
        let battle_count = u32::try_from(ranks.len()).map_err(|_| error("too many periods"))?;
        let terminal_base = battle_count
            .checked_mul(2)
            .ok_or_else(|| error("stage node identity overflow"))?;
        let completed = terminal_base;
        let failed = terminal_base
            .checked_add(1)
            .ok_or_else(|| error("stage node identity overflow"))?;
        let faulted = terminal_base
            .checked_add(2)
            .ok_or_else(|| error("stage node identity overflow"))?;
        let mut nodes = Vec::new();
        for index in 0..battle_count {
            nodes.push(node(index * 2 + 1, section, ActivityNodeKind::Battle)?);
            if index + 1 < battle_count {
                nodes.push(node(index * 2 + 2, section, ActivityNodeKind::Reward)?);
            }
        }
        nodes.extend([
            node(
                completed,
                section,
                ActivityNodeKind::Terminal(ActivityTerminalOutcome::Completed),
            )?,
            node(
                failed,
                section,
                ActivityNodeKind::Terminal(ActivityTerminalOutcome::Failed),
            )?,
            node(
                faulted,
                section,
                ActivityNodeKind::Terminal(ActivityTerminalOutcome::Faulted),
            )?,
        ]);
        let mut edges = Vec::new();
        let mut next_edge = 1;
        for index in 0..battle_count {
            let battle = index * 2 + 1;
            let success = if index + 1 == battle_count {
                completed
            } else {
                battle + 1
            };
            edges.push(outcome_edge(
                next_edge,
                battle,
                success,
                TerminalOutcome::Complete,
            )?);
            next_edge += 1;
            edges.push(outcome_edge(
                next_edge,
                battle,
                failed,
                TerminalOutcome::Failed,
            )?);
            next_edge += 1;
            edges.push(outcome_edge(
                next_edge,
                battle,
                faulted,
                TerminalOutcome::Faulted,
            )?);
            next_edge += 1;
            if index + 1 < battle_count {
                edges.push(option_edge(next_edge, battle + 1, battle + 2)?);
                next_edge += 1;
            }
        }
        let maximum_visits = terminal_base
            .checked_add(2)
            .ok_or_else(|| error("stage visit limit overflow"))?;
        ActivityGraphDefinition::new(node_id(1), nodes, edges, maximum_visits).map_err(debug_error)
    }
}

fn period_ranks(
    periods: &[BaseballerStagePeriod],
) -> Result<Vec<BaseballerPeriodRank>, BaseballerFlowError> {
    let mut ranks = periods.iter().map(|period| period.rank).collect::<Vec<_>>();
    ranks.sort_unstable();
    ranks.dedup();
    let expected = [
        BaseballerPeriodRank::First,
        BaseballerPeriodRank::Second,
        BaseballerPeriodRank::Third,
        BaseballerPeriodRank::Extra,
    ];
    if ranks.is_empty() || ranks.as_slice() != &expected[..ranks.len()] {
        return Err(error("stage period ranks must be contiguous from First"));
    }
    Ok(ranks)
}

fn node(
    raw: u32,
    section: SectionId,
    kind: ActivityNodeKind,
) -> Result<ActivityNodeDefinition, BaseballerFlowError> {
    ActivityNodeDefinition::new(node_id(raw), section, kind, 1).map_err(debug_error)
}

fn outcome_edge(
    id: u32,
    from: u32,
    to: u32,
    outcome: TerminalOutcome,
) -> Result<ActivityEdgeDefinition, BaseballerFlowError> {
    ActivityEdgeDefinition::new(
        edge_id(id),
        node_id(from),
        node_id(to),
        ActivityEdgeCondition::BattleOutcome(outcome),
        i32::try_from(id).expect("stage edge IDs fit i32"),
        1,
    )
    .map_err(debug_error)
}

fn option_edge(id: u32, from: u32, to: u32) -> Result<ActivityEdgeDefinition, BaseballerFlowError> {
    ActivityEdgeDefinition::new(
        edge_id(id),
        node_id(from),
        node_id(to),
        ActivityEdgeCondition::OptionSelected,
        i32::try_from(id).expect("stage edge IDs fit i32"),
        1,
    )
    .map_err(debug_error)
}

fn node_id(raw: u32) -> NodeId {
    NodeId::new(raw).expect("stage node IDs are non-zero")
}

fn edge_id(raw: u32) -> ActivityEdgeId {
    ActivityEdgeId::new(raw).expect("stage edge IDs are non-zero")
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BaseballerFlowError {
    message: Box<str>,
}

impl std::fmt::Display for BaseballerFlowError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for BaseballerFlowError {}

fn debug_error(error: impl std::fmt::Debug) -> BaseballerFlowError {
    BaseballerFlowError {
        message: format!("{error:?}").into_boxed_str(),
    }
}

fn error(message: &str) -> BaseballerFlowError {
    BaseballerFlowError {
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    use starclock_activity::{ActivityNodeKind, SectionId};
    use starclock_combat::EncounterId;

    use super::BaseballerStageFlow;
    use crate::{
        BaseballerPeriodRank, BaseballerStageId, BaseballerStagePeriod, BaseballerStagePeriodId,
    };

    #[test]
    fn stage_uses_three_battles_and_two_interphase_choices() {
        let periods = [
            period(1, BaseballerPeriodRank::First),
            period(2, BaseballerPeriodRank::Second),
            period(3, BaseballerPeriodRank::Third),
        ];
        let graph = BaseballerStageFlow::compile(SectionId::new(1).unwrap(), &periods).unwrap();
        assert_eq!(
            graph
                .nodes()
                .iter()
                .filter(|node| node.kind() == ActivityNodeKind::Battle)
                .count(),
            3
        );
        assert_eq!(
            graph
                .nodes()
                .iter()
                .filter(|node| node.kind() == ActivityNodeKind::Reward)
                .count(),
            2
        );
    }

    #[test]
    fn stage_shape_follows_authored_period_ranks() {
        let periods = [
            period(1, BaseballerPeriodRank::First),
            period(2, BaseballerPeriodRank::Second),
        ];
        let graph = BaseballerStageFlow::compile(SectionId::new(1).unwrap(), &periods).unwrap();

        assert_eq!(
            graph
                .nodes()
                .iter()
                .filter(|node| node.kind() == ActivityNodeKind::Battle)
                .count(),
            2
        );
        assert_eq!(
            graph
                .nodes()
                .iter()
                .filter(|node| node.kind() == ActivityNodeKind::Reward)
                .count(),
            1
        );
    }

    fn period(raw: u32, rank: BaseballerPeriodRank) -> BaseballerStagePeriod {
        BaseballerStagePeriod {
            id: BaseballerStagePeriodId::new(raw).unwrap(),
            stage: BaseballerStageId::new(1).unwrap(),
            rank,
            encounter: EncounterId::new(raw).unwrap(),
            battle_event_id: raw,
            wave_count: 1,
            countdown_by_wave: Box::new([99]),
            period_score: 1,
            stage_score: Some(1),
            selection_weight: 1,
        }
    }
}
