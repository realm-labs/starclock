use serde_json::Value;

use crate::error::{UniverseCatalogLoadError, UniverseCatalogLoadErrorKind};

use super::SwarmDisasterContentCatalog;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SwarmMapTrigger {
    Unspecified,
    EnterRow,
    EnterCell,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SwarmMapEffect {
    Unspecified,
    ReplaceBlock,
    GrantCurio,
    Shuffle,
    RandomReplace,
    GenerateMark,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SwarmMapEventInput {
    pub(crate) id: u32,
    pub(crate) board_id: u32,
    pub(crate) trigger: SwarmMapTrigger,
    pub(crate) trigger_parameters: Box<[u32]>,
    pub(crate) effect: SwarmMapEffect,
    pub(crate) effect_parameters: Box<[u32]>,
    pub(crate) secondary_parameters: Box<[u32]>,
    pub(crate) weight: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SwarmBlockRuleInput {
    pub(crate) board_id: u32,
    pub(crate) domain_id: u32,
    pub(crate) order: u16,
    pub(crate) create_counts: Box<[(u16, u64)]>,
    pub(crate) beacons: Box<[(Option<Box<str>>, u64)]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SwarmMapRuntimeInput {
    pub(crate) events: Box<[SwarmMapEventInput]>,
    pub(crate) rules: Box<[SwarmBlockRuleInput]>,
}

impl SwarmDisasterContentCatalog {
    pub(crate) fn map_runtime_input(
        &self,
    ) -> Result<SwarmMapRuntimeInput, UniverseCatalogLoadError> {
        let events = self
            .map_events
            .iter()
            .map(|event| {
                let trigger = object(&event.trigger)?;
                let operations = array(&event.operations)?;
                if operations.len() != 1 {
                    return Err(invalid("Swarm map event must contain one ordered effect"));
                }
                let operation = object_value(&operations[0])?;
                Ok(SwarmMapEventInput {
                    id: event.id.0,
                    board_id: event.chessboard_id,
                    trigger: trigger_kind(string(&trigger, "type")?)?,
                    trigger_parameters: numbers(&trigger, "parameters")?,
                    effect: effect_kind(string(operation, "type")?)?,
                    effect_parameters: numbers(operation, "parameters")?,
                    secondary_parameters: numbers(operation, "secondary_parameters")?,
                    weight: decimal(&event.weight)?,
                })
            })
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice();
        let rules = self
            .block_rules
            .iter()
            .map(|rule| {
                let counts = array(&rule.count)?
                    .iter()
                    .map(|value| {
                        let value = object_value(value)?;
                        Ok((
                            number_u16(value, "create_count")?,
                            decimal(string(value, "weight")?)?,
                        ))
                    })
                    .collect::<Result<Vec<_>, UniverseCatalogLoadError>>()?;
                let beacons = array(&rule.candidates)?
                    .iter()
                    .map(|value| {
                        let value = object_value(value)?;
                        let key = string(value, "beacon_id")?;
                        Ok((
                            (!key.is_empty()).then(|| Box::<str>::from(key)),
                            decimal(string(value, "weight")?)?,
                        ))
                    })
                    .collect::<Result<Vec<_>, UniverseCatalogLoadError>>()?;
                Ok(SwarmBlockRuleInput {
                    board_id: rule.chessboard_id,
                    domain_id: rule.domain_id,
                    order: rule.order,
                    create_counts: counts.into_boxed_slice(),
                    beacons: beacons.into_boxed_slice(),
                })
            })
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice();
        Ok(SwarmMapRuntimeInput { events, rules })
    }
}

fn trigger_kind(value: &str) -> Result<SwarmMapTrigger, UniverseCatalogLoadError> {
    match value {
        "Unspecified" => Ok(SwarmMapTrigger::Unspecified),
        "EnterChessRogueRow" => Ok(SwarmMapTrigger::EnterRow),
        "EnterChessRogueCell" => Ok(SwarmMapTrigger::EnterCell),
        _ => Err(invalid("unknown Swarm map trigger")),
    }
}

fn effect_kind(value: &str) -> Result<SwarmMapEffect, UniverseCatalogLoadError> {
    match value {
        "Unspecified" => Ok(SwarmMapEffect::Unspecified),
        "ReplaceBlock" => Ok(SwarmMapEffect::ReplaceBlock),
        "GetRogueMiracle" => Ok(SwarmMapEffect::GrantCurio),
        "TriggerAreaShuffle" => Ok(SwarmMapEffect::Shuffle),
        "RandomReplaceBlock" => Ok(SwarmMapEffect::RandomReplace),
        "TriggerMark" => Ok(SwarmMapEffect::GenerateMark),
        _ => Err(invalid("unknown Swarm map effect")),
    }
}

fn object(value: &str) -> Result<serde_json::Map<String, Value>, UniverseCatalogLoadError> {
    let value = serde_json::from_str::<Value>(value)
        .map_err(|_| invalid("invalid embedded Swarm map object"))?;
    value
        .as_object()
        .cloned()
        .ok_or_else(|| invalid("embedded Swarm map value is not an object"))
}

fn array(value: &str) -> Result<Vec<Value>, UniverseCatalogLoadError> {
    serde_json::from_str::<Vec<Value>>(value)
        .map_err(|_| invalid("invalid embedded Swarm map array"))
}

fn object_value(
    value: &Value,
) -> Result<&serde_json::Map<String, Value>, UniverseCatalogLoadError> {
    value
        .as_object()
        .ok_or_else(|| invalid("embedded Swarm map value is not an object"))
}

fn string<'a>(
    value: &'a serde_json::Map<String, Value>,
    key: &str,
) -> Result<&'a str, UniverseCatalogLoadError> {
    value
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| invalid("embedded Swarm map string is missing"))
}

fn numbers(
    value: &serde_json::Map<String, Value>,
    key: &str,
) -> Result<Box<[u32]>, UniverseCatalogLoadError> {
    value
        .get(key)
        .and_then(Value::as_array)
        .ok_or_else(|| invalid("embedded Swarm parameter list is missing"))?
        .iter()
        .map(|value| {
            value
                .as_str()
                .ok_or_else(|| invalid("embedded Swarm parameter is not text"))?
                .parse::<u32>()
                .map_err(|_| invalid("embedded Swarm parameter is invalid"))
        })
        .collect::<Result<Vec<_>, _>>()
        .map(Vec::into_boxed_slice)
}

fn number_u16(
    value: &serde_json::Map<String, Value>,
    key: &str,
) -> Result<u16, UniverseCatalogLoadError> {
    value
        .get(key)
        .and_then(Value::as_u64)
        .and_then(|value| u16::try_from(value).ok())
        .ok_or_else(|| invalid("embedded Swarm count is invalid"))
}

fn decimal(value: &str) -> Result<u64, UniverseCatalogLoadError> {
    value
        .parse::<u64>()
        .ok()
        .filter(|value| *value > 0)
        .ok_or_else(|| invalid("embedded Swarm weight is invalid"))
}

fn invalid(message: &'static str) -> UniverseCatalogLoadError {
    UniverseCatalogLoadError::new(UniverseCatalogLoadErrorKind::InvalidDefinition, message)
}
