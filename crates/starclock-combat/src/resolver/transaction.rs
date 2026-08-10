mod scratch;

use super::{
    command_resolution,
    journal::{self, AllocationKind, MutationField, MutationJournal, phase_code},
    modifier_snapshot, rule, selector_snapshot,
};
use crate::{
    AbilityId, BattleDiagnostics, DiagnosticRecord, EffectInstanceId as CrateEffectInstanceId,
    Energy, Hp, LifeState, LinkedEntity, ModifierInstanceId, PresenceState, Probability,
    RuleInstanceId, Scalar, SourceDefinitionId as CrateSourceDefinitionId, SpawnSequence, Speed,
    TeamSide, UnitDefinitionId, UnitId,
    action::lower::ActionIdentityAllocator,
    actor::store::{
        EnemyRuntimeState, FormationEntry, LinkState, TimelineActorState, TransformationState,
        UnitState,
    },
    battle::{
        fault::{BattleFault, FaultBoundary, FaultKind, FaultPolicy},
        model::BattlePhase,
        state::BattleState,
    },
    catalog::CombatCatalog,
    catalog::action::ReactionBoundary,
    codec::{BattleStateHash, hash_state},
    command::{model::DecisionPoint, validate::ValidatedCommand},
    event::{
        cause::Cause,
        model::{BattleEvent, BattleEventKind, FaultEventData},
    },
    id::{
        ActionBoundaryId, ActionFrameId, ActionId, CommandId, DecisionId, EffectInstanceId,
        EventId, HitId, OperationId, PhaseId, PreparedActionId, ShieldInstanceId, TimelineActorId,
        WaveInstanceId,
    },
    modifier::model::ActiveModifier,
    numeric::domain::ActionGauge,
    reaction::queue::QueuedAction,
    rng::types::DrawPurpose,
    rule::model::{OnceScope, SlotResetPoint},
    timeline::state::{
        ActionBoundaryState, ActionFrameState, NormalTurnState, PreparedActionState,
    },
};
pub(crate) use scratch::ResolutionScratch;
use std::{collections::BTreeMap, sync::Arc};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FaultInjectionPoint {
    AfterResolvingPhase,
    AfterCommandMutation,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct FaultInjection {
    pub(crate) point: FaultInjectionPoint,
    pub(crate) policy: FaultPolicy,
}
pub(crate) struct TransactionOutput {
    pub(crate) events: Vec<BattleEvent>,
    pub(crate) state_hash: BattleStateHash,
    pub(crate) root_command: CommandId,
    pub(crate) fault: Option<BattleFault>,
    pub(crate) timeline_elapsed_scaled: i64,
}

pub(crate) fn resolve_prepared(
    catalog: &CombatCatalog,
    before: &BattleState,
    scratch: &mut ResolutionScratch,
    command: ValidatedCommand,
    injection: Option<FaultInjection>,
    diagnostics: Option<&mut BattleDiagnostics>,
) -> TransactionOutput {
    let (mut events, root_command, failure, mut timeline_elapsed_scaled) = {
        let mut txn = Transaction::new(
            &mut scratch.working,
            &mut scratch.journal,
            catalog.needs_selector_snapshots(),
            diagnostics,
        );
        let root = txn.begin_command();
        let failure = command_resolution::execute(catalog, &mut txn, root, command, injection)
            .and_then(|()| {
                let parent = txn.events.last().map(BattleEvent::id).ok_or_else(|| {
                    BattleFault::new(
                        FaultKind::InvariantViolation,
                        FaultBoundary::Command,
                        FaultPolicy::Rollback,
                        0x33ff,
                        None,
                    )
                })?;
                rule::dispatch_pending_after_events(catalog, &mut txn, parent).map(drop)
            })
            .err();
        (txn.events, root, failure, txn.timeline_elapsed_scaled)
    };

    let mut committed_fault = failure;
    if committed_fault.is_none() && !scratch.journal.verify(&event_ids(&events)) {
        committed_fault = Some(BattleFault::new(
            FaultKind::InvariantViolation,
            FaultBoundary::Commit,
            FaultPolicy::Rollback,
            1,
            None,
        ));
    }

    if let Some(fault) = committed_fault {
        if fault.policy() == FaultPolicy::Rollback {
            scratch.prepare(before);
            timeline_elapsed_scaled = 0;
            let mut txn = Transaction::new(&mut scratch.working, &mut scratch.journal, false, None);
            let rollback_root = txn.begin_command();
            debug_assert_eq!(rollback_root, root_command);
            events = txn.commit_fault(rollback_root, fault);
        } else {
            let txn = Transaction::with_events(&mut scratch.working, &mut scratch.journal, events);
            events = txn.commit_fault(root_command, fault);
        }
    }

    let event_ids = event_ids(&events);
    if !scratch.journal.verify(&event_ids) {
        // A fault settlement that cannot journal itself is a programmer defect,
        // not a recoverable battle condition.
        panic!("fault settlement produced an inconsistent forward journal");
    }
    let state_hash = hash_state(&scratch.working);
    TransactionOutput {
        events,
        state_hash,
        root_command,
        fault: committed_fault,
        timeline_elapsed_scaled,
    }
}

pub(super) fn action_fault(context: u32) -> BattleFault {
    BattleFault::new(
        FaultKind::InvariantViolation,
        FaultBoundary::Command,
        FaultPolicy::Rollback,
        0x3100 + context,
        None,
    )
}

pub(super) struct Transaction<'a> {
    pub(super) state: &'a mut BattleState,
    pub(super) journal: &'a mut MutationJournal,
    pub(super) events: Vec<BattleEvent>,
    next_rule_event: usize,
    pub(super) selector_event_snapshots:
        BTreeMap<EventId, Arc<selector_snapshot::RuleSelectorSnapshot>>,
    pub(super) selector_action_snapshots:
        BTreeMap<ActionId, Arc<selector_snapshot::RuleSelectorSnapshot>>,
    pub(super) capture_selector_snapshots: bool,
    resolved_reactions: usize,
    pub(super) timeline_elapsed_scaled: i64,
    diagnostics: Option<&'a mut BattleDiagnostics>,
}

