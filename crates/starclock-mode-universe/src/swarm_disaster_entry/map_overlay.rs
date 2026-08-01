//! Typed Swarm board overlays and deterministic graph-program compilation.

use starclock_activity::{
    ActivityExpression, ActivityOperation, ActivityProgramDefinition, ActivityProgramId,
    ActivityRngLabel, ActivityRngStreams, ActivitySlotId, ActivityTransactionState, ActivityValue,
    NodeId,
};

use crate::{
    error::{UniverseCatalogLoadError, UniverseCatalogLoadErrorKind},
    swarm_disaster_content::map_access::{
        SwarmMapEffect, SwarmMapEventInput, SwarmMapRuntimeInput, SwarmMapTrigger,
    },
    swarm_disaster_structural::map_access::SwarmDisasterMapStructuralInput,
};

use super::{
    face_effect::{FaceSelector, MERCY_TARGET_BASE},
    state::{DEFERRED, NODE_BEACON, NODE_DOMAIN, NODE_STATE, PLANE},
};

pub(super) const MAP_EVENT_PURPOSE: u16 = 0x5301;
pub(super) const CREATE_COUNT_PURPOSE: u16 = 0x5302;
pub(super) const BEACON_PURPOSE: u16 = 0x5303;

const CREATION_PROGRAM_BASE: u32 = 0x5300_0000;
const EVENT_CREATION_PROGRAM_BASE: u32 = 0x5310_0000;
const REPLACEMENT_PROGRAM_BASE: u32 = 0x5320_0000;
const COPY_PROGRAM_BASE: u32 = 0x5330_0000;
const BLANK_PROGRAM_BASE: u32 = 0x5340_0000;

const NODE_STATE_CREATED: i64 = 1;
const NODE_STATE_REPLACED: i64 = 2;
const NODE_STATE_COPIED: i64 = 3;
pub(super) const NODE_STATE_BLANKED: i64 = 4;

const PLANE_LAST_MAP_EVENT_KEY: u64 = 1;
const PLANE_LAST_MAP_EFFECT_KEY: u64 = 2;
const PLANE_LAST_MAP_PARAMETER_KEY: u64 = 3;

#[derive(Debug)]
pub(super) struct MapRuntimeCatalog {
    boards: Box<[BoardMapDefinition]>,
    domains: Box<[(Box<str>, u32)]>,
    beacons: Box<[(Box<str>, u32)]>,
    _room_binding_count: usize,
}

#[derive(Debug)]
struct BoardMapDefinition {
    id: u32,
    key: Box<str>,
    events: Box<[SwarmMapEventInput]>,
    rules: Box<[RuntimeBlockRule]>,
}

#[derive(Debug)]
struct RuntimeBlockRule {
    domain: u32,
    create_counts: Box<[(u16, u64)]>,
    beacons: Box<[(u32, u64)]>,
}

impl MapRuntimeCatalog {
    pub(super) fn compile(
        structural: SwarmDisasterMapStructuralInput,
        content: SwarmMapRuntimeInput,
    ) -> Result<Self, UniverseCatalogLoadError> {
        let events = content.events;
        let rules = content.rules;
        let mut boards = Vec::with_capacity(structural.boards.len());
        for board in &structural.boards {
            let board_events = events
                .iter()
                .filter(|event| event.board_id == board.id)
                .cloned()
                .collect::<Vec<_>>();
            let board_rules = rules
                .iter()
                .filter(|rule| rule.board_id == board.id)
                .enumerate()
                .map(|(index, rule)| {
                    if usize::from(rule.order) != index {
                        return Err(invalid("Swarm block-rule order is not contiguous"));
                    }
                    let beacons = rule
                        .beacons
                        .iter()
                        .map(|(key, weight)| {
                            let id = key.as_deref().map_or(Ok(0), |key| {
                                structural
                                    .beacons
                                    .iter()
                                    .find(|(candidate, _)| candidate.as_ref() == key)
                                    .map(|(_, id)| *id)
                                    .ok_or_else(|| invalid("unknown Swarm beacon"))
                            })?;
                            Ok((id, *weight))
                        })
                        .collect::<Result<Vec<_>, UniverseCatalogLoadError>>()?;
                    Ok(RuntimeBlockRule {
                        domain: rule.domain_id,
                        create_counts: rule.create_counts.clone(),
                        beacons: beacons.into_boxed_slice(),
                    })
                })
                .collect::<Result<Vec<_>, UniverseCatalogLoadError>>()?;
            if board.nodes.is_empty() || board_rules.is_empty() {
                return Err(invalid("Swarm map board has no nodes or creation rules"));
            }
            boards.push(BoardMapDefinition {
                id: board.id,
                key: board.key.clone(),
                events: board_events.into_boxed_slice(),
                rules: board_rules.into_boxed_slice(),
            });
        }
        if boards.len() != 101
            || events.len() != 349
            || rules.len() != 1_212
            || structural.domains.len() != 12
            || structural.beacons.len() != 4
            || structural.room_bindings.len() != 861
            || structural
                .room_bindings
                .iter()
                .any(|binding| binding.key.is_empty() || binding.sections.is_empty())
        {
            return Err(invalid("Swarm map runtime denominator drift"));
        }
        Ok(Self {
            boards: boards.into_boxed_slice(),
            domains: structural.domains,
            beacons: structural.beacons,
            _room_binding_count: structural.room_bindings.len(),
        })
    }

