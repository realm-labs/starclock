//! Strict lowering for run services, shop policy and the Ability Tree DAG.

use std::collections::{BTreeMap, BTreeSet};

use crate::error::UniverseCatalogLoadError;
use crate::generated::{
    SoraConfig, universe_ability_effect_class::UniverseAbilityEffectClass,
    universe_ability_operation::UniverseAbilityOperation,
    universe_ability_value_unit::UniverseAbilityValueUnit,
    universe_service_kind::UniverseServiceKind,
};
use crate::id::{AbilityTreeNodeId, ServiceId};
use crate::lowering::{checked_key, checked_source, invalid, localized, reference};
use crate::path_lowering::{parameter_groups, parse_decimal, validate_rule};
use crate::progression::{
    AbilityEffectClass, AbilityOperation, AbilityTreeCost, AbilityTreeEffect,
    AbilityTreeNodeDefinition, AbilityValueUnit, ServiceDefinition, ServiceKind, ServiceParameter,
};

pub(crate) struct ProgressionDefinitions {
    pub(crate) services: Box<[ServiceDefinition]>,
    pub(crate) ability_tree_nodes: Box<[AbilityTreeNodeDefinition]>,
}

pub(crate) fn lower(
    config: &SoraConfig,
) -> Result<ProgressionDefinitions, UniverseCatalogLoadError> {
    let services = lower_services(config)?;
    let ability_tree_nodes = lower_ability_tree(config)?;
    Ok(ProgressionDefinitions {
        services,
        ability_tree_nodes,
    })
}

fn lower_services(
    config: &SoraConfig,
) -> Result<Box<[ServiceDefinition]>, UniverseCatalogLoadError> {
    let parameters = service_parameters(config)?;
    let currencies = config
        .universe_service()
        .ordered_rows()
        .filter(|row| matches!(row.kind, UniverseServiceKind::Currency))
        .map(|row| row.stable_key.as_str())
        .collect::<BTreeSet<_>>();
    let mut definitions = Vec::with_capacity(config.universe_service().len());
    for row in config.universe_service().ordered_rows() {
        validate_rule(config, &row.rule_stable_key)?;
        let kind = service_kind(row.kind);
        let currency_key =
            optional_key(row.currency_stable_key.as_deref(), "Service currency key")?;
        if currency_key
            .as_deref()
            .is_some_and(|key| !currencies.contains(key))
        {
            return Err(reference(
                "Service currency does not resolve to a Currency service",
            ));
        }
        if kind == ServiceKind::Currency && currency_key.as_deref() != Some(row.stable_key.as_str())
        {
            return Err(reference(
                "Currency service must self-identify its currency key",
            ));
        }
        definitions.push(ServiceDefinition::new(
            service_id(row.id, "Service")?,
            checked_key(&row.stable_key, "Service stable key")?,
            kind,
            currency_key,
            optional_key(
                row.price_formula_stable_key.as_deref(),
                "Service price formula",
            )?,
            optional_key(row.offer_pool_stable_key.as_deref(), "Service offer pool")?,
            &row.rule_stable_key,
            localized(
                &row.name_en,
                &row.name_zh_cn,
                &row.summary_en,
                &row.summary_zh_cn,
                "Service",
            )?,
            parameters
                .get(&row.id)
                .cloned()
                .unwrap_or_default()
                .into_boxed_slice(),
        ));
    }
    definitions.sort_by_key(ServiceDefinition::id);
    if definitions.len() != 94 || config.universe_service_parameter().len() != 12 {
        return Err(reference("Service/parameter denominator differs"));
    }
    Ok(definitions.into_boxed_slice())
}

fn service_parameters(
    config: &SoraConfig,
) -> Result<BTreeMap<i32, Vec<ServiceParameter>>, UniverseCatalogLoadError> {
    let mut grouped: BTreeMap<i32, Vec<(u32, ServiceParameter)>> = BTreeMap::new();
    let mut unique = BTreeSet::new();
    for row in config.universe_service_parameter().iter() {
        if config.universe_service().get(&row.service_id).is_none() {
            return Err(reference("Service parameter parent is unresolved"));
        }
        let sequence = positive_sequence(row.sequence, "Service parameter")?;
        checked_key(&row.key, "Service parameter key")?;
        checked_source(&row.value, "Service parameter value")?;
        if !unique.insert((row.service_id, row.key.as_str())) {
            return Err(invalid("Service parameter key is duplicated"));
        }
        grouped
            .entry(row.service_id)
            .or_default()
            .push((sequence, ServiceParameter::new(&row.key, &row.value)));
    }
    ordered_groups(grouped, "Service parameter")
}

