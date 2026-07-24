//! Standard Universe runtime facade over the generic graph Activity.
mod battle_contribution_access;
mod battle_execution_access;

use std::sync::Arc;

use starclock_activity::{
    ActivityDecisionId, ActivityExternalOutcomeId, ActivityInventoryId, ActivityOptionId,
    ActivityPlayerView, ActivityPreparationBoundary, ActivityPreparationView, ActivityRosterLock,
    ActivityScopePath, ActivitySlotId, ActivityStateHash, ActivityValue, AttemptId, BattleSequence,
    GraphActivity, GraphActivityCommandError, GraphActivityEncounterError,
    GraphActivityPreparationResolution, GraphActivityResolution, GraphActivityStartError,
    ParticipantLock,
};

use crate::{
    ability_runtime::{
        AbilityActivityProjection, AbilityExecutionContext, AbilityRuntimeCatalog,
        AbilityRuntimeError, AbilityRuntimeProjection,
    },
    abundance_runtime::AbundanceRuntimeCatalog,
    battle_contribution::{UniverseBattleContributionCompiler, UniverseBattleContributionError},
    battle_overlay::UniverseEncounterOverlay,
    blessing_runtime::{BlessingContributionSet, BlessingRuntimeCatalog, BlessingRuntimeError},
    curio_activity::{
        CurioActivityProjection, CurioActivityProjectionError, lower_effects as lower_curio_effects,
    },
    curio_effect_runtime::{
        AppliedCurioEffect, CurioEffectFacts, CurioEffectRuntimeCatalog, CurioEffectRuntimeError,
        CurioEvent,
    },
    curio_runtime::{CurioContributionSet, CurioRuntimeCatalog, CurioRuntimeError},
    destruction_runtime::DestructionRuntimeCatalog,
    elation_runtime::ElationRuntimeCatalog,
    erudition_runtime::EruditionRuntimeCatalog,
    hunt_runtime::HuntRuntimeCatalog,
    id::{AbilityTreeNodeId, CurioId, OccurrenceChoiceId, PathId, ResonanceId, ServiceId},
    negative_curio_runtime::{
        NegativeCurioEvent, NegativeCurioRuntimeCatalog, NegativeCurioRuntimeError,
    },
    nihility_runtime::NihilityRuntimeCatalog,
    occurrence_effect_runtime::{
        AppliedOccurrenceEffect, OccurrenceEffectRuntimeCatalog, OccurrenceEffectRuntimeError,
    },
    path_effect_runtime::{
        AppliedPathEffect, PathBattleEvent, PathEffectFacts, PathEffectRuntimeError,
    },
    path_runtime::{PathContributionSet, PathRuntimeCatalog, PathRuntimeError},
    preservation_runtime::PreservationRuntimeCatalog,
    propagation_runtime::PropagationRuntimeCatalog,
    remembrance_runtime::RemembranceRuntimeCatalog,
    run_runtime::{
        AbilityTreeContributionSet, CosmicFragments, RunRuntimeCatalog, RunRuntimeError,
    },
    service_effect_runtime::{
        AppliedServiceEffect, ServiceEffectRuntimeCatalog, ServiceEffectRuntimeError,
    },
    topology::EncounterOptionBinding,
};

pub struct StandardUniverseActivity {
    graph: GraphActivity,
    participants: Arc<ParticipantLock>,
    encounter_options: Arc<[EncounterOptionBinding]>,
    overlay: Arc<UniverseEncounterOverlay>,
    blessing_runtime: Arc<BlessingRuntimeCatalog>,
    path_runtime: Arc<PathRuntimeCatalog>,
    preservation_runtime: Arc<PreservationRuntimeCatalog>,
    remembrance_runtime: Arc<RemembranceRuntimeCatalog>,
    nihility_runtime: Arc<NihilityRuntimeCatalog>,
    abundance_runtime: Arc<AbundanceRuntimeCatalog>,
    hunt_runtime: Arc<HuntRuntimeCatalog>,
    destruction_runtime: Arc<DestructionRuntimeCatalog>,
    elation_runtime: Arc<ElationRuntimeCatalog>,
    propagation_runtime: Arc<PropagationRuntimeCatalog>,
    erudition_runtime: Arc<EruditionRuntimeCatalog>,
    curio_runtime: Arc<CurioRuntimeCatalog>,
    curio_effect_runtime: Arc<CurioEffectRuntimeCatalog>,
    negative_curio_runtime: Arc<NegativeCurioRuntimeCatalog>,
    run_runtime: Arc<RunRuntimeCatalog>,
    occurrence_effect_runtime: Arc<OccurrenceEffectRuntimeCatalog>,
    service_effect_runtime: Arc<ServiceEffectRuntimeCatalog>,
    ability_runtime: Arc<AbilityRuntimeCatalog>,
    battle_contribution_compiler: Arc<UniverseBattleContributionCompiler>,
    ability_tree: Box<[AbilityTreeNodeId]>,
    blessing_inventory: ActivityInventoryId,
    formation_inventory: ActivityInventoryId,
    curio_inventory: ActivityInventoryId,
    curio_state_slot: ActivitySlotId,
    curio_charge_slot: ActivitySlotId,
    curio_event_slot: ActivitySlotId,
    cosmic_fragments_slot: ActivitySlotId,
    selected_path_slot: ActivitySlotId,
    ability_projection_slot: ActivitySlotId,
}

