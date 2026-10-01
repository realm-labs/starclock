//! Real Sora inputs execute in the shared battle engine, without kit coverage.
use super::{lower_operation, program_references};
use crate::{
    generated::{
        SoraConfig, operation, operation_payload::OperationPayload, runtime::SoraBundle,
        stat_kind::StatKind as AuthoredStatKind,
    },
    modifier_lower::{expression, stat},
    selector_lower,
};
use sha2::{Digest, Sha256};
use starclock_combat::{
    AssemblyDigest, Battle, BattleEvent, BattleEventKind, BattlePhase, BattleSeed, BattleSpec,
    CombatantSpecDigest, Command, ConcedePolicy, DecisionId, Energy, FormationIndex, Hp,
    ParticipantSource, ParticipantSpec, ResolvedCombatantSpec, ResolvedDefinitionBindings,
    ResolvedModifierBinding, Scalar, Speed, TeamResourceSpec, TeamSide, UnitLevel,
    catalog::{
        CombatCatalog,
        action::{
            AbilityActionDefinition, AbilityKind, AbilityProgramBinding, AbilityProgramTiming,
            ActionHitDefinition, ActionResourcePolicy, TargetInvalidationPolicy, TargetPattern,
            TargetRelation, UnitTargetSelector,
        },
        builder::CombatCatalogBuilder,
        definition::{
            AbilityDefinition, EncounterDefinition, EnemyDefinition, ProgramDefinition,
            SelectorDefinition, UnitDefinition,
        },
    },
    formula::model::DamageClass,
    modifier::model::{
        FormulaPurpose, FormulaStage, ModifierAggregation, ModifierDefinition,
        ModifierStackingGroup, SnapshotPolicy, StatKind, StatQuerySubject,
    },
    rule::model::{
        ProgramStep, RuleOperationTemplate, RuleSource, RuleValue, SourceClass, ValueExpr,
    },
};
use std::{collections::BTreeSet, sync::Arc};

const PRODUCTION: &[u8] = include_bytes!("../../../../config/generated/config.sora");

fn definition<I: TryFrom<u32>>(value: u32) -> I
where
    I::Error: core::fmt::Debug,
{
    I::try_from(value).unwrap()
}

fn config() -> SoraConfig {
    let bundle = SoraBundle::parse(PRODUCTION).unwrap();
    SoraConfig::from_source(&bundle).unwrap()
}

fn action() -> AbilityActionDefinition {
    AbilityActionDefinition::new(
        AbilityKind::Basic,
        1,
        TargetInvalidationPolicy::KeepIfPresent,
        ActionResourcePolicy::new(0, 0, Energy::ZERO, Energy::ZERO),
    )
    .unwrap()
    .with_hits(vec![ActionHitDefinition::new(vec![])])
    .unwrap()
}