fn lower_ability_tree(
    config: &SoraConfig,
) -> Result<Box<[AbilityTreeNodeDefinition]>, UniverseCatalogLoadError> {
    let prerequisites = ability_edges(config)?;
    validate_acyclic(&prerequisites)?;
    let costs = ability_costs(config)?;
    let effects = ability_effects(config)?;
    let parameters = parameter_groups(
        config
            .universe_ability_tree_parameter()
            .iter()
            .map(|row| (row.node_id, row.sequence, row.value_decimal.as_str())),
        "Ability Tree parameter",
    )?;
    let mut definitions = Vec::with_capacity(config.universe_ability_tree_node().len());
    for row in config.universe_ability_tree_node().ordered_rows() {
        validate_rule(config, &row.rule_stable_key)?;
        definitions.push(AbilityTreeNodeDefinition::new(
            ability_id(row.id, "Ability Tree node")?,
            checked_key(&row.stable_key, "Ability Tree stable key")?,
            row.important,
            match row.effect_class {
                UniverseAbilityEffectClass::Run => AbilityEffectClass::Run,
                UniverseAbilityEffectClass::Battle => AbilityEffectClass::Battle,
                UniverseAbilityEffectClass::RunAndBattle => AbilityEffectClass::RunAndBattle,
            },
            display(&row.effect_tag_en, "Ability Tree English effect tag")?,
            display(&row.effect_tag_zh_cn, "Ability Tree Chinese effect tag")?,
            key_list(
                row.external_unlock_ids.as_deref(),
                "Ability Tree external unlock",
            )?,
            &row.rule_stable_key,
            localized(
                &row.name_en,
                &row.name_zh_cn,
                &row.summary_en,
                &row.summary_zh_cn,
                "Ability Tree node",
            )?,
            prerequisites
                .get(&row.id)
                .cloned()
                .unwrap_or_default()
                .into_boxed_slice(),
            costs
                .get(&row.id)
                .cloned()
                .unwrap_or_default()
                .into_boxed_slice(),
            effects
                .get(&row.id)
                .cloned()
                .unwrap_or_default()
                .into_boxed_slice(),
            parameters
                .get(&row.id)
                .cloned()
                .unwrap_or_default()
                .into_boxed_slice(),
        ));
    }
    definitions.sort_by_key(AbilityTreeNodeDefinition::id);
    let valid = definitions.len() == 42
        && config.universe_ability_tree_edge().len() == 55
        && config.universe_ability_tree_cost().len() == 42
        && config.universe_ability_tree_effect().len() == 50
        && config.universe_ability_tree_parameter().len() == 43
        && definitions
            .iter()
            .all(|node| !node.costs().is_empty() && !node.effects().is_empty());
    if !valid {
        return Err(reference(
            "Ability Tree denominator or required child differs",
        ));
    }
    Ok(definitions.into_boxed_slice())
}

fn ability_edges(
    config: &SoraConfig,
) -> Result<BTreeMap<i32, Vec<AbilityTreeNodeId>>, UniverseCatalogLoadError> {
    let mut grouped: BTreeMap<i32, Vec<(u32, AbilityTreeNodeId)>> = BTreeMap::new();
    let mut unique = BTreeSet::new();
    for row in config.universe_ability_tree_edge().iter() {
        if config
            .universe_ability_tree_node()
            .get(&row.node_id)
            .is_none()
            || config
                .universe_ability_tree_node()
                .get(&row.prerequisite_node_id)
                .is_none()
        {
            return Err(reference("Ability Tree edge has an unresolved endpoint"));
        }
        if row.node_id == row.prerequisite_node_id
            || !unique.insert((row.node_id, row.prerequisite_node_id))
        {
            return Err(invalid(
                "Ability Tree edge is self-referential or duplicated",
            ));
        }
        grouped.entry(row.node_id).or_default().push((
            positive_sequence(row.sequence, "Ability Tree edge")?,
            ability_id(row.prerequisite_node_id, "Ability Tree prerequisite")?,
        ));
    }
    ordered_groups(grouped, "Ability Tree edge")
}

