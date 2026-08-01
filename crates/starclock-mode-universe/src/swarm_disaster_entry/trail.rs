//! Communing Trail selection, Activity effects and immutable battle projections.

use std::collections::{BTreeMap, BTreeSet};

use serde::Deserialize;
use starclock_activity::{
    ActivityCondition, ActivityExpression, ActivityOperation, ActivityProgramDefinition,
    ActivityProgramId, ActivitySlotId, ActivityTransactionState, ActivityValue,
};

use crate::{
    digest::Encoder,
    error::{UniverseCatalogLoadError, UniverseCatalogLoadErrorKind},
    swarm_disaster_unique::runtime_access::{SwarmTrailEffectRuntimeInput, SwarmTrailRuntimeInput},
};

use super::{
    dice_control::CHEAT_CHARGE_KEY,
    state::{COUNTDOWN, PROGRESSION, RESOURCES},
};

pub(super) const RUN_START_APPLIED_KEY: u64 = 2;
const FIRST_PLANE_ENTRY_BATTLES_KEY: u64 = 3;
const COSMIC_FRAGMENTS_KEY: u64 = 1;
const RUN_START_PROGRAM_ID: u32 = 0x5344_4101;
const BATTLE_ENTRY_PROGRAM_ID: u32 = 0x5344_4102;
const PREDECESSOR_POLICY: &str = "ReleasedThresholdThenStableTalentIdImmediatePredecessor";

const INITIAL_FRAGMENTS_EFFECT: &str = "source-effect.104";
const INITIAL_CHEAT_EFFECT: &str = "source-effect.204";
const ABANDON_REWARD_EFFECT: &str = "source-effect.304";
const INITIAL_COUNTDOWN_EFFECT: &str = "source-effect.404";
const BOSS_ENERGY_EFFECT: &str = "source-effect.504";
const FIRST_PLANE_ENTRY_EFFECT: &str = "source-effect.604";
const NEXT_PLANE_REROLL_EFFECT: &str = "source-effect.704";

#[derive(Clone, Debug)]
pub(super) struct TrailRuntimeCatalog {
    nodes: Box<[RuntimeTrailNode]>,
}

#[derive(Clone, Debug)]
struct RuntimeTrailNode {
    id: u32,
    key: Box<str>,
    dimension_id: u32,
    threshold: u16,
    prerequisites: Box<[Box<str>]>,
    effect: RuntimeTrailEffect,
}

#[derive(Clone, Debug)]
struct RuntimeTrailEffect {
    node_id: u32,
    key: Box<str>,
    effect_ref: Box<str>,
    parameters: Box<[Box<str>]>,
    battle_projection: bool,
}

#[derive(Clone, Debug)]
pub(super) struct CompiledTrailRuntime {
    nodes: Box<[RuntimeTrailNode]>,
    battle: Box<[RuntimeBattleContribution]>,
    initial_fragments: i64,
    initial_cheats: i64,
    abandon_reward: i64,
    initial_countdown: i64,
    next_plane_rerolls: i64,
    fixed_entry: Option<FixedEntryEffect>,
    digest: [u8; 32],
}

#[derive(Clone, Debug)]
struct RuntimeBattleContribution {
    source_node: Box<str>,
    effect_key: Box<str>,
    effect_ref: Box<str>,
    parameters: Box<[Box<str>]>,
}

#[derive(Clone, Debug)]
struct FixedEntryEffect {
    eligible_battle_limit: u32,
}

