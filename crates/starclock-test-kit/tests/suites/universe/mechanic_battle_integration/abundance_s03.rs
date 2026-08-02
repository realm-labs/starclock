use super::*;
use starclock_combat::{
    ParticipantInitialState, ProgramId, RuleBundleId, SelectorId,
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
    formula::model::CombatElement,
    modifier::model::{FormulaPurpose, FormulaStage, FormulaSubject, ModifierFilter, StatKind},
    rule::model::{
        OnceScope, ProgramStep, RuleEventPoint, RuleOperationTemplate, RuleValue, SourceClass,
        ValueExpr,
    },
};

const HEALING_RECEIVED: (&str, u32) = ("universe.blessing.612351", 2);
const ENTRY_HEALING: (&str, u32) = ("universe.blessing.612352", 2);
const BREAK_HEALING: (&str, u32) = ("universe.blessing.612353", 2);
const HEALED_DEFENSE: (&str, u32) = ("universe.blessing.612354", 2);
const PROVIDER_HEALING: (&str, u32) = ("universe.blessing.612355", 2);
const FIXTURE_ABILITY: u32 = 0x7f50_0001;
const FIXTURE_UNIT: u32 = 0x7f50_0002;
const FIXTURE_ACTION_SELECTOR: u32 = 0x7f50_0003;
const FIXTURE_OWNER_SELECTOR: u32 = 0x7f50_0004;
const FIXTURE_TARGET_SELECTOR: u32 = 0x7f50_0005;
const FIXTURE_PROGRAM: u32 = 0x7f50_0006;

#[test]
fn goal07_p2_m05_s03_materializes_all_assigned_mechanics_without_native_handlers() {
    let catalog = catalog();
    let selected = [
        HEALING_RECEIVED,
        ENTRY_HEALING,
        BREAK_HEALING,
        HEALED_DEFENSE,
        PROVIDER_HEALING,
    ];
    let contributions =
        contributions_many(&catalog, "universe.path.abundance", &selected, None, false);
    let materialization = materialize(&catalog, &contributions);
    let combat = materialization.combat_catalog();
    for key in [
        "StageAbility_612351",
        "StageAbility_612352",
        "StageAbility_612353",
        "StageAbility_612354",
        "StageAbility_612355",
    ] {
        let rule = combat.rule(binding(&contributions, key).rule()).unwrap();
        assert!(
            rule.runtime()
                .is_some_and(|runtime| runtime.native_handler().is_none()),
            "{key} stays in generic Rule IR"
        );
    }

    let incoming = first_modifier(
        combat,
        binding(&contributions, "StageAbility_612351").rule(),
    );
    assert_eq!(
        (incoming.stat, incoming.stage, incoming.purpose),
        (
            StatKind::IncomingHealing,
            FormulaStage::Healing,
            FormulaPurpose::Healing
        )
    );
    assert_eq!(literal_scalar(&incoming.value), 180_000);
    assert_eq!(
        incoming.filters.as_ref(),
        [ModifierFilter::FormulaSubject(FormulaSubject::Target)]
    );

    assert_maximum_hp_heal(
        combat,
        &contributions,
        "StageAbility_612352",
        RuleEventPoint::BattleStarted,
        360_000,
    );
    let break_rule = assert_maximum_hp_heal(
        combat,
        &contributions,
        "StageAbility_612353",
        RuleEventPoint::WeaknessBroken,
        240_000,
    );
    assert!(
        break_rule.runtime().unwrap().triggers()[0]
            .filter
            .actor_selector
            .is_some()
    );

    let defense = binding(&contributions, "StageAbility_612354");
    let defense_effect = first_effect(combat, defense.rule());
    assert!(matches!(
        defense_effect
            .runtime_template()
            .unwrap()
            .duration_expression(),
        Some(ValueExpr::Literal(RuleValue::Integer(1)))
    ));
    let defense_modifier = combat.modifier(defense_effect.modifiers()[0]).unwrap();
    assert_eq!(
        (
            defense_modifier.stat,
            defense_modifier.stage,
            literal_scalar(&defense_modifier.value)
        ),
        (StatKind::Def, FormulaStage::PercentOfBase, 360_000)
    );

    let provider = binding(&contributions, "StageAbility_612355");
    let provider_rule = combat.rule(provider.rule()).unwrap();
    let trigger = &provider_rule.runtime().unwrap().triggers()[0];
    assert_eq!(
        (trigger.event_point, trigger.once_scope),
        (RuleEventPoint::HealApplied, OnceScope::Action)
    );
    assert_eq!(trigger.filter.source_class, Some(SourceClass::Ability));
    assert_eq!(trigger.filter.has_action, Some(true));
    let provider_heal = rule_steps(combat, provider.rule())
        .into_iter()
        .find_map(|step| match step {
            ProgramStep::Operation(RuleOperationTemplate::Heal {
                amount,
                apply_formula_modifiers: true,
                ..
            }) => Some(amount),
            _ => None,
        })
        .unwrap();
    assert!(expression_has_stat(provider_heal, StatKind::Hp));
    assert!(expression_has_scalar(provider_heal, 180_000));
}

