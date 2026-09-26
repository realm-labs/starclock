//! Ordinary/Cyclical entry and ordered layer flow over generic GraphActivity.

#[cfg(test)]
#[path = "tests/decision_identity.rs"]
mod decision_identity_tests;

use std::sync::Arc;

use super::battle_settlement_runtime::SETTLEMENT_POLICY_IDENTITY;
use crate::digest::CanonicalDigestBuilder;
use starclock_activity::{
    ActivityConfigDigest, ActivityDefinitionDigest, ActivityDefinitionId,
    ActivityDefinitionIdentity, ActivityInstanceId, ActivityMasterSeed,
    ActivityProgramDefinitionError, ActivityRandomPolicies, GraphActivity, GraphActivityDefinition,
    GraphActivityResolution, GraphActivityStartError, ParticipantLock,
};
use starclock_data::{
    divergent_universe::{
        DivergentUniverseBundleCandidate, DivergentUniverseDataError,
        load_divergent_universe_bundle,
    },
    divergent_universe_catalog::{
        DivergentUniverseAreaDefinition, DivergentUniverseAreaId,
        DivergentUniverseCyclicalChallengeId, DivergentUniverseDifficultyId,
        DivergentUniverseEntryKind, DivergentUniverseFinishConditionId, DivergentUniverseLayerId,
        DivergentUniverseRunFamily,
    },
    divergent_universe_decisions::{BattleRoutePolicyKind, DecisionCatalog, DecisionDataError},
    divergent_universe_service_catalog::DivergentUniverseOccurrenceVariantId,
};

use super::battle_blessings::BattleBlessings;
use super::battle_fragments::BattleFragments;
use super::battle_room::BoundBattleRooms;
use super::curio_battle_grants::CurioBattleGrants;
use super::curio_battle_reactions::CurioBattleReactions;
use super::curio_battle_stats::CurioBattleStats;
use super::curio_victory_blessings::CurioVictoryBlessings;
use super::domain_choices::DomainChoices;
use super::encounter_pool::EncounterPool;
use super::entry_graph::{compile_graph, graph_start_node};
use super::evolution_events::EvolutionEvents;
use super::initial_equations::InitialEquations;
use super::occurrence_binding::{OccurrenceBinding, OccurrenceBindingError};
use super::progression::{
    DivergentUniverseAstronomicalEntry, DivergentUniverseCyclicalRefresh,
    DivergentUniverseProgressionProjection, DivergentUniverseProgressionRuntimeError,
};
use super::source_deck_selection::SourceDeckSelection;
use super::state::{EntryStateValues, compile_state};
use super::tawot_service::TawotService;
use super::vertical_slice::DivergentUniverseVerticalSliceError;
use super::{
    economy::{DivergentUniverseEconomyError, DivergentUniverseEconomyProjection},
    snapshot::{DivergentUniverseInputSnapshot, DivergentUniverseSnapshotError},
};

/// Explicit policy for the released data's missing layer-to-room selector.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DivergentUniverseRoomPolicy {
    /// Use one logical checkpoint per layer and keep `room=None`; no retained
    /// shared candidate becomes reachable content.
    LogicalLayerCheckpointNoCandidatePromotion,
    /// Explicitly supplied source-position programs and stage candidates; no
    /// automatic original room, encounter or boss admission is implied.
    ExplicitSourcePositionProgramsNoAutomaticAdmission,
}

/// Caller-owned immutable entry selection.
#[derive(Clone, Debug)]
pub struct DivergentUniverseEntry {
    area: DivergentUniverseAreaId,
    difficulty: DivergentUniverseDifficultyId,
    participants: Arc<ParticipantLock>,
    input_snapshot: DivergentUniverseInputSnapshot,
    permanent_unlocks: Box<[u64]>,
    requested_module: Option<starclock_data::divergent_universe_catalog::DivergentUniverseModuleId>,
    astronomical: Option<DivergentUniverseAstronomicalEntry>,
    cyclical_refresh: Option<DivergentUniverseCyclicalRefresh>,
    mapping_snapshot: Option<Arc<super::mapping::DivergentUniverseMappingSnapshot>>,
    first_ordinary_vertical_slice: bool,
    runtime_battle_route: bool,
    layer_battle_route: bool,
    initial_occurrence: Option<DivergentUniverseOccurrenceVariantId>,
    initial_equation: bool,
    initial_tawot_service: Option<u16>,
    source_deck_selection: bool,
}

