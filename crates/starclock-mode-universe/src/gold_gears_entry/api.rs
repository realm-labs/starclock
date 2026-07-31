//! Public Gold and Gears entry and entry-compiled instance types.

use std::sync::Arc;

use starclock_activity::{
    ActivityEdgeId, ActivityGraphDefinition, ActivityProgramDefinition, ActivityRngStreams,
    ActivitySlotId, ActivityStateDefinition, ActivityTransactionState, ActivityValue, NodeId,
    ParticipantLock,
};

use super::{
    EXPECTED_PROFILE_KEY, GoldAndGearsEntryError,
    cognition::CognitionRuntimeCatalog,
    content_link_runtime::GoldAndGearsContentRuntimeCatalog,
    conundrum_runtime::{CompiledConundrumRuntime, ConundrumRuntimeCatalog},
    dice_face::{DiceFaceRuntimeCatalog, RuntimeDiceFace},
    dice_loadout::DiceLoadoutRuntimeCatalog,
    dice_passive::{
        GoldAndGearsDicePassiveEvent, allows_same_domain_movement, compile_passive,
        path_boost_stacks, persists_general_buff_faces, preserves_knowledge_domains,
    },
    dice_resolution::{
        CompiledDiceRuntime, DiceRuntimeCatalog, compile_cheat, compile_plane_start,
        compile_reroll, compile_roll, resolution_face, resolution_kind,
    },
    knowledge::KnowledgeRuntimeCatalog,
    knowledge_execution::{
        KnowledgeFaceContext, compile_collapse, compile_countdown_initial_adjustment,
        compile_domain_entry, compile_face_effect, compile_mark_for_collapse, knowledge_countdown,
        knowledge_nodes, movement_targets,
    },
    knowledge_resolution::{
        GoldAndGearsKnowledgeResolution, KnowledgeResolutionContext, compile_resolution,
    },
    map_overlay::{MapRuntimeCatalog, NODE_STATE_BLANKED},
    neural_runtime::{CompiledNeuralRuntime, NeuralRuntimeCatalog},
    plane_transition::PlaneTransitionRuntimeCatalog,
    progression_runtime::{CompiledProgressionRuntime, ProgressionRuntimeCatalog},
    state::compile_state,
    topology::compile_topology,
    validate::{
        canonical_completed_areas, canonical_neural_network, canonical_unlocked_dice,
        validate_conundrum, validate_participants,
    },
};
use crate::{
    gold_gears_content::GoldAndGearsContentCatalog,
    gold_gears_structural::{AreaDefinition, AreaGroup, GoldAndGearsStructuralCatalog},
    gold_gears_unique::GoldAndGearsUniqueCatalog,
};

/// Entry-policy revision that resolves `G14-R01`.
pub const GOLD_AND_GEARS_ENTRY_REVISION: &str = "gold-and-gears-entry-policy-v1";

/// Versioned root-board and forward-edge topology construction policy.
pub const GOLD_AND_GEARS_TOPOLOGY_REVISION: &str = "gold-and-gears-topology-policy-v1";

/// Caller-owned selections for one Gold and Gears run.
///
/// The six dice faces retain slot order. Neural nodes, completed formal areas,
/// and unlocked dice are sets: compilation canonicalizes them and rejects
/// duplicates. No omitted selection is filled with a random or hidden default.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoldAndGearsEntry {
    area: Box<str>,
    path: Box<str>,
    custom_dice: Box<str>,
    dice_faces: Box<[Box<str>]>,
    participants: ParticipantLock,
    neural_network: Box<[Box<str>]>,
    stats_conundrum: u8,
    auxiliary_conundrum: u8,
    completed_formal_areas: Box<[Box<str>]>,
    trailblaze_bonus: Option<Box<str>>,
    unlocked_dice: Box<[Box<str>]>,
}

impl GoldAndGearsEntry {
    #[must_use]
    pub fn new(
        area: impl Into<Box<str>>,
        path: impl Into<Box<str>>,
        custom_dice: impl Into<Box<str>>,
        dice_faces: Vec<String>,
        participants: ParticipantLock,
    ) -> Self {
        Self {
            area: area.into(),
            path: path.into(),
            custom_dice: custom_dice.into(),
            dice_faces: boxed_strings(dice_faces),
            participants,
            neural_network: Box::new([]),
            stats_conundrum: 0,
            auxiliary_conundrum: 0,
            completed_formal_areas: Box::new([]),
            trailblaze_bonus: None,
            unlocked_dice: Box::new([]),
        }
    }

    #[must_use]
    pub fn with_neural_network(mut self, nodes: Vec<String>) -> Self {
        self.neural_network = boxed_strings(nodes);
        self
    }

    #[must_use]
    pub fn with_conundrum(
        mut self,
        stats: u8,
        auxiliary: u8,
        completed_formal_areas: Vec<String>,
    ) -> Self {
        self.stats_conundrum = stats;
        self.auxiliary_conundrum = auxiliary;
        self.completed_formal_areas = boxed_strings(completed_formal_areas);
        self
    }

    #[must_use]
    pub fn with_trailblaze_bonus(mut self, bonus: impl Into<Box<str>>) -> Self {
        self.trailblaze_bonus = Some(bonus.into());
        self
    }

    #[must_use]
    pub fn with_unlocked_dice(mut self, dice: Vec<String>) -> Self {
        self.unlocked_dice = boxed_strings(dice);
        self
    }

