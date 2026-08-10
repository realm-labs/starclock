use crate::{
    ConcedePolicy, ControlledAction,
    actor::store::{FormationState, TeamStateStore, UnitStore},
    battle::{spec::TeamSide, state::BattleState},
    catalog::{
        CombatCatalog,
        action::{AbilityKind, ActionSegmentDefinition},
    },
    effect::state::EffectStore,
    id::{AbilityId, DecisionId, UnitId},
    resource::check::can_pay,
    target::select::legal_primary_targets,
};

use super::model::{
    ActionFrameInput, Command, DecisionKind, DecisionOwner, DecisionPoint, UltimateOption,
};

pub(crate) fn battle_start(id: DecisionId) -> DecisionPoint {
    DecisionPoint::new(
        id,
        DecisionKind::BattleStart,
        DecisionOwner::System,
        vec![Command::StartBattle { decision: id }],
    )
}

pub(crate) fn ultimate_options(
    owner: TeamSide,
    units: &UnitStore,
    formations: &FormationState,
    teams: &TeamStateStore,
    effects: &EffectStore,
    catalog: &CombatCatalog,
) -> Vec<UltimateOption> {
    let mut options = Vec::new();
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
                || effects.blocks(unit.id, ControlledAction::Ultimate)
            {
                continue;
            }
            if legal_primary_targets(units, formations, unit.id, selector)
                .is_ok_and(|primaries| !primaries.is_empty())
            {
                options.push(UltimateOption::new(unit.id, ability));
            }
        }
    }
    options.sort_by_key(|option| (option.actor(), option.ability()));
    options.dedup();
    options
}

pub(crate) fn prepared_action(
    id: DecisionId,
    actor: UnitId,
    ability: AbilityId,
    catalog: &CombatCatalog,
    state: &BattleState,
) -> Option<DecisionPoint> {
    let selector = catalog
        .ability(ability)
        .and_then(|definition| catalog.selector(definition.selector()))?
        .unit_targets()?;
    let primaries = legal_primary_targets(&state.units, &state.formations, actor, selector).ok()?;
    let mut commands = primaries
        .into_iter()
        .map(|primary_target| Command::CommitPreparedAction {
            decision: id,
            primary_target,
        })
        .collect::<Vec<_>>();
    commands.push(Command::CancelPreparedAction { decision: id });
    Some(DecisionPoint::new(
        id,
        DecisionKind::PreparedAction,
        DecisionOwner::Team(TeamSide::Player),
        commands,
    ))
}

pub(crate) fn action_frame(
    id: DecisionId,
    catalog: &CombatCatalog,
    state: &BattleState,
) -> Option<DecisionPoint> {
    let frame = state.timeline.action_frame.as_ref()?;
    let flow = catalog.ability(frame.ability)?.action()?.segmented_flow()?;
    let step = flow.steps().get(usize::from(frame.cursor))?;
    let commands = match step {
        ActionSegmentDefinition::SelectTarget { ability } => {
            let selector = catalog
                .ability(*ability)
                .and_then(|definition| catalog.selector(definition.selector()))?
                .unit_targets()?;
            legal_primary_targets(&state.units, &state.formations, frame.actor, selector)
                .ok()?
                .into_iter()
                .flatten()
                .map(|target| Command::CommitActionFrame {
                    decision: id,
                    input: ActionFrameInput::Target(target),
                })
                .collect()
        }
        ActionSegmentDefinition::SelectOption { abilities } => abilities
            .iter()
            .copied()
            .map(|ability| Command::CommitActionFrame {
                decision: id,
                input: ActionFrameInput::Option(ability),
            })
            .collect(),
        ActionSegmentDefinition::Automatic { .. } => return None,
    };
    Some(DecisionPoint::new(
        id,
        DecisionKind::ActionFrame,
        DecisionOwner::Team(TeamSide::Player),
        commands,
    ))
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
            || state.effects.blocks(actor, ControlledAction::NormalAction)
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

pub(crate) fn effective_abilities(
    innate: &[AbilityId],
    effects: &EffectStore,
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
