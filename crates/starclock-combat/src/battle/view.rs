//! Line-limit exception: the closed battle observation contract stays together while detail records live in submodules.
mod team_resource;
mod timeline_detail;
mod unit_detail;

use super::{
    fault::BattleFault,
    model::BattlePhase,
    spec::{
        AssemblyDigest, BattleSeed, CombatInputDigest, CombatantSpecDigest, FormationIndex,
        ParticipantSource, TeamSide, UnitLevel,
    },
    state::{BattleClockState, BattleState},
};

use super::spec as parent_spec;
use crate::{
    ActionFrameId, ActionId, ActionOrigin, AiGraphId, AiStateId, ControlledAction, DispelCategory,
    DotDefinition, DurationClock, EffectCategory, EffectDefinitionId, EffectInstanceId,
    EffectSnapshotPolicy, EffectStackPolicy, EffectTeardownPolicy, EffectTickPhase,
    EnemyDefinitionId, EnemyPhaseId, Energy, EventId, LinkedEntity, LinkedEntityKind,
    ModifierDefinitionId as CrateModifierDefinitionId, ModifierInstanceId, OperationId,
    OwnerLinkPolicy, RawToughness, RuleId, RuleInstanceId, Scalar, SourceDefinitionId,
    Speed as CrateSpeed, StatValue, StateSlotDefinitionId, ToughnessLayerKind, ToughnessLayerSpec,
    WaveLinkPolicy,
    actor::{
        model::{LifeState, PresenceState},
        store::{FormationEntry, LinkState, TeamState, TimelineActorState, UnitState},
    },
    catalog::CatalogDigest,
    command::model::{ActionFrameInput, DecisionPoint},
    effect::{break_effect::BreakEffectState, shield::ShieldState, state::EffectState},
    formula::{
        model::CombatElement,
        shield::ShieldAbsorptionPolicy,
        toughness::{BaseBreakEffect, BreakDamageDefinition, EnemyRank},
    },
    id::{
        AbilityId, ActionBoundaryId, EncounterId, ModifierDefinitionId, PreparedActionId,
        RuleBundleId, ShieldInstanceId, SpawnSequence, TimelineActorId, UnitDefinitionId, UnitId,
        WaveInstanceId,
    },
    modifier::model::{ActiveModifier, StatQuery},
    numeric::domain::{ActionGauge, Hp, ShieldAmount, Speed},
    rule::{
        model::{OnceKey, RuleValue, SourceClass},
        state::RuleInstanceState,
    },
    timeline::state::{ActionFrameState, NormalTurnState},
    toughness::state::ToughnessLayerState,
};
pub use team_resource::TeamResourceView;
pub use timeline_detail::{PendingExtraTurnView, PendingReactionView, SequenceCursorsView};
pub use unit_detail::{CharacterResourceView, TemporaryWeaknessView, TransformationView};

/// Borrowed immutable projection of one authoritative battle state.
#[derive(Clone, Copy)]
pub struct BattleView<'a> {
    pub(crate) state: &'a BattleState,
}

