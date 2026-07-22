//! Strict lowering for Curio definitions and lifecycle states.

use std::collections::{BTreeMap, BTreeSet};

use crate::curio::{CurioDefinition, CurioDefinitions, CurioStateDefinition, CurioStateKind};
use crate::digest::{Encoder, UniverseCurioDefinitionsDigest};
use crate::error::UniverseCatalogLoadError;
use crate::generated::{SoraConfig, universe_curio_state_kind::UniverseCurioStateKind};
use crate::id::{CurioId, CurioStateId};
use crate::lowering::{checked_key, checked_source, invalid, localized, reference};
use crate::path::ExactParameter;
use crate::path_lowering::{parameter_groups, parse_decimal, tags, validate_rule};

pub(crate) fn lower(config: &SoraConfig) -> Result<CurioDefinitions, UniverseCatalogLoadError> {
    let states = lower_states(config)?;
    validate_lifecycle(&states)?;
    let curios = lower_curios(config, &states)?;
    let digest = digest(&curios, &states);
    Ok(CurioDefinitions {
        digest,
        curios,
        states,
    })
}

fn lower_states(
    config: &SoraConfig,
) -> Result<Box<[CurioStateDefinition]>, UniverseCatalogLoadError> {
    let parameters = parameter_groups(
        config
            .universe_curio_parameter()
            .iter()
            .map(|row| (row.curio_state_id, row.sequence, row.value_decimal.as_str())),
        "Curio state parameter",
    )?;
    let mut definitions = Vec::with_capacity(config.universe_curio_state().len());
    for row in config.universe_curio_state().ordered_rows() {
        let id = state_id(row.id, "Curio state")?;
        let curio = curio_id(row.curio_id, "Curio state parent")?;
        if config.universe_curio().get(&row.curio_id).is_none() {
            return Err(reference("Curio state references an unknown Curio"));
        }
        validate_rule(config, &row.rule_stable_key)?;
        let values = parameters.get(&row.id).cloned().unwrap_or_default();
        let charge_parameter_index = match row.charge_parameter_index {
            0 => None,
            value => {
                let value = u8::try_from(value)
                    .ok()
                    .filter(|value| usize::from(*value) <= values.len())
                    .ok_or_else(|| invalid("Curio charge parameter index is out of bounds"))?;
                Some(value)
            }
        };
        if row
            .replacement_curio_id
            .is_some_and(|id| config.universe_curio().get(&id).is_none())
        {
            return Err(reference("Curio replacement reference is unresolved"));
        }
        definitions.push(CurioStateDefinition::new(
            id,
            checked_key(&row.stable_key, "Curio state stable key")?,
            curio,
            match row.state_kind {
                UniverseCurioStateKind::Active => CurioStateKind::Active,
                UniverseCurioStateKind::Repairing => CurioStateKind::Repairing,
                UniverseCurioStateKind::Fixed => CurioStateKind::Fixed,
            },
            row.charges_decimal
                .as_deref()
                .map(parse_decimal)
                .transpose()?,
            charge_parameter_index,
            optional_state_id(row.next_state_id, "Curio next state")?,
            optional_state_id(row.repair_state_id, "Curio repair state")?,
            optional_curio_id(row.replacement_curio_id, "replacement Curio")?,
            checked_source(&row.source_effect_id, "Curio source effect ID")?,
            &row.rule_stable_key,
            localized(
                &row.name_en,
                &row.name_zh_cn,
                &row.summary_en,
                &row.summary_zh_cn,
                "Curio state",
            )?,
            values.into_boxed_slice(),
        ));
    }
    definitions.sort_by_key(CurioStateDefinition::id);
    if definitions.len() != 67
        || config.universe_curio_parameter().len() != 89
        || parameters
            .keys()
            .any(|id| config.universe_curio_state().get(id).is_none())
    {
        return Err(reference(
            "Curio state/parameter denominator or parent differs",
        ));
    }
    Ok(definitions.into_boxed_slice())
}