impl<'a> Transaction<'a> {
    fn new(
        state: &'a mut BattleState,
        journal: &'a mut MutationJournal,
        capture_selector_snapshots: bool,
        diagnostics: Option<&'a mut BattleDiagnostics>,
    ) -> Self {
        Self {
            state,
            journal,
            events: Vec::new(),
            next_rule_event: 0,
            selector_event_snapshots: BTreeMap::new(),
            selector_action_snapshots: BTreeMap::new(),
            capture_selector_snapshots,
            resolved_reactions: 0,
            timeline_elapsed_scaled: 0,
            diagnostics,
        }
    }

    fn with_events(
        state: &'a mut BattleState,
        journal: &'a mut MutationJournal,
        events: Vec<BattleEvent>,
    ) -> Self {
        Self {
            state,
            journal,
            events,
            next_rule_event: 0,
            selector_event_snapshots: BTreeMap::new(),
            selector_action_snapshots: BTreeMap::new(),
            capture_selector_snapshots: false,
            resolved_reactions: 0,
            timeline_elapsed_scaled: 0,
            diagnostics: None,
        }
    }

    pub(super) fn record_diagnostic(&mut self, record: impl FnOnce() -> DiagnosticRecord) {
        if let Some(diagnostics) = self.diagnostics.as_deref_mut() {
            diagnostics.record(record);
        }
    }

    pub(super) fn next_pending_rule_event(&mut self) -> Option<BattleEvent> {
        let event = self.events.get(self.next_rule_event)?.clone();
        self.next_rule_event += 1;
        Some(event)
    }