fn ability_costs(
    config: &SoraConfig,
) -> Result<BTreeMap<i32, Vec<AbilityTreeCost>>, UniverseCatalogLoadError> {
    let mut grouped: BTreeMap<i32, Vec<(u32, AbilityTreeCost)>> = BTreeMap::new();
    for row in config.universe_ability_tree_cost().iter() {
        require_node(config, row.node_id, "Ability Tree cost")?;
        let amount = parse_decimal(&row.amount_decimal)?;
        if amount.coefficient() <= 0 {
            return Err(invalid("Ability Tree cost must be positive"));
        }
        grouped.entry(row.node_id).or_default().push((
            positive_sequence(row.sequence, "Ability Tree cost")?,
            AbilityTreeCost::new(
                checked_source(&row.source_item_id, "Ability Tree source item ID")?,
                amount,
            ),
        ));
    }
    ordered_groups(grouped, "Ability Tree cost")
}

fn ability_effects(
    config: &SoraConfig,
) -> Result<BTreeMap<i32, Vec<AbilityTreeEffect>>, UniverseCatalogLoadError> {
    let mut grouped: BTreeMap<i32, Vec<(u32, AbilityTreeEffect)>> = BTreeMap::new();
    for row in config.universe_ability_tree_effect().iter() {
        require_node(config, row.node_id, "Ability Tree effect")?;
        let operation = ability_operation(row.kind);
        let unit = ability_unit(row.unit);
        validate_operation_unit(operation, unit)?;
        let condition = row
            .condition
            .as_deref()
            .map(|value| checked_source(value, "Ability Tree effect condition").map(Into::into))
            .transpose()?;
        grouped.entry(row.node_id).or_default().push((
            positive_sequence(row.sequence, "Ability Tree effect")?,
            AbilityTreeEffect::new(
                operation,
                checked_key(&row.target, "Ability Tree effect target")?,
                parse_decimal(&row.value_decimal)?,
                unit,
                condition,
            ),
        ));
    }
    ordered_groups(grouped, "Ability Tree effect")
}

fn validate_acyclic(
    edges: &BTreeMap<i32, Vec<AbilityTreeNodeId>>,
) -> Result<(), UniverseCatalogLoadError> {
    fn visit(
        node: i32,
        edges: &BTreeMap<i32, Vec<AbilityTreeNodeId>>,
        visiting: &mut BTreeSet<i32>,
        visited: &mut BTreeSet<i32>,
    ) -> bool {
        if visited.contains(&node) {
            return true;
        }
        if !visiting.insert(node) {
            return false;
        }
        let ok = edges
            .get(&node)
            .into_iter()
            .flatten()
            .all(|parent| visit(parent.get() as i32, edges, visiting, visited));
        visiting.remove(&node);
        if ok {
            visited.insert(node);
        }
        ok
    }
    let mut visiting = BTreeSet::new();
    let mut visited = BTreeSet::new();
    if edges
        .keys()
        .copied()
        .all(|node| visit(node, edges, &mut visiting, &mut visited))
    {
        Ok(())
    } else {
        Err(invalid("Ability Tree prerequisite graph contains a cycle"))
    }
}

fn validate_operation_unit(
    operation: AbilityOperation,
    unit: AbilityValueUnit,
) -> Result<(), UniverseCatalogLoadError> {
    let valid = match operation {
        AbilityOperation::Unlock | AbilityOperation::Enable => unit == AbilityValueUnit::Boolean,
        AbilityOperation::AddChoice
        | AbilityOperation::AddCurrency
        | AbilityOperation::AddLimit
        | AbilityOperation::UnlockFormationSlot => unit == AbilityValueUnit::Count,
        AbilityOperation::AddResource => unit == AbilityValueUnit::Flat,
        AbilityOperation::Set | AbilityOperation::SetRatio => unit == AbilityValueUnit::Ratio,
        AbilityOperation::AddStat => {
            matches!(unit, AbilityValueUnit::Flat | AbilityValueUnit::Ratio)
        }
    };
    if valid {
        Ok(())
    } else {
        Err(invalid("Ability Tree operation/unit contract is invalid"))
    }
}

