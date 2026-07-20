use std::collections::BTreeMap;

use starclock_combat::{
    ModifierDefinitionId, ModifierInstanceId, ModifierStackingGroupId, Scalar, SourceDefinitionId,
    StateSlotDefinitionId, UnitId,
    modifier::model::{
        ActiveModifier, FormulaPurpose, FormulaStage, ModifierAggregation, ModifierDefinition,
        ModifierFilter, ModifierQueryContext, ModifierStackingGroup, SnapshotPolicy, StatKind,
        StatQuery, StatQuerySubject,
    },
    modifier::registry::ModifierRegistry,
    modifier::resolve::{ModifierQueryError, StatResolver},
    rule::model::{RuleValue, SourceClass, ValueExpr},
};

#[test]
fn full_stat_pipeline_applies_groups_filters_and_stage_cap() {
    let subject = unit(1);
    let groups = (1..=5)
        .map(|id| group(id, ModifierAggregation::Sum))
        .collect();
    let definitions = vec![
        definition(1, 1, FormulaStage::BaseAdd, literal(scalar(10))),
        definition(
            2,
            2,
            FormulaStage::PercentOfBase,
            literal(Scalar::from_scaled(500_000)),
        )
        .with_cap(scalar(150)),
        definition(3, 3, FormulaStage::Flat, literal(scalar(20))),
        definition(4, 4, FormulaStage::FinalAdd, literal(scalar(5))),
        definition(5, 5, FormulaStage::FinalMultiply, literal(scalar(2)))
            .with_filter(ModifierFilter::AbilityTag("skill".into())),
    ];
    let registry = ModifierRegistry::new(groups, definitions).unwrap();
    let instances = (1..=5)
        .map(|id| instance(id, u32::try_from(id).unwrap(), subject, id))
        .collect::<Vec<_>>();
    let bases = BTreeMap::from([((subject, StatKind::Atk), scalar(100))]);
    let query = stat_query(subject, StatKind::Atk);

    let without_tag = StatResolver::new(&registry, &bases, &instances)
        .query(query, &ModifierQueryContext::default())
        .unwrap();
    assert_eq!(without_tag, scalar(175));

    let context = ModifierQueryContext {
        ability_tags: vec!["skill".into()].into_boxed_slice(),
        ..ModifierQueryContext::default()
    };
    let with_tag = StatResolver::new(&registry, &bases, &instances)
        .query(query, &context)
        .unwrap();
    assert_eq!(with_tag, scalar(350));
}

#[test]
fn uncapped_damage_pipeline_modifiers_are_valid_registry_definitions() {
    let definition = definition(
        1,
        1,
        FormulaStage::DamageBoost,
        literal(Scalar::from_scaled(320_000)),
    );
    let registry =
        ModifierRegistry::new(vec![group(1, ModifierAggregation::Sum)], vec![definition])
            .expect("uncapped damage-stage modifiers are valid");
    assert_eq!(registry.len(), 1);
}

#[test]
fn every_stacking_policy_has_a_stable_result() {
    let cases = [
        (ModifierAggregation::Sum, 6),
        (ModifierAggregation::Product, 6),
        (ModifierAggregation::Maximum, 3),
        (ModifierAggregation::Minimum, 1),
        (ModifierAggregation::Latest, 3),
        (ModifierAggregation::Earliest, 1),
        (ModifierAggregation::StrongestByComparator, 3),
        (ModifierAggregation::UniquePerSource, 5),
        (ModifierAggregation::ReplaceGroup, 3),
    ];
    for (policy, expected) in cases {
        let subject = unit(1);
        let slot = StateSlotDefinitionId::new(1).unwrap();
        let registry = ModifierRegistry::new(
            vec![group(1, policy)],
            vec![definition(1, 1, FormulaStage::Flat, ValueExpr::Slot(slot))],
        )
        .unwrap();
        let mut instances = [1, 2, 3].map(|id| {
            let mut value = instance(id, 1, subject, id);
            value.slots =
                vec![(slot, RuleValue::Integer(i64::try_from(id).unwrap()))].into_boxed_slice();
            if policy == ModifierAggregation::UniquePerSource && id == 2 {
                value.source = SourceDefinitionId::new(1).unwrap();
            }
            value
        });
        instances.sort_by_key(|value| value.insertion_sequence);
        let bases = BTreeMap::from([((subject, StatKind::Atk), Scalar::ZERO)]);
        let actual = StatResolver::new(&registry, &bases, &instances)
            .query(
                stat_query(subject, StatKind::Atk),
                &ModifierQueryContext::default(),
            )
            .unwrap();
        assert_eq!(actual, scalar(expected), "policy {policy:?}");
    }
}

#[test]
fn snapshots_are_values_and_cache_is_semantically_optional() {
    let subject = unit(7);
    let slot = StateSlotDefinitionId::new(1).unwrap();
    let mut snapshot = definition(1, 1, FormulaStage::Flat, ValueExpr::Slot(slot));
    snapshot.snapshot = SnapshotPolicy::OnApplication;
    let registry =
        ModifierRegistry::new(vec![group(1, ModifierAggregation::Sum)], vec![snapshot]).unwrap();
    let mut active = instance(1, 1, subject, 1);
    active.slots = vec![(slot, RuleValue::Integer(99))].into_boxed_slice();
    active.captured_value = Some(scalar(12));
    let bases = BTreeMap::from([((subject, StatKind::Atk), scalar(100))]);
    let query = stat_query(subject, StatKind::Atk);
    let cached = StatResolver::new(&registry, &bases, &[active.clone()])
        .query(query, &ModifierQueryContext::default())
        .unwrap();
    let uncached = StatResolver::new(&registry, &bases, &[active])
        .without_cache()
        .query(query, &ModifierQueryContext::default())
        .unwrap();
    assert_eq!(cached, scalar(112));
    assert_eq!(cached, uncached);
}