    fn begin_command(&mut self) -> CommandId {
        let command = self
            .state
            .sequences
            .try_command()
            .expect("rules-revision command budget prevents u64 identity exhaustion");
        self.journal
            .allocation(AllocationKind::Command, command.get());
        command
    }

    pub(super) fn allocate_decision(&mut self) -> DecisionId {
        let decision = self
            .state
            .sequences
            .try_decision()
            .expect("rules-revision decision budget prevents u64 identity exhaustion");
        self.journal
            .allocation(AllocationKind::Decision, decision.get());
        decision
    }

    pub(super) fn allocate_action_boundary(&mut self) -> ActionBoundaryId {
        let boundary = self
            .state
            .sequences
            .try_action_boundary()
            .expect("rules-revision action-boundary budget prevents u64 identity exhaustion");
        self.journal
            .allocation(AllocationKind::ActionBoundary, boundary.get());
        boundary
    }

    pub(super) fn allocate_prepared_action(&mut self) -> PreparedActionId {
        let prepared = self
            .state
            .sequences
            .try_prepared_action()
            .expect("rules-revision prepared-action budget prevents u64 identity exhaustion");
        self.journal
            .allocation(AllocationKind::PreparedAction, prepared.get());
        prepared
    }

    pub(super) fn allocate_action_frame(&mut self, action: ActionId) -> ActionFrameId {
        let frame = ActionFrameId::new(action.get())
            .expect("an allocated nonzero action identity is a valid frame identity");
        self.journal
            .allocation(AllocationKind::ActionFrame, frame.get());
        frame
    }

    pub(super) fn allocate_event(&mut self) -> EventId {
        let event = self
            .state
            .sequences
            .try_event()
            .expect("rules-revision event budget prevents u64 identity exhaustion");
        self.journal.allocation(AllocationKind::Event, event.get());
        event
    }

    fn allocate_action(&mut self) -> ActionId {
        let id = self
            .state
            .sequences
            .try_action()
            .expect("rules-revision action budget prevents u64 identity exhaustion");
        self.journal.allocation(AllocationKind::Action, id.get());
        id
    }

    fn allocate_phase(&mut self) -> PhaseId {
        let id = self
            .state
            .sequences
            .try_phase()
            .expect("rules-revision phase budget prevents u64 identity exhaustion");
        self.journal.allocation(AllocationKind::Phase, id.get());
        id
    }

    fn allocate_hit(&mut self) -> HitId {
        let id = self
            .state
            .sequences
            .try_hit()
            .expect("rules-revision hit budget prevents u64 identity exhaustion");
        self.journal.allocation(AllocationKind::Hit, id.get());
        id
    }

    pub(super) fn allocate_operation(&mut self) -> OperationId {
        let id = self
            .state
            .sequences
            .try_operation()
            .expect("rules-revision operation budget prevents u64 identity exhaustion");
        self.journal.allocation(AllocationKind::Operation, id.get());
        id
    }

    pub(super) fn allocate_reaction(&mut self) -> u64 {
        let insertion = self
            .state
            .sequences
            .try_reaction()
            .expect("rules-revision reaction budget prevents sequence exhaustion");
        self.journal.allocation(AllocationKind::Reaction, insertion);
        self.journal
            .queue_insertion(journal::QueueKind::Reaction, insertion);
        insertion
    }

    pub(super) fn push_reaction(&mut self, action: QueuedAction) {
        self.state.reactions.push(action);
    }

    pub(super) fn pop_ready_reaction(
        &mut self,
        boundary: ReactionBoundary,
    ) -> Option<QueuedAction> {
        self.state.reactions.pop_ready(boundary)
    }

    pub(super) fn clear_reactions(&mut self) {
        self.state.reactions.clear();
    }

    pub(super) fn allocate_unit(&mut self) -> UnitId {
        let id = self.state.sequences.unit();
        self.journal.allocation(AllocationKind::Unit, id.get());
        id
    }

