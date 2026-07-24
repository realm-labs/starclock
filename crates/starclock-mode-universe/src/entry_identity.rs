//! Canonical Standard Universe Activity definition identity.

use starclock_activity::{
    ActivityConfigDigest, ActivityDefinitionDigest, ActivityDefinitionId,
    ActivityDefinitionIdentity,
};

use crate::{
    battle_overlay::UniverseEncounterOverlay,
    catalog::UniverseCatalog,
    digest::Encoder,
    entry::{STANDARD_UNIVERSE_ENTRY_REVISION, StandardUniverseCompileError},
    id::{AbilityTreeNodeId, DifficultyId, PathId, WorldId},
};

pub(super) fn compile_identity(
    catalog: &UniverseCatalog,
    world: WorldId,
    difficulty: DifficultyId,
    participant_digest: [u8; 32],
    ability_tree: &[AbilityTreeNodeId],
    path_options: &[PathId],
    encounter_overlay: Option<&UniverseEncounterOverlay>,
) -> Result<ActivityDefinitionIdentity, StandardUniverseCompileError> {
    let catalog_identity = catalog.identity();
    let mut encoder = Encoder::new(b"starclock-standard-universe-entry-definition-v1");
    encoder.text(STANDARD_UNIVERSE_ENTRY_REVISION);
    encoder.text(crate::blessing_runtime::BLESSING_RUNTIME_REVISION);
    encoder.text(crate::path_runtime::PATH_RUNTIME_REVISION);
    encoder.text(crate::preservation_runtime::PRESERVATION_RUNTIME_REVISION);
    encoder.text(crate::remembrance_runtime::REMEMBRANCE_RUNTIME_REVISION);
    encoder.text(crate::nihility_runtime::NIHILITY_RUNTIME_REVISION);
    encoder.text(crate::abundance_runtime::ABUNDANCE_RUNTIME_REVISION);
    encoder.text(crate::hunt_runtime::HUNT_RUNTIME_REVISION);
    encoder.text(crate::destruction_runtime::DESTRUCTION_RUNTIME_REVISION);
    encoder.text(crate::elation_runtime::ELATION_RUNTIME_REVISION);
    encoder.text(crate::propagation_runtime::PROPAGATION_RUNTIME_REVISION);
    encoder.text(crate::erudition_runtime::ERUDITION_RUNTIME_REVISION);
    encoder.text(crate::curio_runtime::CURIO_RUNTIME_REVISION);
    encoder.text(crate::curio_effect_runtime::CURIO_EFFECT_RUNTIME_REVISION);
    encoder.text(crate::negative_curio_runtime::NEGATIVE_CURIO_RUNTIME_REVISION);
    encoder.text(crate::occurrence_effect_runtime::OCCURRENCE_EFFECT_RUNTIME_REVISION);
    encoder.text(crate::occurrence_interaction::OCCURRENCE_INTERACTION_RUNTIME_REVISION);
    encoder.text(crate::service_effect_runtime::SERVICE_EFFECT_RUNTIME_REVISION);
    encoder.text(crate::service_interaction::SERVICE_INTERACTION_RUNTIME_REVISION);
    encoder.text(crate::encounter_content_runtime::ENCOUNTER_CONTENT_RUNTIME_REVISION);
    encoder.text(crate::run_runtime::RUN_RUNTIME_REVISION);
    encoder.text(crate::ability_runtime::ABILITY_RUNTIME_REVISION);
    encoder.digest(catalog_identity.configuration_digest().bytes());
    encoder.digest(catalog_identity.definitions_digest().bytes());
    encoder.digest(catalog_identity.path_definitions_digest().bytes());
    encoder.digest(catalog_identity.run_definitions_digest().bytes());
    encoder.u32(world.get());
    encoder.u32(difficulty.get());
    encoder.digest(participant_digest);
    encoder.u32(ability_tree.len() as u32);
    for node in ability_tree {
        encoder.u32(node.get());
    }
    encoder.u32(path_options.len() as u32);
    for path in path_options {
        encoder.u32(path.get());
    }
    if let Some(overlay) = encounter_overlay {
        encoder.digest(overlay.digest().bytes());
    }
    let definition_digest = ActivityDefinitionDigest::new(encoder.finish())
        .ok_or(StandardUniverseCompileError::InvalidCatalogIdentity)?;
    let config_digest = ActivityConfigDigest::new(catalog_identity.configuration_digest().bytes())
        .ok_or(StandardUniverseCompileError::InvalidCatalogIdentity)?;
    let definition_id = ActivityDefinitionId::new(catalog.activity_binding().id().get())
        .ok_or(StandardUniverseCompileError::InvalidCatalogIdentity)?;
    Ok(ActivityDefinitionIdentity::new(
        definition_id,
        definition_digest,
        config_digest,
    ))
}
