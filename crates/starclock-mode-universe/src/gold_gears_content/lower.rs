use crate::{
    gold_gears_catalog::GoldAndGearsBundleSummary,
    gold_gears_content::{
        GoldAndGearsContentCatalog, GoldAndGearsContentError, GoldAndGearsContentErrorKind,
        types::{
            AdventureOutcome, Blessing, BlessingLevel, BlockCreateRule, CatalogCoverage, Curio,
            CurioState, EncounterGroup, EncounterWave, EnemySlot, JsonPayload, MapEvent,
            MechanicRule, Occurrence, OccurrenceChoice, OccurrenceVariant, Service, StableIndexRow,
            StableKey,
        },
        validate,
    },
    gold_gears_generated::{SoraConfig, gold_gears_ownership::GoldGearsOwnership},
};

pub(super) fn lower(
    bundle: GoldAndGearsBundleSummary,
    source: &SoraConfig,
) -> Result<GoldAndGearsContentCatalog, GoldAndGearsContentError> {
    let catalog =
        GoldAndGearsContentCatalog {
            bundle,
            blessings: collect(source.gold_gears_blessing().ordered_rows().map(|row| {
                Ok(Blessing {
                    id: row.id,
                    key: key(&row.stable_key),
                    path: key(&row.path_id),
                    levels: keys(&row.level_ids),
                    inherited_rules: optional_keys(row.inherited_rule_ids.as_deref()),
                })
            }))?,
            blessing_levels: collect(source.gold_gears_blessing_level().ordered_rows().map(
                |row| {
                    Ok(BlessingLevel {
                        id: row.id,
                        key: key(&row.stable_key),
                        blessing_id: row.blessing_id,
                        inherited_rules: optional_keys(row.inherited_rule_ids.as_deref()),
                        parameters: json(&row.parameter_values_json, &row.stable_key)?,
                    })
                },
            ))?,
            curios: collect(source.gold_gears_curio().ordered_rows().map(|row| {
                Ok(Curio {
                    id: row.id,
                    key: key(&row.stable_key),
                    initial_state_id: row.initial_state_id,
                    states: keys(&row.state_ids),
                    rule: key(&row.rule_contribution_id),
                    shared: row.ownership == GoldGearsOwnership::Shared,
                })
            }))?,
            curio_states: collect(source.gold_gears_curio_state().ordered_rows().map(|row| {
                Ok(CurioState {
                    id: row.id,
                    key: key(&row.stable_key),
                    curio_id: row.curio_id,
                    rule: key(&row.rule_contribution_id),
                    payloads: jsons(
                        [
                            &row.lifecycle_json,
                            &row.parameter_values_json,
                            &row.display_parameter_values_json,
                            &row.repair_target_json,
                            &row.selection_policy_json,
                        ],
                        &row.stable_key,
                    )?,
                })
            }))?,
            occurrences: collect(source.gold_gears_occurrence().ordered_rows().map(|row| {
                Ok(Occurrence {
                    id: row.id,
                    key: key(&row.stable_key),
                    variants: keys(&row.variant_ids),
                    rule: key(&row.rule_contribution_id),
                })
            }))?,
            occurrence_variants: collect(
                source
                    .gold_gears_occurrence_variant()
                    .ordered_rows()
                    .map(|row| {
                        Ok(OccurrenceVariant {
                            id: row.id,
                            key: key(&row.stable_key),
                            occurrence_id: row.occurrence_id,
                            occurrence_keys: keys(&row.occurrence_ids),
                            choices: keys(&row.choice_ids),
                            rule: key(&row.rule_contribution_id),
                        })
                    }),
            )?,
            occurrence_choices: collect(source.gold_gears_occurrence_choice().ordered_rows().map(
                |row| {
                    Ok(OccurrenceChoice {
                        id: row.id,
                        key: key(&row.stable_key),
                        variant_id: row.variant_id,
                        next_node: row.next_node_id.as_deref().map(key),
                        rule: key(&row.rule_contribution_id),
                        payloads: jsons(
                            [
                                &row.costs_json,
                                &row.outcomes_json,
                                &row.parameter_vectors_json,
                                &row.dynamic_display_options_json,
                                &row.quality_overrides_json,
                            ],
                            &row.stable_key,
                        )?,
                    })
                },
            ))?,
            services: collect(source.gold_gears_service().ordered_rows().map(|row| {
                Ok(Service {
                    id: row.id,
                    key: key(&row.stable_key),
                    rule: key(&row.rule_contribution_id),
                    shared: row.ownership == GoldGearsOwnership::Shared,
                    payloads: jsons(
                        [
                            &row.parameters_json,
                            &row.selection_policy_json,
                            &row.gold_gears_offer_rule_json,
                        ],
                        &row.stable_key,
                    )?,
                })
            }))?,
            adventure_outcomes: collect(source.gold_gears_adventure_outcome().ordered_rows().map(
                |row| {
                    Ok(AdventureOutcome {
                        id: row.id,
                        key: key(&row.stable_key),
                        downloader_service_id: row.downloader_service_id,
                        room: key(&row.room_stable_key),
                        rule: key(&row.rule_contribution_id),
                        payloads: jsons(
                            [
                                &row.quality_overrides_json,
                                &row.reward_selection_policy_json,
                                &row.reward_tiers_json,
                            ],
                            &row.stable_key,
                        )?,
                    })
                },
            ))?,
            encounter_groups: collect(source.gold_gears_encounter_group().ordered_rows().map(
                |row| {
                    Ok(EncounterGroup {
                        id: row.id,
                        key: key(&row.stable_key),
                        parent_room: row.parent_room_id.as_deref().map(key),
                        areas: keys(&row.eligible_area_ids),
                        payloads: jsons(
                            [
                                &row.parent_room_scope_json,
                                &row.difficulty_binding_json,
                                &row.weighted_members_json,
                                &row.selection_policy_json,
                            ],
                            &row.stable_key,
                        )?,
                    })
                },
            ))?,
            encounter_waves: collect(source.gold_gears_encounter_wave().ordered_rows().map(
                |row| {
                    Ok(EncounterWave {
                        id: row.id,
                        key: key(&row.stable_key),
                        group_id: row.encounter_group_id,
                        slots: keys(&row.enemy_slot_ids),
                        payload: json(&row.level_binding_json, &row.stable_key)?,
                    })
                },
            ))?,
            enemy_slots: collect(source.gold_gears_enemy_slot().ordered_rows().map(|row| {
                Ok(EnemySlot {
                    id: row.id,
                    key: key(&row.stable_key),
                    wave_id: row.encounter_wave_id,
                    enemy: key(&row.enemy_variant_id),
                    boss_choices: optional_keys(row.boss_choice_ids.as_deref()),
                })
            }))?,
            map_events: collect(source.gold_gears_map_event().ordered_rows().map(|row| {
                Ok(MapEvent {
                    id: row.id,
                    key: key(&row.stable_key),
                    chessboard_id: row.chessboard_id,
                    parameters: [
                        row.trigger_params.as_deref().unwrap_or_default(),
                        row.effect_params.as_deref().unwrap_or_default(),
                        row.secondary_effect_params.as_deref().unwrap_or_default(),
                    ]
                    .into_iter()
                    .flatten()
                    .chain([&row.trigger_type, &row.effect_type, &row.weight])
                    .map(|value| value.as_str().into())
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
                })
            }))?,
            block_create_rules: collect(source.gold_gears_block_create_rule().ordered_rows().map(
                |row| {
                    Ok(BlockCreateRule {
                        id: row.id,
                        key: key(&row.stable_key),
                        chessboard_id: row.chessboard_id,
                        domain_id: row.domain_id,
                        payloads: jsons(
                            [&row.create_count_weights_json, &row.beacon_weights_json],
                            &row.stable_key,
                        )?,
                    })
                },
            ))?,
            mechanic_rules: collect(source.gold_gears_mechanic_rule().ordered_rows().map(|row| {
                Ok(MechanicRule {
                    id: row.id,
                    key: key(&row.stable_key),
                    owner: key(&row.owner_id),
                    fixtures: keys(&row.fixture_ids),
                    disposition: row.execution_disposition.clone().into(),
                    policy_bound: row.policy_bound,
                    payloads: jsons(
                        [
                            &row.effect_contributions_json,
                            &row.outcome_program_json,
                            &row.parameter_values_json,
                            &row.selection_policy_json,
                            &row.source_binding_json,
                            &row.state_contract_json,
                        ],
                        &row.stable_key,
                    )?,
                })
            }))?,
            source_records: source
                .gold_gears_source_record()
                .ordered_rows()
                .map(|row| StableIndexRow {
                    id: row.id,
                    key: key(&row.stable_key),
                })
                .collect::<Vec<_>>()
                .into_boxed_slice(),
            coverage: source
                .gold_gears_coverage()
                .ordered_rows()
                .map(|row| CatalogCoverage {
                    id: row.id,
                    key: key(&row.stable_key),
                    category: row.category_id.clone().into(),
                    required: row.required,
                    accounted: row.accounted,
                    data_ready: row.data_ready,
                    blocking_gaps: optional_keys(row.blocking_gap_ids.as_deref()),
                })
                .collect::<Vec<_>>()
                .into_boxed_slice(),
            research_gaps: source
                .gold_gears_research_gap()
                .ordered_rows()
                .map(|row| StableIndexRow {
                    id: row.id,
                    key: key(&row.stable_key),
                })
                .collect::<Vec<_>>()
                .into_boxed_slice(),
            gap_affected_records: source
                .gold_gears_research_gap_affected_record()
                .ordered_rows()
                .map(|row| StableIndexRow {
                    id: row.id,
                    key: key(&format!(
                        "{}:{}:{}",
                        row.research_gap_id, row.ordinal, row.record_stable_key
                    )),
                })
                .collect::<Vec<_>>()
                .into_boxed_slice(),
            review_fixtures: source
                .gold_gears_review_fixture()
                .ordered_rows()
                .map(|row| StableIndexRow {
                    id: row.id,
                    key: key(&row.stable_key),
                })
                .collect::<Vec<_>>()
                .into_boxed_slice(),
            pack_index: source
                .gold_gears_pack_index()
                .ordered_rows()
                .map(|row| StableIndexRow {
                    id: row.id,
                    key: key(&row.stable_key),
                })
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        };
    validate::validate(&catalog, source)?;
    Ok(catalog)
}