#[test]
fn partial_snapshot_policies_capture_only_declared_query_subjects() {
    let subject = unit(8);
    let value = ValueExpr::QueryStat {
        subject: StatQuerySubject::Owner,
        stat: StatKind::Def,
        purpose: FormulaPurpose::Stat,
    };
    let mut definition = definition(1, 1, FormulaStage::Flat, value);
    definition.snapshot = SnapshotPolicy::SourceSnapshotTargetDynamic;
    let registry =
        ModifierRegistry::new(vec![group(1, ModifierAggregation::Sum)], vec![definition]).unwrap();
    let mut active = instance(1, 1, subject, 1);
    active.captured_stats =
        vec![(stat_query(subject, StatKind::Def), scalar(50))].into_boxed_slice();
    let bases = BTreeMap::from([
        ((subject, StatKind::Atk), scalar(100)),
        ((subject, StatKind::Def), scalar(99)),
    ]);
    let actual = StatResolver::new(&registry, &bases, &[active])
        .query(
            stat_query(subject, StatKind::Atk),
            &ModifierQueryContext::default(),
        )
        .unwrap();
    assert_eq!(actual, scalar(150));
}

#[test]
fn recursive_stat_queries_report_the_ordered_cycle_path() {
    let subject = unit(9);
    let atk_from_def = ValueExpr::QueryStat {
        subject: StatQuerySubject::CurrentTarget,
        stat: StatKind::Def,
        purpose: FormulaPurpose::Stat,
    };
    let def_from_atk = ValueExpr::QueryStat {
        subject: StatQuerySubject::CurrentTarget,
        stat: StatKind::Atk,
        purpose: FormulaPurpose::Stat,
    };
    let registry = ModifierRegistry::new(
        vec![
            group(1, ModifierAggregation::Sum),
            group(2, ModifierAggregation::Sum),
        ],
        vec![
            definition(1, 1, FormulaStage::Flat, atk_from_def),
            definition_for_stat(2, 2, StatKind::Def, FormulaStage::Flat, def_from_atk),
        ],
    )
    .unwrap();
    let instances = vec![instance(1, 1, subject, 1), instance(2, 2, subject, 2)];
    let bases = BTreeMap::from([
        ((subject, StatKind::Atk), scalar(100)),
        ((subject, StatKind::Def), scalar(50)),
    ]);
    let error = StatResolver::new(&registry, &bases, &instances)
        .query(
            stat_query(subject, StatKind::Atk),
            &ModifierQueryContext::default(),
        )
        .unwrap_err();
    let ModifierQueryError::StatQueryCycle(path) = error else {
        panic!("wrong fault: {error:?}")
    };
    assert_eq!(
        path.as_ref(),
        &[
            stat_query(subject, StatKind::Atk),
            stat_query(subject, StatKind::Def),
            stat_query(subject, StatKind::Atk),
        ]
    );
}

trait DefinitionFixture {
    fn with_cap(self, cap: Scalar) -> Self;
    fn with_filter(self, filter: ModifierFilter) -> Self;
}
impl DefinitionFixture for ModifierDefinition {
    fn with_cap(mut self, cap: Scalar) -> Self {
        self.cap = Some(cap);
        self
    }
    fn with_filter(mut self, filter: ModifierFilter) -> Self {
        self.filters = vec![filter].into_boxed_slice();
        self
    }
}

fn definition(id: u32, group_id: u32, stage: FormulaStage, value: ValueExpr) -> ModifierDefinition {
    definition_for_stat(id, group_id, StatKind::Atk, stage, value)
}
fn definition_for_stat(
    id: u32,
    group_id: u32,
    stat: StatKind,
    stage: FormulaStage,
    value: ValueExpr,
) -> ModifierDefinition {
    ModifierDefinition {
        id: ModifierDefinitionId::new(id).unwrap(),
        stat,
        stage,
        purpose: FormulaPurpose::Stat,
        value,
        stacking_group: ModifierStackingGroupId::new(group_id).unwrap(),
        priority: 0,
        floor: None,
        cap: None,
        cap_stage: stage,
        snapshot: SnapshotPolicy::Dynamic,
        filters: Box::new([]),
    }
}
fn group(id: u32, aggregation: ModifierAggregation) -> ModifierStackingGroup {
    ModifierStackingGroup {
        id: ModifierStackingGroupId::new(id).unwrap(),
        aggregation,
    }
}
fn instance(id: u64, definition: u32, subject: UnitId, sequence: u64) -> ActiveModifier {
    ActiveModifier {
        instance: ModifierInstanceId::new(id).unwrap(),
        definition: ModifierDefinitionId::new(definition).unwrap(),
        owner: subject,
        subject,
        source: SourceDefinitionId::new(u32::try_from(id).unwrap()).unwrap(),
        source_class: SourceClass::Ability,
        insertion_sequence: sequence,
        application_action: None,
        slots: Box::new([]),
        captured_value: None,
        captured_stats: Box::new([]),
    }
}
fn stat_query(subject: UnitId, stat: StatKind) -> StatQuery {
    StatQuery {
        subject,
        stat,
        purpose: FormulaPurpose::Stat,
    }
}
fn literal(value: Scalar) -> ValueExpr {
    ValueExpr::Literal(RuleValue::Scalar(value))
}
fn scalar(value: i64) -> Scalar {
    Scalar::from_scaled(value * 1_000_000)
}
fn unit(id: u64) -> UnitId {
    UnitId::new(id).unwrap()
}
