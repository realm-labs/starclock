//! Incremental external-control facade over the canonical Gold run executor.

use std::collections::{BTreeSet, VecDeque};

use starclock_activity::{
    ActivityDecisionId, ActivityEdgeId, ActivityMasterSeed, ActivityOperation, ActivityPlayerView,
    ActivityProgramDefinition, ActivityProgramId, ActivityRngContext, ActivityRngStreams,
    ActivityStateHash, ActivityTerminalOutcome, ActivityTransactionState, AttemptId,
    BattleSequence, NodeId,
};

use crate::battle_materialization::UniverseBattleRoster;

use super::GoldAndGearsSeededRunStep;
use super::{
    GoldAndGearsBattleAssemblyContext, GoldAndGearsEncounterRole, GoldAndGearsExtrapolationContext,
    GoldAndGearsOfferedAction, GoldAndGearsOfferedCommand, GoldAndGearsRuntimeInstance,
    GoldAndGearsSeededRunAction, GoldAndGearsSeededRunError, GoldAndGearsSeededRunRequest,
    GoldAndGearsSeededRunStepKind,
    seeded_run::{
        GoldAndGearsRecordedExecution, GoldAndGearsSeededBattleRecord,
        GoldAndGearsSeededReplayStep, MAX_SEEDED_RUN_STEPS, apply_program, step,
        terminal_execution,
    },
};

const TRAVERSE_PROGRAM_BASE: u32 = 0x7f74_0000;
const DEFAULT_BOSS: &str = "gold-gears.boss-choice.1013014";
const FINAL_BOSS: &str = "gold-gears.boss-choice.8024011";

/// Accepted work performed automatically after an external action.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct GoldAndGearsIncrementalSettlement {
    accepted_actions: u32,
    nested_battles: u32,
}

impl GoldAndGearsIncrementalSettlement {
    #[must_use]
    pub const fn accepted_actions(self) -> u32 {
        self.accepted_actions
    }

    #[must_use]
    pub const fn nested_battles(self) -> u32 {
        self.nested_battles
    }
}

/// One resumable Gold run. All mutation still uses ordinary Activity programs
/// and the released nested-battle handoff owned by the mode runtime.
pub struct GoldAndGearsIncrementalRun {
    request: GoldAndGearsSeededRunRequest,
    state: ActivityTransactionState,
    rng: ActivityRngStreams,
    created_planes: [bool; 3],
    selected_boss_nodes: BTreeSet<NodeId>,
    steps: Vec<GoldAndGearsSeededRunStep>,
    replay: Vec<GoldAndGearsSeededReplayStep>,
    battle_count: u32,
}

impl GoldAndGearsIncrementalRun {
    #[must_use]
    pub fn start(
        instance: &GoldAndGearsRuntimeInstance,
        request: GoldAndGearsSeededRunRequest,
    ) -> Self {
        Self {
            request,
            state: ActivityTransactionState::new(
                instance.state_definition().clone(),
                instance.graph_definition().entry(),
            ),
            rng: ActivityRngStreams::new(ActivityRngContext::new(
                ActivityMasterSeed::from_u64(request.seed()),
                request.identity().id(),
                request.identity().definition_digest(),
                request.identity().config_digest(),
                instance.graph_definition().digest(),
                request.activity_instance(),
                None,
                Some(instance.graph_definition().entry()),
                None,
                0,
            )),
            created_planes: [false; 3],
            selected_boss_nodes: BTreeSet::new(),
            steps: Vec::new(),
            replay: Vec::new(),
            battle_count: 0,
        }
    }

    #[must_use]
    pub const fn request(&self) -> GoldAndGearsSeededRunRequest {
        self.request
    }

    #[must_use]
    pub fn player_view(&self, instance: &GoldAndGearsRuntimeInstance) -> ActivityPlayerView {
        self.state.player_view(
            self.request.identity(),
            instance.graph_definition(),
            self.request.activity_instance(),
            &self.rng,
        )
    }

    #[must_use]
    pub const fn terminal(&self) -> Option<ActivityTerminalOutcome> {
        self.state.terminal()
    }

    #[must_use]
    pub fn state_hash(&self, instance: &GoldAndGearsRuntimeInstance) -> ActivityStateHash {
        self.state.state_hash(
            self.request.identity(),
            instance.graph_definition(),
            self.request.activity_instance(),
            &self.rng,
        )
    }

    #[must_use]
    pub fn action_count(&self) -> usize {
        self.replay.len()
    }

    #[must_use]
    pub const fn battle_count(&self) -> u32 {
        self.battle_count
    }

