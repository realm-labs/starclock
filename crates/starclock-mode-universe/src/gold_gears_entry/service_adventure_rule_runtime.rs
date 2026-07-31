//! Terminal dispatch for frozen Service and Adventure rule identities.

use crate::digest::Encoder;

use super::{
    GoldAndGearsEntryError,
    api::{GoldAndGearsRuntimeFactory, GoldAndGearsRuntimeInstance},
    service_adventure_types::{
        GoldAndGearsAdventureDefinition, GoldAndGearsServiceAdventureRuleAccuracy,
        GoldAndGearsServiceAdventureRuleBinding, GoldAndGearsServiceAdventureRuleKind,
        GoldAndGearsServiceDefinition,
    },
};

pub const GOLD_AND_GEARS_SERVICE_ADVENTURE_EXECUTION_REVISION: &str =
    "gold-and-gears-service-adventure-execution-v1";

pub(super) fn compile_rule_runtime(
    services: &[GoldAndGearsServiceDefinition],
    adventures: &[GoldAndGearsAdventureDefinition],
) -> Result<(Box<[GoldAndGearsServiceAdventureRuleBinding]>, [u8; 32]), GoldAndGearsEntryError> {
    let mut bindings = Vec::with_capacity(38);
    for service in services {
        bindings.push(binding(
            service.bridge_rule(),
            service.stable_key(),
            GoldAndGearsServiceAdventureRuleKind::ServiceBridge,
            GoldAndGearsServiceAdventureRuleAccuracy::ExactPublic,
        )?);
        bindings.push(binding(
            service.released_rule(),
            service.stable_key(),
            GoldAndGearsServiceAdventureRuleKind::ReleasedService,
            GoldAndGearsServiceAdventureRuleAccuracy::ExactPublic,
        )?);
    }
    for adventure in adventures {
        bindings.push(binding(
            adventure.rule(),
            adventure.stable_key(),
            GoldAndGearsServiceAdventureRuleKind::AdventureOutcome,
            GoldAndGearsServiceAdventureRuleAccuracy::VersionedProjectPolicy,
        )?);
    }
    bindings.sort_by(|left, right| left.rule_id.cmp(&right.rule_id));
    if bindings.len() != 38
        || bindings
            .windows(2)
            .any(|pair| pair[0].rule_id >= pair[1].rule_id)
        || bindings
            .iter()
            .filter(|binding| {
                binding.accuracy == GoldAndGearsServiceAdventureRuleAccuracy::ExactPublic
            })
            .count()
            != 30
    {
        return Err(GoldAndGearsEntryError::InvalidServiceRuntime);
    }
    let digest = execution_digest(&bindings);
    Ok((bindings.into_boxed_slice(), digest))
}

impl GoldAndGearsRuntimeFactory {
    #[must_use]
    pub fn service_adventure_rule_bindings(&self) -> &[GoldAndGearsServiceAdventureRuleBinding] {
        self.content_runtime.service_adventure.rule_bindings()
    }

    #[must_use]
    pub fn service_adventure_execution_digest(&self) -> [u8; 32] {
        self.content_runtime.service_adventure.execution_digest()
    }
}

impl GoldAndGearsRuntimeInstance {
    #[must_use]
    pub fn service_adventure_rule_bindings(&self) -> &[GoldAndGearsServiceAdventureRuleBinding] {
        self.content_runtime.service_adventure.rule_bindings()
    }

    #[must_use]
    pub fn service_adventure_execution_digest(&self) -> [u8; 32] {
        self.content_runtime.service_adventure.execution_digest()
    }
}

fn binding(
    rule_id: &str,
    owner_id: &str,
    kind: GoldAndGearsServiceAdventureRuleKind,
    accuracy: GoldAndGearsServiceAdventureRuleAccuracy,
) -> Result<GoldAndGearsServiceAdventureRuleBinding, GoldAndGearsEntryError> {
    if rule_id.is_empty()
        || !owner_id.starts_with("universe.") && !owner_id.starts_with("gold-gears.")
    {
        return Err(GoldAndGearsEntryError::InvalidServiceRuntime);
    }
    Ok(GoldAndGearsServiceAdventureRuleBinding {
        rule_id: rule_id.into(),
        owner_id: owner_id.into(),
        kind,
        accuracy,
    })
}

fn execution_digest(bindings: &[GoldAndGearsServiceAdventureRuleBinding]) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock-gold-gears-service-adventure-execution-v1");
    encoder.text(GOLD_AND_GEARS_SERVICE_ADVENTURE_EXECUTION_REVISION);
    encoder.u32(bindings.len() as u32);
    for binding in bindings {
        encoder.text(&binding.rule_id);
        encoder.text(&binding.owner_id);
        encoder.u8(binding.kind as u8);
        encoder.u8(binding.accuracy as u8);
    }
    encoder.finish()
}
