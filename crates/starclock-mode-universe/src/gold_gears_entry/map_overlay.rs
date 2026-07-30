//! Typed chessboard overlays and deterministic map-program compilation.

use starclock_activity::{
    ActivityExpression, ActivityOperation, ActivityProgramDefinition, ActivityProgramId,
    ActivityRngLabel, ActivityRngStreams, ActivitySlotId, ActivityValue, NodeId,
};

use crate::{
    gold_gears_content::{
        BlockCreateRule, GoldAndGearsContentCatalog, MapEventEffect, MapEventTrigger,
    },
    gold_gears_structural::GoldAndGearsStructuralCatalog,
};

use super::{
    GoldAndGearsEntryError,
    state_layout::{
        BOARD_NODE_BEACON_SLOT, BOARD_NODE_DOMAIN_SLOT, BOARD_NODE_STATE_SLOT, PLANE_STATE_SLOT,
    },
};

pub(super) const MAP_EVENT_PURPOSE: u16 = 0x4701;
pub(super) const CREATE_COUNT_PURPOSE: u16 = 0x4702;
pub(super) const BEACON_PURPOSE: u16 = 0x4703;

const CREATION_PROGRAM_BASE: u32 = 0x4700_0000;
const EVENT_CREATION_PROGRAM_BASE: u32 = 0x4710_0000;
const REPLACEMENT_PROGRAM_BASE: u32 = 0x4720_0000;
const COPY_PROGRAM_BASE: u32 = 0x4730_0000;
const BLANK_PROGRAM_BASE: u32 = 0x4740_0000;

const NODE_STATE_CREATED: i64 = 1;
const NODE_STATE_REPLACED: i64 = 2;
const NODE_STATE_COPIED: i64 = 3;
pub(super) const NODE_STATE_BLANKED: i64 = 4;

const PLANE_LAST_MAP_EVENT_KEY: u64 = 1;
const PLANE_LAST_MAP_EFFECT_KEY: u64 = 2;
const PLANE_LAST_MAP_PARAMETER_KEY: u64 = 3;
const PLANE_ACTION_POINTS_KEY: u64 = 4;

#[derive(Debug)]
pub(super) struct MapRuntimeCatalog {
    boards: Box<[BoardMapDefinition]>,
    domains: Box<[(Box<str>, u32)]>,
    beacons: Box<[(Box<str>, u32)]>,
}

#[derive(Debug)]
struct BoardMapDefinition {
    key: Box<str>,
    nodes: Box<[u32]>,
    events: Box<[RuntimeMapEvent]>,
    rules: Box<[RuntimeBlockRule]>,
}

#[derive(Debug)]
struct RuntimeMapEvent {
    id: u32,
    trigger: MapEventTrigger,
    trigger_parameters: Box<[u32]>,
    effect: MapEventEffect,
    effect_parameters: Box<[u32]>,
    secondary_effect_parameters: Box<[u32]>,
    weight: u64,
}

#[derive(Debug)]
struct RuntimeBlockRule {
    order: u16,
    domain: u32,
    create_counts: Box<[(u16, u64)]>,
    beacons: Box<[(u32, u64)]>,
}

impl MapRuntimeCatalog {
    pub(super) fn compile(
        structural: &GoldAndGearsStructuralCatalog,
        content: &GoldAndGearsContentCatalog,
    ) -> Result<Self, GoldAndGearsEntryError> {
        let domains = structural
            .domains
            .iter()
            .map(|domain| (domain.stable_key.clone(), domain.id.0))
            .collect::<Vec<_>>()
            .into_boxed_slice();
        let beacons = structural
            .beacons
            .iter()
            .map(|beacon| (beacon.stable_key.clone(), beacon.id.0))
            .collect::<Vec<_>>()
            .into_boxed_slice();
        let mut boards = Vec::with_capacity(structural.chessboards.len());
        for board in &structural.chessboards {
            let board_id =
                i32::try_from(board.id.0).map_err(|_| GoldAndGearsEntryError::InvalidMapRuntime)?;
            let nodes = structural
                .nodes
                .iter()
                .filter(|node| node.chessboard == board.id)
                .map(|node| node.id.0)
                .collect::<Vec<_>>();
            let events = content
                .map_events
                .iter()
                .filter(|event| event.chessboard_id == board_id)
                .map(runtime_event)
                .collect::<Result<Vec<_>, _>>()?;
            let rules = content
                .block_create_rules
                .iter()
                .filter(|rule| rule.chessboard_id == board_id)
                .map(|rule| runtime_rule(rule, structural, &beacons))
                .collect::<Result<Vec<_>, _>>()?;
            if nodes.is_empty()
                || rules.is_empty()
                || rules
                    .iter()
                    .enumerate()
                    .any(|(index, rule)| usize::from(rule.order) != index)
            {
                return Err(GoldAndGearsEntryError::InvalidMapRuntime);
            }
            boards.push(BoardMapDefinition {
                key: board.stable_key.clone(),
                nodes: nodes.into_boxed_slice(),
                events: events.into_boxed_slice(),
                rules: rules.into_boxed_slice(),
            });
        }
        Ok(Self {
            boards: boards.into_boxed_slice(),
            domains,
            beacons,
        })
    }

