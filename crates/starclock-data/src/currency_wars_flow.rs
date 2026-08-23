use std::collections::BTreeMap;

use serde::Deserialize;
use starclock_combat::{EncounterId, Ratio, Scalar};
use starclock_mode_currency_wars::{
    CurrencyWarsAreaGroup, CurrencyWarsAreaSelectionPolicy, CurrencyWarsBattlePenaltyRule,
    CurrencyWarsBattleStageFinish, CurrencyWarsDifficulty, CurrencyWarsDifficultyEnemyScaling,
    CurrencyWarsDomainComposition, CurrencyWarsDomainFallback, CurrencyWarsDomainSelectionPolicy,
    CurrencyWarsEntry, CurrencyWarsEntryKind, CurrencyWarsEntryRule, CurrencyWarsFinishCondition,
    CurrencyWarsFinishRule, CurrencyWarsFlowCatalog, CurrencyWarsFlowCatalogParts,
    CurrencyWarsGambit, CurrencyWarsGambitDefinition, CurrencyWarsLayer, CurrencyWarsModule,
    CurrencyWarsNode, CurrencyWarsNodeId, CurrencyWarsNodeKind, CurrencyWarsProfile,
    CurrencyWarsRoom, CurrencyWarsRoomReachability, CurrencyWarsRoute, CurrencyWarsRouteId,
    CurrencyWarsRouteTransitionRule, CurrencyWarsStageFlow, CurrencyWarsTransitionKind,
    CurrencyWarsUnlockCondition,
};

use crate::{
    currency_wars::{CurrencyWarsDataError, debug_error, error},
    currency_wars_generated::SoraConfig,
    currency_wars_rank::lower_currency_wars_rank_progression,
};

pub(super) fn lower_currency_wars_flow(
    config: &SoraConfig,
) -> Result<CurrencyWarsFlowCatalog, CurrencyWarsDataError> {
    let nodes = lower_nodes(config)?;
    let parts = CurrencyWarsFlowCatalogParts {
        profile: lower_profile(config)?,
        modules: lower_modules(config)?,
        entries: lower_entries(config)?,
        gambits: lower_gambits(config)?,
        finish_conditions: lower_finish_conditions(config)?,
        area_group: lower_area_group(config)?,
        routes: lower_routes(config, &nodes)?,
        difficulties: lower_difficulties(config)?,
        layers: lower_layers(config, &nodes.identities)?,
        rooms: lower_rooms(config)?,
        domain_compositions: lower_domain_compositions(config)?,
        stage_flow: lower_stage_flow(config, &nodes)?,
        rank_progression: lower_currency_wars_rank_progression(config)?,
    };
    CurrencyWarsFlowCatalog::new(parts).map_err(debug_error)
}

fn lower_profile(config: &SoraConfig) -> Result<CurrencyWarsProfile, CurrencyWarsDataError> {
    let mut rows = config.currency_wars_profiles().ordered_rows();
    let row = rows
        .next()
        .ok_or_else(|| error("Currency Wars profile is missing"))?;
    if rows.next().is_some() {
        return Err(error("Currency Wars profile is duplicated"));
    }
    Ok(CurrencyWarsProfile {
        stable_key: row.stable_key.clone().into(),
        entry_ids: parse_boxed_strings(row.entry_refs.as_ref())?,
        module_id: required(&row.module_id, "profile module")?.into(),
        gambits: parse_gambit_ids(row.gambit_mode_ids.as_ref())?,
        initial_resource_ids: parse_boxed_strings(row.initial_resources.as_ref())?,
        finish_condition_ids: parse_boxed_strings(row.finish_condition_ids.as_ref())?,
    })
}

fn lower_modules(config: &SoraConfig) -> Result<Vec<CurrencyWarsModule>, CurrencyWarsDataError> {
    config
        .currency_wars_modules()
        .ordered_rows()
        .map(|row| {
            if required(&row.sub_mode, "module sub-mode")? != "GridFight"
                || row
                    .tourn_mode
                    .as_deref()
                    .is_some_and(|value| !value.is_empty())
            {
                return Err(error("Currency Wars module mode is invalid"));
            }
            Ok(CurrencyWarsModule {
                stable_key: row.stable_key.clone().into(),
                source_id: stable_tail(&row.stable_key)?,
                season_id: parse_required(row.main_tourn_id.as_ref(), "module season")?,
                sub_season_id: parse_required(row.sub_tourn_id.as_ref(), "module sub-season")?,
            })
        })
        .collect()
}

