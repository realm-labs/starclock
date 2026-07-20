use std::sync::Arc;

use starclock_build::{
    ability::{AbilityInvestment, AbilityLevel, AbilityLevelRow, AbilityLevelTable},
    catalog::{
        BuildCatalog, BuildCatalogBuilder, BuildCatalogErrorKind, BuildCatalogRevision,
        CharacterBuildDefinition, CharacterStatRow,
    },
    compiler::{BuildCompileErrorKind, LoadoutCompiler},
    eidolon::{EidolonDefinition, EidolonSetDefinition},
    id::{EidolonDefinitionId, TraceNodeId},
    light_cone::CombatPath,
    patch::BuildPatch,
    spec::{CombatantBuildSpec, EidolonLevel, PromotionStage},
    trace::{TraceGraphDefinition, TraceNodeDefinition},
};
use starclock_combat::{
    AbilityId, Energy, Hp, ModifierDefinitionId, ModifierStackingGroupId,
    ResolvedDefinitionBindings, RuleBundleId, Scalar, SourceDefinitionId, Speed, UnitDefinitionId,
    UnitLevel,
    catalog::{
        CombatCatalog,
        action::{
            AbilityActionDefinition, AbilityKind, ActionHitDefinition, ActionResourcePolicy,
            TargetInvalidationPolicy, TargetPattern, TargetRelation, UnitTargetSelector,
        },
        builder::CombatCatalogBuilder,
        definition::{
            AbilityDefinition, ProgramDefinition, RuleBundle, RuleDefinition, SelectorDefinition,
            UnitDefinition,
        },
    },
    modifier::model::{
        FormulaPurpose, FormulaStage, ModifierAggregation, ModifierDefinition,
        ModifierStackingGroup, SnapshotPolicy, StatKind,
    },
    rule::model::{RuleSource, RuleValue, SourceClass, ValueExpr},
};

#[test]
fn ability_and_trace_input_order_produces_one_canonical_result() {
    let combat = combat_catalog();
    let catalog = build_catalog(&combat, false);
    let reversed_catalog = build_catalog(&combat, true);
    let first = build_spec(vec![20, 10], true);
    let second = build_spec(vec![10, 20], false);

    let first = LoadoutCompiler.compile(&catalog, &combat, &first).unwrap();
    let second = LoadoutCompiler
        .compile(&reversed_catalog, &combat, &second)
        .unwrap();

    assert_eq!(first, second);
    assert_eq!(
        first.combatant().abilities(),
        &[ability(3), ability(4), ability(5)]
    );
    assert_eq!(first.combatant().rule_bundles(), &[rule_bundle(1)]);
    assert_eq!(first.combatant().modifiers(), &[modifier(1)]);
}

#[test]
fn trace_prerequisites_promotion_and_ability_caps_are_exact_errors() {
    let combat = combat_catalog();
    let catalog = build_catalog(&combat, false);

    let missing_prerequisite = build_spec(vec![20], false);
    let error = LoadoutCompiler
        .compile(&catalog, &combat, &missing_prerequisite)
        .unwrap_err();
    assert_eq!(error.kind(), BuildCompileErrorKind::InvalidTraceSelection);

    let low_promotion = CombatantBuildSpec::new(form(1), UnitLevel::new(20).unwrap(), promotion(1))
        .with_ability_levels(investments(false, 2))
        .unwrap()
        .with_traces(vec![trace(10), trace(20)])
        .unwrap();
    let error = LoadoutCompiler
        .compile(&catalog, &combat, &low_promotion)
        .unwrap_err();
    assert_eq!(error.kind(), BuildCompileErrorKind::InvalidTraceSelection);

    let overinvested = CombatantBuildSpec::new(
        form(1),
        UnitLevel::new(80).unwrap(),
        PromotionStage::new(6).unwrap(),
    )
    .with_ability_levels(investments(false, 3))
    .unwrap();
    let error = LoadoutCompiler
        .compile(&catalog, &combat, &overinvested)
        .unwrap_err();
    assert_eq!(error.kind(), BuildCompileErrorKind::InvalidAbilitySelection);
}

