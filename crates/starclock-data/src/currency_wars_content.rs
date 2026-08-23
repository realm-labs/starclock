use serde::Serialize;
use starclock_mode_currency_wars::{
    CurrencyWarsContentCatalog, CurrencyWarsContentKind, CurrencyWarsContentRecord,
    CurrencyWarsContentReference, CurrencyWarsReferenceKind,
};

use crate::{
    currency_wars::{CurrencyWarsDataError, debug_error},
    currency_wars_flow::{parse_boxed_strings, parse_json},
    currency_wars_generated::{
        SoraConfig, currency_wars_adventure_outcomes::CurrencyWarsAdventureOutcomes,
        currency_wars_augment_maze_buffs::CurrencyWarsAugmentMazeBuffs,
        currency_wars_curse_chests::CurrencyWarsCurseChests,
        currency_wars_gamble_groups::CurrencyWarsGambleGroups,
        currency_wars_gamble_units::CurrencyWarsGambleUnits,
        currency_wars_module_ban_rules::CurrencyWarsModuleBanRules,
        currency_wars_orb_displays::CurrencyWarsOrbDisplays,
        currency_wars_portal_maze_buffs::CurrencyWarsPortalMazeBuffs,
        currency_wars_projection_maze_buffs::CurrencyWarsProjectionMazeBuffs,
        currency_wars_shop_services::CurrencyWarsShopServices,
        currency_wars_talent_maze_buffs::CurrencyWarsTalentMazeBuffs,
        currency_wars_workbench_functions::CurrencyWarsWorkbenchFunctions,
    },
};