    pub(super) fn allocate_actor(&mut self) -> TimelineActorId {
        let id = self.state.sequences.actor();
        self.journal.allocation(AllocationKind::Actor, id.get());
        id
    }

    pub(super) fn allocate_spawn(&mut self) -> SpawnSequence {
        let id = self.state.sequences.spawn();
        self.journal.allocation(AllocationKind::Spawn, id.get());
        id
    }

    pub(super) fn allocate_rule(&mut self) -> RuleInstanceId {
        let id = self.state.sequences.rule();
        self.journal.allocation(AllocationKind::Rule, id.get());
        id
    }

    pub(super) fn allocate_modifier(&mut self) -> ModifierInstanceId {
        let id = self.state.sequences.modifier();
        self.journal.allocation(AllocationKind::Modifier, id.get());
        id
    }
    pub(super) fn consume_reaction_budget(&mut self, maximum: usize) -> bool {
        let Some(next) = self.resolved_reactions.checked_add(1) else {
            return false;
        };
        if next > maximum {
            return false;
        }
        self.resolved_reactions = next;
        true
    }

    pub(super) fn allocate_shield(&mut self) -> ShieldInstanceId {
        let id = self
            .state
            .sequences
            .try_shield()
            .expect("rules-revision shield budget prevents u64 identity exhaustion");
        self.journal.allocation(AllocationKind::Shield, id.get());
        id
    }

    pub(super) fn allocate_effect(&mut self) -> EffectInstanceId {
        let id = self
            .state
            .sequences
            .try_effect()
            .expect("rules-revision effect budget prevents u64 identity exhaustion");
        self.journal.allocation(AllocationKind::Effect, id.get());
        id
    }

    pub(super) fn allocate_wave(&mut self) -> WaveInstanceId {
        let id = self
            .state
            .sequences
            .try_wave()
            .expect("rules-revision wave budget prevents u64 identity exhaustion");
        self.journal.allocation(AllocationKind::Wave, id.get());
        id
    }

    pub(super) fn set_phase(&mut self, phase: BattlePhase) {
        let before = self.state.phase;
        if before != phase {
            self.state.phase = phase;
            self.journal
                .mutation(MutationField::Phase, phase_code(before), phase_code(phase));
        }
    }

    pub(super) fn set_decision(&mut self, decision: Option<DecisionPoint>) {
        let before = self
            .state
            .decision
            .as_ref()
            .map_or(0, |value| value.id().get());
        let after = decision.as_ref().map_or(0, |value| value.id().get());
        if self.state.decision != decision {
            self.state.decision = decision;
            self.journal
                .mutation(MutationField::Decision, before, after);
        }
    }

    fn set_fault(&mut self, fault: BattleFault) {
        let before = self.state.fault.map_or(0, fault_code);
        let after = fault_code(fault);
        if self.state.fault != Some(fault) {
            self.state.fault = Some(fault);
            self.journal.mutation(MutationField::Fault, before, after);
        }
    }

    pub(super) fn set_active_turn(&mut self, turn: Option<NormalTurnState>) {
        let before = self
            .state
            .timeline
            .active_turn
            .map_or(0, |value| value.actor.get());
        let after = turn.map_or(0, |value| value.actor.get());
        if self.state.timeline.active_turn != turn {
            self.state.timeline.active_turn = turn;
            self.journal
                .mutation(MutationField::Timeline, before, after);
        }
    }

    pub(super) fn set_action_boundary(&mut self, boundary: Option<ActionBoundaryState>) {
        let before = self
            .state
            .timeline
            .boundary
            .as_ref()
            .map_or(0, |value| value.id.get());
        let after = boundary.as_ref().map_or(0, |value| value.id.get());
        if self.state.timeline.boundary != boundary {
            self.state.timeline.boundary = boundary;
            self.journal
                .mutation(MutationField::Timeline, before, after);
        }
    }

