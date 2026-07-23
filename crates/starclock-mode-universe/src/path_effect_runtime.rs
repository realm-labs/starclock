//! Engine-independent typed proposals emitted by Path mechanic executors.

use crate::path::ExactParameter;

/// Signed six-decimal value used at the Universe-to-battle contribution boundary.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PathEffectValue(i64);

impl PathEffectValue {
    pub const ZERO: Self = Self(0);
    pub const ONE: Self = Self(1_000_000);

    #[must_use]
    pub const fn from_raw_six_decimal(value: i64) -> Self {
        Self(value)
    }

    pub fn from_integral(value: i64) -> Result<Self, PathEffectRuntimeError> {
        value
            .checked_mul(1_000_000)
            .map(Self)
            .ok_or(PathEffectRuntimeError::Overflow)
    }

    #[must_use]
    pub const fn raw_six_decimal(self) -> i64 {
        self.0
    }

    pub(crate) fn checked_add(self, other: Self) -> Result<Self, PathEffectRuntimeError> {
        self.0
            .checked_add(other.0)
            .map(Self)
            .ok_or(PathEffectRuntimeError::Overflow)
    }

    pub(crate) fn checked_multiply_ratio(
        self,
        ratio: Self,
    ) -> Result<Self, PathEffectRuntimeError> {
        let product = i128::from(self.0)
            .checked_mul(i128::from(ratio.0))
            .ok_or(PathEffectRuntimeError::Overflow)?;
        let scaled = product / 1_000_000;
        i64::try_from(scaled)
            .map(Self)
            .map_err(|_| PathEffectRuntimeError::Overflow)
    }

    pub(crate) fn checked_multiply_count(self, count: u32) -> Result<Self, PathEffectRuntimeError> {
        self.0
            .checked_mul(i64::from(count))
            .map(Self)
            .ok_or(PathEffectRuntimeError::Overflow)
    }
}

/// Stable combat observation point accepted by released Path mechanics.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum PathBattleEvent {
    BattleStarted = 0,
    TurnEnded = 1,
    AttackHit = 2,
    CharacterAttacked = 3,
    WeaknessBroken = 4,
    ShieldGranted = 5,
    ShieldGrantedToAlly = 6,
    PathDamageDealt = 7,
    PathResonanceActivated = 8,
    StatQueried = 9,
    DamageCalculated = 10,
    DamageDealt = 11,
    IceDamageDealt = 12,
    EnemyFrozen = 13,
    DissociationRemoved = 14,
    UltimateUsed = 15,
    DotDamageTaken = 16,
    DotApplied = 17,
    DotRefreshed = 18,
    EnemyTurnStarted = 19,
    EnemyDefeated = 20,
    SuspicionApplying = 21,
    HealingReceived = 22,
    TurnStarted = 23,
    HealingProvided = 24,
    DewdropRuptured = 25,
    AttackCompleted = 26,
    LethalDamageReceived = 27,
    FollowUpAttackUsed = 28,
    ConsecutiveActionStarted = 29,
}

/// Cause-relative target retained until the combat adapter resolves unit IDs.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum PathEffectTarget {
    Actor = 0,
    Attacker = 1,
    HitEnemies = 2,
    PrimaryEnemy = 3,
    AdjacentEnemies = 4,
    OtherEnemies = 5,
    AllEnemies = 6,
    AllAllies = 7,
    ShieldProvider = 8,
    RandomEnemy = 9,
    RandomEnemyWithoutIceWeakness = 10,
    RandomOtherEnemies = 11,
    RandomAlly = 12,
    OtherAllies = 13,
    HealerAndHealed = 14,
    HighestAttackAlly = 15,
}

