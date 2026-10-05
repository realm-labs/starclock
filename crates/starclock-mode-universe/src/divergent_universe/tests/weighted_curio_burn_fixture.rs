//! Controlled actions retain independently constructed source-owned Burn rules.
use crate::divergent_universe::DivergentUniverseAssembledBattle;
use crate::divergent_universe::tests::weighted_curio_burn_lifecycle_fixture::{
    END_TRANSFORM, SETUP, Setup, add_setup,
};
use starclock_combat::{
    AbilityId, AssemblyDigest, Battle, BattleEvent, BattleEventKind, BattleSeed, BattleSpec,
    CombatantSpecDigest, Command, ConcedePolicy, DispelCategory, DotDefinition,
    DotDetonationDefinition, DotFamily, DurationClock, EffectApplicationDefinition, EffectCategory,
    EffectChancePolicy, EffectRuntimeDefinition, EffectStackPolicy, EffectTickPhase, Energy,
    FormationIndex, Hp, LifeState, ParticipantInitialState, ParticipantSource, ParticipantSpec,
    PresenceState, Probability, Ratio, RawToughness, ResolvedCombatantSpec,
    ResolvedDefinitionBindings, Scalar, SourceDefinitionId, Speed, StatValue, TeamResourceSpec,
    TeamSide, ToughnessLayerSpec, ToughnessReductionDefinition, UnitId, UnitLevel,
    catalog::{
        CombatCatalog,
        action::{
            AbilityActionDefinition, AbilityKind, AbilityProgramBinding, AbilityProgramTiming,
            AbilityTag, ActionHitDefinition, ActionResourcePolicy, HitCritPolicy,
            HitOperationDefinition, HitTargetGroup, OrdinaryDamageDefinition,
            OrdinaryDamageMultipliers, TargetInvalidationPolicy, TargetPattern, TargetRelation,
            UnitTargetSelector,
        },
        builder::CombatCatalogBuilder,
        definition::{
            AbilityDefinition, EffectDefinition, EncounterDefinition, EnemyDefinition,
            ProgramDefinition, SelectorDefinition, UnitDefinition,
        },
        selector::{
            RuleEmptyPoolPolicy, RuleLifePredicate, RulePresencePredicate, RuleSelectorChoice,
            RuleSelectorOrdering, RuleSelectorOrigin, RuleSelectorPredicate, RuleSelectorReference,
            RuleSelectorSide, RuleUnitSelector,
        },
    },
    formula::{
        model::{CombatElement, DamageClass},
        toughness::{BreakDamageDefinition, EnemyRank, ToughnessReductionContext},
    },
    rule::model::{ProgramStep, RuleOperationTemplate},
};
use std::sync::Arc;

pub(super) const ATTACK: u32 = 0x7dc3_0001;
pub(super) const DETONATE: u32 = 0x7dc3_0004;
pub(super) const RESTORE: u32 = 0x7dc3_0005;
pub(super) const CLEAR_WAVE: u32 = 0x7dc3_0006;
pub(super) const SEED: u32 = 0x7dc3_0002;
pub(super) const IDLE: u32 = 0x7dc3_0003;
pub(super) const BURN: u32 = 0x7dc6_0001;
pub(super) const UNCLASSIFIED: u32 = 0x7dc6_0002;
pub(super) const SHOCK: u32 = 0x7dc6_0003;

fn id<I: TryFrom<u32>>(raw: u32) -> I
where
    I::Error: core::fmt::Debug,
{
    I::try_from(raw).unwrap()
}

