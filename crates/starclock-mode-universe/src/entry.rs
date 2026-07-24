//! Standard Universe entry validation and generic Activity-state compilation.
mod runtime_access;

use starclock_activity::{
    ActivityDefinitionIdentity, ActivityInstanceId, ActivityInventoryDefinition,
    ActivityInventoryId, ActivityMasterSeed, ActivityScope, ActivitySlotDefinition, ActivitySlotId,
    ActivityStateDefinition, ActivityStateSource, ActivityStateVisibility, ActivityValue,
    GraphActivity, GraphActivityDefinition, GraphActivityResolution, GraphActivityStartError,
    LoadoutLockScope, ParticipantLock, ParticipantPolicy, ParticipantUniquenessScope,
    SlotCarryPolicy, SlotResetPoint,
};
use std::sync::{Arc, OnceLock};

use crate::{
    ability_runtime::{
        AbilityExecutionContext, AbilityRuntimeCatalog, AbilityRuntimeProjection, AbilityTarget,
    },
    abundance_runtime::AbundanceRuntimeCatalog,
    battle_contribution::UniverseBattleContributionCompiler,
    battle_overlay::UniverseEncounterOverlay,
    blessing_runtime::BlessingRuntimeCatalog,
    catalog::UniverseCatalog,
    curio_activity::{CurioActivityBindings, compile_records as compile_curio_activity_records},
    curio_effect_runtime::CurioEffectRuntimeCatalog,
    curio_runtime::CurioRuntimeCatalog,
    destruction_runtime::DestructionRuntimeCatalog,
    elation_runtime::ElationRuntimeCatalog,
    encounter_content_runtime::EncounterContentRuntimeCatalog,
    entry_identity::compile_identity,
    erudition_runtime::EruditionRuntimeCatalog,
    hunt_runtime::HuntRuntimeCatalog,
    id::{AbilityTreeNodeId, DifficultyId, PathId, WorldId},
    negative_curio_runtime::NegativeCurioRuntimeCatalog,
    nihility_runtime::NihilityRuntimeCatalog,
    occurrence_effect_runtime::OccurrenceEffectRuntimeCatalog,
    occurrence_interaction::OccurrenceInteractionRuntimeCatalog,
    path_runtime::PathRuntimeCatalog,
    preservation_runtime::PreservationRuntimeCatalog,
    propagation_runtime::PropagationRuntimeCatalog,
    remembrance_runtime::RemembranceRuntimeCatalog,
    run_runtime::RunRuntimeCatalog,
    service_effect_runtime::ServiceEffectRuntimeCatalog,
    service_interaction::{ServiceActivityBindings, ServiceInteractionRuntimeCatalog},
};

pub const STANDARD_UNIVERSE_ENTRY_REVISION: &str = "standard-universe-entry-v4";

const WORLD_SLOT: u32 = 1;
const DIFFICULTY_SLOT: u32 = 2;
const PATH_SLOT: u32 = 3;
const ABILITY_TREE_SLOT: u32 = 4;
const TOPOLOGY_SLOT: u32 = 5;
const HUB_CLEAR_SLOT: u32 = 6;
const ROOM_SLOT: u32 = 7;
const ENCOUNTER_MEMBER_SLOT: u32 = 8;
const BLESSING_REROLL_SLOT: u32 = 9;
const PATH_BLESSING_COUNT_SLOT: u32 = 10;
const CURIO_STATE_SLOT: u32 = 11;
const CURIO_CHARGE_SLOT: u32 = 12;
const COSMIC_FRAGMENTS_SLOT: u32 = 13;
const EXTERNAL_OUTCOME_SLOT: u32 = 14;
const OCCURRENCE_EFFECT_SLOT: u32 = 15;
const SERVICE_USE_SLOT: u32 = 16;
const SERVICE_EFFECT_SLOT: u32 = 17;
const CURIO_EVENT_SLOT: u32 = 18;
const ABILITY_PROJECTION_SLOT: u32 = 19;
const BLESSING_INVENTORY: u32 = 1;
const FORMATION_INVENTORY: u32 = 2;
const CURIO_INVENTORY: u32 = 3;
const WORLD_SOURCE: u64 = 0x5355_0001;
const DIFFICULTY_SOURCE: u64 = 0x5355_0002;
const PATH_SOURCE: u64 = 0x5355_0003;
const ABILITY_TREE_SOURCE: u64 = 0x5355_0004;
const TOPOLOGY_SOURCE: u64 = 0x5355_0005;
const HUB_CLEAR_SOURCE: u64 = 0x5355_0006;
const ROOM_SOURCE: u64 = 0x5355_0007;
const ENCOUNTER_MEMBER_SOURCE: u64 = 0x5355_0008;
const BLESSING_REROLL_SOURCE: u64 = 0x5355_0009;
const PATH_BLESSING_COUNT_SOURCE: u64 = 0x5355_000A;
const CURIO_STATE_SOURCE: u64 = 0x5355_000B;
const CURIO_CHARGE_SOURCE: u64 = 0x5355_000C;
const COSMIC_FRAGMENTS_SOURCE: u64 = 0x5355_000D;
const EXTERNAL_OUTCOME_SOURCE: u64 = 0x5355_000E;
const OCCURRENCE_EFFECT_SOURCE: u64 = 0x5355_000F;
const SERVICE_USE_SOURCE: u64 = 0x5355_0010;
const SERVICE_EFFECT_SOURCE: u64 = 0x5355_0011;
const CURIO_EVENT_SOURCE: u64 = 0x5355_0012;
const ABILITY_PROJECTION_SOURCE: u64 = 0x5355_0013;
const BLESSING_INVENTORY_SOURCE: u64 = 0x5355_1001;
const FORMATION_INVENTORY_SOURCE: u64 = 0x5355_1002;
const CURIO_INVENTORY_SOURCE: u64 = 0x5355_1003;