pub(crate) struct StandardUniverseRuntimeContext {
    pub(crate) participants: Arc<ParticipantLock>,
    pub(crate) encounter_options: Arc<[EncounterOptionBinding]>,
    pub(crate) overlay: Arc<UniverseEncounterOverlay>,
    pub(crate) blessing_runtime: Arc<BlessingRuntimeCatalog>,
    pub(crate) path_runtime: Arc<PathRuntimeCatalog>,
    pub(crate) preservation_runtime: Arc<PreservationRuntimeCatalog>,
    pub(crate) remembrance_runtime: Arc<RemembranceRuntimeCatalog>,
    pub(crate) nihility_runtime: Arc<NihilityRuntimeCatalog>,
    pub(crate) abundance_runtime: Arc<AbundanceRuntimeCatalog>,
    pub(crate) hunt_runtime: Arc<HuntRuntimeCatalog>,
    pub(crate) destruction_runtime: Arc<DestructionRuntimeCatalog>,
    pub(crate) elation_runtime: Arc<ElationRuntimeCatalog>,
    pub(crate) propagation_runtime: Arc<PropagationRuntimeCatalog>,
    pub(crate) erudition_runtime: Arc<EruditionRuntimeCatalog>,
    pub(crate) curio_runtime: Arc<CurioRuntimeCatalog>,
    pub(crate) curio_effect_runtime: Arc<CurioEffectRuntimeCatalog>,
    pub(crate) negative_curio_runtime: Arc<NegativeCurioRuntimeCatalog>,
    pub(crate) run_runtime: Arc<RunRuntimeCatalog>,
    pub(crate) occurrence_effect_runtime: Arc<OccurrenceEffectRuntimeCatalog>,
    pub(crate) service_effect_runtime: Arc<ServiceEffectRuntimeCatalog>,
    pub(crate) ability_runtime: Arc<AbilityRuntimeCatalog>,
    pub(crate) battle_contribution_compiler: Arc<UniverseBattleContributionCompiler>,
    pub(crate) ability_tree: Box<[AbilityTreeNodeId]>,
    pub(crate) blessing_inventory: ActivityInventoryId,
    pub(crate) formation_inventory: ActivityInventoryId,
    pub(crate) curio_inventory: ActivityInventoryId,
    pub(crate) curio_state_slot: ActivitySlotId,
    pub(crate) curio_charge_slot: ActivitySlotId,
    pub(crate) curio_event_slot: ActivitySlotId,
    pub(crate) cosmic_fragments_slot: ActivitySlotId,
    pub(crate) selected_path_slot: ActivitySlotId,
    pub(crate) ability_projection_slot: ActivitySlotId,
}

impl StandardUniverseActivity {
    pub(crate) fn new(graph: GraphActivity, context: StandardUniverseRuntimeContext) -> Self {
        Self {
            graph,
            participants: context.participants,
            encounter_options: context.encounter_options,
            overlay: context.overlay,
            blessing_runtime: context.blessing_runtime,
            path_runtime: context.path_runtime,
            preservation_runtime: context.preservation_runtime,
            remembrance_runtime: context.remembrance_runtime,
            nihility_runtime: context.nihility_runtime,
            abundance_runtime: context.abundance_runtime,
            hunt_runtime: context.hunt_runtime,
            destruction_runtime: context.destruction_runtime,
            elation_runtime: context.elation_runtime,
            propagation_runtime: context.propagation_runtime,
            erudition_runtime: context.erudition_runtime,
            curio_runtime: context.curio_runtime,
            curio_effect_runtime: context.curio_effect_runtime,
            negative_curio_runtime: context.negative_curio_runtime,
            run_runtime: context.run_runtime,
            occurrence_effect_runtime: context.occurrence_effect_runtime,
            service_effect_runtime: context.service_effect_runtime,
            ability_runtime: context.ability_runtime,
            battle_contribution_compiler: context.battle_contribution_compiler,
            ability_tree: context.ability_tree,
            blessing_inventory: context.blessing_inventory,
            formation_inventory: context.formation_inventory,
            curio_inventory: context.curio_inventory,
            curio_state_slot: context.curio_state_slot,
            curio_charge_slot: context.curio_charge_slot,
            curio_event_slot: context.curio_event_slot,
            cosmic_fragments_slot: context.cosmic_fragments_slot,
            selected_path_slot: context.selected_path_slot,
            ability_projection_slot: context.ability_projection_slot,
        }
    }

    #[must_use]
    pub const fn graph(&self) -> &GraphActivity {
        &self.graph
    }
    #[must_use]
    pub fn view(&self) -> ActivityPlayerView {
        self.graph.player_view()
    }
    #[must_use]
    pub fn preparation_view(&self) -> Option<ActivityPreparationView> {
        self.graph.preparation_view()
    }

    pub fn blessing_contributions(&self) -> Result<BlessingContributionSet, BlessingRuntimeError> {
        let view = self.graph.player_view();
        let inventory = view
            .inventories()
            .iter()
            .find(|inventory| inventory.id() == self.blessing_inventory)
            .expect("compiled Standard Universe state contains Blessing inventory");
        self.blessing_runtime.contributions(inventory)
    }

