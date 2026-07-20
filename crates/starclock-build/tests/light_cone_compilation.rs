use std::sync::Arc;

use starclock_build::{
    catalog::{
        BuildCatalog, BuildCatalogBuilder, BuildCatalogErrorKind, BuildCatalogRevision,
        CharacterBuildDefinition, CharacterStatRow,
    },
    compiler::{BuildCompileErrorKind, LoadoutCompiler},
    eidolon::{EidolonDefinition, EidolonSetDefinition},
    id::{EidolonDefinitionId, LightConeId},
    light_cone::{
        CombatPath, LightConeApplicability, LightConeDefinition, LightConeLevel,
        LightConePassiveRank, LightConeStatRow, Superimposition,
    },
    patch::BuildPatch,
    report::BuildValidationStage,
    spec::{CombatantBuildSpec, EidolonLevel, LightConeLoadout, PromotionStage},
};
use starclock_combat::{
    AbilityId, Energy, Hp, ResolvedDefinitionBindings, RuleBundleId, SourceDefinitionId, Speed,
    StatValue, UnitDefinitionId, UnitLevel,
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
    rule::model::{RuleSource, SourceClass},
};

#[test]
fn exact_s1_and_s5_rows_compile_with_canonical_cone_input() {
    let combat = combat_catalog();
    let catalog = build_catalog(&combat, false);
    let reversed = build_catalog(&combat, true);

    let s1 = compile(&catalog, &combat, loadout(1, 80, 6, 1)).unwrap();
    let s5 = compile(&catalog, &combat, loadout(1, 80, 6, 5)).unwrap();
    let reversed_s5 = compile(&reversed, &combat, loadout(1, 80, 6, 5)).unwrap();

    assert_eq!(s1.combatant().maximum_hp().get(), 11_000);
    assert_eq!(s1.combatant().base_attack().scaled(), 700_000_000);
    assert_eq!(s1.combatant().base_defense().scaled(), 400_000_000);
    assert_eq!(s1.combatant().rule_bundles(), &[rule(1), rule(2)]);
    assert_eq!(s5.combatant().rule_bundles(), &[rule(1), rule(6)]);
    assert_eq!(s5, reversed_s5);
    assert_eq!(
        s5.report().entries()[6].stage(),
        BuildValidationStage::LightConeSelection
    );
}

#[test]
fn applicability_separates_base_stats_from_passive_activation() {
    let combat = combat_catalog();
    let catalog = build_catalog(&combat, false);

    let wrong_path = compile(&catalog, &combat, loadout(4, 80, 6, 3)).unwrap();
    assert_eq!(wrong_path.combatant().maximum_hp().get(), 11_000);
    assert_eq!(wrong_path.combatant().rule_bundles(), &[rule(1)]);

    let always = compile(&catalog, &combat, loadout(2, 80, 6, 3)).unwrap();
    assert_eq!(always.combatant().rule_bundles(), &[rule(1), rule(4)]);

    let base_only = compile(&catalog, &combat, loadout(3, 80, 6, 3)).unwrap();
    assert_eq!(base_only.combatant().rule_bundles(), &[rule(1)]);
}

#[test]
fn invalid_cone_selections_fail_at_the_typed_stage_without_partial_output() {
    let combat = combat_catalog();
    let catalog = build_catalog(&combat, false);

    let unknown = compile(&catalog, &combat, loadout(99, 80, 6, 1)).unwrap_err();
    assert_eq!(unknown.kind(), BuildCompileErrorKind::UnknownLightCone);
    assert_eq!(
        unknown.report().entries().last().unwrap().stage(),
        BuildValidationStage::LightConeSelection
    );

    let unsupported = compile(&catalog, &combat, loadout(1, 40, 3, 1)).unwrap_err();
    assert_eq!(
        unsupported.kind(),
        BuildCompileErrorKind::UnsupportedLightConeLevel
    );

    let mut builder = build_builder();
    builder.add_character(character());
    builder.add_light_cone(cone(
        5,
        CombatPath::Harmony,
        LightConeApplicability::MatchingPath,
        false,
        Some(rule(1)),
    ));
    let conflicting = builder.build(&combat).unwrap();
    let error = compile(&conflicting, &combat, loadout(5, 80, 6, 1)).unwrap_err();
    assert_eq!(error.kind(), BuildCompileErrorKind::PatchConflict);
    assert_eq!(
        error.report().entries().last().unwrap().stage(),
        BuildValidationStage::LightConeSelection
    );

    assert!(LightConeLevel::new(0).is_none());
    assert!(LightConeLevel::new(81).is_none());
    assert!(Superimposition::new(0).is_none());
    assert!(Superimposition::new(6).is_none());
}

