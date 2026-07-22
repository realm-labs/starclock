//! Strict lowering for Path, Blessing and Resonance content.

use std::collections::{BTreeMap, BTreeSet};

use crate::digest::{Encoder, UniversePathDefinitionsDigest};
use crate::error::UniverseCatalogLoadError;
use crate::generated::{SoraConfig, universe_resonance_kind::UniverseResonanceKind};
use crate::id::{BlessingId, BlessingLevelId, PathId, ResonanceId};
use crate::lowering::{
    checked_key, checked_source, invalid, localized, parse_digest, positive_u8, reference,
};
use crate::path::{
    BlessingDefinition, BlessingLevelDefinition, ExactParameter, PathDefinition, PathDefinitions,
    ResonanceDefinition, ResonanceKind,
};

pub(crate) fn lower(config: &SoraConfig) -> Result<PathDefinitions, UniverseCatalogLoadError> {
    let levels = lower_levels(config)?;
    let blessings = lower_blessings(config, &levels)?;
    let resonances = lower_resonances(config)?;
    let paths = lower_paths(config, &blessings, &resonances)?;
    let digest = digest(&paths, &blessings, &levels, &resonances);
    Ok(PathDefinitions {
        digest,
        paths,
        blessings,
        levels,
        resonances,
    })
}

fn lower_levels(
    config: &SoraConfig,
) -> Result<Box<[BlessingLevelDefinition]>, UniverseCatalogLoadError> {
    let parameters = parameter_groups(
        config.universe_blessing_parameter().iter().map(|row| {
            (
                row.blessing_level_id,
                row.sequence,
                row.value_decimal.as_str(),
            )
        }),
        "Blessing level parameter",
    )?;
    let mut semantic = BTreeSet::new();
    let mut definitions = Vec::with_capacity(config.universe_blessing_level().len());
    for row in config.universe_blessing_level().ordered_rows() {
        let id = transport_id::<BlessingLevelId>(row.id, "Blessing level")?;
        let blessing = transport_id::<BlessingId>(row.blessing_id, "Blessing level parent")?;
        if config.universe_blessing().get(&row.blessing_id).is_none() {
            return Err(reference("Blessing level references an unknown Blessing"));
        }
        let level = positive_u8(row.level, "Blessing level ordinal")?;
        if !semantic.insert((blessing, level)) {
            return Err(invalid("Blessing level ordinal is duplicated"));
        }
        validate_rule(config, &row.rule_stable_key)?;
        definitions.push(BlessingLevelDefinition::new(
            id,
            checked_key(&row.stable_key, "Blessing level stable key")?,
            blessing,
            level,
            checked_source(&row.source_binding_key, "Blessing source binding")?,
            &row.rule_stable_key,
            localized(
                &row.name_en,
                &row.name_zh_cn,
                &row.summary_en,
                &row.summary_zh_cn,
                "Blessing level",
            )?,
            parameters
                .get(&row.id)
                .cloned()
                .unwrap_or_default()
                .into_boxed_slice(),
        ));
    }
    definitions.sort_by_key(BlessingLevelDefinition::id);
    if definitions.len() != 324
        || parameters
            .keys()
            .any(|id| config.universe_blessing_level().get(id).is_none())
    {
        return Err(reference(
            "Blessing level/parameter denominator or parent differs",
        ));
    }
    Ok(definitions.into_boxed_slice())
}