impl<'a> BattleView<'a> {
    /// Returns the current top-level lifecycle phase.
    #[must_use]
    pub const fn phase(self) -> BattlePhase {
        self.state.phase
    }
    /// Returns the optional authoritative battle-local challenge clock.
    #[must_use]
    pub const fn clock(self) -> Option<BattleClockView<'a>> {
        match self.state.clock.as_ref() {
            Some(clock) => Some(BattleClockView { clock }),
            None => None,
        }
    }
    /// Returns the persisted terminal fault, if resolution faulted.
    #[must_use]
    pub const fn fault(self) -> Option<BattleFault> {
        self.state.fault
    }
    /// Returns the active decision, or `None` at a terminal boundary.
    #[must_use]
    pub const fn decision(self) -> Option<&'a DecisionPoint> {
        self.state.decision.as_ref()
    }
    /// Returns the count of accepted command commits.
    #[must_use]
    pub const fn committed_revision(self) -> u64 {
        self.state.committed_revision
    }
    /// Returns the authoritative raw RNG draw count.
    #[must_use]
    pub const fn rng_draw_count(self) -> u64 {
        self.state.rng.draw_count()
    }
    /// Returns immutable catalog/spec/seed compatibility identity.
    #[must_use]
    pub const fn identity(self) -> BattleIdentityView<'a> {
        BattleIdentityView { state: self.state }
    }
    /// Returns encounter and current wave identity.
    #[must_use]
    pub const fn encounter(self) -> EncounterView {
        EncounterView {
            definition: self.state.encounter.definition,
            wave: self.state.encounter.wave,
            number: self.state.encounter.number,
            total_waves: self.state.encounter.total_waves,
            spawn_defeats: self.state.encounter.spawn_defeats,
        }
    }
    /// Iterates active unit records in stable runtime-ID order.
    pub fn units_by_id(self) -> impl Iterator<Item = UnitView<'a>> + 'a {
        self.state
            .units
            .iter_by_id()
            .map(|state| UnitView { state })
    }
    /// Iterates occupied formation slots in canonical slot order for one side.
    pub fn formation(self, side: TeamSide) -> impl Iterator<Item = FormationView> + 'a {
        self.state.formations.on_side(side).map(FormationView::from)
    }
    /// Iterates timeline actors in stable runtime-ID order.
    pub fn timeline_actors(self) -> impl Iterator<Item = TimelineActorView<'a>> + 'a {
        self.state
            .actors
            .iter_by_id()
            .map(|state| TimelineActorView { state })
    }
    /// Iterates explicit owner/entity links in canonical insertion order.
    pub fn links(self) -> impl Iterator<Item = LinkView<'a>> + 'a {
        self.state
            .links
            .canonical_entries()
            .iter()
            .map(|state| LinkView { state })
    }
    /// Iterates active shield instances in stable runtime-ID order.
    pub fn shields_by_id(self) -> impl Iterator<Item = ShieldView<'a>> + 'a {
        self.state.shields.iter_by_id().map(|entry| ShieldView {
            state: entry.state,
            owner: entry.owner,
            policy: entry.policy,
        })
    }
    /// Iterates retained base Break effects in stable instance order.
    pub fn break_effects_by_id(self) -> impl Iterator<Item = BreakEffectView<'a>> + 'a {
        self.state
            .break_effects
            .iter_by_id()
            .map(|state| BreakEffectView { state })
    }
    /// Iterates every retained base Break-effect record, including expired entries.
    pub fn retained_break_effects_by_id(self) -> impl Iterator<Item = BreakEffectView<'a>> + 'a {
        self.state
            .break_effects
            .canonical_entries()
            .iter()
            .map(|state| BreakEffectView { state })
    }
    /// Iterates retained generic effect instances in stable instance order.
    pub fn effects_by_id(self) -> impl Iterator<Item = EffectView<'a>> + 'a {
        self.state
            .effects
            .iter_by_id()
            .map(|state| EffectView { state })
    }
    /// Returns the effective strongest-wins instance using the canonical comparator.
    #[must_use]
    pub fn strongest_effect(
        self,
        definition: EffectDefinitionId,
        target: UnitId,
    ) -> Option<EffectInstanceId> {
        self.state.effects.active_strongest(definition, target)
    }
    /// Iterates battle-bound rule instances in stable runtime order.
    pub fn rule_instances_by_id(self) -> impl Iterator<Item = RuleInstanceView<'a>> + 'a {
        self.state
            .rules
            .iter_by_id()
            .map(|state| RuleInstanceView { state })
    }
    /// Iterates battle-owned modifier instances in stable runtime-ID order.
    pub fn modifier_instances_by_id(self) -> impl Iterator<Item = ModifierInstanceView<'a>> + 'a {
        self.state
            .modifiers
            .iter_by_id()
            .map(|state| ModifierInstanceView { state })
    }
    /// Returns one side's team-scoped resources.
    #[must_use]
    pub fn team(self, side: TeamSide) -> TeamView<'a> {
        TeamView {
            state: self.state.teams.get(side),
        }
    }
    /// Returns the selected normal turn that persists across its decisions.
    #[must_use]
    pub fn active_turn(self) -> Option<ActiveTurnView> {
        self.state.timeline.active_turn.map(ActiveTurnView::from)
    }
    /// Returns the current stable boundary between independent actions, if any.
    #[must_use]
    pub fn action_boundary(self) -> Option<ActionBoundaryView> {
        self.state
            .timeline
            .boundary
            .as_ref()
            .map(|boundary| ActionBoundaryView {
                id: boundary.id,
                turn: ActiveTurnView::from(boundary.turn),
            })
    }
    /// Returns the action currently waiting for target or variant input.
    #[must_use]
    pub fn prepared_action(self) -> Option<PreparedActionView> {
        self.state
            .timeline
            .prepared_action
            .as_ref()
            .map(|prepared| PreparedActionView {
                id: prepared.id,
                actor: prepared.actor,
                ability: prepared.ability,
                suspended_boundary: prepared.boundary.id,
            })
    }
    /// Returns the declared segmented action currently waiting between segments.
    #[must_use]
    pub fn action_frame(self) -> Option<ActionFrameView<'a>> {
        self.state
            .timeline
            .action_frame
            .as_ref()
            .map(|frame| ActionFrameView { frame })
    }
    /// Iterates pending extra turns in their authoritative queue order.
    pub fn pending_extra_turns(self) -> impl Iterator<Item = PendingExtraTurnView> + 'a {
        self.state
            .timeline
            .extra_turns
            .iter()
            .copied()
            .map(PendingExtraTurnView::from)
    }
    /// Iterates queued reactions in their authoritative execution order.
    pub fn pending_reactions(self) -> impl Iterator<Item = PendingReactionView> + 'a {
        self.state
            .reactions
            .entries()
            .iter()
            .map(PendingReactionView::from)
    }
    /// Returns the authored concession policy retained by this battle.
    #[must_use]
    pub const fn concede_policy(self) -> parent_spec::ConcedePolicy {
        self.state.concede
    }
    /// Returns the canonical next-ID cursors used by deterministic allocation.
    #[must_use]
    pub const fn sequence_cursors(self) -> SequenceCursorsView {
        SequenceCursorsView::new(self.state.sequences.canonical_next_values())
    }
}