impl TrailRuntimeCatalog {
    pub(super) fn compile(input: SwarmTrailRuntimeInput) -> Result<Self, UniverseCatalogLoadError> {
        let effects = input
            .effects
            .iter()
            .map(|effect| Ok((effect.key.clone(), compile_effect(effect)?)))
            .collect::<Result<BTreeMap<_, _>, UniverseCatalogLoadError>>()?;
        let prerequisites = input
            .prerequisites
            .iter()
            .map(|row| (row.key.clone(), row))
            .collect::<BTreeMap<_, _>>();
        if input.nodes.len() != 63
            || input.effects.len() != 63
            || effects.len() != 63
            || input.prerequisites.len() != 56
            || prerequisites.len() != 56
        {
            return Err(invalid("Swarm Communing Trail denominator drift"));
        }

        let node_keys = input
            .nodes
            .iter()
            .map(|node| (node.id, node.key.as_ref()))
            .collect::<BTreeMap<_, _>>();
        let mut nodes = Vec::with_capacity(input.nodes.len());
        let mut used_effects = BTreeSet::new();
        let mut used_prerequisites = BTreeSet::new();
        for node in &input.nodes {
            let [effect_key] = node.effect_keys.as_ref() else {
                return Err(invalid("Communing Trail node must own one effect"));
            };
            let effect = effects
                .get(effect_key)
                .ok_or_else(|| invalid("Communing Trail effect reference is missing"))?
                .clone();
            if effect.node_id != node.id {
                return Err(invalid("Communing Trail effect owner drift"));
            }
            if !used_effects.insert(effect_key.as_ref()) {
                return Err(invalid("Communing Trail effect is multiply owned"));
            }
            let threshold = positive_u16(&node.threshold)?;
            let mut required = Vec::with_capacity(node.prerequisite_keys.len());
            for key in &node.prerequisite_keys {
                let row = prerequisites
                    .get(key)
                    .ok_or_else(|| invalid("Communing Trail prerequisite is missing"))?;
                if row.node_id != node.id
                    || row.ordinal != 0
                    || positive_u16(&row.required_points)? != threshold
                {
                    return Err(invalid("Communing Trail prerequisite contract drift"));
                }
                let required_key = node_keys
                    .get(&row.required_node_id)
                    .ok_or_else(|| invalid("Communing Trail predecessor is missing"))?;
                required.push((*required_key).into());
                if !used_prerequisites.insert(key.as_ref()) {
                    return Err(invalid("Communing Trail prerequisite is multiply owned"));
                }
            }
            nodes.push(RuntimeTrailNode {
                id: node.id,
                key: node.key.clone(),
                dimension_id: node.dimension_id,
                threshold,
                prerequisites: required.into_boxed_slice(),
                effect,
            });
        }
        nodes.sort_unstable_by_key(|node| (node.dimension_id, node.threshold, node.id));
        validate_nodes(&nodes, &used_effects, &used_prerequisites)?;
        Ok(Self {
            nodes: nodes.into_boxed_slice(),
        })
    }

    pub(super) fn select(
        &self,
        progression: &[Box<str>],
        communing: &[(u32, u16)],
    ) -> Result<CompiledTrailRuntime, UniverseCatalogLoadError> {
        let requested = progression.iter().map(Box::as_ref).collect::<BTreeSet<_>>();
        let selected = self
            .nodes
            .iter()
            .filter(|node| requested.contains(node.key.as_ref()))
            .cloned()
            .collect::<Vec<_>>();
        for node in &selected {
            let points = communing
                .binary_search_by_key(&node.dimension_id, |(id, _)| *id)
                .ok()
                .map_or(0, |index| communing[index].1);
            if points < node.threshold {
                return Err(reference("Communing Trail threshold is not met"));
            }
            if node
                .prerequisites
                .iter()
                .any(|key| !requested.contains(key.as_ref()))
            {
                return Err(reference("Communing Trail prerequisite is not unlocked"));
            }
        }
        compile_selected(selected)
    }

    #[cfg(test)]
    pub(super) fn denominators(&self) -> (usize, usize, usize) {
        (
            self.nodes.len(),
            self.nodes.iter().map(|node| node.prerequisites.len()).sum(),
            self.nodes
                .iter()
                .filter(|node| node.effect.battle_projection)
                .count(),
        )
    }
}

impl CompiledTrailRuntime {
    pub(super) fn compile_run_start(
        &self,
        state: &ActivityTransactionState,
    ) -> Result<ActivityProgramDefinition, UniverseCatalogLoadError> {
        if counter_value(state, PROGRESSION, RUN_START_APPLIED_KEY)? != 0 {
            return Err(invalid("Communing Trail run-start effects already applied"));
        }
        let mut operations = vec![require_counter(PROGRESSION, RUN_START_APPLIED_KEY, 0)];
        add_if_nonzero(
            &mut operations,
            RESOURCES,
            COSMIC_FRAGMENTS_KEY,
            self.initial_fragments,
        );
        if self.initial_countdown != 0 {
            operations.push(ActivityOperation::AddToSlot {
                slot: slot(COUNTDOWN),
                delta: integer(self.initial_countdown),
            });
        }
        add_if_nonzero(
            &mut operations,
            RESOURCES,
            CHEAT_CHARGE_KEY,
            self.initial_cheats,
        );
        operations.push(add_counter(PROGRESSION, RUN_START_APPLIED_KEY, 1));
        program(RUN_START_PROGRAM_ID, operations)
    }

    pub(super) const fn abandon_reward(&self) -> i64 {
        self.abandon_reward
    }

