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
    patch::BuildPatch,
    report::BuildValidationStage,
    spec::{CombatantBuildSpec, EidolonLevel, PromotionStage},
    trace::{TraceGraphDefinition, TraceNodeDefinition},
};
use starclock_combat::{
    AbilityId, CombatantSpecDigest, Energy, Hp, ModifierDefinitionId, ModifierStackingGroupId,
    ResolvedDefinitionBindings, RuleBundleId, Scalar, Speed, UnitDefinitionId, UnitLevel,
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
    rule::model::{RuleValue, ValueExpr},
};

#[test]
fn e0_and_e6_compile_through_exact_rank_order() {
    let combat = combat_catalog();
    let catalog = build_catalog(&combat, false, false);
    let reversed = build_catalog(&combat, true, false);

    let e0 = LoadoutCompiler
        .compile(&catalog, &combat, &build_spec(0, false))
        .unwrap();
    assert_eq!(e0.combatant().abilities(), &[ability(2), ability(5)]);
    assert_eq!(e0.combatant().rule_bundles(), &[rule_bundle(1)]);
    assert_eq!(e0.combatant().modifiers(), &[modifier(1)]);

    let e3 = LoadoutCompiler
        .compile(&catalog, &combat, &build_spec(3, false))
        .unwrap();
    assert_eq!(e3.combatant().abilities(), &[ability(3), ability(6)]);
    assert_eq!(e3.combatant().rule_bundles(), &[rule_bundle(2)]);
    assert_eq!(e3.combatant().modifiers(), &[modifier(1)]);

    let e6 = LoadoutCompiler
        .compile(&catalog, &combat, &build_spec(6, false))
        .unwrap();
    let reordered = LoadoutCompiler
        .compile(&reversed, &combat, &build_spec(6, false))
        .unwrap();
    assert_eq!(e6, reordered);
    assert_eq!(
        e6.combatant().abilities(),
        &[ability(4), ability(6), ability(7)]
    );
    assert_eq!(e6.combatant().rule_bundles(), &[rule_bundle(2)]);
    assert_eq!(e6.combatant().modifiers(), &[modifier(1), modifier(2)]);
    assert_eq!(
        e6.report().entries()[5].stage(),
        BuildValidationStage::EidolonSelection
    );
}

#[test]
fn catalog_requires_one_canonical_e1_through_e6_set() {
    let combat = combat_catalog();
    assert!(EidolonLevel::new(7).is_none());
    let mut incomplete = base_character();
    incomplete = incomplete.with_eidolons(EidolonSetDefinition::new(
        form(1),
        (1..=5).map(|rank| eidolon(rank, vec![])).collect(),
    ));
    assert_catalog_error(
        &combat,
        incomplete,
        BuildCatalogErrorKind::IncompleteEidolonSet,
    );

    let duplicate_id = EidolonSetDefinition::new(
        form(1),
        (1..=6)
            .map(|rank| EidolonDefinition::new(eidolon_id(1), eidolon_level(rank), vec![]))
            .collect(),
    );
    assert_catalog_error(
        &combat,
        base_character().with_eidolons(duplicate_id),
        BuildCatalogErrorKind::InvalidEidolonSet,
    );

    assert_catalog_error(
        &combat,
        base_character().with_eidolons(EidolonSetDefinition::new(
            form(2),
            (1..=6).map(|rank| eidolon(rank, vec![])).collect(),
        )),
        BuildCatalogErrorKind::InvalidEidolonSet,
    );
}

#[test]
fn replacement_and_binding_conflicts_fail_before_output() {
    let combat = combat_catalog();
    let conflicting = complete_eidolons(vec![BuildPatch::ReplaceAbility {
        old: ability(5),
        new: ability(1),
    }]);
    assert_catalog_error(
        &combat,
        base_character().with_eidolons(conflicting),
        BuildCatalogErrorKind::InvalidEidolonPatch,
    );

    let missing_target = complete_eidolons(vec![BuildPatch::ReplaceAbility {
        old: ability(7),
        new: ability(6),
    }]);
    assert_catalog_error(
        &combat,
        base_character().with_eidolons(missing_target),
        BuildCatalogErrorKind::InvalidEidolonPatch,
    );

    let catalog = build_catalog(&combat, false, true);
    let error = LoadoutCompiler
        .compile(&catalog, &combat, &build_spec(6, true))
        .unwrap_err();
    assert_eq!(error.kind(), BuildCompileErrorKind::InvalidEidolonSelection);
    assert_eq!(
        error.report().entries().last().unwrap().stage(),
        BuildValidationStage::EidolonSelection
    );
}

fn build_catalog(combat: &CombatCatalog, reverse: bool, add_trace: bool) -> BuildCatalog {
    let mut ranks = vec![
        eidolon(
            1,
            vec![
                BuildPatch::RemoveRuleBundle(rule_bundle(1)),
                BuildPatch::AddRuleBundle(rule_bundle(2)),
            ],
        ),
        eidolon(
            2,
            vec![BuildPatch::ReplaceAbility {
                old: ability(5),
                new: ability(6),
            }],
        ),
        eidolon(
            3,
            vec![BuildPatch::AdjustAbilityLevel {
                family: ability(1),
                bonus: 1,
                cap_delta: 1,
            }],
        ),
        eidolon(4, vec![BuildPatch::AddModifier(modifier(2))]),
        eidolon(
            5,
            vec![BuildPatch::AdjustAbilityLevel {
                family: ability(1),
                bonus: 1,
                cap_delta: 1,
            }],
        ),
        eidolon(6, vec![BuildPatch::AddAbility(ability(7))]),
    ];
    if reverse {
        ranks.reverse();
    }
    let mut character = base_character().with_eidolons(EidolonSetDefinition::new(form(1), ranks));
    if add_trace {
        character = character.with_trace_graph(TraceGraphDefinition::new(
            form(1),
            vec![TraceNodeDefinition::new(
                trace(1),
                vec![],
                promotion(0),
                vec![BuildPatch::AddAbility(ability(7))],
            )],
        ));
    }
    let mut builder = build_builder();
    builder.add_character(character);
    builder.build(combat).unwrap()
}