/// Validated caller-owned inputs for one Standard Universe run.
///
/// Path selection is deliberately not an entry argument. It is the first
/// authoritative Activity decision and begins as an empty optional slot.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StandardUniverseEntry {
    world: WorldId,
    difficulty: DifficultyId,
    participants: ParticipantLock,
    ability_tree: Box<[AbilityTreeNodeId]>,
    encounter_overlay: Option<Arc<UniverseEncounterOverlay>>,
}

impl StandardUniverseEntry {
    #[must_use]
    pub fn new(
        world: WorldId,
        difficulty: DifficultyId,
        participants: ParticipantLock,
        ability_tree: Vec<AbilityTreeNodeId>,
    ) -> Self {
        Self {
            world,
            difficulty,
            participants,
            ability_tree: ability_tree.into_boxed_slice(),
            encounter_overlay: None,
        }
    }

    #[must_use]
    pub const fn world(&self) -> WorldId {
        self.world
    }

    #[must_use]
    pub const fn difficulty(&self) -> DifficultyId {
        self.difficulty
    }

    #[must_use]
    pub const fn participants(&self) -> &ParticipantLock {
        &self.participants
    }

    #[must_use]
    pub fn ability_tree(&self) -> &[AbilityTreeNodeId] {
        &self.ability_tree
    }

    #[must_use]
    pub fn with_encounter_overlay(mut self, overlay: UniverseEncounterOverlay) -> Self {
        self.encounter_overlay = Some(Arc::new(overlay));
        self
    }

    #[must_use]
    pub const fn encounter_overlay(&self) -> Option<&Arc<UniverseEncounterOverlay>> {
        self.encounter_overlay.as_ref()
    }
}

/// Immutable compiler facade over one validated shared Universe catalog.
#[derive(Clone, Debug)]
pub struct StandardUniverseProfile {
    catalog: Arc<UniverseCatalog>,
    topology_template: Arc<OnceLock<crate::topology::CompiledUniverseTopology>>,
}

impl StandardUniverseProfile {
    #[must_use]
    pub fn new(catalog: Arc<UniverseCatalog>) -> Self {
        Self {
            catalog,
            topology_template: Arc::new(OnceLock::new()),
        }
    }

    #[must_use]
    pub const fn catalog(&self) -> &Arc<UniverseCatalog> {
        &self.catalog
    }

