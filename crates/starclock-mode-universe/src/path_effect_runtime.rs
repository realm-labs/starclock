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

/// Typed, immutable facts supplied by the battle adapter for one observation.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PathEffectFacts {
    pub actor_current_shield: PathEffectValue,
    pub actor_shield_before_hit: PathEffectValue,
    pub teammate_shield_total: PathEffectValue,
    pub party_shield_total: PathEffectValue,
    pub actor_maximum_hp: PathEffectValue,
    pub actor_defense: PathEffectValue,
    pub actor_base_attack: PathEffectValue,
    pub hp_lost: PathEffectValue,
    pub provided_shield: PathEffectValue,
    pub path_damage: PathEffectValue,
    pub path_base_damage: PathEffectValue,
    pub damage_dealt: PathEffectValue,
    pub enemy_current_hp_ratio: PathEffectValue,
    pub path_blessing_count: u32,
    pub shielded_allies: u32,
    pub enemy_attack_count: u32,
    pub actor_is_shielded: bool,
    pub enemy_is_frozen: bool,
    pub enemy_is_dissociated: bool,
    pub enemy_has_dissociation_vulnerability: bool,
    pub enemy_crossed_hp_threshold_first_time: bool,
    pub action_is_skill_or_ultimate: bool,
}

impl PathEffectFacts {
    pub(crate) fn validate(self) -> Result<Self, PathEffectRuntimeError> {
        let values = [
            self.actor_current_shield,
            self.actor_shield_before_hit,
            self.teammate_shield_total,
            self.party_shield_total,
            self.actor_maximum_hp,
            self.actor_defense,
            self.actor_base_attack,
            self.hp_lost,
            self.provided_shield,
            self.path_damage,
            self.path_base_damage,
            self.damage_dealt,
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
            if parameter.scale() > 6 {
                return Err(PathEffectRuntimeError::InvalidParameter);
            }
            let multiplier = 10_i64
                .checked_pow(u32::from(6 - parameter.scale()))
                .ok_or(PathEffectRuntimeError::Overflow)?;
            parameter
                .coefficient()
                .checked_mul(multiplier)
                .map(PathEffectValue::from_raw_six_decimal)
                .ok_or(PathEffectRuntimeError::Overflow)
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