impl DivergentUniverseEntry {
    /// Prepends an explicit choice among the authored source decks. This is a
    /// headless policy option, not original mask-offer eligibility or effects.
    #[must_use]
    pub fn with_source_deck_selection(mut self) -> Self {
        self.source_deck_selection = true;
        self
    }
    /// Explicitly admits a Tawot service after the initial checkpoint, before
    /// its battle. This caller-selected binding does not generate Forge domains.
    /// Levels without authored service data reject at compilation.
    #[must_use]
    pub fn with_initial_tawot_service(mut self, forge_level: u16) -> Self {
        self.initial_tawot_service = Some(forge_level);
        self
    }

    pub fn new(
        area: DivergentUniverseAreaId,
        difficulty: DivergentUniverseDifficultyId,
        participants: Arc<ParticipantLock>,
        input_snapshot: DivergentUniverseInputSnapshot,
        mut permanent_unlocks: Vec<u64>,
    ) -> Result<Self, DivergentUniverseEntryFlowError> {
        input_snapshot.validate_participants(&participants)?;
        permanent_unlocks.sort_unstable();
        if permanent_unlocks.contains(&0)
            || permanent_unlocks.windows(2).any(|pair| pair[0] == pair[1])
        {
            return Err(DivergentUniverseEntryFlowError::InvalidPermanentUnlocks);
        }
        Ok(Self {
            area,
            difficulty,
            participants,
            input_snapshot,
            permanent_unlocks: permanent_unlocks.into_boxed_slice(),
            requested_module: None,
            astronomical: None,
            cyclical_refresh: None,
            mapping_snapshot: None,
            first_ordinary_vertical_slice: false,
            runtime_battle_route: false,
            layer_battle_route: false,
            initial_occurrence: None,
            initial_equation: false,
            initial_tawot_service: None,
            source_deck_selection: false,
        })
    }

    /// Adds a caller-observed module identity for explicit stale/historical rejection.
    #[must_use]
    pub fn with_module(
        mut self,
        module: starclock_data::divergent_universe_catalog::DivergentUniverseModuleId,
    ) -> Self {
        self.requested_module = Some(module);
        self
    }

    #[must_use]
    pub fn with_astronomical(mut self, entry: DivergentUniverseAstronomicalEntry) -> Self {
        self.astronomical = Some(entry);
        self
    }

    #[must_use]
    pub fn with_cyclical_refresh(mut self, refresh: DivergentUniverseCyclicalRefresh) -> Self {
        self.cyclical_refresh = Some(refresh);
        self
    }

    #[must_use]
    pub fn with_mapping_snapshot(
        mut self,
        snapshot: Arc<super::mapping::DivergentUniverseMappingSnapshot>,
    ) -> Self {
        self.mapping_snapshot = Some(snapshot);
        self
    }

    /// Selects the frozen production Ordinary vertical-slice graph.
    ///
    /// Compilation rejects every area/difficulty other than the exact
    /// Version 4.4 selection and requires an Arithmetic Mapping snapshot.
    #[must_use]
    pub fn with_first_ordinary_vertical_slice(mut self) -> Self {
        self.first_ordinary_vertical_slice = true;
        self
    }

    /// Adds one production battle boundary after the first layer for a
    /// complete-run controller. Unlike the frozen Ordinary vertical slice,
    /// this route is valid for both Ordinary and Cyclical entries and does not
    /// manufacture any pre-battle content grants.
    #[must_use]
    pub fn with_runtime_battle_route(mut self) -> Self {
        self.runtime_battle_route = true;
        self
    }

    /// Executes the authored provisional battle route at every logical layer.
    /// This does not assert complete room or boss reachability.
    #[must_use]
    pub fn with_layer_battle_route(mut self) -> Self {
        self.runtime_battle_route = true;
        self.layer_battle_route = true;
        self
    }

