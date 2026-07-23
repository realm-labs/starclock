//! Standard Universe runtime facade over the generic graph Activity.

use std::sync::Arc;

use starclock_activity::{
    ActivityBattleHandoff, ActivityDecisionId, ActivityExternalOutcomeId, ActivityInventoryId,
    ActivityOptionId, ActivityPlayerView, ActivityPreparationBoundary, ActivityPreparationView,
    ActivityRosterLock, ActivityScopePath, ActivitySlotId, ActivityStateHash, ActivityValue,
    AttemptId, BattleResult, BattleSequence, GraphActivity, GraphActivityBattleError,
    GraphActivityBattleResolution, GraphActivityCommandError, GraphActivityEncounterError,
    GraphActivityPreparationResolution, GraphActivityResolution, GraphActivityStartError,
    ParticipantLock,
};

use crate::{
    ability_runtime::{
        AbilityExecutionContext, AbilityRuntimeCatalog, AbilityRuntimeError,
        AbilityRuntimeProjection,
    },
    battle_overlay::UniverseEncounterOverlay,
    blessing_runtime::{BlessingContributionSet, BlessingRuntimeCatalog, BlessingRuntimeError},
    curio_runtime::{CurioContributionSet, CurioRuntimeCatalog, CurioRuntimeError},
    id::{AbilityTreeNodeId, PathId, ResonanceId},
    nihility_runtime::NihilityRuntimeCatalog,
    path_effect_runtime::{
        AppliedPathEffect, PathBattleEvent, PathEffectFacts, PathEffectRuntimeError,
    },
    path_runtime::{PathContributionSet, PathRuntimeCatalog, PathRuntimeError},
    preservation_runtime::PreservationRuntimeCatalog,
    remembrance_runtime::RemembranceRuntimeCatalog,
    run_runtime::{
        AbilityTreeContributionSet, CosmicFragments, RunRuntimeCatalog, RunRuntimeError,
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
    curio_runtime: Arc<CurioRuntimeCatalog>,
    run_runtime: Arc<RunRuntimeCatalog>,
    ability_runtime: Arc<AbilityRuntimeCatalog>,
    ability_tree: Box<[AbilityTreeNodeId]>,
    blessing_inventory: ActivityInventoryId,
    formation_inventory: ActivityInventoryId,
    curio_inventory: ActivityInventoryId,
    curio_state_slot: ActivitySlotId,
    curio_charge_slot: ActivitySlotId,
    cosmic_fragments_slot: ActivitySlotId,
    selected_path_slot: ActivitySlotId,
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
    pub(crate) curio_runtime: Arc<CurioRuntimeCatalog>,
    pub(crate) run_runtime: Arc<RunRuntimeCatalog>,
    pub(crate) ability_runtime: Arc<AbilityRuntimeCatalog>,
    pub(crate) ability_tree: Box<[AbilityTreeNodeId]>,
    pub(crate) blessing_inventory: ActivityInventoryId,
    pub(crate) formation_inventory: ActivityInventoryId,
    pub(crate) curio_inventory: ActivityInventoryId,
    pub(crate) curio_state_slot: ActivitySlotId,
    pub(crate) curio_charge_slot: ActivitySlotId,
    pub(crate) cosmic_fragments_slot: ActivitySlotId,
    pub(crate) selected_path_slot: ActivitySlotId,
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
            curio_runtime: context.curio_runtime,
            run_runtime: context.run_runtime,
            ability_runtime: context.ability_runtime,
            ability_tree: context.ability_tree,
            blessing_inventory: context.blessing_inventory,
            formation_inventory: context.formation_inventory,
            curio_inventory: context.curio_inventory,
            curio_state_slot: context.curio_state_slot,
            curio_charge_slot: context.curio_charge_slot,
            cosmic_fragments_slot: context.cosmic_fragments_slot,
            selected_path_slot: context.selected_path_slot,
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

    pub fn ability_tree_contributions(
        &self,
    ) -> Result<AbilityTreeContributionSet, RunRuntimeError> {
        self.run_runtime.ability_contributions(&self.ability_tree)
    }

    pub fn ability_tree_projection(
        &self,
        context: AbilityExecutionContext,
    ) -> Result<AbilityRuntimeProjection, AbilityRuntimeError> {
        self.ability_runtime.project(&self.ability_tree, context)
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

    pub fn start_pending_battle(
        &mut self,
        expected_state_hash: ActivityStateHash,
    ) -> Result<ActivityBattleHandoff, StandardUniverseBattleStartError> {
        let digest = self
            .graph
            .pending_battle()
            .ok_or(StandardUniverseBattleStartError::MissingPendingBattle)?
            .battle_spec_digest();
        let binding = self
            .overlay
            .binding_for_spec(digest.bytes())
            .ok_or(StandardUniverseBattleStartError::MissingBattleOverlay)?;
        self.graph
            .start_pending_battle(expected_state_hash, Arc::clone(binding.contract()))
            .map_err(StandardUniverseBattleStartError::Activity)
    }

    pub fn submit_pending_battle_result(
        &mut self,
        expected_state_hash: ActivityStateHash,
        result: BattleResult,
    ) -> Result<GraphActivityBattleResolution, GraphActivityBattleError> {
        self.graph
            .submit_pending_battle_result(expected_state_hash, result)
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

pub(crate) fn start(
    resolution: Result<GraphActivityResolution, GraphActivityStartError>,
    context: Option<StandardUniverseRuntimeContext>,
) -> Result<StandardUniverseStartResolution, StandardUniverseStartError> {
    let context = context.ok_or(StandardUniverseStartError::MissingEncounterOverlay)?;
    let resolution = resolution.map_err(StandardUniverseStartError::Activity)?;
    Ok(StandardUniverseStartResolution::new(resolution, context))
}
