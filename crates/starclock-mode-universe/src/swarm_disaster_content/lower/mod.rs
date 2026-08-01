mod audit;
mod encounter;
mod inventory;
mod topology;

use crate::{
    swarm_disaster_catalog::SwarmDisasterBundleSummary, swarm_disaster_generated::SoraConfig,
    swarm_disaster_structural::SwarmDisasterStructuralCatalog,
    swarm_disaster_unique::SwarmDisasterUniqueCatalog,
};

use super::{
    SwarmDisasterContentCatalog, SwarmDisasterContentError, SwarmDisasterContentErrorKind, validate,
};

const ROW_REVISION: &str = "starclock.swarm-disaster-row.v1";

pub(super) fn lower(
    bundle: SwarmDisasterBundleSummary,
    source: &SoraConfig,
    structural: &SwarmDisasterStructuralCatalog,
    unique: &SwarmDisasterUniqueCatalog,
) -> Result<SwarmDisasterContentCatalog, SwarmDisasterContentError> {
    let (map_events, block_rules, topology_consequences) = topology::lower(source)?;
    let inventory::InventoryTables {
        blessings,
        blessing_levels,
        pool_memberships,
        curios,
        curio_states,
        curio_rules,
        occurrences,
        occurrence_variants,
        occurrence_choices,
        services,
        adventure_outcomes,
        currencies,
        service_rules,
    } = inventory::lower(source)?;
    let (encounter_groups, encounter_waves, enemy_slots, boss_pools) = encounter::lower(source)?;
    let (mechanic_rules, review_fixtures, audit) = audit::lower(source)?;
    let catalog = SwarmDisasterContentCatalog {
        bundle,
        map_events,
        block_rules,
        topology_consequences,
        blessings,
        blessing_levels,
        pool_memberships,
        curios,
        curio_states,
        curio_rules,
        occurrences,
        occurrence_variants,
        occurrence_choices,
        services,
        adventure_outcomes,
        currencies,
        service_rules,
        encounter_groups,
        encounter_waves,
        enemy_slots,
        boss_pools,
        mechanic_rules,
        review_fixtures,
        audit,
    };
    validate::catalog(&catalog, structural, unique)?;
    Ok(catalog)
}

pub(super) fn metadata(
    key: &str,
    revision: &str,
    kind: &str,
) -> Result<(), SwarmDisasterContentError> {
    stable(key, key)?;
    if revision != ROW_REVISION || kind.is_empty() {
        return fail(SwarmDisasterContentErrorKind::Metadata, key);
    }
    Ok(())
}

pub(super) fn stable(value: &str, key: &str) -> Result<Box<str>, SwarmDisasterContentError> {
    if value.is_empty() || value.len() > 256 || !value.bytes().all(|byte| byte.is_ascii_graphic()) {
        return fail(SwarmDisasterContentErrorKind::Identifier, key);
    }
    Ok(value.into())
}

pub(super) fn nonempty(value: &str, key: &str) -> Result<Box<str>, SwarmDisasterContentError> {
    if value.trim().is_empty() {
        return fail(SwarmDisasterContentErrorKind::Metadata, key);
    }
    Ok(value.into())
}

pub(super) fn json(value: &str, key: &str) -> Result<Box<str>, SwarmDisasterContentError> {
    serde_json::from_str::<serde_json::Value>(value)
        .map_err(|_| error(SwarmDisasterContentErrorKind::Metadata, key))?;
    Ok(value.into())
}

pub(super) fn scalar(value: &str, key: &str) -> Result<Box<str>, SwarmDisasterContentError> {
    let unsigned = value.strip_prefix('-').unwrap_or(value);
    let (integer, fraction) = unsigned
        .split_once('.')
        .map_or((unsigned, None), |(integer, fraction)| {
            (integer, Some(fraction))
        });
    let valid_integer = integer == "0"
        || (!integer.starts_with('0') && integer.bytes().all(|byte| byte.is_ascii_digit()));
    let valid_fraction = fraction.is_none_or(|fraction| {
        !fraction.is_empty()
            && !fraction.ends_with('0')
            && fraction.bytes().all(|byte| byte.is_ascii_digit())
    });
    if value.is_empty()
        || value == "-0"
        || value.starts_with('+')
        || !valid_integer
        || !valid_fraction
    {
        return fail(SwarmDisasterContentErrorKind::Identifier, key);
    }
    Ok(value.into())
}

pub(super) fn text_list(
    values: &[String],
    key: &str,
) -> Result<Box<[Box<str>]>, SwarmDisasterContentError> {
    values
        .iter()
        .map(|value| stable(value, key))
        .collect::<Result<Vec<_>, _>>()
        .map(Vec::into_boxed_slice)
}

pub(super) fn optional_text_list(
    values: Option<&[String]>,
    key: &str,
) -> Result<Box<[Box<str>]>, SwarmDisasterContentError> {
    values.map_or_else(
        || Ok(Vec::<Box<str>>::new().into_boxed_slice()),
        |values| text_list(values, key),
    )
}

pub(super) fn text_values(
    values: &[String],
    key: &str,
) -> Result<Box<[Box<str>]>, SwarmDisasterContentError> {
    values
        .iter()
        .map(|value| nonempty(value, key))
        .collect::<Result<Vec<_>, _>>()
        .map(Vec::into_boxed_slice)
}

pub(super) fn positive(value: i32, key: &str) -> Result<u32, SwarmDisasterContentError> {
    u32::try_from(value)
        .ok()
        .filter(|value| *value > 0)
        .ok_or_else(|| error(SwarmDisasterContentErrorKind::Identifier, key))
}

pub(super) fn positive_u16(value: i32, key: &str) -> Result<u16, SwarmDisasterContentError> {
    u16::try_from(value)
        .ok()
        .filter(|value| *value > 0)
        .ok_or_else(|| error(SwarmDisasterContentErrorKind::Identifier, key))
}

pub(super) fn nonnegative_u16(value: i32, key: &str) -> Result<u16, SwarmDisasterContentError> {
    u16::try_from(value).map_err(|_| error(SwarmDisasterContentErrorKind::Identifier, key))
}

pub(super) fn positive_u8(value: i32, key: &str) -> Result<u8, SwarmDisasterContentError> {
    u8::try_from(value)
        .ok()
        .filter(|value| *value > 0)
        .ok_or_else(|| error(SwarmDisasterContentErrorKind::Identifier, key))
}

pub(super) fn fail<T>(
    kind: SwarmDisasterContentErrorKind,
    key: &str,
) -> Result<T, SwarmDisasterContentError> {
    Err(error(kind, key))
}

pub(super) fn error(kind: SwarmDisasterContentErrorKind, key: &str) -> SwarmDisasterContentError {
    SwarmDisasterContentError {
        kind,
        key: key.into(),
    }
}
