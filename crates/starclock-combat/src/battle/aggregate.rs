use std::sync::Arc;

use crate::{
    actor::{
        model::{LifeState, PresenceState},
        store::{
            FormationEntry, FormationState, TeamState, TeamStateStore, TimelineActorState,
            TimelineActorStore, UnitState, UnitStore,
        },
    },
    catalog::CombatCatalog,
    command::{
        legal,
        model::{Command, CommandError, DecisionPoint},
        validate::{ValidatedCommand, validate},
    },
    numeric::domain::ActionGauge,
};

use super::{
    build::{BattleBuildError, validate as validate_build},
    model::{BattlePhase, Resolution},
    spec::{BattleSeed, BattleSpec, TeamSide},
    state::{BattleIdentity, BattleState, EncounterState, SequenceState},
    view::BattleView,
};

const BASE_ACTION_GAUGE_SCALED: i64 = 10_000_000_000;

/// Deterministic aggregate owning exactly one isolated battle.
#[derive(Debug)]
pub struct Battle {
    _catalog: Arc<CombatCatalog>,
    state: BattleState,
}

impl Battle {
    /// Validates a complete battle request and allocates runtime IDs canonically.
    pub fn create(
        catalog: Arc<CombatCatalog>,
        spec: BattleSpec,
        seed: BattleSeed,
    ) -> Result<Self, BattleBuildError> {
        validate_build(&catalog, &spec)?;
        let mut sequences = SequenceState::new();
        let first_decision = sequences.decision();
        let wave = sequences.wave();
        let mut units = UnitStore::default();
        let mut actors = TimelineActorStore::default();
        let mut formations = FormationState::default();

        for participant in spec.participants() {
            let unit_id = sequences.unit();
            let actor_id = sequences.actor();
            let spawn = sequences.spawn();
            let combatant = participant.combatant();
            units.insert(UnitState {
                id: unit_id,
                spawn,
                form: combatant.form(),
                source: participant.source(),
                side: participant.side(),
                formation: participant.formation(),
                level: combatant.level(),
                life: LifeState::Alive,
                presence: PresenceState::Present,
                current_hp: combatant.maximum_hp(),
                maximum_hp: combatant.maximum_hp(),
                abilities: combatant.abilities().into(),
                rule_bundles: combatant.rule_bundles().into(),
                modifiers: combatant.modifiers().into(),
                digest: combatant.digest(),
            });
            actors.insert(TimelineActorState {
                id: actor_id,
                owner: unit_id,
                gauge: ActionGauge::from_scaled(BASE_ACTION_GAUGE_SCALED)
                    .expect("positive base Action Gauge is in domain"),
                speed: combatant.speed(),
            });
            formations.push(FormationEntry {
                side: participant.side(),
                index: participant.formation(),
                unit: unit_id,
            });
        }

        let player_resources = spec.resources(TeamSide::Player);
        let enemy_resources = spec.resources(TeamSide::Enemy);
        let teams = TeamStateStore::new(
            TeamState {
                side: TeamSide::Player,
                skill_points: player_resources.skill_points(),
                maximum_skill_points: player_resources.maximum_skill_points(),
            },
            TeamState {
                side: TeamSide::Enemy,
                skill_points: enemy_resources.skill_points(),
                maximum_skill_points: enemy_resources.maximum_skill_points(),
            },
        );
        let state = BattleState {
            identity: BattleIdentity {
                catalog_revision: catalog.revision().clone(),
                catalog_digest: catalog.digest(),
                rules_revision: spec.rules_revision().into(),
                spec_digest: spec.digest(),
                seed,
            },
            phase: BattlePhase::Initializing,
            decision: Some(legal::battle_start(first_decision)),
            units,
            actors,
            formations,
            teams,
            encounter: EncounterState {
                definition: spec.encounter(),
                wave,
            },
            concede: spec.concede_policy(),
            rng: BattleState::rng_from_seed(seed),
            sequences,
            committed_revision: 0,
        };
        Ok(Self {
            _catalog: catalog,
            state,
        })
    }

    /// Applies exactly one offered command and returns at a stable boundary.
    ///
    /// Rejections complete before mutation and consume no RNG. The reusable
    /// transaction, event/cause and hash boundary is introduced by `G01-P3-B2`.
    pub fn apply(&mut self, command: Command) -> Result<Resolution, CommandError> {
        let validated = validate(&self.state, &command)?;
        self.state.phase = BattlePhase::Resolving;
        match validated {
            ValidatedCommand::StartBattle => {
                let decision_id = self.state.sequences.decision();
                self.state.decision = Some(legal::initial_player_action(
                    decision_id,
                    self.state.concede,
                ));
                self.state.phase = BattlePhase::AwaitingCommand;
            }
            ValidatedCommand::Concede => {
                self.state.decision = None;
                self.state.phase = BattlePhase::Lost;
            }
        }
        self.state.committed_revision = self
            .state
            .committed_revision
            .checked_add(1)
            .expect("a battle cannot accept u64::MAX commands under reviewed limits");
        Ok(Resolution::new(
            self.state.phase,
            self.state.decision.clone(),
            self.state.committed_revision,
            self.state.rng.draw_count(),
        ))
    }

    /// Returns an immutable projection of authoritative state.
    #[must_use]
    pub const fn view(&self) -> BattleView<'_> {
        BattleView { state: &self.state }
    }

    /// Returns the active offered decision, or `None` at a terminal boundary.
    #[must_use]
    pub const fn decision(&self) -> Option<&DecisionPoint> {
        self.state.decision.as_ref()
    }
}