    pub fn compile(
        &self,
        entry: StandardUniverseEntry,
    ) -> Result<CompiledActivity, StandardUniverseCompileError> {
        let world = self
            .catalog
            .world(entry.world)
            .ok_or(StandardUniverseCompileError::UnknownWorld(entry.world))?;
        let difficulty = self.catalog.difficulty(entry.difficulty).ok_or(
            StandardUniverseCompileError::UnknownDifficulty(entry.difficulty),
        )?;
        if difficulty.world() != world.id() || !world.difficulties().contains(&difficulty.id()) {
            return Err(StandardUniverseCompileError::DifficultyWorldMismatch {
                world: world.id(),
                difficulty: difficulty.id(),
            });
        }
        validate_participants(entry.participants.policy())?;
        if entry.encounter_overlay.as_ref().is_some_and(|overlay| {
            overlay.participant_lock_digest() != Some(entry.participants.digest())
        }) {
            return Err(StandardUniverseCompileError::EncounterOverlayParticipantMismatch);
        }

        let ability_tree = canonical_ability_tree(&self.catalog, &entry.ability_tree)?;
        let ability_runtime = Arc::new(
            AbilityRuntimeCatalog::compile(&self.catalog)
                .map_err(|_| StandardUniverseCompileError::InvalidAbilityRuntime)?,
        );
        let battle_contribution_compiler = Arc::new(
            UniverseBattleContributionCompiler::compile(Arc::clone(&self.catalog))
                .map_err(|_| StandardUniverseCompileError::InvalidBattleContributionRuntime)?,
        );
        let run_start = ability_runtime
            .project(&ability_tree, AbilityExecutionContext::run_start())
            .map_err(|_| StandardUniverseCompileError::InvalidAbilityRuntime)?;
        let initial_cosmic_fragments = run_start
            .value(AbilityTarget::InitialCosmicFragments)
            .map_or(Ok(0), |value| {
                value
                    .integral()
                    .ok_or(StandardUniverseCompileError::InvalidAbilityRuntime)
            })?;
        let blessing_runtime = Arc::new(
            BlessingRuntimeCatalog::compile(&self.catalog)
                .map_err(|_| StandardUniverseCompileError::InvalidBlessingRuntime)?,
        );
        let path_runtime = Arc::new(
            PathRuntimeCatalog::compile(&self.catalog)
                .map_err(|_| StandardUniverseCompileError::InvalidPathRuntime)?,
        );
        let preservation_runtime = Arc::new(
            PreservationRuntimeCatalog::compile(&self.catalog)
                .map_err(|_| StandardUniverseCompileError::InvalidPathRuntime)?,
        );
        let remembrance_runtime = Arc::new(
            RemembranceRuntimeCatalog::compile(&self.catalog)
                .map_err(|_| StandardUniverseCompileError::InvalidPathRuntime)?,
        );
        let nihility_runtime = Arc::new(
            NihilityRuntimeCatalog::compile(&self.catalog)
                .map_err(|_| StandardUniverseCompileError::InvalidPathRuntime)?,
        );
        let abundance_runtime = Arc::new(
            AbundanceRuntimeCatalog::compile(&self.catalog)
                .map_err(|_| StandardUniverseCompileError::InvalidPathRuntime)?,
        );
        let hunt_runtime = Arc::new(
            HuntRuntimeCatalog::compile(&self.catalog)
                .map_err(|_| StandardUniverseCompileError::InvalidPathRuntime)?,
        );
        let destruction_runtime = Arc::new(
            DestructionRuntimeCatalog::compile(&self.catalog)
                .map_err(|_| StandardUniverseCompileError::InvalidPathRuntime)?,
        );
        let elation_runtime = Arc::new(
            ElationRuntimeCatalog::compile(&self.catalog)
                .map_err(|_| StandardUniverseCompileError::InvalidPathRuntime)?,
        );
        let propagation_runtime = Arc::new(
            PropagationRuntimeCatalog::compile(&self.catalog)
                .map_err(|_| StandardUniverseCompileError::InvalidPathRuntime)?,
        );
        let erudition_runtime = Arc::new(
            EruditionRuntimeCatalog::compile(&self.catalog)
                .map_err(|_| StandardUniverseCompileError::InvalidPathRuntime)?,
        );
        let curio_runtime = Arc::new(
            CurioRuntimeCatalog::compile(&self.catalog)
                .map_err(|_| StandardUniverseCompileError::InvalidCurioRuntime)?,
        );
        let curio_activity_records = compile_curio_activity_records(&curio_runtime)
            .map_err(|_| StandardUniverseCompileError::InvalidCurioRuntime)?;
        let curio_activity_bindings = CurioActivityBindings {
            inventory: inventory(CURIO_INVENTORY),
            state_slot: slot(CURIO_STATE_SLOT),
            charge_slot: slot(CURIO_CHARGE_SLOT),
            event_slot: slot(CURIO_EVENT_SLOT),
        };
        let curio_effect_runtime = Arc::new(
            CurioEffectRuntimeCatalog::compile(&self.catalog, &curio_runtime)
                .map_err(|_| StandardUniverseCompileError::InvalidCurioRuntime)?,
        );
        let negative_curio_runtime = Arc::new(
            NegativeCurioRuntimeCatalog::compile(&curio_runtime)
                .map_err(|_| StandardUniverseCompileError::InvalidCurioRuntime)?,
        );
        let run_runtime = Arc::new(
            RunRuntimeCatalog::compile(&self.catalog)
                .map_err(|_| StandardUniverseCompileError::InvalidRunRuntime)?,
        );
        let occurrence_effect_runtime = Arc::new(
            OccurrenceEffectRuntimeCatalog::compile(&self.catalog, &run_runtime)
                .map_err(|_| StandardUniverseCompileError::InvalidRunRuntime)?,
        );
        let occurrence_interaction_runtime = Arc::new(
            OccurrenceInteractionRuntimeCatalog::compile(
                &self.catalog,
                slot(COSMIC_FRAGMENTS_SLOT),
                inventory(BLESSING_INVENTORY),
                &curio_activity_records,
                curio_activity_bindings,
                slot(OCCURRENCE_EFFECT_SLOT),
            )
            .map_err(|_| StandardUniverseCompileError::InvalidRunRuntime)?,
        );
        let service_effect_runtime = Arc::new(
            ServiceEffectRuntimeCatalog::compile(&run_runtime)
                .map_err(|_| StandardUniverseCompileError::InvalidRunRuntime)?,
        );
        let service_interaction_runtime = Arc::new(
            ServiceInteractionRuntimeCatalog::compile(
                &self.catalog,
                service_effect_runtime.as_ref().clone(),
                &curio_runtime,
                curio_activity_bindings,
                ServiceActivityBindings {
                    cosmic_fragments: slot(COSMIC_FRAGMENTS_SLOT),
                    service_uses: slot(SERVICE_USE_SLOT),
                    service_effects: slot(SERVICE_EFFECT_SLOT),
                    blessing_inventory: inventory(BLESSING_INVENTORY),
                    curio_inventory: inventory(CURIO_INVENTORY),
                },
            )
            .map_err(|_| StandardUniverseCompileError::InvalidRunRuntime)?,
        );
        let encounter_content_runtime = Arc::new(
            EncounterContentRuntimeCatalog::compile(&self.catalog)
                .map_err(|_| StandardUniverseCompileError::InvalidEncounterContentRuntime)?,
        );
        if let Some(overlay) = entry.encounter_overlay.as_deref() {
            encounter_content_runtime
                .validate_overlay(overlay)
                .map_err(|_| StandardUniverseCompileError::InvalidEncounterContentRuntime)?;
        }
        let path_options = self
            .catalog
            .paths()
            .iter()
            .map(|path| path.id())
            .collect::<Vec<_>>();
        if path_options.is_empty() {
            return Err(StandardUniverseCompileError::NoAvailablePaths);
        }

        let state = compile_state(
            world.id(),
            difficulty.id(),
            &ability_tree,
            initial_cosmic_fragments,
            &run_start,
        )?;
        let participant_digest = entry.participants.digest();
        let identity = compile_identity(
            &self.catalog,
            world.id(),
            difficulty.id(),
            participant_digest.bytes(),
            &ability_tree,
            &path_options,
            entry.encounter_overlay.as_deref(),
        )?;
        let participants = Arc::new(entry.participants);
        let topology = if let Some(template) = self.topology_template.get() {
            crate::topology::rebind(template, identity, state.clone(), Arc::clone(&participants))
                .map_err(StandardUniverseCompileError::Topology)?
        } else {
            let compiled = crate::topology::compile(
                &self.catalog,
                blessing_runtime.as_ref(),
                path_runtime.as_ref(),
                identity,
                state.clone(),
                Arc::clone(&participants),
                slot(PATH_SLOT),
                slot(TOPOLOGY_SLOT),
                slot(HUB_CLEAR_SLOT),
                slot(ROOM_SLOT),
                slot(ENCOUNTER_MEMBER_SLOT),
                inventory(BLESSING_INVENTORY),
                slot(BLESSING_REROLL_SLOT),
                slot(PATH_BLESSING_COUNT_SLOT),
                inventory(FORMATION_INVENTORY),
                occurrence_interaction_runtime.as_ref(),
                service_interaction_runtime.as_ref(),
                slot(EXTERNAL_OUTCOME_SLOT),
            )
            .map_err(StandardUniverseCompileError::Topology)?;
            let _ = self.topology_template.set(compiled.clone());
            compiled
        };

        Ok(CompiledActivity {
            catalog: Arc::clone(&self.catalog),
            identity,
            world: world.id(),
            difficulty: difficulty.id(),
            participants,
            ability_tree: ability_tree.into_boxed_slice(),
            path_options: path_options.into_boxed_slice(),
            state: topology.runtime.state_definition().clone(),
            runtime: topology.runtime,
            hubs: topology.hubs,
            topology_candidates: topology.candidates,
            encounter_options: topology.encounter_options,
            interactions: topology.interactions,
            encounter_overlay: entry.encounter_overlay,
            blessing_runtime,
            path_runtime,
            preservation_runtime,
            remembrance_runtime,
            nihility_runtime,
            abundance_runtime,
            hunt_runtime,
            destruction_runtime,
            elation_runtime,
            propagation_runtime,
            erudition_runtime,
            curio_runtime,
            curio_effect_runtime,
            negative_curio_runtime,
            run_runtime,
            occurrence_effect_runtime,
            occurrence_interaction_runtime,
            service_effect_runtime,
            service_interaction_runtime,
            encounter_content_runtime,
            ability_runtime,
            battle_contribution_compiler,
        })
    }
}

