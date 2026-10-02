//! Controlled commands preserve the production Curio rules and original forms.
use crate::divergent_universe::{
    DivergentUniverseAssembledBattle, battle_team_resources::PUNCHLINE,
};
use starclock_combat::{
    ActionGauge, AssemblyDigest, Battle, BattleEvent, BattleSeed, BattleSpec, CombatantSpecDigest,
    Command, ConcedePolicy, DispelCategory, EffectRemovalDefinition, Energy, FormationIndex, Hp,
    KeyedTeamResourceSpec, LifeState, LinkedEntityKind, LinkedUnitCatalogDefinition,
    LinkedUnitDefinition, OwnerLinkPolicy, ParticipantInitialState, ParticipantSource,
    ParticipantSpec, PresenceState, Ratio, ResolvedCombatantSpec, ResolvedDefinitionBindings,
    Rounding, Scalar, SourceDefinitionId, Speed, TeamResourceSpec, TeamResourceWavePolicy,
    TeamSide, UnitId, WaveLinkPolicy,
    catalog::{
        action::{
            AbilityActionDefinition, AbilityKind, AbilityProgramBinding, AbilityProgramTiming,
            AbilityTag, ActionHitDefinition, ActionResourcePolicy, HitCritPolicy,
            HitOperationDefinition, HitTargetGroup, OrdinaryDamageDefinition,
            OrdinaryDamageMultipliers, ReactionBoundary, TargetInvalidationPolicy, TargetPattern,
            TargetRelation, TeamResourceChange, TeamResourceChangeDefinition, UnitTargetSelector,
        },
        builder::CombatCatalogBuilder,
        definition::{
            AbilityDefinition, EncounterDefinition, EnemyDefinition, ProgramDefinition,
            SelectorDefinition, UnitDefinition,
        },
        selector::{
            RuleEmptyPoolPolicy, RuleLifePredicate, RulePresencePredicate, RuleSelectorChoice,
            RuleSelectorOrdering, RuleSelectorOrigin, RuleSelectorPredicate, RuleSelectorReference,
            RuleSelectorSide, RuleUnitSelector,
        },
    },
    modifier::model::{FormulaPurpose, StatKind, StatQuerySubject},
    rule::model::{
        ProgramStep, ReactionPriority, ResourceUpdateKind, RuleActionOwner,
        RuleActionPaymentPolicy, RuleOperationTemplate, RuleResourceKind, RuleValue, ValueExpr,
    },
};

pub(super) const EFFECT: u32 = 0x7ec5_0001;
pub(super) const BASIC: u32 = 0x7dd0_0001;
pub(super) const SKILL: u32 = 0x7dd0_0002;
pub(super) const IDLE: u32 = 0x7dd0_0003;
pub(super) const KILL_ALLY: u32 = 0x7dd0_0004;
pub(super) const CAP: u32 = 0x7dd0_0005;
pub(super) const READ: u32 = 0x7dd0_0006;
pub(super) const ULTIMATE: u32 = 0x7dd0_0007;
pub(super) const KILL_ENEMY: u32 = 0x7dd0_0008;
pub(super) const DISPEL: u32 = 0x7dd0_0009;
pub(super) const QUEUED: u32 = 0x7dd0_000a;
pub(super) const LINKED_ACTION: u32 = 0x7dd0_000b;
const OBSERVATION: u32 = 0x7dd4_0001;

