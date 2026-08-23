use serde::Deserialize;
use starclock_mode_currency_wars::{
    CurrencyWarsRankBoundary, CurrencyWarsRankProgression, CurrencyWarsRankProgressionKey,
};

use crate::{
    currency_wars::{CurrencyWarsDataError, debug_error, error},
    currency_wars_flow::{encounter_id, parse_boxed_strings, parse_json, required},
    currency_wars_generated::SoraConfig,
};

pub(super) fn lower_currency_wars_rank_progression(
    config: &SoraConfig,
) -> Result<Vec<CurrencyWarsRankProgression>, CurrencyWarsDataError> {
    config
        .currency_wars_rank_gambit_progression()
        .ordered_rows()
        .map(|row| {
            let mode = required(&row.gambit_mode, "rank Gambit mode")?;
            let (key, boundary) = match mode {
                "StandardGambitWithOverclockCap" => {
                    let rank: DivisionRank = parse_json(required(&row.rank, "division rank")?)?;
                    let boundary: GambitBoundary =
                        parse_json(required(&row.entry_boundary, "Gambit boundary")?)?;
                    (
                        CurrencyWarsRankProgressionKey::Division {
                            season: rank.season_id.parse().map_err(debug_error)?,
                            level: rank.division_level.parse().map_err(debug_error)?,
                        },
                        CurrencyWarsRankBoundary::GambitDifficulty {
                            maximum_standard: boundary
                                .maximum_standard_difficulty
                                .parse()
                                .map_err(debug_error)?,
                            maximum_overclock: boundary
                                .maximum_overclock_difficulty
                                .parse()
                                .map_err(debug_error)?,
                            reward_quest_fields_excluded: boundary.reward_quest_fields_excluded,
                        },
                    )
                }
                "SharedGridFightDifficulty" => {
                    let base: SharedBattleBase =
                        parse_json(required(&row.entry_boundary, "shared battle base")?)?;
                    let rank: SharedRank = parse_json(required(&row.rank, "shared rank")?)?;
                    let key = match rank {
                        SharedRank::Level {
                            chapter_id,
                            section_id,
                        } => CurrencyWarsRankProgressionKey::LevelBase {
                            plane: chapter_id.parse().map_err(debug_error)?,
                            section: section_id.parse().map_err(debug_error)?,
                        },
                        SharedRank::Stage { stage_id } => {
                            CurrencyWarsRankProgressionKey::StageBase(encounter_id(
                                stage_id.parse().map_err(debug_error)?,
                            )?)
                        }
                    };
                    (
                        key,
                        CurrencyWarsRankBoundary::SharedBattleBase {
                            attack: base.level_base_attack.parse().map_err(debug_error)?,
                            hp: base.level_base_hp.parse().map_err(debug_error)?,
                        },
                    )
                }
                "BinaryDifficultyAddition" => {
                    let rank: BinaryRank =
                        parse_json(required(&row.rank, "binary difficulty rank")?)?;
                    let boundary: BinaryDifficultyBoundary =
                        parse_json(required(&row.entry_boundary, "binary difficulty boundary")?)?;
                    (
                        CurrencyWarsRankProgressionKey::BinaryDifficulty {
                            rule: rank.rule_id.parse().map_err(debug_error)?,
                            quality: rank.quality.parse().map_err(debug_error)?,
                        },
                        CurrencyWarsRankBoundary::BinaryDifficultyAddition {
                            enemy_difficulty_level_add: boundary
                                .enemy_difficulty_level_add
                                .parse()
                                .map_err(debug_error)?,
                        },
                    )
                }
                "BinaryNodePerformLevel" => {
                    let rank: BinaryRank = parse_json(required(&row.rank, "binary node rank")?)?;
                    let boundary: BinaryNodeBoundary =
                        parse_json(required(&row.entry_boundary, "binary node boundary")?)?;
                    (
                        CurrencyWarsRankProgressionKey::BinaryNode(
                            rank.rule_id.parse().map_err(debug_error)?,
                        ),
                        CurrencyWarsRankBoundary::BinaryNodePerformLevel {
                            quality: rank.quality.parse().map_err(debug_error)?,
                            perform_level: boundary.perform_level.parse().map_err(debug_error)?,
                        },
                    )
                }
                _ => return Err(error("Currency Wars rank Gambit mode is unknown")),
            };
            Ok(CurrencyWarsRankProgression {
                stable_key: row.stable_key.clone().into(),
                key,
                boundary,
                enemy_affix_ids: parse_boxed_strings(row.enemy_affix_ids.as_ref())?,
            })
        })
        .collect()
}

#[derive(Deserialize)]
struct DivisionRank {
    division_level: String,
    season_id: String,
}

#[derive(Deserialize)]
struct GambitBoundary {
    maximum_overclock_difficulty: String,
    maximum_standard_difficulty: String,
    reward_quest_fields_excluded: bool,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum SharedRank {
    Level {
        chapter_id: String,
        section_id: String,
    },
    Stage {
        stage_id: String,
    },
}

#[derive(Deserialize)]
struct SharedBattleBase {
    level_base_attack: String,
    level_base_hp: String,
}

#[derive(Deserialize)]
struct BinaryRank {
    rule_id: String,
    quality: String,
}

#[derive(Deserialize)]
struct BinaryDifficultyBoundary {
    enemy_difficulty_level_add: String,
}

#[derive(Deserialize)]
struct BinaryNodeBoundary {
    perform_level: String,
}