/// Immutable mode-compiled Activity entry contract.
///
/// P3-B2 attaches the topology graph and generic runtime start operation to
/// this same type. This batch owns only entry selections and Activity state.
#[derive(Clone, Debug)]
pub struct CompiledActivity {
    catalog: Arc<UniverseCatalog>,
    identity: ActivityDefinitionIdentity,
    world: WorldId,
    difficulty: DifficultyId,
    participants: Arc<ParticipantLock>,
    ability_tree: Box<[AbilityTreeNodeId]>,
    path_options: Box<[PathId]>,
    state: ActivityStateDefinition,
    runtime: Arc<GraphActivityDefinition>,
    hubs: Arc<[crate::topology::DomainHubDefinition]>,
    topology_candidates: Arc<[crate::id::TopologyId]>,
    encounter_options: Arc<[crate::topology::EncounterOptionBinding]>,
    interactions: Arc<[crate::topology::AbstractInteractionBinding]>,
    encounter_overlay: Option<Arc<UniverseEncounterOverlay>>,
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
    occurrence_interaction_runtime: Arc<OccurrenceInteractionRuntimeCatalog>,
    service_effect_runtime: Arc<ServiceEffectRuntimeCatalog>,
    service_interaction_runtime: Arc<ServiceInteractionRuntimeCatalog>,
    encounter_content_runtime: Arc<EncounterContentRuntimeCatalog>,
    ability_runtime: Arc<AbilityRuntimeCatalog>,
    battle_contribution_compiler: Arc<UniverseBattleContributionCompiler>,
}

impl CompiledActivity {
    #[must_use]
    pub const fn catalog(&self) -> &Arc<UniverseCatalog> {
        &self.catalog
    }

