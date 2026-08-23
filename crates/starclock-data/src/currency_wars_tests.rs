//! Line-limit exception: this test-only production catalog matrix shares one expensive immutable fixture.
use std::collections::BTreeSet;

use starclock_build::{
    compiler::LoadoutCompiler,
    light_cone::{LightConeLevel, Superimposition},
    spec::{LightConeLoadout, PromotionStage},
};
use starclock_combat::{EncounterId, Ratio, Scalar};
use starclock_mode_currency_wars::{
    COMPLEX_AI_MULTIRANGE_POLICY_ID, COMPLEX_AI_SOURCE_AND_MULTIRANGE_POLICY_ID,
    CurrencyWarsActionValueBudget, CurrencyWarsAvatarBattleBehaviorArchetype,
    CurrencyWarsAvatarBattleBehaviorBindingPolicy, CurrencyWarsBattleBehaviorArchetype,
    CurrencyWarsBattleBehaviorFallbackRank, CurrencyWarsBattleBoundary,
    CurrencyWarsBattleConfigurationArchetype, CurrencyWarsBattleOverrideDefinition,
    CurrencyWarsBattleProgramBinding, CurrencyWarsBattleProgramBindingArchetype,
    CurrencyWarsBondBattleBehaviorArchetype, CurrencyWarsCarryResetPolicy,
    CurrencyWarsCharacterOverrideBinding, CurrencyWarsComplexAiContext, CurrencyWarsContentKind,
    CurrencyWarsEnemyAffixBehavior, CurrencyWarsEnemyAffixDefinition,
    CurrencyWarsEnemyAffixExecutionOwner, CurrencyWarsEntryState, CurrencyWarsEquipmentCategory,
    CurrencyWarsFinishRule, CurrencyWarsFlow, CurrencyWarsGambit, CurrencyWarsGlobalTaskCandidate,
    CurrencyWarsGlobalTaskExecutionError, CurrencyWarsGlobalTaskInvocation,
    CurrencyWarsGlobalTaskTargetPopulation, CurrencyWarsGlobalTaskTemplateDefinition,
    CurrencyWarsMechanicActivityProgram, CurrencyWarsMechanicMetadataAudit,
    CurrencyWarsMechanicPresentationKind, CurrencyWarsMechanicProgramDisposition,
    CurrencyWarsMechanicScope, CurrencyWarsProgressionProgram, CurrencyWarsRankBoundary,
    CurrencyWarsRankProgressionKey, CurrencyWarsRoleId, CurrencyWarsRoleState, CurrencyWarsRoster,
    CurrencyWarsRouteMembershipPolicy, CurrencyWarsRunPosition, CurrencyWarsServiceCatalog,
    CurrencyWarsServiceConstantValue, CurrencyWarsSharedBattleBase,
    CurrencyWarsSpecialGoodAcquisition, CurrencyWarsStarStateOwner, CurrencyWarsTalentKind,
    CurrencyWarsTransitionKind,
};

use crate::{catalog as core_catalog, load_currency_wars_battle_resources};

use super::{
    load_currency_wars_catalog, load_currency_wars_catalog_candidate,
    load_currency_wars_catalog_candidate_from_bundle, summarize_currency_wars_catalog,
};