pub(super) fn id<I: TryFrom<u32>>(raw: u32) -> I
where
    I::Error: core::fmt::Debug,
{
    I::try_from(raw).unwrap()
}
#[derive(Clone, Copy, Default)]
pub(super) struct Probe {
    pub(super) defeated: Option<u8>,
    pub(super) absent: Option<u8>,
    pub(super) sole_owner: bool,
    pub(super) queued_kind: Option<AbilityKind>,
    pub(super) linked_kind: Option<LinkedEntityKind>,
    pub(super) linked_presence: Option<PresenceState>,
}
pub(super) fn scenario(source: &DivergentUniverseAssembledBattle, probe: Probe) -> Battle {
    let mut builder = CombatCatalogBuilder::from_catalog(source.combat_catalog(), [0xe8; 32]);
    let owner = id(0x7dd1_0000);
    builder.add_selector(
        SelectorDefinition::new(owner).with_rule_units(
            RuleUnitSelector::new(
                RuleSelectorOrigin::Owner,
                RuleSelectorSide::Same,
                RuleLifePredicate::Any,
                RulePresencePredicate::Present,
                RuleSelectorReference::CurrentState,
                RuleSelectorOrdering::Formation,
                0,
                1,
                RuleEmptyPoolPolicy::NoOp,
                RuleSelectorChoice::First,
                None,
                false,
            )
            .unwrap(),
        ),
    );
    let mut abilities = Vec::new();
    for (raw, kind, relation, hits) in [
        (BASIC, AbilityKind::Basic, TargetRelation::Opposing, 3),
        (SKILL, AbilityKind::Skill, TargetRelation::SelfUnit, 1),
        (IDLE, AbilityKind::Basic, TargetRelation::Opposing, 1),
        (KILL_ALLY, AbilityKind::Ultimate, TargetRelation::Allied, 1),
        (CAP, AbilityKind::Ultimate, TargetRelation::SelfUnit, 1),
        (READ, AbilityKind::Skill, TargetRelation::SelfUnit, 1),
        (ULTIMATE, AbilityKind::Ultimate, TargetRelation::SelfUnit, 1),
        (
            KILL_ENEMY,
            AbilityKind::Ultimate,
            TargetRelation::Opposing,
            1,
        ),
        (DISPEL, AbilityKind::Ultimate, TargetRelation::Allied, 1),
        (
            QUEUED,
            probe.queued_kind.unwrap_or(AbilityKind::FollowUp),
            TargetRelation::Allied,
            1,
        ),
        (
            LINKED_ACTION,
            match probe.linked_kind {
                Some(LinkedEntityKind::Summon) => AbilityKind::Summon,
                Some(LinkedEntityKind::Memosprite) => AbilityKind::Memosprite,
                _ => AbilityKind::Skill,
            },
            if probe.linked_kind == Some(LinkedEntityKind::SharedActor) {
                TargetRelation::Allied
            } else {
                TargetRelation::SelfUnit
            },
            1,
        ),
    ] {
        let selector = id(raw + 0x10000);
        let program = id(raw + 0x20000);
        builder.add_selector(
            SelectorDefinition::new(selector).with_unit_targets(
                UnitTargetSelector::new(relation, TargetPattern::Single).unwrap(),
            ),
        );
        let mut definition = ProgramDefinition::new(program, vec![], vec![owner], vec![], vec![]);
        if raw == READ {
            definition = definition.with_steps(vec![ProgramStep::Operation(
                RuleOperationTemplate::ModifyResource {
                    selector: owner,
                    resource: RuleResourceKind::Team("probe.elation".into()),
                    update: ResourceUpdateKind::Set,
                    amount: ValueExpr::Multiply {
                        lhs: Box::new(ValueExpr::QueryStat {
                            subject: StatQuerySubject::Owner,
                            stat: StatKind::Elation,
                            purpose: FormulaPurpose::Stat,
                        }),
                        rhs: Box::new(ValueExpr::Literal(RuleValue::Scalar(
                            Scalar::checked_from_integer(100).unwrap(),
                        ))),
                        rounding: Rounding::Floor,
                    },
                    scales_with_regeneration: false,
                    rounding: Rounding::Floor,
                },
            )]);
        }
        if raw == ULTIMATE && probe.queued_kind.is_some() {
            definition = definition.with_steps(vec![ProgramStep::Operation(
                RuleOperationTemplate::QueueAction {
                    actor_selector: owner,
                    target_selector: owner,
                    ability: id(QUEUED),
                    priority: ReactionPriority::new(0),
                    forced_use: false,
                    boundary: ReactionBoundary::AfterAction,
                    owner: RuleActionOwner::Actor,
                    payment: None,
                },
            )]);
        }
        if raw == ULTIMATE && probe.linked_kind.is_some() {
            definition = definition.with_steps(vec![ProgramStep::Operation(
                RuleOperationTemplate::Summon {
                    owner_selector: owner,
                    unit_definition: id(0x7dd8_0001),
                },
            )]);
        }
        builder.add_program(definition);
        let operations = match raw {
            KILL_ALLY => vec![HitOperationDefinition::Damage(
                OrdinaryDamageDefinition::new(
                    Scalar::checked_from_integer(100_000).unwrap(),
                    OrdinaryDamageMultipliers::new([Ratio::ONE; 9]).unwrap(),
                )
                .unwrap(),
            )],
            CAP => vec![HitOperationDefinition::ModifyTeamResource(
                TeamResourceChangeDefinition::new(PUNCHLINE, TeamResourceChange::Set(9999)),
            )],
            KILL_ENEMY => vec![HitOperationDefinition::Damage(
                OrdinaryDamageDefinition::new(
                    Scalar::checked_from_integer(2_000_000).unwrap(),
                    OrdinaryDamageMultipliers::new([Ratio::ONE; 9]).unwrap(),
                )
                .unwrap(),
            )],
            DISPEL => vec![HitOperationDefinition::RemoveEffects(
                EffectRemovalDefinition::new(DispelCategory::DispellableBuff, None, 4).unwrap(),
            )],
            _ => vec![],
        };
        let mut action = AbilityActionDefinition::new(
            kind,
            hits,
            TargetInvalidationPolicy::CancelRemainingForTarget,
            ActionResourcePolicy::new(
                0,
                0,
                if kind == AbilityKind::Ultimate {
                    Energy::from_scaled(1_000_000).unwrap()
                } else {
                    Energy::ZERO
                },
                Energy::ZERO,
            ),
        )
        .unwrap()
        .with_hits(
            (0..hits)
                .map(|_| {
                    ActionHitDefinition::new(operations.clone()).with_profile(
                        HitTargetGroup::Selected,
                        Ratio::ONE,
                        Ratio::ONE,
                        HitCritPolicy::Never,
                    )
                })
                .collect(),
        )
        .unwrap();
        if raw == SKILL || raw == READ {
            action = action.with_tags(&[AbilityTag::Skill]);
        }
        if raw == LINKED_ACTION && probe.linked_kind == Some(LinkedEntityKind::SharedActor) {
            action = action.with_tags(&[AbilityTag::Skill, AbilityTag::ElationSkill]);
        }
        let mut programs =
            vec![AbilityProgramBinding::new(1, AbilityProgramTiming::Entry, program).unwrap()];
        if raw == ULTIMATE && probe.linked_kind == Some(LinkedEntityKind::SharedActor) {
            programs.push(
                AbilityProgramBinding::new(2, AbilityProgramTiming::AfterHits, id(0x7dd9_0001))
                    .unwrap(),
            );
        }
        builder.add_ability(
            AbilityDefinition::new(id(raw), program, selector, vec![])
                .with_action(action)
                .with_programs(programs),
        );
        abilities.push(id(raw));
    }
    abilities.sort_unstable();
    if let Some(kind) = probe.linked_kind {
        let linked_form = id(0x7dd8_0001);
        builder.add_unit(UnitDefinition::new(
            linked_form,
            vec![id(LINKED_ACTION)],
            vec![],
        ));
        let original = source
            .battle_spec()
            .participants()
            .iter()
            .find(|p| p.side() == TeamSide::Player && p.formation().get() == 1)
            .unwrap()
            .combatant();
        // Deliberately retain the owner's bundle: linked actors still cannot
        // masquerade as original owners or Elation recipients.
        let linked_spec = ResolvedCombatantSpec::new(
            linked_form,
            original.level(),
            Hp::new(10_000).unwrap(),
            Speed::from_scaled(500_000_000).unwrap(),
            ResolvedDefinitionBindings::new(
                vec![id(LINKED_ACTION)],
                original.rule_bundles().to_vec(),
                original.modifiers().to_vec(),
            )
            .unwrap(),
            CombatantSpecDigest::new([0xec; 32]).unwrap(),
        )
        .unwrap()
        .with_sources(original.sources().to_vec())
        .unwrap()
        .with_modifier_bindings(original.modifier_bindings().to_vec())
        .unwrap();
        let linked = LinkedUnitDefinition::new(
            linked_spec,
            id(0x7dda_0001),
            FormationIndex::new(4).unwrap(),
            kind,
            probe.linked_presence.unwrap_or(PresenceState::Linked),
            if kind == LinkedEntityKind::SharedActor {
                None
            } else {
                Some(id(LINKED_ACTION))
            },
            ActionGauge::from_scaled(100_000_000).unwrap(),
            OwnerLinkPolicy::Depart,
            OwnerLinkPolicy::Depart,
            WaveLinkPolicy::Depart,
        )
        .unwrap();
        builder.add_linked_unit(LinkedUnitCatalogDefinition::new(linked_form, linked).unwrap());
        if kind == LinkedEntityKind::SharedActor {
            let selector = id(0x7ddb_0001);
            builder.add_selector(
                SelectorDefinition::new(selector).with_rule_units(
                    RuleUnitSelector::new(
                        RuleSelectorOrigin::Team,
                        RuleSelectorSide::Same,
                        RuleLifePredicate::Alive,
                        RulePresencePredicate::Any,
                        RuleSelectorReference::CurrentState,
                        RuleSelectorOrdering::StableId,
                        1,
                        1,
                        RuleEmptyPoolPolicy::Fault,
                        RuleSelectorChoice::First,
                        None,
                        false,
                    )
                    .unwrap()
                    .with_predicates(vec![RuleSelectorPredicate::UnitForm(linked_form)]),
                ),
            );
            builder.add_program(
                ProgramDefinition::new(
                    id(0x7dd9_0001),
                    vec![],
                    vec![owner, selector],
                    vec![],
                    vec![],
                )
                .with_steps(vec![ProgramStep::Operation(
                    RuleOperationTemplate::QueueAction {
                        actor_selector: selector,
                        target_selector: owner,
                        ability: id(LINKED_ACTION),
                        priority: ReactionPriority::new(0),
                        forced_use: true,
                        boundary: ReactionBoundary::AfterAction,
                        owner: RuleActionOwner::Actor,
                        payment: Some(RuleActionPaymentPolicy::Suppressed),
                    },
                )]),
            );
        }
    }
    let mut participants = Vec::new();
    for player in source
        .battle_spec()
        .participants()
        .iter()
        .filter(|p| p.side() == TeamSide::Player)
    {
        let base = player.combatant();
        let formation = player.formation().get();
        let defeated = probe.defeated == Some(formation) || (probe.sole_owner && formation >= 2);
        let spec = ResolvedCombatantSpec::new(
            base.form(),
            base.level(),
            Hp::new(10_000).unwrap(),
            Speed::from_scaled(100_000_000).unwrap(),
            ResolvedDefinitionBindings::new(
                abilities.clone(),
                base.rule_bundles().to_vec(),
                base.modifiers().to_vec(),
            )
            .unwrap(),
            CombatantSpecDigest::new([formation + 1; 32]).unwrap(),
        )
        .unwrap()
        .with_sources(base.sources().to_vec())
        .unwrap()
        .with_modifier_bindings(base.modifier_bindings().to_vec())
        .unwrap()
        .with_energy(
            Energy::from_scaled(100_000_000).unwrap(),
            Energy::from_scaled(100_000_000).unwrap(),
        )
        .unwrap();
        participants.push(
            ParticipantSpec::new(player.side(), player.formation(), player.source(), spec)
                .with_initial_state(
                    ParticipantInitialState::new(
                        Hp::new(if defeated { 0 } else { 10_000 }).unwrap(),
                        Hp::new(10_000).unwrap(),
                        Energy::from_scaled(100_000_000).unwrap(),
                        Energy::from_scaled(100_000_000).unwrap(),
                        if defeated {
                            LifeState::Defeated
                        } else {
                            LifeState::Alive
                        },
                        if probe.absent == Some(formation) {
                            PresenceState::Linked
                        } else {
                            PresenceState::Present
                        },
                    )
                    .unwrap(),
                )
                .unwrap(),
        );
    }
    let enemy_form = id(0x7dd5_0001);
    builder.add_unit(UnitDefinition::new(enemy_form, vec![id(IDLE)], vec![]));
    let mut waves = Vec::new();
    for wave in 1_u16..=2 {
        let enemy = id(0x7dd6_0000 + u32::from(wave));
        builder.add_enemy(EnemyDefinition::new(enemy, enemy_form, vec![id(IDLE)]));
        let enemy_spec = ResolvedCombatantSpec::new(
            enemy_form,
            participants[0].combatant().level(),
            Hp::new(1_000_000).unwrap(),
            Speed::from_scaled(1_000_000).unwrap(),
            ResolvedDefinitionBindings::new(vec![id(IDLE)], vec![], vec![]).unwrap(),
            CombatantSpecDigest::new([0xe8; 32]).unwrap(),
        )
        .unwrap();
        participants.push(
            ParticipantSpec::new(
                TeamSide::Enemy,
                FormationIndex::new(0).unwrap(),
                ParticipantSource::EncounterEnemy(enemy),
                enemy_spec,
            )
            .with_wave(wave)
            .unwrap(),
        );
        waves.push(vec![enemy]);
    }
    let encounter = id(0x7dd7_0001);
    builder.add_encounter(
        EncounterDefinition::new(encounter, vec![], vec![])
            .with_waves(waves)
            .unwrap(),
    );
    let original = source.battle_spec().resources(TeamSide::Player);
    let mut keyed = original.keyed().to_vec();
    keyed.push(
        KeyedTeamResourceSpec::new(id(OBSERVATION), 0, 1000, TeamResourceWavePolicy::Persist)
            .unwrap()
            .with_stable_key("probe.elation")
            .unwrap(),
    );
    let resources = TeamResourceSpec::new(original.skill_points(), original.maximum_skill_points())
        .unwrap()
        .with_keyed(keyed)
        .unwrap();
    let spec = BattleSpec::new(
        AssemblyDigest::new([0xe8; 32]).unwrap(),
        encounter,
        participants,
        resources,
        TeamResourceSpec::new(0, 0).unwrap(),
        ConcedePolicy::Allowed,
    )
    .unwrap();
    let mut battle =
        Battle::create(builder.build().unwrap(), spec, BattleSeed::new([0xe8; 32])).unwrap();
    let command = Command::StartBattle {
        decision: battle.decision().unwrap().id(),
    };
    accept(&mut battle, command);
    battle
}
pub(super) fn balance(battle: &Battle) -> u16 {
    battle
        .view()
        .team(TeamSide::Player)
        .keyed_resource(PUNCHLINE)
        .map_or(0, |value| value.0)
}
pub(super) fn observed(battle: &Battle) -> u16 {
    battle
        .view()
        .team(TeamSide::Player)
        .keyed_resource(id::<SourceDefinitionId>(OBSERVATION))
        .unwrap()
        .0
}
pub(super) fn accept(battle: &mut Battle, command: Command) -> Vec<BattleEvent> {
    let result = battle.apply(command).unwrap();
    assert!(result.fault().is_none(), "{:?}", result.fault());
    result.events().to_vec()
}
pub(super) fn idle(battle: &mut Battle) -> Vec<BattleEvent> {
    let command = if let Some(decision) = battle.decision() {
        decision
            .legal_commands()
            .iter()
            .find(|command| {
                matches!(command,
            Command::UseAbility {ability, ..} if ability.get() == IDLE)
            })
            .unwrap()
            .clone()
    } else {
        Command::Advance {
            boundary: battle.view().action_boundary().unwrap().id(),
        }
    };
    accept(battle, command)
}
pub(super) fn cast(
    battle: &mut Battle,
    formation: u8,
    raw: u32,
    target: Option<UnitId>,
) -> Vec<BattleEvent> {
    let actor = battle
        .view()
        .units_by_id()
        .find(|unit| unit.side() == TeamSide::Player && unit.formation().get() == formation)
        .unwrap()
        .id();
    for _ in 0..256 {
        if let Some(command) = battle.decision().and_then(|decision| decision.legal_commands().iter()
            .find(|command| matches!(command, Command::UseAbility {actor: offered, ability, primary_target, ..}
                if *offered == actor && ability.get() == raw && *primary_target == target)).cloned()) {
            return accept(battle, command);
        }
        if battle.decision().is_none()
            && battle
                .available_ultimates()
                .iter()
                .any(|option| option.actor() == actor && option.ability().get() == raw)
        {
            let boundary = battle.view().action_boundary().unwrap().id();
            accept(
                battle,
                Command::RequestUltimate {
                    boundary,
                    actor,
                    ability: id(raw),
                },
            );
            let command = battle.decision().unwrap().legal_commands().iter().find(|command|
                matches!(command, Command::CommitPreparedAction {primary_target, ..} if *primary_target == target)).unwrap().clone();
            return accept(battle, command);
        }
        idle(battle);
    }
    panic!("controlled action not offered: {formation}/{raw}");
}