fn lower_blessings(
    config: &SoraConfig,
    levels: &[BlessingLevelDefinition],
) -> Result<Box<[BlessingDefinition]>, UniverseCatalogLoadError> {
    let prerequisites = string_groups(
        config.universe_blessing_prerequisite().iter().map(|row| {
            (
                row.blessing_id,
                row.sequence,
                row.prerequisite_stable_key.as_str(),
            )
        }),
        "Blessing prerequisite",
    )?;
    let mut definitions = Vec::with_capacity(config.universe_blessing().len());
    for row in config.universe_blessing().ordered_rows() {
        let blessing_id = transport_id::<BlessingId>(row.id, "Blessing")?;
        let path = transport_id::<PathId>(row.path_id, "Blessing Path")?;
        if config.universe_path().get(&row.path_id).is_none() {
            return Err(reference("Blessing references an unknown Path"));
        }
        validate_rule(config, &row.rule_stable_key)?;
        let blessing_levels = levels
            .iter()
            .filter(|level| level.blessing() == blessing_id)
            .map(BlessingLevelDefinition::id)
            .collect::<Vec<_>>();
        if blessing_levels.len() != 2
            || levels
                .iter()
                .filter(|level| level.blessing() == blessing_id)
                .map(BlessingLevelDefinition::level)
                .collect::<BTreeSet<_>>()
                != BTreeSet::from([1, 2])
        {
            return Err(reference(
                "Blessing must have exact base and enhanced levels",
            ));
        }
        definitions.push(BlessingDefinition::new(
            blessing_id,
            checked_key(&row.stable_key, "Blessing stable key")?,
            path,
            match row.rarity {
                1..=3 => row.rarity as u8,
                _ => return Err(invalid("Blessing rarity is outside 1 through 3")),
            },
            localized(
                &row.name_en,
                &row.name_zh_cn,
                &row.summary_en,
                &row.summary_zh_cn,
                "Blessing",
            )?,
            tags(row.pool_tags.as_deref(), "Blessing pool tag")?,
            tags(row.mechanic_tags.as_deref(), "Blessing mechanic tag")?,
            &row.rule_stable_key,
            parse_digest(
                &row.source_description_sha256_en,
                "Blessing English description digest",
            )?,
            parse_digest(
                &row.source_description_sha256_zh_cn,
                "Blessing Chinese description digest",
            )?,
            blessing_levels.into_boxed_slice(),
            prerequisites
                .get(&row.id)
                .cloned()
                .unwrap_or_default()
                .into_boxed_slice(),
        ));
    }
    definitions.sort_by_key(BlessingDefinition::id);
    if definitions.len() != 162
        || prerequisites
            .keys()
            .any(|id| config.universe_blessing().get(id).is_none())
    {
        return Err(reference(
            "Blessing/prerequisite denominator or parent differs",
        ));
    }
    Ok(definitions.into_boxed_slice())
}

fn lower_resonances(
    config: &SoraConfig,
) -> Result<Box<[ResonanceDefinition]>, UniverseCatalogLoadError> {
    let parameters = parameter_groups(
        config
            .universe_resonance_parameter()
            .iter()
            .map(|row| (row.resonance_id, row.sequence, row.value_decimal.as_str())),
        "Resonance parameter",
    )?;
    let mut definitions = Vec::with_capacity(config.universe_resonance().len());
    for row in config.universe_resonance().ordered_rows() {
        let path = transport_id::<PathId>(row.path_id, "Resonance Path")?;
        if config.universe_path().get(&row.path_id).is_none() {
            return Err(reference("Resonance references an unknown Path"));
        }
        let kind = match row.kind {
            UniverseResonanceKind::Resonance => ResonanceKind::Resonance,
            UniverseResonanceKind::Formation => ResonanceKind::Formation,
        };
        let threshold = u8::try_from(row.threshold)
            .map_err(|_| invalid("Resonance threshold must be a non-negative u8"))?;
        if (kind == ResonanceKind::Resonance && threshold != 3)
            || (kind == ResonanceKind::Formation && threshold != 0)
        {
            return Err(invalid("Resonance/Formation threshold contract differs"));
        }
        validate_rule(config, &row.rule_stable_key)?;
        definitions.push(ResonanceDefinition::new(
            transport_id::<ResonanceId>(row.id, "Resonance")?,
            checked_key(&row.stable_key, "Resonance stable key")?,
            path,
            kind,
            threshold,
            parse_decimal(&row.energy_max_decimal)?,
            parse_decimal(&row.initial_energy_decimal)?,
            localized(
                &row.name_en,
                &row.name_zh_cn,
                &row.summary_en,
                &row.summary_zh_cn,
                "Resonance",
            )?,
            tags(row.mechanic_tags.as_deref(), "Resonance mechanic tag")?,
            checked_source(&row.source_binding_key, "Resonance source binding")?,
            &row.rule_stable_key,
            parameters
                .get(&row.id)
                .cloned()
                .unwrap_or_default()
                .into_boxed_slice(),
        ));
    }
    definitions.sort_by_key(ResonanceDefinition::id);
    if definitions.len() != 36
        || parameters
            .keys()
            .any(|id| config.universe_resonance().get(id).is_none())
    {
        return Err(reference(
            "Resonance/parameter denominator or parent differs",
        ));
    }
    Ok(definitions.into_boxed_slice())
}

