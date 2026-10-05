//! Equipment handoffs preserve authored bindings; controlled actions are probes.
use crate::divergent_universe::{
    DivergentUniverseAssembledBattle, DivergentUniverseBaselineFixture,
    tests::{
        curio_battle_grants::ready,
        curio_battle_stats::assemble,
        weighted_curio_footstep_fixture::{concede, remainder},
        weighted_curio_overflow_fixture::{accept, id, start},
    },
    weighted_curio::WeightedCurioSlotLimit,
};
use starclock_build::light_cone::CombatPath;
use starclock_combat::{
    AssemblyDigest, Battle, BattleEvent, BattleEventKind, BattleSeed, BattleSpec,
    CombatantSpecDigest, Command, ConcedePolicy, DecisionId, Energy, FormationIndex, Hp,
    ParticipantSource, ParticipantSpec, Ratio, ResolvedCombatantSpec, ResolvedDefinitionBindings,
    Scalar, Speed, TeamResourceSpec, TeamSide, UnitLevel,
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
use starclock_data::divergent_universe_catalog::DivergentUniverseRunFamily;
use starclock_replay::battle_event::encode_battle_event_payload;
use std::sync::Arc;

const IDLE: u32 = 0x7f60_0001;
const LOSS: u32 = 0x7f60_0002;
const HEAL: u32 = 0x7f60_0003;
const DAMAGE: u32 = 0x7f60_0004;

fn scalar(value: i64) -> ValueExpr {
    ValueExpr::Literal(RuleValue::Scalar(
        Scalar::checked_from_integer(value).unwrap(),
    ))
}

fn probe(source: &DivergentUniverseAssembledBattle) -> Battle {
    let mut builder = CombatCatalogBuilder::from_catalog(source.combat_catalog(), [0xb1; 32]);
    let rule_selector = id(0x7f61_0000);
    builder.add_selector(
        SelectorDefinition::new(rule_selector).with_rule_units(
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
        // Crucially retain normal assembly's bundles, sources and modifier bindings.
        // No Footstep policy constructor is invoked in this probe.
        let spec = ResolvedCombatantSpec::new(
            base.form(),
            base.level(),
            Hp::new(100).unwrap(),
            Speed::from_scaled(100_000_000).unwrap(),
            ResolvedDefinitionBindings::new(
                vec![id(IDLE), id(LOSS), id(HEAL), id(DAMAGE)],
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
        players.push(ParticipantSpec::new(
            original.side(),
            original.formation(),
            original.source(),
            spec,
        ));
    }
    let form = id(0x7f64_0001);
    let enemy = id(0x7f65_0001);
    let encounter = id(0x7f66_0001);
    builder.add_unit(UnitDefinition::new(form, vec![id(IDLE)], vec![]));
    builder.add_enemy(EnemyDefinition::new(enemy, form, vec![id(IDLE)]));
    builder.add_encounter(EncounterDefinition::new(encounter, vec![enemy], vec![]));
    players.push(ParticipantSpec::new(
        TeamSide::Enemy,
        FormationIndex::new(0).unwrap(),
        ParticipantSource::EncounterEnemy(enemy),
        ResolvedCombatantSpec::new(
            form,
            UnitLevel::new(1).unwrap(),
            Hp::new(100_000).unwrap(),
            Speed::from_scaled(1_000_000).unwrap(),
            ResolvedDefinitionBindings::new(vec![id(IDLE)], vec![], vec![]).unwrap(),
            CombatantSpecDigest::new([0xb2; 32]).unwrap(),
        )
        .unwrap(),
    ));
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

fn cast(battle: &mut Battle, owner: u64, raw: u32) -> Vec<BattleEvent> {
    let target = if raw == DAMAGE { 5 } else { owner };
    for _ in 0..64 {
        if let Some(command) = battle.decision().and_then(|decision| decision.legal_commands().iter().find(|command|
            matches!(command, Command::UseAbility {actor, ability, primary_target, ..}
                if actor.get() == owner && ability.get() == raw && primary_target.is_some_and(|unit| unit.get() == target))).cloned()) {
            return accept(battle, command);
        }
        let command = battle.decision().and_then(|decision| decision.legal_commands().iter()
            .find(|command| matches!(command, Command::UseAbility {ability, ..} if ability.get() == IDLE)).cloned())
            .or_else(|| battle.advance_command()).unwrap();
        accept(battle, command);
    }
    panic!("equipment probe command not offered: {owner}/{raw}");
}
fn stacks(battle: &Battle, owner: u64) -> u16 {
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
fn damage(events: &[BattleEvent]) -> Vec<i64> {
    events
        .iter()
        .filter_map(|event| match event.kind() {
            BattleEventKind::Damage(data) => Some(data.applied.get()),
            _ => None,
        })
        .collect()
}

#[test]
fn weighted_curio_footstep_equipment_handoffs_both_families_execute_authored_original_policies() {
    let fixture =
        DivergentUniverseBaselineFixture::production_for_source_party([1107, 1402, 1009, 1002])
            .unwrap();
    let runtime = fixture.factory().weighted_curio_runtime().unwrap();
    let definition = &fixture
        .factory()
        .decision_catalog()
        .weighted_curio_footsteps()[0];
    for family in [
        DivergentUniverseRunFamily::Ordinary,
        DivergentUniverseRunFamily::Cyclical,
    ] {
        let (flow, mut activity) = ready(&fixture, family);
        let stale = activity.state_hash();
        runtime
            .replace_accepted_loadout(
                &flow,
                &mut activity,
                stale,
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
                    stale,
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
                CombatPath::Destruction,
                CombatPath::Remembrance,
                CombatPath::Harmony,
                CombatPath::Hunt
            ]
        );
        let mut actual = Battle::create(
            Arc::clone(source.combat_catalog()),
            source.battle_spec().clone(),
            BattleSeed::new([0xb5; 32]),
        )
        .unwrap();
        start(&mut actual);
        concede(&mut actual);
        for owner in 1..=4 {
            let run = || {
                let mut battle = probe(&source);
                assert_eq!(stacks(&battle, owner), 0);
                let mut events = cast(&mut battle, owner, LOSS);
                events.extend(cast(&mut battle, owner, HEAL));
                events.extend(cast(&mut battle, owner, LOSS));
                assert_eq!(
                    remainder(&battle, owner),
                    if owner <= 2 {
                        Scalar::checked_from_integer(10).unwrap()
                    } else {
                        Scalar::ZERO
                    }
                );
                assert_eq!(
                    battle.view().team(TeamSide::Player).skill_points(),
                    u16::from(owner <= 2)
                );
                assert_eq!(stacks(&battle, owner), if owner <= 2 { 3 } else { 0 });
                assert_eq!(
                    damage(&cast(&mut battle, owner, DAMAGE)),
                    [if owner <= 2 { 124 } else { 100 }]
                );
                for other in 1..=4 {
                    if other != owner {
                        assert_eq!(stacks(&battle, other), 0);
                    }
                }
                assert_eq!(battle.view().rng_draw_count(), 0);
                let hash = battle.state_hash();
                assert!(
                    battle
                        .apply(Command::Concede {
                            decision: DecisionId::new(999_999).unwrap()
                        })
                        .is_err()
                );
                assert_eq!(battle.state_hash(), hash);
                let payloads = events
                    .iter()
                    .map(|event| encode_battle_event_payload(event).unwrap())
                    .collect::<Vec<_>>();
                concede(&mut battle);
                assert_eq!(stacks(&battle, owner), 0);
                assert_eq!(remainder(&battle, owner), Scalar::ZERO);
                (events, payloads, battle.state_hash())
            };
            assert_eq!(run(), run());
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
        assert_ne!(empty.battle_spec(), source.battle_spec());
        let mut battle = probe(&empty);
        cast(&mut battle, 1, LOSS);
        cast(&mut battle, 1, HEAL);
        cast(&mut battle, 1, LOSS);
        assert_eq!(battle.view().team(TeamSide::Player).skill_points(), 0);
        assert_eq!(stacks(&battle, 1), 0);
        assert_eq!(damage(&cast(&mut battle, 1, DAMAGE)), [100]);
    }
}
