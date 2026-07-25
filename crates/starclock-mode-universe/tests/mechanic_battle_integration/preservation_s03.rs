use super::*;
use starclock_combat::{
    ProgramId, RuleBundleId, SelectorId,
    catalog::{
        action::{
            AbilityActionDefinition, AbilityProgramBinding, AbilityProgramTiming,
            TargetInvalidationPolicy, TargetPattern, TargetRelation, UnitTargetSelector,
        },
        builder::CombatCatalogBuilder,
        definition::{AbilityDefinition, ProgramDefinition, SelectorDefinition, UnitDefinition},
        selector::{
            RuleEmptyPoolPolicy, RuleLifePredicate, RulePresencePredicate, RuleSelectorChoice,
            RuleSelectorOrdering, RuleSelectorOrigin, RuleSelectorReference, RuleSelectorSide,
            RuleUnitSelector,
        },
    },
    formula::model::{CombatElement, DamageClass},
    rule::model::{ProgramStep, RuleOperationTemplate, RuleValue, ValueExpr},
};

const FIXTURE_ABILITY: u32 = 0x7f10_0001;
const FIXTURE_UNIT: u32 = 0x7f10_0002;
const FIXTURE_ACTION_SELECTOR: u32 = 0x7f10_0003;
const FIXTURE_RULE_SELECTOR: u32 = 0x7f10_0004;
const FIXTURE_PROGRAM: u32 = 0x7f10_0005;

#[derive(Clone, Copy)]
enum FixtureOperation {
    DamageAlly,
    BreakEnemy,
}

#[test]
fn goal07_p2_m02_s03_executes_timed_shield_and_mitigation_rules() {
    let catalog = catalog();
    let entry_shields = |level, marker| {
        let contributions = contributions_many(
            &catalog,
            "universe.path.preservation",
            &[
                ("universe.blessing.612051", level),
                ("universe.blessing.612054", level),
                ("universe.blessing.612050", 2),
            ],
            None,
            false,
        );
        assert_eq!(contributions.materialized_rule_binding_count(), 4);
        let materialization = materialize(&catalog, &contributions);
        let (battle, start) = start(
            &materialization,
            durable_spec(&materialization, marker, false),
            marker.wrapping_add(1),
        );
        assert!(start.fault().is_none(), "{:?}", start.fault());
        let mut shields = battle
            .view()
            .shields_by_id()
            .filter(|shield| shield.remaining().get() > 0)
            .map(|shield| shield.remaining().get())
            .collect::<Vec<_>>();
        shields.sort_unstable();
        shields
    };
    assert_eq!(entry_shields(1, 0xd1), vec![16_000; 4]);
    assert_eq!(entry_shields(2, 0xd2), vec![24_000; 4]);
}

