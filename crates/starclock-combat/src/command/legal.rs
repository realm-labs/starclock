use crate::{
    battle::spec::{ConcedePolicy, TeamSide},
    id::DecisionId,
};

use super::model::{Command, DecisionKind, DecisionOwner, DecisionPoint};

pub(crate) fn battle_start(id: DecisionId) -> DecisionPoint {
    DecisionPoint::new(
        id,
        DecisionKind::BattleStart,
        DecisionOwner::System,
        vec![Command::StartBattle { decision: id }],
    )
}

pub(crate) fn initial_player_action(id: DecisionId, concede: ConcedePolicy) -> DecisionPoint {
    let legal_commands = match concede {
        ConcedePolicy::Allowed => vec![Command::Concede { decision: id }],
    };
    DecisionPoint::new(
        id,
        DecisionKind::NormalAction,
        DecisionOwner::Team(TeamSide::Player),
        legal_commands,
    )
}