fn lower_paths(
    config: &SoraConfig,
    blessings: &[BlessingDefinition],
    resonances: &[ResonanceDefinition],
) -> Result<Box<[PathDefinition]>, UniverseCatalogLoadError> {
    let authored = string_groups(
        config
            .universe_path_blessing()
            .iter()
            .map(|row| (row.path_id, row.sequence, row.blessing_stable_key.as_str())),
        "Path Blessing",
    )?;
    let blessing_by_key = blessings
        .iter()
        .map(|value| (value.stable_key(), value))
        .collect::<BTreeMap<_, _>>();
    let mut buff_types = BTreeSet::new();
    let mut definitions = Vec::with_capacity(config.universe_path().len());
    for row in config.universe_path().ordered_rows() {
        let path_id = transport_id::<PathId>(row.id, "Path")?;
        let buff_type = row
            .buff_type
            .parse::<u32>()
            .ok()
            .filter(|value| *value != 0)
            .ok_or_else(|| invalid("Path buff type is not a positive u32"))?;
        if !buff_types.insert(buff_type) {
            return Err(invalid("Path buff type is duplicated"));
        }
        let blessing_ids = authored
            .get(&row.id)
            .ok_or_else(|| reference("Path has no authored Blessing order"))?
            .iter()
            .map(|key| {
                let blessing = blessing_by_key
                    .get(key.as_ref())
                    .ok_or_else(|| reference("Path Blessing stable key is unresolved"))?;
                if blessing.path() != path_id {
                    return Err(reference("Path Blessing belongs to another Path"));
                }
                Ok(blessing.id())
            })
            .collect::<Result<Vec<_>, UniverseCatalogLoadError>>()?;
        if blessing_ids.len() != 18
            || blessing_ids.iter().copied().collect::<BTreeSet<_>>().len() != 18
        {
            return Err(reference(
                "each Path must bind exactly 18 distinct Blessings",
            ));
        }
        let path_resonances = resonances
            .iter()
            .filter(|value| value.path() == path_id)
            .collect::<Vec<_>>();
        let resonance = path_resonances
            .iter()
            .find(|value| value.kind() == ResonanceKind::Resonance)
            .ok_or_else(|| reference("Path has no Resonance"))?
            .id();
        let formations = path_resonances
            .iter()
            .filter(|value| value.kind() == ResonanceKind::Formation)
            .map(|value| value.id())
            .collect::<Vec<_>>();
        if path_resonances.len() != 4 || formations.len() != 3 {
            return Err(reference(
                "Path must have one Resonance and three Formations",
            ));
        }
        definitions.push(PathDefinition::new(
            path_id,
            checked_key(&row.stable_key, "Path stable key")?,
            buff_type,
            localized(
                &row.name_en,
                &row.name_zh_cn,
                &row.summary_en,
                &row.summary_zh_cn,
                "Path",
            )?,
            checked_key(&row.unlock_policy_stable_key, "Path unlock policy")?,
            resonance,
            formations.into_boxed_slice(),
            blessing_ids.into_boxed_slice(),
        ));
    }
    definitions.sort_by_key(PathDefinition::id);
    if definitions.len() != 9
        || authored
            .keys()
            .any(|id| config.universe_path().get(id).is_none())
    {
        return Err(reference("Path denominator or Blessing parent differs"));
    }
    Ok(definitions.into_boxed_slice())
}

pub(crate) fn validate_rule(
    config: &SoraConfig,
    key: &str,
) -> Result<(), UniverseCatalogLoadError> {
    checked_key(key, "Universe mechanic rule key")?;
    if config
        .universe_mechanic_rule()
        .get_by_stable_key(key)
        .is_none()
    {
        Err(reference("Path-family mechanic rule key is unresolved"))
    } else {
        Ok(())
    }
}

pub(crate) fn tags(
    values: Option<&[String]>,
    label: &str,
) -> Result<Box<[Box<str>]>, UniverseCatalogLoadError> {
    let mut result = Vec::new();
    let mut seen = BTreeSet::new();
    for value in values.unwrap_or_default() {
        if value.is_empty()
            || value.len() > 160
            || !value.bytes().all(|byte| {
                byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_' | b':')
            })
        {
            return Err(invalid(format!("{label} is malformed")));
        }
        if !seen.insert(value.as_str()) {
            return Err(invalid(format!("{label} is duplicated")));
        }
        result.push(value.as_str().into());
    }
    Ok(result.into_boxed_slice())
}