#[test]
fn catalog_rejects_duplicate_incomplete_and_unresolved_cones() {
    let combat = combat_catalog();

    let mut duplicates = build_builder();
    duplicates.add_character(character());
    duplicates.add_light_cone(default_cone(1));
    duplicates.add_light_cone(default_cone(1));
    let error = duplicates.build(&combat).unwrap_err();
    assert_eq!(error.kind(), BuildCatalogErrorKind::DuplicateLightCone);
    assert_eq!(error.light_cone(), Some(cone_id(1)));

    let mut incomplete = default_cone(1);
    incomplete = LightConeDefinition::new(
        incomplete.id(),
        incomplete.source().clone(),
        incomplete.path(),
        incomplete.applicability(),
        incomplete.stats().to_vec(),
        ranks(Some(rule(2))).into_iter().take(4).collect(),
    );
    assert_cone_error(
        &combat,
        incomplete,
        BuildCatalogErrorKind::IncompleteLightConePassive,
    );

    let mut duplicate_stats = stat_rows();
    duplicate_stats.push(duplicate_stats[0]);
    assert_cone_error(
        &combat,
        LightConeDefinition::new(
            cone_id(1),
            source(301, SourceClass::Equipment),
            CombatPath::Harmony,
            LightConeApplicability::MatchingPath,
            duplicate_stats,
            ranks(Some(rule(2))),
        ),
        BuildCatalogErrorKind::InvalidLightConeStatCurve,
    );

    assert_cone_error(
        &combat,
        cone(
            1,
            CombatPath::Harmony,
            LightConeApplicability::MatchingPath,
            false,
            Some(rule(99)),
        ),
        BuildCatalogErrorKind::InvalidLightConePassive,
    );
}

fn compile<'a>(
    catalog: &'a BuildCatalog,
    combat: &'a CombatCatalog,
    light_cone: LightConeLoadout,
) -> Result<starclock_build::output::CompiledBuild, starclock_build::compiler::BuildCompileError> {
    LoadoutCompiler.compile(catalog, combat, &build_spec(light_cone))
}

fn build_catalog(combat: &CombatCatalog, reverse: bool) -> BuildCatalog {
    let mut cones = vec![
        default_cone(1),
        cone(
            2,
            CombatPath::Hunt,
            LightConeApplicability::Always,
            reverse,
            None,
        ),
        cone(
            3,
            CombatPath::Harmony,
            LightConeApplicability::BaseStatsOnly,
            reverse,
            None,
        ),
        cone(
            4,
            CombatPath::Hunt,
            LightConeApplicability::MatchingPath,
            reverse,
            None,
        ),
    ];
    if reverse {
        cones.reverse();
    }
    let mut builder = build_builder();
    builder.add_character(character());
    for cone in cones {
        builder.add_light_cone(cone);
    }
    builder.build(combat).unwrap()
}

fn default_cone(id: u32) -> LightConeDefinition {
    cone(
        id,
        CombatPath::Harmony,
        LightConeApplicability::MatchingPath,
        false,
        None,
    )
}

fn cone(
    id: u32,
    path: CombatPath,
    applicability: LightConeApplicability,
    reverse: bool,
    fixed_rule: Option<RuleBundleId>,
) -> LightConeDefinition {
    let mut stats = stat_rows();
    let mut passive = ranks(fixed_rule);
    if reverse {
        stats.reverse();
        passive.reverse();
    }
    LightConeDefinition::new(
        cone_id(id),
        source(300 + id, SourceClass::Equipment),
        path,
        applicability,
        stats,
        passive,
    )
}

