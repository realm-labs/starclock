//! Immutable enemy-AI, boss-phase, and encounter orchestration definitions.

use crate::{
    AbilityId, AiCandidateId, AiGraphId, AiStateId, AiTransitionId, EncounterWaveId,
    EnemyDefinitionId, EnemyPhaseId, FormationIndex, ProgramId, SelectorId,
    rng::types::DrawPurpose, rule::model::ConditionExpr,
};

/// How a legal candidate is selected after canonical priority ordering.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum AiCandidateSelection {
    /// Select the first legal candidate without consuming RNG.
    FirstLegal,
    /// Participate in one explicitly authored weighted behavior draw.
    WeightedDraw { weight: u32, purpose: DrawPurpose },
}

/// Authored behavior when a selected candidate has no legal target.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum AiNoTargetFallback {
    UseFallbackAbility(AbilityId),
    StayInState,
    Transition(AiStateId),
    SkipAction,
    Fault,
}

/// Boundary at which an AI transition may be evaluated.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum AiTransitionTiming {
    AutomaticBeforeDecision,
    AfterAction,
    AfterPhase,
    Explicit,
}

/// One finite action candidate. Ordering is `(priority, ability, id)`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AiCandidateDefinition {
    id: AiCandidateId,
    ability: AbilityId,
    condition: ConditionExpr,
    target_selector: SelectorId,
    priority: i32,
    selection: AiCandidateSelection,
    no_target: AiNoTargetFallback,
}

impl AiCandidateDefinition {
    #[must_use]
    pub const fn new(
        id: AiCandidateId,
        ability: AbilityId,
        condition: ConditionExpr,
        target_selector: SelectorId,
        priority: i32,
        selection: AiCandidateSelection,
        no_target: AiNoTargetFallback,
    ) -> Self {
        Self {
            id,
            ability,
            condition,
            target_selector,
            priority,
            selection,
            no_target,
        }
    }
    #[must_use]
    pub const fn id(&self) -> AiCandidateId {
        self.id
    }
    #[must_use]
    pub const fn ability(&self) -> AbilityId {
        self.ability
    }
    #[must_use]
    pub const fn condition(&self) -> &ConditionExpr {
        &self.condition
    }
    #[must_use]
    pub const fn target_selector(&self) -> SelectorId {
        self.target_selector
    }
    #[must_use]
    pub const fn priority(&self) -> i32 {
        self.priority
    }
    #[must_use]
    pub const fn selection(&self) -> AiCandidateSelection {
        self.selection
    }
    #[must_use]
    pub const fn no_target(&self) -> AiNoTargetFallback {
        self.no_target
    }
}

/// One finite state transition. Ordering is `(priority, target state, id)`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AiTransitionDefinition {
    id: AiTransitionId,
    target: AiStateId,
    condition: ConditionExpr,
    priority: i32,
    timing: AiTransitionTiming,
}

impl AiTransitionDefinition {
    #[must_use]
    pub const fn new(
        id: AiTransitionId,
        target: AiStateId,
        condition: ConditionExpr,
        priority: i32,
        timing: AiTransitionTiming,
    ) -> Self {
        Self {
            id,
            target,
            condition,
            priority,
            timing,
        }
    }
    #[must_use]
    pub const fn id(&self) -> AiTransitionId {
        self.id
    }
    #[must_use]
    pub const fn target(&self) -> AiStateId {
        self.target
    }
    #[must_use]
    pub const fn condition(&self) -> &ConditionExpr {
        &self.condition
    }
    #[must_use]
    pub const fn priority(&self) -> i32 {
        self.priority
    }
    #[must_use]
    pub const fn timing(&self) -> AiTransitionTiming {
        self.timing
    }
}

/// One AI state with a mandatory executable fallback.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AiStateDefinition {
    id: AiStateId,
    entry_program: Option<ProgramId>,
    mandatory_fallback: AbilityId,
    reset_turn_counter: bool,
    candidates: Box<[AiCandidateDefinition]>,
    transitions: Box<[AiTransitionDefinition]>,
}

