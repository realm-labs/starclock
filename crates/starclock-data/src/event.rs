//! Production Sora lowering for retained event-mode profiles.

use starclock_combat::EncounterId;
use starclock_mode_baseballer::{
    BaseballerAdventureStrategy, BaseballerAdventureStrategyId, BaseballerAdventureStrategyKind,
    BaseballerCatalog, BaseballerEquipment, BaseballerEquipmentId, BaseballerEquipmentKind,
    BaseballerPeriodRank, BaseballerProfile, BaseballerProfileId, BaseballerProfileKind,
    BaseballerRecipe, BaseballerRecipeInput, BaseballerRecipeInputKind, BaseballerRecipeTier,
    BaseballerRuntimeCatalogContent, BaseballerRuntimePolicy, BaseballerScoreRule,
    BaseballerShopUpgrade, BaseballerShopUpgradeId, BaseballerShopUpgradeKind, BaseballerStage,
    BaseballerStageId, BaseballerStagePeriod, BaseballerStagePeriodId, BaseballerTeamBonus,
};
use starclock_mode_fate_night::{
    FateBoard, FateBoardEdge, FateBoardNode, FateBoardNodeKind, FateCard, FateCardId,
    FateCardOwner, FateCardRarity, FateCatalog, FateCatalogParts, FateChallengeFight,
    FateChallengeFightId, FateDeck, FateDeckId, FateDeckRecommendation, FateDeckRecommendationId,
    FateDeckRecommendationKind, FateMapFight, FateMapFightId, FateRuntimePolicy, FateStoryFight,
    FateStoryFightId,
};

use crate::event_generated::{
    SoraConfig, fate_runtime_board_node_kind::FateRuntimeBoardNodeKind as GeneratedBoardNodeKind,
    fate_runtime_card_owner::FateRuntimeCardOwner as GeneratedCardOwner,
    fate_runtime_card_rarity::FateRuntimeCardRarity as GeneratedCardRarity,
    fate_runtime_deck_kind::FateRuntimeDeckKind as GeneratedDeckKind,
    gb_runtime_equipment_kind::GbRuntimeEquipmentKind, gb_runtime_input_kind::GbRuntimeInputKind,
    gb_runtime_period_rank::GbRuntimePeriodRank, gb_runtime_profile_kind::GbRuntimeProfileKind,
    gb_runtime_shop_upgrade_kind::GbRuntimeShopUpgradeKind,
    gb_runtime_strategy_kind::GbRuntimeStrategyKind, runtime::SoraBundle,
};

const PRODUCTION_BUNDLE: &[u8] =
    include_bytes!("../../../config/event-runtime-generated/config.sora");

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventDataError {
    message: Box<str>,
}