/// Borrowed clock projection without mode-specific presentation metadata.
#[derive(Clone, Copy)]
pub struct BattleClockView<'a> {
    clock: &'a BattleClockState,
}

impl BattleClockView<'_> {
    #[must_use]
    pub const fn remaining_cycles(self) -> Option<u16> {
        match self.clock {
            BattleClockState::Cycles {
                remaining_cycles, ..
            } => Some(*remaining_cycles),
            BattleClockState::ActionValue { .. } => None,
        }
    }

    #[must_use]
    pub const fn cycle_index(self) -> Option<u32> {
        match self.clock {
            BattleClockState::Cycles { cycle_index, .. } => Some(*cycle_index),
            BattleClockState::ActionValue { .. } => None,
        }
    }

    #[must_use]
    pub const fn elapsed_in_window_scaled(self) -> Option<i64> {
        match self.clock {
            BattleClockState::Cycles {
                elapsed_in_window_scaled,
                ..
            } => Some(*elapsed_in_window_scaled),
            BattleClockState::ActionValue { .. } => None,
        }
    }

    #[must_use]
    pub const fn remaining_action_value_scaled(self) -> Option<i64> {
        match self.clock {
            BattleClockState::Cycles { .. } => None,
            BattleClockState::ActionValue {
                remaining_scaled, ..
            } => Some(*remaining_scaled),
        }
    }
}

/// Immutable projection of one active shield instance.
#[derive(Clone, Copy)]
pub struct ShieldView<'a> {
    state: &'a ShieldState,
    owner: UnitId,
    policy: ShieldAbsorptionPolicy,
}

/// Immutable projection of one retained base Break effect.
#[derive(Clone, Copy)]
pub struct BreakEffectView<'a> {
    state: &'a BreakEffectState,
}

/// Immutable projection of one retained generic effect instance.
#[derive(Clone, Copy)]
pub struct EffectView<'a> {
    state: &'a EffectState,
}

/// Immutable projection of one battle-owned modifier instance.
#[derive(Clone, Copy)]
pub struct ModifierInstanceView<'a> {
    state: &'a ActiveModifier,
}

impl<'a> ModifierInstanceView<'a> {
    #[must_use]
    pub const fn id(self) -> ModifierInstanceId {
        self.state.instance
    }
    #[must_use]
    pub const fn definition(self) -> CrateModifierDefinitionId {
        self.state.definition
    }
    #[must_use]
    pub const fn owner(self) -> UnitId {
        self.state.owner
    }
    #[must_use]
    pub const fn subject(self) -> UnitId {
        self.state.subject
    }
    #[must_use]
    pub const fn source(self) -> SourceDefinitionId {
        self.state.source
    }
    #[must_use]
    pub const fn source_class(self) -> SourceClass {
        self.state.source_class
    }
    /// Returns the effect instance that owns this modifier attachment, if any.
    #[must_use]
    pub const fn source_effect(self) -> Option<EffectInstanceId> {
        self.state.source_effect
    }
    #[must_use]
    pub const fn insertion_sequence(self) -> u64 {
        self.state.insertion_sequence
    }
    #[must_use]
    pub const fn application_action(self) -> Option<ActionId> {
        self.state.application_action
    }
    pub fn slots(self) -> impl Iterator<Item = (StateSlotDefinitionId, &'a RuleValue)> {
        self.state.slots.iter().map(|(slot, value)| (*slot, value))
    }
    #[must_use]
    pub const fn captured_value(self) -> Option<Scalar> {
        self.state.captured_value
    }
    pub fn captured_stats(self) -> impl Iterator<Item = (&'a StatQuery, Scalar)> {
        self.state
            .captured_stats
            .iter()
            .map(|(query, value)| (query, *value))
    }
}