#[test]
fn enhanced_dharma_rain_applies_its_nine_blessing_cap() {
    let catalog = catalog();
    let selected = [
        ("universe.blessing.612350", 2),
        HEALING_RECEIVED,
        ENTRY_HEALING,
        BREAK_HEALING,
        HEALED_DEFENSE,
        PROVIDER_HEALING,
        ("universe.blessing.612330", 1),
        ("universe.blessing.612331", 1),
        ("universe.blessing.612332", 1),
    ];
    let contributions =
        contributions_many(&catalog, "universe.path.abundance", &selected, None, false);
    let materialization = materialize(&catalog, &contributions);
    let modifier = first_modifier(
        materialization.combat_catalog(),
        binding(&contributions, "StageAbility_612350").rule(),
    );
    assert_eq!(literal_scalar(&modifier.value), 630_000);
}

#[test]
fn entry_healing_executes_after_incoming_healing_is_installed() {
    let catalog = catalog();
    let contributions = contributions_many(
        &catalog,
        "universe.path.abundance",
        &[HEALING_RECEIVED, ENTRY_HEALING],
        None,
        false,
    );
    let materialization = materialize(&catalog, &contributions);
    let spec = wounded_players(durable_spec(&materialization, 0xb1, false), 50_000, 0xb2);
    let (_, started) = start(&materialization, spec, 0xb3);
    assert!(started.fault().is_none(), "{:?}", started.fault());
    let source = binding(&contributions, "StageAbility_612352")
        .source()
        .definition();
    let heals = started
        .events()
        .iter()
        .filter_map(|event| match event.kind() {
            BattleEventKind::Heal(data) if event.cause().source_definition() == Some(source) => {
                Some((data.calculated.get(), data.effective.get()))
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(heals, vec![(42_480, 42_480); 4]);
}

#[test]
fn healing_action_triggers_provider_once_defense_and_break_healing() {
    let catalog = catalog();
    let contributions = contributions_many(
        &catalog,
        "universe.path.abundance",
        &[
            HEALING_RECEIVED,
            BREAK_HEALING,
            HEALED_DEFENSE,
            PROVIDER_HEALING,
        ],
        None,
        false,
    );
    let materialization = materialize(&catalog, &contributions);
    let result = fixture_action(&materialization, 0xb4);
    assert!(
        result.resolution.fault().is_none(),
        "{:?}",
        result.resolution.fault()
    );

    let provider_source = binding(&contributions, "StageAbility_612355")
        .source()
        .definition();
    let break_source = binding(&contributions, "StageAbility_612353")
        .source()
        .definition();
    let provider = source_heals(&result.resolution, provider_source);
    let break_heals = source_heals(&result.resolution, break_source);
    assert_eq!(
        provider,
        vec![21_240],
        "provider healing is once per action"
    );
    assert_eq!(break_heals, vec![28_320]);
    assert_eq!(
        result
            .resolution
            .events()
            .iter()
            .filter_map(|event| match event.kind() {
                BattleEventKind::Heal(data)
                    if event
                        .cause()
                        .source_definition()
                        .is_some_and(|source| { source.get() == FIXTURE_ABILITY }) =>
                {
                    Some(data.calculated.get())
                }
                _ => None,
            })
            .collect::<Vec<_>>(),
        vec![1_180, 1_180]
    );
    assert!(result.resolution.events().iter().any(|event| {
        matches!(
            event.kind(),
            BattleEventKind::Toughness(starclock_combat::ToughnessEventData::LayerDepleted {
                changed_global_broken: true,
                ..
            })
        )
    }));
    let defense_source = binding(&contributions, "StageAbility_612354")
        .source()
        .definition();
    assert!(result.resolution.events().iter().any(|event| {
        event.cause().source_definition() == Some(defense_source)
            && matches!(
                event.kind(),
                BattleEventKind::Effect(
                    starclock_combat::EffectEventData::Applied { .. }
                        | starclock_combat::EffectEventData::Refreshed { .. }
                )
            )
    }));
}

struct FixtureResult {
    resolution: starclock_combat::Resolution,
}

fn fixture_action(materialization: &UniverseBattleMaterialization, marker: u8) -> FixtureResult {
    let original = durable_spec(materialization, marker, false);
    let first = original
        .participants()
        .iter()
        .position(|participant| participant.side() == TeamSide::Player)
        .unwrap();
    let base = original.participants()[first].combatant();
    let bundles = base.rule_bundles().to_vec();
    let combat_catalog = fixture_catalog(materialization.combat_catalog(), &bundles, marker);
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
    let initial = ParticipantInitialState::new(
        Hp::new(10_000).unwrap(),
        base.maximum_hp(),
        base.current_energy(),
        base.maximum_energy(),
        starclock_combat::LifeState::Alive,
        starclock_combat::PresenceState::Present,
    )
    .unwrap();
    let original_player = &participants[first];
    participants[first] = ParticipantSpec::new(
        TeamSide::Player,
        original_player.formation(),
        original_player.source(),
        fixture,
    )
    .with_wave(original_player.wave())
    .unwrap()
    .with_initial_state(initial)
    .unwrap();

    let enemy = participants
        .iter()
        .position(|participant| participant.side() == TeamSide::Enemy)
        .unwrap();
    let participant = &participants[enemy];
    let base = participant.combatant();
    let enemy_combatant = ResolvedCombatantSpec::new(
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
        CombatantSpecDigest::new([marker.wrapping_add(11); 32]).unwrap(),
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
    participants[enemy] = ParticipantSpec::new(
        TeamSide::Enemy,
        participant.formation(),
        participant.source(),
        enemy_combatant,
    )
    .with_wave(participant.wave())
    .unwrap();

    let spec = BattleSpec::new(
        AssemblyDigest::new([marker.wrapping_add(12); 32]).unwrap(),
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
        BattleSeed::new([marker.wrapping_add(13); 32]),
    )
    .unwrap();
    let started = battle
        .apply(Command::StartBattle {
            decision: battle.decision().unwrap().id(),
        })
        .unwrap();
    assert!(started.fault().is_none(), "{:?}", started.fault());
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
    let command = battle
        .decision()
        .unwrap()
        .legal_commands()
        .iter()
        .find(|command| {
            matches!(
                command,
                Command::UseAbility { ability, .. } if ability.get() == FIXTURE_ABILITY
            )
        })
        .unwrap()
        .clone();
    FixtureResult {
        resolution: battle.apply(command).unwrap(),
    }
}

fn fixture_catalog(
    base: &Arc<starclock_combat::catalog::CombatCatalog>,
    bundles: &[RuleBundleId],
    marker: u8,
) -> Arc<starclock_combat::catalog::CombatCatalog> {
    let action_selector = SelectorId::new(FIXTURE_ACTION_SELECTOR).unwrap();
    let owner = SelectorId::new(FIXTURE_OWNER_SELECTOR).unwrap();
    let target = SelectorId::new(FIXTURE_TARGET_SELECTOR).unwrap();
    let program = ProgramId::new(FIXTURE_PROGRAM).unwrap();
    let ability = AbilityId::new(FIXTURE_ABILITY).unwrap();
    let mut builder =
        CombatCatalogBuilder::from_catalog(base, [marker; 32]);
    builder.add_selector(SelectorDefinition::new(action_selector).with_unit_targets(
        UnitTargetSelector::new(TargetRelation::Opposing, TargetPattern::Single).unwrap(),
    ));
    builder.add_selector(
        SelectorDefinition::new(owner).with_rule_units(rule_selector(
            RuleSelectorOrigin::Actor,
            RuleSelectorSide::Same,
        )),
    );
    builder.add_selector(
        SelectorDefinition::new(target).with_rule_units(rule_selector(
            RuleSelectorOrigin::PrimaryTarget,
            RuleSelectorSide::Opposing,
        )),
    );
    builder.add_program(
        ProgramDefinition::new(
            program,
            Vec::new(),
            vec![owner, target],
            Vec::new(),
            Vec::new(),
        )
        .with_steps(vec![
            ProgramStep::Operation(RuleOperationTemplate::Heal {
                selector: owner,
                amount: ValueExpr::Literal(RuleValue::Scalar(
                    starclock_combat::Scalar::checked_from_integer(1_000).unwrap(),
                )),
                apply_formula_modifiers: true,
            }),
            ProgramStep::Operation(RuleOperationTemplate::Heal {
                selector: owner,
                amount: ValueExpr::Literal(RuleValue::Scalar(
                    starclock_combat::Scalar::checked_from_integer(1_000).unwrap(),
                )),
                apply_formula_modifiers: true,
            }),
            ProgramStep::Operation(RuleOperationTemplate::Break {
                selector: target,
                element: CombatElement::Physical,
            }),
        ]),
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
        bundles.to_vec(),
    ));
    builder.build().unwrap()
}

fn rule_selector(origin: RuleSelectorOrigin, side: RuleSelectorSide) -> RuleUnitSelector {
    RuleUnitSelector::new(
        origin,
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
    .unwrap()
}

fn wounded_players(original: BattleSpec, current_hp: i64, marker: u8) -> BattleSpec {
    let participants = original
        .participants()
        .iter()
        .map(|participant| {
            if participant.side() != TeamSide::Player {
                return participant.clone();
            }
            let combatant = participant.combatant();
            participant
                .clone()
                .with_initial_state(
                    ParticipantInitialState::new(
                        Hp::new(current_hp).unwrap(),
                        combatant.maximum_hp(),
                        combatant.current_energy(),
                        combatant.maximum_energy(),
                        starclock_combat::LifeState::Alive,
                        starclock_combat::PresenceState::Present,
                    )
                    .unwrap(),
                )
                .unwrap()
        })
        .collect();
    BattleSpec::new(
        AssemblyDigest::new([marker; 32]).unwrap(),
        original.encounter(),
        participants,
        original.resources(TeamSide::Player).clone(),
        original.resources(TeamSide::Enemy).clone(),
        original.concede_policy(),
    )
    .unwrap()
}

fn assert_maximum_hp_heal<'a>(
    combat: &'a starclock_combat::catalog::CombatCatalog,
    contributions: &UniverseBattleContributionSet,
    key: &str,
    point: RuleEventPoint,
    ratio: i64,
) -> &'a starclock_combat::catalog::definition::RuleDefinition {
    let rule = combat.rule(binding(contributions, key).rule()).unwrap();
    assert_eq!(rule.runtime().unwrap().triggers()[0].event_point, point);
    let amount = rule_steps(combat, rule.id())
        .into_iter()
        .find_map(|step| match step {
            ProgramStep::Operation(RuleOperationTemplate::Heal {
                amount,
                apply_formula_modifiers: true,
                ..
            }) => Some(amount),
            _ => None,
        })
        .unwrap();
    assert!(expression_has_stat(amount, StatKind::Hp));
    assert!(expression_has_scalar(amount, ratio));
    rule
}

fn source_heals(
    resolution: &starclock_combat::Resolution,
    source: starclock_combat::SourceDefinitionId,
) -> Vec<i64> {
    resolution
        .events()
        .iter()
        .filter_map(|event| match event.kind() {
            BattleEventKind::Heal(data) if event.cause().source_definition() == Some(source) => {
                Some(data.calculated.get())
            }
            _ => None,
        })
        .collect()
}

fn binding<'a>(
    contributions: &'a UniverseBattleContributionSet,
    key: &str,
) -> &'a starclock_mode_universe::battle_contribution::UniverseBattleRuleBinding {
    contributions
        .rules()
        .iter()
        .find(|binding| binding.source_binding_key() == Some(key))
        .unwrap()
}