impl AiStateDefinition {
    #[must_use]
    pub fn new(
        id: AiStateId,
        entry_program: Option<ProgramId>,
        mandatory_fallback: AbilityId,
        reset_turn_counter: bool,
        mut candidates: Vec<AiCandidateDefinition>,
        mut transitions: Vec<AiTransitionDefinition>,
    ) -> Self {
        candidates.sort_by_key(|item| (item.priority, item.ability, item.id));
        transitions.sort_by_key(|item| (item.priority, item.target, item.id));
        Self {
            id,
            entry_program,
            mandatory_fallback,
            reset_turn_counter,
            candidates: candidates.into_boxed_slice(),
            transitions: transitions.into_boxed_slice(),
        }
    }
    #[must_use]
    pub const fn id(&self) -> AiStateId {
        self.id
    }
    #[must_use]
    pub const fn entry_program(&self) -> Option<ProgramId> {
        self.entry_program
    }
    #[must_use]
    pub const fn mandatory_fallback(&self) -> AbilityId {
        self.mandatory_fallback
    }
    #[must_use]
    pub const fn resets_turn_counter(&self) -> bool {
        self.reset_turn_counter
    }
    #[must_use]
    pub fn candidates(&self) -> &[AiCandidateDefinition] {
        &self.candidates
    }
    #[must_use]
    pub fn transitions(&self) -> &[AiTransitionDefinition] {
        &self.transitions
    }
}

/// Finite deterministic state machine shared by one or more enemy variants.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AiGraphDefinition {
    id: AiGraphId,
    initial_state: AiStateId,
    automatic_transition_budget: u16,
    states: Box<[AiStateDefinition]>,
}

impl AiGraphDefinition {
    #[must_use]
    pub fn new(
        id: AiGraphId,
        initial_state: AiStateId,
        automatic_transition_budget: u16,
        mut states: Vec<AiStateDefinition>,
    ) -> Option<Self> {
        if automatic_transition_budget == 0 || states.is_empty() {
            return None;
        }
        states.sort_by_key(AiStateDefinition::id);
        if states.windows(2).any(|pair| pair[0].id == pair[1].id)
            || states
                .binary_search_by_key(&initial_state, AiStateDefinition::id)
                .is_err()
        {
            return None;
        }
        Some(Self {
            id,
            initial_state,
            automatic_transition_budget,
            states: states.into_boxed_slice(),
        })
    }
    #[must_use]
    pub const fn id(&self) -> AiGraphId {
        self.id
    }
    #[must_use]
    pub const fn initial_state(&self) -> AiStateId {
        self.initial_state
    }
    #[must_use]
    pub const fn automatic_transition_budget(&self) -> u16 {
        self.automatic_transition_budget
    }
    #[must_use]
    pub fn states(&self) -> &[AiStateDefinition] {
        &self.states
    }
    #[must_use]
    pub fn state(&self, id: AiStateId) -> Option<&AiStateDefinition> {
        self.states
            .binary_search_by_key(&id, AiStateDefinition::id)
            .ok()
            .map(|index| &self.states[index])
    }
}

/// Data-selected boss transition representation.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EnemyPhaseTransitionModel {
    TransformSameUnit,
    ReplaceLinkedVariant(EnemyDefinitionId),
    ExplicitWave,
}

/// Stable semantic relationship between an enemy owner and linked definition.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EnemyLinkKind {
    Summon,
    SharedHp,
    Part,
    Countdown,
    TimelineActor,
}

/// Deterministic behavior when the authored simultaneous-link limit is full.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum LinkOverflowPolicy {
    Reject,
    ReplaceOldest,
    ReplaceNewest,
    Skip,
}

/// Formation ownership of a linked enemy occurrence.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum LinkedFormationPolicy {
    NextAvailable,
    Fixed(FormationIndex),
    NoFormationSlot,
}

/// One validated definition-level summon/part/countdown relationship.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EnemyLinkDefinition {
    sequence: u16,
    linked_enemy: EnemyDefinitionId,
    kind: EnemyLinkKind,
    maximum_simultaneous: u16,
    overflow: LinkOverflowPolicy,
    owner_defeat: crate::OwnerLinkPolicy,
    wave: crate::WaveLinkPolicy,
    contributes_to_victory: bool,
    formation: LinkedFormationPolicy,
}