    #[must_use]
    pub const fn identity(&self) -> ActivityDefinitionIdentity {
        self.identity
    }

    #[must_use]
    pub const fn world(&self) -> WorldId {
        self.world
    }

    #[must_use]
    pub const fn difficulty(&self) -> DifficultyId {
        self.difficulty
    }

    #[must_use]
    pub const fn participants(&self) -> &Arc<ParticipantLock> {
        &self.participants
    }

    #[must_use]
    pub fn ability_tree(&self) -> &[AbilityTreeNodeId] {
        &self.ability_tree
    }

    /// Canonical Path candidates for the first runtime choice.
    #[must_use]
    pub fn path_options(&self) -> &[PathId] {
        &self.path_options
    }

    #[must_use]
    pub const fn state_definition(&self) -> &ActivityStateDefinition {
        &self.state
    }

    #[must_use]
    pub const fn runtime_definition(&self) -> &Arc<GraphActivityDefinition> {
        &self.runtime
    }

    #[must_use]
    pub fn domain_hubs(&self) -> &[crate::topology::DomainHubDefinition] {
        &self.hubs
    }

    #[must_use]
    pub fn topology_candidates(&self) -> &[crate::id::TopologyId] {
        &self.topology_candidates
    }

    #[must_use]
    pub fn encounter_options(&self) -> &[crate::topology::EncounterOptionBinding] {
        &self.encounter_options
    }

    #[must_use]
    pub fn abstract_interactions(&self) -> &[crate::topology::AbstractInteractionBinding] {
        &self.interactions
    }

    #[must_use]
    pub const fn encounter_overlay(&self) -> Option<&Arc<UniverseEncounterOverlay>> {
        self.encounter_overlay.as_ref()
    }

    pub fn start(
        &self,
        instance: ActivityInstanceId,
        master_seed: ActivityMasterSeed,
    ) -> Result<GraphActivityResolution, GraphActivityStartError> {
        GraphActivity::start(Arc::clone(&self.runtime), instance, master_seed)
    }

    pub fn start_standard(
        &self,
        instance: ActivityInstanceId,
        master_seed: ActivityMasterSeed,
    ) -> Result<
        crate::runtime::StandardUniverseStartResolution,
        crate::runtime::StandardUniverseStartError,
    > {
        crate::runtime::start(
            GraphActivity::start(Arc::clone(&self.runtime), instance, master_seed),
            self.encounter_overlay.as_ref().map(|overlay| {
                crate::runtime::StandardUniverseRuntimeContext {
                    participants: Arc::clone(&self.participants),
                    encounter_options: Arc::clone(&self.encounter_options),
                    overlay: Arc::clone(overlay),
                    blessing_runtime: Arc::clone(&self.blessing_runtime),
                    path_runtime: Arc::clone(&self.path_runtime),
                    preservation_runtime: Arc::clone(&self.preservation_runtime),
                    remembrance_runtime: Arc::clone(&self.remembrance_runtime),
                    nihility_runtime: Arc::clone(&self.nihility_runtime),
                    abundance_runtime: Arc::clone(&self.abundance_runtime),
                    hunt_runtime: Arc::clone(&self.hunt_runtime),
                    destruction_runtime: Arc::clone(&self.destruction_runtime),
                    elation_runtime: Arc::clone(&self.elation_runtime),
                    propagation_runtime: Arc::clone(&self.propagation_runtime),
                    erudition_runtime: Arc::clone(&self.erudition_runtime),
                    curio_runtime: Arc::clone(&self.curio_runtime),
                    curio_effect_runtime: Arc::clone(&self.curio_effect_runtime),
                    negative_curio_runtime: Arc::clone(&self.negative_curio_runtime),
                    run_runtime: Arc::clone(&self.run_runtime),
                    occurrence_effect_runtime: Arc::clone(&self.occurrence_effect_runtime),
                    service_effect_runtime: Arc::clone(&self.service_effect_runtime),
                    ability_runtime: Arc::clone(&self.ability_runtime),
                    battle_contribution_compiler: Arc::clone(&self.battle_contribution_compiler),
                    ability_tree: self.ability_tree.clone(),
                    blessing_inventory: self.blessing_inventory(),
                    formation_inventory: self.formation_inventory(),
                    curio_inventory: self.curio_inventory(),
                    curio_state_slot: self.curio_state_slot(),
                    curio_charge_slot: self.curio_charge_slot(),
                    curio_event_slot: self.curio_event_slot(),
                    cosmic_fragments_slot: self.cosmic_fragments_slot(),
                    selected_path_slot: self.selected_path_slot(),
                    ability_projection_slot: self.ability_projection_slot(),
                }
            }),
        )
    }

    #[must_use]
    pub const fn world_slot(&self) -> ActivitySlotId {
        slot(WORLD_SLOT)
    }

    #[must_use]
    pub const fn difficulty_slot(&self) -> ActivitySlotId {
        slot(DIFFICULTY_SLOT)
    }

    #[must_use]
    pub const fn selected_path_slot(&self) -> ActivitySlotId {
        slot(PATH_SLOT)
    }

    #[must_use]
    pub const fn ability_tree_slot(&self) -> ActivitySlotId {
        slot(ABILITY_TREE_SLOT)
    }