fn stat_rows() -> Vec<LightConeStatRow> {
    vec![
        LightConeStatRow::new(
            cone_level(1),
            promotion(0),
            Hp::new(40).unwrap(),
            stat(25_000_000),
            stat(20_000_000),
        ),
        LightConeStatRow::new(
            cone_level(80),
            promotion(6),
            Hp::new(1_000).unwrap(),
            stat(200_000_000),
            stat(100_000_000),
        ),
    ]
}

fn ranks(fixed_rule: Option<RuleBundleId>) -> Vec<LightConePassiveRank> {
    (1..=5)
        .map(|rank| {
            LightConePassiveRank::new(
                superimposition(rank),
                vec![BuildPatch::AddRuleBundle(
                    fixed_rule.unwrap_or_else(|| rule(u32::from(rank) + 1)),
                )],
            )
        })
        .collect()
}

fn character() -> CharacterBuildDefinition {
    CharacterBuildDefinition::new(
        form(1),
        CombatPath::Harmony,
        source(100, SourceClass::Unit),
        CharacterStatRow::new(
            UnitLevel::new(80).unwrap(),
            promotion(6),
            Hp::new(10_000).unwrap(),
            Speed::from_scaled(100_000_000).unwrap(),
        )
        .with_attack_defense(stat(500_000_000), stat(300_000_000)),
        ResolvedDefinitionBindings::new(vec![ability(1)], vec![rule(1)], vec![]).unwrap(),
    )
    .with_eidolons(EidolonSetDefinition::new(
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
    ))
}

fn build_spec(light_cone: LightConeLoadout) -> CombatantBuildSpec {
    CombatantBuildSpec::new(form(1), UnitLevel::new(80).unwrap(), promotion(6))
        .with_light_cone(light_cone)
}

fn loadout(id: u32, level: u8, promotion: u8, rank: u8) -> LightConeLoadout {
    LightConeLoadout::new(
        cone_id(id),
        cone_level(level),
        self::promotion(promotion),
        superimposition(rank),
    )
}

fn assert_cone_error(
    combat: &CombatCatalog,
    cone: LightConeDefinition,
    expected: BuildCatalogErrorKind,
) {
    let id = cone.id();
    let mut builder = build_builder();
    builder.add_character(character());
    builder.add_light_cone(cone);
    let error = builder.build(combat).unwrap_err();
    assert_eq!(error.kind(), expected);
    assert_eq!(error.light_cone(), Some(id));
}

fn build_builder() -> BuildCatalogBuilder {
    BuildCatalogBuilder::new(
        BuildCatalogRevision::new("build-b4-v1").unwrap(),
        "combat-build-b4-v1",
    )
    .unwrap()
}

fn combat_catalog() -> Arc<CombatCatalog> {
    let mut builder = CombatCatalogBuilder::new("combat-build-b4-v1", [0xb4; 32]);
    for raw in 1..=6 {
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
    }
    builder.add_ability(
        AbilityDefinition::new(ability(1), definition(1), definition(1), vec![])
            .with_action(basic_action()),
    );
    for raw in 1..=6 {
        builder.add_rule(RuleDefinition::new(
            definition(raw),
            vec![definition(raw)],
            vec![definition(raw)],
        ));
        builder.add_rule_bundle(RuleBundle::new(rule(raw), vec![definition(raw)]));
    }
    builder.add_unit(UnitDefinition::new(
        form(1),
        vec![ability(1)],
        vec![rule(1)],
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

fn stat(raw: i64) -> StatValue {
    StatValue::from_scaled(raw).unwrap()
}
fn cone_level(raw: u8) -> LightConeLevel {
    LightConeLevel::new(raw).unwrap()
}
fn superimposition(raw: u8) -> Superimposition {
    Superimposition::new(raw).unwrap()
}
fn promotion(raw: u8) -> PromotionStage {
    PromotionStage::new(raw).unwrap()
}
fn cone_id(raw: u32) -> LightConeId {
    LightConeId::new(raw).unwrap()
}
fn form(raw: u32) -> UnitDefinitionId {
    UnitDefinitionId::new(raw).unwrap()
}
fn ability(raw: u32) -> AbilityId {
    AbilityId::new(raw).unwrap()
}
fn rule(raw: u32) -> RuleBundleId {
    RuleBundleId::new(raw).unwrap()
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