    pub(super) const fn next_plane_rerolls(&self) -> i64 {
        self.next_plane_rerolls
    }

    pub(super) fn compile_battle_entry_accounting(
        &self,
        state: &ActivityTransactionState,
        plane_layer: u8,
        boss: bool,
        previous_first_plane_completed: bool,
    ) -> Result<Option<ActivityProgramDefinition>, UniverseCatalogLoadError> {
        if !(1..=3).contains(&plane_layer) {
            return Err(invalid("invalid Swarm plane layer"));
        }
        let Some(effect) = &self.fixed_entry else {
            return Ok(None);
        };
        if plane_layer != 1 || boss || !previous_first_plane_completed {
            return Ok(None);
        }
        let consumed = counter_value(state, PROGRESSION, FIRST_PLANE_ENTRY_BATTLES_KEY)?;
        if consumed >= i64::from(effect.eligible_battle_limit) {
            return Ok(None);
        }
        program(
            BATTLE_ENTRY_PROGRAM_ID,
            vec![
                ActivityOperation::Require(ActivityCondition::All(
                    vec![
                        ActivityCondition::Equal(
                            counter(PROGRESSION, FIRST_PLANE_ENTRY_BATTLES_KEY),
                            integer(consumed),
                        ),
                        ActivityCondition::LessThan(
                            counter(PROGRESSION, FIRST_PLANE_ENTRY_BATTLES_KEY),
                            integer(i64::from(effect.eligible_battle_limit)),
                        ),
                    ]
                    .into_boxed_slice(),
                )),
                add_counter(PROGRESSION, FIRST_PLANE_ENTRY_BATTLES_KEY, 1),
            ],
        )
        .map(Some)
    }

    pub(super) fn nodes(&self) -> impl ExactSizeIterator<Item = (&str, u16)> {
        self.nodes
            .iter()
            .map(|node| (node.key.as_ref(), node.threshold))
    }

    pub(super) fn prerequisites(&self, node: &str) -> Option<impl ExactSizeIterator<Item = &str>> {
        self.nodes
            .iter()
            .find(|candidate| candidate.key.as_ref() == node)
            .map(|node| node.prerequisites.iter().map(Box::as_ref))
    }

    pub(super) fn battle(&self) -> impl ExactSizeIterator<Item = (&str, &str, &str)> {
        self.battle.iter().map(|entry| {
            (
                entry.source_node.as_ref(),
                entry.effect_key.as_ref(),
                entry.effect_ref.as_ref(),
            )
        })
    }

    pub(super) fn battle_parameters(
        &self,
        effect_ref: &str,
    ) -> Option<impl ExactSizeIterator<Item = &str>> {
        self.battle
            .iter()
            .find(|entry| entry.effect_ref.as_ref() == effect_ref)
            .map(|entry| entry.parameters.iter().map(Box::as_ref))
    }

    pub(super) const fn digest(&self) -> [u8; 32] {
        self.digest
    }

    #[cfg(test)]
    pub(super) const fn activity_totals(&self) -> (i64, i64, i64, i64, i64) {
        (
            self.initial_fragments,
            self.initial_cheats,
            self.abandon_reward,
            self.initial_countdown,
            self.next_plane_rerolls,
        )
    }
}

