use std::sync::Arc;

use crate::{
    actor::{
        model::{LifeState, PresenceState},
        store::{
            EnemyRuntimeState, FormationEntry, FormationState, KeyedTeamResourceState, TeamState,
            TeamStateStore, TimelineActorState, TimelineActorStore, UnitState, UnitStore,
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
        let mut modifiers = crate::modifier::state::ModifierStore::default();

        for participant in spec.participants() {
            let unit_id = sequences.unit();
            let actor_id = sequences.actor();
            let spawn = sequences.spawn();
            let combatant = participant.combatant();
            let enemy = enemy_runtime(&catalog, encounter_slot(&catalog, &spec, participant));
            let definition = catalog
                .unit(combatant.form())
                .expect("battle build validated unit definition");
            units.insert(UnitState {
                id: unit_id,
                spawn,
                form: combatant.form(),
                source: participant.source(),
                side: participant.side(),
                formation: participant.formation(),
                entry_wave: participant.wave(),
                level: combatant.level(),
                life: LifeState::Alive,
                presence: if participant.side() == TeamSide::Enemy && participant.wave() > 1 {
                    PresenceState::Reserved
                } else {
                    PresenceState::Present
                },
                current_hp: combatant.maximum_hp(),
                maximum_hp: combatant.maximum_hp(),
                base_attack: combatant.base_attack(),
                base_defense: combatant.base_defense(),
                base_speed: combatant.speed(),
                current_energy: combatant.current_energy(),
                maximum_energy: combatant.maximum_energy(),
                rank: combatant.rank(),
                weaknesses: combatant.weaknesses().to_vec(),
                permanent_weaknesses: combatant.weaknesses().into(),
                temporary_weaknesses: Vec::new(),
                toughness_layers: combatant
                    .toughness_layers()
                    .iter()
                    .cloned()
                    .map(crate::toughness::state::ToughnessLayerState::from_spec)
                    .collect::<Vec<_>>(),
                weakness_broken: false,
                abilities: combatant.abilities().into(),
                rule_bundles: combatant.rule_bundles().into(),
                modifiers: combatant.modifiers().into(),
                resources: definition
                    .resources()
                    .iter()
                    .map(|resource| crate::actor::store::CharacterResourceState {
                        stable_key: resource.stable_key().into(),
                        initial: resource.initial(),
                        current: resource.initial(),
                        maximum: resource.maximum(),
                    })
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
                digest: combatant.digest(),
                transformation: None,
                enemy,
            });
            for binding in combatant.modifier_bindings() {
                let source = combatant
                    .sources()
                    .binary_search_by_key(&binding.source(), |source| source.definition())
                    .ok()
                    .map(|index| &combatant.sources()[index])
                    .expect("battle build validated modifier source");
                let instance = sequences.modifier();
                let inserted = modifiers.insert(crate::modifier::model::ActiveModifier {
                    instance,
                    definition: binding.definition(),
                    owner: unit_id,
                    subject: unit_id,
                    source: binding.source(),
                    source_class: source.class(),
                    insertion_sequence: instance.get(),
                    application_action: None,
                    source_effect: None,
                    slots: Box::new([]),
                    captured_value: None,
                    captured_stats: Box::new([]),
                });
                debug_assert!(inserted);
            }
            actors.insert(TimelineActorState {
                id: actor_id,
                owner: unit_id,
                unit: Some(unit_id),
                kind: None,
                automatic_ability: None,
                active: true,
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
                initial_skill_points: player_resources.skill_points(),
                skill_points: player_resources.skill_points(),
                maximum_skill_points: player_resources.maximum_skill_points(),
                keyed_resources: player_resources
                    .keyed()
                    .iter()
                    .map(|entry| KeyedTeamResourceState {
                        id: entry.id(),
                        stable_key: entry.stable_key().map(Into::into),
                        initial: entry.initial(),
                        current: entry.initial(),
                        maximum: entry.maximum(),
                        wave: entry.wave(),
                    })
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
            },
            TeamState {
                side: TeamSide::Enemy,
                initial_skill_points: enemy_resources.skill_points(),
                skill_points: enemy_resources.skill_points(),
                maximum_skill_points: enemy_resources.maximum_skill_points(),
                keyed_resources: enemy_resources
                    .keyed()
                    .iter()
                    .map(|entry| KeyedTeamResourceState {
                        id: entry.id(),
                        stable_key: entry.stable_key().map(Into::into),
                        initial: entry.initial(),
                        current: entry.initial(),
                        maximum: entry.maximum(),
                        wave: entry.wave(),
                    })
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
            },
        );
        let mut rules = crate::rule::state::RuleStateStore::default();
        for unit in units.iter_by_id() {
            for bundle_id in &unit.rule_bundles {
                let bundle = catalog
                    .rule_bundle(*bundle_id)
                    .expect("battle build validated bundle");
                for rule_id in bundle.rules() {
                    let definition = catalog
                        .rule(*rule_id)
                        .expect("catalog validated rule reference");
                    if let Some(runtime) = definition.runtime() {
                        let inserted =
                            rules.insert(sequences.rule(), *rule_id, Some(unit.id), runtime);
                        debug_assert!(inserted);
                    }
                }
            }
        }
        let encounter = catalog
            .encounter(spec.encounter())
            .expect("battle build validated encounter");
        for bundle_id in encounter.rule_bundles() {
            let bundle = catalog
                .rule_bundle(*bundle_id)
                .expect("catalog validated bundle");
            for rule_id in bundle.rules() {
                let definition = catalog
                    .rule(*rule_id)
                    .expect("catalog validated rule reference");
                if let Some(runtime) = definition.runtime() {
                    let inserted = rules.insert(sequences.rule(), *rule_id, None, runtime);
                    debug_assert!(inserted);
                }
            }
        }
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
            links: crate::actor::store::LinkStore::default(),
            formations,
            teams,
            shields: crate::effect::shield::ShieldStore::default(),
            break_effects: crate::effect::break_effect::BreakEffectStore::default(),
            effects: crate::effect::state::EffectStore::default(),
            rules,
            modifiers,
            encounter: EncounterState {
                definition: spec.encounter(),
                wave,
                number: 1,
                total_waves: u16::try_from(
                    catalog
                        .encounter(spec.encounter())
                        .expect("battle build validation resolved encounter")
                        .waves()
                        .len(),
                )
                .expect("catalog encounter wave count is bounded by u16"),
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

    /// Returns non-authoritative structural measurements for benchmark tooling.
    #[cfg(feature = "benchmark-instrumentation")]
    #[must_use]
    pub fn performance_snapshot(&self) -> crate::benchmark::BattlePerformanceSnapshot {
        let metrics = self
            .scratch
            .as_ref()
            .map_or_else(Default::default, ResolutionScratch::last_metrics);
        crate::benchmark::BattlePerformanceSnapshot::new(
            crate::codec::canonical_state_len(&self.state),
            metrics.entries,
            metrics.events,
            metrics.operations,
            metrics.retained_bytes,
        )
    }
}

fn encounter_slot(
    catalog: &CombatCatalog,
    spec: &BattleSpec,
    participant: &super::spec::ParticipantSpec,
) -> Option<(crate::EnemyDefinitionId, Option<crate::EnemyPhaseId>)> {
    let super::spec::ParticipantSource::EncounterEnemy(enemy) = participant.source() else {
        return None;
    };
    let slot = catalog
        .encounter(spec.encounter())?
        .wave(participant.wave())?
        .slots()
        .iter()
        .find(|slot| {
            slot.enemy() == enemy
                && slot
                    .formation()
                    .is_none_or(|formation| formation == participant.formation())
        })?;
    Some((enemy, slot.initial_phase()))
}

fn enemy_runtime(
    catalog: &CombatCatalog,
    occurrence: Option<(crate::EnemyDefinitionId, Option<crate::EnemyPhaseId>)>,
) -> Option<EnemyRuntimeState> {
    let (definition, phase_id) = occurrence?;
    let enemy = catalog
        .enemy(definition)
        .expect("battle build validated enemy definition");
    let phase = phase_id.and_then(|id| enemy.phases().iter().find(|phase| phase.id() == id));
    let graph = phase
        .map(crate::catalog::encounter::EnemyPhaseDefinition::ai_graph)
        .or_else(|| enemy.ai_graph())?;
    let state = catalog
        .ai_graph(graph)
        .expect("catalog validated enemy AI graph")
        .initial_state();
    Some(EnemyRuntimeState {
        definition,
        graph,
        state,
        turn_counter: 0,
        phase: phase_id,
    })
}

#[cfg(test)]
mod tests {
    use proptest::{
        collection::vec,
        prelude::*,
        test_runner::{Config as ProptestConfig, FileFailurePersistence, RngAlgorithm, RngSeed},
    };

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
        command::model::{CommandErrorKind, DecisionKind},
        event::model::FaultEventData,
        id::{AbilityId, EncounterId, EnemyDefinitionId, ProgramId, SelectorId, UnitDefinitionId},
        resolver::transaction::{FaultInjection, FaultInjectionPoint},
    };

    const COMMAND_SEQUENCE_SEED: u64 = 0x636f_6d6d_616e_6431;
    const ROLLBACK_SEQUENCE_SEED: u64 = 0x726f_6c6c_6261_636b;

    fn property_config(seed: u64) -> ProptestConfig {
        ProptestConfig {
            cases: 256,
            max_shrink_iters: 4_096,
            failure_persistence: Some(Box::new(FileFailurePersistence::SourceParallel(
                "proptest-regressions",
            ))),
            rng_algorithm: RngAlgorithm::ChaCha,
            rng_seed: RngSeed::Fixed(seed),
            ..ProptestConfig::default()
        }
    }

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

    fn supported_command(battle: &Battle) -> Command {
        let decision = battle.decision().expect("fixture remains nonterminal");
        let selected = match decision.kind() {
            DecisionKind::BattleStart => decision.legal_commands().first(),
            DecisionKind::InterruptWindow => decision
                .legal_commands()
                .iter()
                .find(|command| matches!(command, Command::PassInterruptWindow { .. })),
            DecisionKind::NormalAction => decision
                .legal_commands()
                .iter()
                .find(|command| matches!(command, Command::UseAbility { .. })),
            DecisionKind::BattleChoice => None,
        };
        selected
            .cloned()
            .expect("fixture offers a supported command")
    }

    fn injected_command(
        battle: &mut Battle,
        command: &Command,
        point: FaultInjectionPoint,
    ) -> Resolution {
        let validated = validate(&battle.state, command).expect("offered command validates");
        battle.apply_validated(
            validated,
            Some(FaultInjection {
                point,
                policy: FaultPolicy::Rollback,
            }),
        )
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

    proptest! {
        #![proptest_config(property_config(COMMAND_SEQUENCE_SEED))]

        #[test]
        fn generated_command_sequences_are_deterministic_and_rejections_are_inert(
            steps in vec(any::<u8>(), 1..129),
        ) {
            let mut first = fixture_battle();
            let mut second = fixture_battle();
            let mut trace = Vec::new();

            for step in steps {
                if step % 4 == 0 {
                    let decision = first.decision().unwrap().id();
                    prop_assert_eq!(decision, second.decision().unwrap().id());
                    let forged_decision = crate::DecisionId::new(
                        decision.get().checked_add(10_000).unwrap()
                    ).unwrap();
                    let forged = Command::StartBattle { decision: forged_decision };
                    let before_bytes = collect_state(&first.state);
                    let before_hash = first.state_hash();
                    let before_draws = first.view().rng_draw_count();
                    let before_decision = first.decision().cloned();
                    let first_error = first.apply(forged.clone()).unwrap_err();
                    let second_error = second.apply(forged).unwrap_err();
                    prop_assert_eq!(first_error, second_error);
                    prop_assert_eq!(collect_state(&first.state), before_bytes);
                    prop_assert_eq!(first.state_hash(), before_hash);
                    prop_assert_eq!(first.view().rng_draw_count(), before_draws);
                    prop_assert_eq!(first.decision(), before_decision.as_ref());
                    prop_assert_eq!(collect_state(&first.state), collect_state(&second.state));
                } else {
                    let first_command = supported_command(&first);
                    let second_command = supported_command(&second);
                    prop_assert_eq!(&first_command, &second_command);
                    let first_resolution = first.apply(first_command).unwrap();
                    let second_resolution = second.apply(second_command).unwrap();
                    prop_assert_eq!(&first_resolution, &second_resolution);
                    prop_assert_eq!(collect_state(&first.state), collect_state(&second.state));
                    trace.push(first_resolution.state_hash());
                }
            }
            prop_assert_eq!(first.state_hash(), second.state_hash());
            prop_assert_eq!(trace.last().copied().unwrap_or_else(|| first.state_hash()), first.state_hash());
        }
    }

    proptest! {
        #![proptest_config(property_config(ROLLBACK_SEQUENCE_SEED))]

        #[test]
        fn rollback_converges_after_every_generated_valid_prefix(prefix in 0_usize..65) {
            let mut early = fixture_battle();
            let mut late = fixture_battle();
            for _ in 0..prefix {
                let command = supported_command(&early);
                prop_assert_eq!(&command, &supported_command(&late));
                let early_resolution = early.apply(command.clone()).unwrap();
                let late_resolution = late.apply(command).unwrap();
                prop_assert_eq!(early_resolution, late_resolution);
            }
            let command = supported_command(&early);
            prop_assert_eq!(&command, &supported_command(&late));
            let early_resolution = injected_command(
                &mut early,
                &command,
                FaultInjectionPoint::AfterResolvingPhase,
            );
            let late_resolution = injected_command(
                &mut late,
                &command,
                FaultInjectionPoint::AfterCommandMutation,
            );
            prop_assert_eq!(early_resolution, late_resolution);
            prop_assert_eq!(collect_state(&early.state), collect_state(&late.state));
            prop_assert_eq!(early.state_hash(), late.state_hash());
            prop_assert_eq!(early.state_hash(), hash_collected_state(&early.state));
        }
    }
}
