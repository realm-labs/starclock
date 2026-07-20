use std::sync::Arc;

use starclock_build::{
    catalog::{
        BuildCatalog, BuildCatalogBuilder, BuildCatalogErrorKind, BuildCatalogRevision,
        CharacterBuildDefinition, CharacterStatRow,
    },
    compiler::{BuildPresetCompileError, LoadoutCompiler},
    digest::CombatantBuildDigest,
    eidolon::{EidolonDefinition, EidolonSetDefinition},
    id::{BuildPresetId, EidolonDefinitionId, LightConeId, TraceNodeId},
    light_cone::{
        CombatPath, LightConeApplicability, LightConeDefinition, LightConeLevel,
        LightConePassiveRank, LightConeStatRow, Superimposition,
    },
    output::BuildLockError,
    patch::BuildPatch,
    preset::BuildPreset,
    report::BuildSourceOwner,
    spec::{CombatantBuildSpec, EidolonLevel, LightConeLoadout, PromotionStage},
    trace::{TraceGraphDefinition, TraceNodeDefinition},
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
fn canonical_definition_catalog_build_and_spec_digests_are_stable() {
    let combat = combat_catalog();
    let first = build_catalog(&combat, false, None);
    let reversed = build_catalog(&combat, true, None);
    let spec = exact_spec(1);
    let compiled = LoadoutCompiler.compile(&first, &combat, &spec).unwrap();
    let reversed_compiled = LoadoutCompiler.compile(&reversed, &combat, &spec).unwrap();

    assert_eq!(first.digest(), reversed.digest());
    assert_eq!(
        first.character_digest(form(1)),
        reversed.character_digest(form(1))
    );
    assert_eq!(
        first.light_cone_digest(cone_id(1)),
        reversed.light_cone_digest(cone_id(1))
    );
    assert_eq!(compiled, reversed_compiled);
    assert_eq!(
        hex(first.digest().bytes()),
        "0d7f0288c2bc0fbdc1c22bdf69505b36245fce0ecfd82de82e8152c4bb687562"
    );
    assert_eq!(
        hex(first.character_digest(form(1)).unwrap().bytes()),
        "4d52d6aacdf1643903345ba1d4fe4a5fec5788cc86150745cf6c84d19f76e268"
    );
    assert_eq!(
        hex(first.light_cone_digest(cone_id(1)).unwrap().bytes()),
        "3988011da64bf3f0d4ec9d017fb8670c57a78a3b569da05fdccc9aacfb665f85"
    );
    assert_eq!(
        hex(compiled.build_digest().bytes()),
        "dd3d94158a289d54d4d78fe68e94591df3620968644d54548bd25e65ce1c8572"
    );
    assert_eq!(
        hex(compiled.combatant().digest().bytes()),
        "fd966e983776290d286e6aae271791a31207c4d395fda0e248ce4c18e83f23c8"
    );
}

#[test]
fn selected_sources_cross_the_generic_boundary_and_keep_detailed_owners() {
    let combat = combat_catalog();
    let catalog = build_catalog(&combat, false, None);
    let compiled = LoadoutCompiler
        .compile(&catalog, &combat, &exact_spec(1))
        .unwrap();

    assert_eq!(
        compiled
            .combatant()
            .sources()
            .iter()
            .map(|source| (source.definition().get(), source.class()))
            .collect::<Vec<_>>(),
        vec![
            (100, SourceClass::Unit),
            (110, SourceClass::Progression),
            (201, SourceClass::Progression),
            (301, SourceClass::Equipment),
        ]
    );
    assert_eq!(
        compiled
            .report()
            .sources()
            .iter()
            .map(|source| source.owner())
            .collect::<Vec<_>>(),
        vec![
            BuildSourceOwner::Character(form(1)),
            BuildSourceOwner::Trace(trace(1)),
            BuildSourceOwner::Eidolon(eidolon_id(1)),
            BuildSourceOwner::LightCone(cone_id(1)),
        ]
    );

    let changed = LoadoutCompiler
        .compile(&catalog, &combat, &exact_spec(5))
        .unwrap();
    assert_ne!(compiled.build_digest(), changed.build_digest());
    assert_ne!(compiled.combatant().digest(), changed.combatant().digest());
}

#[test]
fn named_presets_expand_exactly_and_build_locks_reject_stale_values() {
    let combat = combat_catalog();
    let preset = BuildPreset::new(preset_id(1), "fixture-max", exact_spec(1)).unwrap();
    let initial = build_catalog(&combat, false, Some(preset));
    let direct = LoadoutCompiler
        .compile(&initial, &combat, &exact_spec(1))
        .unwrap();
    let expected = direct.build_digest();

    let verified_preset = BuildPreset::new(preset_id(1), "fixture-max", exact_spec(1))
        .unwrap()
        .with_expected_build_digest(expected);
    let catalog = build_catalog(&combat, false, Some(verified_preset));
    let expanded = LoadoutCompiler
        .compile_preset(&catalog, &combat, preset_id(1))
        .unwrap();
    assert_eq!(direct, expanded);
    expanded.lock().verify(&catalog, &expanded).unwrap();

    let changed = LoadoutCompiler
        .compile(&catalog, &combat, &exact_spec(5))
        .unwrap();
    assert_eq!(
        expanded.lock().verify(&catalog, &changed),
        Err(BuildLockError::BuildMismatch)
    );
    let other_catalog = build_catalog_with_revision(&combat, "build-b5-v2", None);
    assert_eq!(
        expanded.lock().verify(&other_catalog, &expanded),
        Err(BuildLockError::CatalogMismatch)
    );
    assert!(matches!(
        LoadoutCompiler.compile_preset(&catalog, &combat, preset_id(99)),
        Err(BuildPresetCompileError::UnknownPreset)
    ));
}

#[test]
fn catalog_rejects_duplicate_invalid_and_digest_mismatched_presets() {
    let combat = combat_catalog();
    let mut duplicates = builder("build-b5-v1");
    duplicates.add_character(character(false));
    duplicates.add_light_cone(light_cone(false));
    duplicates.add_preset(BuildPreset::new(preset_id(1), "same", exact_spec(1)).unwrap());
    duplicates.add_preset(BuildPreset::new(preset_id(2), "same", exact_spec(1)).unwrap());
    let error = duplicates.build(&combat).unwrap_err();
    assert_eq!(error.kind(), BuildCatalogErrorKind::DuplicateBuildPreset);
    assert_eq!(error.preset(), Some(preset_id(2)));

    let invalid = BuildPreset::new(
        preset_id(1),
        "unknown-form",
        CombatantBuildSpec::new(form(99), level(80), promotion(6)),
    )
    .unwrap();
    let error = build_error(&combat, invalid);
    assert_eq!(error.kind(), BuildCatalogErrorKind::InvalidBuildPreset);

    let mismatched = BuildPreset::new(preset_id(1), "wrong-digest", exact_spec(1))
        .unwrap()
        .with_expected_build_digest(CombatantBuildDigest::new([0xff; 32]));
    let error = build_error(&combat, mismatched);
    assert_eq!(
        error.kind(),
        BuildCatalogErrorKind::BuildPresetDigestMismatch
    );
}

fn build_error(
    combat: &CombatCatalog,
    preset: BuildPreset,
) -> starclock_build::catalog::BuildCatalogError {
    let mut value = builder("build-b5-v1");
    value.add_character(character(false));
    value.add_light_cone(light_cone(false));
    value.add_preset(preset);
    value.build(combat).unwrap_err()
}

fn build_catalog(
    combat: &CombatCatalog,
    reverse: bool,
    preset: Option<BuildPreset>,
) -> BuildCatalog {
    build_catalog_with_revision_and_order(combat, "build-b5-v1", reverse, preset)
}

fn build_catalog_with_revision(
    combat: &CombatCatalog,
    revision: &str,
    preset: Option<BuildPreset>,
) -> BuildCatalog {
    build_catalog_with_revision_and_order(combat, revision, false, preset)
}

fn build_catalog_with_revision_and_order(
    combat: &CombatCatalog,
    revision: &str,
    reverse: bool,
    preset: Option<BuildPreset>,
) -> BuildCatalog {
    let mut value = builder(revision);
    if let Some(preset) = preset {
        value.add_preset(preset);
    }
    value.add_light_cone(light_cone(reverse));
    value.add_character(character(reverse));
    value.build(combat).unwrap()
}

fn builder(revision: &str) -> BuildCatalogBuilder {
    BuildCatalogBuilder::new(
        BuildCatalogRevision::new(revision).unwrap(),
        "combat-build-b5-v1",
    )
    .unwrap()
}

fn character(reverse: bool) -> CharacterBuildDefinition {
    let mut eidolons = (1..=6)
        .map(|rank| {
            EidolonDefinition::new(
                eidolon_id(rank),
                source(200 + rank, SourceClass::Progression),
                EidolonLevel::new(u8::try_from(rank).unwrap()).unwrap(),
                if rank == 1 {
                    vec![BuildPatch::AddRuleBundle(rule(3))]
                } else {
                    vec![]
                },
            )
        })
        .collect::<Vec<_>>();
    if reverse {
        eidolons.reverse();
    }
    CharacterBuildDefinition::new(
        form(1),
        CombatPath::Harmony,
        source(100, SourceClass::Unit),
        CharacterStatRow::new(
            level(80),
            promotion(6),
            Hp::new(10_000).unwrap(),
            Speed::from_scaled(100_000_000).unwrap(),
        )
        .with_attack_defense(stat(500_000_000), stat(300_000_000)),
        ResolvedDefinitionBindings::new(vec![ability(1)], vec![rule(1)], vec![]).unwrap(),
    )
    .with_trace_graph(TraceGraphDefinition::new(
        form(1),
        vec![TraceNodeDefinition::new(
            trace(1),
            source(110, SourceClass::Progression),
            vec![],
            promotion(0),
            vec![BuildPatch::AddRuleBundle(rule(2))],
        )],
    ))
    .with_eidolons(EidolonSetDefinition::new(form(1), eidolons))
}

fn light_cone(reverse: bool) -> LightConeDefinition {
    let mut stats = vec![
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
    ];
    let mut ranks = (1..=5)
        .map(|rank| {
            LightConePassiveRank::new(
                superimposition(rank),
                vec![BuildPatch::AddRuleBundle(rule(3 + u32::from(rank)))],
            )
        })
        .collect::<Vec<_>>();
    if reverse {
        stats.reverse();
        ranks.reverse();
    }
    LightConeDefinition::new(
        cone_id(1),
        source(301, SourceClass::Equipment),
        CombatPath::Harmony,
        LightConeApplicability::MatchingPath,
        stats,
        ranks,
    )
}

fn exact_spec(rank: u8) -> CombatantBuildSpec {
    CombatantBuildSpec::new(form(1), level(80), promotion(6))
        .with_traces(vec![trace(1)])
        .unwrap()
        .with_eidolon(EidolonLevel::new(1).unwrap())
        .with_light_cone(LightConeLoadout::new(
            cone_id(1),
            cone_level(80),
            promotion(6),
            superimposition(rank),
        ))
}

fn combat_catalog() -> Arc<CombatCatalog> {
    let mut value = CombatCatalogBuilder::new("combat-build-b5-v1", [0xb5; 32]);
    for raw in 1..=8 {
        value.add_selector(SelectorDefinition::new(definition(raw)).with_unit_targets(
            UnitTargetSelector::new(TargetRelation::SelfUnit, TargetPattern::Single).unwrap(),
        ));
        value.add_program(ProgramDefinition::new(
            definition(raw),
            vec![],
            vec![definition(raw)],
            vec![],
            vec![],
        ));
        value.add_rule(RuleDefinition::new(
            definition(raw),
            vec![definition(raw)],
            vec![definition(raw)],
        ));
        value.add_rule_bundle(RuleBundle::new(rule(raw), vec![definition(raw)]));
    }
    value.add_ability(
        AbilityDefinition::new(ability(1), definition(1), definition(1), vec![])
            .with_action(basic_action()),
    );
    value.add_unit(UnitDefinition::new(
        form(1),
        vec![ability(1)],
        vec![rule(1), rule(2), rule(3)],
    ));
    value.build().unwrap()
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

fn hex(bytes: [u8; 32]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
fn source(raw: u32, class: SourceClass) -> RuleSource {
    RuleSource::new(
        SourceDefinitionId::new(raw).unwrap(),
        class,
        vec![SourceDefinitionId::new(900 + raw).unwrap()],
        [u8::try_from(raw).unwrap_or(0x7f); 32],
    )
}
fn stat(raw: i64) -> StatValue {
    StatValue::from_scaled(raw).unwrap()
}
fn level(raw: u8) -> UnitLevel {
    UnitLevel::new(raw).unwrap()
}
fn promotion(raw: u8) -> PromotionStage {
    PromotionStage::new(raw).unwrap()
}
fn cone_level(raw: u8) -> LightConeLevel {
    LightConeLevel::new(raw).unwrap()
}
fn superimposition(raw: u8) -> Superimposition {
    Superimposition::new(raw).unwrap()
}
fn preset_id(raw: u32) -> BuildPresetId {
    BuildPresetId::new(raw).unwrap()
}
fn eidolon_id(raw: u32) -> EidolonDefinitionId {
    EidolonDefinitionId::new(raw).unwrap()
}
fn cone_id(raw: u32) -> LightConeId {
    LightConeId::new(raw).unwrap()
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
fn rule(raw: u32) -> RuleBundleId {
    RuleBundleId::new(raw).unwrap()
}
fn definition<I: TryFrom<u32>>(raw: u32) -> I
where
    I::Error: core::fmt::Debug,
{
    I::try_from(raw).unwrap()
}