/// Generic stat families used by Path conditional modifiers.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum PathEffectStat {
    AttackFlat = 0,
    DefenseRatio = 1,
    CriticalRateRatio = 2,
    CriticalDamageRatio = 3,
    DamageTakenReductionRatio = 4,
    ShieldCapacityRatio = 5,
    PathDamageRatio = 6,
    DamageRatio = 7,
    DamageTakenRatio = 8,
    EffectHitRateRatio = 9,
    FreezeResistanceReductionRatio = 10,
    AttackReductionRatio = 11,
    EffectResistanceReductionRatio = 12,
    WeaknessBreakEfficiencyRatio = 13,
    BreakEffectRatio = 14,
    DotDamageRatio = 15,
    DotDamageTakenRatio = 16,
    MaximumHpRatio = 17,
    EffectResistanceRatio = 18,
    HealingReceivedRatio = 19,
    SpeedRatio = 20,
    AttackRatio = 21,
}

/// Mode damage family retained until lowering into the combat damage class.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum PathEffectDamageKind {
    PathAdditional = 0,
    PathResonance = 1,
}

/// Element selection retained explicitly at the mode boundary.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum PathEffectElement {
    Physical = 0,
    Fire = 1,
    Ice = 2,
    Lightning = 3,
    Wind = 4,
    Quantum = 5,
    Imaginary = 6,
    InheritActor = 7,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum PathDotSelection {
    All = 0,
    RandomOne = 1,
}

/// Typed, immutable facts supplied by the battle adapter for one observation.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PathEffectFacts {
    pub actor_current_shield: PathEffectValue,
    pub actor_shield_before_hit: PathEffectValue,
    pub teammate_shield_total: PathEffectValue,
    pub party_shield_total: PathEffectValue,
    pub actor_maximum_hp: PathEffectValue,
    pub actor_current_hp: PathEffectValue,
    pub actor_defense: PathEffectValue,
    pub actor_base_attack: PathEffectValue,
    pub hp_lost: PathEffectValue,
    pub provided_shield: PathEffectValue,
    pub path_damage: PathEffectValue,
    pub path_base_damage: PathEffectValue,
    pub damage_dealt: PathEffectValue,
    pub healing_amount: PathEffectValue,
    pub dewdrop_charge: PathEffectValue,
    pub actor_critical_rate_ratio: PathEffectValue,
    pub highest_ally_attack: PathEffectValue,
    pub last_acting_ally_attack: PathEffectValue,
    pub enemy_current_hp_ratio: PathEffectValue,
    pub path_blessing_count: u32,
    pub shielded_allies: u32,
    pub enemy_attack_count: u32,
    pub suspicion_stacks: u32,
    pub dot_count: u32,
    pub critical_boost_stacks: u32,
    pub consecutive_action_count: u32,
    pub allied_turn_count: u32,
    pub actor_is_shielded: bool,
    pub enemy_is_frozen: bool,
    pub enemy_is_dissociated: bool,
    pub enemy_has_dissociation_vulnerability: bool,
    pub enemy_crossed_hp_threshold_first_time: bool,
    pub action_is_skill_or_ultimate: bool,
    pub enemy_is_weakness_broken: bool,
    pub enemy_has_dot: bool,
    pub dot_was_refreshed: bool,
    pub actor_is_full_hp: bool,
    pub healing_was_from_ally: bool,
    pub weakness_broken_enemy_is_elite: bool,
}

impl PathEffectFacts {
    pub(crate) fn validate(self) -> Result<Self, PathEffectRuntimeError> {
        let values = [
            self.actor_current_shield,
            self.actor_shield_before_hit,
            self.teammate_shield_total,
            self.party_shield_total,
            self.actor_maximum_hp,
            self.actor_current_hp,
            self.actor_defense,
            self.actor_base_attack,
            self.hp_lost,
            self.provided_shield,
            self.path_damage,
            self.path_base_damage,
            self.damage_dealt,
            self.healing_amount,
            self.dewdrop_charge,
            self.actor_critical_rate_ratio,
            self.highest_ally_attack,
            self.last_acting_ally_attack,
            self.enemy_current_hp_ratio,
        ];
        if values.iter().any(|value| value.raw_six_decimal() < 0)
            || self.enemy_current_hp_ratio > PathEffectValue::ONE
        {
            Err(PathEffectRuntimeError::InvalidFacts)
        } else {
            Ok(self)
        }
    }
}