    pub fn path_contributions(
        &self,
    ) -> Result<PathContributionSet, StandardUniversePathContributionError> {
        let view = self.graph.player_view();
        let selected = view
            .slots()
            .iter()
            .find(|slot| slot.id() == self.selected_path_slot)
            .and_then(|slot| match slot.value() {
                ActivityValue::OptionalId(Some(raw)) => u32::try_from(*raw).ok(),
                _ => None,
            })
            .and_then(PathId::new)
            .ok_or(StandardUniversePathContributionError::PathNotSelected)?;
        let blessing_inventory = view
            .inventories()
            .iter()
            .find(|inventory| inventory.id() == self.blessing_inventory)
            .ok_or(StandardUniversePathContributionError::MissingInventory)?;
        let formation_inventory = view
            .inventories()
            .iter()
            .find(|inventory| inventory.id() == self.formation_inventory)
            .ok_or(StandardUniversePathContributionError::MissingInventory)?;
        let blessings = self
            .blessing_runtime
            .contributions(blessing_inventory)
            .map_err(StandardUniversePathContributionError::Blessing)?;
        let formations = formation_inventory
            .entries()
            .iter()
            .map(|(raw, stacks)| {
                let raw = u32::try_from(*raw)
                    .map_err(|_| StandardUniversePathContributionError::UnknownFormation(*raw))?;
                let id = ResonanceId::new(raw).ok_or(
                    StandardUniversePathContributionError::UnknownFormation(u64::from(raw)),
                )?;
                Ok((id, *stacks))
            })
            .collect::<Result<Vec<_>, StandardUniversePathContributionError>>()?;
        self.path_runtime
            .contributions(selected, &blessings, &formations)
            .map_err(StandardUniversePathContributionError::Path)
    }

    /// Executes every owned Preservation Blessing plus the currently available
    /// Preservation Resonance/Formations for one battle observation.
    pub fn preservation_effects(
        &self,
        event: PathBattleEvent,
        facts: PathEffectFacts,
    ) -> Result<Box<[AppliedPathEffect]>, StandardUniversePreservationError> {
        let view = self.graph.player_view();
        let blessing_inventory = view
            .inventories()
            .iter()
            .find(|inventory| inventory.id() == self.blessing_inventory)
            .ok_or(StandardUniversePreservationError::MissingInventory)?;
        let mut effects = Vec::new();
        for (raw, level) in blessing_inventory.entries() {
            let Ok(raw) = u32::try_from(*raw) else {
                continue;
            };
            let Some(id) = crate::id::BlessingId::new(raw) else {
                continue;
            };
            if !self
                .preservation_runtime
                .blessing_ids()
                .any(|candidate| candidate == id)
            {
                continue;
            }
            let level = u8::try_from(*level)
                .map_err(|_| StandardUniversePreservationError::InvalidLevel)?;
            effects.extend(
                self.preservation_runtime
                    .execute_blessing(id, level, event, facts)
                    .map_err(StandardUniversePreservationError::Effect)?,
            );
        }

        let path = self
            .path_contributions()
            .map_err(StandardUniversePreservationError::Path)?;
        if path.passive().path() == self.preservation_runtime.path() {
            if let Some(resonance) = path.resonance() {
                effects.extend(
                    self.preservation_runtime
                        .execute_resonance(resonance.id(), event, facts)
                        .map_err(StandardUniversePreservationError::Effect)?,
                );
            }
            for formation in path.formations() {
                effects.extend(
                    self.preservation_runtime
                        .execute_resonance(formation.id(), event, facts)
                        .map_err(StandardUniversePreservationError::Effect)?,
                );
            }
        }
        Ok(effects.into_boxed_slice())
    }

    /// Executes every owned Remembrance Blessing plus the currently available
    /// Remembrance Resonance/Formations for one battle observation.
    pub fn remembrance_effects(
        &self,
        event: PathBattleEvent,
        facts: PathEffectFacts,
    ) -> Result<Box<[AppliedPathEffect]>, StandardUniverseRemembranceError> {
        let view = self.graph.player_view();
        let blessing_inventory = view
            .inventories()
            .iter()
            .find(|inventory| inventory.id() == self.blessing_inventory)
            .ok_or(StandardUniverseRemembranceError::MissingInventory)?;
        let mut effects = Vec::new();
        for (raw, level) in blessing_inventory.entries() {
            let Ok(raw) = u32::try_from(*raw) else {
                continue;
            };
            let Some(id) = crate::id::BlessingId::new(raw) else {
                continue;
            };
            if !self
                .remembrance_runtime
                .blessing_ids()
                .any(|candidate| candidate == id)
            {
                continue;
            }
            let level =
                u8::try_from(*level).map_err(|_| StandardUniverseRemembranceError::InvalidLevel)?;
            effects.extend(
                self.remembrance_runtime
                    .execute_blessing(id, level, event, facts)
                    .map_err(StandardUniverseRemembranceError::Effect)?,
            );
        }

        let path = self
            .path_contributions()
            .map_err(StandardUniverseRemembranceError::Path)?;
        if path.passive().path() == self.remembrance_runtime.path() {
            if let Some(resonance) = path.resonance() {
                effects.extend(
                    self.remembrance_runtime
                        .execute_resonance(resonance.id(), event, facts)
                        .map_err(StandardUniverseRemembranceError::Effect)?,
                );
            }
            for formation in path.formations() {
                effects.extend(
                    self.remembrance_runtime
                        .execute_resonance(formation.id(), event, facts)
                        .map_err(StandardUniverseRemembranceError::Effect)?,
                );
            }
        }
        Ok(effects.into_boxed_slice())
    }