    pub(super) fn set_prepared_action(&mut self, prepared: Option<PreparedActionState>) {
        let before = self
            .state
            .timeline
            .prepared_action
            .as_ref()
            .map_or(0, |value| value.id.get());
        let after = prepared.as_ref().map_or(0, |value| value.id.get());
        if self.state.timeline.prepared_action != prepared {
            self.state.timeline.prepared_action = prepared;
            self.journal
                .mutation(MutationField::Timeline, before, after);
        }
    }

    pub(super) fn set_action_frame(&mut self, frame: Option<ActionFrameState>) {
        let before = self
            .state
            .timeline
            .action_frame
            .as_ref()
            .map_or(0, |value| value.id.get());
        let after = frame.as_ref().map_or(0, |value| value.id.get());
        if self.state.timeline.action_frame != frame {
            self.state.timeline.action_frame = frame;
            self.journal
                .mutation(MutationField::Timeline, before, after);
        }
    }

    pub(super) fn set_actor_gauge(
        &mut self,
        actor: TimelineActorId,
        gauge: ActionGauge,
    ) -> Result<(), BattleFault> {
        let state = self
            .state
            .actors
            .get_mut(actor)
            .ok_or_else(|| action_fault(8))?;
        let before = state.gauge.scaled();
        let after = gauge.scaled();
        if before != after {
            state.gauge = gauge;
            self.journal.mutation(
                MutationField::ActionGauge,
                before.cast_unsigned(),
                after.cast_unsigned(),
            );
        }
        Ok(())
    }

    pub(super) fn set_actor_active(
        &mut self,
        actor: TimelineActorId,
        active: bool,
    ) -> Result<(), BattleFault> {
        let state = self
            .state
            .actors
            .get_mut(actor)
            .ok_or_else(|| action_fault(74))?;
        if state.active != active {
            let before = u64::from(state.active);
            state.active = active;
            self.journal
                .mutation(MutationField::ActorActive, before, u64::from(active));
        }
        Ok(())
    }

    pub(super) fn insert_unit(&mut self, state: UnitState) {
        let id = state.id;
        self.state.units.insert(state);
        self.journal.mutation(MutationField::UnitStore, 0, id.get());
    }

    pub(super) fn insert_actor(&mut self, state: TimelineActorState) {
        let id = state.id;
        self.state.actors.insert(state);
        self.journal
            .mutation(MutationField::ActorStore, 0, id.get());
    }

    pub(super) fn insert_modifier(
        &mut self,
        catalog: &CombatCatalog,
        mut state: ActiveModifier,
    ) -> Result<(), BattleFault> {
        modifier_snapshot::initialize(catalog, self, &mut state)?;
        let id = state.instance;
        if !self.state.modifiers.insert(state) {
            return Err(action_fault(76));
        }
        self.journal
            .mutation(MutationField::ModifierStore, 0, id.get());
        Ok(())
    }

    pub(super) fn remove_effect_attachments(&mut self, effect: CrateEffectInstanceId) {
        for modifier in self.state.modifiers.remove_by_effect(effect) {
            self.journal
                .mutation(MutationField::EffectAttachment, modifier.get(), 0);
        }
        for rule in self.state.rules.remove_by_effect(effect) {
            self.journal
                .mutation(MutationField::EffectAttachment, rule.get(), 0);
        }
    }
    pub(super) fn insert_formation(&mut self, entry: FormationEntry) {
        self.state.formations.push(entry);
        self.journal
            .mutation(MutationField::Formation, 0, entry.unit.get());
    }

    pub(super) fn insert_link(&mut self, state: LinkState) -> Result<(), BattleFault> {
        let code = match state.entity {
            LinkedEntity::Unit(unit) => unit.get(),
            LinkedEntity::TimelineActor(actor) => actor.get() | (1_u64 << 63),
        };
        if !self.state.links.insert(state) {
            return Err(action_fault(75));
        }
        self.journal.mutation(MutationField::LinkStore, 0, code);
        Ok(())
    }