pub(super) fn lower_currency_wars_content(
    config: &SoraConfig,
) -> Result<CurrencyWarsContentCatalog, CurrencyWarsDataError> {
    let mut records = Vec::with_capacity(1_392);

    macro_rules! simple {
        ($rows:expr, $kind:expr, $fields:expr) => {
            for row in $rows.ordered_rows() {
                records.push(record(
                    &row.stable_key,
                    None,
                    $kind,
                    Vec::new(),
                    Box::new([]),
                    &$fields(row),
                )?);
            }
        };
    }

    simple!(
        config.currency_wars_augment_maze_buffs(),
        CurrencyWarsContentKind::AugmentMazeBuff,
        |row: &CurrencyWarsAugmentMazeBuffs| (
            row.buff_series.clone(),
            row.level.clone(),
            row.binding.clone(),
            row.parameters.clone(),
            row.modifier.clone()
        )
    );
    for row in config.currency_wars_augment_monster_rules().ordered_rows() {
        records.push(record(
            &row.stable_key,
            None,
            CurrencyWarsContentKind::AugmentMonsterRule,
            Vec::new(),
            parse_boxed_strings(row.effect_ids.as_ref())?,
            &(&row.quality, &row.parameters),
        )?);
    }
    for row in config.currency_wars_augment_remarks().ordered_rows() {
        records.push(record(
            &row.stable_key,
            None,
            CurrencyWarsContentKind::AugmentRemark,
            single_reference(CurrencyWarsReferenceKind::Augment, row.augment_id.as_ref()),
            Box::new([]),
            &(&row.augment_id, &row.remark),
        )?);
    }
    simple!(
        config.currency_wars_module_ban_rules(),
        CurrencyWarsContentKind::ModuleBanRule,
        |row: &CurrencyWarsModuleBanRules| (
            row.module_id.clone(),
            row.subject_kind.clone(),
            row.subject_id.clone()
        )
    );
    simple!(
        config.currency_wars_orb_displays(),
        CurrencyWarsContentKind::OrbDisplay,
        |row: &CurrencyWarsOrbDisplays| (row.orb_type.clone(), row.display_locator.clone())
    );
    simple!(
        config.currency_wars_portal_maze_buffs(),
        CurrencyWarsContentKind::PortalMazeBuff,
        |row: &CurrencyWarsPortalMazeBuffs| (
            row.buff_series.clone(),
            row.level.clone(),
            row.binding.clone(),
            row.parameters.clone(),
            row.modifier.clone()
        )
    );
    for row in config.currency_wars_portal_remarks().ordered_rows() {
        records.push(record(
            &row.stable_key,
            None,
            CurrencyWarsContentKind::PortalRemark,
            single_reference(CurrencyWarsReferenceKind::Portal, row.portal_id.as_ref()),
            Box::new([]),
            &(&row.portal_id, &row.remark),
        )?);
    }
    simple!(
        config.currency_wars_projection_maze_buffs(),
        CurrencyWarsContentKind::ProjectionMazeBuff,
        |row: &CurrencyWarsProjectionMazeBuffs| (
            row.buff_series.clone(),
            row.level.clone(),
            row.binding.clone(),
            row.parameters.clone(),
            row.modifier.clone()
        )
    );
    for row in config
        .currency_wars_season_augment_memberships()
        .ordered_rows()
    {
        records.push(record(
            &row.stable_key,
            None,
            CurrencyWarsContentKind::SeasonAugmentMembership,
            single_reference(CurrencyWarsReferenceKind::Augment, row.augment_id.as_ref()),
            Box::new([]),
            &(&row.season_id, &row.augment_id),
        )?);
    }
    for row in config
        .currency_wars_season_portal_memberships()
        .ordered_rows()
    {
        records.push(record(
            &row.stable_key,
            None,
            CurrencyWarsContentKind::SeasonPortalMembership,
            single_reference(CurrencyWarsReferenceKind::Portal, row.portal_id.as_ref()),
            Box::new([]),
            &(&row.season_id, &row.portal_id),
        )?);
    }
    for row in config.currency_wars_season_talents().ordered_rows() {
        let mut refs = references(
            CurrencyWarsReferenceKind::Prerequisite,
            row.prerequisite_ids.as_ref(),
        )?;
        refs.extend(references(
            CurrencyWarsReferenceKind::Successor,
            row.successor_ids.as_ref(),
        )?);
        records.push(record(
            &row.stable_key,
            None,
            CurrencyWarsContentKind::SeasonTalent,
            refs,
            parse_boxed_strings(row.effect_ids.as_ref())?,
            &(&row.season_id, &row.cost, &row.config_path),
        )?);
    }
    for row in config.currency_wars_selected_enhancements().ordered_rows() {
        records.push(record(
            &row.stable_key,
            None,
            CurrencyWarsContentKind::SelectedEnhancement,
            single_reference(
                CurrencyWarsReferenceKind::Trait,
                row.trait_effect_id.as_ref(),
            ),
            parse_boxed_strings(row.effect_ids.as_ref())?,
            &(&row.trait_effect_id, &row.cost, &row.parameters),
        )?);
    }
    simple!(
        config.currency_wars_talent_maze_buffs(),
        CurrencyWarsContentKind::TalentMazeBuff,
        |row: &CurrencyWarsTalentMazeBuffs| (
            row.buff_series.clone(),
            row.level.clone(),
            row.binding.clone(),
            row.parameters.clone(),
            row.modifier.clone()
        )
    );
    for row in config.currency_wars_blessing_paths().ordered_rows() {
        let mut refs = references(
            CurrencyWarsReferenceKind::Candidate,
            row.offer_roles.as_ref(),
        )?;
        refs.extend(references(
            CurrencyWarsReferenceKind::Formula,
            row.formula_roles.as_ref(),
        )?);
        records.push(record(
            &row.stable_key,
            row.path_id.as_ref(),
            CurrencyWarsContentKind::BlessingPath,
            refs,
            Box::new([]),
            &(&row.offer_roles, &row.formula_roles),
        )?);
    }
    for row in config.currency_wars_blessings().ordered_rows() {
        records.push(record(
            &row.stable_key,
            row.path_id.as_ref(),
            CurrencyWarsContentKind::Blessing,
            references(CurrencyWarsReferenceKind::Blessing, row.level_ids.as_ref())?,
            parse_boxed_strings(row.effect_ids.as_ref())?,
            &(&row.path_id, &row.category),
        )?);
    }
    for row in config.currency_wars_blessing_levels().ordered_rows() {
        records.push(record(
            &row.stable_key,
            None,
            CurrencyWarsContentKind::BlessingLevel,
            single_reference(
                CurrencyWarsReferenceKind::Blessing,
                row.blessing_id.as_ref(),
            ),
            parse_boxed_strings(row.effect_ids.as_ref())?,
            &(&row.blessing_id, &row.level, &row.parameters),
        )?);
    }
    for row in config.currency_wars_blessing_groups().ordered_rows() {
        records.push(record(
            &row.stable_key,
            None,
            CurrencyWarsContentKind::BlessingGroup,
            references(
                CurrencyWarsReferenceKind::Candidate,
                row.candidate_ids.as_ref(),
            )?,
            Box::new([]),
            &(&row.selection_policy, &row.weight_program),
        )?);
    }
    for row in config.currency_wars_formulas().ordered_rows() {
        records.push(record(
            &row.stable_key,
            row.recipe_id.as_ref(),
            CurrencyWarsContentKind::Formula,
            Vec::new(),
            parse_boxed_strings(row.effect_ids.as_ref())?,
            &(&row.formula_kind, &row.progress_states),
        )?);
    }
    for row in config.currency_wars_formula_displays().ordered_rows() {
        records.push(record(
            &row.stable_key,
            None,
            CurrencyWarsContentKind::FormulaDisplay,
            single_reference(CurrencyWarsReferenceKind::Formula, row.formula_id.as_ref()),
            Box::new([]),
            &(&row.display_state, &row.mechanical_summary_ids),
        )?);
    }
    for row in config.currency_wars_formula_randomizers().ordered_rows() {
        records.push(record(
            &row.stable_key,
            None,
            CurrencyWarsContentKind::FormulaRandomizer,
            references(
                CurrencyWarsReferenceKind::Candidate,
                row.candidate_ids.as_ref(),
            )?,
            Box::new([]),
            &(&row.weight_program, &row.reroll_rule, &row.fallback),
        )?);
    }
    for row in config.currency_wars_occurrences().ordered_rows() {
        let mut refs = references(
            CurrencyWarsReferenceKind::OccurrenceVariant,
            row.variant_ids.as_ref(),
        )?;
        refs.extend(references(
            CurrencyWarsReferenceKind::OccurrenceChoice,
            row.choice_ids.as_ref(),
        )?);
        records.push(record(
            &row.stable_key,
            None,
            CurrencyWarsContentKind::Occurrence,
            refs,
            Box::new([]),
            &row.unlock_rules,
        )?);
    }
    for row in config.currency_wars_occurrence_variants().ordered_rows() {
        let mut refs = single_reference(
            CurrencyWarsReferenceKind::Occurrence,
            row.occurrence_id.as_ref(),
        );
        refs.extend(references(
            CurrencyWarsReferenceKind::OccurrenceChoice,
            row.choice_ids.as_ref(),
        )?);
        records.push(record(
            &row.stable_key,
            None,
            CurrencyWarsContentKind::OccurrenceVariant,
            refs,
            Box::new([]),
            &(&row.graph_path, &row.entry_conditions),
        )?);
    }
    for row in config.currency_wars_occurrence_choices().ordered_rows() {
        records.push(record(
            &row.stable_key,
            None,
            CurrencyWarsContentKind::OccurrenceChoice,
            occurrence_parent(row.variant_id.as_ref()),
            Box::new([]),
            &(
                &row.ordinal,
                &row.conditions,
                &row.costs,
                &row.ordered_outcomes,
            ),
        )?);
    }
    for row in config.currency_wars_workbenches().ordered_rows() {
        let mut refs = references(
            CurrencyWarsReferenceKind::WorkbenchFunction,
            row.function_ids.as_ref(),
        )?;
        refs.extend(references(
            CurrencyWarsReferenceKind::Currency,
            row.currency_ids.as_ref(),
        )?);
        records.push(record(
            &row.stable_key,
            None,
            CurrencyWarsContentKind::Workbench,
            refs,
            Box::new([]),
            &row.availability,
        )?);
    }
    simple!(
        config.currency_wars_workbench_functions(),
        CurrencyWarsContentKind::WorkbenchFunction,
        |row: &CurrencyWarsWorkbenchFunctions| (
            row.function_type.clone(),
            row.input_policy.clone(),
            row.output_policy.clone(),
            row.price_rule.clone()
        )
    );
    simple!(
        config.currency_wars_gamble_groups(),
        CurrencyWarsContentKind::GambleGroup,
        |row: &CurrencyWarsGambleGroups| (
            row.group_type.clone(),
            row.unit_ids.clone(),
            row.offer_policy.clone()
        )
    );
    simple!(
        config.currency_wars_gamble_units(),
        CurrencyWarsContentKind::GambleUnit,
        |row: &CurrencyWarsGambleUnits| (
            row.unit_type.clone(),
            row.parameters.clone(),
            row.outcome_program.clone()
        )
    );
    simple!(
        config.currency_wars_curse_chests(),
        CurrencyWarsContentKind::CurseChest,
        |row: &CurrencyWarsCurseChests| (
            row.chest_type.clone(),
            row.parameters.clone(),
            row.choice_program.clone()
        )
    );
    simple!(
        config.currency_wars_adventure_outcomes(),
        CurrencyWarsContentKind::AdventureOutcome,
        |row: &CurrencyWarsAdventureOutcomes| (
            row.adventure_type.clone(),
            row.parameter_group_id.clone(),
            row.abstract_outcome.clone()
        )
    );
    simple!(
        config.currency_wars_shop_services(),
        CurrencyWarsContentKind::ShopService,
        |row: &CurrencyWarsShopServices| (
            row.service_kind.clone(),
            row.price_rule.clone(),
            row.inventory_rule.clone(),
            row.refresh_rule.clone()
        )
    );
    for row in config.currency_wars_service_offer_rules().ordered_rows() {
        let mut refs =
            single_reference(CurrencyWarsReferenceKind::Service, row.service_id.as_ref());
        refs.extend(references(
            CurrencyWarsReferenceKind::Candidate,
            row.candidate_ids.as_ref(),
        )?);
        records.push(record(
            &row.stable_key,
            None,
            CurrencyWarsContentKind::ServiceOfferRule,
            refs,
            Box::new([]),
            &(&row.weights, &row.fallback),
        )?);
    }

    CurrencyWarsContentCatalog::new(records).map_err(debug_error)
}