    /// Executes every owned Nihility Blessing plus the currently available
    /// Nihility Resonance/Formations for one battle observation.
    pub fn nihility_effects(
        &self,
        event: PathBattleEvent,
        facts: PathEffectFacts,
    ) -> Result<Box<[AppliedPathEffect]>, StandardUniverseNihilityError> {
        let view = self.graph.player_view();
        let blessing_inventory = view
            .inventories()
            .iter()
            .find(|inventory| inventory.id() == self.blessing_inventory)
            .ok_or(StandardUniverseNihilityError::MissingInventory)?;
        let mut effects = Vec::new();
        for (raw, level) in blessing_inventory.entries() {
            let Ok(raw) = u32::try_from(*raw) else {
                continue;
            };
            let Some(id) = crate::id::BlessingId::new(raw) else {
                continue;
            };
            if !self
                .nihility_runtime
                .blessing_ids()
                .any(|candidate| candidate == id)
            {
                continue;
            }
            let level =
                u8::try_from(*level).map_err(|_| StandardUniverseNihilityError::InvalidLevel)?;
            effects.extend(
                self.nihility_runtime
                    .execute_blessing(id, level, event, facts)
                    .map_err(StandardUniverseNihilityError::Effect)?,
            );
        }
        let path = self
            .path_contributions()
            .map_err(StandardUniverseNihilityError::Path)?;
        if path.passive().path() == self.nihility_runtime.path() {
            if let Some(resonance) = path.resonance() {
                effects.extend(
                    self.nihility_runtime
                        .execute_resonance(resonance.id(), event, facts)
                        .map_err(StandardUniverseNihilityError::Effect)?,
                );
            }
            for formation in path.formations() {
                effects.extend(
                    self.nihility_runtime
                        .execute_resonance(formation.id(), event, facts)
                        .map_err(StandardUniverseNihilityError::Effect)?,
                );
            }
        }
        Ok(effects.into_boxed_slice())
    }

    /// Executes every owned Abundance Blessing plus the currently available
    /// Abundance Resonance/Formations for one battle observation.
    pub fn abundance_effects(
        &self,
        event: PathBattleEvent,
        facts: PathEffectFacts,
    ) -> Result<Box<[AppliedPathEffect]>, StandardUniverseAbundanceError> {
        let view = self.graph.player_view();
        let blessing_inventory = view
            .inventories()
            .iter()
            .find(|inventory| inventory.id() == self.blessing_inventory)
            .ok_or(StandardUniverseAbundanceError::MissingInventory)?;
        let mut effects = Vec::new();
        for (raw, level) in blessing_inventory.entries() {
            let Ok(raw) = u32::try_from(*raw) else {
                continue;
            };
            let Some(id) = crate::id::BlessingId::new(raw) else {
                continue;
            };
            if !self
                .abundance_runtime
                .blessing_ids()
                .any(|candidate| candidate == id)
            {
                continue;
            }
            let level =
                u8::try_from(*level).map_err(|_| StandardUniverseAbundanceError::InvalidLevel)?;
            effects.extend(
                self.abundance_runtime
                    .execute_blessing(id, level, event, facts)
                    .map_err(StandardUniverseAbundanceError::Effect)?,
            );
        }
        let path = self
            .path_contributions()
            .map_err(StandardUniverseAbundanceError::Path)?;
        if path.passive().path() == self.abundance_runtime.path() {
            if let Some(resonance) = path.resonance() {
                effects.extend(
                    self.abundance_runtime
                        .execute_resonance(resonance.id(), event, facts)
                        .map_err(StandardUniverseAbundanceError::Effect)?,
                );
            }
            for formation in path.formations() {
                effects.extend(
                    self.abundance_runtime
                        .execute_resonance(formation.id(), event, facts)
                        .map_err(StandardUniverseAbundanceError::Effect)?,
                );
            }
        }
        Ok(effects.into_boxed_slice())
    }