impl<'a> EffectView<'a> {
    #[must_use]
    pub const fn id(self) -> EffectInstanceId {
        self.state.id
    }
    #[must_use]
    pub const fn definition(self) -> EffectDefinitionId {
        self.state.definition
    }
    #[must_use]
    pub const fn source_definition(self) -> SourceDefinitionId {
        self.state.source_definition
    }
    #[must_use]
    pub const fn applier(self) -> UnitId {
        self.state.applier
    }
    #[must_use]
    pub const fn target(self) -> UnitId {
        self.state.target
    }
    #[must_use]
    pub const fn category(self) -> EffectCategory {
        self.state.category
    }
    #[must_use]
    pub const fn stacks(self) -> u16 {
        self.state.stacks
    }
    #[must_use]
    pub const fn remaining(self) -> Option<u16> {
        self.state.remaining
    }
    #[must_use]
    pub const fn duration_clock(self) -> DurationClock {
        self.state.duration_clock
    }
    #[must_use]
    pub const fn snapshot_policy(self) -> EffectSnapshotPolicy {
        self.state.snapshot_policy
    }
    #[must_use]
    pub const fn source_operation(self) -> OperationId {
        self.state.source_operation
    }
    #[must_use]
    pub const fn dispel(self) -> DispelCategory {
        self.state.dispel
    }
    #[must_use]
    pub const fn stack_limit(self) -> u16 {
        self.state.stack_limit
    }
    #[must_use]
    pub const fn tick_phase(self) -> EffectTickPhase {
        self.state.tick_phase
    }
    #[must_use]
    pub const fn stack_policy(self) -> EffectStackPolicy {
        self.state.stack_policy
    }
    #[must_use]
    pub const fn teardown_policy(self) -> EffectTeardownPolicy {
        self.state.teardown_policy
    }
    #[must_use]
    pub const fn application_priority(self) -> i32 {
        self.state.application_priority
    }
    #[must_use]
    pub const fn magnitude(self) -> Scalar {
        self.state.magnitude
    }
    #[must_use]
    pub fn tags(self) -> &'a [SourceDefinitionId] {
        &self.state.tags
    }
    #[must_use]
    pub fn controlled_actions(self) -> &'a [ControlledAction] {
        &self.state.controlled_actions
    }
    #[must_use]
    pub const fn dot(self) -> Option<DotDefinition> {
        self.state.dot
    }
    #[must_use]
    pub const fn application_sequence(self) -> u64 {
        self.state.application_sequence
    }
}

#[derive(Clone, Copy)]
pub struct RuleInstanceView<'a> {
    state: &'a RuleInstanceState,
}

impl<'a> RuleInstanceView<'a> {
    #[must_use]
    pub const fn id(self) -> RuleInstanceId {
        self.state.id
    }
    #[must_use]
    pub const fn rule(self) -> RuleId {
        self.state.rule
    }
    #[must_use]
    pub const fn owner(self) -> Option<UnitId> {
        self.state.owner
    }
    #[must_use]
    pub const fn source_effect(self) -> Option<EffectInstanceId> {
        self.state.source_effect
    }
    pub fn slots(self) -> impl Iterator<Item = (StateSlotDefinitionId, &'a RuleValue)> + 'a {
        self.state
            .slots
            .iter()
            .map(|(definition, value)| (definition.id(), value))
    }
    pub fn once_keys(self) -> impl Iterator<Item = OnceKey> + 'a {
        self.state.ledger.canonical_keys().copied()
    }
}

impl BreakEffectView<'_> {
    #[must_use]
    pub const fn id(self) -> EffectInstanceId {
        self.state.id
    }
    #[must_use]
    pub const fn owner(self) -> UnitId {
        self.state.owner
    }
    #[must_use]
    pub const fn applier(self) -> UnitId {
        self.state.applier
    }
    #[must_use]
    pub const fn source_definition(self) -> SourceDefinitionId {
        self.state.source_definition
    }
    #[must_use]
    pub const fn element(self) -> CombatElement {
        self.state.plan.element
    }
    #[must_use]
    pub const fn remaining_turns(self) -> u8 {
        self.state.remaining_turns
    }
    #[must_use]
    pub const fn stacks(self) -> u8 {
        self.state.stacks
    }
    #[must_use]
    pub const fn source_operation(self) -> OperationId {
        self.state.source_operation
    }
    #[must_use]
    pub const fn plan(self) -> BaseBreakEffect {
        self.state.plan
    }
    #[must_use]
    pub const fn damage(self) -> BreakDamageDefinition {
        self.state.damage
    }
    #[must_use]
    pub const fn speed_before(self) -> Option<CrateSpeed> {
        self.state.speed_before
    }
}

impl ShieldView<'_> {
    #[must_use]
    pub const fn id(self) -> ShieldInstanceId {
        self.state.id
    }

    #[must_use]
    pub const fn owner(self) -> UnitId {
        self.owner
    }

    #[must_use]
    pub const fn source_operation(self) -> OperationId {
        self.state.source_operation
    }

    #[must_use]
    pub const fn remaining(self) -> ShieldAmount {
        self.state.remaining
    }

    #[must_use]
    pub const fn source_effect(self) -> Option<EffectDefinitionId> {
        self.state.source_effect
    }

    #[must_use]
    pub const fn policy(self) -> ShieldAbsorptionPolicy {
        self.policy
    }
}