    /// Adds the authored initial Equation choice, without asserting complete
    /// starting-loadout parity or released initial-pool membership.
    #[must_use]
    pub fn with_initial_equation(mut self) -> Self {
        self.initial_equation = true;
        self
    }

    /// Explicit headless placement at the first logical checkpoint. This input
    /// does not assert a released layer-to-room or occurrence-pool selector.
    #[must_use]
    pub fn with_initial_occurrence(
        mut self,
        variant: DivergentUniverseOccurrenceVariantId,
    ) -> Self {
        self.initial_occurrence = Some(variant);
        self
    }
}

/// Immutable production factory over the reference and typed decision bundles.
#[derive(Clone, Debug)]
pub struct DivergentUniverseRuntimeFactory {
    pub(super) bundle: Arc<DivergentUniverseBundleCandidate>,
    decisions: Arc<DecisionCatalog>,
}

impl DivergentUniverseRuntimeFactory {
    pub fn production() -> Result<Self, DivergentUniverseEntryFlowError> {
        Self::new(Arc::new(load_divergent_universe_bundle()?))
    }

    /// Validates the current decision bundle against the supplied reference
    /// catalog. Neither a partial factory nor an unbound decision digest escapes.
    pub fn new(
        bundle: Arc<DivergentUniverseBundleCandidate>,
    ) -> Result<Self, DivergentUniverseEntryFlowError> {
        let decisions = Arc::new(
            DecisionCatalog::production(&bundle)
                .map_err(DivergentUniverseEntryFlowError::Decisions)?,
        );
        Ok(Self { bundle, decisions })
    }

    /// Authored decision definitions, not an event execution or eligibility claim.
    #[must_use]
    pub fn decision_catalog(&self) -> &DecisionCatalog {
        &self.decisions
    }

    #[must_use]
    pub fn bundle_identity(
        &self,
    ) -> &starclock_data::divergent_universe::DivergentUniverseBundleIdentity {
        self.bundle.identity()
    }

