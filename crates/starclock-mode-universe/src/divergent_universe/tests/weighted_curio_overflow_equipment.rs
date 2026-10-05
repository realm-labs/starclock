//! Both-family normal equipment handoffs; probes retain the assembled bindings.
use crate::divergent_universe::{
    DivergentUniverseAssembledBattle, DivergentUniverseBaselineFixture,
    tests::{
        curio_battle_grants::ready, curio_battle_stats::assemble,
        weighted_curio_encouragement_fixture::accept, weighted_curio_overflow_fixture::id,
    },
    weighted_curio::WeightedCurioSlotLimit,
};
use starclock_build::light_cone::CombatPath;
use starclock_combat::{
    AssemblyDigest, Battle, BattleEvent, BattleEventKind, BattleSeed, BattleSpec,
    CombatantSpecDigest, Command, ConcedePolicy, DecisionId, Energy, FormationIndex, Hp,
    ParticipantSource, ParticipantSpec, Ratio, ResolvedCombatantSpec, ResolvedDefinitionBindings,
    Scalar, Speed, StatValue, TeamResourceSpec, TeamSide, UnitId, UnitLevel,
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
    modifier::model::{FormulaPurpose, StatKind, StatQuerySubject},
    rule::model::{ProgramStep, RuleOperationTemplate, ValueExpr},
};
use starclock_data::divergent_universe_catalog::DivergentUniverseRunFamily;
use starclock_replay::battle_event::encode_battle_event_payload;
use std::sync::Arc;

const ATTACK: u32 = 0x7f40_0001;
const ATK_QUERY: u32 = 0x7f40_0002;
const IDLE: u32 = 0x7f40_0003;

fn cast(battle: &mut Battle, formation: u8, raw: u32, target: Option<UnitId>) -> Vec<BattleEvent> {
    let actor = battle
        .view()
        .units_by_id()
        .find(|unit| unit.side() == TeamSide::Player && unit.formation().get() == formation)
        .unwrap()
        .id();
    for _ in 0..256 {
        if let Some(command) = battle.decision().and_then(|decision| {
            decision
                .legal_commands()
                .iter()
                .find(|command| {
                    matches!(command,
                Command::UseAbility {actor: offered, ability, primary_target, ..}
                    if *offered == actor && ability.get() == raw && *primary_target == target)
                })
                .cloned()
        }) {
            return accept(battle, command);
        }
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
            battle.advance_command().unwrap()
        };
        accept(battle, command);
    }
    panic!("controlled command not offered: {formation}/{raw}");
}