    pub fn hunt_effects(
        &self,
        event: PathBattleEvent,
        facts: PathEffectFacts,
    ) -> Result<Box<[AppliedPathEffect]>, StandardUniverseHuntError> {
        let view = self.graph.player_view();
        let inventory = view
            .inventories()
            .iter()
            .find(|inventory| inventory.id() == self.blessing_inventory)
            .ok_or(StandardUniverseHuntError::MissingInventory)?;
        let mut effects = Vec::new();
        for (raw, level) in inventory.entries() {
            let Ok(raw) = u32::try_from(*raw) else {
                continue;
            };
            let Some(id) = crate::id::BlessingId::new(raw) else {
                continue;
            };
            if !self
                .hunt_runtime
                .blessing_ids()
                .any(|candidate| candidate == id)
            {
                continue;
            }
            let level =
                u8::try_from(*level).map_err(|_| StandardUniverseHuntError::InvalidLevel)?;
            effects.extend(
                self.hunt_runtime
                    .execute_blessing(id, level, event, facts)
                    .map_err(StandardUniverseHuntError::Effect)?,
            );
        }
        let path = self
            .path_contributions()
            .map_err(StandardUniverseHuntError::Path)?;
        if path.passive().path() == self.hunt_runtime.path() {
            if let Some(resonance) = path.resonance() {
                effects.extend(
                    self.hunt_runtime
                        .execute_resonance(resonance.id(), event, facts)
                        .map_err(StandardUniverseHuntError::Effect)?,
                );
            }
            for formation in path.formations() {
                effects.extend(
                    self.hunt_runtime
                        .execute_resonance(formation.id(), event, facts)
                        .map_err(StandardUniverseHuntError::Effect)?,
                );
            }
        }
        Ok(effects.into_boxed_slice())
    }

    pub fn destruction_effects(
        &self,
        event: PathBattleEvent,
        facts: PathEffectFacts,
    ) -> Result<Box<[AppliedPathEffect]>, StandardUniverseDestructionError> {
        let view = self.graph.player_view();
        let inventory = view
            .inventories()
            .iter()
            .find(|inventory| inventory.id() == self.blessing_inventory)
            .ok_or(StandardUniverseDestructionError::MissingInventory)?;
        let mut effects = Vec::new();
        for (raw, level) in inventory.entries() {
            let Ok(raw) = u32::try_from(*raw) else {
                continue;
            };
            let Some(id) = crate::id::BlessingId::new(raw) else {
                continue;
            };
            if !self
                .destruction_runtime
                .blessing_ids()
                .any(|candidate| candidate == id)
            {
                continue;
            }
            let level =
                u8::try_from(*level).map_err(|_| StandardUniverseDestructionError::InvalidLevel)?;
            effects.extend(
                self.destruction_runtime
                    .execute_blessing(id, level, event, facts)
                    .map_err(StandardUniverseDestructionError::Effect)?,
            );
        }
        let path = self
            .path_contributions()
            .map_err(StandardUniverseDestructionError::Path)?;
        if path.passive().path() == self.destruction_runtime.path() {
            if let Some(resonance) = path.resonance() {
                effects.extend(
                    self.destruction_runtime
                        .execute_resonance(resonance.id(), event, facts)
                        .map_err(StandardUniverseDestructionError::Effect)?,
                );
            }
            for formation in path.formations() {
                effects.extend(
                    self.destruction_runtime
                        .execute_resonance(formation.id(), event, facts)
                        .map_err(StandardUniverseDestructionError::Effect)?,
                );
            }
        }
        Ok(effects.into_boxed_slice())
    }

    pub fn elation_effects(
        &self,
        event: PathBattleEvent,
        facts: PathEffectFacts,
    ) -> Result<Box<[AppliedPathEffect]>, StandardUniverseElationError> {
        let view = self.graph.player_view();
        let inventory = view
            .inventories()
            .iter()
            .find(|inventory| inventory.id() == self.blessing_inventory)
            .ok_or(StandardUniverseElationError::MissingInventory)?;
        let mut effects = Vec::new();
        for (raw, level) in inventory.entries() {
            let Ok(raw) = u32::try_from(*raw) else {
                continue;
            };
            let Some(id) = crate::id::BlessingId::new(raw) else {
                continue;
            };
            if !self
                .elation_runtime
                .blessing_ids()
                .any(|candidate| candidate == id)
            {
                continue;
            }
            let level =
                u8::try_from(*level).map_err(|_| StandardUniverseElationError::InvalidLevel)?;
            effects.extend(
                self.elation_runtime
                    .execute_blessing(id, level, event, facts)
                    .map_err(StandardUniverseElationError::Effect)?,
            );
        }
        let path = self
            .path_contributions()
            .map_err(StandardUniverseElationError::Path)?;
        if path.passive().path() == self.elation_runtime.path() {
            if let Some(resonance) = path.resonance() {
                effects.extend(
                    self.elation_runtime
                        .execute_resonance(resonance.id(), event, facts)
                        .map_err(StandardUniverseElationError::Effect)?,
                );
            }
            for formation in path.formations() {
                effects.extend(
                    self.elation_runtime
                        .execute_resonance(formation.id(), event, facts)
                        .map_err(StandardUniverseElationError::Effect)?,
                );
            }
        }
        Ok(effects.into_boxed_slice())
    }