    pub(super) fn set_link_active(
        &mut self,
        entity: LinkedEntity,
        active: bool,
    ) -> Result<(), BattleFault> {
        let link = self
            .state
            .links
            .get_mut(entity)
            .ok_or_else(|| action_fault(76))?;
        if link.active != active {
            let before = u64::from(link.active);
            link.active = active;
            self.journal
                .mutation(MutationField::LinkStore, before, u64::from(active));
        }
        Ok(())
    }

    pub(super) fn set_unit_definition(
        &mut self,
        unit: UnitId,
        form: UnitDefinitionId,
        abilities: Box<[AbilityId]>,
        presence: PresenceState,
        transformation: Option<TransformationState>,
    ) -> Result<(), BattleFault> {
        let state = self
            .state
            .units
            .get_mut(unit)
            .ok_or_else(|| action_fault(77))?;
        if state.form != form {
            let before = state.form.get();
            state.form = form;
            self.journal.mutation(
                MutationField::UnitDefinition,
                u64::from(before),
                u64::from(form.get()),
            );
        }
        if state.abilities != abilities {
            state.abilities = abilities;
            self.journal.mutation(MutationField::UnitAbilities, 1, 2);
        }
        let before_presence = state.presence;
        if before_presence != presence {
            state.presence = presence;
            self.journal.mutation(
                MutationField::UnitPresence,
                before_presence as u64,
                presence as u64,
            );
        }
        if state.transformation != transformation {
            let (before_transform, after_transform) =
                match (state.transformation.as_ref(), transformation.as_ref()) {
                    (None, Some(_)) => (0, 1),
                    (Some(_), None) => (1, 0),
                    (Some(_), Some(_)) => (1, 2),
                    (None, None) => unreachable!("unequal transformations cannot both be absent"),
                };
            state.transformation = transformation;
            self.journal.mutation(
                MutationField::Transformation,
                before_transform,
                after_transform,
            );
        }
        Ok(())
    }

    pub(super) fn set_enemy_runtime(
        &mut self,
        unit: UnitId,
        enemy: EnemyRuntimeState,
    ) -> Result<(), BattleFault> {
        let state = self
            .state
            .units
            .get_mut(unit)
            .ok_or_else(|| action_fault(97))?;
        let before = state.enemy.ok_or_else(|| action_fault(98))?;
        if before != enemy {
            state.enemy = Some(enemy);
            let before_code = before.phase.map_or(0, |phase| u64::from(phase.get()));
            let after_code = enemy.phase.map_or(0, |phase| u64::from(phase.get()));
            self.journal
                .mutation(MutationField::EnemyOrchestration, before_code, after_code);
        }
        Ok(())
    }

    pub(super) fn unit_speed(&self, owner: UnitId) -> Result<Speed, BattleFault> {
        let actor = self
            .state
            .actors
            .id_for_owner(owner)
            .ok_or_else(|| action_fault(52))?;
        self.state
            .actors
            .get(actor)
            .map(|state| state.speed)
            .ok_or_else(|| action_fault(53))
    }

    pub(super) fn set_unit_speed(
        &mut self,
        owner: UnitId,
        speed: Speed,
    ) -> Result<(), BattleFault> {
        let actor = self
            .state
            .actors
            .id_for_owner(owner)
            .ok_or_else(|| action_fault(54))?;
        let state = self
            .state
            .actors
            .get_mut(actor)
            .ok_or_else(|| action_fault(55))?;
        let before = state.speed;
        if before != speed {
            state.speed = speed;
            self.journal.mutation(
                MutationField::Speed,
                before.scaled() as u64,
                speed.scaled() as u64,
            );
        }
        Ok(())
    }