    pub fn compile(
        &self,
        entry: DivergentUniverseEntry,
    ) -> Result<DivergentUniverseFlowInstance, DivergentUniverseEntryFlowError> {
        let catalog = self.bundle.catalog();
        if let Some(snapshot) = &entry.mapping_snapshot {
            snapshot.validate(
                self.bundle.identity().component_digest().bytes(),
                &entry.participants,
            )?;
        }
        if entry
            .requested_module
            .as_ref()
            .is_some_and(|module| module != &catalog.module().id)
        {
            return Err(DivergentUniverseEntryFlowError::HistoricalModuleRejected);
        }
        let area = catalog
            .area(&entry.area)
            .ok_or(DivergentUniverseEntryFlowError::UnknownArea)?;
        if entry.first_ordinary_vertical_slice
            && (area.id.as_str() != "divergent-universe.area.401"
                || entry.difficulty.as_str() != "divergent-universe.difficulty.3011"
                || area.run_family() != DivergentUniverseRunFamily::Ordinary
                || entry.mapping_snapshot.is_none())
        {
            return Err(DivergentUniverseEntryFlowError::VerticalSliceSelectionMismatch);
        }
        if entry.runtime_battle_route && entry.first_ordinary_vertical_slice {
            return Err(DivergentUniverseEntryFlowError::ConflictingBattleRoutes);
        }
        if entry.runtime_battle_route && entry.mapping_snapshot.is_none() {
            return Err(DivergentUniverseEntryFlowError::BattleRouteRequiresMapping);
        }
        if !area.difficulties.contains(&entry.difficulty) {
            return Err(DivergentUniverseEntryFlowError::DifficultyAreaMismatch);
        }
        let resident_entry = catalog
            .entries()
            .iter()
            .find(|value| value.kind == DivergentUniverseEntryKind::ResidentActivity)
            .ok_or(DivergentUniverseEntryFlowError::MissingResidentEntry)?;
        let challenge = match area.run_family() {
            DivergentUniverseRunFamily::Ordinary => None,
            DivergentUniverseRunFamily::Cyclical => Some(
                catalog
                    .cyclical_challenges()
                    .iter()
                    .find(|value| value.area == area.id)
                    .ok_or(DivergentUniverseEntryFlowError::MissingCyclicalBinding)?,
            ),
        };
        let difficulty = catalog
            .difficulties()
            .iter()
            .find(|value| value.id == entry.difficulty)
            .ok_or(DivergentUniverseEntryFlowError::DifficultyAreaMismatch)?;
        let progression = super::progression::compile(
            self.bundle.progression_catalog(),
            difficulty,
            entry.astronomical.as_ref(),
            area.run_family(),
            challenge,
            entry.cyclical_refresh.as_ref(),
        )?;
        let economy = super::economy::compile(
            self.bundle.service_catalog(),
            self.bundle.progression_catalog(),
            self.bundle.curio_catalog(),
            self.decision_catalog(),
        )?;
        let vertical_slice_content = entry
            .first_ordinary_vertical_slice
            .then(|| {
                super::vertical_slice::compile(
                    &self.bundle,
                    economy
                        .currency(super::economy::DivergentUniverseCurrencyKind::WorkbenchHeat)
                        .key(),
                    Arc::new(
                        self.equation_offer_runtime()
                            .map_err(|_| DivergentUniverseVerticalSliceError::CatalogJoin)?,
                    ),
                    Arc::new(
                        self.blessing_runtime()
                            .map_err(|_| DivergentUniverseVerticalSliceError::CatalogJoin)?,
                    ),
                )
                .map(Arc::new)
            })
            .transpose()?;
        let layer_values = area
            .layers
            .iter()
            .map(|id| stable_index(catalog.layers(), id, |value| &value.id))
            .collect::<Result<Vec<_>, _>>()?;
        let area_value = stable_index(catalog.areas(), &area.id, |value| &value.id)?;
        let difficulty_value =
            stable_index(catalog.difficulties(), &entry.difficulty, |value| &value.id)?;
        let identity = compile_identity(&self.bundle, self.decisions.digest(), area, &entry);
        let has_runtime_battle = entry.first_ordinary_vertical_slice || entry.runtime_battle_route;
        let source_deck_selection = entry
            .source_deck_selection
            .then(|| SourceDeckSelection::compile(self).map(Arc::new))
            .transpose()?;
        let additional_slots = source_deck_selection
            .as_ref()
            .map(|selection| selection.slots())
            .transpose()?
            .unwrap_or_default();
        let state = compile_state(
            area.run_family(),
            EntryStateValues {
                entry: stable_text(resident_entry.id.as_str()),
                module: stable_text(resident_entry.module.as_str()),
                area: area_value,
                difficulty: difficulty_value,
                first_layer: layer_values[0],
                permanent_unlocks: &entry.permanent_unlocks,
                account_loadout_snapshot: entry.input_snapshot.stable_value(),
                party_snapshot: super::snapshot::participant_stable_value(&entry.participants),
                mapping_state: entry.mapping_snapshot.as_ref().map_or_else(
                    || Vec::new().into_boxed_slice(),
                    |snapshot| snapshot.state_values(),
                ),
            },
            &progression,
            super::scope::compile(
                layer_values.len(),
                has_runtime_battle,
                entry.initial_equation,
                entry.layer_battle_route,
                entry.initial_tawot_service.is_some(),
                entry.source_deck_selection,
            )?,
            additional_slots,
        )?;
        let occurrence_binding = entry
            .initial_occurrence
            .as_ref()
            .map(|variant| OccurrenceBinding::compile(self, variant).map(Arc::new))
            .transpose()
            .map_err(DivergentUniverseEntryFlowError::Occurrence)?;
        let battle_blessings = has_runtime_battle
            .then(|| BattleBlessings::compile(self).map(Arc::new))
            .transpose()?;
        let curio_battle_stats = has_runtime_battle
            .then(|| CurioBattleStats::compile(self).map(Arc::new))
            .transpose()?;
        let curio_battle_reactions =
            has_runtime_battle.then(|| Arc::new(CurioBattleReactions::new(self)));
        let curio_battle_grants = has_runtime_battle
            .then(|| CurioBattleGrants::compile(self).map(Arc::new))
            .transpose()?;
        let battle_fragments = has_runtime_battle.then(|| {
            Arc::new(BattleFragments::new(
                self.decision_catalog()
                    .battle_fragments()
                    .to_vec()
                    .into_boxed_slice(),
            ))
        });
        let curio_victory_blessings = has_runtime_battle
            .then(|| CurioVictoryBlessings::compile(self).map(Arc::new))
            .transpose()?;
        let initial_equations = entry
            .initial_equation
            .then(|| InitialEquations::compile(self).map(Arc::new))
            .transpose()?;
        if entry.layer_battle_route {
            match self.decision_catalog().battle_route().kind {
                BattleRoutePolicyKind::VersionedProjectPolicyOneBattlePerLayerWithDomainChoices => {
                }
            }
        }
        let encounter_pool = entry
            .layer_battle_route
            .then(|| -> Result<_, DivergentUniverseEntryFlowError> {
                Ok(Arc::new(EncounterPool::new(
                    self.decision_catalog().encounter_pool().clone(),
                    DomainChoices::new(
                        self,
                        self.decision_catalog()
                            .domain_choices()
                            .to_vec()
                            .into_boxed_slice(),
                    )?,
                )))
            })
            .transpose()?;
        let (graph, mut programs) = compile_graph(
            &layer_values,
            &progression,
            has_runtime_battle,
            occurrence_binding.as_deref(),
            battle_blessings.as_deref(),
            initial_equations.as_deref(),
            encounter_pool.as_deref(),
        )?;
        let tawot_service = entry
            .initial_tawot_service
            .map(|level| TawotService::compile(self, level, &economy).map(Arc::new))
            .transpose()?;
        let graph = if let Some(service) = &tawot_service {
            service.attach(graph, &mut programs)?
        } else {
            graph
        };
        let evolution_events = entry
            .layer_battle_route
            .then(|| EvolutionEvents::compile(self, layer_values.len()).map(Arc::new))
            .transpose()?;
        let graph = if let Some(selection) = &source_deck_selection {
            selection.attach(graph, &mut programs)?
        } else {
            graph
        };
        if let Some(events) = &evolution_events {
            events.wrap_programs(&mut programs)?;
        }
        let mut offers = initial_equations
            .as_ref()
            .map(|initial| initial.random_offer(graph_start_node(layer_values.len())?))
            .transpose()?
            .into_iter()
            .collect::<Vec<_>>();
        if let Some(pool) = &encounter_pool {
            offers.extend(pool.random_offers(layer_values.len())?);
        }
        let definition = GraphActivityDefinition::new(
            identity,
            graph,
            state,
            Arc::clone(&entry.participants),
            programs,
            None,
            ActivityRandomPolicies::new(Vec::new(), offers),
        )
        .map_err(|_| DivergentUniverseEntryFlowError::InvalidActivityDefinition)?;
        Ok(DivergentUniverseFlowInstance {
            position_battles: None,
            source_deck_selection,
            definition: Arc::new(definition),
            occurrence_binding,
            tawot_service,
            battle_blessings,
            curio_battle_stats,
            curio_battle_reactions,
            curio_battle_grants,
            battle_fragments,
            curio_victory_blessings,
            initial_equations,
            evolution_events,
            encounter_pool,
            component_digest: self.bundle.identity().component_digest().bytes(),
            input_snapshot: entry.input_snapshot,
            run_family: area.run_family(),
            area: area.id.clone(),
            difficulty: entry.difficulty,
            layers: area.layers.clone(),
            finish_conditions: catalog.profile().finish_conditions.clone(),
            cyclical_challenge: challenge.map(|value| value.id.clone()),
            weekly_modifier_ids: challenge.map_or_else(
                || Vec::new().into_boxed_slice(),
                |value| value.modifier_ids.clone(),
            ),
            progression: progression.projection,
            economy,
            permanent_unlocks: entry.permanent_unlocks,
            mapping_snapshot: entry.mapping_snapshot,
            room_policy: DivergentUniverseRoomPolicy::LogicalLayerCheckpointNoCandidatePromotion,
            first_ordinary_vertical_slice: entry.first_ordinary_vertical_slice,
            runtime_battle_route: entry.runtime_battle_route,
            vertical_slice_content,
        })
    }
}

