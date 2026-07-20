use crate::{
    actor::store::{FormationState, TeamStateStore, UnitStore},
    battle::{spec::TeamSide, state::BattleState},
    catalog::CombatCatalog,
    catalog::action::AbilityKind,
    id::{AbilityId, DecisionId, UnitId},
    resource::check::can_pay,
    target::select::legal_primary_targets,
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

pub(crate) fn interrupt_window(
    id: DecisionId,
    owner: TeamSide,
    units: &UnitStore,
    formations: &FormationState,
    teams: &TeamStateStore,
    effects: &crate::effect::state::EffectStore,
    catalog: &CombatCatalog,
) -> DecisionPoint {
    let mut commands = vec![Command::PassInterruptWindow { decision: id }];
    for unit in units.iter_by_id().filter(|unit| unit.side == owner) {
        for ability in &unit.abilities {
            let Some((action, selector)) = catalog.ability(*ability).and_then(|definition| {
                Some((
                    definition.action()?,
                    catalog.selector(definition.selector())?.unit_targets()?,
                ))
            }) else {
                continue;
            };
            if action.kind() != AbilityKind::Ultimate
                || !can_pay(units, teams, unit.id, action.resources())
                || effects.blocks(unit.id, crate::ControlledAction::Ultimate)
            {
                continue;
            }
            if let Ok(primaries) = legal_primary_targets(units, formations, unit.id, selector) {
                commands.extend(primaries.into_iter().map(|primary_target| {
                    Command::UseInterrupt {
                        decision: id,
                        actor: unit.id,
                        ability: *ability,
                        primary_target,
                    }
                }));
            }
        }
    }
    DecisionPoint::new(
        id,
        DecisionKind::InterruptWindow,
        DecisionOwner::Team(owner),
        commands,
    )
}

pub(crate) fn normal_action(
    id: DecisionId,
    owner: TeamSide,
    actor: UnitId,
    abilities: &[AbilityId],
    catalog: &CombatCatalog,
    state: &BattleState,
) -> DecisionPoint {
    let mut legal_commands = Vec::new();
    for ability in abilities.iter().copied() {
        let Some((action, selector)) = catalog.ability(ability).and_then(|definition| {
            Some((
                definition.action()?,
                catalog.selector(definition.selector())?.unit_targets()?,
            ))
        }) else {
            continue;
        };
        if !action.kind().is_normal_turn()
            || !can_pay(&state.units, &state.teams, actor, action.resources())
            || state
                .effects
                .blocks(actor, crate::ControlledAction::NormalAction)
        {
            continue;
        }
        if let Ok(primaries) =
            legal_primary_targets(&state.units, &state.formations, actor, selector)
        {
            legal_commands.extend(primaries.into_iter().map(|primary_target| {
                Command::UseAbility {
                    decision: id,
                    actor,
                    ability,
                    primary_target,
                }
            }));
        }
    }
    if owner == TeamSide::Player {
        match state.concede {
            crate::ConcedePolicy::Allowed => legal_commands.push(Command::Concede { decision: id }),
        }
    }
    DecisionPoint::new(
        id,
        DecisionKind::NormalAction,
        DecisionOwner::Team(owner),
        legal_commands,
    )
}