/// Closed proposal set emitted by the nine Path-specific executors.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PathEffect {
    Damage {
        target: PathEffectTarget,
        amount: PathEffectValue,
        kind: PathEffectDamageKind,
        element: PathEffectElement,
        can_defeat: bool,
        force_critical: bool,
        critical_damage_ratio: PathEffectValue,
    },
    Shield {
        target: PathEffectTarget,
        amount: PathEffectValue,
        duration_turns: u8,
        special: bool,
        fixed_chance: PathEffectValue,
    },
    StrengthenSpecialShield {
        target: PathEffectTarget,
        amount: PathEffectValue,
        cycle_turns: u8,
    },
    AddStat {
        target: PathEffectTarget,
        stat: PathEffectStat,
        value: PathEffectValue,
        cap: Option<PathEffectValue>,
    },
    ApplyBleed {
        target: PathEffectTarget,
        base_chance: PathEffectValue,
        maximum_hp_ratio: PathEffectValue,
        damage_cap_ratio: PathEffectValue,
        duration_turns: u8,
    },
    DispelDebuff {
        target: PathEffectTarget,
        fixed_chance: PathEffectValue,
        count: u8,
    },
    GainResonanceEnergy {
        maximum_ratio: PathEffectValue,
    },
    ApplyAmber {
        target: PathEffectTarget,
    },
    ApplyFreeze {
        target: PathEffectTarget,
        base_chance: PathEffectValue,
        duration_turns: u8,
        speed_reduction_ratio: PathEffectValue,
        ignore_freeze_resistance: bool,
    },
    ApplyDissociation {
        target: PathEffectTarget,
        base_chance: PathEffectValue,
        duration_turns: u8,
        maximum_hp_damage_ratio: PathEffectValue,
        removal_damage_bonus_ratio: PathEffectValue,
        ignore_freeze_resistance: bool,
    },
    RemoveDissociation {
        target: PathEffectTarget,
        removal_damage_multiplier: PathEffectValue,
    },
    ApplyIceWeakness {
        target: PathEffectTarget,
        base_chance: PathEffectValue,
        duration_turns: u8,
    },
    ApplyEonianRiver {
        target: PathEffectTarget,
        base_chance: PathEffectValue,
        duration_turns: u8,
    },
    ApplyFreezeResistanceReduction {
        target: PathEffectTarget,
        base_chance: PathEffectValue,
        value: PathEffectValue,
        duration_turns: u8,
    },
    MarkCriticalExposure {
        target: PathEffectTarget,
        attacks: u8,
        critical_rate_ratio: PathEffectValue,
    },
    GainEnergy {
        target: PathEffectTarget,
        amount: PathEffectValue,
        once_per_action: bool,
    },
    ApplySuspicion {
        target: PathEffectTarget,
        stacks: u8,
        maximum_stacks: u8,
        dot_vulnerability_per_stack: PathEffectValue,
        decay_per_turn: u8,
        prevent_decay: bool,
    },
    ModifySuspicionApplication {
        extra_stacks: u8,
        multiplier: u8,
    },
    SpreadSuspicion {
        target: PathEffectTarget,
        target_count: u8,
        stacks: u8,
    },
    TriggerDots {
        target: PathEffectTarget,
        selection: PathDotSelection,
        times: u8,
        damage_ratio: PathEffectValue,
    },
    SpreadWeaknessBreak {
        target: PathEffectTarget,
    },
    ApplyRandomBreakDot {
        target: PathEffectTarget,
        base_chance: PathEffectValue,
        duration_turns: u8,
        wind_shear_stacks: u8,
        burn_shock_attack_ratio: PathEffectValue,
        bleed_maximum_hp_ratio: PathEffectValue,
        dispel_attacker_debuff: bool,
    },
    ExtendStandardDots {
        target: PathEffectTarget,
        duration_turns: u8,
    },
    HealMaximumHpRatio {
        target: PathEffectTarget,
        ratio: PathEffectValue,
    },
    ApplyResonanceDots {
        target: PathEffectTarget,
        base_chance: PathEffectValue,
        duration_turns: u8,
        wind_shear_stacks: u8,
        burn_shock_attack_ratio: PathEffectValue,
        bleed_maximum_hp_ratio: PathEffectValue,
    },
    ModifyResonanceDotApplication {
        base_chance_bonus: PathEffectValue,
        duration_bonus_turns: u8,
        stackable_status_bonus: u8,
    },
    ApplyConfusionAndDevoid {
        target: PathEffectTarget,
        base_chance: PathEffectValue,
        confusion_stacks: u8,
        confusion_dot_trigger_ratio: PathEffectValue,
        devoid_stacks: u8,
        toughness_recovery_reduction_per_stack: PathEffectValue,
        duration_turns: u8,
    },
    ChargeDewdrop {
        target: PathEffectTarget,
        amount: PathEffectValue,
        maximum_hp_cap_ratio: PathEffectValue,
        damage_bonus_ratio: PathEffectValue,
        ruptures_after_attack: bool,
    },
    ModifyDewdropChargeEfficiency {
        target: PathEffectTarget,
        value: PathEffectValue,
    },
    HealAmount {
        target: PathEffectTarget,
        amount: PathEffectValue,
        once_per_action: bool,
    },
    ApplyTimedStat {
        target: PathEffectTarget,
        stat: PathEffectStat,
        value: PathEffectValue,
        duration_turns: u8,
        maximum_stacks: u8,
    },
    ScaleAttackFromHealing {
        target: PathEffectTarget,
        healing_ratio: PathEffectValue,
        base_attack_cap_ratio: PathEffectValue,
        until_next_turn_end: bool,
    },
    GainSkillPoint {
        fixed_chance: PathEffectValue,
        amount: u8,
        once_per_action: bool,
    },
    PreventDefeatAndActivateResonance {
        target: PathEffectTarget,
        maximum_triggers_per_battle: u8,
        consume_all_energy: bool,
    },
    ApplySubduingEvils {
        target: PathEffectTarget,
        stacks: u8,
        maximum_stacks: u8,
        duration_turns: u8,
        blocked_debuffs_per_stack: u8,
        heal_maximum_hp_ratio_on_block: PathEffectValue,
    },
    InstallResonanceAction {
        healing_reduction_ratio: PathEffectValue,
        activate_after_first_manual_use: bool,
    },
    ApplyCriticalBoost {
        target: PathEffectTarget,
        stacks: u8,
        maximum_stacks: u8,
        critical_rate_ratio_per_stack: PathEffectValue,
        critical_damage_ratio_per_stack: PathEffectValue,
        at_next_turn_start: bool,
    },
    ActionAdvance {
        target: PathEffectTarget,
        ratio: PathEffectValue,
        cannot_repeat_for_same_actor: bool,
    },
    IncreaseNextAttackDamage {
        target: PathEffectTarget,
        ratio: PathEffectValue,
    },
    CriticalDamageFromExcessRate {
        target: PathEffectTarget,
        excess_rate_multiplier: PathEffectValue,
        per_critical_boost_stack: PathEffectValue,
        cap: PathEffectValue,
    },
    InheritCriticalBoost {
        target: PathEffectTarget,
        extra_stacks: u8,
        maximum_stacks: u8,
    },
    GainEnergyMaximumRatio {
        target: PathEffectTarget,
        ratio: PathEffectValue,
    },
    DelayAction {
        target: PathEffectTarget,
        ratio: PathEffectValue,
    },
    ApplyUntilAttackedStat {
        target: PathEffectTarget,
        stat: PathEffectStat,
        value: PathEffectValue,
    },
    ScaleAttackFromLastAlly {
        target: PathEffectTarget,
        source_attack: PathEffectValue,
        ratio: PathEffectValue,
        until_next_turn_start: bool,
    },
    ApplyLightHuntingCelestialArrow {
        target: PathEffectTarget,
        critical_damage_from_critical_rate_ratio: PathEffectValue,
        extra_turn_after_defeat: bool,
        cannot_repeat: bool,
        expires_after_ability: bool,
    },
    ModifyResonanceCritical {
        guaranteed_critical_below_hp_ratio: PathEffectValue,
        critical_damage_ratio: PathEffectValue,
        defeated_energy_maximum_ratio: PathEffectValue,
    },
    ConfigureResonanceEnergy {
        maximum: PathEffectValue,
        gain_on_ally_turn_ratio: PathEffectValue,
    },
    ConfigureTurnAdvanceCounter {
        target: PathEffectTarget,
        turn_interval: u8,
        initial_turns: u8,
        cannot_repeat_for_same_actor: bool,
    },
}