/// Entry-compiled Activity definition plus exact mode identities.
#[derive(Clone, Debug)]
pub struct DivergentUniverseFlowInstance {
    pub(super) position_battles: Option<Arc<BoundBattleRooms>>,
    pub(super) source_deck_selection: Option<Arc<SourceDeckSelection>>,
    pub(super) tawot_service: Option<Arc<TawotService>>,
    pub(super) definition: Arc<GraphActivityDefinition>,
    pub(super) occurrence_binding: Option<Arc<OccurrenceBinding>>,
    pub(super) battle_blessings: Option<Arc<BattleBlessings>>,
    pub(super) curio_battle_stats: Option<Arc<CurioBattleStats>>,
    pub(super) curio_battle_reactions: Option<Arc<CurioBattleReactions>>,
    pub(super) curio_battle_grants: Option<Arc<CurioBattleGrants>>,
    pub(super) curio_victory_blessings: Option<Arc<CurioVictoryBlessings>>,
    pub(super) battle_fragments: Option<Arc<BattleFragments>>,
    pub(super) initial_equations: Option<Arc<InitialEquations>>,
    pub(super) evolution_events: Option<Arc<EvolutionEvents>>,
    pub(super) encounter_pool: Option<Arc<EncounterPool>>,
    pub(super) component_digest: [u8; 32],
    pub(super) input_snapshot: DivergentUniverseInputSnapshot,
    run_family: DivergentUniverseRunFamily,
    area: DivergentUniverseAreaId,
    difficulty: DivergentUniverseDifficultyId,
    layers: Box<[DivergentUniverseLayerId]>,
    pub(super) finish_conditions: Box<[DivergentUniverseFinishConditionId]>,
    cyclical_challenge: Option<DivergentUniverseCyclicalChallengeId>,
    weekly_modifier_ids: Box<[Box<str>]>,
    pub(super) progression: DivergentUniverseProgressionProjection,
    economy: DivergentUniverseEconomyProjection,
    pub(super) permanent_unlocks: Box<[u64]>,
    mapping_snapshot: Option<Arc<super::mapping::DivergentUniverseMappingSnapshot>>,
    pub(super) room_policy: DivergentUniverseRoomPolicy,
    first_ordinary_vertical_slice: bool,
    runtime_battle_route: bool,
    pub(super) vertical_slice_content:
        Option<Arc<super::vertical_slice::DivergentUniverseVerticalSliceContent>>,
}

