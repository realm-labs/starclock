//! Generic commands over caller-bound HP-loss policy, not normal Curio admission.
use crate::divergent_universe::{
    DivergentUniverseBaselineFixture,
    tests::weighted_curio_overflow_fixture::{accept, id, start},
    weighted_curio_footstep::{HpLossPointPolicy, bind_hp_loss_point_policy},
};
use starclock_build::light_cone::CombatPath;
use starclock_combat::{
    AssemblyDigest, Battle, BattleEvent, BattleSeed, BattleSpec, CombatantSpecDigest, Command,
    ConcedePolicy, DispelCategory, DurationClock, EffectCategory, EffectRuntimeTemplate,
    EffectStackPolicy, EffectTickPhase, Energy, FormationIndex, Hp, ParticipantSource,
    ParticipantSpec, ResolvedCombatantSpec, ResolvedDefinitionBindings, Rounding, Scalar, Speed,
    TeamResourceSpec, TeamSide, UnitLevel,
    catalog::{
        action::{
            AbilityActionDefinition, AbilityKind, AbilityProgramBinding, AbilityProgramTiming,
            ActionResourcePolicy, TargetInvalidationPolicy, TargetPattern, TargetRelation,
            UnitTargetSelector,
        },
        builder::CombatCatalogBuilder,
        definition::{
            AbilityDefinition, EffectDefinition, EncounterDefinition, EnemyDefinition,
            ProgramDefinition, SelectorDefinition, UnitDefinition,
        },
        selector::{
            RuleEmptyPoolPolicy, RuleLifePredicate, RulePresencePredicate, RuleSelectorChoice,
            RuleSelectorOrdering, RuleSelectorOrigin, RuleSelectorReference, RuleSelectorSide,
            RuleUnitSelector,
        },
    },
    rule::model::{
        ProgramStep, ResourceUpdateKind, RuleEffectChancePolicy, RuleOperationTemplate,
        RuleResourceKind, RuleValue, ValueExpr,
    },
};

pub(super) struct Probe {
    pub(super) path: CombatPath,
    pub(super) hp: i64,
    pub(super) loss: i64,
    pub(super) fraction: Scalar,
    pub(super) initial_points: u16,
    pub(super) bind_buddy: bool,
    pub(super) inherit_buddy: bool,
    pub(super) identity: [u8; 32],
}
impl Default for Probe {
    fn default() -> Self {
        Self {
            path: CombatPath::Destruction,
            hp: 100,
            loss: 30,
            fraction: Scalar::from_scaled(500_000),
            initial_points: 0,
            bind_buddy: false,
            inherit_buddy: false,
            identity: [0x91; 32],
        }
    }
}
pub(super) fn scalar(amount: i64) -> ValueExpr {
    ValueExpr::Literal(RuleValue::Scalar(
        Scalar::checked_from_integer(amount).unwrap(),
    ))
}
fn op(operation: RuleOperationTemplate) -> ProgramStep {
    ProgramStep::Operation(operation)
}