fn catalog(config: &SoraConfig, row: &operation::Operation) -> Arc<CombatCatalog> {
    let operation = lower_operation(config, row, &BTreeSet::new()).unwrap();
    let steps = [ProgramStep::Operation(operation)];
    let (selectors, effects) = program_references(&steps);
    assert_eq!(selectors.as_ref(), &[definition(24051)]);
    assert!(effects.is_empty());
    let mut builder = CombatCatalogBuilder::new(Sha256::digest(PRODUCTION).into());
    let owner = selector_lower::lower(config, config.selector().get(&24051).unwrap()).unwrap();
    builder.add_selector(SelectorDefinition::new(owner.id).with_rule_units(owner.units));
    for id in [1, 2] {
        builder.add_selector(SelectorDefinition::new(definition(id)).with_unit_targets(
            UnitTargetSelector::new(TargetRelation::Opposing, TargetPattern::Single).unwrap(),
        ));
        builder.add_program(
            ProgramDefinition::new(
                definition(id),
                vec![],
                if id == 1 { selectors.to_vec() } else { vec![] },
                vec![],
                vec![],
            )
            .with_steps(if id == 1 { steps.to_vec() } else { vec![] }),
        );
        let ability =
            AbilityDefinition::new(definition(id), definition(id), definition(id), vec![])
                .with_action(action());
        builder.add_ability(if id == 1 {
            ability.with_programs(vec![
                AbilityProgramBinding::new(1, AbilityProgramTiming::AfterHits, definition(id))
                    .unwrap(),
            ])
        } else {
            ability
        });
        builder.add_unit(UnitDefinition::new(
            definition(id),
            vec![definition(id)],
            vec![],
        ));
    }
    // The addition is a native fixture binding, not an authored released base.
    builder.add_modifier_group(ModifierStackingGroup {
        id: definition(1),
        aggregation: ModifierAggregation::Sum,
        comparator: None,
    });
    builder.add_modifier(ModifierDefinition {
        id: definition(1),
        stat: StatKind::Elation,
        stage: FormulaStage::Flat,
        purpose: FormulaPurpose::Stat,
        value: expression(config, 970112, &mut BTreeSet::new()).unwrap(),
        stacking_group: definition(1),
        priority: 0,
        floor: None,
        cap: None,
        cap_stage: FormulaStage::Flat,
        snapshot: SnapshotPolicy::Dynamic,
        source_stack_slot: None,
        filters: Box::new([]),
    });
    builder.add_enemy(EnemyDefinition::new(
        definition(1),
        definition(2),
        vec![definition(2)],
    ));
    builder.add_encounter(EncounterDefinition::new(
        definition(1),
        vec![definition(1)],
        vec![],
    ));
    builder.build().unwrap()
}

fn battle(catalog: Arc<CombatCatalog>, addition: bool) -> Battle {
    let participants = [(1, TeamSide::Player), (2, TeamSide::Enemy)].map(|(id, side)| {
        let bound = id == 1 && addition;
        let mut spec = ResolvedCombatantSpec::new(
            definition(id),
            UnitLevel::new(80).unwrap(),
            Hp::new(1000).unwrap(),
            Speed::from_scaled(if id == 1 { 200_000_000 } else { 100_000_000 }).unwrap(),
            ResolvedDefinitionBindings::new(
                vec![definition(id)],
                vec![],
                if bound { vec![definition(1)] } else { vec![] },
            )
            .unwrap(),
            CombatantSpecDigest::new([u8::try_from(id).unwrap(); 32]).unwrap(),
        )
        .unwrap();
        if bound {
            spec = spec
                .with_sources(vec![RuleSource::new(
                    definition(1),
                    SourceClass::Synthetic,
                    vec![],
                    [0xef; 32],
                )])
                .unwrap()
                .with_modifier_bindings(vec![ResolvedModifierBinding::new(
                    definition(1),
                    definition(1),
                )])
                .unwrap();
        }
        ParticipantSpec::new(
            side,
            FormationIndex::new(0).unwrap(),
            if id == 1 {
                ParticipantSource::Player
            } else {
                ParticipantSource::EncounterEnemy(definition(1))
            },
            spec,
        )
    });
    let spec = BattleSpec::new(
        AssemblyDigest::new([0xee; 32]).unwrap(),
        definition(1),
        participants.to_vec(),
        TeamResourceSpec::new(3, 5).unwrap(),
        TeamResourceSpec::new(0, 0).unwrap(),
        ConcedePolicy::Allowed,
    )
    .unwrap();
    Battle::create(catalog, spec, BattleSeed::new([0xed; 32])).unwrap()
}

fn apply(running: &mut Battle, fresh: &mut Battle, command: Command) -> Vec<BattleEvent> {
    let actual = running.apply(command.clone()).unwrap();
    let replay = fresh.apply(command).unwrap();
    assert_eq!(actual.events(), replay.events());
    assert_eq!(actual.fault(), replay.fault());
    assert_eq!(running.state_hash(), fresh.state_hash());
    assert_eq!(
        running.view().rng_draw_count(),
        fresh.view().rng_draw_count()
    );
    actual.events().to_vec()
}

