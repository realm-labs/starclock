//! Sora native-handler metadata to the immutable compiled registry boundary.

use std::collections::BTreeSet;

use starclock_combat::NativeHandlerId;
use starclock_rules::{
    model::{HandlerDomain, NativeHandlerRequirement},
    registry,
};

use crate::{
    catalog::{CatalogLoadError, domain_fail},
    generated::{SoraConfig, rule_domain},
};

pub(super) fn audit(config: &SoraConfig) -> Result<BTreeSet<NativeHandlerId>, CatalogLoadError> {
    let requirements = config
        .native_handler()
        .ordered_rows()
        .map(|row| {
            Ok(NativeHandlerRequirement {
                id: handler_id(row.id)?,
                stable_key: &row.stable_key,
                domain: match row.domain {
                    rule_domain::RuleDomain::Battle => HandlerDomain::Battle,
                    rule_domain::RuleDomain::Activity => HandlerDomain::Activity,
                },
                version: &row.handler_version,
                argument_schema_digest: digest(&row.argument_schema_sha256)?,
                determinism_note: &row.determinism_note,
                owner: &row.owner_note,
                ir_insufficiency: &row.ir_insufficiency_reason,
                removal_condition: &row.removal_condition,
                enabled: row.enabled,
            })
        })
        .collect::<Result<Vec<_>, CatalogLoadError>>()?;
    registry::production()
        .audit(&requirements)
        .map_err(|error| {
            domain_fail(format!(
                "native-handler registry audit failed: {error}; handler={:?}",
                error.handler().map(NativeHandlerId::get)
            ))
        })?;
    Ok(requirements
        .into_iter()
        .filter(|requirement| requirement.enabled)
        .map(|requirement| requirement.id)
        .collect())
}

pub(super) fn handler_id(value: i32) -> Result<NativeHandlerId, CatalogLoadError> {
    u32::try_from(value)
        .ok()
        .and_then(NativeHandlerId::new)
        .ok_or_else(|| domain_fail("native-handler ID must be positive"))
}

fn digest(value: &str) -> Result<[u8; 32], CatalogLoadError> {
    if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(domain_fail(
            "native-handler argument schema digest must be 64 hexadecimal characters",
        ));
    }
    let mut output = [0_u8; 32];
    for (index, pair) in value.as_bytes().chunks_exact(2).enumerate() {
        output[index] = (nibble(pair[0])? << 4) | nibble(pair[1])?;
    }
    Ok(output)
}

fn nibble(value: u8) -> Result<u8, CatalogLoadError> {
    match value {
        b'0'..=b'9' => Ok(value - b'0'),
        b'a'..=b'f' => Ok(value - b'a' + 10),
        b'A'..=b'F' => Ok(value - b'A' + 10),
        _ => Err(domain_fail("native-handler digest contains non-hex data")),
    }
}