    pub fn decision_id(&self) -> Result<ActivityDecisionId, GoldAndGearsSeededRunError> {
        ActivityDecisionId::new(
            self.state
                .command_sequence()
                .checked_add(1)
                .ok_or(GoldAndGearsSeededRunError::StepBudgetExceeded)?,
        )
        .ok_or(GoldAndGearsSeededRunError::StepBudgetExceeded)
    }

    /// Advances plane setup and real battles until a player decision or a
    /// terminal boundary is reached.
    pub fn settle_automatic(
        &mut self,
        instance: &GoldAndGearsRuntimeInstance,
        roster: &UniverseBattleRoster,
    ) -> Result<GoldAndGearsIncrementalSettlement, GoldAndGearsSeededRunError> {
        let initial_actions = self.replay.len();
        let initial_battles = self.battle_count;
        loop {
            if let Some(terminal) = self.state.terminal() {
                if terminal != ActivityTerminalOutcome::Completed {
                    return Err(GoldAndGearsSeededRunError::UnexpectedTerminal(terminal));
                }
                break;
            }
            self.ensure_budget()?;
            let node = self.state.current_node();
            if let Some(plane) = instance
                .plane_starts()
                .position(|candidate| candidate == node)
                && !self.created_planes[plane]
            {
                self.create_plane(instance, node, plane)?;
                continue;
            }
            if let Some(role) = instance.encounter_role_for_node(&self.state, node)
                && !self.state.current_battle_attempt_is_settled()
            {
                if is_boss(role) && !self.selected_boss_nodes.contains(&node) {
                    break;
                }
                self.execute_battle(instance, roster, node, role)?;
                continue;
            }
            break;
        }
        Ok(GoldAndGearsIncrementalSettlement {
            accepted_actions: u32::try_from(self.replay.len() - initial_actions)
                .map_err(|_| GoldAndGearsSeededRunError::StepBudgetExceeded)?,
            nested_battles: self
                .battle_count
                .checked_sub(initial_battles)
                .ok_or(GoldAndGearsSeededRunError::StepBudgetExceeded)?,
        })
    }

    /// Returns the canonical ordered offer at the current stable boundary.
    pub fn offered_commands(
        &self,
        instance: &GoldAndGearsRuntimeInstance,
    ) -> Result<Box<[GoldAndGearsOfferedCommand]>, GoldAndGearsSeededRunError> {
        if self.state.terminal().is_some() {
            return Ok(Vec::new().into_boxed_slice());
        }
        let node = self.state.current_node();
        if let Some(role) = instance.encounter_role_for_node(&self.state, node)
            && !self.state.current_battle_attempt_is_settled()
            && is_boss(role)
            && !self.selected_boss_nodes.contains(&node)
        {
            let plane = boss_plane(role).expect("the guarded role is a boss");
            let preferred = if role == GoldAndGearsEncounterRole::FinalBoss {
                FINAL_BOSS
            } else {
                DEFAULT_BOSS
            };
            return Ok(instance
                .boss_choices()
                .enumerate()
                .map(|(ordinal, boss)| {
                    GoldAndGearsOfferedCommand::boss(plane, ordinal, boss, boss == preferred)
                })
                .collect::<Vec<_>>()
                .into_boxed_slice());
        }
        let plane = instance
            .graph_definition()
            .node(node)
            .and_then(|definition| usize::try_from(definition.section().get()).ok())
            .and_then(|section| section.checked_sub(1))
            .ok_or(GoldAndGearsSeededRunError::NoRoute(node))?;
        let target = instance
            .plane_ends()
            .nth(plane)
            .ok_or(GoldAndGearsSeededRunError::NoRoute(node))?;
        let preferred = next_route(instance, &self.state, node, target)
            .ok_or(GoldAndGearsSeededRunError::NoRoute(node))?;
        Ok(instance
            .legal_routes(&self.state, node)
            .iter()
            .copied()
            .map(|edge| GoldAndGearsOfferedCommand::route(edge, edge == preferred))
            .collect::<Vec<_>>()
            .into_boxed_slice())
    }