/// Immutable selected normal-turn ownership.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ActiveTurnView {
    actor: TimelineActorId,
    owner: UnitId,
    unit: UnitId,
    automatic: Option<(AbilityId, ActionOrigin)>,
    side: TeamSide,
    formation: FormationIndex,
    spawn: SpawnSequence,
    origin: ActionOrigin,
}

impl ActiveTurnView {
    /// Returns the timeline actor whose gauge reached the boundary.
    #[must_use]
    pub const fn actor(self) -> TimelineActorId {
        self.actor
    }
    /// Returns the unit that owns the selected timeline actor.
    #[must_use]
    pub const fn owner(self) -> UnitId {
        self.owner
    }
    /// Returns the target-capable unit represented by the timeline actor.
    #[must_use]
    pub const fn unit(self) -> UnitId {
        self.unit
    }
    /// Returns the automatic action bound to this turn, when present.
    #[must_use]
    pub const fn automatic(self) -> Option<(AbilityId, ActionOrigin)> {
        self.automatic
    }
    /// Returns the formation side that owns the selected turn.
    #[must_use]
    pub const fn side(self) -> TeamSide {
        self.side
    }
    /// Returns the selected owner's formation position.
    #[must_use]
    pub const fn formation(self) -> FormationIndex {
        self.formation
    }
    /// Returns the stable spawn-order tie breaker.
    #[must_use]
    pub const fn spawn_sequence(self) -> SpawnSequence {
        self.spawn
    }
    /// Returns whether this is a timeline turn or a granted extra turn.
    #[must_use]
    pub const fn origin(self) -> ActionOrigin {
        self.origin
    }
}

impl From<NormalTurnState> for ActiveTurnView {
    fn from(turn: NormalTurnState) -> Self {
        Self {
            actor: turn.actor,
            owner: turn.owner,
            unit: turn.unit,
            automatic: turn.automatic,
            side: turn.side,
            formation: turn.formation,
            spawn: turn.spawn,
            origin: turn.origin,
        }
    }
}

/// Immutable stable action-boundary state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ActionBoundaryView {
    id: ActionBoundaryId,
    turn: ActiveTurnView,
}

impl ActionBoundaryView {
    /// Returns the battle-local monotonic boundary identity.
    #[must_use]
    pub const fn id(self) -> ActionBoundaryId {
        self.id
    }
    /// Returns the normal turn suspended at this action boundary.
    #[must_use]
    pub const fn turn(self) -> ActiveTurnView {
        self.turn
    }
}

/// Immutable prepared-action identity and selected ability.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PreparedActionView {
    id: PreparedActionId,
    actor: UnitId,
    ability: AbilityId,
    suspended_boundary: ActionBoundaryId,
}

impl PreparedActionView {
    #[must_use]
    pub const fn id(self) -> PreparedActionId {
        self.id
    }
    #[must_use]
    pub const fn actor(self) -> UnitId {
        self.actor
    }
    #[must_use]
    pub const fn ability(self) -> AbilityId {
        self.ability
    }
    #[must_use]
    pub const fn suspended_boundary(self) -> ActionBoundaryId {
        self.suspended_boundary
    }
}

/// Borrowed immutable state for one declared segmented action.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ActionFrameView<'a> {
    frame: &'a ActionFrameState,
}

impl<'a> ActionFrameView<'a> {
    #[must_use]
    pub const fn id(self) -> ActionFrameId {
        self.frame.id
    }
    #[must_use]
    pub const fn action(self) -> ActionId {
        self.frame.action
    }
    #[must_use]
    pub const fn actor(self) -> UnitId {
        self.frame.actor
    }
    #[must_use]
    pub const fn owner(self) -> UnitId {
        self.frame.owner
    }
    #[must_use]
    pub const fn ability(self) -> AbilityId {
        self.frame.ability
    }
    #[must_use]
    pub const fn suspended_boundary(self) -> ActionBoundaryId {
        self.frame.boundary.id
    }
    #[must_use]
    pub const fn cursor(self) -> u16 {
        self.frame.cursor
    }
    #[must_use]
    pub const fn retained_primary(self) -> Option<UnitId> {
        self.frame.retained_targets.primary
    }
    #[must_use]
    pub fn retained_targets(self) -> &'a [UnitId] {
        &self.frame.retained_targets.targets
    }
    #[must_use]
    pub fn inputs(self) -> &'a [ActionFrameInput] {
        &self.frame.inputs
    }
    #[must_use]
    pub const fn parent_event(self) -> EventId {
        self.frame.parent
    }
    #[must_use]
    pub const fn paid(self) -> bool {
        self.frame.paid
    }
}

/// Immutable identity included in canonical state.
#[derive(Clone, Copy)]
pub struct BattleIdentityView<'a> {
    state: &'a BattleState,
}