#[derive(Clone, Copy)]
pub(super) struct Probe {
    pub(super) formation: u8,
    pub(super) all: bool,
    pub(super) allied: bool,
    pub(super) kind: AbilityKind,
    pub(super) attack: bool,
    pub(super) hits: u16,
    pub(super) burns: bool,
    pub(super) break_burn: bool,
    pub(super) resistance: i64,
    pub(super) hit_rate: i64,
    pub(super) seed: u8,
    pub(super) enemy_hp: i64,
    pub(super) enemy_level: u8,
    pub(super) second_owner: bool,
    pub(super) full_party: bool,
    pub(super) absent_formation: Option<u8>,
    pub(super) lifecycle: Option<Setup>,
    pub(super) waves: bool,
}
impl Default for Probe {
    fn default() -> Self {
        Self {
            formation: 0,
            all: false,
            allied: false,
            kind: AbilityKind::Basic,
            attack: true,
            hits: 3,
            burns: false,
            break_burn: false,
            resistance: 0,
            hit_rate: 0,
            seed: 85,
            enemy_hp: 1_000_000,
            enemy_level: 80,
            second_owner: false,
            full_party: false,
            absent_formation: None,
            lifecycle: None,
            waves: false,
        }
    }
}

pub(super) struct Scenario {
    pub(super) battle: Battle,
    pub(super) actor: UnitId,
    pub(super) seeder: UnitId,
    pub(super) targets: Vec<UnitId>,
    pub(super) started: Vec<BattleEvent>,
    all: bool,
    allied: bool,
}

pub(super) fn scenario(assembled: &DivergentUniverseAssembledBattle, input: Probe) -> Scenario {
    let players = assembled
        .battle_spec()
        .participants()
        .iter()
        .filter(|player| player.side() == TeamSide::Player)
        .cloned()
        .collect::<Vec<_>>();
    scenario_with_players(assembled.combat_catalog(), &players, input)
}