    pub fn propagation_effects(
        &self,
        event: PathBattleEvent,
        facts: PathEffectFacts,
    ) -> Result<Box<[AppliedPathEffect]>, StandardUniversePropagationError> {
        let view = self.graph.player_view();
        let inventory = view
            .inventories()
            .iter()
            .find(|inventory| inventory.id() == self.blessing_inventory)
            .ok_or(StandardUniversePropagationError::MissingInventory)?;
        let mut effects = Vec::new();
        for (raw, level) in inventory.entries() {
            let Ok(raw) = u32::try_from(*raw) else {
                continue;
            };
            let Some(id) = crate::id::BlessingId::new(raw) else {
                continue;
            };
            if !self
                .propagation_runtime
                .blessing_ids()
                .any(|candidate| candidate == id)
            {
                continue;
            }
            let level =
                u8::try_from(*level).map_err(|_| StandardUniversePropagationError::InvalidLevel)?;
            effects.extend(
                self.propagation_runtime
                    .execute_blessing(id, level, event, facts)
                    .map_err(StandardUniversePropagationError::Effect)?,
            );
        }
        let path = self
            .path_contributions()
            .map_err(StandardUniversePropagationError::Path)?;
        if path.passive().path() == self.propagation_runtime.path() {
            if let Some(resonance) = path.resonance() {
                effects.extend(
                    self.propagation_runtime
                        .execute_resonance(resonance.id(), event, facts)
                        .map_err(StandardUniversePropagationError::Effect)?,
                );
            }
            for formation in path.formations() {
                effects.extend(
                    self.propagation_runtime
                        .execute_resonance(formation.id(), event, facts)
                        .map_err(StandardUniversePropagationError::Effect)?,
                );
            }
        }
        Ok(effects.into_boxed_slice())
    }

    pub fn erudition_effects(
        &self,
        event: PathBattleEvent,
        facts: PathEffectFacts,
    ) -> Result<Box<[AppliedPathEffect]>, StandardUniverseEruditionError> {
        let view = self.graph.player_view();
        let inventory = view
            .inventories()
            .iter()
            .find(|inventory| inventory.id() == self.blessing_inventory)
            .ok_or(StandardUniverseEruditionError::MissingInventory)?;
        let mut effects = Vec::new();
        for (raw, level) in inventory.entries() {
            let Ok(raw) = u32::try_from(*raw) else {
                continue;
            };
            let Some(id) = crate::id::BlessingId::new(raw) else {
                continue;
            };
            if !self
                .erudition_runtime
                .blessing_ids()
                .any(|candidate| candidate == id)
            {
                continue;
            }
            let level =
                u8::try_from(*level).map_err(|_| StandardUniverseEruditionError::InvalidLevel)?;
            effects.extend(
                self.erudition_runtime
                    .execute_blessing(id, level, event, facts)
                    .map_err(StandardUniverseEruditionError::Effect)?,
            );
        }
        let path = self
            .path_contributions()
            .map_err(StandardUniverseEruditionError::Path)?;
        if path.passive().path() == self.erudition_runtime.path() {
            if let Some(resonance) = path.resonance() {
                effects.extend(
                    self.erudition_runtime
                        .execute_resonance(resonance.id(), event, facts)
                        .map_err(StandardUniverseEruditionError::Effect)?,
                );
            }
            for formation in path.formations() {
                effects.extend(
                    self.erudition_runtime
                        .execute_resonance(formation.id(), event, facts)
                        .map_err(StandardUniverseEruditionError::Effect)?,
                );
            }
        }
        Ok(effects.into_boxed_slice())
    }

    pub fn curio_contributions(&self) -> Result<CurioContributionSet, CurioRuntimeError> {
        let view = self.graph.player_view();
        let inventory = view
            .inventories()
            .iter()
            .find(|inventory| inventory.id() == self.curio_inventory)
            .ok_or(CurioRuntimeError::MissingInventory)?;
        let state = view
            .slots()
            .iter()
            .find(|slot| slot.id() == self.curio_state_slot)
            .ok_or(CurioRuntimeError::InvalidStateSlot)?;
        let charges = view
            .slots()
            .iter()
            .find(|slot| slot.id() == self.curio_charge_slot)
            .ok_or(CurioRuntimeError::InvalidChargeSlot)?;
        self.curio_runtime.contributions(inventory, state, charges)
    }

    pub fn curio_effects(
        &self,
        event: CurioEvent,
        facts: CurioEffectFacts,
    ) -> Result<Box<[AppliedCurioEffect]>, StandardUniverseCurioEffectError> {
        let contributions = self
            .curio_contributions()
            .map_err(StandardUniverseCurioEffectError::Contribution)?;
        let mut effects = Vec::new();
        for contribution in contributions.entries() {
            if !self
                .curio_effect_runtime
                .curio_ids()
                .any(|candidate| candidate == contribution.curio())
            {
                continue;
            }
            effects.extend(
                self.curio_effect_runtime
                    .execute(contribution.curio(), event, facts)
                    .map_err(StandardUniverseCurioEffectError::Effect)?,
            );
        }
        Ok(effects.into_boxed_slice())
    }

    pub fn curio_activity_projection(
        &self,
        curio: CurioId,
        event: CurioEvent,
        mut facts: CurioEffectFacts,
    ) -> Result<CurioActivityProjection, StandardUniverseCurioActivityError> {
        if !self
            .curio_contributions()
            .map_err(StandardUniverseCurioActivityError::Contribution)?
            .entries()
            .iter()
            .any(|contribution| contribution.curio() == curio)
        {
            return Err(StandardUniverseCurioActivityError::NotOwned);
        }
        let fragments = self
            .cosmic_fragments()
            .map_err(StandardUniverseCurioActivityError::Fragments)?;
        facts.cosmic_fragments = u32::try_from(fragments.get()).map_err(|_| {
            StandardUniverseCurioActivityError::Fragments(RunRuntimeError::InvalidFragmentAmount)
        })?;
        let effects = self
            .curio_effect_runtime
            .execute(curio, event, facts)
            .map_err(StandardUniverseCurioActivityError::Effect)?;
        lower_curio_effects(
            curio,
            event,
            &effects,
            facts.cosmic_fragments,
            self.cosmic_fragments_slot,
            self.curio_event_slot,
        )
        .map_err(StandardUniverseCurioActivityError::Projection)
    }