impl EnemyLinkDefinition {
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub const fn new(
        sequence: u16,
        linked_enemy: EnemyDefinitionId,
        kind: EnemyLinkKind,
        maximum_simultaneous: u16,
        overflow: LinkOverflowPolicy,
        owner_defeat: crate::OwnerLinkPolicy,
        wave: crate::WaveLinkPolicy,
        contributes_to_victory: bool,
        formation: LinkedFormationPolicy,
    ) -> Option<Self> {
        if sequence == 0 || maximum_simultaneous == 0 || maximum_simultaneous > 32 {
            None
        } else {
            Some(Self {
                sequence,
                linked_enemy,
                kind,
                maximum_simultaneous,
                overflow,
                owner_defeat,
                wave,
                contributes_to_victory,
                formation,
            })
        }
    }
    #[must_use]
    pub const fn sequence(self) -> u16 {
        self.sequence
    }
    #[must_use]
    pub const fn linked_enemy(self) -> EnemyDefinitionId {
        self.linked_enemy
    }
    #[must_use]
    pub const fn kind(self) -> EnemyLinkKind {
        self.kind
    }
    #[must_use]
    pub const fn maximum_simultaneous(self) -> u16 {
        self.maximum_simultaneous
    }
    #[must_use]
    pub const fn overflow(self) -> LinkOverflowPolicy {
        self.overflow
    }
    #[must_use]
    pub const fn owner_defeat(self) -> crate::OwnerLinkPolicy {
        self.owner_defeat
    }
    #[must_use]
    pub const fn wave(self) -> crate::WaveLinkPolicy {
        self.wave
    }
    #[must_use]
    pub const fn contributes_to_victory(self) -> bool {
        self.contributes_to_victory
    }
    #[must_use]
    pub const fn formation(self) -> LinkedFormationPolicy {
        self.formation
    }
}

/// Carry/reset behavior for one boss-phase state family.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PhaseCarryPolicy {
    CarryExact,
    CarryRatio,
    Reset,
    Clear,
    ExplicitProgram(ProgramId),
}

/// Complete carry policy for a boss transition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EnemyPhaseCarry {
    pub hp: PhaseCarryPolicy,
    pub action_gauge: PhaseCarryPolicy,
    pub effects: PhaseCarryPolicy,
    pub toughness: PhaseCarryPolicy,
    pub summons: PhaseCarryPolicy,
}

/// One ordered authored boss phase.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EnemyPhaseDefinition {
    id: EnemyPhaseId,
    sequence: u16,
    entry_condition: ConditionExpr,
    exit_condition: ConditionExpr,
    replacement_priority: i32,
    ai_graph: AiGraphId,
    targetable: bool,
    transition: EnemyPhaseTransitionModel,
    entry_program: Option<ProgramId>,
    carry: EnemyPhaseCarry,
}

impl EnemyPhaseDefinition {
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub const fn new(
        id: EnemyPhaseId,
        sequence: u16,
        entry_condition: ConditionExpr,
        exit_condition: ConditionExpr,
        replacement_priority: i32,
        ai_graph: AiGraphId,
        targetable: bool,
        transition: EnemyPhaseTransitionModel,
        entry_program: Option<ProgramId>,
        carry: EnemyPhaseCarry,
    ) -> Self {
        Self {
            id,
            sequence,
            entry_condition,
            exit_condition,
            replacement_priority,
            ai_graph,
            targetable,
            transition,
            entry_program,
            carry,
        }
    }
    #[must_use]
    pub const fn id(&self) -> EnemyPhaseId {
        self.id
    }
    #[must_use]
    pub const fn sequence(&self) -> u16 {
        self.sequence
    }
    #[must_use]
    pub const fn entry_condition(&self) -> &ConditionExpr {
        &self.entry_condition
    }
    #[must_use]
    pub const fn exit_condition(&self) -> &ConditionExpr {
        &self.exit_condition
    }
    #[must_use]
    pub const fn replacement_priority(&self) -> i32 {
        self.replacement_priority
    }
    #[must_use]
    pub const fn ai_graph(&self) -> AiGraphId {
        self.ai_graph
    }
    #[must_use]
    pub const fn targetable(&self) -> bool {
        self.targetable
    }
    #[must_use]
    pub const fn transition(&self) -> EnemyPhaseTransitionModel {
        self.transition
    }
    #[must_use]
    pub const fn entry_program(&self) -> Option<ProgramId> {
        self.entry_program
    }
    #[must_use]
    pub const fn carry(&self) -> EnemyPhaseCarry {
        self.carry
    }
}

/// Allowed wave-advance boundary.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum WaveTransitionPolicy {
    AfterAction,
    AfterPhase,
    AfterHit,
    Explicit,
}

/// Carry/reset behavior for one state family between waves.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum WaveCarryPolicy {
    CarryExact,
    Reset,
    Clear,
    ExplicitProgram(ProgramId),
}