fn service_kind(value: UniverseServiceKind) -> ServiceKind {
    match value {
        UniverseServiceKind::Currency => ServiceKind::Currency,
        UniverseServiceKind::ResetBlessing => ServiceKind::ResetBlessing,
        UniverseServiceKind::Reviver => ServiceKind::Reviver,
        UniverseServiceKind::Downloader => ServiceKind::Downloader,
        UniverseServiceKind::RespiteOffers => ServiceKind::RespiteOffers,
        UniverseServiceKind::EnhanceBlessing => ServiceKind::EnhanceBlessing,
        UniverseServiceKind::BlessingShop => ServiceKind::BlessingShop,
        UniverseServiceKind::CurioShop => ServiceKind::CurioShop,
        UniverseServiceKind::TrailblazeBonus => ServiceKind::TrailblazeBonus,
    }
}
fn ability_operation(value: UniverseAbilityOperation) -> AbilityOperation {
    match value {
        UniverseAbilityOperation::Unlock => AbilityOperation::Unlock,
        UniverseAbilityOperation::AddStat => AbilityOperation::AddStat,
        UniverseAbilityOperation::UnlockFormationSlot => AbilityOperation::UnlockFormationSlot,
        UniverseAbilityOperation::Set => AbilityOperation::Set,
        UniverseAbilityOperation::AddLimit => AbilityOperation::AddLimit,
        UniverseAbilityOperation::Enable => AbilityOperation::Enable,
        UniverseAbilityOperation::AddCurrency => AbilityOperation::AddCurrency,
        UniverseAbilityOperation::AddChoice => AbilityOperation::AddChoice,
        UniverseAbilityOperation::AddResource => AbilityOperation::AddResource,
        UniverseAbilityOperation::SetRatio => AbilityOperation::SetRatio,
    }
}
fn ability_unit(value: UniverseAbilityValueUnit) -> AbilityValueUnit {
    match value {
        UniverseAbilityValueUnit::Boolean => AbilityValueUnit::Boolean,
        UniverseAbilityValueUnit::Flat => AbilityValueUnit::Flat,
        UniverseAbilityValueUnit::Count => AbilityValueUnit::Count,
        UniverseAbilityValueUnit::Ratio => AbilityValueUnit::Ratio,
    }
}

fn ordered_groups<T>(
    groups: BTreeMap<i32, Vec<(u32, T)>>,
    label: &str,
) -> Result<BTreeMap<i32, Vec<T>>, UniverseCatalogLoadError> {
    groups
        .into_iter()
        .map(|(parent, mut values)| {
            values.sort_by_key(|value| value.0);
            if values
                .iter()
                .map(|value| value.0)
                .ne(1..=values.len() as u32)
            {
                return Err(invalid(format!("{label} sequence is not contiguous")));
            }
            Ok((parent, values.into_iter().map(|value| value.1).collect()))
        })
        .collect()
}
fn positive_sequence(value: i32, label: &str) -> Result<u32, UniverseCatalogLoadError> {
    u32::try_from(value)
        .ok()
        .filter(|value| *value != 0)
        .ok_or_else(|| invalid(format!("{label} sequence must be positive")))
}
fn optional_key(
    value: Option<&str>,
    label: &str,
) -> Result<Option<Box<str>>, UniverseCatalogLoadError> {
    value
        .map(|value| checked_key(value, label).map(Into::into))
        .transpose()
}
fn key_list(
    values: Option<&[String]>,
    label: &str,
) -> Result<Box<[Box<str>]>, UniverseCatalogLoadError> {
    let mut seen = BTreeSet::new();
    values
        .unwrap_or_default()
        .iter()
        .map(|value| {
            checked_key(value, label)?;
            if !seen.insert(value) {
                return Err(invalid(format!("{label} is duplicated")));
            }
            Ok(value.as_str().into())
        })
        .collect::<Result<Vec<_>, _>>()
        .map(Vec::into_boxed_slice)
}
fn display<'a>(value: &'a str, label: &str) -> Result<&'a str, UniverseCatalogLoadError> {
    if !value.trim().is_empty() && value.len() <= 200 {
        Ok(value)
    } else {
        Err(invalid(format!("{label} is empty or too long")))
    }
}
fn require_node(config: &SoraConfig, id: i32, label: &str) -> Result<(), UniverseCatalogLoadError> {
    if config.universe_ability_tree_node().get(&id).is_some() {
        Ok(())
    } else {
        Err(reference(format!("{label} parent node is unresolved")))
    }
}
fn service_id(raw: i32, label: &str) -> Result<ServiceId, UniverseCatalogLoadError> {
    u32::try_from(raw)
        .ok()
        .and_then(ServiceId::new)
        .ok_or_else(|| invalid(format!("{label} ID must be a positive u32")))
}
fn ability_id(raw: i32, label: &str) -> Result<AbilityTreeNodeId, UniverseCatalogLoadError> {
    u32::try_from(raw)
        .ok()
        .and_then(AbilityTreeNodeId::new)
        .ok_or_else(|| invalid(format!("{label} ID must be a positive u32")))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn operation_unit_contract_is_closed() {
        assert!(
            validate_operation_unit(AbilityOperation::Unlock, AbilityValueUnit::Boolean).is_ok()
        );
        assert!(
            validate_operation_unit(AbilityOperation::Unlock, AbilityValueUnit::Ratio).is_err()
        );
    }
}