pub(super) fn scenario(fixture: &DivergentUniverseBaselineFixture, probe: &Probe) -> Battle {
    let form = fixture
        .core()
        .build_catalog()
        .character_ids()
        .find(|id| {
            fixture
                .core()
                .build_catalog()
                .character(*id)
                .unwrap()
                .path()
                == probe.path
        })
        .unwrap();
    let mut builder = CombatCatalogBuilder::new([0x92; 32]);
    let target = id(1);
    builder.add_selector(SelectorDefinition::new(target).with_unit_targets(
        UnitTargetSelector::new(TargetRelation::Allied, TargetPattern::Single).unwrap(),
    ));
    builder.add_selector(
        SelectorDefinition::new(id(2)).with_rule_units(
            RuleUnitSelector::new(
                RuleSelectorOrigin::PrimaryTarget,
                RuleSelectorSide::Same,
                RuleLifePredicate::Alive,
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
    builder.add_effect(
        EffectDefinition::new(id(10), vec![], vec![]).with_runtime_template(
            EffectRuntimeTemplate::new(
                EffectCategory::Shield,
                DispelCategory::NonDispellable,
                1,
                None,
                DurationClock::Permanent,
                EffectTickPhase::None,
                EffectStackPolicy::Replace,
            )
            .unwrap(),
        ),
    );
    let consume = || {
        op(RuleOperationTemplate::ConsumeHp {
            selector: id(2),
            amount: scalar(probe.loss),
            floor: scalar(1),
        })
    };
    let heal = || {
        op(RuleOperationTemplate::Heal {
            selector: id(2),
            amount: scalar(probe.hp),
            apply_formula_modifiers: false,
        })
    };
    let programs = [
        vec![consume()],
        vec![heal()],
        vec![op(RuleOperationTemplate::TrueDamage {
            selector: id(2),
            amount: scalar(probe.loss),
        })],
        vec![
            op(RuleOperationTemplate::ApplyEffect {
                selector: id(2),
                effect: id(10),
                stacks: ValueExpr::Literal(RuleValue::Integer(1)),
                chance: RuleEffectChancePolicy::Guaranteed,
                base_chance: None,
                rng_purpose: None,
            }),
            op(RuleOperationTemplate::Shield {
                selector: id(2),
                effect: id(10),
                amount: scalar(40),
            }),
        ],
        vec![op(RuleOperationTemplate::ReduceMaximumHp {
            selector: id(2),
            amount: scalar(40),
            minimum_ratio: ValueExpr::Literal(RuleValue::Scalar(Scalar::from_scaled(100_000))),
        })],
        vec![consume()],
        vec![consume(), heal(), consume()],
        vec![op(RuleOperationTemplate::ModifyResource {
            selector: id(2),
            resource: RuleResourceKind::SkillPoints,
            update: ResourceUpdateKind::Set,
            amount: scalar(0),
            scales_with_regeneration: false,
            rounding: Rounding::Floor,
        })],
        vec![op(RuleOperationTemplate::TrueDamage {
            selector: id(2),
            amount: scalar(10_000),
        })],
        vec![],
    ];
    let abilities = (1..=10).map(id).collect::<Vec<_>>();
    for (index, steps) in programs.into_iter().enumerate() {
        let raw = u32::try_from(index).unwrap() + 1;
        builder.add_program(
            ProgramDefinition::new(id(raw), vec![], vec![id(2)], vec![id(10)], vec![])
                .with_steps(steps),
        );
        builder.add_ability(
            AbilityDefinition::new(id(raw), id(raw), target, vec![])
                .with_action(
                    AbilityActionDefinition::new(
                        AbilityKind::Skill,
                        1,
                        TargetInvalidationPolicy::CancelRemainingForTarget,
                        ActionResourcePolicy::new(0, 0, Energy::ZERO, Energy::ZERO),
                    )
                    .unwrap(),
                )
                .with_programs(vec![
                    AbilityProgramBinding::new(1, AbilityProgramTiming::Entry, id(raw)).unwrap(),
                ]),
        );
    }
    builder.add_unit(UnitDefinition::new(form, abilities.clone(), vec![]));
    let make = |formation: u8, side, source, form, hp, bindings| {
        ParticipantSpec::new(
            side,
            FormationIndex::new(formation).unwrap(),
            source,
            ResolvedCombatantSpec::new(
                form,
                UnitLevel::new(70).unwrap(),
                Hp::new(hp).unwrap(),
                Speed::from_scaled(if side == TeamSide::Player {
                    100_000_000
                } else {
                    1_000_000
                })
                .unwrap(),
                bindings,
                CombatantSpecDigest::new([formation + 1; 32]).unwrap(),
            )
            .unwrap(),
        )
    };
    let policy = HpLossPointPolicy::new(probe.fraction, probe.identity).unwrap();
    let player = make(
        0,
        TeamSide::Player,
        ParticipantSource::Player,
        form,
        probe.hp,
        ResolvedDefinitionBindings::new(abilities.clone(), vec![], vec![]).unwrap(),
    );
    let player =
        bind_hp_loss_point_policy(&mut builder, fixture.core(), &player, &policy, [0x93; 32])
            .unwrap();
    let mut buddy = make(
        1,
        TeamSide::Player,
        ParticipantSource::Player,
        form,
        probe.hp,
        ResolvedDefinitionBindings::new(abilities.clone(), vec![], vec![]).unwrap(),
    );
    if probe.bind_buddy {
        buddy =
            bind_hp_loss_point_policy(&mut builder, fixture.core(), &buddy, &policy, [0x93; 32])
                .unwrap();
    } else if probe.inherit_buddy {
        let inherited = make(
            1,
            TeamSide::Player,
            ParticipantSource::Player,
            form,
            probe.hp,
            ResolvedDefinitionBindings::new(
                abilities.clone(),
                player.combatant().rule_bundles().to_vec(),
                vec![],
            )
            .unwrap(),
        );
        buddy = ParticipantSpec::new(
            inherited.side(),
            inherited.formation(),
            inherited.source(),
            inherited
                .combatant()
                .clone()
                .with_sources(player.combatant().sources().to_vec())
                .unwrap(),
        );
    }
    let enemy_form = id(99001);
    builder.add_unit(UnitDefinition::new(enemy_form, vec![id(10)], vec![]));
    builder.add_enemy(EnemyDefinition::new(id(1), enemy_form, vec![id(10)]));
    builder.add_encounter(EncounterDefinition::new(id(1), vec![id(1)], vec![]));
    let enemy = make(
        0,
        TeamSide::Enemy,
        ParticipantSource::EncounterEnemy(id(1)),
        enemy_form,
        100_000,
        ResolvedDefinitionBindings::new(vec![id(10)], vec![], vec![]).unwrap(),
    );
    Battle::create(
        builder.build().unwrap(),
        BattleSpec::new(
            AssemblyDigest::new([0x94; 32]).unwrap(),
            id(1),
            vec![player, buddy, enemy],
            TeamResourceSpec::new(probe.initial_points, 5).unwrap(),
            TeamResourceSpec::new(0, 0).unwrap(),
            ConcedePolicy::Allowed,
        )
        .unwrap(),
        BattleSeed::new([0x95; 32]),
    )
    .unwrap()
}

pub(super) fn begin(battle: &mut Battle) {
    start(battle);
}
pub(super) fn cast(battle: &mut Battle, ability: u32, target: u64) -> Vec<BattleEvent> {
    for _ in 0..32 {
        let offered = battle.decision().map(|decision| decision.legal_commands());
        if let Some(command) = offered.and_then(|commands| commands.iter().find(|command| matches!(command,
            Command::UseAbility {actor,ability:observed,primary_target,..}
            if actor.get()==1 && observed.get()==ability && primary_target.is_some_and(|unit|unit.get()==target)))).cloned() {
            return accept(battle,command);
        }
        // Advance through another original member's real offered inert action.
        let command = offered
            .and_then(|commands| {
                commands.iter().find(|command| {
                    matches!(command,
            Command::UseAbility {ability,..} if ability.get()==10)
                })
            })
            .cloned()
            .or_else(|| battle.advance_command())
            .expect("a normal command remains offered");
        accept(battle, command);
    }
    panic!("requested command was not offered");
}
pub(super) fn remainder(battle: &Battle, owner: u64) -> Scalar {
    battle
        .view()
        .rule_instances_by_id()
        .filter(|rule| rule.owner().is_some_and(|unit| unit.get() == owner))
        .flat_map(|rule| rule.slots())
        .find_map(|(slot, value)| {
            if (0x7f25_0000..0x7f26_0000).contains(&slot.get()) {
                let RuleValue::Scalar(value) = value else {
                    panic!("scalar slot")
                };
                Some(*value)
            } else {
                None
            }
        })
        .unwrap_or(Scalar::ZERO)
}
pub(super) fn concede(battle: &mut Battle) -> Vec<BattleEvent> {
    if battle.decision().is_none() {
        let command = battle.advance_command().unwrap();
        accept(battle, command);
    }
    let command = battle
        .decision()
        .unwrap()
        .legal_commands()
        .iter()
        .find(|command| matches!(command, Command::Concede { .. }))
        .unwrap()
        .clone();
    accept(battle, command)
}