/// Complete wave-boundary carry policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WaveCarry {
    pub hp: WaveCarryPolicy,
    pub energy: WaveCarryPolicy,
    pub skill_points: WaveCarryPolicy,
    pub effects: WaveCarryPolicy,
    pub action_gauge: WaveCarryPolicy,
}

impl WaveCarry {
    pub const CARRY_ALL: Self = Self {
        hp: WaveCarryPolicy::CarryExact,
        energy: WaveCarryPolicy::CarryExact,
        skill_points: WaveCarryPolicy::CarryExact,
        effects: WaveCarryPolicy::CarryExact,
        action_gauge: WaveCarryPolicy::CarryExact,
    };
}

/// One exact hostile occurrence in a wave formation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WaveSlotDefinition {
    spawn_sequence: u16,
    formation: Option<FormationIndex>,
    enemy: EnemyDefinitionId,
    level_override: Option<u8>,
    initial_phase: Option<EnemyPhaseId>,
    required_for_victory: bool,
}

impl WaveSlotDefinition {
    #[must_use]
    pub const fn new(
        spawn_sequence: u16,
        formation: FormationIndex,
        enemy: EnemyDefinitionId,
        level_override: Option<u8>,
        initial_phase: Option<EnemyPhaseId>,
        required_for_victory: bool,
    ) -> Option<Self> {
        if spawn_sequence == 0 || matches!(level_override, Some(0 | 96..=u8::MAX)) {
            None
        } else {
            Some(Self {
                spawn_sequence,
                formation: Some(formation),
                enemy,
                level_override,
                initial_phase,
                required_for_victory,
            })
        }
    }
    #[must_use]
    pub const fn spawn_sequence(self) -> u16 {
        self.spawn_sequence
    }
    #[must_use]
    pub const fn formation(self) -> Option<FormationIndex> {
        self.formation
    }

    pub(super) const fn legacy(spawn_sequence: u16, enemy: EnemyDefinitionId) -> Self {
        Self {
            spawn_sequence,
            formation: None,
            enemy,
            level_override: None,
            initial_phase: None,
            required_for_victory: true,
        }
    }
    #[must_use]
    pub const fn enemy(self) -> EnemyDefinitionId {
        self.enemy
    }
    #[must_use]
    pub const fn level_override(self) -> Option<u8> {
        self.level_override
    }
    #[must_use]
    pub const fn initial_phase(self) -> Option<EnemyPhaseId> {
        self.initial_phase
    }
    #[must_use]
    pub const fn required_for_victory(self) -> bool {
        self.required_for_victory
    }
}

/// One ordered encounter wave with explicit boundary programs and carry policy.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EncounterWaveDefinition {
    id: EncounterWaveId,
    sequence: u16,
    entry_program: Option<ProgramId>,
    exit_program: Option<ProgramId>,
    carry: WaveCarry,
    slots: Box<[WaveSlotDefinition]>,
}

impl EncounterWaveDefinition {
    #[must_use]
    pub fn new(
        id: EncounterWaveId,
        sequence: u16,
        entry_program: Option<ProgramId>,
        exit_program: Option<ProgramId>,
        carry: WaveCarry,
        mut slots: Vec<WaveSlotDefinition>,
    ) -> Option<Self> {
        if sequence == 0 || slots.is_empty() {
            return None;
        }
        slots.sort_by_key(|slot| (slot.spawn_sequence, slot.formation, slot.enemy));
        if slots.windows(2).any(|pair| {
            pair[0].spawn_sequence == pair[1].spawn_sequence
                || (pair[0].formation.is_some() && pair[0].formation == pair[1].formation)
        }) {
            return None;
        }
        Some(Self {
            id,
            sequence,
            entry_program,
            exit_program,
            carry,
            slots: slots.into_boxed_slice(),
        })
    }
    #[must_use]
    pub const fn id(&self) -> EncounterWaveId {
        self.id
    }
    #[must_use]
    pub const fn sequence(&self) -> u16 {
        self.sequence
    }
    #[must_use]
    pub const fn entry_program(&self) -> Option<ProgramId> {
        self.entry_program
    }
    #[must_use]
    pub const fn exit_program(&self) -> Option<ProgramId> {
        self.exit_program
    }
    #[must_use]
    pub const fn carry(&self) -> WaveCarry {
        self.carry
    }
    #[must_use]
    pub fn slots(&self) -> &[WaveSlotDefinition] {
        &self.slots
    }
}
