//! Explicit non-parity numeric policy for unpublished Conundrum values.

use super::GoldAndGearsEntryError;

/// Policy revision replacing Goal 08's fail-closed numeric fields.
pub const GOLD_AND_GEARS_CONUNDRUM_POLICY_REVISION: &str =
    "gold-and-gears-conundrum-numeric-policy-v1";

/// Accuracy label retained beside every policy-projected numeric.
pub const GOLD_AND_GEARS_CONUNDRUM_POLICY_ACCURACY: &str =
    "DeterministicProjectPolicyNotObservedParity";

/// Evidence condition that permits replacement of the runtime policy.
pub const GOLD_AND_GEARS_CONUNDRUM_POLICY_REPLACEMENT_CONDITION: &str =
    "Replace with pinned released engine values or reproducible Version 4.4 observations.";

/// Four released qualitative enemy-stat tiers.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum GoldAndGearsEnemyStatTier {
    Slight,
    Moderate,
    Great,
    Massive,
}

/// Versioned numeric projection for one qualitative enemy-stat tier.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GoldAndGearsEnemyStatPolicy {
    tier: GoldAndGearsEnemyStatTier,
    attack_ratio_scaled: i64,
    maximum_hp_ratio_scaled: i64,
    speed_ratio_scaled: i64,
}

impl GoldAndGearsEnemyStatPolicy {
    #[must_use]
    pub const fn tier(self) -> GoldAndGearsEnemyStatTier {
        self.tier
    }

    #[must_use]
    pub const fn attack_ratio_scaled(self) -> i64 {
        self.attack_ratio_scaled
    }

    #[must_use]
    pub const fn maximum_hp_ratio_scaled(self) -> i64 {
        self.maximum_hp_ratio_scaled
    }

    #[must_use]
    pub const fn speed_ratio_scaled(self) -> i64 {
        self.speed_ratio_scaled
    }
}

/// Base or enhanced elite/boss Berserk policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GoldAndGearsBerserkPolicy {
    enhanced: bool,
    trigger_cycle: u8,
    attack_ratio_per_stack_scaled: i64,
    speed_ratio_per_stack_scaled: i64,
    stack_interval_cycles: u8,
    stack_cap: u8,
}

impl GoldAndGearsBerserkPolicy {
    #[must_use]
    pub const fn enhanced(self) -> bool {
        self.enhanced
    }

    #[must_use]
    pub const fn trigger_cycle(self) -> u8 {
        self.trigger_cycle
    }

    #[must_use]
    pub const fn attack_ratio_per_stack_scaled(self) -> i64 {
        self.attack_ratio_per_stack_scaled
    }

    #[must_use]
    pub const fn speed_ratio_per_stack_scaled(self) -> i64 {
        self.speed_ratio_per_stack_scaled
    }

    #[must_use]
    pub const fn stack_interval_cycles(self) -> u8 {
        self.stack_interval_cycles
    }

    #[must_use]
    pub const fn stack_cap(self) -> u8 {
        self.stack_cap
    }
}

/// Level-five elite/boss Toughness and Berserk response policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GoldAndGearsEliteBossResponsePolicy {
    toughness_ratio_scaled: i64,
    action_advance_ratio_scaled: i64,
}

impl GoldAndGearsEliteBossResponsePolicy {
    #[must_use]
    pub const fn toughness_ratio_scaled(self) -> i64 {
        self.toughness_ratio_scaled
    }

    #[must_use]
    pub const fn action_advance_ratio_scaled(self) -> i64 {
        self.action_advance_ratio_scaled
    }
}

pub(super) const fn enemy_stat_policy(
    tier: &str,
) -> Result<GoldAndGearsEnemyStatPolicy, GoldAndGearsEntryError> {
    let (tier, attack, hp, speed) = match tier.as_bytes() {
        b"Slight" => (GoldAndGearsEnemyStatTier::Slight, 100_000, 100_000, 25_000),
        b"Moderate" => (
            GoldAndGearsEnemyStatTier::Moderate,
            200_000,
            200_000,
            50_000,
        ),
        b"Great" => (GoldAndGearsEnemyStatTier::Great, 300_000, 300_000, 75_000),
        b"Massive" => (
            GoldAndGearsEnemyStatTier::Massive,
            400_000,
            400_000,
            100_000,
        ),
        _ => return Err(GoldAndGearsEntryError::InvalidConundrumRuntime),
    };
    Ok(GoldAndGearsEnemyStatPolicy {
        tier,
        attack_ratio_scaled: attack,
        maximum_hp_ratio_scaled: hp,
        speed_ratio_scaled: speed,
    })
}

pub(super) const fn berserk_policy(enhanced: bool) -> GoldAndGearsBerserkPolicy {
    if enhanced {
        GoldAndGearsBerserkPolicy {
            enhanced: true,
            trigger_cycle: 6,
            attack_ratio_per_stack_scaled: 150_000,
            speed_ratio_per_stack_scaled: 75_000,
            stack_interval_cycles: 1,
            stack_cap: 5,
        }
    } else {
        GoldAndGearsBerserkPolicy {
            enhanced: false,
            trigger_cycle: 8,
            attack_ratio_per_stack_scaled: 100_000,
            speed_ratio_per_stack_scaled: 50_000,
            stack_interval_cycles: 1,
            stack_cap: 5,
        }
    }
}

pub(super) const fn elite_boss_response_policy() -> GoldAndGearsEliteBossResponsePolicy {
    GoldAndGearsEliteBossResponsePolicy {
        toughness_ratio_scaled: 100_000,
        action_advance_ratio_scaled: 100_000,
    }
}