#[test]
fn goal07_p2_m02_s03_accumulates_action_hp_loss_and_reduces_damage_while_shielded() {
    let catalog = catalog();
    let incoming = |firmness: bool, marker| {
        let third = if firmness {
            ("universe.blessing.612054", 1)
        } else {
            ("universe.blessing.612043", 1)
        };
        let contributions = contributions_many(
            &catalog,
            "universe.path.preservation",
            &[
                ("universe.blessing.612051", 1),
                ("universe.blessing.612050", 2),
                third,
            ],
            None,
            false,
        );
        let materialization = materialize(&catalog, &contributions);
        fixture_action(&materialization, marker, FixtureOperation::DamageAlly)
            .events()
            .iter()
            .find_map(|event| match event.kind() {
                BattleEventKind::Damage(data) => Some(data.calculated.get()),
                _ => None,
            })
            .expect("fixture damage")
    };
    let normal = incoming(false, 0xd3);
    let reduced = incoming(true, 0xd3);
    assert_eq!((normal, reduced), (1_000, 840));

    for (level, marker) in [(1, 0xd4), (2, 0xd5)] {
        let contributions = contributions_many(
            &catalog,
            "universe.path.preservation",
            &[
                ("universe.blessing.612052", level),
                ("universe.blessing.612054", level),
                ("universe.blessing.612055", level),
            ],
            None,
            false,
        );
        let materialization = materialize(&catalog, &contributions);
        let action = fixture_action(&materialization, marker, FixtureOperation::DamageAlly);
        let lost = action
            .events()
            .iter()
            .filter_map(|event| match event.kind() {
                BattleEventKind::Damage(data) => Some(data.applied.get()),
                _ => None,
            })
            .sum::<i64>();
        let shields = action
            .events()
            .iter()
            .filter_map(|event| match event.kind() {
                BattleEventKind::Shield(starclock_combat::ShieldEventData::Applied {
                    amount,
                    ..
                }) => Some(amount.get()),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(
            shields,
            vec![lost * 18 / 100],
            "events: {:?}",
            action.events()
        );
    }
}

#[test]
fn goal07_p2_m02_s03_executes_break_shields_and_rotation_chance_programs() {
    let catalog = catalog();
    let contributions = contributions_many(
        &catalog,
        "universe.path.preservation",
        &[
            ("universe.blessing.612051", 2),
            ("universe.blessing.612053", 2),
            ("universe.blessing.612055", 2),
        ],
        None,
        false,
    );
    let materialization = materialize(&catalog, &contributions);
    let (_, start) = start(
        &materialization,
        durable_spec(&materialization, 0xd6, false),
        0xd7,
    );
    assert!(start.fault().is_none(), "{:?}", start.fault());
    assert!(
        start.events().iter().any(|event| {
            matches!(
                event.kind(),
                BattleEventKind::Effect(starclock_combat::EffectEventData::Removed { .. })
            )
        }),
        "four enhanced Rotation draws include an executable success for the frozen seed"
    );

    let action = fixture_action(&materialization, 0xd8, FixtureOperation::BreakEnemy);
    assert!(
        action.events().iter().any(|event| {
            matches!(
                event.kind(),
                BattleEventKind::Toughness(starclock_combat::ToughnessEventData::LayerDepleted {
                    changed_global_broken: true,
                    ..
                })
            )
        }),
        "events: {:?}",
        action.events()
    );
    let break_shields = action
        .events()
        .iter()
        .filter_map(|event| match event.kind() {
            BattleEventKind::Shield(starclock_combat::ShieldEventData::Applied {
                amount, ..
            }) if amount.get() == 18_000 => Some(amount.get()),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(break_shields, vec![18_000; 4]);
}

fn fixture_action(
    materialization: &UniverseBattleMaterialization,
    marker: u8,
    operation: FixtureOperation,
) -> starclock_combat::Resolution {
    let original = durable_spec(materialization, marker, false);
    let first = original
        .participants()
        .iter()
        .position(|participant| participant.side() == TeamSide::Player)
        .unwrap();
    let base = original.participants()[first].combatant();
    let bundles = base.rule_bundles().to_vec();
    let combat_catalog = fixture_catalog(
        materialization.combat_catalog(),
        &bundles,
        marker,
        operation,
    );
    let mut participants = original.participants().to_vec();
    let fixture = ResolvedCombatantSpec::new(
        UnitDefinitionId::new(FIXTURE_UNIT).unwrap(),
        base.level(),
        base.maximum_hp(),
        base.speed(),
        ResolvedDefinitionBindings::new(
            vec![AbilityId::new(FIXTURE_ABILITY).unwrap()],
            bundles,
            base.modifiers().to_vec(),
        )
        .unwrap(),
        CombatantSpecDigest::new([marker.wrapping_add(10); 32]).unwrap(),
    )
    .unwrap()
    .with_base_attack_defense(base.base_attack(), base.base_defense())
    .with_energy(base.current_energy(), base.maximum_energy())
    .unwrap()
    .with_sources(base.sources().to_vec())
    .unwrap()
    .with_modifier_bindings(base.modifier_bindings().to_vec())
    .unwrap();
    let original_player = &participants[first];
    participants[first] = ParticipantSpec::new(
        TeamSide::Player,
        original_player.formation(),
        original_player.source(),
        fixture,
    )
    .with_wave(original_player.wave())
    .unwrap();
    if matches!(operation, FixtureOperation::BreakEnemy) {
        let enemy_index = participants
            .iter()
            .position(|participant| participant.side() == TeamSide::Enemy)
            .unwrap();
        let participant = &participants[enemy_index];
        let base = participant.combatant();
        let combatant = ResolvedCombatantSpec::new(
            base.form(),
            base.level(),
            base.maximum_hp(),
            base.speed(),
            ResolvedDefinitionBindings::new(
                base.abilities().to_vec(),
                base.rule_bundles().to_vec(),
                base.modifiers().to_vec(),
            )
            .unwrap(),
            CombatantSpecDigest::new([marker.wrapping_add(13); 32]).unwrap(),
        )
        .unwrap()
        .with_base_attack_defense(base.base_attack(), base.base_defense())
        .with_energy(base.current_energy(), base.maximum_energy())
        .unwrap()
        .with_sources(base.sources().to_vec())
        .unwrap()
        .with_modifier_bindings(base.modifier_bindings().to_vec())
        .unwrap()
        .with_toughness(
            starclock_combat::formula::toughness::EnemyRank::Normal,
            vec![CombatElement::Physical],
            vec![
                starclock_combat::ToughnessLayerSpec::ordinary(
                    1,
                    starclock_combat::RawToughness::new(50).unwrap(),
                )
                .unwrap(),
            ],
        )
        .unwrap();
        participants[enemy_index] = ParticipantSpec::new(
            TeamSide::Enemy,
            participant.formation(),
            participant.source(),
            combatant,
        )
        .with_wave(participant.wave())
        .unwrap();
    }
    let spec = BattleSpec::new(
        original.rules_revision(),
        AssemblyDigest::new([marker.wrapping_add(11); 32]).unwrap(),
        original.encounter(),
        participants,
        original.resources(TeamSide::Player).clone(),
        original.resources(TeamSide::Enemy).clone(),
        original.concede_policy(),
    )
    .unwrap();
    let mut battle = Battle::create(
        combat_catalog,
        spec,
        BattleSeed::new([marker.wrapping_add(12); 32]),
    )
    .unwrap();
    let start = battle
        .apply(Command::StartBattle {
            decision: battle.decision().unwrap().id(),
        })
        .unwrap();
    assert!(start.fault().is_none(), "{:?}", start.fault());
    if battle
        .decision()
        .is_some_and(|decision| decision.kind() == starclock_combat::DecisionKind::InterruptWindow)
    {
        battle
            .apply(Command::PassInterruptWindow {
                decision: battle.decision().unwrap().id(),
            })
            .unwrap();
    }
    let breakable = battle
        .view()
        .units_by_id()
        .filter(|unit| unit.side() == TeamSide::Enemy && unit.toughness_layers().next().is_some())
        .map(|unit| unit.id())
        .collect::<Vec<_>>();
    let command = battle
        .decision()
        .unwrap()
        .legal_commands()
        .iter()
        .find(|command| {
            matches!(
                command,
                Command::UseAbility { ability, primary_target, .. }
                    if ability.get() == FIXTURE_ABILITY
                        && match operation {
                            FixtureOperation::DamageAlly => true,
                            FixtureOperation::BreakEnemy => primary_target
                                .is_some_and(|target| breakable.contains(&target)),
                        }
            )
        })
        .expect("fixture ability is legal")
        .clone();
    battle.apply(command).unwrap()
}

fn fixture_catalog(
    base: &Arc<starclock_combat::catalog::CombatCatalog>,
    rule_bundles: &[RuleBundleId],
    marker: u8,
    operation: FixtureOperation,
) -> Arc<starclock_combat::catalog::CombatCatalog> {
    let action_selector = SelectorId::new(FIXTURE_ACTION_SELECTOR).unwrap();
    let rule_selector = SelectorId::new(FIXTURE_RULE_SELECTOR).unwrap();
    let program = ProgramId::new(FIXTURE_PROGRAM).unwrap();
    let ability = AbilityId::new(FIXTURE_ABILITY).unwrap();
    let relation = match operation {
        FixtureOperation::DamageAlly => TargetRelation::Allied,
        FixtureOperation::BreakEnemy => TargetRelation::Opposing,
    };
    let side = match operation {
        FixtureOperation::DamageAlly => RuleSelectorSide::Same,
        FixtureOperation::BreakEnemy => RuleSelectorSide::Opposing,
    };
    let mut builder = CombatCatalogBuilder::from_catalog(
        base,
        "goal07-preservation-s03-fixture-v1",
        [marker; 32],
    );
    builder.add_selector(
        SelectorDefinition::new(action_selector)
            .with_unit_targets(UnitTargetSelector::new(relation, TargetPattern::Single).unwrap()),
    );
    builder.add_selector(
        SelectorDefinition::new(rule_selector).with_rule_units(
            RuleUnitSelector::new(
                RuleSelectorOrigin::PrimaryTarget,
                side,
                RuleLifePredicate::Alive,
                RulePresencePredicate::Present,
                RuleSelectorReference::CurrentState,
                RuleSelectorOrdering::Formation,
                1,
                1,
                RuleEmptyPoolPolicy::Fault,
                RuleSelectorChoice::First,
                None,
                false,
            )
            .unwrap(),
        ),
    );
    let operation = match operation {
        FixtureOperation::DamageAlly => RuleOperationTemplate::Damage {
            selector: rule_selector,
            amount: ValueExpr::Literal(RuleValue::Scalar(
                starclock_combat::Scalar::checked_from_integer(1_000).unwrap(),
            )),
            class: DamageClass::Direct,
            element: CombatElement::Physical,
            can_crit: false,
            can_defeat: false,
        },
        FixtureOperation::BreakEnemy => RuleOperationTemplate::Break {
            selector: rule_selector,
            element: CombatElement::Physical,
        },
    };
    builder.add_program(
        ProgramDefinition::new(
            program,
            Vec::new(),
            vec![rule_selector],
            Vec::new(),
            Vec::new(),
        )
        .with_steps(vec![ProgramStep::Operation(operation)]),
    );
    let action = AbilityActionDefinition::new(
        AbilityKind::Skill,
        1,
        TargetInvalidationPolicy::CancelRemainingForTarget,
        starclock_combat::catalog::action::ActionResourcePolicy::new(
            0,
            0,
            Energy::ZERO,
            Energy::ZERO,
        ),
    )
    .unwrap();
    builder.add_ability(
        AbilityDefinition::new(ability, program, action_selector, Vec::new())
            .with_action(action)
            .with_programs(vec![
                AbilityProgramBinding::new(1, AbilityProgramTiming::BeforeHits, program).unwrap(),
            ]),
    );
    builder.add_unit(UnitDefinition::new(
        UnitDefinitionId::new(FIXTURE_UNIT).unwrap(),
        vec![ability],
        rule_bundles.to_vec(),
    ));
    builder.build().unwrap()
}