fn collect<T>(
    rows: impl Iterator<Item = Result<T, GoldAndGearsContentError>>,
) -> Result<Box<[T]>, GoldAndGearsContentError> {
    rows.collect::<Result<Vec<_>, _>>()
        .map(Vec::into_boxed_slice)
}

fn key(value: &str) -> StableKey {
    StableKey::new(value)
}

fn keys(values: &[String]) -> Box<[StableKey]> {
    values
        .iter()
        .map(|value| key(value))
        .collect::<Vec<_>>()
        .into_boxed_slice()
}

fn optional_keys(values: Option<&[String]>) -> Box<[StableKey]> {
    values.map_or_else(|| Box::new([]), keys)
}

fn json(value: &str, owner: &str) -> Result<JsonPayload, GoldAndGearsContentError> {
    serde_json::from_str::<serde_json::Value>(value).map_err(|_| GoldAndGearsContentError {
        kind: GoldAndGearsContentErrorKind::Json,
        key: owner.into(),
    })?;
    Ok(JsonPayload::new(value))
}

fn jsons<const N: usize>(
    values: [&str; N],
    owner: &str,
) -> Result<Box<[JsonPayload]>, GoldAndGearsContentError> {
    values
        .into_iter()
        .map(|value| json(value, owner))
        .collect::<Result<Vec<_>, _>>()
        .map(Vec::into_boxed_slice)
}