fn base_character() -> CharacterBuildDefinition {
    CharacterBuildDefinition::new(
        form(1),
        CharacterStatRow::new(
            UnitLevel::new(80).unwrap(),
            promotion(6),
            Hp::new(10_000).unwrap(),
            Speed::from_scaled(100_000_000).unwrap(),
        ),
        ResolvedDefinitionBindings::new(
            vec![ability(1), ability(5)],
            vec![rule_bundle(1)],
            vec![modifier(1)],
        )
        .unwrap(),
        CombatantSpecDigest::new([0xe6; 32]).unwrap(),
    )
    .with_ability_levels(vec![AbilityLevelTable::new(
        ability(1),
        level(2),
        (1..=4)
            .map(|rank| AbilityLevelRow::new(level(rank), ability(u32::from(rank))))
            .collect(),
    )])
}

fn complete_eidolons(first_rank_patches: Vec<BuildPatch>) -> EidolonSetDefinition {
    EidolonSetDefinition::new(
        form(1),
        (1..=6)
            .map(|rank| {
                eidolon(
                    rank,
                    if rank == 1 {
                        first_rank_patches.clone()
                    } else {
                        vec![]
                    },
                )
            })
            .collect(),
    )
}

fn assert_catalog_error(
    combat: &CombatCatalog,
    character: CharacterBuildDefinition,
    expected: BuildCatalogErrorKind,
) {
    let mut builder = build_builder();
    builder.add_character(character);
    assert_eq!(builder.build(combat).unwrap_err().kind(), expected);
}

fn build_builder() -> BuildCatalogBuilder {
    BuildCatalogBuilder::new(
        BuildCatalogRevision::new("build-b3-v1").unwrap(),
        "combat-build-b3-v1",
    )
    .unwrap()
}

fn build_spec(eidolon: u8, trace_selected: bool) -> CombatantBuildSpec {
    let spec = CombatantBuildSpec::new(form(1), UnitLevel::new(80).unwrap(), promotion(6))
        .with_ability_levels(vec![AbilityInvestment::new(ability(1), level(2))])
        .unwrap()
        .with_eidolon(eidolon_level(eidolon));
    if trace_selected {
        spec.with_traces(vec![trace(1)]).unwrap()
    } else {
        spec
    }
}

fn combat_catalog() -> Arc<CombatCatalog> {
    let mut builder = CombatCatalogBuilder::new("combat-build-b3-v1", [0xe6; 32]);
    for raw in 1..=7 {
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
    for raw in 1..=2 {
        builder.add_rule(RuleDefinition::new(
            definition(raw),
            vec![definition(raw)],
            vec![definition(raw)],
        ));
        builder.add_rule_bundle(RuleBundle::new(rule_bundle(raw), vec![definition(raw)]));
        builder.add_modifier_group(ModifierStackingGroup {
            id: definition(raw),
            aggregation: ModifierAggregation::Sum,
        });
        builder.add_modifier(ModifierDefinition {
            id: modifier(raw),
            stat: StatKind::Atk,
            stage: FormulaStage::Flat,
            purpose: FormulaPurpose::Stat,
            value: ValueExpr::Literal(RuleValue::Scalar(Scalar::ZERO)),
            stacking_group: ModifierStackingGroupId::new(raw).unwrap(),
            priority: 0,
            floor: None,
            cap: None,
            cap_stage: FormulaStage::Flat,
            snapshot: SnapshotPolicy::Dynamic,
            filters: Box::new([]),
        });
    }
    builder.add_unit(UnitDefinition::new(
        form(1),
        (1..=7).map(ability).collect(),
        vec![rule_bundle(1), rule_bundle(2)],
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

fn eidolon(rank: u8, patches: Vec<BuildPatch>) -> EidolonDefinition {
    EidolonDefinition::new(eidolon_id(u32::from(rank)), eidolon_level(rank), patches)
}

fn level(raw: u8) -> AbilityLevel {
    AbilityLevel::new(raw).unwrap()
}
fn promotion(raw: u8) -> PromotionStage {
    PromotionStage::new(raw).unwrap()
}
fn eidolon_level(raw: u8) -> EidolonLevel {
    EidolonLevel::new(raw).unwrap()
}
fn trace(raw: u32) -> TraceNodeId {
    TraceNodeId::new(raw).unwrap()
}
fn eidolon_id(raw: u32) -> EidolonDefinitionId {
    EidolonDefinitionId::new(raw).unwrap()
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
fn definition<I: TryFrom<u32>>(raw: u32) -> I
where
    I::Error: core::fmt::Debug,
{
    I::try_from(raw).unwrap()
}