fn lower_curios(
    config: &SoraConfig,
    states: &[CurioStateDefinition],
) -> Result<Box<[CurioDefinition]>, UniverseCatalogLoadError> {
    let states_by_key = states
        .iter()
        .map(|state| (state.stable_key(), state))
        .collect::<BTreeMap<_, _>>();
    let mut handbook_orders = BTreeSet::new();
    let mut definitions = Vec::with_capacity(config.universe_curio().len());
    for row in config.universe_curio().ordered_rows() {
        let id = curio_id(row.id, "Curio")?;
        let initial_key = checked_key(
            &row.initial_state_stable_key,
            "Curio initial state stable key",
        )?;
        let initial = states_by_key
            .get(initial_key)
            .copied()
            .ok_or_else(|| reference("Curio initial state stable key is unresolved"))?;
        if initial.curio() != id {
            return Err(reference("Curio initial state belongs to another Curio"));
        }
        let handbook_order = u32::try_from(row.handbook_order)
            .ok()
            .filter(|value| *value != 0)
            .ok_or_else(|| invalid("Curio handbook order must be positive"))?;
        if !handbook_orders.insert(handbook_order) {
            return Err(invalid("Curio handbook order is duplicated"));
        }
        validate_rule(config, &row.rule_stable_key)?;
        let state_ids = states
            .iter()
            .filter(|state| state.curio() == id)
            .map(CurioStateDefinition::id)
            .collect::<Vec<_>>();
        if state_ids.is_empty() {
            return Err(reference("Curio has no lifecycle state"));
        }
        definitions.push(CurioDefinition::new(
            id,
            checked_key(&row.stable_key, "Curio stable key")?,
            initial.id(),
            handbook_order,
            localized(
                &row.name_en,
                &row.name_zh_cn,
                &row.summary_en,
                &row.summary_zh_cn,
                "Curio",
            )?,
            tags(row.tags.as_deref(), "Curio tag")?,
            tags(row.pool_tags.as_deref(), "Curio pool tag")?,
            &row.rule_stable_key,
            state_ids.into_boxed_slice(),
        ));
    }
    definitions.sort_by_key(CurioDefinition::id);
    if definitions.len() != 61 {
        return Err(reference("Curio denominator differs from frozen release"));
    }
    Ok(definitions.into_boxed_slice())
}

fn validate_lifecycle(states: &[CurioStateDefinition]) -> Result<(), UniverseCatalogLoadError> {
    let by_id = states
        .iter()
        .map(|state| (state.id(), state))
        .collect::<BTreeMap<_, _>>();
    for state in states {
        for target in [state.next_state(), state.repair_state()]
            .into_iter()
            .flatten()
        {
            let target = by_id
                .get(&target)
                .ok_or_else(|| reference("Curio lifecycle state reference is unresolved"))?;
            if target.curio() != state.curio() {
                return Err(reference(
                    "Curio lifecycle transition crosses Curio ownership",
                ));
            }
        }
        if state
            .charges()
            .is_some_and(|value| value.coefficient() <= 0)
        {
            return Err(invalid("Curio state charges must be positive"));
        }
        match state.kind() {
            CurioStateKind::Repairing => {
                let next = state.next_state().ok_or_else(|| {
                    invalid("Repairing Curio state must declare its fixed transition")
                })?;
                if state.repair_state() != Some(next)
                    || by_id.get(&next).map(|value| value.kind()) != Some(CurioStateKind::Fixed)
                    || state.charges().is_none()
                {
                    return Err(invalid(
                        "Repairing Curio transition must consume charges and target its Fixed state",
                    ));
                }
            }
            CurioStateKind::Fixed => {
                if state.next_state().is_some()
                    || state.repair_state().is_some()
                    || state.charges().is_some()
                {
                    return Err(invalid("Fixed Curio state must be terminal"));
                }
            }
            CurioStateKind::Active => {
                if state.next_state().is_some() || state.repair_state().is_some() {
                    return Err(invalid(
                        "Active Curio state has an unsupported lifecycle edge",
                    ));
                }
            }
        }
    }
    Ok(())
}