fn probe(source: &DivergentUniverseAssembledBattle) -> Battle {
    let mut builder = CombatCatalogBuilder::from_catalog(source.combat_catalog(), [0xf4; 32]);
    let query_selector = id(0x7f41_0010);
    builder.add_selector(
        SelectorDefinition::new(query_selector).with_rule_units(
            RuleUnitSelector::new(
                RuleSelectorOrigin::Encounter,
                RuleSelectorSide::Opposing,
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
    for raw in [ATTACK, ATK_QUERY, IDLE] {
        let selector = id(raw + 0x10000);
        let program = id(raw + 0x20000);
        builder.add_selector(
            SelectorDefinition::new(selector).with_unit_targets(
                UnitTargetSelector::new(
                    if raw == IDLE {
                        TargetRelation::SelfUnit
                    } else {
                        TargetRelation::Opposing
                    },
                    if raw == ATTACK {
                        TargetPattern::All
                    } else {
                        TargetPattern::Single
                    },
                )
                .unwrap(),
            ),
        );
        builder.add_program(
            ProgramDefinition::new(
                program,
                vec![],
                if raw == ATK_QUERY {
                    vec![query_selector]
                } else {
                    vec![]
                },
                vec![],
                vec![],
            )
            .with_steps(if raw == ATK_QUERY {
                vec![ProgramStep::Operation(RuleOperationTemplate::TrueDamage {
                    selector: query_selector,
                    amount: ValueExpr::QueryStat {
                        subject: StatQuerySubject::Actor,
                        stat: StatKind::Atk,
                        purpose: FormulaPurpose::Stat,
                    },
                })]
            } else {
                vec![]
            }),
        );
        let hit = if raw == ATTACK {
            vec![HitOperationDefinition::Damage(
                OrdinaryDamageDefinition::new(
                    Scalar::checked_from_integer(250).unwrap(),
                    OrdinaryDamageMultipliers::new([Ratio::ONE; 9]).unwrap(),
                )
                .unwrap(),
            )]
        } else {
            vec![]
        };
        builder.add_ability(
            AbilityDefinition::new(id(raw), program, selector, vec![])
                .with_action(
                    AbilityActionDefinition::new(
                        if raw == ATK_QUERY {
                            AbilityKind::Skill
                        } else {
                            AbilityKind::Basic
                        },
                        1,
                        TargetInvalidationPolicy::CancelRemainingForTarget,
                        ActionResourcePolicy::new(0, 0, Energy::ZERO, Energy::ZERO),
                    )
                    .unwrap()
                    .with_hits(vec![ActionHitDefinition::new(hit).with_profile(
                        HitTargetGroup::Selected,
                        Ratio::ONE,
                        Ratio::ONE,
                        HitCritPolicy::Never,
                    )])
                    .unwrap(),
                )
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
        let spec = ResolvedCombatantSpec::new(
            base.form(),
            base.level(),
            Hp::new(10_000).unwrap(),
            Speed::from_scaled(100_000_000).unwrap(),
            ResolvedDefinitionBindings::new(
                vec![id(ATTACK), id(ATK_QUERY), id(IDLE)],
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
        .unwrap()
        .with_base_attack_defense(
            StatValue::from_scaled(100_000_000).unwrap(),
            StatValue::from_scaled(0).unwrap(),
        );
        players.push(ParticipantSpec::new(
            original.side(),
            original.formation(),
            original.source(),
            spec,
        ));
    }
    let form = id(0x7f44_0001);
    builder.add_unit(UnitDefinition::new(form, vec![id(IDLE)], vec![]));
    let mut enemies = Vec::new();
    for (formation, hp, level) in [(0, 100, 95), (1, 3_000_000, 1)] {
        let enemy = id(0x7f45_0000 + u32::from(formation));
        builder.add_enemy(EnemyDefinition::new(enemy, form, vec![id(IDLE)]));
        let spec = ResolvedCombatantSpec::new(
            form,
            UnitLevel::new(level).unwrap(),
            Hp::new(hp).unwrap(),
            Speed::from_scaled(100_000_000).unwrap(),
            ResolvedDefinitionBindings::new(vec![id(IDLE)], vec![], vec![]).unwrap(),
            CombatantSpecDigest::new([formation + 10; 32]).unwrap(),
        )
        .unwrap();
        players.push(ParticipantSpec::new(
            TeamSide::Enemy,
            FormationIndex::new(formation).unwrap(),
            ParticipantSource::EncounterEnemy(enemy),
            spec,
        ));
        enemies.push(enemy);
    }
    let encounter = id(0x7f46_0001);
    builder.add_encounter(
        EncounterDefinition::new(encounter, vec![], vec![])
            .with_waves(vec![enemies])
            .unwrap(),
    );
    let spec = BattleSpec::new(
        AssemblyDigest::new([0xf4; 32]).unwrap(),
        encounter,
        players,
        TeamResourceSpec::new(3, 5).unwrap(),
        TeamResourceSpec::new(0, 0).unwrap(),
        ConcedePolicy::Allowed,
    )
    .unwrap();
    let mut battle =
        Battle::create(builder.build().unwrap(), spec, BattleSeed::new([0xf4; 32])).unwrap();
    let command = Command::StartBattle {
        decision: battle.decision().unwrap().id(),
    };
    accept(&mut battle, command);
    battle
}
fn amounts(events: &[BattleEvent]) -> Vec<i64> {
    events
        .iter()
        .filter_map(|event| match event.kind() {
            BattleEventKind::Damage(data) => Some(data.calculated.get()),
            _ => None,
        })
        .collect()
}
fn attack_targets(battle: &Battle) -> Vec<u64> {
    battle
        .view()
        .effects_by_id()
        .filter(|effect| (0x7f0e_0000..0x7f0f_0000).contains(&effect.definition().get()))
        .map(|effect| effect.target().get())
        .collect()
}

#[test]
fn weighted_curio_overflow_equipment_executes_both_families_and_unequips_without_residue() {
    let fixture =
        DivergentUniverseBaselineFixture::production_for_source_party([1002, 1003, 1009, 1001])
            .unwrap();
    let runtime = fixture.factory().weighted_curio_runtime().unwrap();
    let definition = &fixture
        .factory()
        .decision_catalog()
        .weighted_curio_overflows()[0];
    for family in [
        DivergentUniverseRunFamily::Ordinary,
        DivergentUniverseRunFamily::Cyclical,
    ] {
        let (flow, mut activity) = ready(&fixture, family);
        let hash = activity.state_hash();
        runtime
            .replace_accepted_loadout(
                &flow,
                &mut activity,
                hash,
                WeightedCurioSlotLimit::new(1).unwrap(),
                std::slice::from_ref(&definition.weighted_curio),
            )
            .unwrap();
        let before = activity.canonical_state_bytes();
        let debug = activity.debug_view();
        assert!(
            runtime
                .replace_accepted_loadout(
                    &flow,
                    &mut activity,
                    hash,
                    WeightedCurioSlotLimit::new(1).unwrap(),
                    &[]
                )
                .is_err()
        );
        assert_eq!(activity.canonical_state_bytes(), before);
        assert_eq!(activity.debug_view(), debug);
        let source = assemble(&fixture, &flow, &activity);
        assert_eq!(activity.canonical_state_bytes(), before);
        assert_eq!(activity.debug_view(), debug);
        let paths = source
            .battle_spec()
            .participants()
            .iter()
            .filter(|p| p.side() == TeamSide::Player)
            .map(|p| {
                fixture
                    .core()
                    .build_catalog()
                    .character(p.combatant().form())
                    .unwrap()
                    .path()
            })
            .collect::<Vec<_>>();
        assert_eq!(
            paths,
            [
                CombatPath::Hunt,
                CombatPath::Erudition,
                CombatPath::Harmony,
                CombatPath::Preservation
            ]
        );
        // Also start the unmodified production materialization, not only probes.
        let mut actual = Battle::create(
            Arc::clone(source.combat_catalog()),
            source.battle_spec().clone(),
            BattleSeed::new([0xf4; 32]),
        )
        .unwrap();
        let command = Command::StartBattle {
            decision: actual.decision().unwrap().id(),
        };
        accept(&mut actual, command);
        assert_eq!(attack_targets(&actual), [1, 2]);
        let concede = actual
            .decision()
            .unwrap()
            .legal_commands()
            .iter()
            .find(|command| matches!(command, Command::Concede { .. }))
            .unwrap()
            .clone();
        accept(&mut actual, concede);
        assert!(attack_targets(&actual).is_empty());
        for formation in 0..4 {
            let mut battle = probe(&source);
            assert_eq!(attack_targets(&battle), [1, 2]);
            let target = battle
                .view()
                .units_by_id()
                .find(|u| u.side() == TeamSide::Enemy && u.formation().get() == 0)
                .unwrap()
                .id();
            let query = cast(&mut battle, formation, ATK_QUERY, Some(target));
            assert_eq!(amounts(&query), [if formation < 2 { 180 } else { 100 }]);
            let mut a = probe(&source);
            let mut b = probe(&source);
            let events = cast(&mut a, formation, ATTACK, None);
            let repeated = cast(&mut b, formation, ATTACK, None);
            assert_eq!(
                amounts(&events),
                if formation < 2 {
                    vec![250, 250, 950]
                } else {
                    vec![250, 250]
                }
            );
            assert_eq!(a.view().rng_draw_count(), if formation < 2 { 1 } else { 0 });
            assert_eq!(events, repeated);
            let payloads = |events: &[BattleEvent]| {
                events
                    .iter()
                    .map(|event| encode_battle_event_payload(event).unwrap())
                    .collect::<Vec<_>>()
            };
            assert_eq!(payloads(&events), payloads(&repeated));
            assert_eq!(a.state_hash(), b.state_hash());
            let before = a.state_hash();
            let draws = a.view().rng_draw_count();
            assert!(
                a.apply(Command::Concede {
                    decision: DecisionId::new(999_999).unwrap()
                })
                .is_err()
            );
            assert_eq!(a.state_hash(), before);
            assert_eq!(a.view().rng_draw_count(), draws);
        }
        let hash = activity.state_hash();
        runtime
            .replace_accepted_loadout(
                &flow,
                &mut activity,
                hash,
                WeightedCurioSlotLimit::new(1).unwrap(),
                &[],
            )
            .unwrap();
        let empty = assemble(&fixture, &flow, &activity);
        let mut battle = probe(&empty);
        assert!(attack_targets(&battle).is_empty());
        let events = cast(&mut battle, 0, ATTACK, None);
        assert_eq!(amounts(&events), [250, 250]);
        assert_eq!(battle.view().rng_draw_count(), 0);
    }
}