impl DivergentUniverseFlowInstance {
    #[must_use]
    pub const fn run_family(&self) -> DivergentUniverseRunFamily {
        self.run_family
    }
    #[must_use]
    pub const fn area(&self) -> &DivergentUniverseAreaId {
        &self.area
    }
    #[must_use]
    pub const fn difficulty(&self) -> &DivergentUniverseDifficultyId {
        &self.difficulty
    }
    #[must_use]
    pub fn layers(&self) -> &[DivergentUniverseLayerId] {
        &self.layers
    }
    /// Exact source-only conditions associated with the active profile.
    ///
    /// Completing this Activity executes the run terminal transition; account
    /// progress consumers may evaluate these immutable source identities after
    /// settlement without granting the Activity authority over account state.
    #[must_use]
    pub fn finish_conditions(&self) -> &[DivergentUniverseFinishConditionId] {
        &self.finish_conditions
    }
    #[must_use]
    pub const fn cyclical_challenge(&self) -> Option<&DivergentUniverseCyclicalChallengeId> {
        self.cyclical_challenge.as_ref()
    }
    #[must_use]
    pub fn weekly_modifier_ids(&self) -> &[Box<str>] {
        &self.weekly_modifier_ids
    }
    #[must_use]
    pub const fn progression(&self) -> &DivergentUniverseProgressionProjection {
        &self.progression
    }
    #[must_use]
    pub const fn economy(&self) -> &DivergentUniverseEconomyProjection {
        &self.economy
    }
    #[must_use]
    pub const fn input_snapshot(&self) -> &DivergentUniverseInputSnapshot {
        &self.input_snapshot
    }
    #[must_use]
    pub fn mapping_snapshot(&self) -> Option<&super::mapping::DivergentUniverseMappingSnapshot> {
        self.mapping_snapshot.as_deref()
    }
    #[must_use]
    pub const fn room_policy(&self) -> DivergentUniverseRoomPolicy {
        self.room_policy
    }
    #[must_use]
    pub const fn is_first_ordinary_vertical_slice(&self) -> bool {
        self.first_ordinary_vertical_slice
    }
    #[must_use]
    pub const fn has_runtime_battle_route(&self) -> bool {
        self.first_ordinary_vertical_slice || self.runtime_battle_route
    }
    #[must_use]
    pub const fn definition(&self) -> &Arc<GraphActivityDefinition> {
        &self.definition
    }
    pub fn start(
        &self,
        instance: ActivityInstanceId,
        seed: ActivityMasterSeed,
    ) -> Result<GraphActivityResolution, GraphActivityStartError> {
        GraphActivity::start(Arc::clone(&self.definition), instance, seed)
    }
}