fn curio_id(raw: i32, label: &str) -> Result<CurioId, UniverseCatalogLoadError> {
    u32::try_from(raw)
        .ok()
        .and_then(CurioId::new)
        .ok_or_else(|| invalid(format!("{label} ID must be a positive u32")))
}

fn state_id(raw: i32, label: &str) -> Result<CurioStateId, UniverseCatalogLoadError> {
    u32::try_from(raw)
        .ok()
        .and_then(CurioStateId::new)
        .ok_or_else(|| invalid(format!("{label} ID must be a positive u32")))
}

fn optional_curio_id(
    raw: Option<i32>,
    label: &str,
) -> Result<Option<CurioId>, UniverseCatalogLoadError> {
    raw.map(|value| curio_id(value, label)).transpose()
}

fn optional_state_id(
    raw: Option<i32>,
    label: &str,
) -> Result<Option<CurioStateId>, UniverseCatalogLoadError> {
    raw.map(|value| state_id(value, label)).transpose()
}

fn digest(
    curios: &[CurioDefinition],
    states: &[CurioStateDefinition],
) -> UniverseCurioDefinitionsDigest {
    let mut encoder = Encoder::new(b"starclock-standard-universe-curio-definitions-v1");
    encoder.u32(curios.len() as u32);
    for value in curios {
        encoder.u32(value.id().get());
        encoder.text(value.stable_key());
        encoder.u32(value.initial_state().get());
        encoder.u32(value.handbook_order());
        encode_text(&mut encoder, value.text());
        encode_texts(&mut encoder, value.tags());
        encode_texts(&mut encoder, value.pool_tags());
        encoder.text(value.rule_key());
        encoder.u32(value.states().len() as u32);
        for state in value.states() {
            encoder.u32(state.get());
        }
    }
    encoder.u32(states.len() as u32);
    for value in states {
        encoder.u32(value.id().get());
        encoder.text(value.stable_key());
        encoder.u32(value.curio().get());
        encoder.u8(value.kind() as u8);
        encode_optional_parameter(&mut encoder, value.charges());
        encode_optional_u32(&mut encoder, value.charge_parameter_index().map(u32::from));
        encode_optional_u32(&mut encoder, value.next_state().map(CurioStateId::get));
        encode_optional_u32(&mut encoder, value.repair_state().map(CurioStateId::get));
        encode_optional_u32(&mut encoder, value.replacement_curio().map(CurioId::get));
        encoder.text(value.source_effect_id());
        encoder.text(value.rule_key());
        encode_text(&mut encoder, value.text());
        encoder.u32(value.parameters().len() as u32);
        for parameter in value.parameters() {
            encode_parameter(&mut encoder, *parameter);
        }
    }
    UniverseCurioDefinitionsDigest::new(encoder.finish())
}

fn encode_text(encoder: &mut Encoder, text: &crate::definition::LocalizedText) {
    encoder.text(text.name_en());
    encoder.text(text.name_zh_cn());
    encoder.text(text.summary_en());
    encoder.text(text.summary_zh_cn());
}

fn encode_texts(encoder: &mut Encoder, values: &[Box<str>]) {
    encoder.u32(values.len() as u32);
    for value in values {
        encoder.text(value);
    }
}

fn encode_parameter(encoder: &mut Encoder, value: ExactParameter) {
    encoder.i64(value.coefficient());
    encoder.u8(value.scale());
}

fn encode_optional_parameter(encoder: &mut Encoder, value: Option<ExactParameter>) {
    encoder.bool(value.is_some());
    if let Some(value) = value {
        encode_parameter(encoder, value);
    }
}

fn encode_optional_u32(encoder: &mut Encoder, value: Option<u32>) {
    encoder.bool(value.is_some());
    if let Some(value) = value {
        encoder.u32(value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_decimal_charge_parser_rejects_noncanonical_input() {
        assert_eq!(parse_decimal("3").expect("integer").coefficient(), 3);
        assert!(parse_decimal("03").is_err());
        assert!(parse_decimal("1.").is_err());
        assert!(parse_decimal("0.12345678901").is_err());
    }
}