fn rule_steps(
    combat: &starclock_combat::catalog::CombatCatalog,
    rule: starclock_combat::RuleId,
) -> Vec<&ProgramStep> {
    combat
        .rule(rule)
        .unwrap()
        .programs()
        .iter()
        .filter_map(|program| combat.program(*program))
        .flat_map(|program| program.steps())
        .collect()
}

fn first_effect(
    combat: &starclock_combat::catalog::CombatCatalog,
    rule: starclock_combat::RuleId,
) -> &starclock_combat::catalog::definition::EffectDefinition {
    combat
        .rule(rule)
        .unwrap()
        .programs()
        .iter()
        .filter_map(|program| combat.program(*program))
        .flat_map(|program| program.effects())
        .find_map(|effect| combat.effect(*effect))
        .unwrap()
}

fn first_modifier(
    combat: &starclock_combat::catalog::CombatCatalog,
    rule: starclock_combat::RuleId,
) -> &starclock_combat::modifier::model::ModifierDefinition {
    let effect = first_effect(combat, rule);
    combat.modifier(effect.modifiers()[0]).unwrap()
}

fn literal_scalar(value: &ValueExpr) -> i64 {
    match value {
        ValueExpr::Literal(RuleValue::Scalar(value)) => value.scaled(),
        _ => panic!("expected literal scalar: {value:?}"),
    }
}

