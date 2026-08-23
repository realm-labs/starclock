use starclock_combat::EncounterId;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CurrencyWarsRankProgressionKey {
    Division { season: u16, level: u8 },
    LevelBase { plane: u8, section: u8 },
    StageBase(EncounterId),
    BinaryDifficulty { rule: u8, quality: u8 },
    BinaryNode(u32),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CurrencyWarsRankBoundary {
    GambitDifficulty {
        maximum_standard: u8,
        maximum_overclock: u8,
        reward_quest_fields_excluded: bool,
    },
    SharedBattleBase {
        attack: u32,
        hp: u32,
    },
    BinaryDifficultyAddition {
        enemy_difficulty_level_add: u8,
    },
    BinaryNodePerformLevel {
        quality: u8,
        perform_level: u8,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsRankProgression {
    pub stable_key: Box<str>,
    pub key: CurrencyWarsRankProgressionKey,
    pub boundary: CurrencyWarsRankBoundary,
    pub enemy_affix_ids: Box<[Box<str>]>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CurrencyWarsSharedBattleBase {
    pub attack: u32,
    pub hp: u32,
}