fn compile_identity(
    bundle: &DivergentUniverseBundleCandidate,
    decisions_digest: [u8; 32],
    area: &DivergentUniverseAreaDefinition,
    entry: &DivergentUniverseEntry,
) -> ActivityDefinitionIdentity {
    let definition = digest(&[
        b"starclock.divergent-universe.entry-flow.definition.v1",
        area.id.as_str().as_bytes(),
        entry.difficulty.as_str().as_bytes(),
        &entry.participants.digest().bytes(),
        &[battle_route_identity(entry)],
        &[u8::from(entry.initial_equation)],
        &[u8::from(entry.source_deck_selection)],
        &entry.initial_tawot_service.unwrap_or(0).to_le_bytes(),
        entry
            .initial_occurrence
            .as_ref()
            .map_or(&[][..], |variant| variant.as_str().as_bytes()),
    ]);
    let config = digest(&[
        b"starclock.divergent-universe.entry-flow.config.v1",
        SETTLEMENT_POLICY_IDENTITY,
        &bundle.identity().component_digest().bytes(),
        &decisions_digest,
        area.id.as_str().as_bytes(),
        &entry.input_snapshot.digest().bytes(),
        &entry_state_digest(entry),
        &[battle_route_identity(entry)],
    ]);
    ActivityDefinitionIdentity::new(
        ActivityDefinitionId::new(22).expect("non-zero definition ID"),
        ActivityDefinitionDigest::new(definition).expect("SHA-256 is non-zero"),
        ActivityConfigDigest::new(config).expect("SHA-256 is non-zero"),
    )
}

fn entry_state_digest(entry: &DivergentUniverseEntry) -> [u8; 32] {
    let mut hash = CanonicalDigestBuilder::new();
    digest_part(&mut hash, b"starclock.divergent-universe.entry-state.v1");
    digest_part(&mut hash, &[u8::from(entry.initial_equation)]);
    digest_part(&mut hash, &[u8::from(entry.source_deck_selection)]);
    digest_part(
        &mut hash,
        &entry.initial_tawot_service.unwrap_or(0).to_le_bytes(),
    );
    digest_part(&mut hash, &[u8::from(entry.layer_battle_route)]);
    digest_part(
        &mut hash,
        entry
            .initial_occurrence
            .as_ref()
            .map_or(&[][..], |variant| variant.as_str().as_bytes()),
    );
    for value in entry.permanent_unlocks.iter().copied() {
        digest_part(&mut hash, &value.to_le_bytes());
    }
    if let Some(value) = &entry.astronomical {
        let mode = match value.mode {
            super::progression::DivergentUniverseAstronomicalMode::StarPioneer => 1,
            super::progression::DivergentUniverseAstronomicalMode::Practice => 2,
        };
        digest_part(&mut hash, &[mode]);
        digest_part(&mut hash, value.division.as_str().as_bytes());
        digest_part(&mut hash, value.protocol.as_str().as_bytes());
        digest_part(&mut hash, &value.cognoculi.to_le_bytes());
        digest_part(
            &mut hash,
            &[u8::from(value.ordinary_difficulty_five_complete)],
        );
    }
    if let Some(value) = &entry.cyclical_refresh {
        digest_part(&mut hash, &value.epoch.to_le_bytes());
        digest_part(&mut hash, value.challenge.as_str().as_bytes());
    }
    if let Some(value) = &entry.mapping_snapshot {
        digest_part(&mut hash, &value.digest().bytes());
    }
    hash.finalize()
}