    #[cfg(test)]
    pub(super) fn denominators(&self) -> (usize, usize, usize) {
        (
            self.boards.len(),
            self.boards.iter().map(|board| board.events.len()).sum(),
            self.boards.iter().map(|board| board.rules.len()).sum(),
        )
    }

    pub(super) fn compile_creation(
        &self,
        board: &str,
        rng: &mut ActivityRngStreams,
    ) -> Result<ActivityProgramDefinition, GoldAndGearsEntryError> {
        let board = self.board(board)?;
        let operations = self.creation_operations(board, rng)?;
        program(CREATION_PROGRAM_BASE, board_id(board)?, operations)
    }

    pub(super) fn compile_event_then_creation(
        &self,
        board: &str,
        trigger: &str,
        parameter: u32,
        rng: &mut ActivityRngStreams,
    ) -> Result<ActivityProgramDefinition, GoldAndGearsEntryError> {
        let board = self.board(board)?;
        let trigger = match trigger {
            "EnterChessRogueCell" => MapEventTrigger::EnterCell,
            "EnterChessRogueRow" => MapEventTrigger::EnterRow,
            _ => return Err(GoldAndGearsEntryError::InvalidMapRuntime),
        };
        let candidates = board
            .events
            .iter()
            .filter(|event| {
                event.trigger == trigger
                    && (event.trigger_parameters.is_empty()
                        || event.trigger_parameters.contains(&parameter))
            })
            .collect::<Vec<_>>();
        let selected = choose(rng, MAP_EVENT_PURPOSE, &candidates, |event| event.weight)?
            .ok_or(GoldAndGearsEntryError::MissingMapEvent)?;
        let mut operations = event_operations(selected);
        operations.extend(self.creation_operations(board, rng)?);
        program(EVENT_CREATION_PROGRAM_BASE, board_id(board)?, operations)
    }

    pub(super) fn compile_replacement(
        &self,
        target: NodeId,
        domain: &str,
        beacon: Option<&str>,
    ) -> Result<ActivityProgramDefinition, GoldAndGearsEntryError> {
        let domain = i64::from(self.domain(domain)?);
        let beacon = i64::from(self.beacon(beacon)?);
        let operations = node_values(
            target,
            NODE_STATE_REPLACED,
            literal(domain),
            literal(beacon),
        );
        program(REPLACEMENT_PROGRAM_BASE, target.get(), operations)
    }

    pub(super) fn compile_copy(
        &self,
        source: NodeId,
        target: NodeId,
    ) -> Result<ActivityProgramDefinition, GoldAndGearsEntryError> {
        let operations = node_values(
            target,
            NODE_STATE_COPIED,
            counter(BOARD_NODE_DOMAIN_SLOT, source),
            counter(BOARD_NODE_BEACON_SLOT, source),
        );
        program(COPY_PROGRAM_BASE, target.get(), operations)
    }

    pub(super) fn compile_blank(
        &self,
        target: NodeId,
    ) -> Result<ActivityProgramDefinition, GoldAndGearsEntryError> {
        let operations = node_values(target, NODE_STATE_BLANKED, literal(0), literal(0));
        program(BLANK_PROGRAM_BASE, target.get(), operations)
    }