    #[must_use]
    pub fn area(&self) -> &str {
        &self.area
    }

    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    #[must_use]
    pub fn custom_dice(&self) -> &str {
        &self.custom_dice
    }

    #[must_use]
    pub fn dice_faces(&self) -> impl ExactSizeIterator<Item = &str> {
        self.dice_faces.iter().map(Box::as_ref)
    }

    #[must_use]
    pub const fn participants(&self) -> &ParticipantLock {
        &self.participants
    }

    #[must_use]
    pub fn neural_network(&self) -> impl ExactSizeIterator<Item = &str> {
        self.neural_network.iter().map(Box::as_ref)
    }

    #[must_use]
    pub const fn stats_conundrum(&self) -> u8 {
        self.stats_conundrum
    }

    #[must_use]
    pub const fn auxiliary_conundrum(&self) -> u8 {
        self.auxiliary_conundrum
    }

    #[must_use]
    pub fn trailblaze_bonus(&self) -> Option<&str> {
        self.trailblaze_bonus.as_deref()
    }
}

/// Shared immutable catalog facade and the only Gold entry compiler.
#[derive(Clone, Debug)]
pub struct GoldAndGearsRuntimeFactory {
    pub(super) structural: Arc<GoldAndGearsStructuralCatalog>,
    pub(super) unique: Arc<GoldAndGearsUniqueCatalog>,
    pub(super) map: Arc<MapRuntimeCatalog>,
    pub(super) cognition: Arc<CognitionRuntimeCatalog>,
    pub(super) transitions: Arc<PlaneTransitionRuntimeCatalog>,
    pub(super) dice_loadouts: Arc<DiceLoadoutRuntimeCatalog>,
    pub(super) dice_runtime: Arc<DiceRuntimeCatalog>,
    pub(super) dice_faces: Arc<DiceFaceRuntimeCatalog>,
    pub(super) knowledge: Arc<KnowledgeRuntimeCatalog>,
    pub(super) neural: Arc<NeuralRuntimeCatalog>,
    pub(super) conundrum: Arc<ConundrumRuntimeCatalog>,
    pub(super) progression: Arc<ProgressionRuntimeCatalog>,
    pub(super) content_runtime: Arc<GoldAndGearsContentRuntimeCatalog>,
}

impl GoldAndGearsRuntimeFactory {
    /// Loads and validates the exact Candidate component used by this phase.
    ///
    /// Core combat and shared Universe components are added to this same
    /// factory before its production release boundary is enabled.
    pub fn load_candidate(bytes: &[u8]) -> Result<Self, GoldAndGearsEntryError> {
        let structural = GoldAndGearsStructuralCatalog::load(bytes)
            .map_err(|_| GoldAndGearsEntryError::InvalidCatalog)?;
        let unique = GoldAndGearsUniqueCatalog::load(bytes)
            .map_err(|_| GoldAndGearsEntryError::InvalidCatalog)?;
        let content = GoldAndGearsContentCatalog::load(bytes)
            .map_err(|_| GoldAndGearsEntryError::InvalidCatalog)?;
        if structural.bundle != unique.bundle
            || structural.bundle != content.bundle
            || !structural
                .profiles
                .iter()
                .any(|profile| profile.stable_key.as_ref() == EXPECTED_PROFILE_KEY)
        {
            return Err(GoldAndGearsEntryError::InvalidCatalog);
        }
        let map = MapRuntimeCatalog::compile(&structural, &content)?;
        let content_runtime = GoldAndGearsContentRuntimeCatalog::compile(&content, &unique)?;
        let cognition = CognitionRuntimeCatalog::compile(&unique)?;
        let transitions = PlaneTransitionRuntimeCatalog::compile(&structural)?;
        let dice_loadouts = DiceLoadoutRuntimeCatalog::compile(&unique)?;
        let dice_runtime = DiceRuntimeCatalog::compile(&unique)?;
        let dice_faces = DiceFaceRuntimeCatalog::compile(&unique)?;
        let knowledge = KnowledgeRuntimeCatalog::compile(&unique)?;
        let neural = NeuralRuntimeCatalog::compile(&unique)?;
        let conundrum = ConundrumRuntimeCatalog::compile(&unique)?;
        let progression = ProgressionRuntimeCatalog::compile(&unique)?;
        Ok(Self {
            structural: Arc::new(structural),
            unique: Arc::new(unique),
            map: Arc::new(map),
            cognition: Arc::new(cognition),
            transitions: Arc::new(transitions),
            dice_loadouts: Arc::new(dice_loadouts),
            dice_runtime: Arc::new(dice_runtime),
            dice_faces: Arc::new(dice_faces),
            knowledge: Arc::new(knowledge),
            neural: Arc::new(neural),
            conundrum: Arc::new(conundrum),
            progression: Arc::new(progression),
            content_runtime: Arc::new(content_runtime),
        })
    }

