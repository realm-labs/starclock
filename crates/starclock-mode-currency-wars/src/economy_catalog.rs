use std::collections::BTreeSet;

use crate::{
    CurrencyWarsAuthoredProperty, CurrencyWarsDecimal, CurrencyWarsGambit, CurrencyWarsOfferLevel,
    CurrencyWarsPriceRule, CurrencyWarsRoleId, CurrencyWarsStarRule, CurrencyWarsTeamLevel,
};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CurrencyWarsCurrencyGain {
    AuthoredBattleEventInterestAndServiceOutcomes,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CurrencyWarsCurrencySpend {
    RecruitmentRefreshAndPricedServices,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CurrencyWarsCurrencyReset {
    DiscardAtRunTeardown,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsCurrency {
    pub stable_key: Box<str>,
    pub gains: Box<[CurrencyWarsCurrencyGain]>,
    pub spends: Box<[CurrencyWarsCurrencySpend]>,
    pub reset: CurrencyWarsCurrencyReset,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CurrencyWarsExperienceRules {
    pub resource_id: u32,
    pub standard_wave_gain: u32,
    pub standard_boss_wave_gain: u32,
    pub overclock_wave_gain: [u32; 3],
    pub overclock_boss_wave_gain: [u32; 3],
    pub direct_level_up_experience: u32,
    pub direct_level_up_gold: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CurrencyWarsInterestRules {
    pub deposit_per_interest: u32,
    pub standard_maximum: u32,
    pub overclock_maximum: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CurrencyWarsRefreshRules {
    pub cards_per_refresh: u8,
    pub gold_cost: u32,
    pub copies_per_role_by_rarity: [u32; 5],
    pub role_initial_weight: u32,
    pub maximum_stolen_same_card_by_rarity: [u32; 5],
    pub stolen_pool_refund_initial_purchase: u32,
    pub stolen_pool_refund_sell: u32,
    pub stolen_pool_refund_hold: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CurrencyWarsTeamSizeRules {
    pub front_minimum: u8,
    pub front_maximum: u8,
    pub back_initial: u8,
    pub back_maximum: u8,
    pub bench_authored: u8,
    pub bench_overflow: u16,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsEconomyRules {
    pub stable_key: Box<str>,
    pub currency_ids: Box<[Box<str>]>,
    pub experience: CurrencyWarsExperienceRules,
    pub interest: CurrencyWarsInterestRules,
    pub refresh: CurrencyWarsRefreshRules,
    pub team_size: CurrencyWarsTeamSizeRules,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CurrencyWarsOfferCostRule {
    BuyGoldAtStarOne,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CurrencyWarsOfferFallback {
    RejectIfNoPositiveRarityWeight,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CurrencyWarsTransactionChange {
    ValidateRosterAndGold,
    ApplyAuthoredGoldPrice,
    ApplyRosterMutation,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CurrencyWarsTeamLevelTransition {
    SpendExperienceToNext,
    MaximumAuthoredLevel,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CurrencyWarsPositionEligibility {
    DirectFront,
    DirectBack,
    MissingSourceTypeWithBothDisplays,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CurrencyWarsPositionDefinitionKind {
    Front,
    Back,
    FrontBackCandidate,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsPositionDefinition {
    pub stable_key: Box<str>,
    pub kind: CurrencyWarsPositionDefinitionKind,
    pub field_index: Box<str>,
    pub eligibility: CurrencyWarsPositionEligibility,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsStarState {
    pub stable_key: Box<str>,
    pub owner: CurrencyWarsStarStateOwner,
    pub star: u8,
    pub copy_count: u32,
    pub scaling_refs: Box<[Box<str>]>,
    pub rank_attachments: Box<[CurrencyWarsRankAttachment]>,
    pub battle_event_id: Option<u32>,
    pub skill_override_source_ids: Box<[u32]>,
    pub front_execution_skill_ids: Box<[u32]>,
    pub front_display_skill_ids: Box<[u32]>,
    pub back_execution_skill_ids: Box<[u32]>,
    pub back_display_skill_ids: Box<[u32]>,
    pub back_ability_name: Option<Box<str>>,
    pub config_path: Option<Box<str>>,
    pub ai_path: Option<Box<str>>,
    pub property_modifiers: Box<[CurrencyWarsAuthoredProperty]>,
    pub front_power_base: Option<CurrencyWarsDecimal>,
    pub back_power_base: Option<CurrencyWarsDecimal>,
    pub luck_chance: Option<CurrencyWarsDecimal>,
    pub luck_damage: Option<CurrencyWarsDecimal>,
    pub extra_heal_base: Option<CurrencyWarsDecimal>,
    pub extra_shield_base: Option<CurrencyWarsDecimal>,
    pub hp_base: Option<Box<str>>,
    pub hp_inherit: Option<Box<str>>,
    pub hp_skill_id: Option<u32>,
    pub speed_base: Option<Box<str>>,
    pub speed_inherit: Option<Box<str>>,
    pub speed_skill_id: Option<u32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsRankAttachment {
    pub rank: u8,
    pub properties: Box<[CurrencyWarsAuthoredProperty]>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CurrencyWarsInfluenceSubject {
    Star,
    Rarity,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsInfluenceProperty {
    pub stable_key: Box<str>,
    pub subject: CurrencyWarsInfluenceSubject,
    pub level: u8,
    pub properties: Box<[CurrencyWarsAuthoredProperty]>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CurrencyWarsContributionParameterKind {
    CombinationBonus,
    RuntimeConstant,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsContributionParameter {
    pub stable_key: Box<str>,
    pub kind: CurrencyWarsContributionParameterKind,
    pub source_id: Box<str>,
    pub combination_ids: Box<[u32]>,
    pub bonus_numbers: Box<[u32]>,
    pub value_json: Option<Box<str>>,
    pub consumer_policy: Box<str>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CurrencyWarsStarStateOwner {
    Role(CurrencyWarsRoleId),
    Servant { avatar_id: u32, servant_id: u32 },
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CurrencyWarsStarOverflowRule {
    RepeatEqualStarTriples,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CurrencyWarsStarLifecycleOperation {
    AcquireCopy,
    SellRole,
    AcquireAtMaximumStar,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsStarLifecycleRule {
    pub stable_key: Box<str>,
    pub operation: CurrencyWarsStarLifecycleOperation,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CurrencyWarsSquadHpMaximum {
    InitialWithContentDefinedIncreaseOrRecovery,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CurrencyWarsSquadHpLossRule {
    ConfiguredNodeOrDifficultyOnNonVictory,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CurrencyWarsSquadHpRecoveryRule {
    PreserveOnVictory,
    AuthoredContentRestoreOrMaximumIncrease,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsSquadHpRules {
    pub stable_key: Box<str>,
    pub initial: u32,
    pub minimum: u32,
    pub maximum: CurrencyWarsSquadHpMaximum,
    pub loss_rules: Box<[CurrencyWarsSquadHpLossRule]>,
    pub recovery_rules: Box<[CurrencyWarsSquadHpRecoveryRule]>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CurrencyWarsActionValueInitial {
    ConfiguredByNodeOrDifficulty,
    Infinite,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CurrencyWarsActionValueDecrement {
    ElapsedAuthoritativeActionValue,
    ConfiguredCharacterLethalRescue,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CurrencyWarsTimeoutBoundary {
    NonVictoryAndConfiguredSquadHpLoss,
    Unreachable,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CurrencyWarsActionValueLimitKind {
    FiniteNodeConfigured,
    Unlimited,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsActionValueLimit {
    pub stable_key: Box<str>,
    pub kind: CurrencyWarsActionValueLimitKind,
    pub initial: CurrencyWarsActionValueInitial,
    pub decrements: Box<[CurrencyWarsActionValueDecrement]>,
    pub timeout: CurrencyWarsTimeoutBoundary,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CurrencyWarsBattleOutcomeProjection {
    Victory,
    NonVictory,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CurrencyWarsSquadHpProjection {
    PreserveBeforeContentContributions,
    SubtractConfiguredLossClampToZero,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CurrencyWarsActionValueProjection {
    CaptureForFinalizationThenDiscard,
    CaptureExhaustedThenDiscard,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CurrencyWarsRunDisposition {
    ContinueUnlessFinalBoss,
    FailAtZeroOtherwiseContinue,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsBattleResultProjection {
    pub stable_key: Box<str>,
    pub outcome: CurrencyWarsBattleOutcomeProjection,
    pub squad_hp: CurrencyWarsSquadHpProjection,
    pub action_value: CurrencyWarsActionValueProjection,
    pub run: CurrencyWarsRunDisposition,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsEconomyCatalog {
    currencies: Box<[CurrencyWarsCurrency]>,
    rules: CurrencyWarsEconomyRules,
    offers: Box<[CurrencyWarsOfferLevel]>,
    prices: Box<[CurrencyWarsPriceRule]>,
    team_levels: Box<[CurrencyWarsTeamLevel]>,
    positions: Box<[CurrencyWarsPositionDefinition]>,
    star_states: Box<[CurrencyWarsStarState]>,
    influence_properties: Box<[CurrencyWarsInfluenceProperty]>,
    contribution_parameters: Box<[CurrencyWarsContributionParameter]>,
    star_rules: Box<[CurrencyWarsStarRule]>,
    star_lifecycle: Box<[CurrencyWarsStarLifecycleRule]>,
    squad_hp: CurrencyWarsSquadHpRules,
    action_value_limits: Box<[CurrencyWarsActionValueLimit]>,
    battle_result_projections: Box<[CurrencyWarsBattleResultProjection]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsEconomyCatalogParts {
    pub currencies: Vec<CurrencyWarsCurrency>,
    pub rules: CurrencyWarsEconomyRules,
    pub offers: Vec<CurrencyWarsOfferLevel>,
    pub prices: Vec<CurrencyWarsPriceRule>,
    pub team_levels: Vec<CurrencyWarsTeamLevel>,
    pub positions: Vec<CurrencyWarsPositionDefinition>,
    pub star_states: Vec<CurrencyWarsStarState>,
    pub influence_properties: Vec<CurrencyWarsInfluenceProperty>,
    pub contribution_parameters: Vec<CurrencyWarsContributionParameter>,
    pub star_rules: Vec<CurrencyWarsStarRule>,
    pub star_lifecycle: Vec<CurrencyWarsStarLifecycleRule>,
    pub squad_hp: CurrencyWarsSquadHpRules,
    pub action_value_limits: Vec<CurrencyWarsActionValueLimit>,
    pub battle_result_projections: Vec<CurrencyWarsBattleResultProjection>,
}

impl CurrencyWarsEconomyCatalog {
    pub fn new(
        mut parts: CurrencyWarsEconomyCatalogParts,
    ) -> Result<Self, CurrencyWarsEconomyCatalogError> {
        parts
            .currencies
            .sort_by(|a, b| a.stable_key.cmp(&b.stable_key));
        parts.offers.sort_by_key(|value| value.level);
        parts.prices.sort_by_key(|value| value.rarity);
        parts.team_levels.sort_by_key(|value| value.level);
        parts.positions.sort_by_key(|value| value.kind);
        parts
            .star_states
            .sort_by_key(|value| (value.owner, value.star));
        parts
            .influence_properties
            .sort_by_key(|value| (value.subject, value.level));
        parts
            .contribution_parameters
            .sort_by(|left, right| left.stable_key.cmp(&right.stable_key));
        parts
            .star_rules
            .sort_by_key(|value| (value.role, value.input_star));
        parts.star_lifecycle.sort_by_key(|value| value.operation);
        parts.action_value_limits.sort_by_key(|value| value.kind);
        parts
            .battle_result_projections
            .sort_by_key(|value| value.outcome);
        validate(&parts)?;
        Ok(Self {
            currencies: parts.currencies.into_boxed_slice(),
            rules: parts.rules,
            offers: parts.offers.into_boxed_slice(),
            prices: parts.prices.into_boxed_slice(),
            team_levels: parts.team_levels.into_boxed_slice(),
            positions: parts.positions.into_boxed_slice(),
            star_states: parts.star_states.into_boxed_slice(),
            influence_properties: parts.influence_properties.into_boxed_slice(),
            contribution_parameters: parts.contribution_parameters.into_boxed_slice(),
            star_rules: parts.star_rules.into_boxed_slice(),
            star_lifecycle: parts.star_lifecycle.into_boxed_slice(),
            squad_hp: parts.squad_hp,
            action_value_limits: parts.action_value_limits.into_boxed_slice(),
            battle_result_projections: parts.battle_result_projections.into_boxed_slice(),
        })
    }

    #[must_use]
    pub fn currencies(&self) -> &[CurrencyWarsCurrency] {
        &self.currencies
    }

    #[must_use]
    pub const fn rules(&self) -> &CurrencyWarsEconomyRules {
        &self.rules
    }

    #[must_use]
    pub fn offers(&self) -> &[CurrencyWarsOfferLevel] {
        &self.offers
    }

    #[must_use]
    pub fn prices(&self) -> &[CurrencyWarsPriceRule] {
        &self.prices
    }

    #[must_use]
    pub fn team_levels(&self) -> &[CurrencyWarsTeamLevel] {
        &self.team_levels
    }

    #[must_use]
    pub fn positions(&self) -> &[CurrencyWarsPositionDefinition] {
        &self.positions
    }

    #[must_use]
    pub fn star_states(&self) -> &[CurrencyWarsStarState] {
        &self.star_states
    }

    #[must_use]
    pub fn influence_properties(&self) -> &[CurrencyWarsInfluenceProperty] {
        &self.influence_properties
    }

    #[must_use]
    pub fn contribution_parameters(&self) -> &[CurrencyWarsContributionParameter] {
        &self.contribution_parameters
    }

    #[must_use]
    pub fn star_rules(&self) -> &[CurrencyWarsStarRule] {
        &self.star_rules
    }

    #[must_use]
    pub fn star_lifecycle(&self) -> &[CurrencyWarsStarLifecycleRule] {
        &self.star_lifecycle
    }

    #[must_use]
    pub const fn squad_hp(&self) -> &CurrencyWarsSquadHpRules {
        &self.squad_hp
    }

    #[must_use]
    pub fn action_value_limits(&self) -> &[CurrencyWarsActionValueLimit] {
        &self.action_value_limits
    }

    #[must_use]
    pub fn battle_result_projections(&self) -> &[CurrencyWarsBattleResultProjection] {
        &self.battle_result_projections
    }
}

#[cfg(test)]
impl CurrencyWarsEconomyCatalog {
    pub(crate) fn test_fixture(role: CurrencyWarsRoleId) -> Self {
        Self::new(test_parts(role)).expect("test economy catalog is valid")
    }
}

#[cfg(test)]
fn test_parts(role: CurrencyWarsRoleId) -> CurrencyWarsEconomyCatalogParts {
    let star_states = (1..=3)
        .map(|star| CurrencyWarsStarState {
            stable_key: format!("star.{}.{}", role.get(), star).into_boxed_str(),
            owner: CurrencyWarsStarStateOwner::Role(role),
            star,
            copy_count: 3_u32.pow(u32::from(star.saturating_sub(1))),
            scaling_refs: Box::new([]),
            rank_attachments: Box::new([]),
            battle_event_id: None,
            skill_override_source_ids: Box::new([1]),
            front_execution_skill_ids: Box::new([1]),
            front_display_skill_ids: Box::new([1]),
            back_execution_skill_ids: Box::new([]),
            back_display_skill_ids: Box::new([]),
            back_ability_name: None,
            config_path: None,
            ai_path: None,
            property_modifiers: Box::new([]),
            front_power_base: CurrencyWarsDecimal::new(100, 0),
            back_power_base: CurrencyWarsDecimal::new(100, 0),
            luck_chance: None,
            luck_damage: None,
            extra_heal_base: None,
            extra_shield_base: None,
            hp_base: None,
            hp_inherit: None,
            hp_skill_id: None,
            speed_base: None,
            speed_inherit: None,
            speed_skill_id: None,
        })
        .collect();
    let star_rules = (1..=2)
        .map(|input_star| CurrencyWarsStarRule {
            stable_key: format!("star-rule.{}.{}", role.get(), input_star).into_boxed_str(),
            role,
            input_star,
            required_copies: 3,
            output_star: input_star + 1,
            overflow: CurrencyWarsStarOverflowRule::RepeatEqualStarTriples,
        })
        .collect();
    CurrencyWarsEconomyCatalogParts {
        currencies: vec![CurrencyWarsCurrency {
            stable_key: "currency-wars.currency.gold-coin".into(),
            gains: Box::new([
                CurrencyWarsCurrencyGain::AuthoredBattleEventInterestAndServiceOutcomes,
            ]),
            spends: Box::new([CurrencyWarsCurrencySpend::RecruitmentRefreshAndPricedServices]),
            reset: CurrencyWarsCurrencyReset::DiscardAtRunTeardown,
        }],
        rules: CurrencyWarsEconomyRules {
            stable_key: "economy.default".into(),
            currency_ids: Box::new(["currency-wars.currency.gold-coin".into()]),
            experience: CurrencyWarsExperienceRules {
                resource_id: 1,
                standard_wave_gain: 2,
                standard_boss_wave_gain: 10,
                overclock_wave_gain: [6, 8, 12],
                overclock_boss_wave_gain: [10, 12, 16],
                direct_level_up_experience: 4,
                direct_level_up_gold: 4,
            },
            interest: CurrencyWarsInterestRules {
                deposit_per_interest: 10,
                standard_maximum: 5,
                overclock_maximum: 0,
            },
            refresh: CurrencyWarsRefreshRules {
                cards_per_refresh: 5,
                gold_cost: 2,
                copies_per_role_by_rarity: [30, 25, 18, 10, 9],
                role_initial_weight: 100,
                maximum_stolen_same_card_by_rarity: [10, 8, 6, 3, 3],
                stolen_pool_refund_initial_purchase: 4,
                stolen_pool_refund_sell: 2,
                stolen_pool_refund_hold: 2,
            },
            team_size: CurrencyWarsTeamSizeRules {
                front_minimum: 1,
                front_maximum: 4,
                back_initial: 6,
                back_maximum: 9,
                bench_authored: 9,
                bench_overflow: 100,
            },
        },
        offers: (1..=10)
            .map(|level| CurrencyWarsOfferLevel {
                level,
                candidates: Box::new([role]),
                rarity_weights: [100, 0, 0, 0, 0],
                cost_rule: CurrencyWarsOfferCostRule::BuyGoldAtStarOne,
                fallback: CurrencyWarsOfferFallback::RejectIfNoPositiveRarityWeight,
            })
            .collect(),
        prices: (1..=5)
            .map(|rarity| CurrencyWarsPriceRule {
                stable_key: format!("price.{rarity}").into_boxed_str(),
                rarity,
                star_levels: Box::new([1, 2, 3, 4]),
                buy_by_star: Box::new([u32::from(rarity), 3, 9, 27]),
                sell_by_star: Box::new([u32::from(rarity), 3, 9, 27]),
                ordered_changes: Box::new([
                    CurrencyWarsTransactionChange::ValidateRosterAndGold,
                    CurrencyWarsTransactionChange::ApplyAuthoredGoldPrice,
                    CurrencyWarsTransactionChange::ApplyRosterMutation,
                ]),
            })
            .collect(),
        team_levels: (1..=10)
            .map(|level| CurrencyWarsTeamLevel {
                level,
                field_cap: level,
                bench_cap: 9,
                experience_to_next: (level < 10).then_some(2),
                transition: if level < 10 {
                    CurrencyWarsTeamLevelTransition::SpendExperienceToNext
                } else {
                    CurrencyWarsTeamLevelTransition::MaximumAuthoredLevel
                },
                properties: Box::new([]),
            })
            .collect(),
        positions: vec![
            position_fixture(
                "position.back",
                CurrencyWarsPositionDefinitionKind::Back,
                CurrencyWarsPositionEligibility::DirectBack,
            ),
            position_fixture(
                "position.front",
                CurrencyWarsPositionDefinitionKind::Front,
                CurrencyWarsPositionEligibility::DirectFront,
            ),
            position_fixture(
                "position.either",
                CurrencyWarsPositionDefinitionKind::FrontBackCandidate,
                CurrencyWarsPositionEligibility::MissingSourceTypeWithBothDisplays,
            ),
        ],
        star_states,
        influence_properties: vec![CurrencyWarsInfluenceProperty {
            stable_key: "influence.star.2".into(),
            subject: CurrencyWarsInfluenceSubject::Star,
            level: 2,
            properties: Box::new([CurrencyWarsAuthoredProperty {
                property: "fixture".into(),
                value: Some(CurrencyWarsDecimal::new(10, 0).unwrap()),
            }]),
        }],
        contribution_parameters: vec![CurrencyWarsContributionParameter {
            stable_key: "parameter.fixture".into(),
            kind: CurrencyWarsContributionParameterKind::RuntimeConstant,
            source_id: "fixture".into(),
            combination_ids: Box::new([]),
            bonus_numbers: Box::new([]),
            value_json: Some("0".into()),
            consumer_policy: "fixture".into(),
        }],
        star_rules,
        star_lifecycle: vec![
            lifecycle_fixture(
                "lifecycle.acquire",
                CurrencyWarsStarLifecycleOperation::AcquireCopy,
            ),
            lifecycle_fixture(
                "lifecycle.sell",
                CurrencyWarsStarLifecycleOperation::SellRole,
            ),
            lifecycle_fixture(
                "lifecycle.maximum",
                CurrencyWarsStarLifecycleOperation::AcquireAtMaximumStar,
            ),
        ],
        squad_hp: CurrencyWarsSquadHpRules {
            stable_key: "squad-hp.default".into(),
            initial: 100,
            minimum: 0,
            maximum: CurrencyWarsSquadHpMaximum::InitialWithContentDefinedIncreaseOrRecovery,
            loss_rules: Box::new([
                CurrencyWarsSquadHpLossRule::ConfiguredNodeOrDifficultyOnNonVictory,
            ]),
            recovery_rules: Box::new([
                CurrencyWarsSquadHpRecoveryRule::PreserveOnVictory,
                CurrencyWarsSquadHpRecoveryRule::AuthoredContentRestoreOrMaximumIncrease,
            ]),
        },
        action_value_limits: vec![
            CurrencyWarsActionValueLimit {
                stable_key: "action-value.finite".into(),
                kind: CurrencyWarsActionValueLimitKind::FiniteNodeConfigured,
                initial: CurrencyWarsActionValueInitial::ConfiguredByNodeOrDifficulty,
                decrements: Box::new([
                    CurrencyWarsActionValueDecrement::ElapsedAuthoritativeActionValue,
                    CurrencyWarsActionValueDecrement::ConfiguredCharacterLethalRescue,
                ]),
                timeout: CurrencyWarsTimeoutBoundary::NonVictoryAndConfiguredSquadHpLoss,
            },
            CurrencyWarsActionValueLimit {
                stable_key: "action-value.unlimited".into(),
                kind: CurrencyWarsActionValueLimitKind::Unlimited,
                initial: CurrencyWarsActionValueInitial::Infinite,
                decrements: Box::new([]),
                timeout: CurrencyWarsTimeoutBoundary::Unreachable,
            },
        ],
        battle_result_projections: vec![
            CurrencyWarsBattleResultProjection {
                stable_key: "projection.victory".into(),
                outcome: CurrencyWarsBattleOutcomeProjection::Victory,
                squad_hp: CurrencyWarsSquadHpProjection::PreserveBeforeContentContributions,
                action_value: CurrencyWarsActionValueProjection::CaptureForFinalizationThenDiscard,
                run: CurrencyWarsRunDisposition::ContinueUnlessFinalBoss,
            },
            CurrencyWarsBattleResultProjection {
                stable_key: "projection.non-victory".into(),
                outcome: CurrencyWarsBattleOutcomeProjection::NonVictory,
                squad_hp: CurrencyWarsSquadHpProjection::SubtractConfiguredLossClampToZero,
                action_value: CurrencyWarsActionValueProjection::CaptureExhaustedThenDiscard,
                run: CurrencyWarsRunDisposition::FailAtZeroOtherwiseContinue,
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unresolved_currency_reference_rejects_the_catalog() {
        let role = CurrencyWarsRoleId::new(1).unwrap();
        let mut parts = test_parts(role);
        parts.rules.currency_ids = Box::new(["currency-wars.currency.missing".into()]);

        let error = CurrencyWarsEconomyCatalog::new(parts).unwrap_err();

        assert!(error.to_string().contains("currency reference"));
    }
}

#[cfg(test)]
fn position_fixture(
    stable_key: &'static str,
    kind: CurrencyWarsPositionDefinitionKind,
    eligibility: CurrencyWarsPositionEligibility,
) -> CurrencyWarsPositionDefinition {
    CurrencyWarsPositionDefinition {
        stable_key: stable_key.into(),
        kind,
        field_index: "fixture".into(),
        eligibility,
    }
}

#[cfg(test)]
fn lifecycle_fixture(
    stable_key: &'static str,
    operation: CurrencyWarsStarLifecycleOperation,
) -> CurrencyWarsStarLifecycleRule {
    CurrencyWarsStarLifecycleRule {
        stable_key: stable_key.into(),
        operation,
    }
}

fn validate(
    parts: &CurrencyWarsEconomyCatalogParts,
) -> Result<(), CurrencyWarsEconomyCatalogError> {
    if parts.currencies.is_empty()
        || parts.offers.is_empty()
        || parts.prices.is_empty()
        || parts.team_levels.is_empty()
        || parts.positions.is_empty()
        || parts.star_states.is_empty()
        || parts.influence_properties.is_empty()
        || parts.contribution_parameters.is_empty()
        || parts.star_rules.is_empty()
        || parts.star_lifecycle.is_empty()
        || parts.action_value_limits.is_empty()
        || parts.battle_result_projections.is_empty()
        || parts.squad_hp.initial == 0
        || parts.squad_hp.minimum > parts.squad_hp.initial
        || parts.rules.interest.deposit_per_interest == 0
        || parts.rules.refresh.cards_per_refresh == 0
        || parts.rules.refresh.copies_per_role_by_rarity.contains(&0)
        || parts
            .rules
            .refresh
            .maximum_stolen_same_card_by_rarity
            .contains(&0)
        || parts.rules.refresh.stolen_pool_refund_initial_purchase == 0
        || parts.rules.refresh.stolen_pool_refund_sell == 0
        || parts.rules.refresh.stolen_pool_refund_hold == 0
        || parts.rules.refresh.role_initial_weight == 0
        || parts.rules.team_size.front_minimum > parts.rules.team_size.front_maximum
        || parts.rules.team_size.back_initial > parts.rules.team_size.back_maximum
        || u16::from(parts.rules.team_size.bench_authored) > parts.rules.team_size.bench_overflow
    {
        return Err(error("Currency Wars economy catalog is empty or invalid"));
    }
    let currency_ids = parts
        .currencies
        .iter()
        .map(|value| value.stable_key.as_ref())
        .collect::<BTreeSet<_>>();
    if currency_ids.len() != parts.currencies.len()
        || parts
            .rules
            .currency_ids
            .iter()
            .any(|id| !currency_ids.contains(id.as_ref()))
    {
        return Err(error("Currency Wars economy currency reference is invalid"));
    }
    if !all_unique(parts.offers.iter().map(|value| value.level))
        || !all_unique(parts.prices.iter().map(|value| value.rarity))
        || !all_unique(parts.team_levels.iter().map(|value| value.level))
        || !all_unique(parts.positions.iter().map(|value| value.kind))
        || !all_unique(parts.star_lifecycle.iter().map(|value| value.operation))
        || !all_unique(
            parts
                .influence_properties
                .iter()
                .map(|value| (value.subject, value.level)),
        )
        || !all_unique(parts.action_value_limits.iter().map(|value| value.kind))
        || !all_unique(
            parts
                .battle_result_projections
                .iter()
                .map(|value| value.outcome),
        )
    {
        return Err(error(
            "Currency Wars economy catalog identity is duplicated",
        ));
    }
    if parts.prices.iter().any(|price| {
        price.star_levels.as_ref() != [1, 2, 3, 4]
            || price.buy_by_star.len() != price.star_levels.len()
            || price.sell_by_star.len() != price.star_levels.len()
            || price.ordered_changes.as_ref()
                != [
                    CurrencyWarsTransactionChange::ValidateRosterAndGold,
                    CurrencyWarsTransactionChange::ApplyAuthoredGoldPrice,
                    CurrencyWarsTransactionChange::ApplyRosterMutation,
                ]
    }) {
        return Err(error(
            "Currency Wars roster transaction definition is invalid",
        ));
    }
    let states = parts
        .star_states
        .iter()
        .map(|state| (state.owner, state.star))
        .collect::<BTreeSet<_>>();
    if states.len() != parts.star_states.len()
        || parts.star_states.iter().any(|state| {
            state.skill_override_source_ids.len() != state.front_execution_skill_ids.len()
        })
        || !all_unique(
            parts
                .star_rules
                .iter()
                .map(|rule| (rule.role, rule.input_star)),
        )
        || parts.star_rules.iter().any(|rule| {
            rule.required_copies < 2
                || rule.output_star != rule.input_star.saturating_add(1)
                || !states.contains(&(CurrencyWarsStarStateOwner::Role(rule.role), rule.input_star))
                || !states.contains(&(
                    CurrencyWarsStarStateOwner::Role(rule.role),
                    rule.output_star,
                ))
        })
    {
        return Err(error("Currency Wars star-rule state reference is invalid"));
    }
    Ok(())
}

fn all_unique<T: Ord>(values: impl Iterator<Item = T>) -> bool {
    let mut seen = BTreeSet::new();
    values.into_iter().all(|value| seen.insert(value))
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsEconomyCatalogError {
    message: Box<str>,
}

impl std::fmt::Display for CurrencyWarsEconomyCatalogError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for CurrencyWarsEconomyCatalogError {}

fn error(message: &'static str) -> CurrencyWarsEconomyCatalogError {
    CurrencyWarsEconomyCatalogError {
        message: message.into(),
    }
}

impl CurrencyWarsEconomyCatalog {
    #[must_use]
    pub fn experience_reward(&self, gambit: CurrencyWarsGambit, plane: u8, boss: bool) -> u32 {
        let rules = self.rules.experience;
        match (gambit, boss) {
            (CurrencyWarsGambit::Standard, true) => rules.standard_boss_wave_gain,
            (CurrencyWarsGambit::Standard, false) => rules.standard_wave_gain,
            (CurrencyWarsGambit::Overclock, true) => {
                rules.overclock_boss_wave_gain[usize::from(plane.saturating_sub(1).min(2))]
            }
            (CurrencyWarsGambit::Overclock, false) => {
                rules.overclock_wave_gain[usize::from(plane.saturating_sub(1).min(2))]
            }
        }
    }
}