impl<'a> BattleIdentityView<'a> {
    /// Returns the exact catalog digest.
    #[must_use]
    pub const fn catalog_digest(self) -> CatalogDigest {
        self.state.identity.catalog_digest
    }
    /// Returns the combat-owned canonical battle-input digest.
    #[must_use]
    pub const fn combat_input_digest(self) -> CombatInputDigest {
        self.state.identity.combat_input_digest
    }
    /// Returns opaque provenance for the outer assembly.
    #[must_use]
    pub const fn assembly_digest(self) -> AssemblyDigest {
        self.state.identity.assembly_digest
    }
    /// Returns the exact isolated battle seed.
    #[must_use]
    pub const fn seed(self) -> BattleSeed {
        self.state.identity.seed
    }
}

/// Immutable combatant projection without mutable store access.
#[derive(Clone, Copy)]
pub struct UnitView<'a> {
    state: &'a UnitState,
}

impl<'a> UnitView<'a> {
    /// Returns the stable battle-local unit identity.
    #[must_use]
    pub const fn id(self) -> UnitId {
        self.state.id
    }
    /// Returns the monotonic spawn order used by deterministic tie-breaking.
    #[must_use]
    pub const fn spawn_sequence(self) -> SpawnSequence {
        self.state.spawn
    }
    /// Returns the immutable combat-form definition.
    #[must_use]
    pub const fn form(self) -> UnitDefinitionId {
        self.state.form
    }
    /// Returns the generic player/encounter source binding.
    #[must_use]
    pub const fn source(self) -> ParticipantSource {
        self.state.source
    }
    /// Returns the formation side.
    #[must_use]
    pub const fn side(self) -> TeamSide {
        self.state.side
    }
    /// Returns the occupied formation slot.
    #[must_use]
    pub const fn formation(self) -> FormationIndex {
        self.state.formation
    }
    /// Returns the one-based encounter wave in which this unit enters.
    #[must_use]
    pub const fn entry_wave(self) -> u16 {
        self.state.entry_wave
    }
    /// Returns the checked level.
    #[must_use]
    pub const fn level(self) -> UnitLevel {
        self.state.level
    }
    /// Returns life independently from battlefield presence.
    #[must_use]
    pub const fn life(self) -> LifeState {
        self.state.life
    }
    /// Returns battlefield presence independently from life.
    #[must_use]
    pub const fn presence(self) -> PresenceState {
        self.state.presence
    }
    /// Returns current integral HP.
    #[must_use]
    pub const fn current_hp(self) -> Hp {
        self.state.current_hp
    }
    /// Returns maximum HP at battle entry before battle-local reductions.
    #[must_use]
    pub const fn initial_maximum_hp(self) -> Hp {
        self.state.initial_maximum_hp
    }
    /// Returns maximum integral HP.
    #[must_use]
    pub const fn maximum_hp(self) -> Hp {
        self.state.maximum_hp
    }
    /// Returns effective HP damage credited to this combatant in this battle.
    #[must_use]
    pub const fn damage_dealt(self) -> i64 {
        self.state.damage_dealt
    }
    /// Returns the immutable authored base ATK retained for staged queries.
    #[must_use]
    pub const fn base_attack(self) -> StatValue {
        self.state.base_attack
    }
    /// Returns the immutable authored base DEF retained for staged queries.
    #[must_use]
    pub const fn base_defense(self) -> StatValue {
        self.state.base_defense
    }
    /// Returns the immutable authored base SPD retained for staged queries.
    #[must_use]
    pub const fn base_speed(self) -> CrateSpeed {
        self.state.base_speed
    }
    /// Returns the immutable authored base Effect Hit Rate.
    #[must_use]
    pub const fn base_effect_hit_rate(self) -> Scalar {
        self.state.base_effect_hit_rate
    }
    /// Returns the immutable authored base Effect Resistance.
    #[must_use]
    pub const fn base_effect_resistance(self) -> Scalar {
        self.state.base_effect_resistance
    }
    /// Returns current personal Energy.
    #[must_use]
    pub const fn current_energy(self) -> Energy {
        self.state.current_energy
    }
    /// Returns maximum personal Energy.
    #[must_use]
    pub const fn maximum_energy(self) -> Energy {
        self.state.maximum_energy
    }
    /// Returns the authored encounter rank used by Break formulas and rules.
    #[must_use]
    pub const fn rank(self) -> EnemyRank {
        self.state.rank
    }
    /// Returns a named form-scoped resource and its cap.
    #[must_use]
    pub fn character_resource(self, stable_key: &str) -> Option<(Scalar, Scalar)> {
        self.state
            .resource(stable_key)
            .map(|resource| (resource.current, resource.maximum))
    }
    /// Iterates every form-scoped resource in canonical key order.
    pub fn character_resources(self) -> impl Iterator<Item = CharacterResourceView<'a>> + 'a {
        self.state
            .resources
            .iter()
            .map(|state| CharacterResourceView { state })
    }
    /// Returns the canonical selected ability set.
    #[must_use]
    pub fn abilities(self) -> &'a [AbilityId] {
        &self.state.abilities
    }
    /// Returns whether the unit currently retains an authored transformation.
    #[must_use]
    pub const fn is_transformed(self) -> bool {
        self.state.transformation.is_some()
    }
    /// Returns the transform-owned countdown actor when present.
    #[must_use]
    pub const fn transformation_countdown(self) -> Option<TimelineActorId> {
        match &self.state.transformation {
            None => None,
            Some(state) => state.countdown_actor,
        }
    }
    /// Returns the complete retained transformation state, when active.
    #[must_use]
    pub const fn transformation(self) -> Option<TransformationView<'a>> {
        match &self.state.transformation {
            None => None,
            Some(state) => Some(TransformationView { state }),
        }
    }
    /// Returns the bound enemy definition for authored hostile occurrences.
    #[must_use]
    pub const fn enemy_definition(self) -> Option<EnemyDefinitionId> {
        match self.state.enemy {
            None => None,
            Some(enemy) => Some(enemy.definition),
        }
    }
    /// Returns the authoritative AI graph/state cursor for an executable enemy.
    #[must_use]
    pub const fn enemy_ai_state(self) -> Option<(AiGraphId, AiStateId, u16)> {
        match self.state.enemy {
            None => None,
            Some(enemy) => Some((enemy.graph, enemy.state, enemy.turn_counter)),
        }
    }
    /// Returns the current authored boss phase, when one is active.
    #[must_use]
    pub const fn enemy_phase(self) -> Option<EnemyPhaseId> {
        match self.state.enemy {
            None => None,
            Some(enemy) => enemy.phase,
        }
    }
    /// Returns canonical selected rule bundles.
    #[must_use]
    pub fn rule_bundles(self) -> &'a [RuleBundleId] {
        &self.state.rule_bundles
    }
    /// Returns canonical selected modifiers.
    #[must_use]
    pub fn modifiers(self) -> &'a [ModifierDefinitionId] {
        &self.state.modifiers
    }
    /// Returns owner modifiers inherited by linked units created by this unit.
    #[must_use]
    pub fn linked_subject_modifiers(self) -> &'a [ModifierDefinitionId] {
        &self.state.linked_subject_modifiers
    }
    /// Returns active elemental weaknesses in canonical element order.
    #[must_use]
    pub fn weaknesses(self) -> &'a [CombatElement] {
        &self.state.weaknesses
    }
    /// Returns the immutable authored weakness baseline.
    #[must_use]
    pub fn permanent_weaknesses(self) -> &'a [CombatElement] {
        &self.state.permanent_weaknesses
    }
    /// Iterates temporary weakness contributions in canonical insertion order.
    pub fn temporary_weaknesses(self) -> impl Iterator<Item = TemporaryWeaknessView> + 'a {
        self.state
            .temporary_weaknesses
            .iter()
            .map(|state| TemporaryWeaknessView {
                element: state.element,
                applier: state.applier,
                source_operation: state.source_operation,
                remaining_turns: state.remaining_turns,
            })
    }
    /// Returns whether a layer has placed this unit in the global broken state.
    #[must_use]
    pub const fn weakness_broken(self) -> bool {
        self.state.weakness_broken
    }
    /// Iterates Toughness layers in authored routing order.
    pub fn toughness_layers(self) -> impl Iterator<Item = ToughnessLayerView<'a>> + 'a {
        self.state
            .toughness_layers
            .iter()
            .map(|state| ToughnessLayerView { state })
    }
    /// Returns the generic resolved combatant digest.
    #[must_use]
    pub const fn digest(self) -> CombatantSpecDigest {
        self.state.digest
    }
}