    /// Validates every selected input and compiles exactly one generic
    /// Activity state profile. It performs no random draw.
    pub fn compile_entry(
        &self,
        entry: GoldAndGearsEntry,
    ) -> Result<GoldAndGearsRuntimeInstance, GoldAndGearsEntryError> {
        validate_participants(entry.participants.policy())?;
        let area = self.formal_area(&entry.area)?;
        let path = self
            .unique
            .paths
            .iter()
            .find(|path| path.identity.stable_key.as_ref() == entry.path.as_ref())
            .ok_or_else(|| GoldAndGearsEntryError::UnknownPath(entry.path.clone()))?;
        let unlocked_dice = canonical_unlocked_dice(&self.unique, &entry.unlocked_dice)?;
        let dice = self
            .unique
            .dice
            .iter()
            .find(|dice| dice.identity.stable_key.as_ref() == entry.custom_dice.as_ref())
            .ok_or_else(|| GoldAndGearsEntryError::UnknownDice(entry.custom_dice.clone()))?;
        if !dice.available_by_default
            && unlocked_dice
                .binary_search_by(|candidate| candidate.as_ref().cmp(&entry.custom_dice))
                .is_err()
        {
            return Err(GoldAndGearsEntryError::LockedDice(
                entry.custom_dice.clone(),
            ));
        }
        let neural = canonical_neural_network(&self.unique, &entry.neural_network)?;
        let neural_runtime = self.neural.select(&neural)?;
        let loadout = self.dice_loadouts.compile_loadout(
            &self.unique,
            dice,
            &entry.dice_faces,
            &neural,
            &unlocked_dice,
        )?;
        let dice_runtime = self.dice_runtime.select(
            &dice.identity.stable_key,
            &path.identity.stable_key,
            &neural,
        )?;
        let dice_face_runtime = self.dice_faces.select(&loadout.faces)?;
        let completed_areas =
            canonical_completed_areas(&self.structural, &entry.completed_formal_areas)?;
        validate_conundrum(
            &self.unique,
            area,
            entry.stats_conundrum,
            entry.auxiliary_conundrum,
            &completed_areas,
        )?;
        let conundrum_runtime = self
            .conundrum
            .select(entry.stats_conundrum, entry.auxiliary_conundrum)?;
        let trailblaze_bonus = entry
            .trailblaze_bonus
            .as_deref()
            .map(|key| {
                self.unique
                    .trailblaze_bonuses
                    .iter()
                    .find(|bonus| bonus.identity.stable_key.as_ref() == key)
                    .ok_or_else(|| GoldAndGearsEntryError::UnknownTrailblazeBonus(key.into()))
            })
            .transpose()?;
        if let Some(bonus) = trailblaze_bonus
            && !neural_runtime.allows_trailblaze_bonus(&bonus.identity.stable_key)
        {
            return Err(GoldAndGearsEntryError::LockedTrailblazeBonus(
                bonus.identity.stable_key.clone(),
            ));
        }
        let progression_runtime = self.progression.select(
            &path.identity.stable_key,
            trailblaze_bonus.map(|bonus| bonus.identity.stable_key.as_ref()),
            dice_runtime.path_value_id,
            dice_runtime.path_boost_value_scaled,
        )?;
        let (cognition_minimum, cognition_maximum) = self.cognition.bounds(area)?;
        let topology = compile_topology(&self.structural, area)?;
        let initial_cosmic_fragments = conundrum_runtime
            .initial_cosmic_fragments(super::state_layout::INITIAL_COSMIC_FRAGMENTS)?;
        let initial_dice_rerolls =
            conundrum_runtime.initial_dice_rerolls(super::state_layout::INITIAL_DICE_REROLLS)?;
        let state = compile_state(
            area,
            path.identity.id.0,
            dice.identity.id.0,
            &loadout.faces,
            &loadout.maximum_rarities,
            &neural,
            entry.stats_conundrum,
            entry.auxiliary_conundrum,
            trailblaze_bonus.map(|bonus| bonus.identity.id.0),
            dice_runtime.path_value_id,
            dice_runtime.path_trigger_interval,
            dice_runtime.path_boost_value_scaled,
            self.cognition.initial(),
            cognition_minimum,
            cognition_maximum,
            initial_cosmic_fragments,
            initial_dice_rerolls,
            conundrum_runtime.berserk_state(),
        )?
        .with_logical_scopes(topology.scopes);

        Ok(GoldAndGearsRuntimeInstance {
            area: area.stable_key.clone(),
            difficulty: area.difficulty,
            path: path.identity.stable_key.clone(),
            custom_dice: dice.identity.stable_key.clone(),
            dice_faces: loadout
                .faces
                .iter()
                .map(|face| face.identity.stable_key.clone())
                .collect::<Vec<_>>()
                .into_boxed_slice(),
            dice_slot_max_rarities: loadout.maximum_rarities,
            eligible_dice_faces: loadout.eligible_faces,
            suggestive_dice_faces: loadout.suggestive_faces,
            recommended_dice_faces: loadout.recommended_faces,
            dice_face_ids: loadout
                .faces
                .iter()
                .map(|face| (face.identity.stable_key.clone(), face.identity.id.0))
                .collect::<Vec<_>>()
                .into_boxed_slice(),
            dice_runtime,
            dice_face_runtime,
            participants: Arc::new(entry.participants),
            neural_network: neural
                .iter()
                .map(|node| node.identity.stable_key.clone())
                .collect::<Vec<_>>()
                .into_boxed_slice(),
            neural_runtime,
            conundrum_runtime,
            progression_runtime,
            stats_conundrum: entry.stats_conundrum,
            auxiliary_conundrum: entry.auxiliary_conundrum,
            trailblaze_bonus: trailblaze_bonus.map(|bonus| bonus.identity.stable_key.clone()),
            state,
            graph: topology.graph,
            planes: topology
                .planes
                .iter()
                .map(|plane| plane.plane_key.clone())
                .collect::<Vec<_>>()
                .into_boxed_slice(),
            plane_starts: topology
                .planes
                .iter()
                .map(|plane| plane.start)
                .collect::<Vec<_>>()
                .into_boxed_slice(),
            plane_ends: topology
                .planes
                .iter()
                .map(|plane| plane.end)
                .collect::<Vec<_>>()
                .into_boxed_slice(),
            chessboards: topology
                .planes
                .iter()
                .map(|plane| plane.chessboard_key.clone())
                .collect::<Vec<_>>()
                .into_boxed_slice(),
            map: Arc::clone(&self.map),
            knowledge: Arc::clone(&self.knowledge),
            cognition: Arc::clone(&self.cognition),
            transitions: Arc::clone(&self.transitions),
            progression_catalog: Arc::clone(&self.progression),
            content_runtime: Arc::clone(&self.content_runtime),
        })
    }