    fn creation_operations(
        &self,
        board: &BoardMapDefinition,
        rng: &mut ActivityRngStreams,
    ) -> Result<Vec<ActivityOperation>, GoldAndGearsEntryError> {
        let mut operations = Vec::new();
        let mut node_index = 0_usize;
        for rule in &board.rules {
            let selected_count = choose_pair(rng, CREATE_COUNT_PURPOSE, &rule.create_counts)?
                .map_or(0, |candidate| candidate.0);
            for _ in 0..selected_count {
                let node = board
                    .nodes
                    .get(node_index)
                    .copied()
                    .and_then(NodeId::new)
                    .ok_or(GoldAndGearsEntryError::MapCapacityExceeded)?;
                node_index += 1;
                let beacon = choose_pair(rng, BEACON_PURPOSE, &rule.beacons)?
                    .map_or(0, |candidate| candidate.0);
                operations.extend(node_values(
                    node,
                    NODE_STATE_CREATED,
                    literal(i64::from(rule.domain)),
                    literal(i64::from(beacon)),
                ));
            }
        }
        for node in &board.nodes[node_index..] {
            operations.extend(node_values(
                NodeId::new(*node).ok_or(GoldAndGearsEntryError::InvalidMapRuntime)?,
                NODE_STATE_BLANKED,
                literal(0),
                literal(0),
            ));
        }
        Ok(operations)
    }

    fn board(&self, key: &str) -> Result<&BoardMapDefinition, GoldAndGearsEntryError> {
        self.boards
            .iter()
            .find(|board| board.key.as_ref() == key)
            .ok_or_else(|| GoldAndGearsEntryError::UnknownChessboard(key.into()))
    }

    fn domain(&self, key: &str) -> Result<u32, GoldAndGearsEntryError> {
        self.domains
            .iter()
            .find(|(candidate, _)| candidate.as_ref() == key)
            .map(|(_, id)| *id)
            .ok_or_else(|| GoldAndGearsEntryError::UnknownDomain(key.into()))
    }

    fn beacon(&self, key: Option<&str>) -> Result<u32, GoldAndGearsEntryError> {
        key.map_or(Ok(0), |key| {
            self.beacons
                .iter()
                .find(|(candidate, _)| candidate.as_ref() == key)
                .map(|(_, id)| *id)
                .ok_or_else(|| GoldAndGearsEntryError::UnknownBeacon(key.into()))
        })
    }
}

fn runtime_event(
    event: &crate::gold_gears_content::MapEvent,
) -> Result<RuntimeMapEvent, GoldAndGearsEntryError> {
    Ok(RuntimeMapEvent {
        id: u32::try_from(event.id).map_err(|_| GoldAndGearsEntryError::InvalidMapRuntime)?,
        trigger: event.trigger,
        trigger_parameters: event.trigger_parameters.clone(),
        effect: event.effect,
        effect_parameters: event.effect_parameters.clone(),
        secondary_effect_parameters: event.secondary_effect_parameters.clone(),
        weight: event.weight,
    })
}