    pub(super) fn roll_probability(
        &mut self,
        probability: Probability,
        purpose: DrawPurpose,
    ) -> Result<bool, BattleFault> {
        let threshold = probability.millionths();
        if threshold == 0 {
            return Ok(false);
        }
        if threshold == 1_000_000 {
            return Ok(true);
        }
        Ok(self.probability_draw(purpose)? < threshold)
    }

    pub(super) fn roll_shared_probability(
        &mut self,
        probability: Probability,
        purpose: DrawPurpose,
        shared_draw: &mut Option<u32>,
    ) -> Result<bool, BattleFault> {
        let threshold = probability.millionths();
        if threshold == 0 {
            return Ok(false);
        }
        if threshold == 1_000_000 {
            return Ok(true);
        }
        let draw = match *shared_draw {
            Some(value) => value,
            None => {
                let value = self.probability_draw(purpose)?;
                *shared_draw = Some(value);
                value
            }
        };
        Ok(draw < threshold)
    }

    fn probability_draw(&mut self, purpose: DrawPurpose) -> Result<u32, BattleFault> {
        let before = self.state.rng.draw_count();
        let draw = self
            .state
            .rng
            .sample_below(purpose, 1_000_000)
            .map_err(|_| action_fault(51))?;
        for index in before..self.state.rng.draw_count() {
            self.journal.rng_draw(index, purpose.code());
        }
        u32::try_from(draw.value()).map_err(|_| action_fault(51))
    }

    pub(super) fn set_skill_points(&mut self, side: TeamSide, value: u16) {
        let state = self.state.teams.get_mut(side);
        let before = state.skill_points;
        if before != value {
            state.skill_points = value;
            self.journal.mutation(
                MutationField::TeamSkillPoints,
                u64::from(before),
                u64::from(value),
            );
        }
    }

    pub(super) fn set_team_resource(
        &mut self,
        side: TeamSide,
        resource: CrateSourceDefinitionId,
        value: u16,
    ) -> Result<(), BattleFault> {
        let state = self
            .state
            .teams
            .get_mut(side)
            .keyed_mut(resource)
            .ok_or_else(|| action_fault(53))?;
        let before = state.current;
        if value > state.maximum {
            return Err(action_fault(54));
        }
        if before != value {
            state.current = value;
            self.journal.mutation(
                MutationField::TeamKeyedResource,
                u64::from(before),
                u64::from(value),
            );
        }
        Ok(())
    }

    pub(super) fn set_character_resource(
        &mut self,
        unit: UnitId,
        stable_key: &str,
        value: Scalar,
    ) -> Result<(), BattleFault> {
        let state = self
            .state
            .units
            .get_mut(unit)
            .and_then(|unit| unit.resource_mut(stable_key))
            .ok_or_else(|| action_fault(55))?;
        let before = state.current;
        if value.scaled() < 0 || value > state.maximum {
            return Err(action_fault(56));
        }
        if before != value {
            state.current = value;
            self.journal.mutation(
                MutationField::UnitCharacterResource,
                before.scaled().cast_unsigned(),
                value.scaled().cast_unsigned(),
            );
        }
        Ok(())
    }

    pub(super) fn set_energy(&mut self, unit: UnitId, value: Energy) -> Result<(), BattleFault> {
        let state = self
            .state
            .units
            .get_mut(unit)
            .ok_or_else(|| action_fault(31))?;
        let before = state.current_energy;
        if before != value {
            state.current_energy = value;
            self.journal.mutation(
                MutationField::UnitEnergy,
                before.scaled().cast_unsigned(),
                value.scaled().cast_unsigned(),
            );
        }
        Ok(())
    }

    pub(super) fn set_hp(&mut self, unit: UnitId, value: Hp) -> Result<(), BattleFault> {
        let state = self
            .state
            .units
            .get_mut(unit)
            .ok_or_else(|| action_fault(33))?;
        let before = state.current_hp;
        if before != value {
            state.current_hp = value;
            self.journal.mutation(
                MutationField::UnitHp,
                before.get().cast_unsigned(),
                value.get().cast_unsigned(),
            );
        }
        Ok(())
    }