fn record(
    stable_key: &str,
    source_id: Option<&String>,
    kind: CurrencyWarsContentKind,
    references: Vec<CurrencyWarsContentReference>,
    effect_ids: Box<[Box<str>]>,
    attributes: &impl Serialize,
) -> Result<CurrencyWarsContentRecord, CurrencyWarsDataError> {
    Ok(CurrencyWarsContentRecord {
        stable_key: stable_key.into(),
        source_id: source_id
            .filter(|value| !value.is_empty())
            .map(|value| value.clone().into_boxed_str()),
        kind,
        references: references.into_boxed_slice(),
        effect_ids,
        attributes_json: serde_json::to_string(attributes)
            .map_err(debug_error)?
            .into_boxed_str(),
    })
}

fn references(
    kind: CurrencyWarsReferenceKind,
    value: Option<&String>,
) -> Result<Vec<CurrencyWarsContentReference>, CurrencyWarsDataError> {
    let values = value
        .filter(|value| !value.is_empty())
        .map_or_else(|| Ok(Vec::new()), |value| parse_json::<Vec<String>>(value))?;
    Ok(values
        .into_iter()
        .map(|target| CurrencyWarsContentReference {
            kind,
            target: target.into_boxed_str(),
        })
        .collect())
}

fn single_reference(
    kind: CurrencyWarsReferenceKind,
    value: Option<&String>,
) -> Vec<CurrencyWarsContentReference> {
    value
        .filter(|value| !value.is_empty())
        .map(|target| {
            vec![CurrencyWarsContentReference {
                kind,
                target: target.clone().into_boxed_str(),
            }]
        })
        .unwrap_or_default()
}

fn occurrence_parent(value: Option<&String>) -> Vec<CurrencyWarsContentReference> {
    let kind = if value.is_some_and(|value| value.contains("occurrence-variant")) {
        CurrencyWarsReferenceKind::OccurrenceVariant
    } else {
        CurrencyWarsReferenceKind::Occurrence
    };
    single_reference(kind, value)
}