    #[must_use]
    pub const fn selected_topology_slot(&self) -> ActivitySlotId {
        slot(TOPOLOGY_SLOT)
    }

    #[must_use]
    pub const fn hub_clear_slot(&self) -> ActivitySlotId {
        slot(HUB_CLEAR_SLOT)
    }

    #[must_use]
    pub const fn selected_room_slot(&self) -> ActivitySlotId {
        slot(ROOM_SLOT)
    }

    #[must_use]
    pub const fn selected_encounter_member_slot(&self) -> ActivitySlotId {
        slot(ENCOUNTER_MEMBER_SLOT)
    }

    #[must_use]
    pub const fn blessing_reroll_slot(&self) -> ActivitySlotId {
        slot(BLESSING_REROLL_SLOT)
    }

    #[must_use]
    pub const fn path_blessing_count_slot(&self) -> ActivitySlotId {
        slot(PATH_BLESSING_COUNT_SLOT)
    }

    #[must_use]
    pub const fn curio_state_slot(&self) -> ActivitySlotId {
        slot(CURIO_STATE_SLOT)
    }

    #[must_use]
    pub const fn curio_charge_slot(&self) -> ActivitySlotId {
        slot(CURIO_CHARGE_SLOT)
    }

    #[must_use]
    pub const fn cosmic_fragments_slot(&self) -> ActivitySlotId {
        slot(COSMIC_FRAGMENTS_SLOT)
    }

    #[must_use]
    pub const fn external_outcome_slot(&self) -> ActivitySlotId {
        slot(EXTERNAL_OUTCOME_SLOT)
    }

    #[must_use]
    pub const fn occurrence_effect_slot(&self) -> ActivitySlotId {
        slot(OCCURRENCE_EFFECT_SLOT)
    }

    #[must_use]
    pub const fn service_use_slot(&self) -> ActivitySlotId {
        slot(SERVICE_USE_SLOT)
    }

    #[must_use]
    pub const fn service_effect_slot(&self) -> ActivitySlotId {
        slot(SERVICE_EFFECT_SLOT)
    }

    #[must_use]
    pub const fn curio_event_slot(&self) -> ActivitySlotId {
        slot(CURIO_EVENT_SLOT)
    }

    #[must_use]
    pub const fn ability_projection_slot(&self) -> ActivitySlotId {
        slot(ABILITY_PROJECTION_SLOT)
    }

    #[must_use]
    pub const fn blessing_inventory(&self) -> ActivityInventoryId {
        inventory(BLESSING_INVENTORY)
    }

    #[must_use]
    pub const fn formation_inventory(&self) -> ActivityInventoryId {
        inventory(FORMATION_INVENTORY)
    }

    #[must_use]
    pub const fn curio_inventory(&self) -> ActivityInventoryId {
        inventory(CURIO_INVENTORY)
    }
}

fn validate_participants(actual: ParticipantPolicy) -> Result<(), StandardUniverseCompileError> {
    let expected = ParticipantPolicy::new(
        1,
        1,
        4,
        ParticipantUniquenessScope::Activity,
        LoadoutLockScope::Activity,
    )
    .expect("static Standard Universe participant policy is valid");
    if actual != expected {
        return Err(StandardUniverseCompileError::ParticipantPolicyMismatch);
    }
    Ok(())
}

fn canonical_ability_tree(
    catalog: &UniverseCatalog,
    input: &[AbilityTreeNodeId],
) -> Result<Vec<AbilityTreeNodeId>, StandardUniverseCompileError> {
    let mut selected = input.to_vec();
    selected.sort_unstable();
    if let Some(pair) = selected.windows(2).find(|pair| pair[0] == pair[1]) {
        return Err(StandardUniverseCompileError::DuplicateAbilityTreeNode(
            pair[0],
        ));
    }
    for node in &selected {
        let definition = catalog
            .ability_tree_node(*node)
            .ok_or(StandardUniverseCompileError::UnknownAbilityTreeNode(*node))?;
        if let Some(prerequisite) = definition
            .prerequisites()
            .iter()
            .find(|prerequisite| selected.binary_search(prerequisite).is_err())
        {
            return Err(
                StandardUniverseCompileError::MissingAbilityTreePrerequisite {
                    node: *node,
                    prerequisite: *prerequisite,
                },
            );
        }
    }
    Ok(selected)
}