    fn formal_area(&self, key: &str) -> Result<&AreaDefinition, GoldAndGearsEntryError> {
        let area = self
            .structural
            .areas
            .iter()
            .find(|area| area.stable_key.as_ref() == key)
            .ok_or_else(|| GoldAndGearsEntryError::UnknownArea(key.into()))?;
        if area.group != AreaGroup::Formal {
            return Err(GoldAndGearsEntryError::GuideArea(key.into()));
        }
        Ok(area)
    }
}

/// Entry-compiled immutable Activity profile.
///
/// The immutable three-plane graph is compiled at entry. Later batches attach
/// its generic programs, mutable overlays and runtime controller.
#[derive(Clone, Debug)]
pub struct GoldAndGearsRuntimeInstance {
    area: Box<str>,
    difficulty: u8,
    path: Box<str>,
    custom_dice: Box<str>,
    dice_faces: Box<[Box<str>]>,
    dice_slot_max_rarities: Box<[u8]>,
    eligible_dice_faces: Box<[Box<[Box<str>]>]>,
    suggestive_dice_faces: Box<[Box<str>]>,
    recommended_dice_faces: Box<[Box<str>]>,
    dice_face_ids: Box<[(Box<str>, u32)]>,
    dice_runtime: CompiledDiceRuntime,
    dice_face_runtime: Box<[RuntimeDiceFace]>,
    participants: Arc<ParticipantLock>,
    neural_network: Box<[Box<str>]>,
    pub(super) neural_runtime: CompiledNeuralRuntime,
    pub(super) conundrum_runtime: CompiledConundrumRuntime,
    pub(super) progression_runtime: CompiledProgressionRuntime,
    stats_conundrum: u8,
    auxiliary_conundrum: u8,
    trailblaze_bonus: Option<Box<str>>,
    state: ActivityStateDefinition,
    graph: ActivityGraphDefinition,
    planes: Box<[Box<str>]>,
    plane_starts: Box<[NodeId]>,
    plane_ends: Box<[NodeId]>,
    chessboards: Box<[Box<str>]>,
    map: Arc<MapRuntimeCatalog>,
    knowledge: Arc<KnowledgeRuntimeCatalog>,
    cognition: Arc<CognitionRuntimeCatalog>,
    transitions: Arc<PlaneTransitionRuntimeCatalog>,
    pub(super) progression_catalog: Arc<ProgressionRuntimeCatalog>,
    pub(super) content_runtime: Arc<GoldAndGearsContentRuntimeCatalog>,
}

impl GoldAndGearsRuntimeInstance {
    #[must_use]
    pub fn area(&self) -> &str {
        &self.area
    }

    #[must_use]
    pub const fn difficulty(&self) -> u8 {
        self.difficulty
    }

    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    #[must_use]
    pub fn custom_dice(&self) -> &str {
        &self.custom_dice
    }

    #[must_use]
    pub fn dice_faces(&self) -> impl ExactSizeIterator<Item = &str> {
        self.dice_faces.iter().map(Box::as_ref)
    }