fn expression_has_scalar(value: &ValueExpr, expected: i64) -> bool {
    match value {
        ValueExpr::Literal(RuleValue::Scalar(value)) => value.scaled() == expected,
        ValueExpr::Add(lhs, rhs)
        | ValueExpr::Subtract(lhs, rhs)
        | ValueExpr::Minimum(lhs, rhs)
        | ValueExpr::Maximum(lhs, rhs) => {
            expression_has_scalar(lhs, expected) || expression_has_scalar(rhs, expected)
        }
        ValueExpr::Multiply { lhs, rhs, .. } | ValueExpr::Divide { lhs, rhs, .. } => {
            expression_has_scalar(lhs, expected) || expression_has_scalar(rhs, expected)
        }
        _ => false,
    }
}

fn expression_has_stat(value: &ValueExpr, expected: StatKind) -> bool {
    match value {
        ValueExpr::QueryStat { stat, .. } => *stat == expected,
        ValueExpr::Add(lhs, rhs)
        | ValueExpr::Subtract(lhs, rhs)
        | ValueExpr::Minimum(lhs, rhs)
        | ValueExpr::Maximum(lhs, rhs) => {
            expression_has_stat(lhs, expected) || expression_has_stat(rhs, expected)
        }
        ValueExpr::Multiply { lhs, rhs, .. } | ValueExpr::Divide { lhs, rhs, .. } => {
            expression_has_stat(lhs, expected) || expression_has_stat(rhs, expected)
        }
        _ => false,
    }
}
