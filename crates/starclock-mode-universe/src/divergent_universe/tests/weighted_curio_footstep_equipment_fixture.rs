//! Equipment handoffs preserve authored bindings; controlled actions are probes.
use crate::divergent_universe::{
    DivergentUniverseAssembledBattle,
    tests::{
        weighted_curio_footstep_damage_fixture::{ABILITIES, add_consumers, toughness},
        weighted_curio_footstep_lifecycle_fixture::{RESTORE, SETUP, Setup, add_setup},
        weighted_curio_overflow_fixture::{accept, id, start},
    },
};
use starclock_combat::{
    AbilityId, AssemblyDigest, Battle, BattleEvent, BattleEventKind, BattleSeed, BattleSpec,
    CombatantSpecDigest, Command, ConcedePolicy, Energy, FormationIndex, Hp, ParticipantSource,
    ParticipantSpec, Ratio, ResolvedCombatantSpec, ResolvedDefinitionBindings, Scalar, Speed,
    TeamResourceSpec, TeamSide, UnitLevel,
    catalog::{
        action::{
            AbilityActionDefinition, AbilityKind, AbilityProgramBinding, AbilityProgramTiming,
            ActionHitDefinition, ActionResourcePolicy, HitCritPolicy, HitOperationDefinition,
            HitTargetGroup, OrdinaryDamageDefinition, OrdinaryDamageMultipliers,
            TargetInvalidationPolicy, TargetPattern, TargetRelation, UnitTargetSelector,
        },
        builder::CombatCatalogBuilder,
        definition::{
            AbilityDefinition, EncounterDefinition, EnemyDefinition, ProgramDefinition,
            SelectorDefinition, UnitDefinition,
        },
        selector::{
            RuleEmptyPoolPolicy, RuleLifePredicate, RulePresencePredicate, RuleSelectorChoice,
            RuleSelectorOrdering, RuleSelectorOrigin, RuleSelectorReference, RuleSelectorSide,
            RuleUnitSelector,
        },
    },
    rule::model::{ProgramStep, RuleOperationTemplate, RuleValue, ValueExpr},
};

pub(super) const IDLE: u32 = 0x7f60_0001;
pub(super) const LOSS: u32 = 0x7f60_0002;
pub(super) const HEAL: u32 = 0x7f60_0003;
pub(super) const DAMAGE: u32 = 0x7f60_0004;

pub(super) fn scalar(value: i64) -> ValueExpr {
    ValueExpr::Literal(RuleValue::Scalar(
        Scalar::checked_from_integer(value).unwrap(),
    ))
}

pub(super) fn probe(source: &DivergentUniverseAssembledBattle) -> Battle {
    probe_with_setup(source, None)
}

pub(super) fn probe_with_setup(
    source: &DivergentUniverseAssembledBattle,
    setup: Option<(u8, Setup)>,
) -> Battle {
    build_probe(source, setup, false, false)
}

pub(super) fn probe_with_consumers(
    source: &DivergentUniverseAssembledBattle,
    waves: bool,
) -> Battle {
    build_probe(source, None, true, waves)
}