/// Immutable ordered Toughness-layer projection.
#[derive(Clone, Copy)]
pub struct ToughnessLayerView<'a> {
    state: &'a ToughnessLayerState,
}

impl<'a> ToughnessLayerView<'a> {
    /// Returns the complete immutable authored layer policy.
    #[must_use]
    pub const fn spec(self) -> &'a ToughnessLayerSpec {
        &self.state.spec
    }
    #[must_use]
    pub const fn key(self) -> u32 {
        self.state.spec.key()
    }
    /// Returns the authored semantic layer key when one is available.
    #[must_use]
    pub fn stable_key(self) -> Option<&'a str> {
        self.state.spec.stable_key()
    }
    #[must_use]
    pub const fn kind(self) -> ToughnessLayerKind {
        self.state.spec.kind()
    }
    #[must_use]
    pub const fn current(self) -> RawToughness {
        self.state.current
    }
    #[must_use]
    pub const fn maximum(self) -> RawToughness {
        self.state.spec.maximum()
    }
    #[must_use]
    pub const fn active(self) -> bool {
        self.state.spec.active()
    }
    #[must_use]
    pub const fn locked(self) -> bool {
        self.state.spec.locked()
    }
}

/// Canonical occupied formation entry.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FormationView {
    side: TeamSide,
    index: FormationIndex,
    unit: UnitId,
}