fn lower_entries(config: &SoraConfig) -> Result<Vec<CurrencyWarsEntry>, CurrencyWarsDataError> {
    config
        .currency_wars_entries()
        .ordered_rows()
        .map(|row| {
            Ok(CurrencyWarsEntry {
                stable_key: row.stable_key.clone().into(),
                kind: match required(&row.entry_kind, "entry kind")? {
                    "GuideData" => CurrencyWarsEntryKind::GuideData,
                    "GuideTab" => CurrencyWarsEntryKind::GuideTab,
                    _ => return Err(error("Currency Wars entry kind is unknown")),
                },
                module_id: required(&row.module_id, "entry module")?.into(),
                unlocks: parse_unlocks(row.unlock_ids.as_ref())?,
                gambits: parse_gambit_ids(row.gambit_mode_ids.as_ref())?,
            })
        })
        .collect()
}

fn lower_gambits(
    config: &SoraConfig,
) -> Result<Vec<CurrencyWarsGambitDefinition>, CurrencyWarsDataError> {
    config
        .currency_wars_gambit_modes()
        .ordered_rows()
        .map(|row| {
            let gambit = parse_gambit(required(&row.mode_kind, "Gambit kind")?)?;
            let entry_rules = parse_strings(row.entry_rules.as_ref())?
                .into_iter()
                .map(|rule| match rule.as_str() {
                    "Challenge difficulty is bounded by the current highest rank." => {
                        Ok(CurrencyWarsEntryRule::StandardDifficultyBoundedByHighestRank)
                    }
                    "Victory may advance rank; defeat does not reduce current rank." => {
                        Ok(CurrencyWarsEntryRule::StandardVictoryMayAdvanceAndDefeatPreservesRank)
                    }
                    "Challenge difficulty cannot exceed the highest Standard Gambit rank." => {
                        Ok(CurrencyWarsEntryRule::OverclockDifficultyBoundedByHighestStandardRank)
                    }
                    "Completion does not change current rank." => {
                        Ok(CurrencyWarsEntryRule::OverclockCompletionPreservesRank)
                    }
                    _ => Err(error("Currency Wars Gambit entry rule is unknown")),
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(CurrencyWarsGambitDefinition {
                stable_key: row.stable_key.clone().into(),
                gambit,
                unlocks: parse_unlocks(row.unlock_ids.as_ref())?,
                entry_rules: entry_rules.into_boxed_slice(),
                initial_resource_ids: parse_boxed_strings(row.initial_resources.as_ref())?,
            })
        })
        .collect()
}

fn lower_finish_conditions(
    config: &SoraConfig,
) -> Result<Vec<CurrencyWarsFinishCondition>, CurrencyWarsDataError> {
    config
        .currency_wars_finish_conditions()
        .ordered_rows()
        .map(|row| {
            let kind = required(&row.condition_kind, "finish-condition kind")?;
            let terminal = required(&row.terminal_disposition, "finish terminal disposition")?;
            let rule = match kind {
                "SettlementRank" => {
                    if terminal != "ClassifySettlement" {
                        return Err(error("Currency Wars settlement disposition is invalid"));
                    }
                    let values: SettlementRankParameters =
                        parse_json(required(&row.parameters, "settlement parameters")?)?;
                    CurrencyWarsFinishRule::SettlementRank {
                        left_inclusive: parse_optional_text_number(&values.left_inclusive)?,
                        right_inclusive: parse_optional_text_number(&values.right_inclusive)?,
                        rank_type: (!values.rank_type.is_empty())
                            .then(|| values.rank_type.into_boxed_str()),
                    }
                }
                "BattleStageRule" => {
                    if terminal != "ProjectBattleResultToRun" {
                        return Err(error("Currency Wars battle disposition is invalid"));
                    }
                    let values: BattleStageParameters =
                        parse_json(required(&row.parameters, "battle-stage parameters")?)?;
                    CurrencyWarsFinishRule::BattleStage(CurrencyWarsBattleStageFinish {
                        stage_rule_id: values.stage_rule_id.parse().map_err(debug_error)?,
                        total_turns: values.total_turn.parse().map_err(debug_error)?,
                        threshold_position: Ratio::from_scaled(parse_decimal(
                            &values.threshold_position,
                        )?),
                    })
                }
                "BattlePenaltyRule" => {
                    if terminal != "ResolveBattleBoundary" {
                        return Err(error("Currency Wars penalty disposition is invalid"));
                    }
                    let values: BattlePenaltyParameters =
                        parse_json(required(&row.parameters, "battle-penalty parameters")?)?;
                    CurrencyWarsFinishRule::BattlePenalty(CurrencyWarsBattlePenaltyRule {
                        source_id: stable_tail(&row.stable_key)?,
                        progress_values: parse_number_strings(values.progress_values)?,
                        hp_progress_values: parse_number_strings(values.hp_progress_values)?,
                        threshold_percent: parse_optional_text_number(&values.threshold_percent)?,
                        threshold_fail_extra_squad_hp_loss: values
                            .threshold_fail_extra_squad_hp_loss
                            .parse()
                            .map_err(debug_error)?,
                        base_squad_hp_loss: values
                            .base_squad_hp_loss
                            .parse()
                            .map_err(debug_error)?,
                        progress_penalty_coefficient: values
                            .progress_penalty_coefficient
                            .parse()
                            .map_err(debug_error)?,
                        total_turns: values.total_turn.parse().map_err(debug_error)?,
                        lethal_rescue_action_value_ratio: Ratio::from_scaled(parse_decimal(
                            &values.lethal_rescue_action_value_ratio,
                        )?),
                    })
                }
                _ => return Err(error("Currency Wars finish-condition kind is unknown")),
            };
            Ok(CurrencyWarsFinishCondition {
                stable_key: row.stable_key.clone().into(),
                rule,
            })
        })
        .collect()
}

fn lower_area_group(config: &SoraConfig) -> Result<CurrencyWarsAreaGroup, CurrencyWarsDataError> {
    let mut rows = config.currency_wars_area_groups().ordered_rows();
    let row = rows
        .next()
        .ok_or_else(|| error("Currency Wars area group is missing"))?;
    if rows.next().is_some()
        || required(&row.selection_policy, "area selection policy")?
            != "CompleteGridFightStageRouteClosure"
    {
        return Err(error("Currency Wars area-group policy is invalid"));
    }
    let transition_rules = parse_strings(row.transition_rules.as_ref())?
        .into_iter()
        .map(|rule| match rule.as_str() {
            "ChapterID and SectionID define the authored route order." => {
                Ok(CurrencyWarsRouteTransitionRule::AuthoredChapterAndSectionOrder)
            }
            "Gambit-specific route membership remains unresolved." => {
                Ok(CurrencyWarsRouteTransitionRule::GambitMembershipUnresolved)
            }
            _ => Err(error("Currency Wars route transition rule is unknown")),
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(CurrencyWarsAreaGroup {
        stable_key: row.stable_key.clone().into(),
        routes: parse_strings(row.area_ids.as_ref())?
            .into_iter()
            .map(|stable| route_id(stable_component(&stable, "route")?))
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice(),
        selection_policy: CurrencyWarsAreaSelectionPolicy::CompleteGridFightStageRouteClosure,
        transition_rules: transition_rules.into_boxed_slice(),
    })
}

struct LoweredNodes {
    by_route: BTreeMap<u32, Vec<CurrencyWarsNode>>,
    identities: BTreeMap<String, CurrencyWarsNodeId>,
    stable_by_id: BTreeMap<CurrencyWarsNodeId, String>,
    next_by_id: BTreeMap<CurrencyWarsNodeId, Option<CurrencyWarsNodeId>>,
}

fn lower_nodes(config: &SoraConfig) -> Result<LoweredNodes, CurrencyWarsDataError> {
    let identities = config
        .currency_wars_nodes()
        .ordered_rows()
        .map(|row| {
            Ok((
                row.stable_key.clone(),
                CurrencyWarsNodeId::new(u32::try_from(row.id).map_err(debug_error)?)
                    .ok_or_else(|| error("Currency Wars node ID is zero"))?,
            ))
        })
        .collect::<Result<BTreeMap<_, _>, CurrencyWarsDataError>>()?;
    let mut by_route = BTreeMap::<u32, Vec<CurrencyWarsNode>>::new();
    let mut stable_by_id = BTreeMap::new();
    let mut next_by_id = BTreeMap::new();
    for row in config.currency_wars_nodes().ordered_rows() {
        let route = stable_component(&row.stable_key, "route")?;
        let id = identities[&row.stable_key];
        let next = row
            .next_node_id
            .as_deref()
            .filter(|value| !value.is_empty())
            .map(|stable| {
                identities
                    .get(stable)
                    .copied()
                    .ok_or_else(|| error("Currency Wars next node is missing"))
            })
            .transpose()?;
        let node = CurrencyWarsNode {
            id,
            stable_key: row.stable_key.clone().into(),
            plane: stable_component(required(&row.plane_id, "node plane")?, "plane")?
                .try_into()
                .map_err(debug_error)?,
            ordinal: parse_required(row.ordinal.as_ref(), "node ordinal")?,
            kind: node_kind(required(&row.node_type, "node type")?)?,
            layer_id: required(&row.layer_id, "node layer")?.into(),
            domain_composition_id: required(&row.domain_composition_id, "node domain")?.into(),
            room_id: required(&row.room_pool_id, "node room")?.into(),
            node_template_id: parse_required(row.node_template_id.as_ref(), "node template ID")?,
            encounter: encounter_id(parse_required(row.stage_id.as_ref(), "node Stage ID")?)?,
            parameter_ids: parse_strings(row.parameter_ids.as_ref())?
                .into_iter()
                .map(|value| value.parse().map_err(debug_error))
                .collect::<Result<Vec<_>, _>>()?
                .into_boxed_slice(),
            penalty_bonus_rule_id: parse_optional_number(row.penalty_bonus_rule_id.as_ref())?,
            basic_gold_reward: parse_optional_number(row.basic_gold_reward.as_ref())?,
            next,
        };
        stable_by_id.insert(id, row.stable_key.clone());
        next_by_id.insert(id, next);
        by_route.entry(route).or_default().push(node);
    }
    Ok(LoweredNodes {
        by_route,
        identities,
        stable_by_id,
        next_by_id,
    })
}

fn lower_routes(
    config: &SoraConfig,
    nodes: &LoweredNodes,
) -> Result<Vec<CurrencyWarsRoute>, CurrencyWarsDataError> {
    config
        .currency_wars_areas()
        .ordered_rows()
        .map(|row| {
            let raw = stable_component(&row.stable_key, "route")?;
            let mut route_nodes = nodes
                .by_route
                .get(&raw)
                .cloned()
                .ok_or_else(|| error("Currency Wars route has no nodes"))?;
            route_nodes.sort_by_key(|node| (node.plane, node.ordinal));
            Ok(CurrencyWarsRoute {
                id: route_id(raw)?,
                stable_key: row.stable_key.clone().into(),
                map_entry_id: parse_required(row.map_entry_id.as_ref(), "route map entry")?,
                difficulty_ids: parse_strings(row.difficulty_ids.as_ref())?
                    .into_iter()
                    .map(|stable| stable_tail(&stable))
                    .collect::<Result<Vec<_>, _>>()?
                    .into_boxed_slice(),
                layer_ids: parse_boxed_strings(row.layer_ids.as_ref())?,
                nodes: route_nodes.into_boxed_slice(),
            })
        })
        .collect()
}

fn lower_difficulties(
    config: &SoraConfig,
) -> Result<Vec<CurrencyWarsDifficulty>, CurrencyWarsDataError> {
    config
        .currency_wars_difficulties()
        .ordered_rows()
        .map(|row| {
            let rank: RankBounds = parse_json(required(&row.rank_bounds, "difficulty rank")?)?;
            let gambit: GambitRules =
                parse_json(required(&row.gambit_rules, "difficulty Gambit rules")?)?;
            let scaling: EnemyScaling =
                parse_json(required(&row.enemy_scaling, "difficulty enemy scaling")?)?;
            Ok(CurrencyWarsDifficulty {
                source_id: stable_tail(&row.stable_key)?,
                stable_key: row.stable_key.clone().into(),
                season_id: rank.season_id.parse().map_err(debug_error)?,
                division_level: rank.division_level.parse().map_err(debug_error)?,
                progress: rank.progress.parse().map_err(debug_error)?,
                standard_score_rule: gambit.standard_score_rule.parse().map_err(debug_error)?,
                overclock_score_rule: gambit.overclock_score_rule.parse().map_err(debug_error)?,
                weekly_score_modifier: Ratio::from_scaled(parse_decimal(
                    &gambit.weekly_score_modifier,
                )?),
                experience_modifier: Ratio::from_scaled(parse_decimal(
                    &gambit.experience_modifier,
                )?),
                enemy_scaling_refs: parse_boxed_strings(row.enemy_scaling_refs.as_ref())?,
                enemy_scaling: CurrencyWarsDifficultyEnemyScaling {
                    enemy_difficulty_level: scaling
                        .enemy_difficulty_level
                        .parse()
                        .map_err(debug_error)?,
                    level_base_hp_ratio: Scalar::from_scaled(parse_decimal(
                        &scaling.level_base_hp_ratio,
                    )?),
                    level_base_attack_ratio: Scalar::from_scaled(parse_decimal(
                        &scaling.level_base_attack_ratio,
                    )?),
                },
                enemy_affix_choice_counts: parse_strings(row.enemy_affix_choice_counts.as_ref())?
                    .into_iter()
                    .map(|value| value.parse().map_err(debug_error))
                    .collect::<Result<Vec<_>, _>>()?
                    .into_boxed_slice(),
                binary_difficulty_rule: row
                    .binary_difficulty_rule
                    .as_deref()
                    .filter(|value| !value.is_empty())
                    .map(|value| value.parse().map_err(debug_error))
                    .transpose()?,
            })
        })
        .collect()
}

fn lower_layers(
    config: &SoraConfig,
    node_ids: &BTreeMap<String, CurrencyWarsNodeId>,
) -> Result<Vec<CurrencyWarsLayer>, CurrencyWarsDataError> {
    config
        .currency_wars_layers()
        .ordered_rows()
        .map(|row| {
            let plane = parse_required(row.layer_number.as_ref(), "layer number")?;
            if stable_component(required(&row.plane_id, "layer plane")?, "plane")?
                != u32::from(plane)
            {
                return Err(error("Currency Wars layer plane reference is invalid"));
            }
            Ok(CurrencyWarsLayer {
                stable_key: row.stable_key.clone().into(),
                route: route_id(stable_component(&row.stable_key, "route")?)?,
                plane,
                nodes: parse_strings(row.ordered_node_ids.as_ref())?
                    .into_iter()
                    .map(|stable| {
                        node_ids
                            .get(&stable)
                            .copied()
                            .ok_or_else(|| error("Currency Wars layer node is missing"))
                    })
                    .collect::<Result<Vec<_>, _>>()?
                    .into_boxed_slice(),
            })
        })
        .collect()
}

fn lower_rooms(config: &SoraConfig) -> Result<Vec<CurrencyWarsRoom>, CurrencyWarsDataError> {
    config
        .currency_wars_rooms()
        .ordered_rows()
        .map(|row| {
            if required(&row.reachability_disposition, "room reachability")?
                != "DirectGridFightNodeType"
            {
                return Err(error("Currency Wars room reachability is unknown"));
            }
            Ok(CurrencyWarsRoom {
                stable_key: row.stable_key.clone().into(),
                kind: node_kind(required(&row.room_type, "room type")?)?,
                reachability: CurrencyWarsRoomReachability::DirectGridFightNodeType,
                stage_refs: parse_strings(row.stage_refs.as_ref())?
                    .into_iter()
                    .map(|value| value.parse().map_err(debug_error).and_then(encounter_id))
                    .collect::<Result<Vec<_>, _>>()?
                    .into_boxed_slice(),
            })
        })
        .collect()
}

fn lower_domain_compositions(
    config: &SoraConfig,
) -> Result<Vec<CurrencyWarsDomainComposition>, CurrencyWarsDataError> {
    config
        .currency_wars_domain_compositions()
        .ordered_rows()
        .map(|row| {
            if required(&row.selection_policy, "domain selection policy")? != "ExactNodeType"
                || required(&row.fallback, "domain fallback")? != "RejectUnknownNodeType"
            {
                return Err(error("Currency Wars domain policy is unknown"));
            }
            Ok(CurrencyWarsDomainComposition {
                stable_key: row.stable_key.clone().into(),
                kind: node_kind(required(&row.domain_type, "domain type")?)?,
                room_ids: parse_boxed_strings(row.room_candidate_ids.as_ref())?,
                selection_policy: CurrencyWarsDomainSelectionPolicy::ExactNodeType,
                fallback: CurrencyWarsDomainFallback::RejectUnknownNodeType,
            })
        })
        .collect()
}

fn lower_stage_flow(
    config: &SoraConfig,
    nodes: &LoweredNodes,
) -> Result<Vec<CurrencyWarsStageFlow>, CurrencyWarsDataError> {
    let flow_by_node = config
        .currency_wars_stage_flow()
        .ordered_rows()
        .map(|row| {
            let refs = parse_strings(row.ordered_node_refs.as_ref())?;
            let [node] = refs.as_slice() else {
                return Err(error("Currency Wars stage flow must own exactly one node"));
            };
            Ok((node.clone(), row.stable_key.clone()))
        })
        .collect::<Result<BTreeMap<_, _>, CurrencyWarsDataError>>()?;
    config
        .currency_wars_stage_flow()
        .ordered_rows()
        .map(|row| {
            let stable_nodes = parse_strings(row.ordered_node_refs.as_ref())?;
            let flow_nodes = stable_nodes
                .iter()
                .map(|stable| {
                    nodes
                        .identities
                        .get(stable)
                        .copied()
                        .ok_or_else(|| error("Currency Wars stage-flow node is missing"))
                })
                .collect::<Result<Vec<_>, _>>()?;
            let node = flow_nodes[0];
            let carry_rules = parse_boxed_strings(row.carry_rules.as_ref())?;
            let reset_rules = parse_boxed_strings(row.reset_rules.as_ref())?;
            if !carry_rules.is_empty() || !reset_rules.is_empty() {
                return Err(error(
                    "Currency Wars stage-flow carry/reset rule is unknown",
                ));
            }
            let next = nodes.next_by_id[&node]
                .map(|next| {
                    let stable = nodes
                        .stable_by_id
                        .get(&next)
                        .ok_or_else(|| error("Currency Wars next-node identity is missing"))?;
                    flow_by_node
                        .get(stable)
                        .cloned()
                        .ok_or_else(|| error("Currency Wars next flow is missing"))
                })
                .transpose()?;
            Ok(CurrencyWarsStageFlow {
                stable_key: row.stable_key.clone().into(),
                profile_id: required(&row.entry_id, "stage-flow profile")?.into(),
                nodes: flow_nodes.into_boxed_slice(),
                transition: if next.is_some() {
                    CurrencyWarsTransitionKind::NextSection
                } else {
                    CurrencyWarsTransitionKind::PlaneTerminal
                },
                next: next.map(String::into_boxed_str),
                carry_rules: Box::new([]),
                reset_rules: Box::new([]),
            })
        })
        .collect()
}

#[derive(Deserialize)]
struct UnlockObject {
    #[serde(rename = "Param")]
    param: String,
    #[serde(rename = "Type")]
    kind: String,
}

fn parse_unlocks(
    value: Option<&String>,
) -> Result<Box<[CurrencyWarsUnlockCondition]>, CurrencyWarsDataError> {
    let values: Vec<serde_json::Value> = parse_json(value.map_or("[]", String::as_str))?;
    values
        .into_iter()
        .map(|value| match value {
            serde_json::Value::String(value) if value == "complete-one-standard-gambit" => {
                Ok(CurrencyWarsUnlockCondition::CompleteOneStandardGambit)
            }
            serde_json::Value::Object(_) => {
                let value: UnlockObject = serde_json::from_value(value).map_err(debug_error)?;
                match value.kind.as_str() {
                    "PlayerLevel" => Ok(CurrencyWarsUnlockCondition::PlayerLevel(
                        value.param.parse().map_err(debug_error)?,
                    )),
                    _ => Err(error("Currency Wars unlock condition is unknown")),
                }
            }
            _ => Err(error("Currency Wars unlock condition is invalid")),
        })
        .collect::<Result<Vec<_>, _>>()
        .map(Vec::into_boxed_slice)
}

#[derive(Deserialize)]
struct SettlementRankParameters {
    left_inclusive: String,
    right_inclusive: String,
    rank_type: String,
}

#[derive(Deserialize)]
struct BattleStageParameters {
    stage_rule_id: String,
    total_turn: String,
    threshold_position: String,
}

#[derive(Deserialize)]
struct BattlePenaltyParameters {
    progress_values: Vec<String>,
    hp_progress_values: Vec<String>,
    threshold_percent: String,
    threshold_fail_extra_squad_hp_loss: String,
    base_squad_hp_loss: String,
    progress_penalty_coefficient: String,
    total_turn: String,
    lethal_rescue_action_value_ratio: String,
}

fn parse_number_strings(values: Vec<String>) -> Result<Box<[u32]>, CurrencyWarsDataError> {
    values
        .into_iter()
        .map(|value| value.parse().map_err(debug_error))
        .collect::<Result<Vec<_>, _>>()
        .map(Vec::into_boxed_slice)
}

#[derive(Deserialize)]
struct RankBounds {
    division_level: String,
    progress: String,
    season_id: String,
}

#[derive(Deserialize)]
struct GambitRules {
    standard_score_rule: String,
    overclock_score_rule: String,
    weekly_score_modifier: String,
    experience_modifier: String,
}

#[derive(Deserialize)]
struct EnemyScaling {
    enemy_difficulty_level: String,
    level_base_hp_ratio: String,
    level_base_attack_ratio: String,
}

fn parse_gambit_ids(
    value: Option<&String>,
) -> Result<Box<[CurrencyWarsGambit]>, CurrencyWarsDataError> {
    parse_strings(value)?
        .into_iter()
        .map(|stable| {
            stable
                .rsplit('.')
                .next()
                .ok_or_else(|| error("Currency Wars Gambit ID is invalid"))
                .and_then(parse_gambit)
        })
        .collect::<Result<Vec<_>, _>>()
        .map(Vec::into_boxed_slice)
}

fn parse_gambit(value: &str) -> Result<CurrencyWarsGambit, CurrencyWarsDataError> {
    match value {
        "Standard" | "standard" => Ok(CurrencyWarsGambit::Standard),
        "Overclock" | "overclock" => Ok(CurrencyWarsGambit::Overclock),
        _ => Err(error("Currency Wars Gambit is unknown")),
    }
}

fn node_kind(value: &str) -> Result<CurrencyWarsNodeKind, CurrencyWarsDataError> {
    match value {
        "Monster" => Ok(CurrencyWarsNodeKind::Monster),
        "CampMonster" => Ok(CurrencyWarsNodeKind::CampMonster),
        "EliteBranch" => Ok(CurrencyWarsNodeKind::EliteBranch),
        "Boss" => Ok(CurrencyWarsNodeKind::Boss),
        "Supply" => Ok(CurrencyWarsNodeKind::Supply),
        _ => Err(error("Currency Wars node type is unknown")),
    }
}

fn route_id(raw: u32) -> Result<CurrencyWarsRouteId, CurrencyWarsDataError> {
    CurrencyWarsRouteId::new(raw).ok_or_else(|| error("Currency Wars route ID is zero"))
}

pub(super) fn encounter_id(raw: u32) -> Result<EncounterId, CurrencyWarsDataError> {
    EncounterId::new(raw).ok_or_else(|| error("Currency Wars encounter ID is zero"))
}

pub(super) fn required<'a>(
    value: &'a Option<String>,
    name: &str,
) -> Result<&'a str, CurrencyWarsDataError> {
    value
        .as_deref()
        .filter(|value| !value.is_empty())
        .ok_or_else(|| error(&format!("{name} is missing")))
}

fn parse_required<T: std::str::FromStr>(
    value: Option<&String>,
    name: &str,
) -> Result<T, CurrencyWarsDataError>
where
    T::Err: std::fmt::Debug,
{
    value
        .filter(|value| !value.is_empty())
        .ok_or_else(|| error(&format!("{name} is missing")))?
        .parse()
        .map_err(debug_error)
}

fn parse_optional_number<T: std::str::FromStr>(
    value: Option<&String>,
) -> Result<Option<T>, CurrencyWarsDataError>
where
    T::Err: std::fmt::Debug,
{
    value
        .map(String::as_str)
        .filter(|value| !value.is_empty() && *value != "undefined")
        .map(|value| value.parse().map_err(debug_error))
        .transpose()
}

fn parse_optional_text_number<T: std::str::FromStr>(
    value: &str,
) -> Result<Option<T>, CurrencyWarsDataError>
where
    T::Err: std::fmt::Debug,
{
    (!value.is_empty())
        .then(|| value.parse().map_err(debug_error))
        .transpose()
}

pub(super) fn parse_boxed_strings(
    value: Option<&String>,
) -> Result<Box<[Box<str>]>, CurrencyWarsDataError> {
    parse_strings(value).map(|values| {
        values
            .into_iter()
            .map(String::into_boxed_str)
            .collect::<Vec<_>>()
            .into_boxed_slice()
    })
}

fn parse_strings(value: Option<&String>) -> Result<Vec<String>, CurrencyWarsDataError> {
    parse_json(value.map_or("[]", String::as_str))
}

pub(super) fn parse_json<T: for<'de> Deserialize<'de>>(
    value: &str,
) -> Result<T, CurrencyWarsDataError> {
    serde_json::from_str(value).map_err(debug_error)
}

fn stable_tail(stable: &str) -> Result<u32, CurrencyWarsDataError> {
    stable
        .rsplit('.')
        .next()
        .ok_or_else(|| error("stable ID has no tail"))?
        .parse()
        .map_err(debug_error)
}

fn stable_component(stable: &str, label: &str) -> Result<u32, CurrencyWarsDataError> {
    let values = stable.split('.').collect::<Vec<_>>();
    values
        .windows(2)
        .find(|pair| pair[0] == label)
        .map(|pair| pair[1])
        .ok_or_else(|| error(&format!("stable ID has no {label} component")))?
        .parse()
        .map_err(debug_error)
}

pub(super) fn parse_decimal(source: &str) -> Result<i64, CurrencyWarsDataError> {
    let (negative, unsigned) = source
        .strip_prefix('-')
        .map_or((false, source), |rest| (true, rest));
    let mut parts = unsigned.split('.');
    let integer = parts.next().unwrap_or_default();
    let fraction = parts.next();
    if unsigned.is_empty()
        || (negative && unsigned == "0")
        || parts.next().is_some()
        || integer.is_empty()
        || !integer.bytes().all(|byte| byte.is_ascii_digit())
        || (integer.len() > 1 && integer.starts_with('0'))
        || fraction.is_some_and(|value| {
            value.is_empty()
                || value.len() > 6
                || !value.bytes().all(|byte| byte.is_ascii_digit())
                || value.ends_with('0')
        })
    {
        return Err(error("Currency Wars decimal is not canonical"));
    }
    let integer = integer.parse::<i128>().map_err(debug_error)?;
    let fraction = fraction.unwrap_or("");
    let fraction = if fraction.is_empty() {
        0
    } else {
        fraction.parse::<i128>().map_err(debug_error)?
            * 10_i128.pow(6 - u32::try_from(fraction.len()).map_err(debug_error)?)
    };
    let magnitude = integer
        .checked_mul(1_000_000)
        .and_then(|value| value.checked_add(fraction))
        .ok_or_else(|| error("Currency Wars decimal overflows"))?;
    i64::try_from(if negative { -magnitude } else { magnitude }).map_err(debug_error)
}
