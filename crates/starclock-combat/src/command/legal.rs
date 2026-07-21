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
        for ability in effective_abilities(&unit.abilities, effects, catalog, unit.id) {
            let Some((action, selector)) = catalog.ability(ability).and_then(|definition| {
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
                        ability,
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
    for ability in effective_abilities(abilities, &state.effects, catalog, actor) {
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

fn effective_abilities(
    innate: &[AbilityId],
    effects: &crate::effect::state::EffectStore,
    catalog: &CombatCatalog,
    actor: UnitId,
) -> Vec<AbilityId> {
    let mut abilities = innate.to_vec();
    abilities.extend(
        effects
            .iter_by_id()
            .filter(|effect| effect.target == actor)
            .filter_map(|effect| catalog.effect(effect.definition))
            .flat_map(|effect| effect.granted_abilities().iter().copied()),
    );
    abilities.sort_unstable();
    abilities.dedup();
    abilities
}

pub(crate) fn ability_owner(
    state: &BattleState,
    catalog: &CombatCatalog,
    actor: UnitId,
    ability: AbilityId,
) -> Option<UnitId> {
    let unit = state.units.get(actor)?;
    if unit.abilities.binary_search(&ability).is_ok() {
        return Some(actor);
    }
    state
        .effects
        .iter_by_id()
        .filter(|effect| effect.target == actor)
        .filter(|effect| {
            catalog.effect(effect.definition).is_some_and(|definition| {
                definition
                    .granted_abilities()
                    .binary_search(&ability)
                    .is_ok()
            })
        })
        .min_by_key(|effect| effect.id)
        .map(|effect| effect.applier)
}