    pub fn negative_curio_effects(
        &self,
        event: NegativeCurioEvent,
    ) -> Result<Box<[AppliedCurioEffect]>, StandardUniverseCurioEffectError> {
        let contributions = self
            .curio_contributions()
            .map_err(StandardUniverseCurioEffectError::Contribution)?;
        let mut effects = Vec::new();
        for contribution in contributions.entries() {
            if !self
                .negative_curio_runtime
                .contains_curio(contribution.curio())
            {
                continue;
            }
            effects.extend(
                self.negative_curio_runtime
                    .execute(contribution, event)
                    .map_err(StandardUniverseCurioEffectError::NegativeEffect)?,
            );
        }
        Ok(effects.into_boxed_slice())
    }

    pub fn ability_tree_contributions(
        &self,
    ) -> Result<AbilityTreeContributionSet, RunRuntimeError> {
        self.run_runtime.ability_contributions(&self.ability_tree)
    }

    pub fn occurrence_effect(
        &self,
        choice: OccurrenceChoiceId,
    ) -> Result<AppliedOccurrenceEffect, OccurrenceEffectRuntimeError> {
        self.occurrence_effect_runtime.execute(choice)
    }

    pub fn service_effect(
        &self,
        service: ServiceId,
    ) -> Result<AppliedServiceEffect, ServiceEffectRuntimeError> {
        self.service_effect_runtime.execute(service)
    }

    pub fn ability_tree_projection(
        &self,
        context: AbilityExecutionContext,
    ) -> Result<AbilityRuntimeProjection, AbilityRuntimeError> {
        self.ability_runtime.project(&self.ability_tree, context)
    }

    pub fn ability_activity_projection(
        &self,
        context: AbilityExecutionContext,
    ) -> Result<AbilityActivityProjection, AbilityRuntimeError> {
        self.ability_runtime.project_activity_operations(
            &self.ability_tree,
            context,
            self.ability_projection_slot,
        )
    }

    pub fn cosmic_fragments(&self) -> Result<CosmicFragments, RunRuntimeError> {
        let view = self.graph.player_view();
        let value = view
            .slots()
            .iter()
            .find(|slot| slot.id() == self.cosmic_fragments_slot)
            .and_then(|slot| match slot.value() {
                ActivityValue::BoundedInteger(value) => Some(*value),
                _ => None,
            })
            .ok_or(RunRuntimeError::InvalidFragmentAmount)?;
        CosmicFragments::new(value)
    }

    pub fn reroll_blessing_offer(
        &mut self,
        expected_state_hash: ActivityStateHash,
    ) -> Result<
        Box<[starclock_activity::ActivityTransactionEvent]>,
        starclock_activity::GraphActivityRandomOfferError,
    > {
        self.graph.reroll_random_offer(expected_state_hash)
    }

    pub fn choose_option(
        &mut self,
        expected_state_hash: ActivityStateHash,
        decision: ActivityDecisionId,
        option: ActivityOptionId,
    ) -> Result<Box<[starclock_activity::ActivityTransactionEvent]>, GraphActivityCommandError>
    {
        self.graph
            .choose_option(expected_state_hash, decision, option)
    }

    pub fn submit_external_outcome(
        &mut self,
        expected_state_hash: ActivityStateHash,
        decision: ActivityDecisionId,
        outcome: ActivityExternalOutcomeId,
    ) -> Result<Box<[starclock_activity::ActivityTransactionEvent]>, GraphActivityCommandError>
    {
        self.graph
            .submit_external_outcome(expected_state_hash, decision, outcome)
    }

    pub fn engage_encounter(
        &mut self,
        expected_state_hash: ActivityStateHash,
        decision: ActivityDecisionId,
        option: ActivityOptionId,
        technique_points: u16,
    ) -> Result<GraphActivityPreparationResolution, StandardUniverseEncounterError> {
        let member = self
            .encounter_options
            .binary_search_by_key(&option, |binding| binding.option())
            .ok()
            .map(|index| self.encounter_options[index].member())
            .ok_or(StandardUniverseEncounterError::UnknownEncounterOption)?;
        let binding = self
            .overlay
            .binding(member)
            .ok_or(StandardUniverseEncounterError::MissingBattleOverlay(member))?;
        let current = self.graph.current_node();
        let section = self
            .graph
            .definition()
            .graph()
            .node(current)
            .ok_or(StandardUniverseEncounterError::InvalidScope)?
            .section();
        let instance = self.graph.instance();
        let path = ActivityScopePath::new(instance)
            .enter_section(section)
            .and_then(|path| path.enter_node(current))
            .and_then(|path| {
                path.enter_attempt(AttemptId::new(1).expect("static attempt ID is non-zero"))
            })
            .map_err(|_| StandardUniverseEncounterError::InvalidScope)?;
        let roster = ActivityRosterLock::new(
            ActivityScopePath::new(instance),
            self.participants.as_ref().clone(),
        )
        .map_err(|_| StandardUniverseEncounterError::InvalidScope)?;
        let sequence = BattleSequence::new(current.get())
            .ok_or(StandardUniverseEncounterError::InvalidScope)?;
        self.graph
            .engage_encounter(
                expected_state_hash,
                decision,
                option,
                starclock_activity::ActivityBattlePreparationRequest::new(
                    path,
                    roster,
                    sequence,
                    technique_points,
                    Arc::clone(binding.preparation()),
                ),
            )
            .map_err(StandardUniverseEncounterError::Activity)
    }