/// One source-attributed proposal. Adapters must preserve this source in causes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppliedPathEffect {
    source_key: Box<str>,
    effect: PathEffect,
}

impl AppliedPathEffect {
    #[must_use]
    pub(crate) fn new(source_key: &str, effect: PathEffect) -> Self {
        Self {
            source_key: source_key.into(),
            effect,
        }
    }

    #[must_use]
    pub fn source_key(&self) -> &str {
        &self.source_key
    }

    #[must_use]
    pub const fn effect(&self) -> &PathEffect {
        &self.effect
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PathEffectRuntimeError {
    InvalidFacts,
    InvalidParameter,
    InvalidDefinition,
    UnknownSource,
    Overflow,
}

pub(crate) fn exact_parameters(
    parameters: &[ExactParameter],
) -> Result<Box<[PathEffectValue]>, PathEffectRuntimeError> {
    parameters
        .iter()
        .map(|parameter| {
            let coefficient = parameter.coefficient();
            if parameter.scale() <= 6 {
                let multiplier = 10_i64
                    .checked_pow(u32::from(6 - parameter.scale()))
                    .ok_or(PathEffectRuntimeError::Overflow)?;
                return coefficient
                    .checked_mul(multiplier)
                    .map(PathEffectValue::from_raw_six_decimal)
                    .ok_or(PathEffectRuntimeError::Overflow);
            }
            let divisor = 10_i64
                .checked_pow(u32::from(parameter.scale() - 6))
                .ok_or(PathEffectRuntimeError::Overflow)?;
            let quotient = coefficient / divisor;
            let remainder = coefficient % divisor;
            let rounds_away = i128::from(remainder).abs() * 2 >= i128::from(divisor);
            let rounded = if rounds_away {
                quotient
                    .checked_add(coefficient.signum())
                    .ok_or(PathEffectRuntimeError::Overflow)?
            } else {
                quotient
            };
            Ok(PathEffectValue::from_raw_six_decimal(rounded))
        })
        .collect::<Result<Vec<_>, _>>()
        .map(Vec::into_boxed_slice)
}

pub(crate) fn count(value: PathEffectValue) -> Result<u32, PathEffectRuntimeError> {
    let raw = value.raw_six_decimal();
    if raw < 0 || raw % 1_000_000 != 0 {
        return Err(PathEffectRuntimeError::InvalidParameter);
    }
    u32::try_from(raw / 1_000_000).map_err(|_| PathEffectRuntimeError::InvalidParameter)
}

pub(crate) fn turns(value: PathEffectValue) -> Result<u8, PathEffectRuntimeError> {
    u8::try_from(count(value)?).map_err(|_| PathEffectRuntimeError::InvalidParameter)
}