fn build_probe(
    source: &DivergentUniverseAssembledBattle,
    setup: Option<(u8, Setup)>,
    consumers: bool,
    waves: bool,
) -> Battle {
    let mut builder = CombatCatalogBuilder::from_catalog(source.combat_catalog(), [0xb1; 32]);
    if consumers {
        add_consumers(&mut builder);
    }
    let rule_selector = id(0x7f61_0000);
    builder.add_selector(
        SelectorDefinition::new(rule_selector).with_rule_units(
            RuleUnitSelector::new(
                RuleSelectorOrigin::PrimaryTarget,
                RuleSelectorSide::Same,
                RuleLifePredicate::Alive,
                // The action's committed target pool already requires an active
                // unit; the controlled HP operation also supports Transformed.
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
            .unwrap(),
        ),
    );
    for raw in [IDLE, LOSS, HEAL, DAMAGE] {
        let selector = id(raw + 0x10000);
        let program = id(raw + 0x20000);
        builder.add_selector(
            SelectorDefinition::new(selector).with_unit_targets(
                UnitTargetSelector::new(
                    if raw == DAMAGE {
                        TargetRelation::Opposing
                    } else if raw == IDLE {
                        TargetRelation::SelfUnit
                    } else {
                        TargetRelation::Allied
                    },
                    TargetPattern::Single,
                )
                .unwrap(),
            ),
        );
        let steps = match raw {
            LOSS => vec![ProgramStep::Operation(RuleOperationTemplate::ConsumeHp {
                selector: rule_selector,
                amount: scalar(30),
                floor: scalar(1),
            })],
            HEAL => vec![ProgramStep::Operation(RuleOperationTemplate::Heal {
                selector: rule_selector,
                amount: scalar(100),
                apply_formula_modifiers: false,
            })],
            _ => vec![],
        };
        builder.add_program(
            ProgramDefinition::new(program, vec![], vec![rule_selector], vec![], vec![])
                .with_steps(steps),
        );
        let mut action = AbilityActionDefinition::new(
            if matches!(raw, LOSS | HEAL) {
                AbilityKind::Skill
            } else {
                AbilityKind::Basic
            },
            1,
            TargetInvalidationPolicy::CancelRemainingForTarget,
            ActionResourcePolicy::new(0, 0, Energy::ZERO, Energy::ZERO),
        )
        .unwrap();
        if raw == DAMAGE {
            action = action
                .with_hits(vec![
                    ActionHitDefinition::new(vec![HitOperationDefinition::Damage(
                        OrdinaryDamageDefinition::new(
                            Scalar::checked_from_integer(100).unwrap(),
                            OrdinaryDamageMultipliers::new([Ratio::ONE; 9]).unwrap(),
                        )
                        .unwrap(),
                    )])
                    .with_profile(
                        HitTargetGroup::Selected,
                        Ratio::ONE,
                        Ratio::ONE,
                        HitCritPolicy::Never,
                    ),
                ])
                .unwrap();
        }
        builder.add_ability(
            AbilityDefinition::new(id(raw), program, selector, vec![])
                .with_action(action)
                .with_programs(vec![
                    AbilityProgramBinding::new(1, AbilityProgramTiming::Entry, program).unwrap(),
                ]),
        );
    }
    let mut players = Vec::new();
    for original in source
        .battle_spec()
        .participants()
        .iter()
        .filter(|p| p.side() == TeamSide::Player)
    {
        let base = original.combatant();
        let mut abilities: Vec<AbilityId> = vec![id(IDLE), id(LOSS), id(HEAL), id(DAMAGE)];
        if consumers {
            abilities.extend(ABILITIES.map(id::<AbilityId>));
        }
        if setup.is_some_and(|(formation, _)| formation == original.formation().get()) {
            abilities.push(id(SETUP));
            abilities.push(id(RESTORE));
        }
        // Crucially retain normal assembly's bundles, sources and modifier bindings.
        // No Footstep policy constructor is invoked in this probe.
        let mut spec = ResolvedCombatantSpec::new(
            base.form(),
            base.level(),
            Hp::new(100).unwrap(),
            Speed::from_scaled(100_000_000).unwrap(),
            ResolvedDefinitionBindings::new(
                abilities,
                base.rule_bundles().to_vec(),
                base.modifiers().to_vec(),
            )
            .unwrap(),
            CombatantSpecDigest::new([original.formation().get() + 1; 32]).unwrap(),
        )
        .unwrap()
        .with_sources(base.sources().to_vec())
        .unwrap()
        .with_modifier_bindings(base.modifier_bindings().to_vec())
        .unwrap();
        if consumers {
            spec = toughness(spec);
        }
        players.push(ParticipantSpec::new(
            original.side(),
            original.formation(),
            original.source(),
            spec,
        ));
    }
    if let Some((formation, setup)) = setup {
        let owner = players
            .iter()
            .find(|player| player.formation().get() == formation)
            .unwrap();
        let effect = id(0x7f2c_0000 + (u32::from(formation) + 1) * 16);
        let effect = source.combat_catalog().effect(effect).map(|_| effect);
        add_setup(&mut builder, owner, setup, effect, source);
    }
    let form = id(0x7f64_0001);
    let enemy = id(0x7f65_0001);
    let encounter = id(0x7f66_0001);
    builder.add_unit(UnitDefinition::new(form, vec![id(IDLE)], vec![]));
    builder.add_enemy(EnemyDefinition::new(enemy, form, vec![id(IDLE)]));
    let mut enemy_spec = ResolvedCombatantSpec::new(
        form,
        UnitLevel::new(1).unwrap(),
        Hp::new(100_000).unwrap(),
        Speed::from_scaled(if consumers { 100_000_000 } else { 1_000_000 }).unwrap(),
        ResolvedDefinitionBindings::new(vec![id(IDLE)], vec![], vec![]).unwrap(),
        CombatantSpecDigest::new([0xb2; 32]).unwrap(),
    )
    .unwrap();
    if consumers {
        enemy_spec = toughness(enemy_spec);
    }
    let enemy_participant = ParticipantSpec::new(
        TeamSide::Enemy,
        FormationIndex::new(0).unwrap(),
        ParticipantSource::EncounterEnemy(enemy),
        enemy_spec,
    );
    players.push(enemy_participant.clone());
    if waves {
        players.push(enemy_participant.with_wave(2).unwrap());
        builder.add_encounter(
            EncounterDefinition::new(encounter, vec![], vec![])
                .with_waves(vec![vec![enemy], vec![enemy]])
                .unwrap(),
        );
    } else {
        builder.add_encounter(EncounterDefinition::new(encounter, vec![enemy], vec![]));
    }
    let spec = BattleSpec::new(
        AssemblyDigest::new([0xb3; 32]).unwrap(),
        encounter,
        players,
        TeamResourceSpec::new(0, 5).unwrap(),
        TeamResourceSpec::new(0, 0).unwrap(),
        ConcedePolicy::Allowed,
    )
    .unwrap();
    let mut battle =
        Battle::create(builder.build().unwrap(), spec, BattleSeed::new([0xb4; 32])).unwrap();
    start(&mut battle);
    battle
}

pub(super) fn cast(battle: &mut Battle, owner: u64, raw: u32) -> Vec<BattleEvent> {
    let target = if raw == DAMAGE {
        Some(5)
    } else if matches!(raw, SETUP | RESTORE) {
        None
    } else {
        Some(owner)
    };
    for _ in 0..64 {
        if let Some(command) = battle.decision().and_then(|decision| decision.legal_commands().iter().find(|command|
            matches!(command, Command::UseAbility {actor, ability, primary_target, ..}
                if actor.get() == owner && ability.get() == raw && primary_target.map(|unit| unit.get()) == target)).cloned()) {
            return accept(battle, command);
        }
        progress(battle);
    }
    panic!("equipment probe command not offered: {owner}/{raw}");
}

pub(super) fn progress(battle: &mut Battle) -> Vec<BattleEvent> {
    let command = battle.decision().and_then(|decision| decision.legal_commands().iter()
            .find(|command| matches!(command, Command::UseAbility {ability, ..} if ability.get() == IDLE)).cloned())
            .or_else(|| battle.advance_command()).unwrap();
    accept(battle, command)
}
pub(super) fn stacks(battle: &Battle, owner: u64) -> u16 {
    battle
        .view()
        .effects_by_id()
        .filter(|effect| {
            effect.target().get() == owner
                && (0x7f2c_0000..0x7f2d_0000).contains(&effect.definition().get())
        })
        .map(|effect| effect.stacks())
        .sum()
}
pub(super) fn damage(events: &[BattleEvent]) -> Vec<i64> {
    events
        .iter()
        .filter_map(|event| match event.kind() {
            BattleEventKind::Damage(data) => Some(data.applied.get()),
            _ => None,
        })
        .collect()
}