    pub fn choose_preparation_option(
        &mut self,
        expected_state_hash: ActivityStateHash,
        option: ActivityOptionId,
    ) -> Result<ActivityPreparationBoundary, GraphActivityEncounterError> {
        self.graph
            .choose_preparation_option(expected_state_hash, option)
    }
}

pub struct StandardUniverseStartResolution {
    activity: StandardUniverseActivity,
    events: Box<[starclock_activity::ActivityTransactionEvent]>,
}

impl StandardUniverseStartResolution {
    pub(crate) fn new(
        resolution: GraphActivityResolution,
        context: StandardUniverseRuntimeContext,
    ) -> Self {
        let events = resolution.events().to_vec().into_boxed_slice();
        let activity = StandardUniverseActivity::new(resolution.into_activity(), context);
        Self { activity, events }
    }
    #[must_use]
    pub fn into_activity(self) -> StandardUniverseActivity {
        self.activity
    }
    #[must_use]
    pub fn events(&self) -> &[starclock_activity::ActivityTransactionEvent] {
        &self.events
    }
    #[must_use]
    pub fn view(&self) -> ActivityPlayerView {
        self.activity.view()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StandardUniverseStartError {
    MissingEncounterOverlay,
    Activity(GraphActivityStartError),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StandardUniverseEncounterError {
    UnknownEncounterOption,
    MissingBattleOverlay(crate::id::EncounterMemberId),
    InvalidScope,
    Activity(GraphActivityEncounterError),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StandardUniverseBattleStartError {
    MissingPendingBattle,
    MissingBattleOverlay,
    Activity(starclock_activity::ActivityBattleSettlementError),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StandardUniversePathContributionError {
    PathNotSelected,
    MissingInventory,
    UnknownFormation(u64),
    Blessing(BlessingRuntimeError),
    Path(PathRuntimeError),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StandardUniversePreservationError {
    MissingInventory,
    InvalidLevel,
    Path(StandardUniversePathContributionError),
    Effect(PathEffectRuntimeError),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StandardUniverseRemembranceError {
    MissingInventory,
    InvalidLevel,
    Path(StandardUniversePathContributionError),
    Effect(PathEffectRuntimeError),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StandardUniverseNihilityError {
    MissingInventory,
    InvalidLevel,
    Path(StandardUniversePathContributionError),
    Effect(PathEffectRuntimeError),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StandardUniverseAbundanceError {
    MissingInventory,
    InvalidLevel,
    Path(StandardUniversePathContributionError),
    Effect(PathEffectRuntimeError),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StandardUniverseHuntError {
    MissingInventory,
    InvalidLevel,
    Path(StandardUniversePathContributionError),
    Effect(PathEffectRuntimeError),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StandardUniverseDestructionError {
    MissingInventory,
    InvalidLevel,
    Path(StandardUniversePathContributionError),
    Effect(PathEffectRuntimeError),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StandardUniverseElationError {
    MissingInventory,
    InvalidLevel,
    Path(StandardUniversePathContributionError),
    Effect(PathEffectRuntimeError),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StandardUniversePropagationError {
    MissingInventory,
    InvalidLevel,
    Path(StandardUniversePathContributionError),
    Effect(PathEffectRuntimeError),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StandardUniverseEruditionError {
    MissingInventory,
    InvalidLevel,
    Path(StandardUniversePathContributionError),
    Effect(PathEffectRuntimeError),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StandardUniverseCurioEffectError {
    Contribution(CurioRuntimeError),
    Effect(CurioEffectRuntimeError),
    NegativeEffect(NegativeCurioRuntimeError),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StandardUniverseCurioActivityError {
    NotOwned,
    Contribution(CurioRuntimeError),
    Fragments(RunRuntimeError),
    Effect(CurioEffectRuntimeError),
    Projection(CurioActivityProjectionError),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StandardUniverseBattleContributionError {
    InvalidScope,
    Blessing(BlessingRuntimeError),
    Path(StandardUniversePathContributionError),
    Curio(CurioRuntimeError),
    Ability(RunRuntimeError),
    Projection(AbilityRuntimeError),
    Compile(UniverseBattleContributionError),
}

pub(crate) fn start(
    resolution: Result<GraphActivityResolution, GraphActivityStartError>,
    context: Option<StandardUniverseRuntimeContext>,
) -> Result<StandardUniverseStartResolution, StandardUniverseStartError> {
    let context = context.ok_or(StandardUniverseStartError::MissingEncounterOverlay)?;
    let resolution = resolution.map_err(StandardUniverseStartError::Activity)?;
    Ok(StandardUniverseStartResolution::new(resolution, context))
}