    /// Returns the effective rarity ceiling for each of the six stable slots.
    ///
    /// Selected Neural slot upgrades are already applied.
    pub fn dice_slot_max_rarities(&self) -> impl ExactSizeIterator<Item = u8> + '_ {
        self.dice_slot_max_rarities.iter().copied()
    }

    /// Returns canonical eligible faces for one 1-based slot after applying
    /// unlock, color/slot, Custom Dice and effective-rarity constraints.
    pub fn eligible_dice_faces(
        &self,
        slot_index: u8,
    ) -> Option<impl ExactSizeIterator<Item = &str> + '_> {
        self.eligible_dice_faces
            .get(usize::from(slot_index.checked_sub(1)?))
            .map(|faces| faces.iter().map(Box::as_ref))
    }

    /// Returns the authored suggestive pool filtered to legal unlocked faces.
    pub fn suggestive_dice_faces(&self) -> impl ExactSizeIterator<Item = &str> {
        self.suggestive_dice_faces.iter().map(Box::as_ref)
    }

    /// Returns the authored recommended pool filtered to legal unlocked faces.
    pub fn recommended_dice_faces(&self) -> impl ExactSizeIterator<Item = &str> {
        self.recommended_dice_faces.iter().map(Box::as_ref)
    }

    /// Returns the activation boundary (1 immediate, 2 after movement,
    /// 3 next battle) for a selected loadout face.
    #[must_use]
    pub fn dice_face_activation_stage(&self, face: &str) -> Option<u8> {
        self.face_runtime(face)
            .map(RuntimeDiceFace::activation_stage)
    }

    /// Returns the executable target contract for a selected loadout face.
    #[must_use]
    pub fn dice_face_target_contract(&self, face: &str) -> Option<&'static str> {
        self.face_runtime(face)
            .map(RuntimeDiceFace::target_contract)
    }

    /// Returns the exact selector policy for a selected loadout face.
    #[must_use]
    pub fn dice_face_selector(&self, face: &str) -> Option<&'static str> {
        self.face_runtime(face).map(RuntimeDiceFace::selector_name)
    }

    /// Returns the maximum number of Spawn-selected targets, when random.
    #[must_use]
    pub fn dice_face_random_target_maximum(&self, face: &str) -> Option<u8> {
        self.face_runtime(face)
            .and_then(RuntimeDiceFace::random_target_maximum)
    }

    /// Returns exact face parameters in signed millionths.
    pub fn dice_face_parameters_scaled(
        &self,
        face: &str,
    ) -> Option<impl ExactSizeIterator<Item = i64> + '_> {
        self.face_runtime(face)
            .map(|runtime| runtime.parameters_scaled().iter().copied())
    }

    /// Returns exact private effect identities lowered as non-zero integers.
    pub fn dice_face_effect_ids(
        &self,
        face: &str,
    ) -> Option<impl ExactSizeIterator<Item = u64> + '_> {
        self.face_runtime(face)
            .map(|runtime| runtime.effect_ids().iter().copied())
    }

    /// Returns typed mechanical codes after validating the numeric tag join.
    pub fn dice_face_mechanical_codes(
        &self,
        face: &str,
    ) -> Option<impl ExactSizeIterator<Item = &'static str> + '_> {
        self.face_runtime(face)
            .map(RuntimeDiceFace::mechanical_codes)
    }

    /// Returns the explicit empty-candidate disposition.
    #[must_use]
    pub fn dice_face_no_target_behavior(&self, face: &str) -> Option<&'static str> {
        self.face_runtime(face)
            .map(RuntimeDiceFace::no_target_behavior)
    }

    /// Returns exact private effect IDs attached to the selected dice's
    /// initial lifecycle contribution.
    pub fn dice_initial_effect_ids(&self) -> impl ExactSizeIterator<Item = &str> {
        self.dice_runtime.initial_effect_ids.iter().map(Box::as_ref)
    }

    /// Returns exact private effect IDs attached to the selected dice's
    /// passive lifecycle contribution.
    pub fn dice_passive_effect_ids(&self) -> impl ExactSizeIterator<Item = &str> {
        self.dice_runtime.passive_effect_ids.iter().map(Box::as_ref)
    }

    /// Returns the selected dice/path trigger stat.
    #[must_use]
    pub fn dice_path_boost_stat(&self) -> &str {
        &self.dice_runtime.path_boost_stat
    }

    /// Returns the exact positive trigger interval.
    #[must_use]
    pub const fn dice_path_trigger_interval(&self) -> i64 {
        self.dice_runtime.path_trigger_interval
    }

    /// Returns the exact six-decimal fixed-point boost value in millionths.
    #[must_use]
    pub const fn dice_path_boost_value_scaled(&self) -> i64 {
        self.dice_runtime.path_boost_value_scaled
    }

    /// Returns the authored unit carried by the selected dice/path value.
    #[must_use]
    pub fn dice_path_boost_unit(&self) -> &str {
        &self.dice_runtime.path_boost_unit
    }

    /// Returns the canonical numeric parameters for the three authored effect
    /// parts in signed millionths.
    pub fn dice_initial_parameters_scaled(&self) -> impl ExactSizeIterator<Item = i64> + '_ {
        self.dice_runtime.initial_parameters_scaled.iter().copied()
    }

    pub fn dice_passive_parameters_scaled(&self) -> impl ExactSizeIterator<Item = i64> + '_ {
        self.dice_runtime.passive_parameters_scaled.iter().copied()
    }

    pub fn dice_path_trigger_parameters_scaled(&self) -> impl ExactSizeIterator<Item = i64> + '_ {
        self.dice_runtime
            .path_trigger_parameters_scaled
            .iter()
            .copied()
    }

    /// Compiles all immediate selected-dice consequences for one validated
    /// Activity fact and records downstream-owned contributions explicitly.
    pub fn compile_dice_passive(
        &self,
        state: &ActivityTransactionState,
        event: GoldAndGearsDicePassiveEvent,
    ) -> Result<Option<ActivityProgramDefinition>, GoldAndGearsEntryError> {
        compile_passive(&self.dice_runtime, state, event)
    }

    /// Returns the accumulated selected-Path boost stack count.
    #[must_use]
    pub fn dice_path_boost_stacks(&self, state: &ActivityTransactionState) -> Option<u32> {
        path_boost_stacks(state)
    }

    /// Whether the selected dice permits movement to any same-domain target.
    #[must_use]
    pub const fn dice_allows_same_domain_movement(&self) -> bool {
        allows_same_domain_movement(&self.dice_runtime)
    }

    /// Whether Knowledge protects a domain from collapse for this dice.
    #[must_use]
    pub const fn dice_preserves_knowledge_domains(&self) -> bool {
        preserves_knowledge_domains(&self.dice_runtime)
    }

    /// Whether rolled General Buff faces persist and activate next room.
    #[must_use]
    pub const fn dice_persists_general_buff_faces(&self) -> bool {
        persists_general_buff_faces(&self.dice_runtime)
    }

    /// Compiles this plane's Custom Dice initial activation. Immediate resource
    /// changes commit now; map/Knowledge contributions are recorded as typed
    /// deferred work for their owning runtime boundaries.
    pub fn compile_dice_plane_start(
        &self,
        plane_layer: u8,
    ) -> Result<Option<ActivityProgramDefinition>, GoldAndGearsEntryError> {
        compile_plane_start(&self.dice_runtime, plane_layer)
    }

    /// Uniformly rolls the six selected faces in stable face-ID order using
    /// only the Activity Spawn stream.
    pub fn compile_dice_roll(
        &self,
        state: &ActivityTransactionState,
        rng: &mut ActivityRngStreams,
    ) -> Result<ActivityProgramDefinition, GoldAndGearsEntryError> {
        compile_roll(state, &self.dice_face_ids, rng)
    }

    /// Consumes one reroll and resolves through the Spawn stream. When the
    /// selected Neural exclusion leaves no candidate, the prior face is kept,
    /// the attempt is consumed and no draw occurs.
    pub fn compile_dice_reroll(
        &self,
        state: &ActivityTransactionState,
        rng: &mut ActivityRngStreams,
    ) -> Result<ActivityProgramDefinition, GoldAndGearsEntryError> {
        compile_reroll(
            state,
            &self.dice_face_ids,
            self.dice_runtime.reroll_excludes_previous,
            rng,
        )
    }

    /// Consumes one cheat to select an exact face from the six-slot loadout.
    /// Cheats never draw RNG.
    pub fn compile_dice_cheat(
        &self,
        state: &ActivityTransactionState,
        selected_face: &str,
    ) -> Result<ActivityProgramDefinition, GoldAndGearsEntryError> {
        compile_cheat(state, &self.dice_face_ids, selected_face)
    }

    /// Activates the currently resolved face through its validated selector.
    ///
    /// Explicit selectors require one eligible target. Random selectors derive
    /// canonical candidates from the current board overlay and use only Spawn.
    /// Global/event-derived faces reject an explicit node.
    pub fn compile_dice_face_activation(
        &self,
        state: &ActivityTransactionState,
        explicit_target: Option<NodeId>,
        rng: &mut ActivityRngStreams,
    ) -> Result<ActivityProgramDefinition, GoldAndGearsEntryError> {
        let face = self
            .dice_resolution_face(state)
            .ok_or(GoldAndGearsEntryError::DiceFaceNotRolled)?;
        let runtime = self
            .face_runtime(face)
            .ok_or(GoldAndGearsEntryError::DiceFaceNotRolled)?;
        let candidates =
            self.map
                .dice_face_candidates(state, &self.graph, runtime.selector_name())?;
        runtime.compile_activation(state, &candidates, explicit_target, rng)
    }

    /// Commits the authored no-effect result for a rolled face whose required
    /// Curio/Negative-Curio pool is empty. Other faces fail closed.
    pub fn compile_dice_face_empty_content(
        &self,
        state: &ActivityTransactionState,
    ) -> Result<ActivityProgramDefinition, GoldAndGearsEntryError> {
        let face = self
            .dice_resolution_face(state)
            .ok_or(GoldAndGearsEntryError::DiceFaceNotRolled)?;
        self.face_runtime(face)
            .ok_or(GoldAndGearsEntryError::DiceFaceNotRolled)?
            .compile_empty_content(state)
    }

    /// Executes the Knowledge-owned consequence of the currently resolved
    /// dice face. A selected source is supplied as `anchor`; a selector-owned
    /// destination is supplied as `explicit_target`.
    ///
    /// Empty random/all candidate sets commit the authored no-effect marker
    /// without consuming RNG. Rejected explicit selections leave RNG intact.
    pub fn compile_knowledge_face_effect(
        &self,
        state: &ActivityTransactionState,
        anchor: Option<NodeId>,
        explicit_target: Option<NodeId>,
        rng: &mut ActivityRngStreams,
    ) -> Result<Option<ActivityProgramDefinition>, GoldAndGearsEntryError> {
        compile_face_effect(
            KnowledgeFaceContext {
                catalog: &self.knowledge,
                map: &self.map,
                graph: &self.graph,
                dice: &self.dice_runtime,
            },
            state,
            anchor,
            explicit_target,
            rng,
        )
    }

    /// Compiles movement, after-movement face work, Knowledge mutation,
    /// selected-dice callbacks, collapse and derived rewards into one ordered
    /// Activity transaction.
    pub fn compile_knowledge_resolution(
        &self,
        state: &ActivityTransactionState,
        request: &GoldAndGearsKnowledgeResolution,
        rng: &mut ActivityRngStreams,
    ) -> Result<ActivityProgramDefinition, GoldAndGearsEntryError> {
        compile_resolution(
            KnowledgeResolutionContext {
                catalog: &self.knowledge,
                map: &self.map,
                graph: &self.graph,
                dice: &self.dice_runtime,
            },
            state,
            request,
            rng,
        )
    }

    /// Returns the authored lifecycle boundary for a Knowledge-bound face.
    #[must_use]
    pub fn knowledge_face_trigger(&self, face: &str) -> Option<&'static str> {
        self.dice_face_ids
            .iter()
            .find(|(key, _)| key.as_ref() == face)
            .and_then(|(_, id)| self.knowledge.rule_for_face(*id))
            .map(super::knowledge::RuntimeKnowledgeRule::trigger_name)
    }

    /// Returns all stable Knowledge movement-override destinations for the
    /// currently resolved movement face.
    pub fn knowledge_movement_targets(
        &self,
        state: &ActivityTransactionState,
    ) -> Result<Box<[NodeId]>, GoldAndGearsEntryError> {
        movement_targets(&self.knowledge, &self.map, &self.graph, state)
    }

    /// Marks one Knowledge domain for the later collapse boundary.
    pub fn compile_knowledge_mark_for_collapse(
        &self,
        state: &ActivityTransactionState,
        target: NodeId,
    ) -> Result<ActivityProgramDefinition, GoldAndGearsEntryError> {
        compile_mark_for_collapse(state, target)
    }

    /// Resolves one already-marked Knowledge-domain collapse. The selected
    /// Custom Dice may preserve the domain or contribute exact collapse
    /// rewards; otherwise the board overlay is blanked atomically.
    pub fn compile_knowledge_collapse(
        &self,
        state: &ActivityTransactionState,
        target: NodeId,
    ) -> Result<ActivityProgramDefinition, GoldAndGearsEntryError> {
        compile_collapse(&self.map, &self.dice_runtime, state, target)
    }

    /// Executes selected-dice callbacks for entering a Knowledge domain.
    pub fn compile_knowledge_domain_entry(
        &self,
        state: &ActivityTransactionState,
        target: NodeId,
    ) -> Result<ActivityProgramDefinition, GoldAndGearsEntryError> {
        compile_domain_entry(&self.dice_runtime, state, target)
    }

    /// Applies the released Countdown-dice initial reduction to the current
    /// plane counter. Other selected dice return no program.
    pub fn compile_countdown_initial_adjustment(
        &self,
        state: &ActivityTransactionState,
    ) -> Result<Option<ActivityProgramDefinition>, GoldAndGearsEntryError> {
        compile_countdown_initial_adjustment(&self.dice_runtime, state)
    }

    /// Returns canonical Knowledge nodes in stable node-ID order.
    #[must_use]
    pub fn knowledge_nodes(&self, state: &ActivityTransactionState) -> Box<[NodeId]> {
        knowledge_nodes(state)
    }

    /// Returns the current plane Countdown/action-point value.
    #[must_use]
    pub fn knowledge_countdown(&self, state: &ActivityTransactionState) -> i64 {
        knowledge_countdown(state)
    }

    #[must_use]
    pub fn dice_resolution_face<'a>(&'a self, state: &ActivityTransactionState) -> Option<&'a str> {
        resolution_face(state, &self.dice_face_ids)
    }

    #[must_use]
    pub fn dice_resolution_kind(&self, state: &ActivityTransactionState) -> Option<u8> {
        resolution_kind(state)
    }

    fn face_runtime(&self, face: &str) -> Option<&RuntimeDiceFace> {
        self.dice_face_ids
            .iter()
            .position(|(key, _)| key.as_ref() == face)
            .and_then(|index| self.dice_face_runtime.get(index))
    }

    #[must_use]
    pub const fn participants(&self) -> &Arc<ParticipantLock> {
        &self.participants
    }

    #[must_use]
    pub fn neural_network(&self) -> impl ExactSizeIterator<Item = &str> {
        self.neural_network.iter().map(Box::as_ref)
    }

    #[must_use]
    pub const fn stats_conundrum(&self) -> u8 {
        self.stats_conundrum
    }

    #[must_use]
    pub const fn auxiliary_conundrum(&self) -> u8 {
        self.auxiliary_conundrum
    }

    #[must_use]
    pub fn trailblaze_bonus(&self) -> Option<&str> {
        self.trailblaze_bonus.as_deref()
    }

    #[must_use]
    pub const fn state_definition(&self) -> &ActivityStateDefinition {
        &self.state
    }

    #[must_use]
    pub const fn graph_definition(&self) -> &ActivityGraphDefinition {
        &self.graph
    }

    #[must_use]
    pub fn planes(&self) -> impl ExactSizeIterator<Item = &str> {
        self.planes.iter().map(Box::as_ref)
    }

    #[must_use]
    pub fn chessboards(&self) -> impl ExactSizeIterator<Item = &str> {
        self.chessboards.iter().map(Box::as_ref)
    }

    #[must_use]
    pub fn plane_starts(&self) -> impl ExactSizeIterator<Item = NodeId> + '_ {
        self.plane_starts.iter().copied()
    }

    #[must_use]
    pub fn plane_ends(&self) -> impl ExactSizeIterator<Item = NodeId> + '_ {
        self.plane_ends.iter().copied()
    }

    /// Returns the six released boss-display candidates in stable source order.
    pub fn boss_choices(&self) -> impl ExactSizeIterator<Item = &str> {
        self.transitions.choices()
    }

    /// Records one caller-explicit released boss choice for a plane.
    pub fn compile_boss_selection(
        &self,
        plane_layer: u8,
        boss: &str,
    ) -> Result<ActivityProgramDefinition, GoldAndGearsEntryError> {
        self.transitions.compile_selection(plane_layer, boss)
    }

    /// Atomically evaluates Cognition/Secrets and enters the next plane, or
    /// enters the synthetic completed terminal after the third plane.
    pub fn compile_plane_completion(
        &self,
        plane_layer: u8,
    ) -> Result<ActivityProgramDefinition, GoldAndGearsEntryError> {
        self.transitions.compile_completion(
            &self.cognition,
            &self.area,
            &self.graph,
            &self.plane_ends,
            plane_layer,
        )
    }

    /// Compiles deterministic initial node/domain/beacon overlay operations for
    /// one authored plane. Random choices use only the caller's Activity Graph
    /// stream and canonical authored candidate order.
    pub fn compile_plane_creation(
        &self,
        plane_ordinal: usize,
        rng: &mut ActivityRngStreams,
    ) -> Result<ActivityProgramDefinition, GoldAndGearsEntryError> {
        let board = self
            .chessboards
            .get(plane_ordinal)
            .ok_or(GoldAndGearsEntryError::InvalidPlaneCount)?;
        self.map.compile_creation(board, rng)
    }

    /// Applies the selected released map event before compiling the same
    /// plane's block-creation operations.
    pub fn compile_map_event_then_creation(
        &self,
        plane_ordinal: usize,
        trigger: &str,
        parameter: u32,
        rng: &mut ActivityRngStreams,
    ) -> Result<ActivityProgramDefinition, GoldAndGearsEntryError> {
        let board = self
            .chessboards
            .get(plane_ordinal)
            .ok_or(GoldAndGearsEntryError::InvalidPlaneCount)?;
        self.map
            .compile_event_then_creation(board, trigger, parameter, rng)
    }

    /// Compiles an exact node replacement through ordinary bounded counters.
    pub fn compile_node_replacement(
        &self,
        target: NodeId,
        domain: &str,
        beacon: Option<&str>,
    ) -> Result<ActivityProgramDefinition, GoldAndGearsEntryError> {
        self.map.compile_replacement(target, domain, beacon)
    }

    /// Compiles an exact domain/beacon copy from one immutable graph node to another.
    pub fn compile_node_copy(
        &self,
        source: NodeId,
        target: NodeId,
    ) -> Result<ActivityProgramDefinition, GoldAndGearsEntryError> {
        self.map.compile_copy(source, target)
    }

    /// Compiles an exact blanking operation without editing the immutable graph.
    pub fn compile_node_blanking(
        &self,
        target: NodeId,
    ) -> Result<ActivityProgramDefinition, GoldAndGearsEntryError> {
        self.map.compile_blank(target)
    }

    /// Compiles one checked Cognition delta followed by global and selected-area
    /// clamping. The program consumes no RNG.
    pub fn compile_cognition_adjustment(
        &self,
        delta: i64,
    ) -> Result<ActivityProgramDefinition, GoldAndGearsEntryError> {
        self.cognition.compile_adjustment(&self.area, delta)
    }

    /// Reapplies the selected-area bounds to the exact carried Cognition value.
    pub fn compile_cognition_carry(
        &self,
    ) -> Result<ActivityProgramDefinition, GoldAndGearsEntryError> {
        self.cognition.compile_carry(&self.area)
    }

    /// Compiles the deterministic Secret evaluation performed after the
    /// current plane's boss has been defeated.
    pub fn compile_plane_boss_cognition_evaluation(
        &self,
        plane_layer: u8,
    ) -> Result<ActivityProgramDefinition, GoldAndGearsEntryError> {
        self.cognition
            .compile_plane_boss_evaluation(&self.area, plane_layer)
    }

    /// Returns all currently eligible Secrets in canonical policy order. The
    /// plane-boss evaluator unlocks the first item, or none when this is empty.
    pub fn secret_frontier(
        &self,
        state: &ActivityTransactionState,
        plane_layer: u8,
    ) -> Result<Box<[Box<str>]>, GoldAndGearsEntryError> {
        self.cognition.frontier(&self.area, state, plane_layer)
    }

    /// Returns canonical static outgoing edges whose targets are not blanked
    /// by the current bounded board overlay.
    #[must_use]
    pub fn legal_routes(
        &self,
        state: &ActivityTransactionState,
        source: NodeId,
    ) -> Box<[ActivityEdgeId]> {
        self.graph
            .outgoing(source)
            .filter(|edge| !node_is_blanked(state, edge.to()))
            .map(|edge| edge.id())
            .collect::<Vec<_>>()
            .into_boxed_slice()
    }
}

fn node_is_blanked(state: &ActivityTransactionState, node: NodeId) -> bool {
    let slot = ActivitySlotId::new(super::state_layout::BOARD_NODE_STATE_SLOT)
        .expect("static Gold and Gears slot is non-zero");
    match state.slot(slot) {
        Some(ActivityValue::BoundedCounterMap(values)) => values
            .binary_search_by_key(&u64::from(node.get()), |(key, _)| *key)
            .ok()
            .is_some_and(|index| values[index].1 == NODE_STATE_BLANKED),
        _ => false,
    }
}

fn boxed_strings(values: Vec<String>) -> Box<[Box<str>]> {
    values
        .into_iter()
        .map(String::into_boxed_str)
        .collect::<Vec<_>>()
        .into_boxed_slice()
}