fn compile_state(
    world: WorldId,
    difficulty: DifficultyId,
    ability_tree: &[AbilityTreeNodeId],
    initial_cosmic_fragments: i64,
    run_start: &AbilityRuntimeProjection,
) -> Result<ActivityStateDefinition, StandardUniverseCompileError> {
    let slots = vec![
        activity_slot(
            WORLD_SLOT,
            ActivityValue::StableId(u64::from(world.get())),
            None,
            WORLD_SOURCE,
            ActivityStateVisibility::Player,
        )?,
        activity_slot(
            DIFFICULTY_SLOT,
            ActivityValue::StableId(u64::from(difficulty.get())),
            None,
            DIFFICULTY_SOURCE,
            ActivityStateVisibility::Player,
        )?,
        activity_slot(
            PATH_SLOT,
            ActivityValue::OptionalId(None),
            None,
            PATH_SOURCE,
            ActivityStateVisibility::Player,
        )?,
        activity_slot(
            ABILITY_TREE_SLOT,
            ActivityValue::OrderedIdSet(
                ability_tree
                    .iter()
                    .map(|id| u64::from(id.get()))
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
            ),
            Some(4_096),
            ABILITY_TREE_SOURCE,
            ActivityStateVisibility::Player,
        )?,
        activity_slot(
            TOPOLOGY_SLOT,
            ActivityValue::OptionalId(None),
            None,
            TOPOLOGY_SOURCE,
            ActivityStateVisibility::Private,
        )?,
        activity_slot(
            HUB_CLEAR_SLOT,
            ActivityValue::BoundedCounterMap(Box::new([])),
            Some(4_096),
            HUB_CLEAR_SOURCE,
            ActivityStateVisibility::Private,
        )?,
        activity_slot(
            ROOM_SLOT,
            ActivityValue::OptionalId(None),
            None,
            ROOM_SOURCE,
            ActivityStateVisibility::Private,
        )?,
        activity_slot(
            ENCOUNTER_MEMBER_SLOT,
            ActivityValue::OptionalId(None),
            None,
            ENCOUNTER_MEMBER_SOURCE,
            ActivityStateVisibility::Private,
        )?,
        activity_slot(
            BLESSING_REROLL_SLOT,
            ActivityValue::BoundedCounterMap(Box::new([])),
            Some(4_096),
            BLESSING_REROLL_SOURCE,
            ActivityStateVisibility::Private,
        )?,
        counter_slot(
            PATH_BLESSING_COUNT_SLOT,
            9,
            0,
            18,
            PATH_BLESSING_COUNT_SOURCE,
            ActivityStateVisibility::Player,
        )?,
        counter_slot(
            CURIO_STATE_SLOT,
            61,
            0,
            i64::from(u32::MAX),
            CURIO_STATE_SOURCE,
            ActivityStateVisibility::Player,
        )?,
        counter_slot(
            CURIO_CHARGE_SLOT,
            61,
            0,
            3,
            CURIO_CHARGE_SOURCE,
            ActivityStateVisibility::Player,
        )?,
        integer_slot(
            COSMIC_FRAGMENTS_SLOT,
            initial_cosmic_fragments,
            0,
            crate::run_runtime::MAX_COSMIC_FRAGMENTS,
            COSMIC_FRAGMENTS_SOURCE,
            ActivityStateVisibility::Player,
        )?,
        counter_slot(
            EXTERNAL_OUTCOME_SLOT,
            579,
            0,
            1,
            EXTERNAL_OUTCOME_SOURCE,
            ActivityStateVisibility::Player,
        )?,
        counter_slot(
            OCCURRENCE_EFFECT_SLOT,
            4_096,
            0,
            i64::from(u32::MAX),
            OCCURRENCE_EFFECT_SOURCE,
            ActivityStateVisibility::Private,
        )?,
        counter_slot(
            SERVICE_USE_SLOT,
            94,
            0,
            i64::from(u32::MAX),
            SERVICE_USE_SOURCE,
            ActivityStateVisibility::Player,
        )?,
        counter_slot(
            SERVICE_EFFECT_SLOT,
            94,
            0,
            i64::from(u32::MAX),
            SERVICE_EFFECT_SOURCE,
            ActivityStateVisibility::Private,
        )?,
        counter_slot(
            CURIO_EVENT_SLOT,
            640,
            0,
            i64::from(u32::MAX),
            CURIO_EVENT_SOURCE,
            ActivityStateVisibility::Private,
        )?,
        counter_slot_with_initial(
            ABILITY_PROJECTION_SLOT,
            run_start
                .values()
                .iter()
                .map(|value| {
                    (
                        value.target().activity_key(),
                        value.value().raw_six_decimal(),
                    )
                })
                .collect::<Vec<_>>(),
            22,
            0,
            i64::MAX,
            ABILITY_PROJECTION_SOURCE,
            ActivityStateVisibility::Private,
        )?,
    ];
    let inventories = vec![
        ActivityInventoryDefinition::new(
            inventory(BLESSING_INVENTORY),
            ActivityScope::Activity,
            162,
            2,
            SlotCarryPolicy::CarryExact,
            ActivityStateVisibility::Player,
            ActivityStateSource::new(BLESSING_INVENTORY_SOURCE)
                .expect("static inventory source is non-zero"),
        )
        .map_err(|_| StandardUniverseCompileError::InvalidActivityState)?,
        ActivityInventoryDefinition::new(
            inventory(FORMATION_INVENTORY),
            ActivityScope::Activity,
            27,
            1,
            SlotCarryPolicy::CarryExact,
            ActivityStateVisibility::Player,
            ActivityStateSource::new(FORMATION_INVENTORY_SOURCE)
                .expect("static inventory source is non-zero"),
        )
        .map_err(|_| StandardUniverseCompileError::InvalidActivityState)?,
        ActivityInventoryDefinition::new(
            inventory(CURIO_INVENTORY),
            ActivityScope::Activity,
            61,
            1,
            SlotCarryPolicy::CarryExact,
            ActivityStateVisibility::Player,
            ActivityStateSource::new(CURIO_INVENTORY_SOURCE)
                .expect("static inventory source is non-zero"),
        )
        .map_err(|_| StandardUniverseCompileError::InvalidActivityState)?,
    ];
    ActivityStateDefinition::new(slots, inventories, vec![])
        .map_err(|_| StandardUniverseCompileError::InvalidActivityState)
}