const fn battle_route_identity(entry: &DivergentUniverseEntry) -> u8 {
    if entry.runtime_battle_route {
        2
    } else if entry.first_ordinary_vertical_slice {
        1
    } else {
        0
    }
}

fn digest_part(hash: &mut CanonicalDigestBuilder, value: &[u8]) {
    let length = u64::try_from(value.len()).expect("slice length fits u64");
    hash.update(length.to_le_bytes());
    hash.update(value);
}

fn digest(parts: &[&[u8]]) -> [u8; 32] {
    let mut hash = CanonicalDigestBuilder::new();
    for part in parts {
        let length = u64::try_from(part.len()).expect("slice length fits u64");
        hash.update(length.to_le_bytes());
        hash.update(part);
    }
    hash.finalize()
}
fn stable_text(value: &str) -> u64 {
    let bytes = digest(&[
        b"starclock.divergent-universe.stable-value.v1",
        value.as_bytes(),
    ]);
    let mut raw = [0_u8; 8];
    raw.copy_from_slice(&bytes[..8]);
    u64::from_le_bytes(raw).max(1)
}
fn stable_index<T, K: PartialEq>(
    values: &[T],
    selected: &K,
    key: impl Fn(&T) -> &K,
) -> Result<u64, DivergentUniverseEntryFlowError> {
    values
        .iter()
        .position(|value| key(value) == selected)
        .and_then(|index| u64::try_from(index + 1).ok())
        .ok_or(DivergentUniverseEntryFlowError::InvalidActivityDefinition)
}
#[derive(Debug)]
pub enum DivergentUniverseEntryFlowError {
    Data(DivergentUniverseDataError),
    Decisions(DecisionDataError),
    Occurrence(OccurrenceBindingError),
    UnknownArea,
    DifficultyAreaMismatch,
    HistoricalModuleRejected,
    MissingResidentEntry,
    MissingCyclicalBinding,
    InvalidPermanentUnlocks,
    InvalidActivityDefinition,
    EvolutionProgram(ActivityProgramDefinitionError),
    VerticalSliceSelectionMismatch,
    ConflictingBattleRoutes,
    BattleRouteRequiresMapping,
    VerticalSlice(super::vertical_slice::DivergentUniverseVerticalSliceError),
    Progression(DivergentUniverseProgressionRuntimeError),
    Economy(DivergentUniverseEconomyError),
    Snapshot(DivergentUniverseSnapshotError),
    Mapping(super::mapping::DivergentUniverseMappingRuntimeError),
}
impl From<DivergentUniverseDataError> for DivergentUniverseEntryFlowError {
    fn from(value: DivergentUniverseDataError) -> Self {
        Self::Data(value)
    }
}
impl From<DivergentUniverseProgressionRuntimeError> for DivergentUniverseEntryFlowError {
    fn from(value: DivergentUniverseProgressionRuntimeError) -> Self {
        Self::Progression(value)
    }
}
impl From<DivergentUniverseEconomyError> for DivergentUniverseEntryFlowError {
    fn from(value: DivergentUniverseEconomyError) -> Self {
        Self::Economy(value)
    }
}
impl From<DivergentUniverseSnapshotError> for DivergentUniverseEntryFlowError {
    fn from(value: DivergentUniverseSnapshotError) -> Self {
        Self::Snapshot(value)
    }
}
impl From<super::mapping::DivergentUniverseMappingRuntimeError>
    for DivergentUniverseEntryFlowError
{
    fn from(value: super::mapping::DivergentUniverseMappingRuntimeError) -> Self {
        Self::Mapping(value)
    }
}
impl From<super::vertical_slice::DivergentUniverseVerticalSliceError>
    for DivergentUniverseEntryFlowError
{
    fn from(value: super::vertical_slice::DivergentUniverseVerticalSliceError) -> Self {
        Self::VerticalSlice(value)
    }
}
impl std::fmt::Display for DivergentUniverseEntryFlowError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "invalid Divergent Universe entry flow: {self:?}")
    }
}
impl std::error::Error for DivergentUniverseEntryFlowError {}
