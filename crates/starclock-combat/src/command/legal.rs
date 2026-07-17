use crate::{
    battle::spec::{ConcedePolicy, TeamSide},
    catalog::CombatCatalog,
    id::{AbilityId, DecisionId, UnitId},
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

pub(crate) fn interrupt_window(id: DecisionId, owner: TeamSide) -> DecisionPoint {
    DecisionPoint::new(
        id,
        DecisionKind::InterruptWindow,
        DecisionOwner::Team(owner),
        vec![Command::PassInterruptWindow { decision: id }],
    )
}

pub(crate) fn normal_action(
    id: DecisionId,
    owner: TeamSide,
    actor: UnitId,
    abilities: &[AbilityId],
    catalog: &CombatCatalog,
    concede: ConcedePolicy,
) -> DecisionPoint {
    let mut legal_commands = abilities
        .iter()
        .copied()
        .filter(|ability| {
            catalog
                .ability(*ability)
                .is_some_and(|definition| definition.is_single_hit_action())
        })
        .map(|ability| Command::UseAbility {
            decision: id,
            actor,
            ability,
            primary_target: None,
        })
        .collect::<Vec<_>>();
    if owner == TeamSide::Player {
        match concede {
            ConcedePolicy::Allowed => legal_commands.push(Command::Concede { decision: id }),
        }
    }
    DecisionPoint::new(
        id,
        DecisionKind::NormalAction,
        DecisionOwner::Team(owner),
        legal_commands,
    )
}