#[test]
fn catalog_rejects_incomplete_curves_and_trace_cycles_before_compilation() {
    let combat = combat_catalog();
    let mut incomplete = base_character();
    incomplete = incomplete.with_ability_levels(vec![AbilityLevelTable::new(
        ability(1),
        level(2),
        vec![
            AbilityLevelRow::new(level(1), ability(1)),
            AbilityLevelRow::new(level(3), ability(3)),
        ],
    )]);
    let mut builder = build_builder();
    builder.add_character(incomplete);
    assert_eq!(
        builder.build(&combat).unwrap_err().kind(),
        BuildCatalogErrorKind::InvalidAbilityCurve
    );

    let cycle = TraceGraphDefinition::new(
        form(1),
        vec![
            TraceNodeDefinition::new(
                trace(10),
                source(110, SourceClass::Progression),
                vec![trace(20)],
                promotion(2),
                vec![],
            ),
            TraceNodeDefinition::new(
                trace(20),
                source(120, SourceClass::Progression),
                vec![trace(10)],
                promotion(2),
                vec![],
            ),
        ],
    );
    let mut builder = build_builder();
    builder.add_character(base_character().with_trace_graph(cycle));
    assert_eq!(
        builder.build(&combat).unwrap_err().kind(),
        BuildCatalogErrorKind::InvalidTraceGraph
    );
}

#[test]
fn normalized_build_input_rejects_duplicate_families_and_traces() {
    let base = CombatantBuildSpec::new(
        form(1),
        UnitLevel::new(80).unwrap(),
        PromotionStage::new(6).unwrap(),
    );
    assert!(
        base.clone()
            .with_ability_levels(vec![
                AbilityInvestment::new(ability(1), level(1)),
                AbilityInvestment::new(ability(1), level(2)),
            ])
            .is_err()
    );
    assert!(base.with_traces(vec![trace(10), trace(10)]).is_err());
}

fn build_catalog(combat: &CombatCatalog, reverse: bool) -> BuildCatalog {
    let mut tables = vec![
        AbilityLevelTable::new(
            ability(1),
            level(2),
            vec![
                AbilityLevelRow::new(level(3), ability(3)),
                AbilityLevelRow::new(level(1), ability(1)),
                AbilityLevelRow::new(level(2), ability(2)),
            ],
        ),
        AbilityLevelTable::new(
            ability(4),
            level(1),
            vec![AbilityLevelRow::new(level(1), ability(4))],
        ),
    ];
    if reverse {
        tables.reverse();
    }
    let mut nodes = vec![
        TraceNodeDefinition::new(
            trace(20),
            source(120, SourceClass::Progression),
            vec![trace(10)],
            promotion(2),
            vec![BuildPatch::AdjustAbilityLevel {
                family: ability(1),
                bonus: 1,
                cap_delta: 1,
            }],
        ),
        TraceNodeDefinition::new(
            trace(10),
            source(110, SourceClass::Progression),
            vec![],
            promotion(1),
            vec![
                BuildPatch::AddRuleBundle(rule_bundle(1)),
                BuildPatch::AddModifier(modifier(1)),
                BuildPatch::AddAbility(ability(5)),
            ],
        ),
    ];
    if reverse {
        nodes.reverse();
    }
    let character = base_character()
        .with_ability_levels(tables)
        .with_trace_graph(TraceGraphDefinition::new(form(1), nodes));
    let mut builder = build_builder();
    builder.add_character(character);
    builder.build(combat).unwrap()
}

fn base_character() -> CharacterBuildDefinition {
    CharacterBuildDefinition::new(
        form(1),
        CombatPath::Harmony,
        source(100, SourceClass::Unit),
        CharacterStatRow::new(
            UnitLevel::new(80).unwrap(),
            promotion(6),
            Hp::new(10_000).unwrap(),
            Speed::from_scaled(100_000_000).unwrap(),
        ),
        ResolvedDefinitionBindings::new(vec![ability(1), ability(4)], vec![], vec![]).unwrap(),
    )
    .with_stat_rows(vec![
        CharacterStatRow::new(
            UnitLevel::new(80).unwrap(),
            promotion(6),
            Hp::new(10_000).unwrap(),
            Speed::from_scaled(100_000_000).unwrap(),
        ),
        CharacterStatRow::new(
            UnitLevel::new(20).unwrap(),
            promotion(1),
            Hp::new(2_500).unwrap(),
            Speed::from_scaled(100_000_000).unwrap(),
        ),
    ])
    .with_eidolons(empty_eidolons())
}

fn empty_eidolons() -> EidolonSetDefinition {
    EidolonSetDefinition::new(
        form(1),
        (1..=6)
            .map(|rank| {
                EidolonDefinition::new(
                    EidolonDefinitionId::new(rank).unwrap(),
                    source(200 + rank, SourceClass::Progression),
                    EidolonLevel::new(u8::try_from(rank).unwrap()).unwrap(),
                    vec![],
                )
            })
            .collect(),
    )
}

