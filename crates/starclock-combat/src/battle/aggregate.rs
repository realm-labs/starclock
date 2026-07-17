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
    codec::{BattleStateHash, hash_state},
    command::{
        legal,
        model::{Command, CommandError, DecisionPoint},
        validate::{ValidatedCommand, validate},
    },
    numeric::domain::ActionGauge,
    resolver::transaction::{FaultInjection, ResolutionScratch, resolve_prepared},
};

use super::{
    build::{BattleBuildError, validate as validate_build},
    model::{BattlePhase, Resolution, ResolutionBoundary},
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
    scratch: Option<ResolutionScratch>,
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
                current_energy: combatant.current_energy(),
                maximum_energy: combatant.maximum_energy(),
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
            fault: None,
            decision: Some(legal::battle_start(first_decision)),
            units,
            actors,
            formations,
            teams,
            encounter: EncounterState {
                definition: spec.encounter(),
                wave,
            },
            timeline: crate::timeline::state::TimelineState::default(),
            concede: spec.concede_policy(),
            rng: BattleState::rng_from_seed(seed),
            sequences,
            committed_revision: 0,
        };
        Ok(Self {
            _catalog: catalog,
            state,
            scratch: None,
        })
    }

    /// Applies exactly one offered command and returns at a stable boundary.
    ///
    /// Rejections complete before scratch preparation or mutation and consume
    /// no IDs or RNG. Accepted resolution settles synchronously and atomically.
    pub fn apply(&mut self, command: Command) -> Result<Resolution, CommandError> {
        let validated = validate(&self.state, &command)?;
        Ok(self.apply_validated(validated, None))
    }

    fn apply_validated(
        &mut self,
        validated: ValidatedCommand,
        injection: Option<FaultInjection>,
    ) -> Resolution {
        let mut scratch = match self.scratch.take() {
            Some(mut scratch) => {
                scratch.prepare(&self.state);
                scratch
            }
            None => ResolutionScratch::from_state(&self.state),
        };
        let output = resolve_prepared(
            &self._catalog,
            &self.state,
            &mut scratch,
            validated,
            injection,
        );
        scratch.commit_into(&mut self.state);
        debug_assert_eq!(hash_state(&self.state), output.state_hash);
        self.scratch = Some(scratch);
        Resolution::new(ResolutionBoundary {
            phase: self.state.phase,
            next_decision: self.state.decision.clone(),
            committed_revision: self.state.committed_revision,
            rng_draw_count: self.state.rng.draw_count(),
            root_command: output.root_command,
            events: output.events,
            state_hash: output.state_hash,
            fault: output.fault,
        })
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

    /// Streams the current canonical state directly into SHA-256.
    #[must_use]
    pub fn state_hash(&self) -> BattleStateHash {
        hash_state(&self.state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        BattleEventData, BattleEventKind, BattleSpecDigest, CombatantSpecDigest, ConcedePolicy,
        FaultKind, FaultPolicy, FormationIndex, Hp, ParticipantSource, ParticipantSpec,
        ResolvedCombatantSpec, ResolvedDefinitionBindings, Speed, TeamResourceSpec, UnitLevel,
        catalog::{
            action::{
                AbilityActionDefinition, AbilityKind, ActionResourcePolicy,
                TargetInvalidationPolicy, TargetPattern, TargetRelation, UnitTargetSelector,
            },
            builder::CombatCatalogBuilder,
            definition::{
                AbilityDefinition, EncounterDefinition, EnemyDefinition, ProgramDefinition,
                SelectorDefinition, UnitDefinition,
            },
        },
        codec::{collect_state, hash_collected_state},
        command::model::CommandErrorKind,
        event::model::FaultEventData,
        id::{AbilityId, EncounterId, EnemyDefinitionId, ProgramId, SelectorId, UnitDefinitionId},
        resolver::transaction::{FaultInjection, FaultInjectionPoint},
    };

    fn definition<I: TryFrom<u32>>(raw: u32) -> I
    where
        I::Error: core::fmt::Debug,
    {
        I::try_from(raw).expect("test definition ID is non-zero")
    }

    fn fixture_catalog() -> Arc<CombatCatalog> {
        let mut builder = CombatCatalogBuilder::new("transaction-test-v1", [0x41; 32]);
        for raw in 1..=2 {
            let selector: SelectorId = definition(raw);
            let program: ProgramId = definition(raw);
            let ability: AbilityId = definition(raw);
            let unit: UnitDefinitionId = definition(raw);
            builder.add_selector(SelectorDefinition::new(selector).with_unit_targets(
                UnitTargetSelector::new(TargetRelation::SelfUnit, TargetPattern::Single).unwrap(),
            ));
            builder.add_program(ProgramDefinition::new(
                program,
                vec![],
                vec![selector],
                vec![],
                vec![],
            ));
            builder.add_ability(
                AbilityDefinition::new(ability, program, selector, vec![]).with_action(
                    AbilityActionDefinition::new(
                        AbilityKind::Basic,
                        1,
                        TargetInvalidationPolicy::CancelRemainingForTarget,
                        ActionResourcePolicy::new(0, 0, crate::Energy::ZERO, crate::Energy::ZERO),
                    )
                    .unwrap(),
                ),
            );
            builder.add_unit(UnitDefinition::new(unit, vec![ability], vec![]));
        }
        let enemy: EnemyDefinitionId = definition(1);
        builder.add_enemy(EnemyDefinition::new(
            enemy,
            definition(2),
            vec![definition(2)],
        ));
        builder.add_encounter(EncounterDefinition::new(
            definition::<EncounterId>(1),
            vec![enemy],
            vec![],
        ));
        builder.build().expect("transaction test catalog is valid")
    }

    fn combatant(form: u32, digest: u8) -> ResolvedCombatantSpec {
        ResolvedCombatantSpec::new(
            definition(form),
            UnitLevel::new(80).unwrap(),
            Hp::new(1_000).unwrap(),
            Speed::from_scaled(100_000_000).unwrap(),
            ResolvedDefinitionBindings::new(vec![definition(form)], vec![], vec![]).unwrap(),
            CombatantSpecDigest::new([digest; 32]).unwrap(),
        )
        .unwrap()
    }

    fn fixture_battle() -> Battle {
        let spec = BattleSpec::new(
            "transaction-rules-v1",
            BattleSpecDigest::new([0x51; 32]).unwrap(),
            definition(1),
            vec![
                ParticipantSpec::new(
                    TeamSide::Player,
                    FormationIndex::new(0).unwrap(),
                    ParticipantSource::Player,
                    combatant(1, 0x61),
                ),
                ParticipantSpec::new(
                    TeamSide::Enemy,
                    FormationIndex::new(4).unwrap(),
                    ParticipantSource::EncounterEnemy(definition(1)),
                    combatant(2, 0x62),
                ),
            ],
            TeamResourceSpec::new(3, 5).unwrap(),
            TeamResourceSpec::new(0, 0).unwrap(),
            ConcedePolicy::Allowed,
        )
        .unwrap();
        Battle::create(fixture_catalog(), spec, BattleSeed::new([0x71; 32])).unwrap()
    }

    fn injected_start(battle: &mut Battle, injection: FaultInjection) -> Resolution {
        let decision = battle.decision().unwrap().id();
        let command = Command::StartBattle { decision };
        let validated = validate(&battle.state, &command).unwrap();
        battle.apply_validated(validated, Some(injection))
    }

    #[test]
    fn rejected_commands_never_prepare_transaction_scratch() {
        let mut battle = fixture_battle();
        assert!(battle.scratch.is_none());
        let before = battle.state_hash();
        let error = battle
            .apply(Command::Concede {
                decision: battle.decision().unwrap().id(),
            })
            .unwrap_err();
        assert_eq!(error.kind(), CommandErrorKind::NotOffered);
        assert_eq!(battle.state_hash(), before);
        assert!(battle.scratch.is_none());

        let start = Command::StartBattle {
            decision: battle.decision().unwrap().id(),
        };
        battle.apply(start).unwrap();
        assert_eq!(battle.scratch.as_ref().unwrap().preparations(), 1);
        let awaiting = battle.state_hash();
        let error = battle
            .apply(Command::StartBattle {
                decision: battle.decision().unwrap().id(),
            })
            .unwrap_err();
        assert_eq!(error.kind(), CommandErrorKind::NotOffered);
        assert_eq!(battle.state_hash(), awaiting);
        assert_eq!(battle.scratch.as_ref().unwrap().preparations(), 1);
    }

    #[test]
    fn rollback_discards_different_transient_work_to_one_byte_identical_fault() {
        let mut early = fixture_battle();
        let mut late = fixture_battle();
        let early_resolution = injected_start(
            &mut early,
            FaultInjection {
                point: FaultInjectionPoint::AfterResolvingPhase,
                policy: FaultPolicy::Rollback,
            },
        );
        let late_resolution = injected_start(
            &mut late,
            FaultInjection {
                point: FaultInjectionPoint::AfterCommandMutation,
                policy: FaultPolicy::Rollback,
            },
        );

        assert_eq!(early_resolution, late_resolution);
        assert_eq!(early_resolution.phase(), BattlePhase::Faulted);
        assert_eq!(early_resolution.committed_revision(), 1);
        let fault = early_resolution.fault().unwrap();
        assert_eq!(fault.kind(), FaultKind::InvariantViolation);
        assert_eq!(fault.policy(), FaultPolicy::Rollback);
        assert_eq!(early_resolution.events().len(), 1);
        assert_eq!(early_resolution.events()[0].cause().parent_event(), None);
        assert_eq!(early_resolution.events()[0].cause().root_command().get(), 1);
        assert_eq!(
            early_resolution.events()[0].kind(),
            &BattleEventKind::Fault(FaultEventData::new(fault))
        );
        assert_eq!(early.state_hash(), late.state_hash());
        assert_eq!(collect_state(&early.state), collect_state(&late.state));
        assert_eq!(early.scratch.as_ref().unwrap().preparations(), 2);
        assert_eq!(late.scratch.as_ref().unwrap().preparations(), 2);

        assert_eq!(hash_collected_state(&early.state), early.state_hash());
    }

    #[test]
    fn commit_fault_preserves_completed_facts_and_appends_stable_failure() {
        let mut battle = fixture_battle();
        let resolution = injected_start(
            &mut battle,
            FaultInjection {
                point: FaultInjectionPoint::AfterCommandMutation,
                policy: FaultPolicy::CommitFault,
            },
        );
        assert_eq!(resolution.phase(), BattlePhase::Faulted);
        assert_eq!(resolution.committed_revision(), 1);
        assert_eq!(resolution.events().len(), 4);
        assert!(matches!(
            resolution.events()[0].kind(),
            BattleEventKind::Battle(BattleEventData::Started)
        ));
        assert!(matches!(
            resolution.events()[1].kind(),
            BattleEventKind::Turn(_)
        ));
        assert!(matches!(
            resolution.events()[2].kind(),
            BattleEventKind::Decision(_)
        ));
        let fault = resolution.fault().unwrap();
        assert_eq!(fault.policy(), FaultPolicy::CommitFault);
        assert_eq!(
            resolution.events()[3].cause().parent_event().unwrap().get(),
            3
        );
        assert_eq!(
            resolution.events()[3].kind(),
            &BattleEventKind::Fault(FaultEventData::new(fault))
        );
        assert!(
            resolution
                .events()
                .iter()
                .all(|event| event.cause().root_command() == resolution.root_command())
        );
        assert_eq!(resolution.state_hash(), battle.state_hash());
        assert_eq!(battle.view().fault(), Some(fault));
        assert!(battle.decision().is_none());
    }
}