fn compile_selected(
    nodes: Vec<RuntimeTrailNode>,
) -> Result<CompiledTrailRuntime, UniverseCatalogLoadError> {
    let mut battle = Vec::new();
    let mut initial_fragments = 0;
    let mut initial_cheats = 0;
    let mut abandon_reward = 0;
    let mut initial_countdown = 0;
    let mut next_plane_rerolls = 0;
    let mut fixed_entry = None;
    let mut encoder = Encoder::new(b"starclock.swarm-disaster.communing-trail.v1");
    encoder.text(PREDECESSOR_POLICY);
    encoder.u32(u32::try_from(nodes.len()).map_err(|_| invalid("too many Trail nodes"))?);
    for node in &nodes {
        let effect = &node.effect;
        encoder.text(&node.key);
        encoder.u32(node.dimension_id);
        encoder.u32(u32::from(node.threshold));
        encoder.text(&effect.key);
        encoder.text(&effect.effect_ref);
        encoder.u32(
            u32::try_from(effect.parameters.len())
                .map_err(|_| invalid("too many Trail effect parameters"))?,
        );
        for parameter in &effect.parameters {
            encoder.text(parameter);
        }
        if effect.battle_projection {
            battle.push(RuntimeBattleContribution {
                source_node: node.key.clone(),
                effect_key: effect.key.clone(),
                effect_ref: effect.effect_ref.clone(),
                parameters: effect.parameters.clone(),
            });
        }
        match effect.effect_ref.as_ref() {
            INITIAL_FRAGMENTS_EFFECT => {
                initial_fragments = add_exact(initial_fragments, one_integer(effect)?)?;
            }
            INITIAL_CHEAT_EFFECT => {
                initial_cheats = add_exact(initial_cheats, one_integer(effect)?)?;
            }
            ABANDON_REWARD_EFFECT => {
                abandon_reward = add_exact(abandon_reward, one_integer(effect)?)?;
            }
            INITIAL_COUNTDOWN_EFFECT => {
                initial_countdown = add_exact(initial_countdown, one_integer(effect)?)?;
            }
            NEXT_PLANE_REROLL_EFFECT => {
                next_plane_rerolls = add_exact(next_plane_rerolls, one_integer(effect)?)?;
            }
            BOSS_ENERGY_EFFECT => require_parameter_count(effect, 0)?,
            FIRST_PLANE_ENTRY_EFFECT => {
                require_parameter_count(effect, 2)?;
                if fixed_entry.is_some() || effect.parameters[1].as_ref() != "0.99" {
                    return Err(invalid("invalid Communing Trail entry-damage effect"));
                }
                fixed_entry = Some(FixedEntryEffect {
                    eligible_battle_limit: effect.parameters[0]
                        .parse::<u32>()
                        .ok()
                        .filter(|value| *value > 0)
                        .ok_or_else(|| invalid("invalid Trail entry battle limit"))?,
                });
            }
            _ if effect.battle_projection => {}
            _ => return Err(invalid("unknown Communing Trail Activity effect")),
        }
    }
    Ok(CompiledTrailRuntime {
        nodes: nodes.into_boxed_slice(),
        battle: battle.into_boxed_slice(),
        initial_fragments,
        initial_cheats,
        abandon_reward,
        initial_countdown,
        next_plane_rerolls,
        fixed_entry,
        digest: encoder.finish(),
    })
}

fn compile_effect(
    input: &SwarmTrailEffectRuntimeInput,
) -> Result<RuntimeTrailEffect, UniverseCatalogLoadError> {
    let operations = serde_json::from_str::<Vec<ReleasedOperation>>(&input.operations)
        .map_err(|_| invalid("invalid Communing Trail effect program"))?;
    let [operation] = operations.as_slice() else {
        return Err(invalid("Trail effect must contain one released operation"));
    };
    if input.id == 0
        || input.node_id == 0
        || input.ordinal != 0
        || operation.operation.as_ref() != "ApplyReleasedGameplayEffect"
        || operation.order != 0
        || operation.effect_ref.is_empty()
    {
        return Err(invalid("invalid Communing Trail effect descriptor"));
    }
    let projection = serde_json::from_str::<BattleProjection>(&input.battle_projection)
        .map_err(|_| invalid("invalid Communing Trail battle projection"))?;
    let battle_projection = match input.domain.as_ref() {
        "Battle" | "ActivityAndBattle"
            if projection.enabled
                && projection.boundary.as_ref() == "BattleSpecCreation"
                && projection.effect_ref == operation.effect_ref =>
        {
            true
        }
        "Activity"
            if !projection.enabled
                && projection.boundary.as_ref() == "NotApplicable"
                && projection.effect_ref.is_empty() =>
        {
            false
        }
        _ => return Err(invalid("Communing Trail effect domain drift")),
    };
    Ok(RuntimeTrailEffect {
        node_id: input.node_id,
        key: input.key.clone(),
        effect_ref: operation.effect_ref.clone(),
        parameters: operation.parameters.clone(),
        battle_projection,
    })
}

fn validate_nodes(
    nodes: &[RuntimeTrailNode],
    effects: &BTreeSet<&str>,
    prerequisites: &BTreeSet<&str>,
) -> Result<(), UniverseCatalogLoadError> {
    if effects.len() != 63 || prerequisites.len() != 56 {
        return Err(invalid("Communing Trail closure drift"));
    }
    let keys = nodes
        .iter()
        .map(|node| node.key.as_ref())
        .collect::<BTreeSet<_>>();
    let mut dimensions = BTreeMap::<u32, Vec<&RuntimeTrailNode>>::new();
    for node in nodes {
        dimensions.entry(node.dimension_id).or_default().push(node);
        if node
            .prerequisites
            .iter()
            .any(|required| !keys.contains(required.as_ref()))
        {
            return Err(invalid("Communing Trail prerequisite closure drift"));
        }
    }
    if dimensions.len() != 7 {
        return Err(invalid("Communing Trail dimension denominator drift"));
    }
    for rows in dimensions.values() {
        let thresholds = rows.iter().map(|node| node.threshold).collect::<Vec<_>>();
        if thresholds != [1, 3, 5, 7, 9, 11, 13, 16, 20]
            || !rows[0].prerequisites.is_empty()
            || rows[1..].iter().any(|node| node.prerequisites.len() != 1)
            || rows
                .windows(2)
                .any(|pair| pair[1].prerequisites[0].as_ref() != pair[0].key.as_ref())
        {
            return Err(invalid("Communing Trail threshold chain drift"));
        }
    }
    Ok(())
}