impl std::fmt::Display for EventDataError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for EventDataError {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BaseballerRuntimeData {
    pub catalog: BaseballerCatalog,
    pub score_rules: Box<[(BaseballerProfileId, BaseballerScoreRule)]>,
    pub policies: Box<[BaseballerRuntimePolicy]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FateRuntimeData {
    pub catalog: FateCatalog,
    pub policies: Box<[FateRuntimePolicy]>,
}

pub fn galactic_baseballer() -> Result<BaseballerRuntimeData, EventDataError> {
    load_galactic_baseballer(PRODUCTION_BUNDLE)
}

pub fn fate_star_rail_night() -> Result<FateRuntimeData, EventDataError> {
    load_fate_star_rail_night(PRODUCTION_BUNDLE)
}

pub fn load_galactic_baseballer(bytes: &[u8]) -> Result<BaseballerRuntimeData, EventDataError> {
    let config = parse(bytes)?;
    let profiles = config
        .gb_runtime_profiles()
        .ordered_rows()
        .map(|row| {
            Ok(BaseballerProfile {
                id: profile_id(row.id)?,
                kind: match row.kind {
                    GbRuntimeProfileKind::Departure => BaseballerProfileKind::Departure,
                    GbRuntimeProfileKind::DemonKing => BaseballerProfileKind::DemonKing,
                },
                stable_key: row.stable_key.clone().into_boxed_str(),
                weapon_slots: small(row.weapon_slots, "weapon slots")?,
                initially_unlocked_weapon_slots: small(
                    row.unlocked_weapon_slots,
                    "unlocked weapon slots",
                )?,
                accessory_slots: small(row.accessory_slots, "accessory slots")?,
                initially_unlocked_accessory_slots: small(
                    row.unlocked_accessory_slots,
                    "unlocked accessory slots",
                )?,
            })
        })
        .collect::<Result<Vec<_>, EventDataError>>()?;
    let equipment = config
        .gb_runtime_equipment()
        .ordered_rows()
        .map(|row| {
            Ok(BaseballerEquipment {
                id: equipment_id(row.id)?,
                stable_key: row.stable_key.clone().into_boxed_str(),
                kind: match row.kind {
                    GbRuntimeEquipmentKind::StandardWeapon => {
                        BaseballerEquipmentKind::StandardWeapon
                    }
                    GbRuntimeEquipmentKind::LegendaryWeapon => {
                        BaseballerEquipmentKind::LegendaryWeapon
                    }
                    GbRuntimeEquipmentKind::Accessory => BaseballerEquipmentKind::Accessory,
                },
                maximum_level: small(row.maximum_level, "equipment maximum level")?,
                profiles: row
                    .profile_ids
                    .iter()
                    .copied()
                    .map(profile_id)
                    .collect::<Result<Vec<_>, _>>()?
                    .into_boxed_slice(),
                runtime_binding_exact: row.runtime_binding_exact,
            })
        })
        .collect::<Result<Vec<_>, EventDataError>>()?;
    let shop_upgrades = config
        .gb_runtime_shop_upgrades()
        .ordered_rows()
        .map(|row| {
            Ok(BaseballerShopUpgrade {
                id: BaseballerShopUpgradeId::new(unsigned(row.id, "shop upgrade row id")?)
                    .ok_or_else(|| error("shop upgrade row id must be non-zero"))?,
                stable_key: row.stable_key.clone().into_boxed_str(),
                profile: profile_id(row.profile_id)?,
                source_numeric_id: unsigned(
                    row.source_numeric_id,
                    "shop upgrade source numeric id",
                )?,
                purchase_level: small(row.purchase_level, "shop upgrade purchase level")?,
                maximum_level: small(row.maximum_level, "shop upgrade maximum level")?,
                kind: match row.kind {
                    GbRuntimeShopUpgradeKind::AddMazeBuff => BaseballerShopUpgradeKind::AddMazeBuff,
                    GbRuntimeShopUpgradeKind::InitWeaponLevel => {
                        BaseballerShopUpgradeKind::InitWeaponLevel
                    }
                    GbRuntimeShopUpgradeKind::AddAccessorySlot => {
                        BaseballerShopUpgradeKind::AddAccessorySlot
                    }
                },
                currency_key: row.currency_key.clone().into_boxed_str(),
                cost: row.cost,
                maze_buff_id: row
                    .maze_buff_id
                    .map(|value| unsigned(value, "shop MazeBuff id"))
                    .transpose()?,
                maze_buff_parameters: boxed_strings(
                    row.maze_buff_parameters.as_deref().unwrap_or_default(),
                ),
                shop_parameter_values: boxed_strings(&row.shop_parameter_values),
                runtime_binding_exact: row.runtime_binding_exact,
            })
        })
        .collect::<Result<Vec<_>, EventDataError>>()?;
    let stages = config
        .gb_runtime_stages()
        .ordered_rows()
        .map(|row| {
            Ok(BaseballerStage {
                id: BaseballerStageId::new(unsigned(row.id, "Baseballer stage id")?)
                    .ok_or_else(|| error("Baseballer stage id must be non-zero"))?,
                profile: profile_id(row.profile_id)?,
                difficulty: small(row.difficulty, "stage difficulty")?,
                weapon_selectable: row.weapon_selectable,
                initial_weapons: row
                    .initial_weapon_ids
                    .as_deref()
                    .unwrap_or_default()
                    .iter()
                    .copied()
                    .map(equipment_id)
                    .collect::<Result<Vec<_>, _>>()?
                    .into_boxed_slice(),
                rating_thresholds: row.rating_thresholds.clone().into_boxed_slice(),
            })
        })
        .collect::<Result<Vec<_>, EventDataError>>()?;
    let stage_periods = config
        .gb_runtime_stage_periods()
        .ordered_rows()
        .map(|row| {
            Ok(BaseballerStagePeriod {
                id: BaseballerStagePeriodId::new(unsigned(row.id, "Baseballer period id")?)
                    .ok_or_else(|| error("Baseballer period id must be non-zero"))?,
                stage: BaseballerStageId::new(unsigned(row.stage_id, "Baseballer stage id")?)
                    .ok_or_else(|| error("Baseballer stage id must be non-zero"))?,
                rank: match row.rank {
                    GbRuntimePeriodRank::First => BaseballerPeriodRank::First,
                    GbRuntimePeriodRank::Second => BaseballerPeriodRank::Second,
                    GbRuntimePeriodRank::Third => BaseballerPeriodRank::Third,
                    GbRuntimePeriodRank::Extra => BaseballerPeriodRank::Extra,
                },
                encounter: EncounterId::new(unsigned(
                    row.encounter_id,
                    "Baseballer period encounter id",
                )?)
                .ok_or_else(|| error("Baseballer period encounter id must be non-zero"))?,
                battle_event_id: unsigned(
                    row.battle_event_id,
                    "Baseballer period battle event id",
                )?,
                wave_count: small(row.wave_count, "Baseballer period wave count")?,
                countdown_by_wave: row
                    .countdown_by_wave
                    .iter()
                    .copied()
                    .map(|value| {
                        u16::try_from(value)
                            .map_err(|_| error("Baseballer period countdown exceeds u16"))
                    })
                    .collect::<Result<Vec<_>, _>>()?
                    .into_boxed_slice(),
                period_score: row.period_score,
                stage_score: row.stage_score,
                selection_weight: unsigned(
                    row.selection_weight,
                    "Baseballer period selection weight",
                )?,
            })
        })
        .collect::<Result<Vec<_>, EventDataError>>()?;
    let recipes = config
        .gb_runtime_recipes()
        .ordered_rows()
        .map(|row| {
            let inputs = config
                .gb_runtime_recipe_inputs()
                .ordered_rows()
                .filter(|input| input.recipe_id == row.id)
                .map(|input| {
                    let kind = match input.kind {
                        GbRuntimeInputKind::Equipment => BaseballerRecipeInputKind::Equipment(
                            equipment_id(input.equipment_id.ok_or_else(|| {
                                error("equipment recipe input has no equipment id")
                            })?)?,
                        ),
                        GbRuntimeInputKind::AnyStandardWeapon => {
                            BaseballerRecipeInputKind::AnyStandardWeapon
                        }
                    };
                    Ok(BaseballerRecipeInput {
                        order: small(input.input_order, "recipe input order")?,
                        kind,
                        required_level: small(input.required_level, "required level")?,
                        consumed: input.consumed,
                    })
                })
                .collect::<Result<Vec<_>, EventDataError>>()?;
            Ok(BaseballerRecipe {
                id: unsigned(row.id, "recipe id")?,
                profile: profile_id(row.profile_id)?,
                tier: match row.tier.as_str() {
                    "Supreme" => BaseballerRecipeTier::Supreme,
                    "Twin" => BaseballerRecipeTier::Twin,
                    "Legendary" => BaseballerRecipeTier::Legendary,
                    _ => return Err(error("unknown Baseballer recipe tier")),
                },
                output: equipment_id(row.output_equipment_id)?,
                inputs: inputs.into_boxed_slice(),
            })
        })
        .collect::<Result<Vec<_>, EventDataError>>()?;
    let strategies = config
        .gb_runtime_strategies()
        .ordered_rows()
        .map(|row| {
            Ok(BaseballerAdventureStrategy {
                id: BaseballerAdventureStrategyId::new(unsigned(row.id, "strategy id")?)
                    .ok_or_else(|| error("strategy id must be non-zero"))?,
                stable_key: row.stable_key.clone().into_boxed_str(),
                profile: profile_id(row.profile_id)?,
                kind: match row.kind {
                    GbRuntimeStrategyKind::Growth => BaseballerAdventureStrategyKind::Growth,
                    GbRuntimeStrategyKind::Power => BaseballerAdventureStrategyKind::Power,
                    GbRuntimeStrategyKind::General => BaseballerAdventureStrategyKind::General,
                    GbRuntimeStrategyKind::DemonKing => BaseballerAdventureStrategyKind::DemonKing,
                },
                maximum_level: small(row.maximum_level, "strategy maximum level")?,
                unlock_quest_id: row
                    .unlock_quest_id
                    .map(|value| unsigned(value, "strategy unlock quest id"))
                    .transpose()?,
                selectable_periods: row
                    .selectable_periods
                    .as_deref()
                    .unwrap_or_default()
                    .iter()
                    .copied()
                    .map(|value| small(value, "strategy selectable period"))
                    .collect::<Result<Vec<_>, _>>()?
                    .into_boxed_slice(),
                influence_scope: row.influence_scope.clone().into_boxed_str(),
                maze_buff_id: unsigned(row.maze_buff_id, "strategy MazeBuff id")?,
                maze_buff_parameters: boxed_strings(
                    row.maze_buff_parameters.as_deref().unwrap_or_default(),
                ),
                ability_binding: row.ability_binding.clone().into_boxed_str(),
                runtime_binding_exact: row.runtime_binding_exact,
            })
        })
        .collect::<Result<Vec<_>, EventDataError>>()?;
    let team_bonuses = config
        .gb_runtime_team_bonuses()
        .ordered_rows()
        .map(|row| {
            Ok(BaseballerTeamBonus {
                stage: BaseballerStageId::new(unsigned(row.stage_id, "team-bonus stage id")?)
                    .ok_or_else(|| error("team-bonus stage id must be non-zero"))?,
                profile: profile_id(row.profile_id)?,
                maze_buff_id: unsigned(row.maze_buff_id, "team-bonus MazeBuff id")?,
                level: small(row.level, "team-bonus level")?,
                parameters: boxed_strings(&row.parameters),
                ability_binding: row.ability_binding.clone().into_boxed_str(),
                runtime_binding_exact: row.runtime_binding_exact,
            })
        })
        .collect::<Result<Vec<_>, EventDataError>>()?;
    let catalog = BaseballerCatalog::new_with_runtime_content(
        profiles,
        stages,
        stage_periods,
        equipment,
        recipes,
        BaseballerRuntimeCatalogContent {
            shop_upgrades,
            strategies,
            team_bonuses,
        },
    )
    .map_err(|value| error(&format!("invalid Baseballer catalog: {value:?}")))?;
    let score_rules = config
        .gb_runtime_score_rules()
        .ordered_rows()
        .map(|row| {
            let profile = profile_id(row.profile_id)?;
            let thresholds = catalog
                .stages()
                .iter()
                .find(|stage| stage.profile == profile)
                .ok_or_else(|| error("score profile has no stage"))?
                .rating_thresholds
                .to_vec();
            let rule = BaseballerScoreRule::new(
                row.monster_base_score,
                row.elite_scores.clone(),
                row.monster_weights.clone(),
                row.score_cap,
                row.final_stage_extra_bonus,
                thresholds,
            )
            .ok_or_else(|| error("invalid Baseballer score rule"))?;
            Ok((profile, rule))
        })
        .collect::<Result<Vec<_>, EventDataError>>()?;
    let policies = config
        .gb_runtime_policies()
        .ordered_rows()
        .map(|row| BaseballerRuntimePolicy {
            id: row.stable_key.clone().into_boxed_str(),
            unavailable_fact: row.unavailable_fact.clone().into_boxed_str(),
            known_facts: row.known_facts.clone().into_boxed_str(),
            selected_behavior: row.selected_behavior.clone().into_boxed_str(),
            rejected_alternatives: boxed_strings(&row.rejected_alternatives),
            rationale: row.rationale.clone().into_boxed_str(),
            affected_tests: boxed_strings(&row.affected_tests),
            confidence: row.confidence.clone().into_boxed_str(),
            replacement_condition: row.replacement_condition.clone().into_boxed_str(),
        })
        .collect::<Vec<_>>();
    if catalog.profiles().len() != 2
        || catalog.stages().len() != 13
        || catalog.stage_periods().len() != 102
        || catalog.equipment().len() != 87
        || catalog.recipes().len() != 27
        || catalog.shop_upgrades().len() != 114
        || catalog.strategies().len() != 56
        || catalog.team_bonuses().len() != 7
        || score_rules.len() != 2
        || policies.len() != 6
    {
        return Err(error("Baseballer runtime denominator drift"));
    }
    Ok(BaseballerRuntimeData {
        catalog,
        score_rules: score_rules.into_boxed_slice(),
        policies: policies.into_boxed_slice(),
    })
}

pub fn load_fate_star_rail_night(bytes: &[u8]) -> Result<FateRuntimeData, EventDataError> {
    let config = parse(bytes)?;
    if config.fate_runtime_profiles().len() != 1 {
        return Err(error("Fate runtime profile denominator drift"));
    }
    let boards = config
        .fate_runtime_boards()
        .ordered_rows()
        .map(|row| {
            let mut board_rows = config
                .fate_runtime_board_nodes()
                .ordered_rows()
                .filter(|node| node.board_id == row.id)
                .collect::<Vec<_>>();
            board_rows.sort_by_key(|node| node.sequence);
            if board_rows
                .windows(2)
                .any(|pair| pair[0].sequence == pair[1].sequence)
            {
                return Err(error("Fate board contains duplicate node sequence"));
            }
            let nodes = board_rows
                .iter()
                .map(|node| {
                    Ok(FateBoardNode {
                        id: unsigned(node.id, "Fate board node id")?,
                        kind: match node.kind {
                            GeneratedBoardNodeKind::Choice => FateBoardNodeKind::Choice,
                            GeneratedBoardNodeKind::Battle => FateBoardNodeKind::Battle,
                            GeneratedBoardNodeKind::Completed => FateBoardNodeKind::Completed,
                        },
                        maximum_visits: 1,
                    })
                })
                .collect::<Result<Vec<_>, EventDataError>>()?;
            let edges = board_rows
                .windows(2)
                .enumerate()
                .map(|(index, pair)| {
                    let offset = u32::try_from(index + 1)
                        .map_err(|_| error("Fate board edge offset exceeds u32"))?;
                    Ok(FateBoardEdge {
                        id: unsigned(row.id, "Fate board id")?
                            .checked_mul(10)
                            .and_then(|value| value.checked_add(offset))
                            .ok_or_else(|| error("Fate board edge id overflow"))?,
                        from: unsigned(pair[0].id, "Fate board edge source")?,
                        to: unsigned(pair[1].id, "Fate board edge target")?,
                        priority: i32::try_from(index)
                            .map_err(|_| error("Fate board edge priority exceeds i32"))?,
                    })
                })
                .collect::<Result<Vec<_>, EventDataError>>()?;
            let entry = nodes
                .first()
                .map(|node| node.id)
                .ok_or_else(|| error("Fate board has no nodes"))?;
            FateBoard::new(unsigned(row.id, "Fate board id")?, entry, nodes, edges)
                .map_err(|value| error(&format!("invalid Fate board: {value:?}")))
        })
        .collect::<Result<Vec<_>, EventDataError>>()?;
    let owners = config
        .fate_runtime_owners()
        .ordered_rows()
        .map(|row| card_owner(row.owner))
        .collect::<Vec<_>>();
    let cards = config
        .fate_runtime_cards()
        .ordered_rows()
        .map(|row| {
            Ok(FateCard {
                id: fate_card_id(row.id)?,
                stable_key: row.stable_key.clone().into_boxed_str(),
                owner: card_owner(row.owner),
                magical_energy_cost: u16::try_from(row.magical_energy_cost)
                    .map_err(|_| error("Fate card cost exceeds u16"))?,
                rarity: match row.rarity {
                    GeneratedCardRarity::R => FateCardRarity::R,
                    GeneratedCardRarity::SR => FateCardRarity::Sr,
                    GeneratedCardRarity::Ssr => FateCardRarity::Ssr,
                },
                ability_program: row.ability_program.clone().map(String::into_boxed_str),
                runtime_binding_exact: row.runtime_binding_exact,
            })
        })
        .collect::<Result<Vec<_>, EventDataError>>()?;
    let decks = config
        .fate_runtime_decks()
        .ordered_rows()
        .map(|row| {
            Ok(FateDeck {
                id: FateDeckId::new(unsigned(row.id, "Fate deck id")?)
                    .ok_or_else(|| error("Fate deck id must be non-zero"))?,
                stable_key: row.stable_key.clone().into_boxed_str(),
                owner: card_owner(row.owner),
                presentation_locator: unsigned(
                    row.presentation_locator,
                    "Fate deck presentation locator",
                )?,
                action_locator: unsigned(row.action_locator, "Fate deck action locator")?,
            })
        })
        .collect::<Result<Vec<_>, EventDataError>>()?;
    let recommendations = config
        .fate_runtime_deck_recommendations()
        .ordered_rows()
        .map(|row| {
            Ok(FateDeckRecommendation {
                id: FateDeckRecommendationId::new(unsigned(row.id, "Fate deck recommendation id")?)
                    .ok_or_else(|| error("Fate deck recommendation id must be non-zero"))?,
                owner: card_owner(row.owner),
                kind: match row.kind {
                    GeneratedDeckKind::Base => FateDeckRecommendationKind::Base,
                    GeneratedDeckKind::Final => FateDeckRecommendationKind::Final,
                },
                owner_cards: row
                    .owner_card_ids
                    .iter()
                    .copied()
                    .map(fate_card_id)
                    .collect::<Result<Vec<_>, _>>()?
                    .into_boxed_slice(),
                neutral_cards: row
                    .neutral_card_ids
                    .iter()
                    .copied()
                    .map(fate_card_id)
                    .collect::<Result<Vec<_>, _>>()?
                    .into_boxed_slice(),
            })
        })
        .collect::<Result<Vec<_>, EventDataError>>()?;
    let story_fights = config
        .fate_runtime_story_fights()
        .ordered_rows()
        .map(|row| {
            Ok(FateStoryFight {
                id: FateStoryFightId::new(unsigned(row.id, "Fate story fight id")?)
                    .ok_or_else(|| error("Fate story fight id must be non-zero"))?,
                battle_event_id: unsigned(row.battle_event_id, "Fate battle event id")?,
                map_entrance_id: unsigned(row.map_entrance_id, "Fate map entrance id")?,
            })
        })
        .collect::<Result<Vec<_>, EventDataError>>()?;
    let challenge_fights = config
        .fate_runtime_challenge_fights()
        .ordered_rows()
        .map(|row| {
            Ok(FateChallengeFight {
                id: FateChallengeFightId::new(unsigned(row.id, "Fate challenge fight id")?)
                    .ok_or_else(|| error("Fate challenge fight id must be non-zero"))?,
                battle_event_id: unsigned(row.battle_event_id, "Fate battle event id")?,
                map_entrance_id: unsigned(row.map_entrance_id, "Fate map entrance id")?,
                enemy_id: unsigned(row.enemy_id, "Fate enemy id")?,
                buff_ids: unsigned_values(&row.buff_ids, "Fate challenge buff id")?,
            })
        })
        .collect::<Result<Vec<_>, EventDataError>>()?;
    let map_fights = config
        .fate_runtime_map_fights()
        .ordered_rows()
        .map(|row| {
            Ok(FateMapFight {
                id: FateMapFightId::new(unsigned(row.id, "Fate map fight id")?)
                    .ok_or_else(|| error("Fate map fight id must be non-zero"))?,
                battle_event_ids: unsigned_values(
                    &row.battle_event_ids,
                    "Fate map battle event id",
                )?,
                map_entrance_id: unsigned(row.map_entrance_id, "Fate map entrance id")?,
                reward_card: row.reward_card_id.map(fate_card_id).transpose()?,
                terminal: row.terminal,
                enemy_id: unsigned(row.enemy_id, "Fate enemy id")?,
                relation: row.relation.clone().map(String::into_boxed_str),
            })
        })
        .collect::<Result<Vec<_>, EventDataError>>()?;
    let catalog = FateCatalog::new(FateCatalogParts {
        boards,
        owners,
        cards,
        decks,
        recommendations,
        story_fights,
        challenge_fights,
        map_fights,
    })
    .map_err(|value| error(&format!("invalid Fate catalog: {value:?}")))?;
    let policies = config
        .fate_runtime_policies()
        .ordered_rows()
        .map(|row| FateRuntimePolicy {
            id: row.stable_key.clone().into_boxed_str(),
            unavailable_fact: row.unavailable_fact.clone().into_boxed_str(),
            known_facts: row.known_facts.clone().into_boxed_str(),
            selected_behavior: row.selected_behavior.clone().into_boxed_str(),
            rejected_alternatives: boxed_strings(&row.rejected_alternatives),
            rationale: row.rationale.clone().into_boxed_str(),
            affected_tests: boxed_strings(&row.affected_tests),
            confidence: row.confidence.clone().into_boxed_str(),
            replacement_condition: row.replacement_condition.clone().into_boxed_str(),
        })
        .collect::<Vec<_>>();
    if catalog.boards().len() != 6
        || catalog.owners().len() != 6
        || catalog.cards().len() != 107
        || catalog.decks().len() != 4
        || catalog.recommendations().len() != 7
        || catalog.story_fights().len() != 6
        || catalog.challenge_fights().len() != 4
        || catalog.map_fights().len() != 15
        || policies.len() != 16
    {
        return Err(error("Fate runtime denominator drift"));
    }
    Ok(FateRuntimeData {
        catalog,
        policies: policies.into_boxed_slice(),
    })
}

fn parse(bytes: &[u8]) -> Result<SoraConfig, EventDataError> {
    let bundle = SoraBundle::parse(bytes).map_err(|value| error(&value.to_string()))?;
    SoraConfig::from_source(&bundle).map_err(|value| error(&value.to_string()))
}

fn profile_id(value: i32) -> Result<BaseballerProfileId, EventDataError> {
    BaseballerProfileId::new(unsigned(value, "Baseballer profile id")?)
        .ok_or_else(|| error("Baseballer profile id must be non-zero"))
}

fn equipment_id(value: i32) -> Result<BaseballerEquipmentId, EventDataError> {
    BaseballerEquipmentId::new(unsigned(value, "Baseballer equipment id")?)
        .ok_or_else(|| error("Baseballer equipment id must be non-zero"))
}

fn fate_card_id(value: i32) -> Result<FateCardId, EventDataError> {
    FateCardId::new(unsigned(value, "Fate card id")?)
        .ok_or_else(|| error("Fate card id must be non-zero"))
}

fn card_owner(value: GeneratedCardOwner) -> FateCardOwner {
    match value {
        GeneratedCardOwner::Trailblazer => FateCardOwner::Trailblazer,
        GeneratedCardOwner::Gilgamesh => FateCardOwner::Gilgamesh,
        GeneratedCardOwner::Archer => FateCardOwner::Archer,
        GeneratedCardOwner::Saber => FateCardOwner::Saber,
        GeneratedCardOwner::Rin => FateCardOwner::Rin,
        GeneratedCardOwner::Neutral => FateCardOwner::Neutral,
    }
}

fn unsigned_values(values: &[i32], field: &str) -> Result<Box<[u32]>, EventDataError> {
    values
        .iter()
        .copied()
        .map(|value| unsigned(value, field))
        .collect::<Result<Vec<_>, _>>()
        .map(Vec::into_boxed_slice)
}

fn unsigned(value: i32, field: &str) -> Result<u32, EventDataError> {
    u32::try_from(value).map_err(|_| error(&format!("{field} must be non-negative")))
}

fn small(value: i32, field: &str) -> Result<u8, EventDataError> {
    u8::try_from(value).map_err(|_| error(&format!("{field} exceeds u8")))
}

fn boxed_strings(values: &[String]) -> Box<[Box<str>]> {
    values
        .iter()
        .map(|value| value.clone().into_boxed_str())
        .collect()
}

fn error(message: &str) -> EventDataError {
    EventDataError {
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::{fate_star_rail_night, galactic_baseballer};

    #[test]
    fn production_baseballer_profiles_and_synthesis_lower() {
        use std::sync::Arc;

        use starclock_activity::{
            ActivityConfigDigest, ActivityDefinitionDigest, ActivityDefinitionId,
            ActivityDefinitionIdentity, ActivityInstanceId, ActivityMasterSeed, BuildDigest,
            LoadoutLockScope, OpaqueParticipantBuild, ParticipantId, ParticipantLock,
            ParticipantLockEntry, ParticipantPolicy, ParticipantSourceKind,
            ParticipantUniquenessScope, SectionId,
        };
        use starclock_combat::{CombatantSpecDigest, UnitDefinitionId};
        use starclock_mode_baseballer::{
            BaseballerRun, BaseballerRunDefinition, BaseballerStageFlow,
        };

        let data = galactic_baseballer().expect("Baseballer bundle lowers");
        assert_eq!(data.catalog.profiles().len(), 2);
        assert_eq!(data.catalog.stages().len(), 13);
        assert_eq!(data.catalog.stage_periods().len(), 102);
        assert_eq!(data.catalog.equipment().len(), 87);
        assert_eq!(data.catalog.recipes().len(), 27);
        assert_eq!(data.catalog.shop_upgrades().len(), 114);
        assert_eq!(data.catalog.strategies().len(), 56);
        assert_eq!(data.catalog.team_bonuses().len(), 7);
        assert!(
            data.catalog
                .strategies()
                .iter()
                .all(|strategy| !strategy.runtime_binding_exact)
        );
        assert!(
            data.catalog
                .team_bonuses()
                .iter()
                .all(|bonus| !bonus.runtime_binding_exact)
        );
        assert_eq!(
            data.catalog
                .shop_upgrades()
                .iter()
                .filter(|upgrade| upgrade.runtime_binding_exact)
                .count(),
            12
        );
        assert_eq!(data.score_rules.len(), 2);
        assert_eq!(data.policies.len(), 6);
        assert!(data.policies.iter().all(|policy| {
            !policy.unavailable_fact.is_empty()
                && !policy.known_facts.is_empty()
                && !policy.rejected_alternatives.is_empty()
                && !policy.rationale.is_empty()
                && !policy.affected_tests.is_empty()
                && !policy.confidence.is_empty()
                && !policy.replacement_condition.is_empty()
        }));
        for (index, stage) in data.catalog.stages().iter().enumerate() {
            let section = SectionId::new(u32::try_from(index + 1).unwrap()).unwrap();
            let periods = data
                .catalog
                .periods_for_stage(stage.id)
                .into_iter()
                .cloned()
                .collect::<Vec<_>>();
            assert!(BaseballerStageFlow::compile(section, &periods).is_ok());
            let score_rule = data
                .score_rules
                .iter()
                .find(|(profile, _)| *profile == stage.profile)
                .unwrap()
                .1
                .clone();
            let identity = ActivityDefinitionIdentity::new(
                ActivityDefinitionId::new(u32::try_from(index + 1).unwrap()).unwrap(),
                ActivityDefinitionDigest::new([u8::try_from(index + 1).unwrap(); 32]).unwrap(),
                ActivityConfigDigest::new([99; 32]).unwrap(),
            );
            let policy = ParticipantPolicy::new(
                1,
                1,
                4,
                ParticipantUniquenessScope::Team,
                LoadoutLockScope::Activity,
            )
            .unwrap();
            let participant = ParticipantLockEntry::new(
                ParticipantId::new(1).unwrap(),
                0,
                0,
                UnitDefinitionId::new(1).unwrap(),
                OpaqueParticipantBuild::new(
                    CombatantSpecDigest::new([3; 32]).unwrap(),
                    BuildDigest::new([4; 32]).unwrap(),
                    ParticipantSourceKind::FixedResolved,
                )
                .unwrap(),
            )
            .unwrap();
            let definition = Arc::new(
                BaseballerRunDefinition::new(
                    identity,
                    Arc::new(data.catalog.clone()),
                    stage.id,
                    score_rule,
                    ParticipantLock::seal(policy, vec![participant]).unwrap(),
                )
                .unwrap(),
            );
            assert!(
                BaseballerRun::start(
                    definition,
                    ActivityInstanceId::new(1).unwrap(),
                    ActivityMasterSeed::from_u64(7),
                )
                .is_ok()
            );
        }
    }

    #[test]
    fn production_fate_card_surface_lowers_with_policies() {
        use starclock_activity::SectionId;

        let data = fate_star_rail_night().expect("Fate bundle lowers");
        assert_eq!(data.catalog.boards().len(), 6);
        assert_eq!(data.catalog.owners().len(), 6);
        assert_eq!(data.catalog.cards().len(), 107);
        assert_eq!(data.catalog.decks().len(), 4);
        assert_eq!(data.catalog.recommendations().len(), 7);
        assert_eq!(data.catalog.story_fights().len(), 6);
        assert_eq!(data.catalog.challenge_fights().len(), 4);
        assert_eq!(data.catalog.map_fights().len(), 15);
        assert!(
            data.catalog
                .cards()
                .iter()
                .all(|card| !card.runtime_binding_exact)
        );
        assert_eq!(data.policies.len(), 16);
        assert!(data.policies.iter().all(|policy| {
            !policy.unavailable_fact.is_empty()
                && !policy.known_facts.is_empty()
                && !policy.rejected_alternatives.is_empty()
                && !policy.rationale.is_empty()
                && !policy.affected_tests.is_empty()
                && !policy.confidence.is_empty()
                && !policy.replacement_condition.is_empty()
        }));
        for (index, board) in data.catalog.boards().iter().enumerate() {
            let section = SectionId::new(u32::try_from(index + 1).unwrap()).unwrap();
            assert!(board.compile(section).is_ok());
        }
    }
}