fn execute(catalog: Arc<CombatCatalog>, addition: bool) -> (Battle, Vec<BattleEvent>) {
    let mut running = battle(Arc::clone(&catalog), addition);
    let mut fresh = battle(catalog, addition);
    let before = running.state_hash();
    assert!(
        running
            .apply(Command::StartBattle {
                decision: DecisionId::new(900).unwrap()
            })
            .is_err()
    );
    assert_eq!(running.state_hash(), before);
    let start = Command::StartBattle {
        decision: running.decision().unwrap().id(),
    };
    apply(&mut running, &mut fresh, start);
    for _ in 0..8 {
        if let Some(advance) = running.advance_command() {
            apply(&mut running, &mut fresh, advance);
            continue;
        }
        let offered = running.decision().unwrap().legal_commands();
        if let Some(command) = offered
            .iter()
            .find(|c| matches!(c, Command::UseAbility { .. }))
        {
            let command = command.clone();
            let events = apply(&mut running, &mut fresh, command);
            return (running, events);
        }
        panic!("fixture must offer its basic ability after the boundary")
    }
    panic!("fixture did not reach its bounded player decision")
}

#[test]
fn production_elation_sora_inputs_execute_exact_damage_and_live_property_queries() {
    let config = config();
    let row = config.operation().get(&970201).unwrap();
    let lowered = lower_operation(&config, row, &BTreeSet::new()).unwrap();
    let RuleOperationTemplate::ElationDamage { inputs, .. } = lowered else {
        panic!("dedicated Sora payload must remain dedicated")
    };
    assert_eq!(
        inputs.base_damage,
        ValueExpr::Literal(RuleValue::Scalar(Scalar::from_scaled(100_900_000),))
    );
    assert_eq!(
        expression(&config, 970110, &mut BTreeSet::new()).unwrap(),
        ValueExpr::QueryStat {
            subject: StatQuerySubject::Owner,
            stat: StatKind::Elation,
            purpose: FormulaPurpose::Stat,
        }
    );
    assert_eq!(stat(AuthoredStatKind::Elation), StatKind::Elation);
    for (addition, raw, applied) in [(false, 47_221_200, 47), (true, 106_247_700, 106)] {
        let (running, events) = execute(catalog(&config, row), addition);
        let damage = events
            .iter()
            .filter_map(|event| match event.kind() {
                BattleEventKind::Damage(data) => Some(data),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(damage.len(), 1);
        assert_eq!(damage[0].class, DamageClass::Elation);
        assert_eq!(
            (damage[0].raw.scaled(), damage[0].calculated.get()),
            (raw, applied)
        );
        assert_eq!(running.view().rng_draw_count(), 0);
    }
}

#[test]
fn production_elation_non_scalar_input_faults_without_damage_or_rng() {
    let config = config();
    for index in 0..9 {
        let mut row = config.operation().get(&970201).unwrap().clone();
        let OperationPayload::ElationDamage {
            base_damage_expression_id,
            original_multiplier_expression_id,
            meter_multiplier_expression_id,
            merrymaking_expression_id,
            target_resistance_expression_id,
            penetration_expression_id,
            resistance_minimum_expression_id,
            resistance_maximum_expression_id,
            unbroken_multiplier_expression_id,
            ..
        } = &mut row.payload
        else {
            panic!("dedicated authored payload")
        };
        [
            base_damage_expression_id,
            original_multiplier_expression_id,
            meter_multiplier_expression_id,
            merrymaking_expression_id,
            target_resistance_expression_id,
            penetration_expression_id,
            resistance_minimum_expression_id,
            resistance_maximum_expression_id,
            unbroken_multiplier_expression_id,
        ][index]
            .clone_from(&970113);
        let (running, events) = execute(catalog(&config, &row), false);
        assert_eq!(running.view().phase(), BattlePhase::Faulted);
        assert!(
            events
                .iter()
                .all(|event| !matches!(event.kind(), BattleEventKind::Damage(_)))
        );
        assert!(
            running
                .view()
                .units_by_id()
                .all(|unit| unit.current_hp().get() == 1000)
        );
        assert_eq!(running.view().rng_draw_count(), 0);
    }
}