fn runtime_rule(
    rule: &BlockCreateRule,
    structural: &GoldAndGearsStructuralCatalog,
    beacons: &[(Box<str>, u32)],
) -> Result<RuntimeBlockRule, GoldAndGearsEntryError> {
    let domain = structural
        .domains
        .iter()
        .find(|domain| i32::try_from(domain.id.0) == Ok(rule.domain_id))
        .map(|domain| domain.id.0)
        .ok_or(GoldAndGearsEntryError::InvalidMapRuntime)?;
    let create_counts = rule
        .create_counts
        .iter()
        .map(|candidate| (candidate.count, candidate.weight))
        .collect::<Vec<_>>()
        .into_boxed_slice();
    let resolved_beacons = rule
        .beacons
        .iter()
        .map(|candidate| {
            let id = candidate.beacon.as_ref().map_or(Ok(0), |key| {
                beacons
                    .iter()
                    .find(|(candidate, _)| candidate.as_ref() == key.as_str())
                    .map(|(_, id)| *id)
                    .ok_or(GoldAndGearsEntryError::InvalidMapRuntime)
            })?;
            Ok((id, candidate.weight))
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_boxed_slice();
    Ok(RuntimeBlockRule {
        order: rule.order,
        domain,
        create_counts,
        beacons: resolved_beacons,
    })
}

fn event_operations(event: &RuntimeMapEvent) -> Vec<ActivityOperation> {
    let effect = match event.effect {
        MapEventEffect::AddActionPoint => 1,
        MapEventEffect::GrantCurio => 2,
        MapEventEffect::GenerateMark => 3,
        MapEventEffect::RandomReplace => 4,
        MapEventEffect::Replace => 5,
        MapEventEffect::Shuffle => 6,
    };
    let mut operations = vec![
        set_counter(
            PLANE_STATE_SLOT,
            PLANE_LAST_MAP_EVENT_KEY,
            literal(i64::from(event.id)),
        ),
        set_counter(PLANE_STATE_SLOT, PLANE_LAST_MAP_EFFECT_KEY, literal(effect)),
        set_counter(
            PLANE_STATE_SLOT,
            PLANE_LAST_MAP_PARAMETER_KEY,
            literal(i64::from(
                event.effect_parameters.first().copied().unwrap_or(0),
            )),
        ),
    ];
    if event.effect == MapEventEffect::AddActionPoint {
        operations.push(ActivityOperation::AddCounter {
            slot: slot(PLANE_STATE_SLOT),
            key: PLANE_ACTION_POINTS_KEY,
            delta: literal(i64::from(
                event.effect_parameters.first().copied().unwrap_or(0),
            )),
        });
    }
    let _ = &event.secondary_effect_parameters;
    operations
}

fn node_values(
    node: NodeId,
    state: i64,
    domain: ActivityExpression,
    beacon: ActivityExpression,
) -> Vec<ActivityOperation> {
    vec![
        set_counter(BOARD_NODE_STATE_SLOT, u64::from(node.get()), literal(state)),
        set_counter(BOARD_NODE_DOMAIN_SLOT, u64::from(node.get()), domain),
        set_counter(BOARD_NODE_BEACON_SLOT, u64::from(node.get()), beacon),
    ]
}

fn set_counter(slot_id: u32, key: u64, desired: ActivityExpression) -> ActivityOperation {
    ActivityOperation::AddCounter {
        slot: slot(slot_id),
        key,
        delta: ActivityExpression::Subtract(
            Box::new(desired),
            Box::new(ActivityExpression::CounterValue {
                slot: slot(slot_id),
                key,
            }),
        ),
    }
}

fn counter(slot_id: u32, node: NodeId) -> ActivityExpression {
    ActivityExpression::CounterValue {
        slot: slot(slot_id),
        key: u64::from(node.get()),
    }
}

fn literal(value: i64) -> ActivityExpression {
    ActivityExpression::Literal(ActivityValue::BoundedInteger(value))
}

fn slot(raw: u32) -> ActivitySlotId {
    ActivitySlotId::new(raw).expect("static Gold and Gears slot is non-zero")
}

fn program(
    base: u32,
    key: u32,
    operations: Vec<ActivityOperation>,
) -> Result<ActivityProgramDefinition, GoldAndGearsEntryError> {
    let id = base
        .checked_add(key)
        .and_then(ActivityProgramId::new)
        .ok_or(GoldAndGearsEntryError::InvalidMapRuntime)?;
    ActivityProgramDefinition::new(id, operations)
        .map_err(|_| GoldAndGearsEntryError::InvalidMapRuntime)
}

fn board_id(board: &BoardMapDefinition) -> Result<u32, GoldAndGearsEntryError> {
    board
        .key
        .rsplit('.')
        .next()
        .and_then(|value| value.parse::<u32>().ok())
        .ok_or(GoldAndGearsEntryError::InvalidMapRuntime)
}

fn choose<'a, T>(
    rng: &mut ActivityRngStreams,
    purpose: u16,
    candidates: &'a [&T],
    weight: impl Fn(&T) -> u64,
) -> Result<Option<&'a T>, GoldAndGearsEntryError> {
    let weights = candidates
        .iter()
        .map(|candidate| weight(candidate))
        .collect::<Vec<_>>();
    rng.choose_weighted(ActivityRngLabel::Graph, purpose, &weights)
        .map_err(|_| GoldAndGearsEntryError::InvalidMapRuntime)
        .map(|selection| selection.map(|(index, _)| candidates[index as usize]))
}

fn choose_pair<'a, T>(
    rng: &mut ActivityRngStreams,
    purpose: u16,
    candidates: &'a [(T, u64)],
) -> Result<Option<&'a (T, u64)>, GoldAndGearsEntryError> {
    let weights = candidates
        .iter()
        .map(|candidate| candidate.1)
        .collect::<Vec<_>>();
    rng.choose_weighted(ActivityRngLabel::Graph, purpose, &weights)
        .map_err(|_| GoldAndGearsEntryError::InvalidMapRuntime)
        .map(|selection| selection.map(|(index, _)| &candidates[index as usize]))
}