    /// Applies one command only after regenerating and matching the exact
    /// current offer. A caller cannot construct an equivalent mutation.
    pub fn apply_offered_command(
        &mut self,
        instance: &GoldAndGearsRuntimeInstance,
        selected: &GoldAndGearsOfferedCommand,
    ) -> Result<(), GoldAndGearsSeededRunError> {
        self.ensure_budget()?;
        let selected = self
            .offered_commands(instance)?
            .iter()
            .find(|candidate| *candidate == selected)
            .cloned()
            .ok_or(GoldAndGearsSeededRunError::CommandNotOffered)?;
        let node = self.state.current_node();
        match selected.action() {
            GoldAndGearsOfferedAction::SelectBoss { plane, boss } => {
                let role = instance
                    .encounter_role_for_node(&self.state, node)
                    .ok_or(GoldAndGearsSeededRunError::CommandNotOffered)?;
                if boss_plane(role) != Some(*plane) {
                    return Err(GoldAndGearsSeededRunError::CommandNotOffered);
                }
                let program = instance
                    .compile_boss_selection(*plane, boss)
                    .map_err(GoldAndGearsSeededRunError::InvalidInput)?;
                apply_program(instance, &mut self.state, program)?;
                self.selected_boss_nodes.insert(node);
                self.record(
                    instance,
                    GoldAndGearsSeededRunStepKind::BossSelection,
                    node,
                    GoldAndGearsSeededRunAction::BossSelection {
                        source_node: node,
                        plane: *plane,
                        boss: boss.clone(),
                    },
                    None,
                );
            }
            GoldAndGearsOfferedAction::Traverse { edge } => {
                let id = TRAVERSE_PROGRAM_BASE
                    .checked_add(edge.get())
                    .and_then(ActivityProgramId::new)
                    .ok_or(GoldAndGearsSeededRunError::StepBudgetExceeded)?;
                let program =
                    ActivityProgramDefinition::new(id, vec![ActivityOperation::Traverse(*edge)])
                        .map_err(|_| GoldAndGearsSeededRunError::ProgramRejected)?;
                apply_program(instance, &mut self.state, program)?;
                self.record(
                    instance,
                    GoldAndGearsSeededRunStepKind::Traverse,
                    node,
                    GoldAndGearsSeededRunAction::Traverse {
                        source_node: node,
                        edge: *edge,
                    },
                    None,
                );
            }
            _ => return Err(GoldAndGearsSeededRunError::CommandNotOffered),
        }
        Ok(())
    }

    pub(super) fn recorded_execution(
        &self,
        instance: &GoldAndGearsRuntimeInstance,
    ) -> Result<GoldAndGearsRecordedExecution, GoldAndGearsSeededRunError> {
        terminal_execution(
            instance,
            self.request,
            &self.state,
            &self.rng,
            self.battle_count,
            &self.steps,
            &self.replay,
        )
    }

    fn create_plane(
        &mut self,
        instance: &GoldAndGearsRuntimeInstance,
        node: NodeId,
        plane: usize,
    ) -> Result<(), GoldAndGearsSeededRunError> {
        let program = instance
            .compile_plane_creation(plane, &mut self.rng)
            .map_err(GoldAndGearsSeededRunError::InvalidInput)?;
        apply_program(instance, &mut self.state, program)?;
        self.created_planes[plane] = true;
        self.record(
            instance,
            GoldAndGearsSeededRunStepKind::PlaneCreation,
            node,
            GoldAndGearsSeededRunAction::PlaneCreation {
                source_node: node,
                plane: u8::try_from(plane + 1).expect("the frozen run has exactly three planes"),
            },
            None,
        );
        Ok(())
    }