fn activity_slot(
    id: u32,
    initial: ActivityValue,
    maximum_entries: Option<u32>,
    source: u64,
    visibility: ActivityStateVisibility,
) -> Result<ActivitySlotDefinition, StandardUniverseCompileError> {
    let bounds = matches!(&initial, ActivityValue::BoundedCounterMap(_)).then_some((0, 1));
    ActivitySlotDefinition::new_with_policy(
        slot(id),
        ActivityScope::Activity,
        initial,
        bounds,
        maximum_entries,
        vec![SlotResetPoint::ActivityStart],
        SlotCarryPolicy::CarryExact,
        visibility,
        ActivityStateSource::new(source).expect("static state source is non-zero"),
    )
    .map_err(|_| StandardUniverseCompileError::InvalidActivityState)
}

fn counter_slot(
    id: u32,
    maximum_entries: u32,
    minimum: i64,
    maximum: i64,
    source: u64,
    visibility: ActivityStateVisibility,
) -> Result<ActivitySlotDefinition, StandardUniverseCompileError> {
    ActivitySlotDefinition::new_with_policy(
        slot(id),
        ActivityScope::Activity,
        ActivityValue::BoundedCounterMap(Box::new([])),
        Some((minimum, maximum)),
        Some(maximum_entries),
        vec![SlotResetPoint::ActivityStart],
        SlotCarryPolicy::CarryExact,
        visibility,
        ActivityStateSource::new(source).expect("static state source is non-zero"),
    )
    .map_err(|_| StandardUniverseCompileError::InvalidActivityState)
}

fn counter_slot_with_initial(
    id: u32,
    initial: Vec<(u64, i64)>,
    maximum_entries: u32,
    minimum: i64,
    maximum: i64,
    source: u64,
    visibility: ActivityStateVisibility,
) -> Result<ActivitySlotDefinition, StandardUniverseCompileError> {
    ActivitySlotDefinition::new_with_policy(
        slot(id),
        ActivityScope::Activity,
        ActivityValue::BoundedCounterMap(initial.into_boxed_slice()),
        Some((minimum, maximum)),
        Some(maximum_entries),
        vec![SlotResetPoint::ActivityStart],
        SlotCarryPolicy::CarryExact,
        visibility,
        ActivityStateSource::new(source).expect("static state source is non-zero"),
    )
    .map_err(|_| StandardUniverseCompileError::InvalidActivityState)
}

fn integer_slot(
    id: u32,
    initial: i64,
    minimum: i64,
    maximum: i64,
    source: u64,
    visibility: ActivityStateVisibility,
) -> Result<ActivitySlotDefinition, StandardUniverseCompileError> {
    ActivitySlotDefinition::new_with_policy(
        slot(id),
        ActivityScope::Activity,
        ActivityValue::BoundedInteger(initial),
        Some((minimum, maximum)),
        None,
        vec![SlotResetPoint::ActivityStart],
        SlotCarryPolicy::CarryExact,
        visibility,
        ActivityStateSource::new(source).expect("static state source is non-zero"),
    )
    .map_err(|_| StandardUniverseCompileError::InvalidActivityState)
}

const fn slot(raw: u32) -> ActivitySlotId {
    match ActivitySlotId::new(raw) {
        Some(value) => value,
        None => panic!("static Activity slot ID must be non-zero"),
    }
}

const fn inventory(raw: u32) -> ActivityInventoryId {
    match ActivityInventoryId::new(raw) {
        Some(value) => value,
        None => panic!("static Activity inventory ID must be non-zero"),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StandardUniverseCompileError {
    UnknownWorld(WorldId),
    UnknownDifficulty(DifficultyId),
    DifficultyWorldMismatch {
        world: WorldId,
        difficulty: DifficultyId,
    },
    ParticipantPolicyMismatch,
    DuplicateAbilityTreeNode(AbilityTreeNodeId),
    UnknownAbilityTreeNode(AbilityTreeNodeId),
    MissingAbilityTreePrerequisite {
        node: AbilityTreeNodeId,
        prerequisite: AbilityTreeNodeId,
    },
    NoAvailablePaths,
    InvalidActivityState,
    InvalidCatalogIdentity,
    InvalidBlessingRuntime,
    InvalidPathRuntime,
    InvalidCurioRuntime,
    InvalidRunRuntime,
    InvalidAbilityRuntime,
    InvalidBattleContributionRuntime,
    InvalidEncounterContentRuntime,
    EncounterOverlayParticipantMismatch,
    Topology(crate::topology::UniverseTopologyCompileError),
}

impl core::fmt::Display for StandardUniverseCompileError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "Standard Universe entry rejected: {self:?}")
    }
}

impl std::error::Error for StandardUniverseCompileError {}
