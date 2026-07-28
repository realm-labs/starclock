//! Read-only access to immutable runtime compilers owned by one compiled entry.

use std::sync::Arc;

use starclock_activity::ActivitySlotId;

use crate::{
    ability_runtime::AbilityRuntimeCatalog, abundance_runtime::AbundanceRuntimeCatalog,
    battle_contribution::UniverseBattleContributionCompiler,
    blessing_runtime::BlessingRuntimeCatalog, curio_effect_runtime::CurioEffectRuntimeCatalog,
    curio_runtime::CurioRuntimeCatalog, destruction_runtime::DestructionRuntimeCatalog,
    elation_runtime::ElationRuntimeCatalog,
    encounter_content_runtime::EncounterContentRuntimeCatalog,
    erudition_runtime::EruditionRuntimeCatalog, hunt_runtime::HuntRuntimeCatalog,
    negative_curio_runtime::NegativeCurioRuntimeCatalog, nihility_runtime::NihilityRuntimeCatalog,
    occurrence_effect_runtime::OccurrenceEffectRuntimeCatalog,
    occurrence_interaction::OccurrenceInteractionRuntimeCatalog, path_runtime::PathRuntimeCatalog,
    preservation_runtime::PreservationRuntimeCatalog,
    propagation_runtime::PropagationRuntimeCatalog, remembrance_runtime::RemembranceRuntimeCatalog,
    run_runtime::RunRuntimeCatalog, service_effect_runtime::ServiceEffectRuntimeCatalog,
    service_interaction::ServiceInteractionRuntimeCatalog,
};

use super::{CompiledActivity, slot, state_layout::OCCURRENCE_INTERACTION_STATE_SLOT};

impl CompiledActivity {
    #[must_use]
    pub const fn occurrence_interaction_state_slot(&self) -> ActivitySlotId {
        slot(OCCURRENCE_INTERACTION_STATE_SLOT)
    }

    pub fn initial_run_capabilities(
        &self,
    ) -> Result<
        crate::runtime::ability_access::StandardUniverseRunCapabilities,
        crate::runtime::ability_access::StandardUniverseRunCapabilityError,
    > {
        let projection = self
            .state
            .slots()
            .iter()
            .find(|slot| slot.id() == self.ability_projection_slot())
            .map(|slot| slot.initial())
            .ok_or(
                crate::runtime::ability_access::StandardUniverseRunCapabilityError::MissingAbilityProjection,
            )?;
        let formation_slots = self
            .state
            .slots()
            .iter()
            .find(|slot| slot.id() == self.formation_capability_slot())
            .map(|slot| slot.initial())
            .ok_or(
                crate::runtime::ability_access::StandardUniverseRunCapabilityError::MissingFormationCapability,
            )?;
        crate::runtime::ability_access::capabilities_from_values(projection, formation_slots)
    }

    #[must_use]
    pub const fn blessing_runtime(&self) -> &Arc<BlessingRuntimeCatalog> {
        &self.blessing_runtime
    }

    #[must_use]
    pub const fn path_runtime(&self) -> &Arc<PathRuntimeCatalog> {
        &self.path_runtime
    }

    #[must_use]
    pub const fn preservation_runtime(&self) -> &Arc<PreservationRuntimeCatalog> {
        &self.preservation_runtime
    }

    #[must_use]
    pub const fn remembrance_runtime(&self) -> &Arc<RemembranceRuntimeCatalog> {
        &self.remembrance_runtime
    }

    #[must_use]
    pub const fn nihility_runtime(&self) -> &Arc<NihilityRuntimeCatalog> {
        &self.nihility_runtime
    }

    #[must_use]
    pub const fn abundance_runtime(&self) -> &Arc<AbundanceRuntimeCatalog> {
        &self.abundance_runtime
    }

    #[must_use]
    pub const fn hunt_runtime(&self) -> &Arc<HuntRuntimeCatalog> {
        &self.hunt_runtime
    }

    #[must_use]
    pub const fn destruction_runtime(&self) -> &Arc<DestructionRuntimeCatalog> {
        &self.destruction_runtime
    }

    #[must_use]
    pub const fn elation_runtime(&self) -> &Arc<ElationRuntimeCatalog> {
        &self.elation_runtime
    }

    #[must_use]
    pub const fn propagation_runtime(&self) -> &Arc<PropagationRuntimeCatalog> {
        &self.propagation_runtime
    }

    #[must_use]
    pub const fn erudition_runtime(&self) -> &Arc<EruditionRuntimeCatalog> {
        &self.erudition_runtime
    }

    #[must_use]
    pub const fn curio_runtime(&self) -> &Arc<CurioRuntimeCatalog> {
        &self.curio_runtime
    }

    #[must_use]
    pub const fn curio_effect_runtime(&self) -> &Arc<CurioEffectRuntimeCatalog> {
        &self.curio_effect_runtime
    }

    #[must_use]
    pub const fn negative_curio_runtime(&self) -> &Arc<NegativeCurioRuntimeCatalog> {
        &self.negative_curio_runtime
    }

    #[must_use]
    pub const fn run_runtime(&self) -> &Arc<RunRuntimeCatalog> {
        &self.run_runtime
    }

    #[must_use]
    pub const fn occurrence_effect_runtime(&self) -> &Arc<OccurrenceEffectRuntimeCatalog> {
        &self.occurrence_effect_runtime
    }

    #[must_use]
    pub const fn occurrence_interaction_runtime(
        &self,
    ) -> &Arc<OccurrenceInteractionRuntimeCatalog> {
        &self.occurrence_interaction_runtime
    }

    #[must_use]
    pub const fn service_effect_runtime(&self) -> &Arc<ServiceEffectRuntimeCatalog> {
        &self.service_effect_runtime
    }

    #[must_use]
    pub const fn service_interaction_runtime(&self) -> &Arc<ServiceInteractionRuntimeCatalog> {
        &self.service_interaction_runtime
    }

    #[must_use]
    pub const fn encounter_content_runtime(&self) -> &Arc<EncounterContentRuntimeCatalog> {
        &self.encounter_content_runtime
    }

    #[must_use]
    pub const fn ability_runtime(&self) -> &Arc<AbilityRuntimeCatalog> {
        &self.ability_runtime
    }

    #[must_use]
    pub const fn battle_contribution_compiler(&self) -> &Arc<UniverseBattleContributionCompiler> {
        &self.battle_contribution_compiler
    }
}