    fn execute_battle(
        &mut self,
        instance: &GoldAndGearsRuntimeInstance,
        roster: &UniverseBattleRoster,
        node: NodeId,
        role: GoldAndGearsEncounterRole,
    ) -> Result<(), GoldAndGearsSeededRunError> {
        let selection = instance
            .select_current_encounter(&self.state, &mut self.rng)
            .map_err(GoldAndGearsSeededRunError::InvalidInput)?;
        self.battle_count = self
            .battle_count
            .checked_add(1)
            .ok_or(GoldAndGearsSeededRunError::StepBudgetExceeded)?;
        let mut context = GoldAndGearsBattleAssemblyContext::new(Vec::new(), false);
        if role == GoldAndGearsEncounterRole::FinalBoss {
            let extrapolation = instance
                .compile_resonance_extrapolation(
                    GoldAndGearsExtrapolationContext::new(3, true, instance.path()),
                    &mut self.rng,
                )
                .map_err(GoldAndGearsSeededRunError::InvalidInput)?;
            context = context.with_extrapolation(extrapolation);
        }
        let expected = self.state_hash(instance);
        let start = instance
            .start_current_battle(
                &mut self.state,
                &self.rng,
                expected,
                self.request.identity(),
                self.request.activity_instance(),
                AttemptId::new(self.battle_count)
                    .ok_or(GoldAndGearsSeededRunError::StepBudgetExceeded)?,
                BattleSequence::new(self.battle_count)
                    .ok_or(GoldAndGearsSeededRunError::StepBudgetExceeded)?,
                &selection,
                roster,
                &context,
            )
            .map_err(GoldAndGearsSeededRunError::Battle)?;
        let execution = instance
            .execute_started_battle(
                &mut self.state,
                &self.rng,
                self.request.identity(),
                self.request.activity_instance(),
                &start,
            )
            .map_err(GoldAndGearsSeededRunError::Battle)?;
        if let Some(fault) = execution.report().terminal_fault() {
            return Err(GoldAndGearsSeededRunError::BattleFault {
                role,
                group: selection.group().into(),
                fault,
            });
        }
        if let Some(fault) = execution.post_battle_events().iter().find_map(|event| {
            if let starclock_activity::ActivityTransactionEventKind::Faulted(fault) = event.kind() {
                Some(*fault)
            } else {
                None
            }
        }) {
            return Err(GoldAndGearsSeededRunError::PostBattleFault { role, fault, node });
        }
        if let Some(terminal) = self.state.terminal()
            && terminal != ActivityTerminalOutcome::Completed
        {
            return Err(GoldAndGearsSeededRunError::UnexpectedBattleTerminal {
                role,
                terminal,
                node,
            });
        }
        let result_digest = execution.result().actual_digest();
        let action = GoldAndGearsSeededRunAction::Battle {
            source_node: node,
            role,
            group: selection.group().into(),
            member: selection.source_rogue_monster_id().into(),
            effective_level: selection.effective_level(),
        };
        let accepted = step(
            GoldAndGearsSeededRunStepKind::Battle(role),
            node,
            &self.state,
            &self.rng,
            self.request,
            Some(result_digest),
            instance,
        );
        self.steps.push(accepted);
        self.replay.push(GoldAndGearsSeededReplayStep {
            action,
            state_hash: accepted.state_hash(),
            battle: Some(GoldAndGearsSeededBattleRecord {
                start_identity: start.handoff().identity(),
                result: execution.result().clone(),
                report: execution.report().clone(),
            }),
        });
        Ok(())
    }

    fn record(
        &mut self,
        instance: &GoldAndGearsRuntimeInstance,
        kind: GoldAndGearsSeededRunStepKind,
        node: NodeId,
        action: GoldAndGearsSeededRunAction,
        result: Option<starclock_activity::BattleResultDigest>,
    ) {
        let accepted = step(
            kind,
            node,
            &self.state,
            &self.rng,
            self.request,
            result,
            instance,
        );
        self.steps.push(accepted);
        self.replay.push(GoldAndGearsSeededReplayStep {
            action,
            state_hash: accepted.state_hash(),
            battle: None,
        });
    }

    fn ensure_budget(&self) -> Result<(), GoldAndGearsSeededRunError> {
        if self.replay.len() >= MAX_SEEDED_RUN_STEPS {
            Err(GoldAndGearsSeededRunError::StepBudgetExceeded)
        } else {
            Ok(())
        }
    }
}

fn is_boss(role: GoldAndGearsEncounterRole) -> bool {
    boss_plane(role).is_some()
}

fn boss_plane(role: GoldAndGearsEncounterRole) -> Option<u8> {
    match role {
        GoldAndGearsEncounterRole::FirstPlaneBoss => Some(1),
        GoldAndGearsEncounterRole::SecondPlaneBoss => Some(2),
        GoldAndGearsEncounterRole::FinalBoss => Some(3),
        GoldAndGearsEncounterRole::Combat | GoldAndGearsEncounterRole::Elite => None,
    }
}

fn next_route(
    instance: &GoldAndGearsRuntimeInstance,
    state: &ActivityTransactionState,
    source: NodeId,
    target: NodeId,
) -> Option<ActivityEdgeId> {
    let mut visited = BTreeSet::from([source]);
    let mut queue = VecDeque::new();
    for edge in instance.graph_definition().outgoing(source) {
        if instance.legal_routes(state, source).contains(&edge.id()) {
            queue.push_back((edge.to(), edge.id()));
        }
    }
    while let Some((node, first)) = queue.pop_front() {
        if node == target {
            return Some(first);
        }
        if !visited.insert(node) {
            continue;
        }
        let legal = instance.legal_routes(state, node);
        for edge in instance.graph_definition().outgoing(node) {
            if legal.contains(&edge.id()) {
                queue.push_back((edge.to(), first));
            }
        }
    }
    None
}