fn one_integer(effect: &RuntimeTrailEffect) -> Result<i64, UniverseCatalogLoadError> {
    require_parameter_count(effect, 1)?;
    effect.parameters[0]
        .parse::<i64>()
        .ok()
        .filter(|value| *value > 0)
        .ok_or_else(|| invalid("invalid Communing Trail integer parameter"))
}

fn require_parameter_count(
    effect: &RuntimeTrailEffect,
    expected: usize,
) -> Result<(), UniverseCatalogLoadError> {
    if effect.parameters.len() == expected {
        Ok(())
    } else {
        Err(invalid("Communing Trail parameter denominator drift"))
    }
}

fn positive_u16(value: &str) -> Result<u16, UniverseCatalogLoadError> {
    value
        .parse::<u16>()
        .ok()
        .filter(|value| *value > 0)
        .ok_or_else(|| invalid("invalid Communing Trail threshold"))
}

fn add_exact(left: i64, right: i64) -> Result<i64, UniverseCatalogLoadError> {
    left.checked_add(right)
        .ok_or_else(|| invalid("Communing Trail effect total overflow"))
}

fn add_if_nonzero(operations: &mut Vec<ActivityOperation>, slot_id: u32, key: u64, delta: i64) {
    if delta != 0 {
        operations.push(add_counter(slot_id, key, delta));
    }
}

fn require_counter(slot_id: u32, key: u64, expected: i64) -> ActivityOperation {
    ActivityOperation::Require(ActivityCondition::Equal(
        counter(slot_id, key),
        integer(expected),
    ))
}

pub(super) fn add_counter(slot_id: u32, key: u64, delta: i64) -> ActivityOperation {
    ActivityOperation::AddCounter {
        slot: slot(slot_id),
        key,
        delta: integer(delta),
    }
}

fn counter(slot_id: u32, key: u64) -> ActivityExpression {
    ActivityExpression::CounterValue {
        slot: slot(slot_id),
        key,
    }
}

fn integer(value: i64) -> ActivityExpression {
    ActivityExpression::Literal(ActivityValue::BoundedInteger(value))
}

fn counter_value(
    state: &ActivityTransactionState,
    slot_id: u32,
    key: u64,
) -> Result<i64, UniverseCatalogLoadError> {
    let Some(ActivityValue::BoundedCounterMap(values)) = state.slot(slot(slot_id)) else {
        return Err(invalid("invalid Communing Trail Activity slot"));
    };
    Ok(values
        .binary_search_by_key(&key, |(candidate, _)| *candidate)
        .ok()
        .map_or(0, |index| values[index].1))
}

fn slot(raw: u32) -> ActivitySlotId {
    ActivitySlotId::new(raw).expect("static Swarm slot ID is non-zero")
}

fn program(
    raw: u32,
    operations: Vec<ActivityOperation>,
) -> Result<ActivityProgramDefinition, UniverseCatalogLoadError> {
    ActivityProgramDefinition::new(
        ActivityProgramId::new(raw).expect("static Swarm program ID is non-zero"),
        operations,
    )
    .map_err(|_| invalid("invalid Communing Trail Activity program"))
}

fn invalid(message: &'static str) -> UniverseCatalogLoadError {
    UniverseCatalogLoadError::new(UniverseCatalogLoadErrorKind::InvalidDefinition, message)
}

fn reference(message: &'static str) -> UniverseCatalogLoadError {
    UniverseCatalogLoadError::new(UniverseCatalogLoadErrorKind::InvalidReference, message)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ReleasedOperation {
    effect_ref: Box<str>,
    operation: Box<str>,
    order: u16,
    parameters: Box<[Box<str>]>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BattleProjection {
    boundary: Box<str>,
    effect_ref: Box<str>,
    enabled: bool,
}