    pub(super) fn set_life(&mut self, unit: UnitId, value: LifeState) -> Result<(), BattleFault> {
        let state = self
            .state
            .units
            .get_mut(unit)
            .ok_or_else(|| action_fault(34))?;
        let before = state.life;
        if before != value {
            state.life = value;
            self.journal
                .mutation(MutationField::UnitLife, before as u64, value as u64);
        }
        Ok(())
    }

    pub(super) fn set_presence(
        &mut self,
        unit: UnitId,
        value: PresenceState,
    ) -> Result<(), BattleFault> {
        let state = self
            .state
            .units
            .get_mut(unit)
            .ok_or_else(|| action_fault(35))?;
        let before = state.presence;
        if before != value {
            state.presence = value;
            self.journal
                .mutation(MutationField::UnitPresence, before as u64, value as u64);
        }
        Ok(())
    }

    pub(super) fn set_encounter_wave(&mut self, wave: WaveInstanceId, number: u16) {
        let before = self.state.encounter.number;
        if self.state.encounter.wave != wave || before != number {
            self.state.encounter.wave = wave;
            self.state.encounter.number = number;
            self.journal.mutation(
                MutationField::Encounter,
                u64::from(before),
                u64::from(number),
            );
        }
    }

    pub(super) fn bump_revision(&mut self) -> Result<(), BattleFault> {
        let before = self.state.committed_revision;
        let after = before.checked_add(1).ok_or_else(|| {
            BattleFault::new(
                FaultKind::SequenceExhausted,
                FaultBoundary::Commit,
                FaultPolicy::Rollback,
                2,
                None,
            )
        })?;
        self.state.committed_revision = after;
        self.journal
            .mutation(MutationField::CommittedRevision, before, after);
        Ok(())
    }

    pub(super) fn reset_rule_slots(&mut self, boundary: SlotResetPoint, owner: Option<UnitId>) {
        let mut count = self.state.rules.reset(boundary, owner);
        if boundary == SlotResetPoint::TurnStart {
            count += self.state.rules.reset_once_scope(OnceScope::Turn);
        }
        if count > 0 {
            self.journal
                .mutation(MutationField::RuleState, 0, count as u64);
        }
    }

    fn commit_fault(mut self, root: CommandId, fault: BattleFault) -> Vec<BattleEvent> {
        self.set_decision(None);
        self.set_action_boundary(None);
        self.set_prepared_action(None);
        self.set_action_frame(None);
        self.set_active_turn(None);
        self.clear_reactions();
        self.set_fault(fault);
        self.set_phase(BattlePhase::Faulted);
        if let Err(revision_fault) = self.bump_revision() {
            debug_assert_eq!(fault.kind(), FaultKind::SequenceExhausted);
            debug_assert_eq!(revision_fault.kind(), FaultKind::SequenceExhausted);
        }
        let cause = self.events.last().map_or_else(
            || Cause::root(root),
            |event| Cause::root(root).with_parent(event.id()),
        );
        self.emit(cause, BattleEventKind::Fault(FaultEventData::new(fault)));
        self.events
    }
}

impl ActionIdentityAllocator for Transaction<'_> {
    fn action(&mut self) -> ActionId {
        self.allocate_action()
    }

    fn phase(&mut self) -> PhaseId {
        self.allocate_phase()
    }

    fn hit(&mut self) -> HitId {
        self.allocate_hit()
    }

    fn operation(&mut self) -> OperationId {
        self.allocate_operation()
    }
}

fn fault_code(fault: BattleFault) -> u64 {
    u64::from(fault.context_code()) << 24
        | (fault.kind() as u64) << 16
        | (fault.boundary() as u64) << 8
        | fault.policy() as u64
}

fn event_ids(events: &[BattleEvent]) -> Vec<EventId> {
    events.iter().map(BattleEvent::id).collect()
}