#[test]
fn production_bundle_lowers_to_complete_runtime_denominators() {
    let catalog = load_currency_wars_catalog().unwrap();
    let summary = summarize_currency_wars_catalog(&catalog);
    let flow = catalog.flow_catalog();
    let economy = catalog.economy_catalog();
    let build = catalog.build_catalog();
    let empowerment = catalog.empowerment_catalog();
    let bonds = catalog.bond_catalog();
    let cross_investments = catalog.cross_investment_catalog();
    let content = catalog.content_catalog();
    let encounter = catalog.encounter_catalog();
    let progression = catalog.progression_catalog();
    let role_overrides = catalog.role_override_catalog();
    let services = catalog.service_catalog();

    assert_eq!(summary.routes, 26);
    assert_eq!(summary.nodes, 493);
    assert_eq!(summary.difficulties, 97);
    assert_eq!(summary.roles, 77);
    assert_eq!(summary.bonds, 49);
    assert_eq!(summary.investments, 834);
    assert_eq!(summary.policies, 12);
    assert_eq!(services.items().len(), 165);
    assert_eq!(services.special_goods().len(), 43);
    assert_eq!(
        services
            .special_goods()
            .iter()
            .filter(|good| matches!(
                good.acquisition,
                CurrencyWarsSpecialGoodAcquisition::ShopPurchase { .. }
            ))
            .count(),
        38
    );
    assert_eq!(
        services
            .special_goods()
            .iter()
            .filter(|good| {
                good.acquisition == CurrencyWarsSpecialGoodAcquisition::CyreneThreeStar
            })
            .count(),
        5
    );
    assert_eq!(services.season_items().len(), 164);
    assert_eq!(services.consumables().len(), 7);
    assert_eq!(services.managed_functions().len(), 9);
    assert_eq!(services.rewards().len(), 811);
    assert_eq!(services.reward_pools().len(), 110);
    assert_eq!(services.recipes().len(), 57);
    assert_eq!(services.upgrades().len(), 37);
    assert_eq!(services.forge_services().len(), 10);
    assert_eq!(services.constants().len(), 18);
    assert_eq!(build.recommendations().len(), 133);
    assert_eq!(services.reward(350_101).unwrap().budget_cost, Some(1));
    assert_eq!(services.reward(101_001).unwrap().budget_cost, None);
    assert_eq!(
        services.constant("GridFight_CoinItemID"),
        Some(&CurrencyWarsServiceConstantValue::Integer(281_031))
    );
    assert_eq!(
        services.constant("GridFight_SpecialAvatarWorldLevel"),
        Some(&CurrencyWarsServiceConstantValue::Integer(6))
    );
    assert_eq!(
        services.constant("GridFight_OCSeasonWeeklyScoreRatio"),
        Some(&CurrencyWarsServiceConstantValue::Integer(100))
    );
    assert_eq!(
        services.constant("GridFight_OCSeasonExpRatio"),
        Some(&CurrencyWarsServiceConstantValue::Integer(60))
    );
    assert_eq!(
        services.constant("GridFight_OCTalentPointRatio"),
        Some(&CurrencyWarsServiceConstantValue::Integer(60))
    );
    assert_eq!(
        services.constant("GridFight_FateEquip_Normal"),
        Some(&CurrencyWarsServiceConstantValue::IntegerArray(Box::new([
            352_601, 352_602, 352_603, 352_604, 352_605, 352_607, 352_608, 352_706,
        ])))
    );
    assert_eq!(CurrencyWarsServiceCatalog::proven_empty_families().len(), 9);
    assert_eq!(economy.currencies().len(), 1);
    assert_eq!(economy.offers().len(), 10);
    assert_eq!(economy.prices().len(), 5);
    assert_eq!(economy.team_levels().len(), 10);
    assert!(
        economy
            .team_levels()
            .iter()
            .all(|level| level.field_cap == level.level && level.bench_cap == 9)
    );
    assert_eq!(economy.rules().team_size.front_minimum, 1);
    assert_eq!(economy.rules().team_size.front_maximum, 4);
    assert_eq!(economy.rules().team_size.back_initial, 6);
    assert_eq!(economy.rules().team_size.back_maximum, 9);
    assert_eq!(economy.rules().team_size.bench_authored, 9);
    assert_eq!(economy.rules().team_size.bench_overflow, 100);
    assert_eq!(economy.positions().len(), 3);
    assert_eq!(economy.star_states().len(), 295);
    assert_eq!(
        economy
            .star_states()
            .iter()
            .filter(|state| matches!(state.owner, CurrencyWarsStarStateOwner::Role(_)))
            .count(),
        266
    );
    assert_eq!(
        economy
            .star_states()
            .iter()
            .filter(|state| matches!(state.owner, CurrencyWarsStarStateOwner::Servant { .. }))
            .count(),
        29
    );
    assert_eq!(economy.star_rules().len(), 189);
    assert_eq!(
        economy
            .star_states()
            .iter()
            .filter(|state| matches!(state.owner, CurrencyWarsStarStateOwner::Role(_)))
            .flat_map(|state| state.back_execution_skill_ids.iter())
            .count(),
        446
    );
    assert!(economy.star_states().iter().any(|state| {
        matches!(state.owner, CurrencyWarsStarStateOwner::Role(_))
            && state.back_execution_skill_ids.len() != state.back_display_skill_ids.len()
    }));
    assert_eq!(economy.star_lifecycle().len(), 3);
    assert_eq!(economy.action_value_limits().len(), 2);
    assert_eq!(economy.battle_result_projections().len(), 2);
    assert_eq!(economy.squad_hp().initial, 100);
    assert_eq!(economy.rules().refresh.cards_per_refresh, 5);
    assert_eq!(economy.rules().refresh.gold_cost, 2);
    assert_eq!(
        economy.rules().refresh.copies_per_role_by_rarity,
        [30, 25, 18, 10, 9],
    );
    assert_eq!(economy.rules().refresh.role_initial_weight, 100);
    assert_eq!(
        economy.rules().refresh.maximum_stolen_same_card_by_rarity,
        [10, 8, 6, 3, 3]
    );
    assert_eq!(
        economy.rules().refresh.stolen_pool_refund_initial_purchase,
        4
    );
    assert_eq!(economy.rules().refresh.stolen_pool_refund_sell, 2);
    assert_eq!(economy.rules().refresh.stolen_pool_refund_hold, 2);
    assert_eq!(economy.rules().interest.deposit_per_interest, 10);
    assert_eq!(economy.rules().interest.standard_maximum, 5);
    assert_eq!(economy.rules().interest.overclock_maximum, 0);
    assert_eq!(build.mappings().len(), 77);
    assert_eq!(build.references().len(), 77);
    assert_eq!(build.trial_builds().len(), 77);
    assert_eq!(build.sources().len(), 12);
    assert_eq!(build.substitution_rules().len(), 2);
    assert_eq!(build.equipment().len(), 520);
    assert_eq!(build.off_field_conversions().len(), 417);
    assert_eq!(
        build
            .equipment()
            .iter()
            .filter(|definition| definition.runtime.is_some())
            .count(),
        148,
    );
    assert_eq!(build.character_equipment_slot_limit(), 3);
    assert_eq!(
        build.equipment_category_limit(CurrencyWarsEquipmentCategory::Basic),
        Some(1),
    );
    assert_eq!(
        build.equipment_category_limit(CurrencyWarsEquipmentCategory::Hack),
        None,
    );
    assert_eq!(empowerment.empowerments().len(), 4_784);
    assert_eq!(
        empowerment
            .empowerments()
            .iter()
            .filter(|value| value.avatar_id.is_some())
            .count(),
        154
    );
    assert_eq!(
        empowerment
            .empowerments()
            .iter()
            .filter(|value| value.skill_id.is_some())
            .count(),
        4_630
    );
    assert_eq!(empowerment.battle_overrides().len(), 341);
    let override_count = |matches: fn(&CurrencyWarsBattleOverrideDefinition) -> bool| {
        empowerment
            .battle_overrides()
            .iter()
            .filter(|value| matches(&value.definition))
            .count()
    };
    assert_eq!(
        override_count(|value| matches!(
            value,
            CurrencyWarsBattleOverrideDefinition::AutomaticTechnique { .. }
        )),
        1
    );
    assert_eq!(
        override_count(|value| matches!(
            value,
            CurrencyWarsBattleOverrideDefinition::DefeatEnergyScaling { .. }
        )),
        1
    );
    assert_eq!(
        override_count(|value| matches!(
            value,
            CurrencyWarsBattleOverrideDefinition::LethalDamageRescue { .. }
        )),
        1
    );
    assert_eq!(
        override_count(|value| matches!(
            value,
            CurrencyWarsBattleOverrideDefinition::BackBattleEvent(_)
        )),
        119
    );
    assert_eq!(
        override_count(|value| matches!(
            value,
            CurrencyWarsBattleOverrideDefinition::FrontSpecialResource(_)
        )),
        24
    );
    assert_eq!(
        override_count(|value| matches!(
            value,
            CurrencyWarsBattleOverrideDefinition::RoleGlobalModifier(_)
        )),
        6
    );
    assert_eq!(
        override_count(|value| matches!(
            value,
            CurrencyWarsBattleOverrideDefinition::RankSkillOverride(_)
        )),
        124
    );
    assert_eq!(
        override_count(|value| matches!(
            value,
            CurrencyWarsBattleOverrideDefinition::SummonBattleEventOverride(_)
        )),
        2
    );
    assert_eq!(
        override_count(|value| matches!(
            value,
            CurrencyWarsBattleOverrideDefinition::CyreneSkillOverride(_)
        )),
        63
    );
    assert_eq!(
        empowerment
            .battle_overrides()
            .iter()
            .filter_map(|value| match &value.definition {
                CurrencyWarsBattleOverrideDefinition::RankSkillOverride(value) => {
                    Some(value.edits.len())
                }
                _ => None,
            })
            .sum::<usize>(),
        150,
    );
    assert_eq!(
        empowerment
            .battle_overrides()
            .iter()
            .filter_map(|value| match &value.definition {
                CurrencyWarsBattleOverrideDefinition::CyreneSkillOverride(value) => {
                    Some(value.edits.len())
                }
                _ => None,
            })
            .sum::<usize>(),
        75,
    );
    assert!(
        build
            .trial_builds()
            .iter()
            .all(|build| build.technique_ability.get() != 0)
    );
    assert_eq!(bonds.bonds().len(), 49);
    assert_eq!(
        bonds
            .bonds()
            .iter()
            .map(|bond| bond.levels.len())
            .sum::<usize>(),
        152
    );
    assert_eq!(bonds.contributions().len(), 683);
    assert_eq!(
        bonds
            .bonds()
            .iter()
            .map(|bond| bond.contributions.len())
            .sum::<usize>(),
        152
    );
    assert_eq!(cross_investments.portals().len(), 84);
    assert_eq!(cross_investments.orbs().len(), 376);
    assert_eq!(cross_investments.projections().len(), 2);
    assert_eq!(cross_investments.talents().len(), 53);
    assert_eq!(
        cross_investments
            .talents()
            .iter()
            .filter(|talent| talent.kind == CurrencyWarsTalentKind::Permanent)
            .count(),
        13,
    );
    assert_eq!(
        cross_investments
            .talents()
            .iter()
            .filter(|talent| talent.kind == CurrencyWarsTalentKind::Season)
            .count(),
        40,
    );
    assert_eq!(cross_investments.maze_buffs().len(), 11);
    assert_eq!(catalog.augment_catalog().enhancements().len(), 25);
    assert_eq!(catalog.augment_catalog().maze_buffs().len(), 57);
    assert_eq!(catalog.augment_catalog().monster_rules().len(), 30);
    let blessing_formula = catalog.blessing_formula_catalog();
    assert_eq!(blessing_formula.maze_buff_enhancements().len(), 7);
    assert_eq!(blessing_formula.blessing_identity_count(), 0);
    assert_eq!(blessing_formula.formula_identity_count(), 0);
    assert_eq!(blessing_formula.recipe_count(), 0);
    assert_eq!(blessing_formula.progress_state_count(), 0);
    assert_eq!(blessing_formula.randomizer_count(), 0);
    assert_eq!(blessing_formula.formula_contribution_count(), 0);
    let occurrences = catalog.occurrence_catalog();
    assert_eq!(occurrences.occurrences().len(), 167);
    assert_eq!(occurrences.variants().len(), 150);
    assert_eq!(occurrences.choices().len(), 90);
    let variant = "currency-wars.occurrence-variant.pray-finish.7320001";
    assert_eq!(
        occurrences.resolve_external_progress(variant, 4).unwrap(),
        starclock_mode_currency_wars::CurrencyWarsOccurrenceProgress {
            current: 4,
            required: 5,
            completed: false,
        }
    );
    assert_eq!(
        occurrences
            .resolve_external_progress(variant, 6)
            .unwrap()
            .current,
        5
    );
    let outcomes = occurrences
        .ordered_outcomes(
            "currency-wars.occurrence.pray.7330025",
            "currency-wars.occurrence-choice.pray.7330025",
            true,
        )
        .unwrap();
    assert_eq!(outcomes.len(), 2);
    assert_eq!(outcomes[0].bonus_id, 352_701);
    assert_eq!(outcomes[1].bonus_id, 352_801);
    assert!(
        occurrences
            .resolve_external_progress("currency-wars.occurrence-variant.tutorial-task.1", 1,)
            .is_err()
    );
    assert_eq!(content.records().len(), 1_392);
    assert_eq!(
        content
            .records_of_kind(CurrencyWarsContentKind::Occurrence)
            .count(),
        167
    );
    assert_eq!(
        content
            .records_of_kind(CurrencyWarsContentKind::OccurrenceVariant)
            .count(),
        150
    );
    assert_eq!(
        content
            .records_of_kind(CurrencyWarsContentKind::OccurrenceChoice)
            .count(),
        90
    );
    assert_eq!(
        content
            .records_of_kind(CurrencyWarsContentKind::ShopService)
            .count(),
        208
    );
    assert_eq!(
        content
            .records_of_kind(CurrencyWarsContentKind::ServiceOfferRule)
            .count(),
        164
    );
    assert_eq!(encounter.groups.len(), 25);
    assert_eq!(encounter.source_obligations.len(), 861);
    assert_eq!(encounter.waves.len(), 5);
    assert_eq!(encounter.enemy_slots.len(), 306);
    assert_eq!(encounter.enemy_affixes.len(), 721);
    assert_eq!(
        encounter
            .enemy_affixes
            .iter()
            .filter(|affix| matches!(
                affix.definition,
                CurrencyWarsEnemyAffixDefinition::Affix { .. }
            ))
            .count(),
        51
    );
    assert_eq!(
        encounter
            .enemy_affixes
            .iter()
            .filter(|affix| matches!(
                affix.definition,
                CurrencyWarsEnemyAffixDefinition::MazeBuff { .. }
            ))
            .count(),
        67
    );
    assert_eq!(encounter.enemy_scalings().count(), 603);
    let enemy_affix_behaviors = encounter
        .enemy_affix_definitions()
        .map(CurrencyWarsEnemyAffixBehavior::compile)
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(enemy_affix_behaviors.len(), 51);
    assert_eq!(
        enemy_affix_behaviors
            .iter()
            .map(|behavior| behavior.semantic)
            .collect::<BTreeSet<_>>()
            .len(),
        51
    );
    for (owner, expected) in [
        (CurrencyWarsEnemyAffixExecutionOwner::PrebattleStats, 5),
        (CurrencyWarsEnemyAffixExecutionOwner::BattleRule, 41),
        (CurrencyWarsEnemyAffixExecutionOwner::ActivityBoundary, 5),
    ] {
        assert_eq!(
            enemy_affix_behaviors
                .iter()
                .filter(|behavior| behavior.semantic.execution_owner() == owner)
                .count(),
            expected
        );
    }
    assert_eq!(encounter.boss_pools.len(), 10);
    assert_eq!(encounter.mechanic_programs.len(), 2_367);
    assert_eq!(
        encounter
            .mechanic_programs
            .iter()
            .filter(|program| program.scope == CurrencyWarsMechanicScope::CrossBattleActivity)
            .count(),
        520
    );
    assert_eq!(
        encounter
            .mechanic_programs
            .iter()
            .filter(|program| {
                program.scope == CurrencyWarsMechanicScope::BattleVisibleOrBattleBoundary
            })
            .count(),
        1_847
    );
    let tutorial_audits = encounter
        .mechanic_programs
        .iter()
        .filter(|program| {
            program
                .source_path
                .starts_with("Config/Level/GridFight/TutorialTask/")
        })
        .map(|program| match &program.disposition {
            CurrencyWarsMechanicProgramDisposition::MetadataOnly(
                CurrencyWarsMechanicMetadataAudit::Presentation(audit),
            ) => audit,
            CurrencyWarsMechanicProgramDisposition::MetadataOnly(_) => {
                panic!("tutorial program has the wrong metadata audit")
            }
            CurrencyWarsMechanicProgramDisposition::PendingExactSource { .. } => {
                panic!("tutorial presentation program remained pending")
            }
            CurrencyWarsMechanicProgramDisposition::ExecutedActivity(_) => {
                panic!("tutorial presentation program was lowered as Activity state")
            }
            CurrencyWarsMechanicProgramDisposition::ExecutedBattlePolicy(_) => {
                panic!("tutorial presentation program was lowered as battle state")
            }
            CurrencyWarsMechanicProgramDisposition::ExecutedAvatarBattlePolicy(_) => {
                panic!("tutorial presentation program was lowered as avatar battle state")
            }
            CurrencyWarsMechanicProgramDisposition::ExecutedBattleConfigurationPolicy(_) => {
                panic!("tutorial presentation program was lowered as configuration battle state")
            }
            CurrencyWarsMechanicProgramDisposition::ExecutedBondBattlePolicy(_) => {
                panic!("tutorial presentation program was lowered as Bond battle state")
            }
            CurrencyWarsMechanicProgramDisposition::ExecutedBattleProgramBindingPolicy(_) => {
                panic!("tutorial presentation program was lowered as program-binding state")
            }
            CurrencyWarsMechanicProgramDisposition::ExecutedEnemyCharacterConfiguration(_) => {
                panic!("tutorial presentation program was lowered as enemy configuration")
            }
            CurrencyWarsMechanicProgramDisposition::ExecutedEnemyAiConfiguration(_) => {
                panic!("tutorial presentation program was lowered as enemy AI")
            }
            CurrencyWarsMechanicProgramDisposition::ExecutedComplexAiGlobalFactors(_) => {
                panic!("tutorial presentation program was lowered as Complex AI factors")
            }
            CurrencyWarsMechanicProgramDisposition::ExecutedGlobalTaskTemplates(_) => {
                panic!("tutorial presentation program was lowered as a global task template")
            }
        })
        .collect::<Vec<_>>();
    assert_eq!(tutorial_audits.len(), 76);
    assert_eq!(
        tutorial_audits
            .iter()
            .flat_map(|audit| audit.configuration_type_counts.iter())
            .map(|entry| entry.count)
            .sum::<u32>(),
        683
    );
    assert_eq!(
        tutorial_audits
            .iter()
            .flat_map(|audit| audit.operation_type_counts.iter())
            .map(|entry| entry.count)
            .sum::<u32>(),
        168
    );
    let console = encounter
        .mechanic_programs
        .iter()
        .find(|program| {
            program.source_path.as_ref()
                == "Config/Level/Props/Common/InitLevelGraph_Prop_Common_GridFightConsole_01.json"
        })
        .expect("world-prop entrance graph is present");
    let CurrencyWarsMechanicProgramDisposition::MetadataOnly(
        CurrencyWarsMechanicMetadataAudit::Presentation(console_audit),
    ) = &console.disposition
    else {
        panic!("world-prop entrance graph remained pending");
    };
    assert_eq!(
        console_audit.reason,
        CurrencyWarsMechanicPresentationKind::WorldPropAndUiEntry
    );
    assert_eq!(
        console_audit
            .configuration_type_counts
            .iter()
            .map(|entry| entry.count)
            .sum::<u32>(),
        46
    );
    assert!(console_audit.operation_type_counts.is_empty());
    assert_eq!(
        encounter
            .mechanic_programs
            .iter()
            .filter(|program| matches!(
                &program.disposition,
                CurrencyWarsMechanicProgramDisposition::ExecutedActivity(_)
            ))
            .count(),
        249
    );
    assert_eq!(
        encounter
            .mechanic_programs
            .iter()
            .filter(|program| matches!(
                &program.disposition,
                CurrencyWarsMechanicProgramDisposition::MetadataOnly(
                    CurrencyWarsMechanicMetadataAudit::LayoutDescriptor(_)
                )
            ))
            .count(),
        459
    );
    assert_eq!(
        encounter
            .mechanic_programs
            .iter()
            .filter(|program| matches!(
                &program.disposition,
                CurrencyWarsMechanicProgramDisposition::MetadataOnly(
                    CurrencyWarsMechanicMetadataAudit::UnreachableCharacterOverride(_)
                )
            ))
            .count(),
        44
    );
    assert_eq!(
        encounter
            .mechanic_programs
            .iter()
            .filter(|program| matches!(
                &program.disposition,
                CurrencyWarsMechanicProgramDisposition::MetadataOnly(
                    CurrencyWarsMechanicMetadataAudit::RolePresentation(_)
                )
            ))
            .count(),
        10
    );
    assert_eq!(
        encounter
            .mechanic_programs
            .iter()
            .filter(|program| matches!(
                &program.disposition,
                CurrencyWarsMechanicProgramDisposition::MetadataOnly(
                    CurrencyWarsMechanicMetadataAudit::StructuredPresentation(_)
                )
            ))
            .count(),
        49
    );
    assert_eq!(
        encounter
            .mechanic_programs
            .iter()
            .filter(|program| matches!(
                &program.disposition,
                CurrencyWarsMechanicProgramDisposition::MetadataOnly(
                    CurrencyWarsMechanicMetadataAudit::EmptyConfiguration(_)
                )
            ))
            .count(),
        1
    );
    assert_eq!(progression.module_role_bans().len(), 2);
    assert_eq!(progression.season_role_pools().len(), 1);
    assert_eq!(progression.season_role_pools()[0].roles.len(), 77);
    assert_eq!(progression.season_trait_role_pools().len(), 32);
    assert_eq!(progression.role_reference_scores().len(), 77);
    let fate_archer = CurrencyWarsRoleId::new(1_508).unwrap();
    let fate_saber = CurrencyWarsRoleId::new(1_509).unwrap();
    let available = CurrencyWarsRoleId::new(1_507).unwrap();
    assert!(!progression.role_available(1, 7_110_501, fate_archer));
    assert!(!progression.role_available(1, 7_110_501, fate_saber));
    assert!(catalog.role_available(fate_archer));
    assert!(catalog.role_available(fate_saber));
    assert!(catalog.role_available(available));
    assert_eq!(
        catalog
            .rank_role_candidates(
                [1_001, 1_408, 1_014].map(|raw| CurrencyWarsRoleId::new(raw).unwrap())
            )
            .iter()
            .map(|role| role.get())
            .collect::<Vec<_>>(),
        vec![1_408, 1_014, 1_001]
    );
    assert_eq!(role_overrides.programs().len(), 52);
    let bronya = role_overrides
        .by_source_path(
            "Config/ConfigCharacter/GridFight/3.5/Avatar_GridFight_Bronya_00_Config.json",
        )
        .unwrap();
    assert!(bronya.bindings.iter().any(|binding| matches!(
        binding,
        CurrencyWarsCharacterOverrideBinding::RoleStar { role, star_levels }
            if role.get() == 11_011 && star_levels.as_ref() == [1, 2, 3]
    )));
    let aglaea_servant = role_overrides
        .by_source_path(
            "Config/ConfigCharacter/GridFight/3.5/Avatar_GridFight_AglaeaServant_00_Config.json",
        )
        .unwrap();
    assert!(aglaea_servant.bindings.iter().any(|binding| matches!(
        binding,
        CurrencyWarsCharacterOverrideBinding::ServantStar {
            role,
            servant_id: 11_402,
            star_levels,
        } if role.get() == 1_402 && star_levels.as_ref() == [1, 2, 3, 4]
    )));
    let lingsha_summon = role_overrides
        .summon_battle_events(1)
        .find(|program| {
            program.source_path.as_ref()
                == "Config/ConfigCharacter/GridFight/3.5/Avatar_GridFight_Lingsha_00_BE_Config.json"
        })
        .unwrap();
    assert!(lingsha_summon.bindings.iter().any(|binding| matches!(
        binding,
        CurrencyWarsCharacterOverrideBinding::SummonBattleEvent {
            season_id: 1,
            unit_id: 11_222,
            ..
        }
    )));
    assert_eq!(
        progression
            .cost_availability()
            .iter()
            .map(|rule| (
                rule.cost,
                (rule.standard.chapter, rule.standard.section),
                (rule.overclock.chapter, rule.overclock.section),
            ))
            .collect::<Vec<_>>(),
        vec![
            (1, (1, 1), (1, 1)),
            (2, (1, 1), (1, 1)),
            (3, (1, 1), (1, 1)),
            (4, (1, 9), (1, 6)),
            (5, (2, 3), (2, 1)),
        ]
    );
    assert_eq!(progression.season_rules().len(), 80);
    assert_eq!(
        encounter
            .mechanic_programs
            .iter()
            .filter(|program| matches!(
                program.disposition,
                CurrencyWarsMechanicProgramDisposition::ExecutedEnemyCharacterConfiguration(_)
            ))
            .count(),
        11
    );
    assert_eq!(
        encounter
            .mechanic_programs
            .iter()
            .filter(|program| matches!(
                &program.disposition,
                CurrencyWarsMechanicProgramDisposition::ExecutedActivity(
                    CurrencyWarsMechanicActivityProgram::Progression(
                        CurrencyWarsProgressionProgram::SeasonScoreAndExperience(_)
                    )
                )
            ))
            .count(),
        80
    );
    let first_released_difficulty = flow
        .difficulties()
        .iter()
        .find(|difficulty| difficulty.source_id == 10_101)
        .unwrap();
    let projected = catalog
        .progression_projection(
            first_released_difficulty,
            CurrencyWarsGambit::Standard,
            CurrencyWarsRunPosition::new(1, 1).unwrap(),
        )
        .unwrap()
        .unwrap();
    assert_eq!(
        projected.exact_weekly_score,
        Some(Scalar::checked_from_integer(1_200).unwrap())
    );
    assert_eq!(
        projected.exact_experience,
        Some(Scalar::checked_from_integer(25).unwrap())
    );
    let overclock = catalog
        .progression_projection(
            first_released_difficulty,
            CurrencyWarsGambit::Overclock,
            CurrencyWarsRunPosition::new(1, 1).unwrap(),
        )
        .unwrap()
        .unwrap();
    assert_eq!(
        overclock.exact_weekly_score,
        Some(Scalar::checked_from_integer(1_800).unwrap())
    );
    assert_eq!(
        overclock.exact_experience,
        Some(Scalar::checked_from_integer(24).unwrap())
    );
    assert_eq!(
        overclock.talent_point_modifier,
        Ratio::from_scaled(60_000_000)
    );
    assert_eq!(
        flow.profile().stable_key.as_ref(),
        "currency-wars.profile.v1"
    );
    assert_eq!(flow.modules().len(), 4);
    assert_eq!(flow.entries().len(), 2);
    assert_eq!(flow.gambits().len(), 2);
    assert_eq!(flow.finish_conditions().len(), 135);
    let boundaries = flow
        .finish_conditions()
        .iter()
        .filter_map(|condition| match &condition.rule {
            CurrencyWarsFinishRule::BattlePenalty(rule) => {
                Some(CurrencyWarsBattleBoundary::from_penalty_rule(rule).unwrap())
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(boundaries.len(), 114);
    assert_eq!(
        boundaries
            .iter()
            .filter(|boundary| matches!(
                boundary.action_value(),
                CurrencyWarsActionValueBudget::Finite(_)
            ))
            .count(),
        89,
    );
    assert_eq!(
        boundaries
            .iter()
            .filter(|boundary| {
                boundary.action_value() == CurrencyWarsActionValueBudget::Unlimited
            })
            .count(),
        25,
    );
    assert_eq!(flow.area_group().routes.len(), 26);
    assert_eq!(flow.layers().len(), 75);
    assert_eq!(flow.rooms().len(), 5);
    assert_eq!(flow.domain_compositions().len(), 5);
    assert_eq!(flow.stage_flow().len(), 493);
    assert_eq!(
        flow.route_membership_policy(),
        CurrencyWarsRouteMembershipPolicy::SharedCompleteRouteSet,
    );
    assert_eq!(
        flow.carry_reset_policy(),
        CurrencyWarsCarryResetPolicy::CarryRunAndParticipantStateResetNodeOffers,
    );
    assert!(
        flow.difficulties()
            .iter()
            .all(|difficulty| (1..=9).contains(&difficulty.division_level))
    );
    let expected_affix_choice_count = [0_usize, 0, 0, 1, 1, 2, 2, 3, 3, 4];
    for difficulty in flow.difficulties() {
        let expected = expected_affix_choice_count[usize::from(difficulty.division_level)];
        assert_eq!(difficulty.enemy_affix_choice_counts.len(), expected);
        assert!(
            difficulty
                .enemy_affix_choice_counts
                .iter()
                .all(|count| *count == 1)
        );
    }
    for (score, expected) in [
        (0, None),
        (1, Some("B")),
        (39, Some("B")),
        (40, Some("A")),
        (69, Some("A")),
        (70, Some("S")),
        (89, Some("S")),
        (90, Some("SS")),
        (99, Some("SS")),
        (100, Some("SSS")),
        (9_999_999, Some("SSS")),
        (10_000_000, None),
    ] {
        assert_eq!(flow.classify_settlement(score).rank_type(), expected);
    }
    let maximum_difficulty = flow
        .difficulties()
        .iter()
        .find(|difficulty| difficulty.division_level == 9)
        .expect("production data has a division-level 9 difficulty")
        .source_id;
    for route in catalog.routes() {
        let compiled = CurrencyWarsFlow::compile(route).unwrap();
        assert_eq!(
            compiled.plane_transitions().len(),
            route
                .nodes
                .iter()
                .map(|node| node.plane)
                .collect::<BTreeSet<_>>()
                .len()
                .saturating_sub(1),
        );
        for gambit in [CurrencyWarsGambit::Standard, CurrencyWarsGambit::Overclock] {
            assert!(
                flow.resolve_entry(
                    route.id,
                    maximum_difficulty,
                    gambit,
                    CurrencyWarsEntryState::new(21, true, 9),
                )
                .is_ok(),
            );
        }
    }
    assert_eq!(flow.rank_progression().len(), 108);
    assert_eq!(
        flow.level_battle_base(1, 1).unwrap(),
        CurrencyWarsSharedBattleBase {
            attack: 16_000,
            hp: 16_000,
        }
    );
    assert_eq!(
        flow.stage_battle_base(EncounterId::new(70_000_001).unwrap())
            .unwrap(),
        CurrencyWarsSharedBattleBase {
            attack: 20_000,
            hp: 16_000,
        }
    );
    assert_eq!(
        flow.stage_flow()
            .iter()
            .filter(|stage| stage.transition == CurrencyWarsTransitionKind::NextSection)
            .count(),
        418
    );
    assert_eq!(
        flow.stage_flow()
            .iter()
            .filter(|stage| stage.transition == CurrencyWarsTransitionKind::PlaneTerminal)
            .count(),
        75
    );
    assert!(
        flow.stage_flow()
            .iter()
            .all(|stage| { stage.carry_rules.is_empty() && stage.reset_rules.is_empty() })
    );
    assert_eq!(
        flow.rank_progression()
            .iter()
            .filter(|rank| matches!(rank.key, CurrencyWarsRankProgressionKey::Division { .. }))
            .count(),
        10
    );
    assert_eq!(
        flow.rank_progression()
            .iter()
            .filter(|rank| matches!(rank.key, CurrencyWarsRankProgressionKey::LevelBase { .. }))
            .count(),
        23
    );
    assert_eq!(
        flow.rank_progression()
            .iter()
            .filter(|rank| matches!(
                rank.key,
                CurrencyWarsRankProgressionKey::BinaryDifficulty { .. }
            ))
            .count(),
        8
    );
    assert_eq!(
        flow.rank_progression()
            .iter()
            .filter(|rank| matches!(rank.key, CurrencyWarsRankProgressionKey::BinaryNode(_)))
            .count(),
        44
    );
    assert_eq!(flow.binary_difficulty_addition(1, 4), Some(20));
    assert_eq!(flow.binary_difficulty_addition(2, 4), Some(30));
    assert_eq!(flow.binary_node_perform_level(33_304), Some((4, 4)));
    assert_eq!(
        flow.rank_progression()
            .iter()
            .filter(|rank| matches!(rank.key, CurrencyWarsRankProgressionKey::StageBase(_)))
            .count(),
        23
    );
    assert!(flow.rank_progression().iter().all(|rank| matches!(
        (&rank.key, &rank.boundary),
        (
            CurrencyWarsRankProgressionKey::Division { .. },
            CurrencyWarsRankBoundary::GambitDifficulty { .. }
        ) | (
            CurrencyWarsRankProgressionKey::LevelBase { .. }
                | CurrencyWarsRankProgressionKey::StageBase(_),
            CurrencyWarsRankBoundary::SharedBattleBase { .. }
        ) | (
            CurrencyWarsRankProgressionKey::BinaryDifficulty { .. },
            CurrencyWarsRankBoundary::BinaryDifficultyAddition { .. }
        ) | (
            CurrencyWarsRankProgressionKey::BinaryNode(_),
            CurrencyWarsRankBoundary::BinaryNodePerformLevel { .. }
        )
    )));
}

#[test]
fn production_m01_battle_behavior_policies_preserve_shape_and_policy_boundaries() {
    let catalog = load_currency_wars_catalog().unwrap();
    let programs = &catalog.encounter_catalog().mechanic_programs;
    let policies = programs
        .iter()
        .filter_map(|program| match &program.disposition {
            CurrencyWarsMechanicProgramDisposition::ExecutedBattlePolicy(policy) => {
                Some((program, policy))
            }
            _ => None,
        })
        .collect::<Vec<_>>();

    assert_eq!(policies.len(), 9);
    assert_eq!(
        policies
            .iter()
            .map(|(_, policy)| policy.ability_names.len())
            .sum::<usize>(),
        43
    );
    assert_eq!(
        policies
            .iter()
            .map(|(_, policy)| policy.global_modifier_names.len())
            .sum::<usize>(),
        23
    );
    assert_eq!(
        policies
            .iter()
            .flat_map(|(_, policy)| policy.callback_event_counts.iter())
            .map(|entry| entry.count)
            .sum::<u32>(),
        66
    );
    assert_eq!(
        policies
            .iter()
            .flat_map(|(_, policy)| policy.configuration_type_counts.iter())
            .map(|entry| entry.count)
            .sum::<u32>(),
        1_628
    );
    for (archetype, expected) in [
        (CurrencyWarsBattleBehaviorArchetype::BossPhaseController, 5),
        (CurrencyWarsBattleBehaviorArchetype::MultiPhaseEnemy, 1),
        (CurrencyWarsBattleBehaviorArchetype::PartnerAssist, 1),
        (CurrencyWarsBattleBehaviorArchetype::MechanicalTrait, 1),
        (
            CurrencyWarsBattleBehaviorArchetype::ShieldAndResourceTrait,
            1,
        ),
    ] {
        assert_eq!(
            policies
                .iter()
                .filter(|(_, policy)| policy.archetype == archetype)
                .count(),
            expected
        );
    }
    for (fallback_rank, expected) in [
        (CurrencyWarsBattleBehaviorFallbackRank::Minion, 1),
        (CurrencyWarsBattleBehaviorFallbackRank::Elite, 2),
        (CurrencyWarsBattleBehaviorFallbackRank::Boss, 6),
    ] {
        assert_eq!(
            policies
                .iter()
                .filter(|(_, policy)| policy.fallback_rank == fallback_rank)
                .count(),
            expected
        );
    }
    assert_eq!(
        policies
            .iter()
            .map(|(program, _)| program.source_path.as_ref())
            .collect::<BTreeSet<_>>()
            .len(),
        policies.len()
    );
    assert!(policies.iter().all(|(program, policy)| {
        program.scope == CurrencyWarsMechanicScope::BattleVisibleOrBattleBoundary
            && program.state_lifecycle.as_ref() == "BattleOwnedTypedEnemyBehaviorPolicy"
            && policy.policy_id.as_ref() == "mechanic.configuration_program"
            && policy.confidence.as_ref() == "PolicyOnlyNotObservedParity"
            && !policy.selected_behavior.is_empty()
            && !policy.unresolved_field.is_empty()
            && !policy.replacement_condition.is_empty()
    }));
    assert_eq!(
        programs
            .iter()
            .filter(|program| matches!(
                program.disposition,
                CurrencyWarsMechanicProgramDisposition::PendingExactSource { .. }
            ))
            .count(),
        1_197
    );
}

#[test]
fn production_avatar_battle_policies_preserve_bindings_shape_and_policy_boundary() {
    let catalog = load_currency_wars_catalog().unwrap();
    let policies = catalog
        .encounter_catalog()
        .mechanic_programs
        .iter()
        .filter_map(|program| match &program.disposition {
            CurrencyWarsMechanicProgramDisposition::ExecutedAvatarBattlePolicy(policy) => {
                Some((program, policy))
            }
            _ => None,
        })
        .collect::<Vec<_>>();

    assert_eq!(policies.len(), 82);
    assert_eq!(
        policies
            .iter()
            .filter(|(_, policy)| policy.archetype
                == CurrencyWarsAvatarBattleBehaviorArchetype::RoleBattleEvent)
            .count(),
        81
    );
    assert_eq!(
        policies
            .iter()
            .filter(|(_, policy)| policy.archetype
                == CurrencyWarsAvatarBattleBehaviorArchetype::AugmentBattleEvent)
            .count(),
        1
    );
    assert_eq!(
        policies
            .iter()
            .filter(|(_, policy)| policy.binding_policy
                == CurrencyWarsAvatarBattleBehaviorBindingPolicy::ExactBattleEvent)
            .count(),
        77
    );
    assert_eq!(
        policies
            .iter()
            .filter(|(_, policy)| policy.binding_policy
                == CurrencyWarsAvatarBattleBehaviorBindingPolicy::SameFamilyBattleEventFallback)
            .count(),
        4
    );
    assert_eq!(
        policies
            .iter()
            .map(|(_, policy)| policy.ability_names.len())
            .sum::<usize>(),
        294
    );
    assert_eq!(
        policies
            .iter()
            .map(|(_, policy)| policy.global_modifier_names.len())
            .sum::<usize>(),
        78
    );
    assert_eq!(
        policies
            .iter()
            .flat_map(|(_, policy)| policy.callback_event_counts.iter())
            .map(|entry| entry.count)
            .sum::<u32>(),
        658
    );
    assert_eq!(
        policies
            .iter()
            .flat_map(|(_, policy)| policy.configuration_type_counts.iter())
            .map(|entry| entry.count)
            .sum::<u32>(),
        10_185
    );
    assert_eq!(
        policies
            .iter()
            .map(|(_, policy)| policy.role_ids.len())
            .sum::<usize>(),
        66
    );
    assert_eq!(
        policies
            .iter()
            .map(|(_, policy)| policy.avatar_ids.len())
            .sum::<usize>(),
        85
    );
    assert_eq!(
        policies
            .iter()
            .map(|(_, policy)| policy.battle_event_ids.len())
            .sum::<usize>(),
        88
    );
    assert!(policies.iter().all(|(program, policy)| {
        program.scope == CurrencyWarsMechanicScope::BattleVisibleOrBattleBoundary
            && program.state_lifecycle.as_ref() == "BattleOwnedTypedAvatarBehaviorPolicy"
            && policy.policy_id.as_ref() == "mechanic.configuration_program"
            && policy.confidence.as_ref() == "PolicyOnlyNotObservedParity"
            && !policy.selected_behavior.is_empty()
            && !policy.unresolved_field.is_empty()
            && !policy.replacement_condition.is_empty()
    }));
}

#[test]
fn production_m04_configuration_policies_preserve_shape_and_controller_boundaries() {
    let catalog = load_currency_wars_catalog().unwrap();
    let policies = catalog
        .encounter_catalog()
        .mechanic_programs
        .iter()
        .filter_map(|program| match &program.disposition {
            CurrencyWarsMechanicProgramDisposition::ExecutedBattleConfigurationPolicy(policy) => {
                Some((program, policy))
            }
            _ => None,
        })
        .collect::<Vec<_>>();

    assert_eq!(policies.len(), 9);
    for (archetype, expected) in [
        (
            CurrencyWarsBattleConfigurationArchetype::CommonBattleKernel,
            2,
        ),
        (
            CurrencyWarsBattleConfigurationArchetype::SharedModifierDefinitions,
            1,
        ),
        (
            CurrencyWarsBattleConfigurationArchetype::MonsterTagController,
            1,
        ),
        (
            CurrencyWarsBattleConfigurationArchetype::CharacterController,
            1,
        ),
        (
            CurrencyWarsBattleConfigurationArchetype::MonsterController,
            1,
        ),
        (CurrencyWarsBattleConfigurationArchetype::StageController, 1),
        (
            CurrencyWarsBattleConfigurationArchetype::SeasonController,
            1,
        ),
        (
            CurrencyWarsBattleConfigurationArchetype::CurrentEquipmentController,
            1,
        ),
    ] {
        assert_eq!(
            policies
                .iter()
                .filter(|(_, policy)| policy.archetype == archetype)
                .count(),
            expected
        );
    }
    assert_eq!(
        policies
            .iter()
            .map(|(_, policy)| policy.ability_names.len())
            .sum::<usize>(),
        110
    );
    assert_eq!(
        policies
            .iter()
            .map(|(_, policy)| policy.global_modifier_names.len())
            .sum::<usize>(),
        25
    );
    assert_eq!(
        policies
            .iter()
            .flat_map(|(_, policy)| policy.callback_event_counts.iter())
            .map(|entry| entry.count)
            .sum::<u32>(),
        317
    );
    assert_eq!(
        policies
            .iter()
            .flat_map(|(_, policy)| policy.configuration_type_counts.iter())
            .map(|entry| entry.count)
            .sum::<u32>(),
        2_796
    );
    assert!(policies.iter().all(|(program, policy)| {
        program.scope == CurrencyWarsMechanicScope::BattleVisibleOrBattleBoundary
            && program.state_lifecycle.as_ref() == "BattleOwnedTypedConfigurationFamilyPolicy"
            && policy.policy_id.as_ref() == "mechanic.configuration_program"
            && policy.confidence.as_ref() == "PolicyOnlyNotObservedParity"
            && !policy.selected_behavior.is_empty()
            && !policy.unresolved_field.is_empty()
            && !policy.replacement_condition.is_empty()
    }));

    let unreachable = catalog
        .encounter_catalog()
        .mechanic_programs
        .iter()
        .filter_map(|program| match &program.disposition {
            CurrencyWarsMechanicProgramDisposition::MetadataOnly(
                CurrencyWarsMechanicMetadataAudit::UnreachableBattleConfiguration(audit),
            ) => Some((program, audit)),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(unreachable.len(), 1);
    let (program, audit) = unreachable[0];
    assert_eq!(
        program.state_lifecycle.as_ref(),
        "MetadataOnlyNoAuthoritativeState"
    );
    assert_eq!(audit.reason.as_ref(), "NoVersion44EquipmentAbilityBinding");
    assert_eq!(audit.ability_names.len(), 20);
    assert_eq!(audit.global_modifier_names.len(), 1);
    assert_eq!(
        audit
            .callback_event_counts
            .iter()
            .map(|entry| entry.count)
            .sum::<u32>(),
        52
    );
    assert_eq!(
        audit
            .configuration_type_counts
            .iter()
            .map(|entry| entry.count)
            .sum::<u32>(),
        369
    );
}

#[test]
fn production_m05_bond_policies_preserve_shape_and_binding_boundaries() {
    let catalog = load_currency_wars_catalog().unwrap();
    let policies = catalog
        .encounter_catalog()
        .mechanic_programs
        .iter()
        .filter_map(|program| match &program.disposition {
            CurrencyWarsMechanicProgramDisposition::ExecutedBondBattlePolicy(policy) => {
                Some((program, policy))
            }
            _ => None,
        })
        .collect::<Vec<_>>();

    assert_eq!(policies.len(), 31);
    for (archetype, expected) in [
        (
            CurrencyWarsBondBattleBehaviorArchetype::BondStageAbilityController,
            25,
        ),
        (
            CurrencyWarsBondBattleBehaviorArchetype::MultiBondStageAbilityController,
            1,
        ),
        (
            CurrencyWarsBondBattleBehaviorArchetype::WolfHuntSummonController,
            5,
        ),
    ] {
        assert_eq!(
            policies
                .iter()
                .filter(|(_, policy)| policy.archetype == archetype)
                .count(),
            expected
        );
    }
    assert_eq!(
        policies
            .iter()
            .map(|(_, policy)| policy.bond_ids.len())
            .sum::<usize>(),
        36
    );
    assert_eq!(
        policies
            .iter()
            .map(|(_, policy)| policy.ability_names.len())
            .sum::<usize>(),
        100
    );
    assert_eq!(
        policies
            .iter()
            .map(|(_, policy)| policy.global_modifier_names.len())
            .sum::<usize>(),
        46
    );
    assert_eq!(
        policies
            .iter()
            .flat_map(|(_, policy)| policy.callback_event_counts.iter())
            .map(|entry| entry.count)
            .sum::<u32>(),
        311
    );
    assert_eq!(
        policies
            .iter()
            .flat_map(|(_, policy)| policy.configuration_type_counts.iter())
            .map(|entry| entry.count)
            .sum::<u32>(),
        5_175
    );
    assert!(policies.iter().all(|(program, policy)| {
        program.scope == CurrencyWarsMechanicScope::BattleVisibleOrBattleBoundary
            && program.state_lifecycle.as_ref() == "BattleOwnedTypedBondBehaviorPolicy"
            && policy.policy_id.as_ref() == "mechanic.configuration_program"
            && policy.confidence.as_ref() == "PolicyOnlyNotObservedParity"
            && !policy.selected_behavior.is_empty()
            && !policy.unresolved_field.is_empty()
            && !policy.replacement_condition.is_empty()
            && policy
                .bond_ids
                .iter()
                .all(|bond| catalog.bond_catalog().bond(*bond).is_some())
    }));
}

#[test]
fn production_m06_program_policies_preserve_shape_and_typed_bindings() {
    let catalog = load_currency_wars_catalog().unwrap();
    let policies = catalog
        .encounter_catalog()
        .mechanic_programs
        .iter()
        .filter_map(|program| match &program.disposition {
            CurrencyWarsMechanicProgramDisposition::ExecutedBattleProgramBindingPolicy(policy) => {
                (!is_m07_avatar_binding_source(&program.source_path)
                    && !is_m08_avatar_or_battle_event_binding_source(&program.source_path)
                    && !is_m09_battle_event_configuration_source(&program.source_path))
                .then_some((program, policy))
            }
            _ => None,
        })
        .collect::<Vec<_>>();

    assert_eq!(policies.len(), 26);
    for (archetype, expected) in [
        (
            CurrencyWarsBattleProgramBindingArchetype::CoreAvatarAbility,
            7,
        ),
        (CurrencyWarsBattleProgramBindingArchetype::ServantAbility, 1),
        (
            CurrencyWarsBattleProgramBindingArchetype::RoleBattleEvent,
            10,
        ),
        (
            CurrencyWarsBattleProgramBindingArchetype::BondStageAbility,
            4,
        ),
        (
            CurrencyWarsBattleProgramBindingArchetype::AugmentStageAbility,
            1,
        ),
        (
            CurrencyWarsBattleProgramBindingArchetype::MonsterTagController,
            2,
        ),
        (
            CurrencyWarsBattleProgramBindingArchetype::EquipmentController,
            1,
        ),
    ] {
        assert_eq!(
            policies
                .iter()
                .filter(|(_, policy)| policy.archetype == archetype)
                .count(),
            expected
        );
    }
    let binding_count = |predicate: fn(CurrencyWarsBattleProgramBinding) -> bool| {
        policies
            .iter()
            .flat_map(|(_, policy)| policy.bindings.iter().copied())
            .filter(|binding| predicate(*binding))
            .count()
    };
    assert_eq!(
        binding_count(|value| matches!(value, CurrencyWarsBattleProgramBinding::Role(_))),
        19
    );
    assert_eq!(
        binding_count(|value| matches!(value, CurrencyWarsBattleProgramBinding::Avatar(_))),
        18
    );
    assert_eq!(
        binding_count(|value| matches!(value, CurrencyWarsBattleProgramBinding::Servant(_))),
        1
    );
    assert_eq!(
        binding_count(|value| matches!(value, CurrencyWarsBattleProgramBinding::BattleEvent(_))),
        10
    );
    assert_eq!(
        binding_count(|value| matches!(value, CurrencyWarsBattleProgramBinding::Bond(_))),
        4
    );
    assert_eq!(
        binding_count(|value| matches!(
            value,
            CurrencyWarsBattleProgramBinding::AugmentMazeBuff(_)
        )),
        7
    );
    assert_eq!(
        binding_count(|value| matches!(
            value,
            CurrencyWarsBattleProgramBinding::EnemyAffixMazeBuff(_)
        )),
        15
    );
    assert_eq!(
        binding_count(|value| matches!(value, CurrencyWarsBattleProgramBinding::Equipment(_))),
        8
    );
    assert_eq!(
        policies
            .iter()
            .map(|(_, policy)| policy.ability_names.len())
            .sum::<usize>(),
        179
    );
    assert_eq!(
        policies
            .iter()
            .map(|(_, policy)| policy.global_modifier_names.len())
            .sum::<usize>(),
        43
    );
    assert_eq!(
        policies
            .iter()
            .flat_map(|(_, policy)| policy.callback_event_counts.iter())
            .map(|entry| entry.count)
            .sum::<u32>(),
        370
    );
    assert_eq!(
        policies
            .iter()
            .flat_map(|(_, policy)| policy.configuration_type_counts.iter())
            .map(|entry| entry.count)
            .sum::<u32>(),
        6_533
    );
    assert!(policies.iter().all(|(program, policy)| {
        program.scope == CurrencyWarsMechanicScope::BattleVisibleOrBattleBoundary
            && program.state_lifecycle.as_ref() == "BattleOwnedTypedProgramBindingPolicy"
            && policy.policy_id.as_ref() == "mechanic.configuration_program"
            && policy.confidence.as_ref() == "PolicyOnlyNotObservedParity"
            && !policy.bindings.is_empty()
            && !policy.selected_behavior.is_empty()
            && !policy.unresolved_field.is_empty()
            && !policy.replacement_condition.is_empty()
    }));

    let empty = catalog
        .encounter_catalog()
        .mechanic_programs
        .iter()
        .find_map(|program| match &program.disposition {
            CurrencyWarsMechanicProgramDisposition::MetadataOnly(
                CurrencyWarsMechanicMetadataAudit::EmptyConfiguration(audit),
            ) => Some((program, audit)),
            _ => None,
        })
        .expect("M06 empty configuration audit is present");
    assert_eq!(
        empty.0.source_path.as_ref(),
        "Config/ConfigAbility/BattleEvent/GridFight/3.5/OriginAbility/GridFight_Origin_Common_StageAbility.json"
    );
    assert_eq!(
        empty.1.reason.as_ref(),
        "NoAbilityModifierCallbackOrConfigurationNode"
    );
}

#[test]
fn production_m07_avatar_programs_preserve_shape_binding_and_metadata_boundaries() {
    let catalog = load_currency_wars_catalog().unwrap();
    let programs = catalog.encounter_catalog().mechanic_programs();
    let bindings =
        programs
            .iter()
            .filter_map(|program| match &program.disposition {
                CurrencyWarsMechanicProgramDisposition::ExecutedBattleProgramBindingPolicy(
                    policy,
                ) if is_m07_avatar_binding_source(&program.source_path) => Some((program, policy)),
                _ => None,
            })
            .collect::<Vec<_>>();
    assert_eq!(bindings.len(), 29);
    for (archetype, expected) in [
        (
            CurrencyWarsBattleProgramBindingArchetype::CoreAvatarAbility,
            26,
        ),
        (CurrencyWarsBattleProgramBindingArchetype::ServantAbility, 2),
        (
            CurrencyWarsBattleProgramBindingArchetype::RoleBattleEvent,
            1,
        ),
    ] {
        assert_eq!(
            bindings
                .iter()
                .filter(|(_, policy)| policy.archetype == archetype)
                .count(),
            expected
        );
    }
    let binding_count = |predicate: fn(CurrencyWarsBattleProgramBinding) -> bool| {
        bindings
            .iter()
            .flat_map(|(_, policy)| policy.bindings.iter().copied())
            .filter(|binding| predicate(*binding))
            .count()
    };
    assert_eq!(
        binding_count(|value| matches!(value, CurrencyWarsBattleProgramBinding::Role(_))),
        31
    );
    assert_eq!(
        binding_count(|value| matches!(value, CurrencyWarsBattleProgramBinding::Avatar(_))),
        28
    );
    assert_eq!(
        binding_count(|value| matches!(value, CurrencyWarsBattleProgramBinding::Servant(_))),
        2
    );
    assert_eq!(
        binding_count(|value| matches!(value, CurrencyWarsBattleProgramBinding::BattleEvent(_))),
        1
    );
    assert_eq!(
        bindings
            .iter()
            .map(|(_, policy)| policy.ability_names.len())
            .sum::<usize>(),
        112
    );
    assert_eq!(
        bindings
            .iter()
            .map(|(_, policy)| policy.global_modifier_names.len())
            .sum::<usize>(),
        32
    );
    assert_eq!(
        bindings
            .iter()
            .flat_map(|(_, policy)| policy.callback_event_counts.iter())
            .map(|entry| entry.count)
            .sum::<u32>(),
        277
    );
    assert_eq!(
        bindings
            .iter()
            .flat_map(|(_, policy)| policy.configuration_type_counts.iter())
            .map(|entry| entry.count)
            .sum::<u32>(),
        6_316
    );

    let common = programs
        .iter()
        .find(|program| program.source_path.as_ref() == M07_COMMON_AVATAR_SOURCE)
        .unwrap();
    let CurrencyWarsMechanicProgramDisposition::ExecutedBattleConfigurationPolicy(common) =
        &common.disposition
    else {
        panic!("M07 common Avatar program did not bind the shared battle kernel");
    };
    assert_eq!(
        common.archetype,
        CurrencyWarsBattleConfigurationArchetype::CommonBattleKernel
    );
    assert_eq!(common.ability_names.len(), 5);
    assert_eq!(
        common
            .callback_event_counts
            .iter()
            .map(|entry| entry.count)
            .sum::<u32>(),
        8
    );
    assert_eq!(
        common
            .configuration_type_counts
            .iter()
            .map(|entry| entry.count)
            .sum::<u32>(),
        35
    );

    let cameras = programs
        .iter()
        .filter(|program| M07_CAMERA_SOURCES.contains(&program.source_path.as_ref()))
        .collect::<Vec<_>>();
    assert_eq!(cameras.len(), 2);
    assert!(cameras.iter().all(|program| matches!(
        &program.disposition,
        CurrencyWarsMechanicProgramDisposition::MetadataOnly(
            CurrencyWarsMechanicMetadataAudit::StructuredPresentation(audit)
        ) if audit.reason.as_ref() == "CameraAndAnimationTimingPresentation"
    )));
}

#[test]
fn production_m08_avatar_and_battle_event_programs_preserve_shape_and_binding_boundaries() {
    let catalog = load_currency_wars_catalog().unwrap();
    let programs = catalog.encounter_catalog().mechanic_programs();
    let bindings = programs
        .iter()
        .filter_map(|program| match &program.disposition {
            CurrencyWarsMechanicProgramDisposition::ExecutedBattleProgramBindingPolicy(policy)
                if is_m08_avatar_or_battle_event_binding_source(&program.source_path) =>
            {
                Some((program, policy))
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(bindings.len(), 35);
    for (archetype, expected) in [
        (
            CurrencyWarsBattleProgramBindingArchetype::CoreAvatarAbility,
            15,
        ),
        (
            CurrencyWarsBattleProgramBindingArchetype::RoleBattleEvent,
            19,
        ),
        (
            CurrencyWarsBattleProgramBindingArchetype::BondStageAbility,
            1,
        ),
    ] {
        assert_eq!(
            bindings
                .iter()
                .filter(|(_, policy)| policy.archetype == archetype)
                .count(),
            expected
        );
    }
    let binding_count = |predicate: fn(CurrencyWarsBattleProgramBinding) -> bool| {
        bindings
            .iter()
            .flat_map(|(_, policy)| policy.bindings.iter().copied())
            .filter(|binding| predicate(*binding))
            .count()
    };
    assert_eq!(
        binding_count(|value| matches!(value, CurrencyWarsBattleProgramBinding::Role(_))),
        31
    );
    assert_eq!(
        binding_count(|value| matches!(value, CurrencyWarsBattleProgramBinding::Avatar(_))),
        29
    );
    assert_eq!(
        binding_count(|value| matches!(value, CurrencyWarsBattleProgramBinding::BattleEvent(_))),
        19
    );
    assert_eq!(
        binding_count(|value| matches!(value, CurrencyWarsBattleProgramBinding::Bond(_))),
        1
    );
    assert_eq!(
        bindings
            .iter()
            .map(|(_, policy)| policy.ability_names.len())
            .sum::<usize>(),
        194
    );
    assert_eq!(
        bindings
            .iter()
            .map(|(_, policy)| policy.global_modifier_names.len())
            .sum::<usize>(),
        20
    );
    assert_eq!(
        bindings
            .iter()
            .flat_map(|(_, policy)| policy.callback_event_counts.iter())
            .map(|entry| entry.count)
            .sum::<u32>(),
        157
    );
    assert_eq!(
        bindings
            .iter()
            .flat_map(|(_, policy)| policy.configuration_type_counts.iter())
            .map(|entry| entry.count)
            .sum::<u32>(),
        3_159
    );
    let partner = bindings
        .iter()
        .find(|(program, _)| {
            program
                .source_path
                .ends_with("BattleEvent_GridFight_Cocolia_Partner_00_Config.json")
        })
        .unwrap();
    assert!(partner.1.ability_names.is_empty());
    assert!(partner.1.bindings.iter().any(|binding| matches!(
        binding,
        CurrencyWarsBattleProgramBinding::Bond(id) if id.get() == 3_001
    )));
    let summon = bindings
        .iter()
        .find(|(program, _)| {
            program
                .source_path
                .ends_with("BattleEvent_GridFight_DanHengPT_00_BE_Config.json")
        })
        .unwrap();
    assert!(summon.1.bindings.iter().any(|binding| matches!(
        binding,
        CurrencyWarsBattleProgramBinding::BattleEvent(11_414)
    )));
    assert_eq!(
        programs
            .iter()
            .filter(|program| is_m08_layout_source(&program.source_path))
            .filter(|program| matches!(
                program.disposition,
                CurrencyWarsMechanicProgramDisposition::MetadataOnly(
                    CurrencyWarsMechanicMetadataAudit::LayoutDescriptor(_)
                )
            ))
            .count(),
        28
    );
}

#[test]
fn production_m09_battle_event_configurations_preserve_shape_and_binding_boundaries() {
    let catalog = load_currency_wars_catalog().unwrap();
    let programs = catalog.encounter_catalog().mechanic_programs();
    let bindings = programs
        .iter()
        .filter_map(|program| match &program.disposition {
            CurrencyWarsMechanicProgramDisposition::ExecutedBattleProgramBindingPolicy(policy)
                if is_m09_battle_event_configuration_source(&program.source_path) =>
            {
                Some((program, policy))
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(bindings.len(), 42);
    assert_eq!(
        bindings
            .iter()
            .filter(|(_, policy)| policy.archetype
                == CurrencyWarsBattleProgramBindingArchetype::CoreAvatarAbility)
            .count(),
        1
    );
    assert_eq!(
        bindings
            .iter()
            .filter(|(_, policy)| policy.archetype
                == CurrencyWarsBattleProgramBindingArchetype::RoleBattleEvent)
            .count(),
        41
    );
    let binding_count = |predicate: fn(CurrencyWarsBattleProgramBinding) -> bool| {
        bindings
            .iter()
            .flat_map(|(_, policy)| policy.bindings.iter().copied())
            .filter(|binding| predicate(*binding))
            .count()
    };
    assert_eq!(
        binding_count(|value| matches!(value, CurrencyWarsBattleProgramBinding::Role(_))),
        62
    );
    assert_eq!(
        binding_count(|value| matches!(value, CurrencyWarsBattleProgramBinding::Avatar(_))),
        57
    );
    assert_eq!(
        binding_count(|value| matches!(value, CurrencyWarsBattleProgramBinding::BattleEvent(_))),
        88
    );
    assert_eq!(
        bindings
            .iter()
            .map(|(_, policy)| policy.ability_names.len())
            .sum::<usize>(),
        330
    );
    assert_eq!(
        bindings
            .iter()
            .map(|(_, policy)| policy.global_modifier_names.len())
            .sum::<usize>(),
        0
    );
    assert_eq!(
        bindings
            .iter()
            .flat_map(|(_, policy)| policy.callback_event_counts.iter())
            .map(|entry| entry.count)
            .sum::<u32>(),
        0
    );
    assert_eq!(
        bindings
            .iter()
            .flat_map(|(_, policy)| policy.configuration_type_counts.iter())
            .map(|entry| entry.count)
            .sum::<u32>(),
        221
    );
    let summoner = bindings
        .iter()
        .find(|(program, _)| {
            program
                .source_path
                .ends_with("BattleEvent_GridFight_TheHerta_00_Summoner01_Config.json")
        })
        .unwrap();
    assert_eq!(
        summoner.1.archetype,
        CurrencyWarsBattleProgramBindingArchetype::CoreAvatarAbility
    );
    assert!(summoner.1.bindings.iter().any(|binding| matches!(
        binding,
        CurrencyWarsBattleProgramBinding::Role(id) if id.get() == 1_401
    )));
    assert!(summoner.1.bindings.iter().any(|binding| matches!(
        binding,
        CurrencyWarsBattleProgramBinding::Avatar(id) if *id == 1_401
    )));
    let no_action_delay = bindings
        .iter()
        .find(|(program, _)| {
            program
                .source_path
                .ends_with("BattleEvent_GridFight_NoActionDelay_Config.json")
        })
        .unwrap();
    assert_eq!(
        no_action_delay
            .1
            .bindings
            .iter()
            .filter(|binding| matches!(binding, CurrencyWarsBattleProgramBinding::BattleEvent(_)))
            .count(),
        43
    );
    assert_eq!(
        programs
            .iter()
            .filter(|program| is_m09_layout_source(&program.source_path))
            .filter(|program| matches!(
                program.disposition,
                CurrencyWarsMechanicProgramDisposition::MetadataOnly(
                    CurrencyWarsMechanicMetadataAudit::LayoutDescriptor(_)
                )
            ))
            .count(),
        22
    );
}

#[test]
fn production_m10_enemy_character_configurations_bind_exact_shared_definitions() {
    let catalog = load_currency_wars_catalog().unwrap();
    let configurations = catalog
        .encounter_catalog()
        .mechanic_programs()
        .iter()
        .filter_map(|program| match &program.disposition {
            CurrencyWarsMechanicProgramDisposition::ExecutedEnemyCharacterConfiguration(
                configuration,
            ) => Some((program, configuration)),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(configurations.len(), 11);
    assert_eq!(
        configurations
            .iter()
            .map(|(_, configuration)| configuration.bindings.len())
            .sum::<usize>(),
        11
    );
    assert_eq!(
        configurations
            .iter()
            .map(|(_, configuration)| configuration.ability_names.len())
            .sum::<usize>(),
        60
    );
    assert_eq!(
        configurations
            .iter()
            .map(|(_, configuration)| configuration.skill_names.len())
            .sum::<usize>(),
        129
    );
    assert_eq!(
        configurations
            .iter()
            .map(|(_, configuration)| configuration.skill_ability_count)
            .sum::<u32>(),
        95
    );
    assert_eq!(
        configurations
            .iter()
            .map(|(_, configuration)| configuration.dynamic_source_count)
            .sum::<u32>(),
        290
    );
    assert_eq!(
        configurations
            .iter()
            .flat_map(|(_, configuration)| configuration.bindings.iter())
            .map(|binding| binding.shared_enemy_key.as_ref())
            .collect::<BTreeSet<_>>()
            .len(),
        11
    );
    assert!(configurations.iter().all(|(program, configuration)| {
        program.scope == CurrencyWarsMechanicScope::BattleVisibleOrBattleBoundary
            && program.state_lifecycle.as_ref() == "BattleOwnedTypedEnemyCharacterConfiguration"
            && configuration.bindings.len() == 1
    }));
    let resources = load_currency_wars_battle_resources(&catalog).unwrap();
    assert_eq!(resources.enemy_character_configurations().len(), 11);
    assert!(
        resources
            .enemy_character_configurations()
            .iter()
            .all(|input| input.bindings.iter().all(|binding| {
                resources
                    .combat()
                    .enemy(binding.definition)
                    .is_some_and(|definition| {
                        definition.ai_graph().is_some() && !definition.abilities().is_empty()
                    })
            }))
    );
    assert_eq!(
        catalog
            .encounter_catalog()
            .mechanic_programs()
            .iter()
            .filter(|program| {
                program.source_path.as_ref()
                    == "Config/ConfigCharacter/BattleEvent/GridFight/4.4/AvatarConfig/BattleEvent_GridFight_Sunday_10_Config.layout.json"
                    && matches!(
                        program.disposition,
                        CurrencyWarsMechanicProgramDisposition::MetadataOnly(
                            CurrencyWarsMechanicMetadataAudit::LayoutDescriptor(_)
                        )
                    )
            })
            .count(),
        1
    );
}

#[test]
fn production_m11_global_complex_ai_factors_preserve_exact_shape_and_policy_boundary() {
    let catalog = load_currency_wars_catalog().unwrap();
    let factors = catalog
        .encounter_catalog()
        .mechanic_programs()
        .iter()
        .filter_map(|program| match &program.disposition {
            CurrencyWarsMechanicProgramDisposition::ExecutedComplexAiGlobalFactors(factors)
                if factors.mapper_policy_id.as_ref() == COMPLEX_AI_MULTIRANGE_POLICY_ID =>
            {
                Some((program, factors))
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(factors.len(), 1);
    let (program, factors) = factors[0];
    assert_eq!(
        program.source_path.as_ref(),
        "Config/ConfigAI/ComplexSkillAIGlobalGroup/Global_FactorGroups_GridFight.json"
    );
    assert_eq!(
        program.state_lifecycle.as_ref(),
        "BattleOwnedTypedComplexAiFactorPolicy"
    );
    assert_eq!(
        factors.mapper_policy_id.as_ref(),
        COMPLEX_AI_MULTIRANGE_POLICY_ID
    );
    assert_eq!(factors.confidence.as_ref(), "PolicyOnlyNotObservedParity");
    assert!(!factors.selected_behavior.is_empty());
    assert!(!factors.unresolved_field.is_empty());
    assert!(!factors.replacement_condition.is_empty());
    assert_eq!(factors.groups.len(), 2);
    assert_eq!(
        factors
            .groups
            .iter()
            .map(|group| group.factors.len())
            .sum::<usize>(),
        5
    );
    assert_eq!(
        factors
            .groups
            .iter()
            .flat_map(|group| group.factors.iter())
            .map(|factor| factor.ranges.len())
            .sum::<usize>(),
        13
    );

    let heal = factors.group("Base_GridFight_SingleHeal").unwrap();
    for (hp_scaled, expected_scaled) in [
        (0, 1_500_000_000),
        (400_000, 1_100_000_000),
        (500_000, 950_000_000),
        (650_000, 450_000_000),
        (700_000, 100_000_000),
        (1_000_000, 1_000_000),
    ] {
        let context = CurrencyWarsComplexAiContext::new(Scalar::from_scaled(hp_scaled));
        assert_eq!(
            heal.evaluate(&context).unwrap(),
            Scalar::from_scaled(expected_scaled)
        );
    }

    let shield = factors.group("Base_GridFight_SingleShield").unwrap();
    let no_shield = CurrencyWarsComplexAiContext::new(Scalar::ZERO);
    assert_eq!(
        shield.evaluate(&no_shield).unwrap(),
        Scalar::checked_from_integer(110).unwrap()
    );
    let mut with_shield = CurrencyWarsComplexAiContext::new(Scalar::ONE);
    with_shield.modifiers =
        BTreeSet::from([Box::<str>::from("MAvatar_March7th_00_BPSkill_Shield")]);
    assert_eq!(
        shield.evaluate(&with_shield).unwrap(),
        Scalar::from_scaled(200_000)
    );
}

#[test]
fn production_m12_complex_ai_and_enemy_ai_preserve_shape_bindings_and_policy_boundary() {
    let catalog = load_currency_wars_catalog().unwrap();
    let programs = catalog.encounter_catalog().mechanic_programs();
    let factors = programs
        .iter()
        .filter_map(|program| match &program.disposition {
            CurrencyWarsMechanicProgramDisposition::ExecutedComplexAiGlobalFactors(factors)
                if factors.mapper_policy_id.as_ref()
                    == COMPLEX_AI_SOURCE_AND_MULTIRANGE_POLICY_ID =>
            {
                Some((program, factors))
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(factors.len(), 1);
    let (program, factors) = factors[0];
    assert_eq!(
        program.source_path.as_ref(),
        "Config/ConfigAI/ComplexSkillAIGlobalGroup/GridFight/Avatar_GridFight_ComplexSkillAI.json"
    );
    assert_eq!(factors.groups.len(), 9);
    assert_eq!(
        factors
            .groups
            .iter()
            .map(|group| group.factors.len())
            .sum::<usize>(),
        20
    );
    assert_eq!(
        factors
            .groups
            .iter()
            .flat_map(|group| group.factors.iter())
            .map(|factor| factor.ranges.len())
            .sum::<usize>(),
        42
    );
    assert_eq!(factors.confidence.as_ref(), "PolicyOnlyNotObservedParity");

    let mut qingque = CurrencyWarsComplexAiContext::new(Scalar::ONE);
    qingque.battle_global_values.insert(
        Box::<str>::from("TeamBoostPoint"),
        Scalar::checked_from_integer(3).unwrap(),
    );
    qingque.caster_modifiers.insert(Box::<str>::from(
        "MAvatar_GridFight_2011_BPTeam_HighestPower_Tag",
    ));
    assert_eq!(
        factors
            .group("Add_GridFight_QingQue_UseBP")
            .unwrap()
            .evaluate(&qingque)
            .unwrap(),
        Scalar::checked_from_integer(100_000).unwrap()
    );

    let mut sunday = CurrencyWarsComplexAiContext::new(Scalar::ONE);
    sunday.combat_power = Scalar::checked_from_integer(2).unwrap();
    sunday.damage_carry = Scalar::checked_from_integer(3).unwrap();
    sunday.servant_damage_carry = Scalar::ONE;
    assert_eq!(
        factors
            .group("Add_GridFight_Sunday_DamageCarryScore")
            .unwrap()
            .evaluate(&sunday)
            .unwrap(),
        Scalar::checked_from_integer(100).unwrap()
    );

    let enemy_ai = programs
        .iter()
        .filter_map(|program| match &program.disposition {
            CurrencyWarsMechanicProgramDisposition::ExecutedEnemyAiConfiguration(config) => {
                Some((program, config))
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(enemy_ai.len(), 3);
    assert_eq!(
        enemy_ai
            .iter()
            .map(|(_, config)| config.bindings.len())
            .sum::<usize>(),
        4
    );
    assert_eq!(
        enemy_ai
            .iter()
            .map(|(_, config)| config.decision_names.len())
            .sum::<usize>(),
        41
    );
    assert_eq!(
        enemy_ai
            .iter()
            .map(|(_, config)| config.skill_names.len())
            .sum::<usize>(),
        55
    );
    assert_eq!(
        enemy_ai
            .iter()
            .flat_map(|(_, config)| config.node_type_counts.iter())
            .map(|entry| entry.count)
            .sum::<u32>(),
        459
    );
    let resources = load_currency_wars_battle_resources(&catalog).unwrap();
    assert_eq!(resources.enemy_ai_configurations().len(), 3);
    assert!(resources.enemy_ai_configurations().iter().all(|input| {
        input.bindings.iter().all(|binding| {
            resources
                .combat()
                .enemy(binding.definition)
                .is_some_and(|definition| {
                    definition.ai_graph().is_some() && !definition.abilities().is_empty()
                })
        })
    }));
}

#[test]
fn production_m13_global_task_templates_execute_exact_selection_and_reject_presentation() {
    let catalog = load_currency_wars_catalog().unwrap();
    let (program, library) = catalog
        .encounter_catalog()
        .mechanic_programs()
        .iter()
        .find_map(|program| match &program.disposition {
            CurrencyWarsMechanicProgramDisposition::ExecutedGlobalTaskTemplates(library) => {
                Some((program, library))
            }
            _ => None,
        })
        .unwrap();
    assert_eq!(
        program.source_path.as_ref(),
        "Config/ConfigGlobalTaskListTemplate/GlobalTaskListTemplate_GridFight.json"
    );
    assert_eq!(library.templates().len(), 13);
    assert_eq!(
        library
            .templates()
            .iter()
            .filter(|template| matches!(
                &template.definition,
                CurrencyWarsGlobalTaskTemplateDefinition::ApplyModifier(_)
            ))
            .count(),
        6
    );
    assert_eq!(
        library
            .templates()
            .iter()
            .map(|template| template.typed_node_count)
            .sum::<u32>(),
        235
    );
    assert_eq!(
        library
            .templates()
            .iter()
            .map(|template| template.add_modifier_node_count)
            .sum::<u32>(),
        11
    );

    let candidate =
        |key: &'static str, formation, selectable, traits: &[&str], modifiers: &[&str]| {
            CurrencyWarsGlobalTaskCandidate {
                stable_key: key.into(),
                formation,
                selectable,
                traits: traits
                    .iter()
                    .map(|value| Box::<str>::from(*value))
                    .collect(),
                modifiers: modifiers
                    .iter()
                    .map(|value| Box::<str>::from(*value))
                    .collect(),
            }
        };
    let candidates = [
        candidate("ally-a", 2, true, &["origin"], &[]),
        candidate("ally-b", 1, false, &["origin"], &[]),
        candidate("ally-c", 3, true, &[], &["origin-member"]),
    ];
    let invocation = CurrencyWarsGlobalTaskInvocation {
        wave_number: 1,
        selected_population: None,
        check_predicate: true,
        maximum_targets: Some(1),
        modifier_name: "origin-bonus".into(),
        predicate_value: Some("origin".into()),
    };
    let lowest = library
        .execute(
            "GT_StageAbility_GridFight_Origin_Bonus_02_LowestX",
            &invocation,
            &candidates,
        )
        .unwrap();
    assert_eq!(lowest.len(), 1);
    assert_eq!(lowest[0].target_key.as_ref(), "ally-b");
    let highest = library
        .execute(
            "GT_StageAbility_GridFight_Origin_Bonus_02_HighestX",
            &invocation,
            &candidates,
        )
        .unwrap();
    assert_eq!(highest.len(), 1);
    assert_eq!(highest[0].target_key.as_ref(), "ally-a");

    let member_invocation = CurrencyWarsGlobalTaskInvocation {
        predicate_value: Some("origin-member".into()),
        maximum_targets: None,
        ..invocation.clone()
    };
    let member = library
        .execute(
            "GT_StageAbility_GridFight_Origin_Bonus_03",
            &member_invocation,
            &candidates,
        )
        .unwrap();
    assert_eq!(member.len(), 1);
    assert_eq!(member[0].target_key.as_ref(), "ally-c");

    let selectable_invocation = CurrencyWarsGlobalTaskInvocation {
        selected_population: Some(CurrencyWarsGlobalTaskTargetPopulation::SelectableAllies),
        check_predicate: false,
        maximum_targets: None,
        predicate_value: None,
        ..invocation.clone()
    };
    let selectable = library
        .execute(
            "GT_GridFight_Common_BuffLightTeam",
            &selectable_invocation,
            &candidates,
        )
        .unwrap();
    assert_eq!(
        selectable
            .iter()
            .map(|application| application.target_key.as_ref())
            .collect::<Vec<_>>(),
        ["ally-a", "ally-c"]
    );

    let later_wave = CurrencyWarsGlobalTaskInvocation {
        wave_number: 2,
        ..invocation
    };
    assert!(
        library
            .execute(
                "GT_StageAbility_GridFight_Origin_Bonus_01",
                &later_wave,
                &candidates,
            )
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        library.execute("GT_GridFight_PFM_CameraShakeBig", &later_wave, &candidates,),
        Err(CurrencyWarsGlobalTaskExecutionError::PresentationOnly)
    );
}

const M07_AVATAR_PREFIX: &str = "Config/ConfigAbility/GridFight/3.5/Avatar_GridFight_";
const M07_COMMON_AVATAR_SOURCE: &str =
    "Config/ConfigAbility/GridFight/3.5/Avatar_GridFight_Common_00_Ability.json";
const M07_CAMERA_SOURCES: [&str; 2] = [
    "Config/ConfigAbility/GridFight/3.5/Avatar_GridFight_Gepard_00_Camera.json",
    "Config/ConfigAbility/GridFight/3.5/Avatar_GridFight_PlayerBoyServant_30_Camera.json",
];

fn is_m07_avatar_binding_source(source_path: &str) -> bool {
    const STEMS: [&str; 29] = [
        "Boothill_00",
        "Bronya_00",
        "Castorice_00",
        "Cerydra_00",
        "Cipher_00",
        "Cyrene_00",
        "DanHengIL_00",
        "Dr_Ratio_00",
        "Evernight_00",
        "Feixiao_00",
        "Gallagher_00",
        "Gepard_00",
        "Guinaifen_00",
        "Harscyline_00",
        "Herta_00",
        "Huohuo_00",
        "Hyacine_00",
        "HyacineServant_00",
        "Jiaoqiu_00",
        "JingYuan_00",
        "Kafka_00",
        "Lingsha_00",
        "Mydeimos_00",
        "Natasha_00",
        "Phainon_00",
        "PlayerBoy_30",
        "PlayerBoyServant_30",
        "Qingque_00",
        "Rappa_00",
    ];
    source_path
        .strip_prefix(M07_AVATAR_PREFIX)
        .and_then(|value| value.strip_suffix("_Ability.json"))
        .is_some_and(|stem| STEMS.contains(&stem))
}

fn is_m08_avatar_or_battle_event_binding_source(source_path: &str) -> bool {
    m08_avatar_stem(source_path, "_Ability.json").is_some_and(|stem| {
        const STEMS: [(&str, &str); 15] = [
            ("3.5", "Saber_00"),
            ("3.5", "Sam_00"),
            ("3.5", "Sampo_00"),
            ("3.5", "Seele_00"),
            ("3.5", "Silwolf_00"),
            ("3.5", "Sunday_10"),
            ("3.5", "TheHerta_00"),
            ("3.5", "Tribbie_00"),
            ("3.5", "Welt_00"),
            ("4.0", "Constance_00"),
            ("4.0", "Sparxie_00"),
            ("4.0", "YaoGuang_00"),
            ("4.2", "Ashveil_00"),
            ("4.2", "Evanescia_00"),
            ("4.2", "SilverWolf999_00"),
        ];
        STEMS.contains(&stem)
    }) || m08_battle_event_stem(source_path, "_Config.json").is_some_and(|stem| {
        const STEMS: [&str; 20] = [
            "Anaxa_00",
            "BlackSwan_00",
            "BloodTrait_Attack_00",
            "BloodTrait_Start_00",
            "Cerydra_00",
            "Cocolia_00",
            "Cocolia_Partner_00",
            "DanHengPT_00_BE",
            "DanHengPT_00",
            "Evernight_00",
            "EvernightServant_00",
            "Fugue_00",
            "FuXuan_00",
            "Gallagher_00",
            "Guinaifen_00",
            "Herta_00",
            "Himeko_00",
            "Huohuo_00",
            "Jade_00",
            "Jingliu_00",
        ];
        STEMS.contains(&stem)
    })
}

fn is_m08_layout_source(source_path: &str) -> bool {
    m08_avatar_stem(source_path, "_Ability.layout.json").is_some_and(|stem| {
        const STEMS: [(&str, &str); 18] = [
            ("3.5", "Saber_00"),
            ("3.5", "Sam_00"),
            ("3.5", "Sampo_00"),
            ("3.5", "Seele_00"),
            ("3.5", "Silwolf_00"),
            ("3.5", "Sunday_10"),
            ("3.5", "TheHerta_00"),
            ("3.5", "Tribbie_00"),
            ("3.5", "Welt_00"),
            ("4.0", "Constance_00"),
            ("4.0", "Sparxie_00"),
            ("4.0", "YaoGuang_00"),
            ("4.2", "Ashveil_00"),
            ("4.2", "Evanescia_00"),
            ("4.2", "SilverWolf999_00"),
            ("4.4", "Gilgamesh_00"),
            ("4.4", "HimekoNova_00"),
            ("4.4", "TohsakaRin_00"),
        ];
        STEMS.contains(&stem)
    }) || m08_battle_event_stem(source_path, "_Config.layout.json").is_some_and(|stem| {
        const STEMS: [&str; 10] = [
            "Anaxa_00",
            "BlackSwan_00",
            "Cerydra_00",
            "DanHengPT_00",
            "Evernight_00",
            "EvernightServant_00",
            "Gallagher_00",
            "Himeko_00",
            "Jade_00",
            "Jingliu_00",
        ];
        STEMS.contains(&stem)
    })
}

fn m08_avatar_stem<'a>(source_path: &'a str, suffix: &str) -> Option<(&'a str, &'a str)> {
    let remainder = source_path.strip_prefix("Config/ConfigAbility/GridFight/")?;
    let (version, file) = remainder.split_once('/')?;
    let stem = file
        .strip_prefix("Avatar_GridFight_")?
        .strip_suffix(suffix)?;
    Some((version, stem))
}

fn m08_battle_event_stem<'a>(source_path: &'a str, suffix: &str) -> Option<&'a str> {
    source_path
        .strip_prefix(
            "Config/ConfigCharacter/BattleEvent/GridFight/3.5/AvatarConfig/BattleEvent_GridFight_",
        )?
        .strip_suffix(suffix)
}

fn is_m09_battle_event_configuration_source(source_path: &str) -> bool {
    is_m09_battle_event_source_with_suffix(source_path, "_Config.json")
}

fn is_m09_layout_source(source_path: &str) -> bool {
    is_m09_battle_event_source_with_suffix(source_path, "_Config.layout.json")
        || matches!(
            source_path,
            "Config/ConfigCharacter/BattleEvent/GridFight/4.0/AvatarConfig/Avatar_GridFight_Sparxie_00_ElationConfig.layout.json"
                | "Config/ConfigCharacter/BattleEvent/GridFight/4.0/AvatarConfig/Avatar_GridFight_YaoGuang_00_ElationConfig.layout.json"
        )
}

fn is_m09_battle_event_source_with_suffix(source_path: &str, suffix: &str) -> bool {
    let Some(remainder) = source_path.strip_prefix("Config/ConfigCharacter/BattleEvent/GridFight/")
    else {
        return false;
    };
    let Some((version, remainder)) = remainder.split_once('/') else {
        return false;
    };
    let Some((category, file)) = remainder.split_once('/') else {
        return false;
    };
    let Some(stem) = file
        .strip_prefix("BattleEvent_GridFight_")
        .and_then(|value| value.strip_suffix(suffix))
    else {
        return false;
    };
    match (version, category) {
        ("3.5", "AvatarConfig") => [
            "Luocha_00",
            "Mar_7th_00",
            "Moze_00",
            "Mydeimos_00",
            "Natasha_00",
            "NoActionDelay",
            "Pela_00",
            "PlayerBoy_30",
            "PlayerGirl_30",
            "Ren_00",
            "Robin_00",
            "RuanMei_00",
            "Saber_00",
            "Sampo_00",
            "Silwolf_00",
            "Sparkle_00",
            "SPTraitMonster_00",
            "TheHerta_00_Summoner01",
            "Tingyun_00",
            "Topaz_00_BE",
            "Topaz_00",
            "Tribbie_00",
            "Yanqing_00",
            "Yunli_00",
        ]
        .contains(&stem),
        ("3.5", "OriginConfig") => [
            "Origin_1001",
            "Origin_1005",
            "Origin_1007_Augment_35402041",
            "Origin_1007",
            "Origin_1008_00",
            "Origin_1008_01",
            "Origin_1008_02",
            "Origin_1008_03",
        ]
        .contains(&stem),
        ("4.0", "AvatarConfig") => [
            "Argenti_00",
            "Constance_00",
            "Sparxie_ExtraElation",
            "YaoGuang_00",
        ]
        .contains(&stem),
        ("4.2", "AvatarConfig") => [
            "Ashveil_00",
            "Evanescia_00",
            "Kafka_00",
            "PlayerBoy_40",
            "PlayerGirl_40",
        ]
        .contains(&stem),
        ("4.2", "OriginConfig") => stem == "Augment_35402045",
        _ => false,
    }
}

#[test]
fn production_trial_build_compiles_released_relic_values_not_only_selector_ids() {
    let catalog = load_currency_wars_catalog().unwrap();
    let core = core_catalog::load(include_bytes!("../../../config/generated/config.sora")).unwrap();
    let role = starclock_mode_currency_wars::CurrencyWarsRoleId::new(1_001).unwrap();
    let selected = catalog
        .build_catalog()
        .resolve_role_build(role, None)
        .unwrap();
    let compiled = LoadoutCompiler
        .compile(core.build_catalog(), core.combat_catalog(), selected.spec())
        .unwrap();
    let combatant = compiled.combatant();

    assert_eq!(combatant.maximum_hp().get(), 3_151);
    assert_eq!(combatant.base_attack().scaled(), 1_425_312_000);
    assert_eq!(combatant.base_defense().scaled(), 2_590_875_000);
    assert_eq!(combatant.speed().scaled(), 113_600_000);
    assert_eq!(
        combatant.base_effect_hit_rate(),
        Scalar::from_scaled(216_000)
    );
    assert_eq!(
        combatant.base_effect_resistance(),
        Scalar::from_scaled(216_000)
    );
    assert_eq!(
        combatant.build_bonuses().secondary(),
        [
            Scalar::from_scaled(162_000),
            Scalar::from_scaled(324_000),
            Scalar::from_scaled(324_000),
            Scalar::from_scaled(194_000),
            Scalar::ZERO,
        ]
    );
    assert_eq!(
        combatant.build_bonuses().element_damage_boosts(),
        [
            Scalar::ZERO,
            Scalar::ZERO,
            Scalar::from_scaled(388_800),
            Scalar::ZERO,
            Scalar::ZERO,
            Scalar::ZERO,
            Scalar::ZERO,
        ]
    );
    let trial = catalog.build_catalog().trial_build(role).unwrap();
    assert_eq!(trial.relic_sets.len(), 2);
    assert_eq!(trial.relic_sets[0].ability_name.as_ref(), "Ability51031");
    assert_eq!(trial.relic_sets[1].ability_name.as_ref(), "Ability53041");
}

#[test]
fn production_off_field_conversions_select_cumulative_eidolons_and_exact_signature_rank() {
    let catalog = load_currency_wars_catalog().unwrap();
    let role = starclock_mode_currency_wars::CurrencyWarsRoleId::new(1_001).unwrap();
    let trial = catalog.build_catalog().trial_build(role).unwrap();
    let back = catalog.build_catalog().resolve_off_field_contributions(
        role,
        starclock_mode_currency_wars::CurrencyWarsPositionKind::Back,
        &trial.spec,
    );

    assert_eq!(trial.spec.eidolon().get(), 3);
    assert_eq!(back.conversion_keys.len(), 3);
    assert!(
        back.conversion_keys
            .iter()
            .all(|key| key.contains(".rank."))
    );
    let front = catalog.build_catalog().resolve_off_field_contributions(
        role,
        starclock_mode_currency_wars::CurrencyWarsPositionKind::Front,
        &trial.spec,
    );
    assert!(front.conversion_keys.is_empty());

    let core = core_catalog::load(include_bytes!("../../../config/generated/config.sora")).unwrap();
    let signature_role = starclock_mode_currency_wars::CurrencyWarsRoleId::new(1_003).unwrap();
    let signature = core.light_cone_for_source_equipment(23_000).unwrap();
    let signature_build = catalog
        .build_catalog()
        .trial_build(signature_role)
        .unwrap()
        .spec
        .clone()
        .with_light_cone(LightConeLoadout::new(
            signature,
            LightConeLevel::new(80).unwrap(),
            PromotionStage::new(6).unwrap(),
            Superimposition::new(5).unwrap(),
        ));
    let signature_contribution = catalog.build_catalog().resolve_off_field_contributions(
        signature_role,
        starclock_mode_currency_wars::CurrencyWarsPositionKind::Back,
        &signature_build,
    );
    assert_eq!(signature_contribution.conversion_keys.len(), 1);
    assert!(signature_contribution.conversion_keys[0].contains(".equipment.1003.23000.5"));
}

#[test]
fn every_production_role_executes_all_authored_star_states_and_maximum_overflow() {
    let catalog = load_currency_wars_catalog().unwrap();

    for role in catalog.roles() {
        for star in 1..=role.maximum_star {
            let copies = catalog.star_copy_count(role.id, star).unwrap();
            let mut roster = CurrencyWarsRoster::default();
            for _ in 0..copies {
                roster = roster.acquire(&catalog, role.id).unwrap();
            }
            assert_eq!(
                roster.count(CurrencyWarsRoleState::new(role.id, star).unwrap()),
                1,
                "{} star {star}",
                role.stable_key,
            );
            assert_eq!(roster.total_units(), 1, "{} star {star}", role.stable_key);
        }

        let maximum = CurrencyWarsRoleState::new(role.id, role.maximum_star).unwrap();
        let base = CurrencyWarsRoleState::new(role.id, 1).unwrap();
        let maximum_copies = catalog.star_copy_count(role.id, role.maximum_star).unwrap();
        let mut overflow = CurrencyWarsRoster::default();
        for _ in 0..=maximum_copies {
            overflow = overflow.acquire(&catalog, role.id).unwrap();
        }
        assert_eq!(overflow.count(maximum), 1, "{} maximum", role.stable_key);
        assert_eq!(overflow.count(base), 1, "{} overflow", role.stable_key);
        let sold = overflow.sell(maximum).unwrap();
        assert_eq!(sold.count(maximum), 0, "{} teardown", role.stable_key);
        assert_eq!(sold.count(base), 1, "{} overflow", role.stable_key);
    }
}

#[test]
fn production_candidate_binds_all_generated_rows_and_content_identity() {
    let candidate = load_currency_wars_catalog_candidate().unwrap();
    let identity = candidate.identity();

    assert_eq!(identity.schema_fingerprint(), "90684d27ea1a7606");
    assert_eq!(identity.table_count(), 111);
    assert_eq!(identity.row_count(), 78_607);
    assert_eq!(
        hex(identity.schema_digest().bytes()),
        "3d70ae9bd9391e6e3278ea8048591b197b0646ac369f433239891ba2a2a6d501"
    );
    assert_eq!(
        hex(identity.configuration_digest().bytes()),
        "0a19e13dff9c73f94d3ed111691fed83e0407809506a0b1a3ae55db514b63506"
    );
    assert_eq!(
        hex(identity.content_digest().bytes()),
        "65ae5929fab11f978b4565aa29b51a5b6e9258ea324a11375babacaa0e7ef0e3"
    );
}

#[test]
fn malformed_candidate_fails_before_returning_a_partial_catalog() {
    let error = load_currency_wars_catalog_candidate_from_bundle(b"not a Sora bundle").unwrap_err();

    assert!(error.to_string().contains("shorter than header"));
}

fn hex(bytes: [u8; 32]) -> String {
    bytes
        .into_iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
