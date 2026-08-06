use starclock_combat::{
    ActionId, ActionOrigin, AiGraphId, AiStateId, AssemblyDigest, Battle, BattleFault, BattlePhase,
    BattleSeed, BattleStateHash, CombatInputDigest, CombatantSpecDigest,
    CommittedTargetsDiagnostic, ConcedePolicy, ControlledAction, DecisionPoint, DispelCategory,
    DotDefinition, DurationClock, EffectCategory, EffectDefinitionId, EffectInstanceId,
    EffectSnapshotPolicy, EffectStackPolicy, EffectTeardownPolicy, EffectTickPhase, EncounterId,
    EnemyDefinitionId, EnemyPhaseId, FormationIndex, Hp, LifeState, LinkedEntity, LinkedEntityKind,
    ModifierDefinitionId, ModifierInstanceId, OperationId, OwnerLinkPolicy, ParticipantSource,
    PresenceState, ReactionOrderDiagnostic, RuleBundleId, RuleId, RuleInstanceId, Scalar,
    ShieldAmount, ShieldInstanceId, SourceDefinitionId, SpawnSequence, Speed, StatValue,
    StateSlotDefinitionId, TeamResourceWavePolicy, TeamSide, TimelineActorId, ToughnessLayerSpec,
    TransformEndPolicy, UnitDefinitionId, UnitId, UnitLevel, WaveInstanceId, WaveLinkPolicy,
    catalog::{CatalogDigest, action::SkillPointPaymentPolicy},
    formula::{model::CombatElement, shield::ShieldAbsorptionPolicy, toughness::EnemyRank},
    modifier::model::StatQuery,
    rule::model::{OnceKey, RuleValue, SourceClass},
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BattleSnapshot {
    pub identity: BattleIdentitySnapshot,
    pub state_hash: BattleStateHash,
    pub phase: BattlePhase,
    pub fault: Option<BattleFault>,
    pub decision: Option<DecisionPoint>,
    pub committed_revision: u64,
    pub rng_draw_count: u64,
    pub encounter: EncounterSnapshot,
    pub units: Box<[UnitSnapshot]>,
    pub formations: Box<[FormationSnapshot]>,
    pub timeline_actors: Box<[TimelineActorSnapshot]>,
    pub links: Box<[LinkSnapshot]>,
    pub shields: Box<[ShieldSnapshot]>,
    pub break_effects: Box<[BreakEffectSnapshot]>,
    pub effects: Box<[EffectSnapshot]>,
    pub rules: Box<[RuleInstanceSnapshot]>,
    pub modifiers: Box<[ModifierSnapshot]>,
    pub teams: [TeamSnapshot; 2],
    pub active_turn: Option<ActiveTurnSnapshot>,
    pub action_boundary: Option<ActionBoundarySnapshot>,
    pub prepared_action: Option<PreparedActionSnapshot>,
    pub pending_extra_turns: Box<[PendingExtraTurnSnapshot]>,
    pub pending_reactions: Box<[PendingReactionSnapshot]>,
    pub concede_policy: ConcedePolicy,
    pub sequence_cursors: SequenceCursorsSnapshot,
}

impl BattleSnapshot {
    #[must_use]
    pub fn capture(battle: &Battle) -> Self {
        let view = battle.view();
        let identity = view.identity();
        let encounter = view.encounter();
        Self {
            identity: BattleIdentitySnapshot {
                catalog_digest: identity.catalog_digest(),
                combat_input_digest: identity.combat_input_digest(),
                assembly_digest: identity.assembly_digest(),
                seed: identity.seed(),
            },
            state_hash: battle.state_hash(),
            phase: view.phase(),
            fault: view.fault(),
            decision: view.decision().cloned(),
            committed_revision: view.committed_revision(),
            rng_draw_count: view.rng_draw_count(),
            encounter: EncounterSnapshot {
                definition: encounter.definition(),
                wave: encounter.wave(),
                number: encounter.number(),
                total_waves: encounter.total_waves(),
            },
            units: view.units_by_id().map(UnitSnapshot::capture).collect(),
            formations: [TeamSide::Player, TeamSide::Enemy]
                .into_iter()
                .flat_map(|side| view.formation(side))
                .map(|entry| FormationSnapshot {
                    side: entry.side(),
                    index: entry.index(),
                    unit: entry.unit(),
                })
                .collect(),
            timeline_actors: view
                .timeline_actors()
                .map(|actor| TimelineActorSnapshot {
                    id: actor.id(),
                    owner: actor.owner(),
                    unit: actor.unit(),
                    linked_kind: actor.linked_kind(),
                    automatic_ability: actor.automatic_ability(),
                    active: actor.is_active(),
                    action_gauge: actor.action_gauge(),
                    speed: actor.speed(),
                })
                .collect(),
            links: view
                .links()
                .map(|link| LinkSnapshot {
                    owner: link.owner(),
                    entity: link.entity(),
                    kind: link.kind(),
                    owner_defeat: link.owner_defeat_policy(),
                    owner_departure: link.owner_departure_policy(),
                    wave: link.wave_policy(),
                    active: link.is_active(),
                })
                .collect(),
            shields: view
                .shields_by_id()
                .map(|shield| ShieldSnapshot {
                    id: shield.id(),
                    owner: shield.owner(),
                    source_operation: shield.source_operation(),
                    source_effect: shield.source_effect(),
                    remaining: shield.remaining(),
                    policy: shield.policy(),
                })
                .collect(),
            break_effects: view
                .retained_break_effects_by_id()
                .map(|effect| BreakEffectSnapshot {
                    id: effect.id(),
                    owner: effect.owner(),
                    applier: effect.applier(),
                    source_operation: effect.source_operation(),
                    source_definition: effect.source_definition(),
                    plan: effect.plan(),
                    damage: effect.damage(),
                    remaining_turns: effect.remaining_turns(),
                    stacks: effect.stacks(),
                    speed_before: effect.speed_before(),
                })
                .collect(),
            effects: view.effects_by_id().map(EffectSnapshot::capture).collect(),
            rules: view
                .rule_instances_by_id()
                .map(|rule| RuleInstanceSnapshot {
                    id: rule.id(),
                    rule: rule.rule(),
                    owner: rule.owner(),
                    source_effect: rule.source_effect(),
                    slots: rule
                        .slots()
                        .map(|(id, value)| (id, value.clone()))
                        .collect(),
                    once_keys: rule.once_keys().collect(),
                })
                .collect(),
            modifiers: view
                .modifier_instances_by_id()
                .map(|modifier| ModifierSnapshot {
                    id: modifier.id(),
                    definition: modifier.definition(),
                    owner: modifier.owner(),
                    subject: modifier.subject(),
                    source: modifier.source(),
                    source_class: modifier.source_class(),
                    insertion_sequence: modifier.insertion_sequence(),
                    application_action: modifier.application_action(),
                    source_effect: modifier.source_effect(),
                    slots: modifier
                        .slots()
                        .map(|(id, value)| (id, value.clone()))
                        .collect(),
                    captured_value: modifier.captured_value(),
                    captured_stats: modifier
                        .captured_stats()
                        .map(|(query, value)| (*query, value))
                        .collect(),
                })
                .collect(),
            teams: [
                TeamSnapshot::capture(view.team(TeamSide::Player)),
                TeamSnapshot::capture(view.team(TeamSide::Enemy)),
            ],
            active_turn: view.active_turn().map(ActiveTurnSnapshot::capture),
            action_boundary: view
                .action_boundary()
                .map(|boundary| ActionBoundarySnapshot {
                    id: boundary.id(),
                    turn: ActiveTurnSnapshot::capture(boundary.turn()),
                }),
            prepared_action: view
                .prepared_action()
                .map(|prepared| PreparedActionSnapshot {
                    id: prepared.id(),
                    actor: prepared.actor(),
                    ability: prepared.ability(),
                    suspended_boundary: prepared.suspended_boundary(),
                }),
            pending_extra_turns: view
                .pending_extra_turns()
                .map(|pending| PendingExtraTurnSnapshot {
                    insertion: pending.insertion(),
                    unit: pending.unit(),
                })
                .collect(),
            pending_reactions: view
                .pending_reactions()
                .map(|pending| PendingReactionSnapshot {
                    order: pending.order(),
                    root_command: pending.root_command(),
                    parent_event: pending.parent_event(),
                    actor: pending.actor(),
                    owner: pending.owner(),
                    ability: pending.ability(),
                    origin: pending.origin(),
                    targets: pending.targets().clone(),
                    payment: pending.payment(),
                })
                .collect(),
            concede_policy: view.concede_policy(),
            sequence_cursors: SequenceCursorsSnapshot::capture(view.sequence_cursors()),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BattleIdentitySnapshot {
    pub catalog_digest: CatalogDigest,
    pub combat_input_digest: CombatInputDigest,
    pub assembly_digest: AssemblyDigest,
    pub seed: BattleSeed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EncounterSnapshot {
    pub definition: EncounterId,
    pub wave: WaveInstanceId,
    pub number: u16,
    pub total_waves: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SequenceCursorsSnapshot {
    pub next_unit: u64,
    pub next_actor: u64,
    pub next_spawn: u64,
    pub next_wave: u64,
    pub next_decision: u64,
    pub next_action_boundary: u64,
    pub next_prepared_action: u64,
    pub next_command: u64,
    pub next_event: u64,
    pub next_action: u64,
    pub next_phase: u64,
    pub next_hit: u64,
    pub next_operation: u64,
    pub next_shield: u64,
    pub next_effect: u64,
    pub next_rule: u64,
    pub next_modifier: u64,
    pub next_extra_turn: u64,
    pub next_reaction: u64,
}

impl SequenceCursorsSnapshot {
    fn capture(cursors: starclock_combat::SequenceCursorsView) -> Self {
        Self {
            next_unit: cursors.next_unit(),
            next_actor: cursors.next_actor(),
            next_spawn: cursors.next_spawn(),
            next_wave: cursors.next_wave(),
            next_decision: cursors.next_decision(),
            next_action_boundary: cursors.next_action_boundary(),
            next_prepared_action: cursors.next_prepared_action(),
            next_command: cursors.next_command(),
            next_event: cursors.next_event(),
            next_action: cursors.next_action(),
            next_phase: cursors.next_phase(),
            next_hit: cursors.next_hit(),
            next_operation: cursors.next_operation(),
            next_shield: cursors.next_shield(),
            next_effect: cursors.next_effect(),
            next_rule: cursors.next_rule(),
            next_modifier: cursors.next_modifier(),
            next_extra_turn: cursors.next_extra_turn(),
            next_reaction: cursors.next_reaction(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnitSnapshot {
    pub id: UnitId,
    pub spawn: SpawnSequence,
    pub form: UnitDefinitionId,
    pub source: ParticipantSource,
    pub side: TeamSide,
    pub formation: FormationIndex,
    pub entry_wave: u16,
    pub level: UnitLevel,
    pub life: LifeState,
    pub presence: PresenceState,
    pub current_hp: Hp,
    pub maximum_hp: Hp,
    pub base_attack: StatValue,
    pub base_defense: StatValue,
    pub base_speed: Speed,
    pub base_effect_hit_rate: Scalar,
    pub base_effect_resistance: Scalar,
    pub current_energy: starclock_combat::Energy,
    pub maximum_energy: starclock_combat::Energy,
    pub rank: EnemyRank,
    pub weaknesses: Box<[CombatElement]>,
    pub permanent_weaknesses: Box<[CombatElement]>,
    pub temporary_weaknesses: Box<[TemporaryWeaknessSnapshot]>,
    pub weakness_broken: bool,
    pub toughness_layers: Box<[ToughnessLayerSnapshot]>,
    pub abilities: Box<[starclock_combat::AbilityId]>,
    pub rule_bundles: Box<[RuleBundleId]>,
    pub modifiers: Box<[ModifierDefinitionId]>,
    pub resources: Box<[CharacterResourceSnapshot]>,
    pub digest: CombatantSpecDigest,
    pub transformation: Option<TransformationSnapshot>,
    pub enemy_definition: Option<EnemyDefinitionId>,
    pub enemy_ai_state: Option<(AiGraphId, AiStateId, u16)>,
    pub enemy_phase: Option<EnemyPhaseId>,
}

impl UnitSnapshot {
    fn capture(unit: starclock_combat::UnitView<'_>) -> Self {
        Self {
            id: unit.id(),
            spawn: unit.spawn_sequence(),
            form: unit.form(),
            source: unit.source(),
            side: unit.side(),
            formation: unit.formation(),
            entry_wave: unit.entry_wave(),
            level: unit.level(),
            life: unit.life(),
            presence: unit.presence(),
            current_hp: unit.current_hp(),
            maximum_hp: unit.maximum_hp(),
            base_attack: unit.base_attack(),
            base_defense: unit.base_defense(),
            base_speed: unit.base_speed(),
            base_effect_hit_rate: unit.base_effect_hit_rate(),
            base_effect_resistance: unit.base_effect_resistance(),
            current_energy: unit.current_energy(),
            maximum_energy: unit.maximum_energy(),
            rank: unit.rank(),
            weaknesses: unit.weaknesses().into(),
            permanent_weaknesses: unit.permanent_weaknesses().into(),
            temporary_weaknesses: unit
                .temporary_weaknesses()
                .map(|weakness| TemporaryWeaknessSnapshot {
                    element: weakness.element(),
                    applier: weakness.applier(),
                    source_operation: weakness.source_operation(),
                    remaining_turns: weakness.remaining_turns(),
                })
                .collect(),
            weakness_broken: unit.weakness_broken(),
            toughness_layers: unit
                .toughness_layers()
                .map(|layer| ToughnessLayerSnapshot {
                    spec: layer.spec().clone(),
                    current: layer.current(),
                })
                .collect(),
            abilities: unit.abilities().into(),
            rule_bundles: unit.rule_bundles().into(),
            modifiers: unit.modifiers().into(),
            resources: unit
                .character_resources()
                .map(|resource| CharacterResourceSnapshot {
                    stable_key: resource.stable_key().into(),
                    initial: resource.initial(),
                    current: resource.current(),
                    maximum: resource.maximum(),
                })
                .collect(),
            digest: unit.digest(),
            transformation: unit.transformation().map(|state| TransformationSnapshot {
                source_operation: state.source_operation(),
                original_form: state.original_form(),
                original_abilities: state.original_abilities().into(),
                original_presence: state.original_presence(),
                countdown_actor: state.countdown_actor(),
                defeat_policy: state.defeat_policy(),
                wave_policy: state.wave_policy(),
            }),
            enemy_definition: unit.enemy_definition(),
            enemy_ai_state: unit.enemy_ai_state(),
            enemy_phase: unit.enemy_phase(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CharacterResourceSnapshot {
    pub stable_key: Box<str>,
    pub initial: Scalar,
    pub current: Scalar,
    pub maximum: Scalar,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TemporaryWeaknessSnapshot {
    pub element: CombatElement,
    pub applier: UnitId,
    pub source_operation: OperationId,
    pub remaining_turns: u8,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransformationSnapshot {
    pub source_operation: OperationId,
    pub original_form: UnitDefinitionId,
    pub original_abilities: Box<[starclock_combat::AbilityId]>,
    pub original_presence: PresenceState,
    pub countdown_actor: Option<TimelineActorId>,
    pub defeat_policy: TransformEndPolicy,
    pub wave_policy: TransformEndPolicy,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ToughnessLayerSnapshot {
    pub spec: ToughnessLayerSpec,
    pub current: starclock_combat::RawToughness,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FormationSnapshot {
    pub side: TeamSide,
    pub index: FormationIndex,
    pub unit: UnitId,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TimelineActorSnapshot {
    pub id: TimelineActorId,
    pub owner: UnitId,
    pub unit: Option<UnitId>,
    pub linked_kind: Option<LinkedEntityKind>,
    pub automatic_ability: Option<starclock_combat::AbilityId>,
    pub active: bool,
    pub action_gauge: starclock_combat::ActionGauge,
    pub speed: Speed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LinkSnapshot {
    pub owner: UnitId,
    pub entity: LinkedEntity,
    pub kind: LinkedEntityKind,
    pub owner_defeat: OwnerLinkPolicy,
    pub owner_departure: OwnerLinkPolicy,
    pub wave: WaveLinkPolicy,
    pub active: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ShieldSnapshot {
    pub id: ShieldInstanceId,
    pub owner: UnitId,
    pub source_operation: OperationId,
    pub source_effect: Option<EffectDefinitionId>,
    pub remaining: ShieldAmount,
    pub policy: ShieldAbsorptionPolicy,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BreakEffectSnapshot {
    pub id: EffectInstanceId,
    pub owner: UnitId,
    pub applier: UnitId,
    pub source_operation: OperationId,
    pub source_definition: SourceDefinitionId,
    pub plan: starclock_combat::formula::toughness::BaseBreakEffect,
    pub damage: starclock_combat::formula::toughness::BreakDamageDefinition,
    pub remaining_turns: u8,
    pub stacks: u8,
    pub speed_before: Option<Speed>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EffectSnapshot {
    pub id: EffectInstanceId,
    pub definition: EffectDefinitionId,
    pub source_definition: SourceDefinitionId,
    pub source_operation: OperationId,
    pub applier: UnitId,
    pub target: UnitId,
    pub category: EffectCategory,
    pub dispel: DispelCategory,
    pub stacks: u16,
    pub stack_limit: u16,
    pub remaining: Option<u16>,
    pub duration_clock: DurationClock,
    pub tick_phase: EffectTickPhase,
    pub stack_policy: EffectStackPolicy,
    pub snapshot_policy: EffectSnapshotPolicy,
    pub teardown_policy: EffectTeardownPolicy,
    pub application_priority: i32,
    pub magnitude: Scalar,
    pub tags: Box<[SourceDefinitionId]>,
    pub controlled_actions: Box<[ControlledAction]>,
    pub dot: Option<DotDefinition>,
    pub application_sequence: u64,
}

impl EffectSnapshot {
    fn capture(effect: starclock_combat::EffectView<'_>) -> Self {
        Self {
            id: effect.id(),
            definition: effect.definition(),
            source_definition: effect.source_definition(),
            source_operation: effect.source_operation(),
            applier: effect.applier(),
            target: effect.target(),
            category: effect.category(),
            dispel: effect.dispel(),
            stacks: effect.stacks(),
            stack_limit: effect.stack_limit(),
            remaining: effect.remaining(),
            duration_clock: effect.duration_clock(),
            tick_phase: effect.tick_phase(),
            stack_policy: effect.stack_policy(),
            snapshot_policy: effect.snapshot_policy(),
            teardown_policy: effect.teardown_policy(),
            application_priority: effect.application_priority(),
            magnitude: effect.magnitude(),
            tags: effect.tags().into(),
            controlled_actions: effect.controlled_actions().into(),
            dot: effect.dot(),
            application_sequence: effect.application_sequence(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuleInstanceSnapshot {
    pub id: RuleInstanceId,
    pub rule: RuleId,
    pub owner: Option<UnitId>,
    pub source_effect: Option<EffectInstanceId>,
    pub slots: Box<[(StateSlotDefinitionId, RuleValue)]>,
    pub once_keys: Box<[OnceKey]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModifierSnapshot {
    pub id: ModifierInstanceId,
    pub definition: ModifierDefinitionId,
    pub owner: UnitId,
    pub subject: UnitId,
    pub source: SourceDefinitionId,
    pub source_class: SourceClass,
    pub insertion_sequence: u64,
    pub application_action: Option<ActionId>,
    pub source_effect: Option<EffectInstanceId>,
    pub slots: Box<[(StateSlotDefinitionId, RuleValue)]>,
    pub captured_value: Option<Scalar>,
    pub captured_stats: Box<[(StatQuery, Scalar)]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TeamSnapshot {
    pub side: TeamSide,
    pub initial_skill_points: u16,
    pub skill_points: u16,
    pub maximum_skill_points: u16,
    pub keyed_resources: Box<[TeamResourceSnapshot]>,
}

impl TeamSnapshot {
    fn capture(team: starclock_combat::TeamView<'_>) -> Self {
        Self {
            side: team.side(),
            initial_skill_points: team.initial_skill_points(),
            skill_points: team.skill_points(),
            maximum_skill_points: team.maximum_skill_points(),
            keyed_resources: team
                .keyed_resources()
                .map(|resource| TeamResourceSnapshot {
                    id: resource.id(),
                    stable_key: resource.stable_key().map(Into::into),
                    initial: resource.initial(),
                    current: resource.current(),
                    maximum: resource.maximum(),
                    wave_policy: resource.wave_policy(),
                })
                .collect(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TeamResourceSnapshot {
    pub id: SourceDefinitionId,
    pub stable_key: Option<Box<str>>,
    pub initial: u16,
    pub current: u16,
    pub maximum: u16,
    pub wave_policy: TeamResourceWavePolicy,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ActiveTurnSnapshot {
    pub actor: TimelineActorId,
    pub owner: UnitId,
    pub unit: UnitId,
    pub automatic: Option<(starclock_combat::AbilityId, ActionOrigin)>,
    pub side: TeamSide,
    pub formation: FormationIndex,
    pub spawn: SpawnSequence,
    pub origin: ActionOrigin,
}

impl ActiveTurnSnapshot {
    fn capture(turn: starclock_combat::ActiveTurnView) -> Self {
        Self {
            actor: turn.actor(),
            owner: turn.owner(),
            unit: turn.unit(),
            automatic: turn.automatic(),
            side: turn.side(),
            formation: turn.formation(),
            spawn: turn.spawn_sequence(),
            origin: turn.origin(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ActionBoundarySnapshot {
    pub id: starclock_combat::ActionBoundaryId,
    pub turn: ActiveTurnSnapshot,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PreparedActionSnapshot {
    pub id: starclock_combat::PreparedActionId,
    pub actor: UnitId,
    pub ability: starclock_combat::AbilityId,
    pub suspended_boundary: starclock_combat::ActionBoundaryId,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PendingExtraTurnSnapshot {
    pub insertion: u64,
    pub unit: UnitId,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PendingReactionSnapshot {
    pub order: ReactionOrderDiagnostic,
    pub root_command: starclock_combat::CommandId,
    pub parent_event: starclock_combat::EventId,
    pub actor: UnitId,
    pub owner: UnitId,
    pub ability: starclock_combat::AbilityId,
    pub origin: ActionOrigin,
    pub targets: CommittedTargetsDiagnostic,
    pub payment: Option<SkillPointPaymentPolicy>,
}