fn build_builder() -> BuildCatalogBuilder {
    BuildCatalogBuilder::new(
        BuildCatalogRevision::new("build-b2-v1").unwrap(),
        "combat-build-b2-v1",
    )
    .unwrap()
}

fn build_spec(trace_ids: Vec<u32>, reverse_investments: bool) -> CombatantBuildSpec {
    CombatantBuildSpec::new(form(1), UnitLevel::new(80).unwrap(), promotion(6))
        .with_ability_levels(investments(reverse_investments, 2))
        .unwrap()
        .with_traces(trace_ids.into_iter().map(trace).collect())
        .unwrap()
}

fn investments(reverse: bool, first_level: u8) -> Vec<AbilityInvestment> {
    let mut values = vec![
        AbilityInvestment::new(ability(1), level(first_level)),
        AbilityInvestment::new(ability(4), level(1)),
    ];
    if reverse {
        values.reverse();
    }
    values
}

fn combat_catalog() -> Arc<CombatCatalog> {
    let mut builder = CombatCatalogBuilder::new("combat-build-b2-v1", [0xb2; 32]);
    for raw in 1..=5 {
        builder.add_selector(SelectorDefinition::new(definition(raw)).with_unit_targets(
            UnitTargetSelector::new(TargetRelation::SelfUnit, TargetPattern::Single).unwrap(),
        ));
        builder.add_program(ProgramDefinition::new(
            definition(raw),
            vec![],
            vec![definition(raw)],
            vec![],
            vec![],
        ));
        builder.add_ability(
            AbilityDefinition::new(ability(raw), definition(raw), definition(raw), vec![])
                .with_action(basic_action()),
        );
    }
    builder.add_rule(RuleDefinition::new(
        definition(1),
        vec![definition(1)],
        vec![definition(1)],
    ));
    builder.add_rule_bundle(RuleBundle::new(rule_bundle(1), vec![definition(1)]));
    builder.add_modifier_group(ModifierStackingGroup {
        id: definition(1),
        aggregation: ModifierAggregation::Sum,
    });
    builder.add_modifier(ModifierDefinition {
        id: modifier(1),
        stat: StatKind::Atk,
        stage: FormulaStage::Flat,
        purpose: FormulaPurpose::Stat,
        value: ValueExpr::Literal(RuleValue::Scalar(Scalar::ZERO)),
        stacking_group: ModifierStackingGroupId::new(1).unwrap(),
        priority: 0,
        floor: None,
        cap: None,
        cap_stage: FormulaStage::Flat,
        snapshot: SnapshotPolicy::Dynamic,
        filters: Box::new([]),
    });
    builder.add_unit(UnitDefinition::new(
        form(1),
        (1..=5).map(ability).collect(),
        vec![rule_bundle(1)],
    ));
    builder.build().unwrap()
}

fn basic_action() -> AbilityActionDefinition {
    AbilityActionDefinition::new(
        AbilityKind::Basic,
        1,
        TargetInvalidationPolicy::CancelRemainingForTarget,
        ActionResourcePolicy::new(0, 0, Energy::ZERO, Energy::ZERO),
    )
    .unwrap()
    .with_hits(vec![ActionHitDefinition::new(vec![])])
    .unwrap()
}

fn level(raw: u8) -> AbilityLevel {
    AbilityLevel::new(raw).unwrap()
}

fn promotion(raw: u8) -> PromotionStage {
    PromotionStage::new(raw).unwrap()
}

fn trace(raw: u32) -> TraceNodeId {
    TraceNodeId::new(raw).unwrap()
}

fn form(raw: u32) -> UnitDefinitionId {
    UnitDefinitionId::new(raw).unwrap()
}

fn ability(raw: u32) -> AbilityId {
    AbilityId::new(raw).unwrap()
}

fn rule_bundle(raw: u32) -> RuleBundleId {
    RuleBundleId::new(raw).unwrap()
}

fn modifier(raw: u32) -> ModifierDefinitionId {
    ModifierDefinitionId::new(raw).unwrap()
}

fn source(raw: u32, class: SourceClass) -> RuleSource {
    RuleSource::new(
        SourceDefinitionId::new(raw).unwrap(),
        class,
        vec![],
        [u8::try_from(raw).unwrap_or(0x7f); 32],
    )
}

fn definition<I: TryFrom<u32>>(raw: u32) -> I
where
    I::Error: core::fmt::Debug,
{
    I::try_from(raw).unwrap()
}