impl FormationView {
    /// Returns the formation side.
    #[must_use]
    pub const fn side(self) -> TeamSide {
        self.side
    }
    /// Returns the authored slot.
    #[must_use]
    pub const fn index(self) -> FormationIndex {
        self.index
    }
    /// Returns the stable unit occupying this slot.
    #[must_use]
    pub const fn unit(self) -> UnitId {
        self.unit
    }
}

impl From<FormationEntry> for FormationView {
    fn from(value: FormationEntry) -> Self {
        Self {
            side: value.side,
            index: value.index,
            unit: value.unit,
        }
    }
}

/// Immutable timeline-actor projection.
#[derive(Clone, Copy)]
pub struct TimelineActorView<'a> {
    state: &'a TimelineActorState,
}

impl TimelineActorView<'_> {
    /// Returns the stable timeline identity.
    #[must_use]
    pub const fn id(self) -> TimelineActorId {
        self.state.id
    }
    /// Returns the linked unit owner.
    #[must_use]
    pub const fn owner(self) -> UnitId {
        self.state.owner
    }
    /// Returns the target-capable action unit, or `None` for a timeline-only actor.
    #[must_use]
    pub const fn unit(self) -> Option<UnitId> {
        self.state.unit
    }
    /// Returns the linked semantic role; ordinary unit actors have no role tag.
    #[must_use]
    pub const fn linked_kind(self) -> Option<LinkedEntityKind> {
        self.state.kind
    }
    /// Returns the automatically executed ability, if any.
    #[must_use]
    pub const fn automatic_ability(self) -> Option<AbilityId> {
        self.state.automatic_ability
    }
    /// Returns whether the actor participates in timeline selection.
    #[must_use]
    pub const fn is_active(self) -> bool {
        self.state.active
    }
    /// Returns current canonical Action Gauge.
    #[must_use]
    pub const fn action_gauge(self) -> ActionGauge {
        self.state.gauge
    }
    /// Returns entry Speed.
    #[must_use]
    pub const fn speed(self) -> Speed {
        self.state.speed
    }
}

/// Immutable explicit owner/entity link projection.
#[derive(Clone, Copy)]
pub struct LinkView<'a> {
    state: &'a LinkState,
}

impl LinkView<'_> {
    #[must_use]
    pub const fn owner(self) -> UnitId {
        self.state.owner
    }
    #[must_use]
    pub const fn entity(self) -> LinkedEntity {
        self.state.entity
    }
    #[must_use]
    pub const fn kind(self) -> LinkedEntityKind {
        self.state.kind
    }
    #[must_use]
    pub const fn owner_defeat_policy(self) -> OwnerLinkPolicy {
        self.state.owner_defeat
    }
    #[must_use]
    pub const fn owner_departure_policy(self) -> OwnerLinkPolicy {
        self.state.owner_departure
    }
    #[must_use]
    pub const fn wave_policy(self) -> WaveLinkPolicy {
        self.state.wave
    }
    #[must_use]
    pub const fn is_active(self) -> bool {
        self.state.active
    }
}

/// Immutable team-resource projection.
#[derive(Clone, Copy)]
pub struct TeamView<'a> {
    state: &'a TeamState,
}

impl<'a> TeamView<'a> {
    /// Returns the team side.
    #[must_use]
    pub const fn side(self) -> TeamSide {
        self.state.side
    }
    /// Returns current Skill Points.
    #[must_use]
    pub const fn skill_points(self) -> u16 {
        self.state.skill_points
    }
    /// Returns the team Skill Point cap.
    #[must_use]
    pub const fn maximum_skill_points(self) -> u16 {
        self.state.maximum_skill_points
    }
    /// Returns a generic team resource and its cap by stable semantic identity.
    #[must_use]
    pub fn keyed_resource(self, id: SourceDefinitionId) -> Option<(u16, u16)> {
        self.state
            .keyed(id)
            .map(|resource| (resource.current, resource.maximum))
    }
}

/// Immutable encounter progress projection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EncounterView {
    definition: EncounterId,
    wave: WaveInstanceId,
    number: u16,
    total_waves: u16,
    spawn_defeats: u16,
}

impl EncounterView {
    /// Returns the selected encounter definition.
    #[must_use]
    pub const fn definition(self) -> EncounterId {
        self.definition
    }
    /// Returns the stable current wave instance.
    #[must_use]
    pub const fn wave(self) -> WaveInstanceId {
        self.wave
    }
    /// Returns the one-based current wave number.
    #[must_use]
    pub const fn number(self) -> u16 {
        self.number
    }
    /// Returns the immutable total number of encounter waves.
    #[must_use]
    pub const fn total_waves(self) -> u16 {
        self.total_waves
    }
    /// Returns refillable occurrences defeated in the current spawn-program wave.
    #[must_use]
    pub const fn spawn_defeats(self) -> u16 {
        self.spawn_defeats
    }
}