pub(crate) fn parameter_groups<'a>(
    rows: impl IntoIterator<Item = (i32, i32, &'a str)>,
    label: &str,
) -> Result<BTreeMap<i32, Vec<ExactParameter>>, UniverseCatalogLoadError> {
    let mut groups: BTreeMap<i32, Vec<(u32, ExactParameter)>> = BTreeMap::new();
    for (parent, sequence, value) in rows {
        let sequence = u32::try_from(sequence)
            .ok()
            .filter(|value| *value != 0)
            .ok_or_else(|| invalid(format!("{label} sequence must be positive")))?;
        groups
            .entry(parent)
            .or_default()
            .push((sequence, parse_decimal(value)?));
    }
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

fn string_groups<'a>(
    rows: impl IntoIterator<Item = (i32, i32, &'a str)>,
    label: &str,
) -> Result<BTreeMap<i32, Vec<Box<str>>>, UniverseCatalogLoadError> {
    let mut groups: BTreeMap<i32, Vec<(u32, Box<str>)>> = BTreeMap::new();
    for (parent, sequence, value) in rows {
        let sequence = u32::try_from(sequence)
            .ok()
            .filter(|value| *value != 0)
            .ok_or_else(|| invalid(format!("{label} sequence must be positive")))?;
        checked_key(value, label)?;
        groups
            .entry(parent)
            .or_default()
            .push((sequence, value.into()));
    }
    groups
        .into_iter()
        .map(|(parent, mut values)| {
            values.sort_by_key(|value| value.0);
            if values
                .iter()
                .map(|value| value.0)
                .ne(1..=values.len() as u32)
                || values
                    .iter()
                    .map(|value| value.1.as_ref())
                    .collect::<BTreeSet<_>>()
                    .len()
                    != values.len()
            {
                return Err(invalid(format!(
                    "{label} order is non-contiguous or duplicated"
                )));
            }
            Ok((parent, values.into_iter().map(|value| value.1).collect()))
        })
        .collect()
}

pub(crate) fn parse_decimal(value: &str) -> Result<ExactParameter, UniverseCatalogLoadError> {
    let (negative, unsigned) = value
        .strip_prefix('-')
        .map_or((false, value), |rest| (true, rest));
    if unsigned.is_empty() || unsigned.starts_with('+') {
        return Err(invalid("parameter decimal is malformed"));
    }
    let mut parts = unsigned.split('.');
    let integer = parts.next().unwrap_or_default();
    let fraction = parts.next();
    if parts.next().is_some()
        || integer.is_empty()
        || !integer.bytes().all(|byte| byte.is_ascii_digit())
        || (integer.len() > 1 && integer.starts_with('0'))
    {
        return Err(invalid("parameter decimal is noncanonical"));
    }
    let fraction = fraction.unwrap_or("");
    if fraction.len() > 10
        || !fraction.bytes().all(|byte| byte.is_ascii_digit())
        || (value.contains('.') && fraction.is_empty())
    {
        return Err(invalid("parameter decimal precision is invalid"));
    }
    let whole = integer
        .parse::<i64>()
        .map_err(|_| invalid("parameter decimal overflows"))?;
    let scale = u8::try_from(fraction.len()).expect("maximum precision validated");
    let base = 10_i64.pow(u32::from(scale));
    let fractional = if fraction.is_empty() {
        0
    } else {
        fraction
            .parse::<i64>()
            .map_err(|_| invalid("parameter fraction overflows"))?
    };
    let coefficient = whole
        .checked_mul(base)
        .and_then(|value| value.checked_add(fractional))
        .ok_or_else(|| invalid("parameter decimal overflows"))?;
    let coefficient = if negative {
        coefficient
            .checked_neg()
            .ok_or_else(|| invalid("parameter decimal overflows"))?
    } else {
        coefficient
    };
    Ok(ExactParameter::new(coefficient, scale))
}

trait TransportId: Sized {
    fn new(raw: u32) -> Option<Self>;
}
macro_rules! transport_ids { ($($name:ty),+ $(,)?) => { $(impl TransportId for $name { fn new(raw: u32) -> Option<Self> { <$name>::new(raw) } })+ }; }
transport_ids!(PathId, BlessingId, BlessingLevelId, ResonanceId);
fn transport_id<T: TransportId>(raw: i32, label: &str) -> Result<T, UniverseCatalogLoadError> {
    u32::try_from(raw)
        .ok()
        .and_then(T::new)
        .ok_or_else(|| invalid(format!("{label} ID must be a positive u32")))
}

fn digest(
    paths: &[PathDefinition],
    blessings: &[BlessingDefinition],
    levels: &[BlessingLevelDefinition],
    resonances: &[ResonanceDefinition],
) -> UniversePathDefinitionsDigest {
    let mut encoder = Encoder::new(b"starclock-standard-universe-path-definitions-v1");
    encoder.u32(paths.len() as u32);
    for value in paths {
        encoder.u32(value.id().get());
        encoder.text(value.stable_key());
        encoder.u32(value.buff_type());
        encode_text(&mut encoder, value.text());
        encoder.text(value.unlock_policy_key());
        encoder.u32(value.resonance().get());
        encoder.u32(value.formations().len() as u32);
        for id in value.formations() {
            encoder.u32(id.get());
        }
        encoder.u32(value.blessings().len() as u32);
        for id in value.blessings() {
            encoder.u32(id.get());
        }
    }
    encoder.u32(blessings.len() as u32);
    for value in blessings {
        encoder.u32(value.id().get());
        encoder.text(value.stable_key());
        encoder.u32(value.path().get());
        encoder.u8(value.rarity());
        encode_text(&mut encoder, value.text());
        encode_strings(&mut encoder, value.pool_tags());
        encode_strings(&mut encoder, value.mechanic_tags());
        encoder.text(value.rule_key());
        encoder.digest(value.source_description_en());
        encoder.digest(value.source_description_zh_cn());
        encoder.u32(value.levels().len() as u32);
        for id in value.levels() {
            encoder.u32(id.get());
        }
        encode_strings(&mut encoder, value.prerequisite_keys());
    }
    encoder.u32(levels.len() as u32);
    for value in levels {
        encoder.u32(value.id().get());
        encoder.text(value.stable_key());
        encoder.u32(value.blessing().get());
        encoder.u8(value.level());
        encoder.text(value.source_binding_key());
        encoder.text(value.rule_key());
        encode_text(&mut encoder, value.text());
        encode_parameters(&mut encoder, value.parameters());
    }
    encoder.u32(resonances.len() as u32);
    for value in resonances {
        encoder.u32(value.id().get());
        encoder.text(value.stable_key());
        encoder.u32(value.path().get());
        encoder.u8(value.kind() as u8);
        encoder.u8(value.threshold());
        encode_parameter(&mut encoder, value.energy_max());
        encode_parameter(&mut encoder, value.initial_energy());
        encode_text(&mut encoder, value.text());
        encode_strings(&mut encoder, value.mechanic_tags());
        encoder.text(value.source_binding_key());
        encoder.text(value.rule_key());
        encode_parameters(&mut encoder, value.parameters());
    }
    UniversePathDefinitionsDigest::new(encoder.finish())
}

fn encode_text(encoder: &mut Encoder, value: &crate::definition::LocalizedText) {
    encoder.text(value.name_en());
    encoder.text(value.name_zh_cn());
    encoder.text(value.summary_en());
    encoder.text(value.summary_zh_cn());
}
fn encode_strings(encoder: &mut Encoder, values: &[Box<str>]) {
    encoder.u32(values.len() as u32);
    for value in values {
        encoder.text(value);
    }
}
fn encode_parameters(encoder: &mut Encoder, values: &[ExactParameter]) {
    encoder.u32(values.len() as u32);
    for value in values {
        encode_parameter(encoder, *value);
    }
}

fn encode_parameter(encoder: &mut Encoder, value: ExactParameter) {
    encoder.i64(value.coefficient());
    encoder.u8(value.scale());
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_decimal_parser_rejects_noncanonical_and_overprecision_values() {
        assert_eq!(
            parse_decimal("12.3456").expect("decimal"),
            ExactParameter::new(123_456, 4)
        );
        assert_eq!(
            parse_decimal("-0.5").expect("negative"),
            ExactParameter::new(-5, 1)
        );
        assert_eq!(
            parse_decimal("0.0019999999").expect("source precision"),
            ExactParameter::new(19_999_999, 10)
        );
        for value in ["+1", "01", "1.", ".5", "1.00000000001"] {
            assert!(parse_decimal(value).is_err(), "{value}");
        }
    }
}