pub(super) fn scenario_with_players(
    catalog: &Arc<CombatCatalog>,
    players: &[ParticipantSpec],
    input: Probe,
) -> Scenario {
    let mut builder = CombatCatalogBuilder::from_catalog(catalog, [0xc3; 32]);
    let form = id(0x7dc4_0001);
    for (raw, amount, family) in [
        (BURN, 100, Some(DotFamily::Burn)),
        (UNCLASSIFIED, 30, None),
        (SHOCK, 20, Some(DotFamily::Shock)),
    ] {
        let runtime = EffectRuntimeDefinition::new(
            EffectCategory::Dot,
            DispelCategory::DispellableDebuff,
            1,
            Some(8),
            DurationClock::TargetTurnEnd,
            EffectTickPhase::None,
            EffectStackPolicy::Replace,
        )
        .unwrap()
        .with_dot(DotDefinition::new(
            damage(amount).with_class(DamageClass::Dot),
            CombatElement::Fire,
            None,
        ))
        .unwrap();
        let mut definition = EffectDefinition::new(id(raw), vec![], vec![]).with_runtime(runtime);
        if let Some(family) = family {
            definition = definition.with_dot_family(family);
        }
        builder.add_effect(definition);
    }
    for (offset, raw, all) in [
        (1, ATTACK, input.all),
        (2, SEED, true),
        (3, IDLE, false),
        (4, DETONATE, false),
        (6, CLEAR_WAVE, true),
    ] {
        let program = id(0x7dc2_0000 + offset);
        let selector = id(0x7dc1_0000 + offset);
        builder.add_selector(
            SelectorDefinition::new(selector).with_unit_targets(
                UnitTargetSelector::new(
                    if raw == ATTACK && input.allied {
                        TargetRelation::Allied
                    } else {
                        TargetRelation::Opposing
                    },
                    if all {
                        TargetPattern::All
                    } else {
                        TargetPattern::Single
                    },
                )
                .unwrap(),
            ),
        );
        builder.add_program(ProgramDefinition::new(
            program,
            vec![],
            vec![selector],
            vec![],
            vec![],
        ));
        let mut operations = vec![HitOperationDefinition::Damage(damage(match raw {
            ATTACK => 2,
            CLEAR_WAVE => 2_000_000,
            _ => 0,
        }))];
        if raw == DETONATE {
            operations.push(HitOperationDefinition::DetonateDots(
                DotDetonationDefinition::new(Ratio::ONE, None).unwrap(),
            ));
        }
        if raw == SEED {
            if input.burns {
                for raw in [BURN, UNCLASSIFIED, SHOCK] {
                    operations.push(HitOperationDefinition::ApplyEffect(
                        EffectApplicationDefinition::new(
                            id(raw),
                            EffectChancePolicy::Guaranteed,
                            1,
                        )
                        .unwrap(),
                    ));
                }
            }
            if input.break_burn {
                operations.push(HitOperationDefinition::ReduceToughness(
                    ToughnessReductionDefinition {
                        reduction: ToughnessReductionContext {
                            base: RawToughness::new(100).unwrap(),
                            additive: RawToughness::new(0).unwrap(),
                            reduction_increase: Ratio::ZERO,
                            weakness_break_efficiency: Ratio::ZERO,
                            weakness_break_efficiency_cap: Ratio::ONE,
                            toughness_vulnerability: Ratio::ZERO,
                            ability_multiplier: Ratio::ONE,
                        },
                        element: CombatElement::Fire,
                        ignores_weakness: true,
                        break_effect_chance: Probability::ONE,
                        break_damage: BreakDamageDefinition {
                            attacker_level_multiplier: Scalar::checked_from_integer(100).unwrap(),
                            ability_multiplier: Ratio::ONE,
                            break_effect: Ratio::ZERO,
                            break_damage_increase: Ratio::ZERO,
                            defense_multiplier: Ratio::ONE,
                            resistance_multiplier: Ratio::ONE,
                            vulnerability_multiplier: Ratio::ONE,
                            mitigation_multiplier: Ratio::ONE,
                            unbroken_multiplier: Ratio::ONE,
                        },
                    },
                ));
            }
        }
        let hits = if raw == ATTACK { input.hits } else { 1 };
        let action = AbilityActionDefinition::new(
            if raw == ATTACK {
                input.kind
            } else {
                AbilityKind::Basic
            },
            hits,
            TargetInvalidationPolicy::CancelRemainingForTarget,
            ActionResourcePolicy::new(0, 0, Energy::ZERO, Energy::ZERO),
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
        let action = if raw == ATTACK && input.attack {
            action
        } else {
            action.with_tags(&[AbilityTag::Basic])
        };
        builder.add_ability(
            AbilityDefinition::new(id(raw), program, selector, vec![]).with_action(action),
        );
    }
    let mut abilities = vec![id(ATTACK), id(SEED), id(IDLE), id(DETONATE)];
    if input.waves {
        abilities.push(id(CLEAR_WAVE));
    }
    if let Some(formation) = input.absent_formation {
        abilities.push(id(RESTORE));
        let selector = id(0x7dc1_0005);
        let program = id(0x7dc2_0005);
        builder.add_selector(
            SelectorDefinition::new(selector)
                .with_unit_targets(
                    UnitTargetSelector::new(TargetRelation::Allied, TargetPattern::All).unwrap(),
                )
                .with_rule_units(
                    RuleUnitSelector::new(
                        RuleSelectorOrigin::Team,
                        RuleSelectorSide::Same,
                        RuleLifePredicate::Any,
                        RulePresencePredicate::Any,
                        RuleSelectorReference::CurrentState,
                        RuleSelectorOrdering::Formation,
                        0,
                        1,
                        RuleEmptyPoolPolicy::NoOp,
                        RuleSelectorChoice::All,
                        None,
                        false,
                    )
                    .unwrap()
                    .with_predicates(vec![
                        RuleSelectorPredicate::FormationRange {
                            minimum: formation,
                            maximum: formation,
                        },
                    ]),
                ),
        );
        builder.add_program(
            ProgramDefinition::new(program, vec![], vec![selector], vec![], vec![]).with_steps(
                vec![ProgramStep::Operation(
                    RuleOperationTemplate::ChangePresence {
                        selector,
                        presence: PresenceState::Present,
                    },
                )],
            ),
        );
        builder.add_ability(
            AbilityDefinition::new(id(RESTORE), id(0x7dc2_0003), id(0x7dc1_0005), vec![])
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
                    AbilityProgramBinding::new(1, AbilityProgramTiming::Entry, program).unwrap(),
                ]),
        );
    }
    abilities.sort_unstable();
    builder.add_unit(UnitDefinition::new(form, abilities.clone(), vec![]));
    let original = players
        .iter()
        .find(|p| p.side() == TeamSide::Player && p.formation().get() == input.formation)
        .unwrap()
        .combatant();
    let mut actor_abilities = vec![id(ATTACK), id(IDLE)];
    if input.lifecycle.is_some() {
        actor_abilities.extend([id::<AbilityId>(SETUP), id(END_TRANSFORM)]);
    }
    let actor = ResolvedCombatantSpec::new(
        if input.lifecycle.is_some() {
            original.form()
        } else {
            form
        },
        UnitLevel::new(80).unwrap(),
        Hp::new(10_000).unwrap(),
        Speed::from_scaled(200_000_000).unwrap(),
        ResolvedDefinitionBindings::new(
            actor_abilities,
            original.rule_bundles().to_vec(),
            original.modifiers().to_vec(),
        )
        .unwrap(),
        CombatantSpecDigest::new([0xc4; 32]).unwrap(),
    )
    .unwrap()
    .with_sources(original.sources().to_vec())
    .unwrap()
    .with_modifier_bindings(original.modifier_bindings().to_vec())
    .unwrap()
    .with_base_attack_defense(
        StatValue::from_scaled(200_000_000).unwrap(),
        StatValue::from_scaled(0).unwrap(),
    )
    .with_base_effect_stats(Scalar::from_scaled(input.hit_rate), Scalar::ZERO);
    let plain = |level, speed, hp, abilities, resistance| {
        ResolvedCombatantSpec::new(
            form,
            UnitLevel::new(level).unwrap(),
            Hp::new(hp).unwrap(),
            Speed::from_scaled(speed).unwrap(),
            ResolvedDefinitionBindings::new(abilities, vec![], vec![]).unwrap(),
            CombatantSpecDigest::new([0xc5; 32]).unwrap(),
        )
        .unwrap()
        .with_base_attack_defense(
            StatValue::from_scaled(100_000_000).unwrap(),
            StatValue::from_scaled(0).unwrap(),
        )
        .with_base_effect_stats(Scalar::ZERO, Scalar::from_scaled(resistance))
    };
    let seeder = if input.second_owner {
        let original = players
            .iter()
            .find(|p| p.side() == TeamSide::Player && p.formation().get() == 1)
            .unwrap()
            .combatant();
        ResolvedCombatantSpec::new(
            form,
            UnitLevel::new(80).unwrap(),
            Hp::new(10_000).unwrap(),
            Speed::from_scaled(300_000_000).unwrap(),
            ResolvedDefinitionBindings::new(
                abilities,
                original.rule_bundles().to_vec(),
                original.modifiers().to_vec(),
            )
            .unwrap(),
            CombatantSpecDigest::new([0xc6; 32]).unwrap(),
        )
        .unwrap()
        .with_sources(original.sources().to_vec())
        .unwrap()
        .with_modifier_bindings(original.modifier_bindings().to_vec())
        .unwrap()
        .with_base_attack_defense(
            StatValue::from_scaled(100_000_000).unwrap(),
            StatValue::from_scaled(0).unwrap(),
        )
    } else {
        plain(
            80,
            300_000_000,
            10_000,
            vec![id(SEED), id(IDLE), id(DETONATE)],
            0,
        )
    };
    let mut participants = vec![
        ParticipantSpec::new(
            TeamSide::Player,
            FormationIndex::new(0).unwrap(),
            ParticipantSource::Player,
            actor,
        ),
        ParticipantSpec::new(
            TeamSide::Player,
            FormationIndex::new(1).unwrap(),
            ParticipantSource::Player,
            seeder,
        ),
    ];
    if let Some(setup) = input.lifecycle {
        add_setup(&mut builder, catalog, &participants[0], setup);
    }
    if input.full_party {
        for original in players
            .iter()
            .filter(|player| player.formation().get() >= 2)
        {
            let spec = ResolvedCombatantSpec::new(
                form,
                original.combatant().level(),
                Hp::new(10_000).unwrap(),
                Speed::from_scaled(1_000_000).unwrap(),
                ResolvedDefinitionBindings::new(
                    vec![id(IDLE)],
                    original.combatant().rule_bundles().to_vec(),
                    original.combatant().modifiers().to_vec(),
                )
                .unwrap(),
                original.combatant().digest(),
            )
            .unwrap()
            .with_sources(original.combatant().sources().to_vec())
            .unwrap()
            .with_modifier_bindings(original.combatant().modifier_bindings().to_vec())
            .unwrap();
            let mut participant = ParticipantSpec::new(
                TeamSide::Player,
                original.formation(),
                ParticipantSource::Player,
                spec,
            );
            if input.absent_formation == Some(original.formation().get()) {
                participant = participant
                    .with_initial_state(
                        ParticipantInitialState::new(
                            Hp::new(10_000).unwrap(),
                            Hp::new(10_000).unwrap(),
                            Energy::ZERO,
                            Energy::ZERO,
                            LifeState::Alive,
                            PresenceState::Reserved,
                        )
                        .unwrap(),
                    )
                    .unwrap();
            }
            participants.push(participant);
        }
    }
    let mut enemies = Vec::new();
    for index in 0..3 {
        let enemy_id = id(0x7dc5_0001 + u32::from(index));
        builder.add_enemy(EnemyDefinition::new(enemy_id, form, vec![id(IDLE)]));
        enemies.push(enemy_id);
        let enemy = plain(
            input.enemy_level,
            100_000_000,
            input.enemy_hp,
            vec![id(IDLE)],
            input.resistance,
        )
        .with_toughness(
            EnemyRank::Elite,
            vec![],
            vec![ToughnessLayerSpec::ordinary(1, RawToughness::new(100).unwrap()).unwrap()],
        )
        .unwrap();
        let enemy = ParticipantSpec::new(
            TeamSide::Enemy,
            FormationIndex::new(index).unwrap(),
            ParticipantSource::EncounterEnemy(enemy_id),
            enemy,
        );
        participants.push(enemy.clone());
        if input.waves {
            participants.push(enemy.with_wave(2).unwrap());
        }
    }
    let encounter = id(0x7dc7_0001);
    let encounter_definition = if input.waves {
        EncounterDefinition::new(encounter, vec![], vec![])
            .with_waves(vec![enemies.clone(), enemies])
            .unwrap()
    } else {
        EncounterDefinition::new(encounter, enemies, vec![])
    };
    builder.add_encounter(encounter_definition);
    let spec = BattleSpec::new(
        AssemblyDigest::new([0xc3; 32]).unwrap(),
        encounter,
        participants,
        TeamResourceSpec::new(0, 5).unwrap(),
        TeamResourceSpec::new(0, 0).unwrap(),
        ConcedePolicy::Allowed,
    )
    .unwrap();
    let mut battle = Battle::create(
        builder.build().unwrap(),
        spec,
        BattleSeed::new([input.seed; 32]),
    )
    .unwrap();
    let mut ids = battle
        .view()
        .units_by_id()
        .filter(|u| u.side() == TeamSide::Player)
        .map(|u| u.id());
    let actor = ids.next().unwrap();
    let seeder = ids.next().unwrap();
    drop(ids);
    let targets = battle
        .view()
        .units_by_id()
        .filter(|u| u.side() == TeamSide::Enemy)
        .map(|u| u.id())
        .collect();
    let started = battle
        .apply(Command::StartBattle {
            decision: battle.decision().unwrap().id(),
        })
        .unwrap()
        .events()
        .to_vec();
    let mut scenario = Scenario {
        battle,
        actor,
        seeder,
        targets,
        started,
        all: input.all,
        allied: input.allied,
    };
    let seed = command(&mut scenario, seeder, id(SEED), None);
    accept(&mut scenario.battle, seed);
    wait_for(&mut scenario, actor);
    scenario
}
fn damage(amount: i64) -> OrdinaryDamageDefinition {
    OrdinaryDamageDefinition::new(
        Scalar::checked_from_integer(amount).unwrap(),
        OrdinaryDamageMultipliers::new([Ratio::ONE; 9]).unwrap(),
    )
    .unwrap()
}
fn accept(battle: &mut Battle, command: Command) -> Vec<BattleEvent> {
    let resolved = battle.apply(command).unwrap();
    assert!(resolved.fault().is_none(), "{:?}", resolved.fault());
    resolved.events().to_vec()
}
pub(super) fn idle_step(scenario: &mut Scenario) -> Vec<BattleEvent> {
    let command = if let Some(decision) = scenario.battle.decision() {
        decision
            .legal_commands()
            .iter()
            .find(
                |command| matches!(command,Command::UseAbility{ability,..} if ability.get()==IDLE),
            )
            .unwrap()
            .clone()
    } else {
        Command::Advance {
            boundary: scenario.battle.view().action_boundary().unwrap().id(),
        }
    };
    accept(&mut scenario.battle, command)
}
fn wait_for(scenario: &mut Scenario, actor: UnitId) {
    for _ in 0..128 {
        if scenario.battle.decision().is_some_and(|d| {
            d.legal_commands()
                .iter()
                .any(|c| matches!(c,Command::UseAbility{actor:offered,..} if *offered==actor))
        }) {
            return;
        }
        idle_step(scenario);
    }
    panic!("controlled actor did not receive a decision");
}
pub(super) fn command(
    scenario: &mut Scenario,
    actor: UnitId,
    ability: AbilityId,
    target: Option<UnitId>,
) -> Command {
    wait_for(scenario, actor);
    scenario
        .battle
        .decision()
        .unwrap()
        .legal_commands()
        .iter()
        .find(|c| {
            matches!(c,Command::UseAbility{actor:offered,ability:offered_ability,primary_target,..}
        if *offered==actor && *offered_ability==ability && *primary_target==target)
        })
        .unwrap()
        .clone()
}
pub(super) fn attack(scenario: &mut Scenario) -> Vec<BattleEvent> {
    let command = command(
        scenario,
        scenario.actor,
        id(ATTACK),
        if scenario.all {
            None
        } else if scenario.allied {
            Some(scenario.actor)
        } else {
            Some(scenario.targets[0])
        },
    );
    accept(&mut scenario.battle, command)
}
pub(super) fn cast(scenario: &mut Scenario, actor: UnitId, raw: u32) -> Vec<BattleEvent> {
    let command = command(scenario, actor, id(raw), Some(scenario.targets[0]));
    accept(&mut scenario.battle, command)
}
pub(super) fn until_source_tick(
    scenario: &mut Scenario,
    source: SourceDefinitionId,
) -> Vec<BattleEvent> {
    let mut events = vec![];
    for _ in 0..128 {
        let next = idle_step(scenario);
        let tick=next.iter().any(|e| matches!(e.kind(), BattleEventKind::Damage(data) if data.class==DamageClass::Dot && e.cause().source_definition()==Some(source)));
        events.extend(next);
        if tick {
            return events;
        }
    }
    panic!("the selected source-owned Burn did not tick");
}