    #[cfg(test)]
    pub(super) fn denominators(&self) -> (usize, usize, usize, usize, usize, usize) {
        (
            self.boards.len(),
            self.boards.iter().map(|board| board.events.len()).sum(),
            self.boards.iter().map(|board| board.rules.len()).sum(),
            self.domains.len(),
            self.beacons.len(),
            self._room_binding_count,
        )
    }

    pub(super) fn compile_creation(
        &self,
        board: &str,
        nodes: &[NodeId],
        terminal: NodeId,
        terminal_domain: &str,
        rng: &mut ActivityRngStreams,
    ) -> Result<ActivityProgramDefinition, UniverseCatalogLoadError> {
        let board = self.board(board)?;
        let operations = self.creation_operations(board, nodes, terminal, terminal_domain, rng)?;
        program(CREATION_PROGRAM_BASE, board.id, operations)
    }

    // This private boundary keeps the event selector and immutable plane inputs explicit.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn compile_event_then_creation(
        &self,
        board: &str,
        trigger: &str,
        parameter: u32,
        nodes: &[NodeId],
        terminal: NodeId,
        terminal_domain: &str,
        rng: &mut ActivityRngStreams,
    ) -> Result<ActivityProgramDefinition, UniverseCatalogLoadError> {
        let board = self.board(board)?;
        let trigger = match trigger {
            "EnterChessRogueRow" => SwarmMapTrigger::EnterRow,
            "EnterChessRogueCell" => SwarmMapTrigger::EnterCell,
            _ => return Err(invalid("unknown Swarm map-event trigger")),
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
            .ok_or_else(|| invalid("no matching Swarm map event"))?;
        let mut operations = event_operations(selected);
        operations.extend(self.creation_operations(
            board,
            nodes,
            terminal,
            terminal_domain,
            rng,
        )?);
        program(EVENT_CREATION_PROGRAM_BASE, board.id, operations)
    }

    pub(super) fn compile_replacement(
        &self,
        target: NodeId,
        domain: &str,
        beacon: Option<&str>,
    ) -> Result<ActivityProgramDefinition, UniverseCatalogLoadError> {
        let beacon = beacon.map_or_else(
            || Ok(counter(NODE_BEACON, target)),
            |key| {
                self.beacon(Some(key))
                    .map(|value| literal(i64::from(value)))
            },
        )?;
        let operations = node_values(
            target,
            NODE_STATE_REPLACED,
            literal(i64::from(self.domain(domain)?)),
            beacon,
        );
        program(REPLACEMENT_PROGRAM_BASE, target.get(), operations)
    }

    pub(super) fn compile_copy(
        &self,
        source: NodeId,
        target: NodeId,
    ) -> Result<ActivityProgramDefinition, UniverseCatalogLoadError> {
        program(
            COPY_PROGRAM_BASE,
            target.get(),
            node_values(
                target,
                NODE_STATE_COPIED,
                counter(NODE_DOMAIN, source),
                counter(NODE_BEACON, target),
            ),
        )
    }

    pub(super) fn compile_blank(
        &self,
        target: NodeId,
    ) -> Result<ActivityProgramDefinition, UniverseCatalogLoadError> {
        program(
            BLANK_PROGRAM_BASE,
            target.get(),
            node_values(
                target,
                NODE_STATE_BLANKED,
                literal(0),
                counter(NODE_BEACON, target),
            ),
        )
    }

    fn creation_operations(
        &self,
        board: &BoardMapDefinition,
        nodes: &[NodeId],
        terminal: NodeId,
        terminal_domain: &str,
        rng: &mut ActivityRngStreams,
    ) -> Result<Vec<ActivityOperation>, UniverseCatalogLoadError> {
        let terminal_domain_id = self.domain(terminal_domain)?;
        let mut available = nodes
            .iter()
            .copied()
            .filter(|node| *node != terminal)
            .collect::<Vec<_>>();
        available.sort_unstable();
        let mut operations = Vec::new();
        let mut node_index = 0_usize;
        for rule in board
            .rules
            .iter()
            .filter(|rule| rule.domain != terminal_domain_id)
        {
            let count = choose_pair(rng, CREATE_COUNT_PURPOSE, &rule.create_counts)?
                .map_or(0, |candidate| candidate.0);
            for _ in 0..count {
                let Some(node) = available.get(node_index).copied() else {
                    // An authored count beyond this bounded graph has no legal target.
                    break;
                };
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
        for node in &available[node_index..] {
            operations.extend(node_values(
                *node,
                NODE_STATE_CREATED,
                literal(i64::from(self.domain("swarm-disaster.domain.empty")?)),
                literal(0),
            ));
        }
        operations.extend(node_values(
            terminal,
            NODE_STATE_CREATED,
            literal(i64::from(terminal_domain_id)),
            literal(0),
        ));
        Ok(operations)
    }

    fn board(&self, key: &str) -> Result<&BoardMapDefinition, UniverseCatalogLoadError> {
        self.boards
            .iter()
            .find(|board| board.key.as_ref() == key)
            .ok_or_else(|| invalid("unknown Swarm chessboard"))
    }

    fn domain(&self, key: &str) -> Result<u32, UniverseCatalogLoadError> {
        self.domains
            .iter()
            .find(|(candidate, _)| candidate.as_ref() == key)
            .map(|(_, id)| *id)
            .ok_or_else(|| invalid("unknown Swarm domain"))
    }

    fn beacon(&self, key: Option<&str>) -> Result<u32, UniverseCatalogLoadError> {
        key.map_or(Ok(0), |key| {
            self.beacons
                .iter()
                .find(|(candidate, _)| candidate.as_ref() == key)
                .map(|(_, id)| *id)
                .ok_or_else(|| invalid("unknown Swarm beacon"))
        })
    }

    pub(super) fn dice_face_candidates(
        &self,
        state: &ActivityTransactionState,
        selector: FaceSelector,
    ) -> Result<Box<[NodeId]>, UniverseCatalogLoadError> {
        let combat = self.domain("swarm-disaster.domain.monsternormal")?;
        let elite = self.domain("swarm-disaster.domain.monsterelite")?;
        let occurrence = self.domain("swarm-disaster.domain.event")?;
        let boss = self.domain("swarm-disaster.domain.monsterboss")?;
        let swarm = self.domain("swarm-disaster.domain.monsterswarm")?;
        let swarm_boss = self.domain("swarm-disaster.domain.monsterswarmboss")?;
        let swarm_occurrence = self.domain("swarm-disaster.domain.swarmevent")?;
        let domains = map_values(state, NODE_DOMAIN)?;
        let node_states = map_values(state, NODE_STATE)?;
        let deferred = map_values(state, DEFERRED)?;
        let mut candidates = Vec::new();
        for (raw, domain) in domains {
            let active = node_states
                .binary_search_by_key(raw, |(key, _)| *key)
                .ok()
                .is_some_and(|index| node_states[index].1 != NODE_STATE_BLANKED);
            if !active {
                continue;
            }
            let domain = u32::try_from(*domain)
                .map_err(|_| invalid("invalid Swarm dice-face candidate domain"))?;
            let node = u32::try_from(*raw)
                .ok()
                .and_then(NodeId::new)
                .ok_or_else(|| invalid("invalid Swarm dice-face candidate node"))?;
            let has_mercy = deferred
                .binary_search_by_key(&(MERCY_TARGET_BASE + *raw), |(key, _)| *key)
                .ok()
                .is_some_and(|index| deferred[index].1 != 0);
            let selected = match selector {
                FaceSelector::Any => true,
                FaceSelector::NonBoss => domain != boss && domain != swarm_boss,
                FaceSelector::Combat => domain == combat,
                FaceSelector::Elite => domain == elite,
                FaceSelector::Occurrence => domain == occurrence,
                FaceSelector::CombatSwarmElite => [combat, swarm, elite].contains(&domain),
                FaceSelector::CombatSwarm => domain == swarm,
                FaceSelector::Swarm => [swarm, swarm_occurrence].contains(&domain),
                FaceSelector::Boss => [boss, swarm_boss].contains(&domain),
                FaceSelector::WithoutMercy => !has_mercy,
            };
            if selected {
                candidates.push((domain, node));
            }
        }
        candidates.sort_unstable_by_key(|(domain, node)| (*domain, *node));
        if candidates.windows(2).any(|pair| pair[0].1 == pair[1].1) {
            return Err(invalid("duplicate Swarm dice-face target candidate"));
        }
        Ok(candidates
            .into_iter()
            .map(|(_, node)| node)
            .collect::<Vec<_>>()
            .into_boxed_slice())
    }
}

fn map_values(
    state: &ActivityTransactionState,
    slot_id: u32,
) -> Result<&[(u64, i64)], UniverseCatalogLoadError> {
    match state.slot(slot(slot_id)) {
        Some(ActivityValue::BoundedCounterMap(values)) => Ok(values),
        _ => Err(invalid("invalid Swarm map overlay slot")),
    }
}

fn event_operations(event: &SwarmMapEventInput) -> Vec<ActivityOperation> {
    let effect = match event.effect {
        SwarmMapEffect::Unspecified => 0,
        SwarmMapEffect::ReplaceBlock => 1,
        SwarmMapEffect::GrantCurio => 2,
        SwarmMapEffect::Shuffle => 3,
        SwarmMapEffect::RandomReplace => 4,
        SwarmMapEffect::GenerateMark => 5,
    };
    let _ = &event.secondary_parameters;
    vec![
        set_counter(
            PLANE,
            PLANE_LAST_MAP_EVENT_KEY,
            literal(i64::from(event.id)),
        ),
        set_counter(PLANE, PLANE_LAST_MAP_EFFECT_KEY, literal(effect)),
        set_counter(
            PLANE,
            PLANE_LAST_MAP_PARAMETER_KEY,
            literal(i64::from(
                event.effect_parameters.first().copied().unwrap_or(0),
            )),
        ),
    ]
}

fn node_values(
    node: NodeId,
    state: i64,
    domain: ActivityExpression,
    beacon: ActivityExpression,
) -> Vec<ActivityOperation> {
    vec![
        set_counter(NODE_STATE, u64::from(node.get()), literal(state)),
        set_counter(NODE_DOMAIN, u64::from(node.get()), domain),
        set_counter(NODE_BEACON, u64::from(node.get()), beacon),
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
    ActivitySlotId::new(raw).expect("static Swarm slot is non-zero")
}

fn program(
    base: u32,
    key: u32,
    operations: Vec<ActivityOperation>,
) -> Result<ActivityProgramDefinition, UniverseCatalogLoadError> {
    let id = base
        .checked_add(key)
        .and_then(ActivityProgramId::new)
        .ok_or_else(|| invalid("invalid Swarm map program ID"))?;
    ActivityProgramDefinition::new(id, operations).map_err(|_| invalid("invalid Swarm map program"))
}

fn choose<'a, T>(
    rng: &mut ActivityRngStreams,
    purpose: u16,
    candidates: &'a [&T],
    weight: impl Fn(&T) -> u64,
) -> Result<Option<&'a T>, UniverseCatalogLoadError> {
    let weights = candidates
        .iter()
        .map(|candidate| weight(candidate))
        .collect::<Vec<_>>();
    rng.choose_weighted(ActivityRngLabel::Graph, purpose, &weights)
        .map_err(|_| invalid("invalid Swarm weighted map selection"))
        .map(|selection| selection.map(|(index, _)| candidates[index as usize]))
}

fn choose_pair<'a, T>(
    rng: &mut ActivityRngStreams,
    purpose: u16,
    candidates: &'a [(T, u64)],
) -> Result<Option<&'a (T, u64)>, UniverseCatalogLoadError> {
    let weights = candidates
        .iter()
        .map(|candidate| candidate.1)
        .collect::<Vec<_>>();
    rng.choose_weighted(ActivityRngLabel::Graph, purpose, &weights)
        .map_err(|_| invalid("invalid Swarm weighted map selection"))
        .map(|selection| selection.map(|(index, _)| &candidates[index as usize]))
}

pub(super) fn terminal_domain(
    plane_ordinal: usize,
) -> Result<&'static str, UniverseCatalogLoadError> {
    match plane_ordinal {
        0 | 1 => Ok("swarm-disaster.domain.monsterboss"),
        2 => Ok("swarm-disaster.domain.monsterswarmboss"),
        _ => Err(invalid_plane()),
    }
}

pub(super) fn node_is_blanked(state: &ActivityTransactionState, node: NodeId) -> bool {
    match state.slot(slot(NODE_STATE)) {
        Some(ActivityValue::BoundedCounterMap(values)) => values
            .binary_search_by_key(&u64::from(node.get()), |(key, _)| *key)
            .ok()
            .is_some_and(|index| values[index].1 == NODE_STATE_BLANKED),
        _ => false,
    }
}

pub(super) fn invalid_plane() -> UniverseCatalogLoadError {
    invalid("invalid Swarm plane ordinal")
}

pub(super) fn invalid_node() -> UniverseCatalogLoadError {
    invalid("Swarm topology node is not in the immutable graph")
}

fn invalid(message: &'static str) -> UniverseCatalogLoadError {
    UniverseCatalogLoadError::new(UniverseCatalogLoadErrorKind::InvalidDefinition, message)
}
