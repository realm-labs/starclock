//! Controlled actions retain production Curio rules and original mapped forms.
use crate::divergent_universe::DivergentUniverseAssembledBattle;
use starclock_combat::{
    ActionGauge, AssemblyDigest, Battle, BattleEvent, BattleSeed, BattleSpec, CombatantSpecDigest,
    Command, ConcedePolicy, DispelCategory, EffectRemovalDefinition, Energy, FormationIndex, Hp,
    LifeState, LinkedEntityKind, LinkedUnitCatalogDefinition, LinkedUnitDefinition,
    OwnerLinkPolicy, ParticipantInitialState, ParticipantSource, ParticipantSpec, PresenceState,
    Ratio, RawToughness, ResolvedBuildBonuses, ResolvedCombatantSpec, ResolvedDefinitionBindings,
    ResolvedModifierBinding, Rounding, Scalar, Speed, StatValue, TeamResourceSpec, TeamSide,
    ToughnessLayerSpec, UnitId, WaveLinkPolicy,
    catalog::{
        action::{
            AbilityActionDefinition, AbilityKind, AbilityProgramBinding, AbilityProgramTiming,
            AbilityTag, ActionHitDefinition, ActionResourcePolicy, HitCritPolicy,
            HitOperationDefinition, HitTargetGroup, OrdinaryDamageDefinition,
            OrdinaryDamageMultipliers, ReactionBoundary, TargetInvalidationPolicy, TargetPattern,
            TargetRelation, UnitTargetSelector,
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
    formula::toughness::EnemyRank,
    modifier::model::{
        FormulaPurpose, FormulaStage, ModifierAggregation, ModifierDefinition,
        ModifierStackingGroup, SnapshotPolicy, StatKind,
    },
    rule::model::{
        ProgramStep, ReactionPriority, ResourceMaximumUpdateKind, ResourceUpdateKind,
        RuleActionOwner, RuleActionPaymentPolicy, RuleEffectChancePolicy, RuleOperationTemplate,
        RuleResourceKind, RuleSource, RuleValue, SourceClass, ValueExpr,
    },
};

pub(super) const STACKS: u32 = 0x7ed6_0001;
pub(super) const ENTANGLE: u32 = 0x7ed7_0001;
pub(super) const BASIC: u32 = 0x7de0_0001;
pub(super) const GAIN: u32 = 0x7de0_0002;
pub(super) const SKILL: u32 = 0x7de0_0003;
pub(super) const IDLE: u32 = 0x7de0_0004;
pub(super) const CAP: u32 = 0x7de0_0005;
pub(super) const SPEND: u32 = 0x7de0_0006;
pub(super) const CLEANSE: u32 = 0x7de0_0007;
pub(super) const KILL_ALLY: u32 = 0x7de0_0008;
pub(super) const KILL_ENEMY: u32 = 0x7de0_0009;
pub(super) const SEED_STACKS: u32 = 0x7de0_000a;
pub(super) const ULTIMATE: u32 = 0x7de0_000b;
pub(super) const LINKED_ACTION: u32 = 0x7de0_000c;

#[derive(Clone, Copy)]
pub(super) struct Probe {
    pub(super) initial_sp: u16,
    pub(super) gain: u16,
    pub(super) hits: u16,
    pub(super) all: bool,
    pub(super) hit_rate: i64,
    pub(super) resistance: i64,
    pub(super) control_resistance: i64,
    pub(super) critical_bonus: i64,
    pub(super) defeated: Option<u8>,
    pub(super) absent: Option<u8>,
    pub(super) enemy_hp: i64,
    pub(super) seed: u8,
    pub(super) linked_kind: Option<LinkedEntityKind>,
    pub(super) linked_presence: PresenceState,
}
impl Default for Probe {
    fn default() -> Self {
        Self {
            initial_sp: 0,
            gain: 1,
            hits: 3,
            all: false,
            hit_rate: 1_000_000,
            resistance: 0,
            control_resistance: 0,
            critical_bonus: -50_000,
            defeated: None,
            absent: None,
            enemy_hp: 1_000_000,
            seed: 0xe7,
            linked_kind: None,
            linked_presence: PresenceState::Linked,
        }
    }
}
pub(super) fn id<I: TryFrom<u32>>(raw: u32) -> I
where
    I::Error: core::fmt::Debug,
{
    I::try_from(raw).unwrap()
}
fn number(value: i64) -> ValueExpr {
    ValueExpr::Literal(RuleValue::Scalar(
        Scalar::checked_from_integer(value).unwrap(),
    ))
}
fn damage(amount: i64) -> HitOperationDefinition {
    HitOperationDefinition::Damage(
        OrdinaryDamageDefinition::new(
            Scalar::checked_from_integer(amount).unwrap(),
            OrdinaryDamageMultipliers::new([Ratio::ONE; 9]).unwrap(),
        )
        .unwrap(),
    )
}

pub(super) fn scenario(source: &DivergentUniverseAssembledBattle, probe: Probe) -> Battle {
    let mut builder = CombatCatalogBuilder::from_catalog(source.combat_catalog(), [0xe7; 32]);
    let owner = id(0x7de1_0001);
    let recipient = id(0x7de1_0002);
    for (selector, origin) in [
        (owner, RuleSelectorOrigin::Owner),
        (recipient, RuleSelectorOrigin::PrimaryTarget),
    ] {
        builder.add_selector(
            SelectorDefinition::new(selector).with_rule_units(
                RuleUnitSelector::new(
                    origin,
                    RuleSelectorSide::Same,
                    RuleLifePredicate::Alive,
                    RulePresencePredicate::Present,
                    RuleSelectorReference::CurrentState,
                    RuleSelectorOrdering::Formation,
                    0,
                    1,
                    RuleEmptyPoolPolicy::NoOp,
                    if origin == RuleSelectorOrigin::Owner {
                        RuleSelectorChoice::First
                    } else {
                        RuleSelectorChoice::All
                    },
                    None,
                    false,
                )
                .unwrap(),
            ),
        );
    }
    let mut abilities = Vec::new();
    builder.add_unit(UnitDefinition::new(
        id(0x7de7_0001),
        vec![id(LINKED_ACTION)],
        vec![],
    ));
    let linked_selector = id(0x7de1_0003);
    builder.add_selector(
        SelectorDefinition::new(linked_selector).with_rule_units(
            RuleUnitSelector::new(
                RuleSelectorOrigin::Team,
                RuleSelectorSide::Same,
                RuleLifePredicate::Alive,
                RulePresencePredicate::Any,
                RuleSelectorReference::CurrentState,
                RuleSelectorOrdering::Formation,
                0,
                1,
                RuleEmptyPoolPolicy::NoOp,
                RuleSelectorChoice::First,
                None,
                false,
            )
            .unwrap()
            .with_predicates(vec![RuleSelectorPredicate::UnitForm(id(0x7de7_0001))]),
        ),
    );
    for raw in BASIC..=LINKED_ACTION {
        let ability = id(raw);
        let selector = id(raw + 0x20000);
        let program = id(raw + 0x30000);
        let allied = matches!(raw, KILL_ALLY | SEED_STACKS) || raw == LINKED_ACTION;
        let self_target =
            matches!(raw, GAIN | CAP | SPEND) || (raw == ULTIMATE && probe.linked_kind.is_some());
        builder.add_selector(
            SelectorDefinition::new(selector).with_unit_targets(
                UnitTargetSelector::new(
                    if allied {
                        TargetRelation::Allied
                    } else if self_target {
                        TargetRelation::SelfUnit
                    } else {
                        TargetRelation::Opposing
                    },
                    if probe.all && matches!(raw, BASIC | SKILL) {
                        TargetPattern::All
                    } else {
                        TargetPattern::Single
                    },
                )
                .unwrap(),
            ),
        );
        let steps = match raw {
            SPEND => vec![ProgramStep::Operation(
                RuleOperationTemplate::ModifyResource {
                    selector: owner,
                    resource: RuleResourceKind::SkillPoints,
                    update: ResourceUpdateKind::Set,
                    amount: number(0),
                    scales_with_regeneration: false,
                    rounding: Rounding::Floor,
                },
            )],
            CAP => vec![ProgramStep::Operation(
                RuleOperationTemplate::ModifySkillPointMaximum {
                    selector: owner,
                    update: ResourceMaximumUpdateKind::Add,
                    amount: number(2),
                },
            )],
            SEED_STACKS => vec![ProgramStep::Operation(RuleOperationTemplate::ApplyEffect {
                selector: recipient,
                effect: id(STACKS),
                stacks: ValueExpr::Literal(RuleValue::Integer(65534)),
                chance: RuleEffectChancePolicy::Guaranteed,
                base_chance: None,
                rng_purpose: None,
            })],
            ULTIMATE if probe.linked_kind.is_some() => {
                vec![ProgramStep::Operation(RuleOperationTemplate::Summon {
                    owner_selector: owner,
                    unit_definition: id(0x7de7_0001),
                })]
            }
            _ => vec![],
        };
        builder.add_program(
            ProgramDefinition::new(
                program,
                vec![],
                vec![owner, recipient, linked_selector],
                if raw == SEED_STACKS {
                    vec![id(STACKS)]
                } else {
                    vec![]
                },
                vec![],
            )
            .with_steps(steps),
        );
        let operations = match raw {
            BASIC | SKILL | ULTIMATE | LINKED_ACTION => vec![damage(10)],
            KILL_ALLY | KILL_ENEMY => vec![damage(2_000_000)],
            CLEANSE => vec![HitOperationDefinition::RemoveEffects(
                EffectRemovalDefinition::new(DispelCategory::CleanseableControl, None, 16).unwrap(),
            )],
            _ => vec![],
        };
        let hits = if matches!(raw, BASIC | SKILL) {
            probe.hits
        } else {
            1
        };
        let kind = if raw == LINKED_ACTION {
            match probe.linked_kind {
                Some(LinkedEntityKind::Summon) => AbilityKind::Summon,
                Some(LinkedEntityKind::Memosprite) => AbilityKind::Memosprite,
                _ => AbilityKind::Skill,
            }
        } else if raw == ULTIMATE {
            AbilityKind::Ultimate
        } else if raw == BASIC || raw == IDLE {
            AbilityKind::Basic
        } else {
            AbilityKind::Skill
        };
        let action = AbilityActionDefinition::new(
            kind,
            hits,
            TargetInvalidationPolicy::CancelRemainingForTarget,
            ActionResourcePolicy::new(
                0,
                if matches!(raw, BASIC | GAIN | LINKED_ACTION) {
                    probe.gain
                } else {
                    0
                },
                if raw == ULTIMATE {
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
        let action = if matches!(raw, BASIC | SKILL | ULTIMATE | LINKED_ACTION) {
            action
        } else {
            action.with_tags(&[if kind == AbilityKind::Basic {
                AbilityTag::Basic
            } else {
                AbilityTag::Skill
            }])
        };
        let action =
            if raw == LINKED_ACTION && probe.linked_kind == Some(LinkedEntityKind::SharedActor) {
                action.with_tags(&[AbilityTag::Skill, AbilityTag::Attack, AbilityTag::Assist])
            } else {
                action
            };
        let mut programs =
            vec![AbilityProgramBinding::new(1, AbilityProgramTiming::Entry, program).unwrap()];
        if raw == ULTIMATE && probe.linked_kind == Some(LinkedEntityKind::SharedActor) {
            let queue = id(0x7de9_0001);
            builder.add_program(
                ProgramDefinition::new(queue, vec![], vec![owner, linked_selector], vec![], vec![])
                    .with_steps(vec![ProgramStep::Operation(
                        RuleOperationTemplate::QueueAction {
                            actor_selector: linked_selector,
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
            programs.push(
                AbilityProgramBinding::new(2, AbilityProgramTiming::AfterHits, queue).unwrap(),
            );
        }
        builder.add_ability(
            AbilityDefinition::new(ability, program, selector, vec![])
                .with_action(action)
                .with_programs(programs),
        );
        abilities.push(ability);
    }
    if let Some(kind) = probe.linked_kind {
        let form = id(0x7de7_0001);
        let original = source
            .battle_spec()
            .participants()
            .iter()
            .find(|p| p.side() == TeamSide::Player && p.formation().get() == 0)
            .unwrap()
            .combatant();
        // Inherit the entire Quantum owner's binding deliberately: ownership
        // predicates, not absent rules, must prevent a false Curio trigger.
        let spec = ResolvedCombatantSpec::new(
            form,
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
            spec,
            id(0x7de8_0001),
            FormationIndex::new(4).unwrap(),
            kind,
            probe.linked_presence,
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
        builder.add_linked_unit(LinkedUnitCatalogDefinition::new(form, linked).unwrap());
    }
    let mut participants = Vec::new();
    for original in source
        .battle_spec()
        .participants()
        .iter()
        .filter(|p| p.side() == TeamSide::Player)
    {
        let base = original.combatant();
        let formation = original.formation().get();
        let dead = probe.defeated == Some(formation);
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
        .with_base_attack_defense(
            StatValue::from_scaled(200_000_000).unwrap(),
            StatValue::from_scaled(0).unwrap(),
        )
        .with_base_effect_stats(Scalar::from_scaled(probe.hit_rate), Scalar::ZERO)
        .with_build_bonuses(ResolvedBuildBonuses::new(
            Scalar::from_scaled(probe.critical_bonus),
            Scalar::ZERO,
            Scalar::ZERO,
            Scalar::ZERO,
            Scalar::ZERO,
            [Scalar::ZERO; 7],
        ))
        .with_energy(
            Energy::from_scaled(100_000_000).unwrap(),
            Energy::from_scaled(100_000_000).unwrap(),
        )
        .unwrap();
        participants.push(
            ParticipantSpec::new(
                original.side(),
                original.formation(),
                original.source(),
                spec,
            )
            .with_initial_state(
                ParticipantInitialState::new(
                    Hp::new(if dead { 0 } else { 10_000 }).unwrap(),
                    Hp::new(10_000).unwrap(),
                    Energy::from_scaled(100_000_000).unwrap(),
                    Energy::from_scaled(100_000_000).unwrap(),
                    if dead {
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
    let enemy_form = id(0x7de4_0001);
    builder.add_modifier_group(ModifierStackingGroup {
        id: id(0x7dea_0001),
        aggregation: ModifierAggregation::Sum,
        comparator: None,
    });
    builder.add_modifier(ModifierDefinition {
        id: id(0x7deb_0001),
        stat: StatKind::ControlResistance,
        stage: FormulaStage::Flat,
        purpose: FormulaPurpose::Stat,
        value: ValueExpr::Literal(RuleValue::Scalar(Scalar::from_scaled(
            probe.control_resistance,
        ))),
        stacking_group: id(0x7dea_0001),
        priority: 0,
        floor: None,
        cap: None,
        cap_stage: FormulaStage::Flat,
        snapshot: SnapshotPolicy::Dynamic,
        source_stack_slot: None,
        filters: Box::new([]),
    });
    builder.add_unit(UnitDefinition::new(enemy_form, vec![id(IDLE)], vec![]));
    let mut waves = Vec::new();
    for wave in 1_u16..=2 {
        let mut enemies = Vec::new();
        for formation in 0..3_u8 {
            let enemy = id(0x7de5_0000 + u32::from(wave) * 4 + u32::from(formation));
            builder.add_enemy(EnemyDefinition::new(enemy, enemy_form, vec![id(IDLE)]));
            let spec = ResolvedCombatantSpec::new(
                enemy_form,
                participants[0].combatant().level(),
                Hp::new(probe.enemy_hp).unwrap(),
                Speed::from_scaled(100_000_000).unwrap(),
                ResolvedDefinitionBindings::new(vec![id(IDLE)], vec![], vec![id(0x7deb_0001)])
                    .unwrap(),
                CombatantSpecDigest::new([0xe7; 32]).unwrap(),
            )
            .unwrap()
            .with_sources(vec![RuleSource::new(
                id(0x7dec_0001),
                SourceClass::Synthetic,
                vec![],
                [0xe7; 32],
            )])
            .unwrap()
            .with_modifier_bindings(vec![ResolvedModifierBinding::new(
                id(0x7deb_0001),
                id(0x7dec_0001),
            )])
            .unwrap()
            .with_base_effect_stats(Scalar::ZERO, Scalar::from_scaled(probe.resistance))
            .with_toughness(
                EnemyRank::Normal,
                vec![],
                vec![ToughnessLayerSpec::ordinary(1, RawToughness::new(60).unwrap()).unwrap()],
            )
            .unwrap();
            participants.push(
                ParticipantSpec::new(
                    TeamSide::Enemy,
                    FormationIndex::new(formation).unwrap(),
                    ParticipantSource::EncounterEnemy(enemy),
                    spec,
                )
                .with_wave(wave)
                .unwrap(),
            );
            enemies.push(enemy);
        }
        waves.push(enemies);
    }
    let encounter = id(0x7de6_0001);
    builder.add_encounter(
        EncounterDefinition::new(encounter, vec![], vec![])
            .with_waves(waves)
            .unwrap(),
    );
    let spec = BattleSpec::new(
        AssemblyDigest::new([0xe7; 32]).unwrap(),
        encounter,
        participants,
        TeamResourceSpec::new(probe.initial_sp, 5).unwrap(),
        TeamResourceSpec::new(0, 0).unwrap(),
        ConcedePolicy::Allowed,
    )
    .unwrap();
    let mut battle = Battle::create(
        builder.build().unwrap(),
        spec,
        BattleSeed::new([probe.seed; 32]),
    )
    .unwrap();
    let start = Command::StartBattle {
        decision: battle.decision().unwrap().id(),
    };
    accept(&mut battle, start);
    battle
}
pub(super) fn accept(battle: &mut Battle, command: Command) -> Vec<BattleEvent> {
    let result = battle.apply(command).unwrap();
    assert!(result.fault().is_none(), "{:?}", result.fault());
    result.events().to_vec()
}
pub(super) fn idle(battle: &mut Battle) -> Vec<BattleEvent> {
    let command = if let Some(decision) = battle.decision() {
        decision.legal_commands().iter().find(|command|
            matches!(command, Command::UseAbility { ability, .. } if ability.get() == IDLE)).unwrap().clone()
    } else {
        battle.advance_command().unwrap()
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
            .find(|command| matches!(command, Command::UseAbility { actor: offered, ability,
                primary_target, .. } if *offered == actor && ability.get() == raw && *primary_target == target)).cloned()) {
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
            let command = battle
                .decision()
                .unwrap()
                .legal_commands()
                .iter()
                .find(|command| {
                    matches!(command, Command::CommitPreparedAction { primary_target, .. }
                    if *primary_target == target)
                })
                .unwrap()
                .clone();
            return accept(battle, command);
        }
        idle(battle);
    }
    panic!("controlled command not offered: {formation}/{raw}");
}
