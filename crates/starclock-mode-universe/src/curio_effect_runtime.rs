//! Closed effect executor for positive, neutral and special Standard Curios.

use crate::{
    catalog::UniverseCatalog,
    curio_runtime::CurioRuntimeCatalog,
    digest::Encoder,
    id::{CurioId, PathId},
    path_effect_runtime::{
        PathEffect, PathEffectStat, PathEffectTarget, PathEffectValue, exact_parameters, turns,
    },
};

pub const CURIO_EFFECT_RUNTIME_REVISION: &str = "standard-universe-curio-effect-runtime-v1";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum CurioEvent {
    Acquired = 0,
    BattleWon = 1,
    BlessingRewardOffered = 2,
    DomainEntered = 3,
    BattleStarted = 4,
    CharacterTurnStarted = 5,
    DestructibleDestroyed = 6,
    TechniqueDamageCalculated = 7,
    StatQueried = 8,
    RunDefeated = 9,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CurioEffectFacts {
    pub cosmic_fragments: u32,
    pub destroyed_curios: u32,
    pub full_hp_allies: u32,
    pub different_path_blessings: u32,
    pub destructibles_destroyed: u32,
    pub actor_maximum_hp: PathEffectValue,
    pub technique_actor_maximum_hp: PathEffectValue,
    pub final_domain: bool,
}

impl CurioEffectFacts {
    fn validate(self) -> Result<Self, CurioEffectRuntimeError> {
        if self.actor_maximum_hp.raw_six_decimal() < 0
            || self.technique_actor_maximum_hp.raw_six_decimal() < 0
        {
            Err(CurioEffectRuntimeError::InvalidFacts)
        } else {
            Ok(self)
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum CurioBlessingRarity {
    ThreeStar = 0,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum CurioDestructibleReward {
    Curio = 0,
    Blessing = 1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum CurioHpChange {
    Consume = 0,
    Restore = 1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum CurioEnergyChange {
    Clear = 0,
    RestoreMaximum = 1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CurioEffect {
    Battle(PathEffect),
    GrantRandomBlessings {
        path: Option<PathId>,
        minimum: u8,
        maximum: u8,
    },
    BiasBlessingOffers {
        path: PathId,
    },
    ConfigureBlessingReward {
        extra_selections: u8,
        offer_count_delta: i8,
        free_rerolls: u8,
        guaranteed_rarity: Option<CurioBlessingRarity>,
        enhance_all_one_star: bool,
        enhance_random_count: u8,
    },
    EnhanceRandomBlessings {
        count: u8,
    },
    DestroyAfterTriggers {
        triggers: u8,
    },
    GrantCosmicFragments {
        amount: u32,
    },
    GrantFragmentsPerFullHpAlly {
        amount_per_ally: u32,
        allies: u8,
    },
    ConfigureFragmentGain {
        ratio: PathEffectValue,
        battle_rewards_only: bool,
    },
    GrantFragmentsFromCurrent {
        ratio: PathEffectValue,
    },
    DestroyAboveFragmentsAndLoseAll {
        threshold: u32,
    },
    LoseFragmentsAndAddCriticalDamage {
        fragments: u32,
        fragments_per_stack: u32,
        critical_damage_ratio_per_stack: PathEffectValue,
    },
    SuppressPostCombatBlessing,
    TechniqueDamageModifier {
        damage_ratio: PathEffectValue,
        maximum_hp_ratio: PathEffectValue,
    },
    DamageEnemiesMaximumHpRatio {
        ratio: PathEffectValue,
    },
    ConsumeHighestAttackHpAndGainSpeed {
        hp_ratio: PathEffectValue,
        speed_ratio: PathEffectValue,
        maximum_stacks: u8,
    },
    ReduceRunServiceCost {
        ratio: PathEffectValue,
    },
    RevivePartyAndRestoreFullHp,
    TreatNonFinalDefeatAsVictoryAndRestoreFullHp,
    IncreaseEidolonResonance {
        levels: u8,
    },
    ConfigureDestructibleLottery {
        reward: CurioDestructibleReward,
        released_small_chance: bool,
        failure_current_hp_loss_ratio: PathEffectValue,
        failure_loses_energy_and_technique_points: bool,
    },
    ConfigureDestructibles {
        more_frequent: bool,
        reward_multiplier: u8,
    },
    AddBattlefieldWeaknesses {
        fixed_chance: PathEffectValue,
        weakness_count: u8,
        duration_turns: u8,
    },
    RepairRandomDestroyedCurios {
        maximum: u8,
        restore_default_charges: bool,
    },
    ReplaceAllOwnedCuriosRandomly {
        include_source: bool,
    },
    ReplaceAllBlessingsRandomly {
        retain_enhancement: bool,
        released_higher_rarity_chance: bool,
    },
    ChangeActorEnergy {
        change: CurioEnergyChange,
    },
    ChangeActorCurrentHpRatio {
        change: CurioHpChange,
        ratio: PathEffectValue,
        can_defeat: bool,
    },
    BattleStatWhileCurrentHpBelow {
        target: PathEffectTarget,
        stat: PathEffectStat,
        value: PathEffectValue,
        threshold: PathEffectValue,
    },
    ModifySkillPoints {
        delta: i8,
    },
    ConfigureParasitized {
        attack_ratio: PathEffectValue,
        turn_current_hp_cost_ratio: PathEffectValue,
        transfer_to_random_ally_when_downed: bool,
    },
    SuppressBattleFragmentsThenDoubleCurrent {
        triggers: u8,
    },
    ApplyReleasedMajorAggro {
        target: PathEffectTarget,
        target_count: u8,
        duration_turns: u8,
    },
    LoseCosmicFragmentsRatio {
        ratio: PathEffectValue,
    },
    IncreaseBlessingServiceCost {
        ratio: PathEffectValue,
        affects_enhance: bool,
        affects_reset: bool,
    },
    ConsumeSkillPoints {
        amount: u8,
    },
    ConfigureCurioFission {
        released_chance: bool,
        maximum_concurrent_copies: u8,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppliedCurioEffect {
    source_key: Box<str>,
    effect: CurioEffect,
}

impl AppliedCurioEffect {
    pub(crate) fn new(source_key: &str, effect: CurioEffect) -> Self {
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
    pub const fn effect(&self) -> &CurioEffect {
        &self.effect
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
enum CurioTemplate {
    SealingWax = 0,
    DestroyedCurioDamage = 1,
    FullHpFragments = 2,
    FragmentGainNoBlessing = 3,
    TechniqueDamage = 4,
    DomainFragmentsThreshold = 5,
    SpendFragmentsCritical = 6,
    TurnHealing = 7,
    SelectedPathBlessings = 8,
    ToxiFlame = 9,
    DifferentPathBreakEffect = 10,
    ExtraBlessingSelection = 11,
    EnhanceBlessings = 12,
    FreeBlessingReroll = 13,
    EnhanceOfferedOneStar = 14,
    BattleRevival = 15,
    RandomBlessings = 16,
    PostBattleHealing = 17,
    GuaranteedThreeStar = 18,
    EntryEnemyDamage = 19,
    ResonanceRobe = 20,
    FragmentDamage = 21,
    BattleFragmentGain = 22,
    ServiceDiscount = 23,
    DomainFragmentInterest = 24,
    ImmediateFragmentInterest = 25,
    DestructibleDamage = 26,
    EntryProtection = 27,
    DefeatCrown = 28,
    EidolonPrism = 29,
    CurioLotto = 30,
    BlessingLotto = 31,
    DestructibleCapsule = 32,
    WeaknessImplant = 33,
    EnhanceOfferedRandom = 34,
}

impl CurioTemplate {
    const fn event(self) -> CurioEvent {
        match self {
            Self::SealingWax
            | Self::EnhanceBlessings
            | Self::RandomBlessings
            | Self::SelectedPathBlessings
            | Self::SpendFragmentsCritical
            | Self::ImmediateFragmentInterest => CurioEvent::Acquired,
            Self::FullHpFragments
            | Self::BattleRevival
            | Self::PostBattleHealing
            | Self::BattleFragmentGain => CurioEvent::BattleWon,
            Self::ExtraBlessingSelection
            | Self::FreeBlessingReroll
            | Self::EnhanceOfferedOneStar
            | Self::GuaranteedThreeStar
            | Self::EnhanceOfferedRandom => CurioEvent::BlessingRewardOffered,
            Self::DomainFragmentsThreshold | Self::DomainFragmentInterest => {
                CurioEvent::DomainEntered
            }
            Self::EntryEnemyDamage
            | Self::ResonanceRobe
            | Self::EntryProtection
            | Self::EidolonPrism
            | Self::WeaknessImplant => CurioEvent::BattleStarted,
            Self::TurnHealing | Self::ToxiFlame => CurioEvent::CharacterTurnStarted,
            Self::CurioLotto | Self::BlessingLotto | Self::DestructibleCapsule => {
                CurioEvent::DestructibleDestroyed
            }
            Self::TechniqueDamage => CurioEvent::TechniqueDamageCalculated,
            Self::DestroyedCurioDamage
            | Self::FragmentGainNoBlessing
            | Self::DifferentPathBreakEffect
            | Self::FragmentDamage
            | Self::ServiceDiscount
            | Self::DestructibleDamage => CurioEvent::StatQueried,
            Self::DefeatCrown => CurioEvent::RunDefeated,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CompiledCurioProgram {
    curio: CurioId,
    source_key: Box<str>,
    template: CurioTemplate,
    path: Option<PathId>,
    parameters: Box<[PathEffectValue]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurioEffectRuntimeCatalog {
    programs: Box<[CompiledCurioProgram]>,
    digest: [u8; 32],
}

impl CurioEffectRuntimeCatalog {
    pub fn compile(
        catalog: &UniverseCatalog,
        runtime: &CurioRuntimeCatalog,
    ) -> Result<Self, CurioEffectRuntimeError> {
        let mut programs = Vec::new();
        for definition in runtime.definitions() {
            let state = definition
                .states()
                .first()
                .ok_or(CurioEffectRuntimeError::InvalidDefinition)?;
            let Some((template, arity, path_key)) = registry(state.source_effect_id()) else {
                continue;
            };
            if definition.states().len() != 1 || state.parameters().len() != arity {
                return Err(CurioEffectRuntimeError::InvalidDefinition);
            }
            let path = path_key
                .map(|key| {
                    catalog
                        .paths()
                        .iter()
                        .find(|path| path.stable_key() == key)
                        .map(|path| path.id())
                        .ok_or(CurioEffectRuntimeError::InvalidDefinition)
                })
                .transpose()?;
            programs.push(CompiledCurioProgram {
                curio: definition.curio(),
                source_key: format!("{}.state.active", definition.stable_key()).into(),
                template,
                path,
                parameters: exact_parameters(state.parameters())
                    .map_err(|_| CurioEffectRuntimeError::InvalidParameter)?,
            });
        }
        programs.sort_by_key(|program| program.curio);
        if programs.len() != 43
            || programs
                .windows(2)
                .any(|pair| pair[0].curio == pair[1].curio)
        {
            return Err(CurioEffectRuntimeError::InvalidDenominator);
        }
        let digest = catalog_digest(&programs);
        Ok(Self {
            programs: programs.into_boxed_slice(),
            digest,
        })
    }

    #[must_use]
    pub const fn content_count(&self) -> usize {
        86
    }
    #[must_use]
    pub const fn rule_count(&self) -> usize {
        86
    }
    #[must_use]
    pub const fn digest(&self) -> [u8; 32] {
        self.digest
    }
    #[must_use]
    pub fn curio_ids(&self) -> impl ExactSizeIterator<Item = CurioId> + '_ {
        self.programs.iter().map(|program| program.curio)
    }

    pub fn execute(
        &self,
        curio: CurioId,
        event: CurioEvent,
        facts: CurioEffectFacts,
    ) -> Result<Box<[AppliedCurioEffect]>, CurioEffectRuntimeError> {
        let program = self
            .programs
            .binary_search_by_key(&curio, |program| program.curio)
            .ok()
            .map(|index| &self.programs[index])
            .ok_or(CurioEffectRuntimeError::UnknownCurio)?;
        execute(program, event, facts.validate()?)
    }
}

fn execute(
    program: &CompiledCurioProgram,
    event: CurioEvent,
    facts: CurioEffectFacts,
) -> Result<Box<[AppliedCurioEffect]>, CurioEffectRuntimeError> {
    if program.template.event() != event {
        return Ok(Box::new([]));
    }
    let p = &program.parameters;
    let mut effects = Vec::with_capacity(3);
    use CurioTemplate as T;
    match program.template {
        T::SealingWax => {
            let path = program
                .path
                .ok_or(CurioEffectRuntimeError::InvalidDefinition)?;
            effects.push(CurioEffect::GrantRandomBlessings {
                path: Some(path),
                minimum: 1,
                maximum: 1,
            });
            effects.push(CurioEffect::BiasBlessingOffers { path });
        }
        T::DestroyedCurioDamage => effects.push(battle_stat(
            PathEffectStat::DamageRatio,
            p[0].checked_multiply_count(facts.destroyed_curios)?,
        )),
        T::FullHpFragments => effects.push(CurioEffect::GrantFragmentsPerFullHpAlly {
            amount_per_ally: integral(p[0])?,
            allies: u8_count(facts.full_hp_allies)?,
        }),
        T::FragmentGainNoBlessing => {
            effects.push(CurioEffect::ConfigureFragmentGain {
                ratio: p[0],
                battle_rewards_only: false,
            });
            effects.push(CurioEffect::SuppressPostCombatBlessing);
        }
        T::TechniqueDamage => effects.push(CurioEffect::TechniqueDamageModifier {
            damage_ratio: p[0],
            maximum_hp_ratio: p[1],
        }),
        T::DomainFragmentsThreshold => {
            effects.push(CurioEffect::GrantCosmicFragments {
                amount: integral(p[0])?,
            });
            effects.push(CurioEffect::DestroyAboveFragmentsAndLoseAll {
                threshold: integral(p[1])?,
            });
        }
        T::SpendFragmentsCritical => {
            effects.push(CurioEffect::LoseFragmentsAndAddCriticalDamage {
                fragments: facts.cosmic_fragments,
                fragments_per_stack: 100,
                critical_damage_ratio_per_stack: p[0],
            });
        }
        T::TurnHealing => effects.push(CurioEffect::Battle(PathEffect::HealMaximumHpRatio {
            target: PathEffectTarget::Actor,
            ratio: p[0],
        })),
        T::SelectedPathBlessings => effects.push(CurioEffect::GrantRandomBlessings {
            path: None,
            minimum: turns(p[0])?,
            maximum: turns(p[1])?,
        }),
        T::ToxiFlame => {
            effects.push(CurioEffect::ConsumeHighestAttackHpAndGainSpeed {
                hp_ratio: p[0],
                speed_ratio: p[1],
                maximum_stacks: turns(p[2])?,
            });
        }
        T::DifferentPathBreakEffect => effects.push(battle_stat(
            PathEffectStat::BreakEffectRatio,
            p[0].checked_multiply_count(facts.different_path_blessings)?,
        )),
        T::ExtraBlessingSelection => {
            effects.push(blessing_reward(
                turns(p[0])?,
                -signed(p[1])?,
                0,
                None,
                false,
                0,
            ));
            effects.push(CurioEffect::DestroyAfterTriggers {
                triggers: turns(p[2])?,
            });
        }
        T::EnhanceBlessings => effects.push(CurioEffect::EnhanceRandomBlessings {
            count: turns(p[0])?,
        }),
        T::FreeBlessingReroll => effects.push(blessing_reward(0, 0, turns(p[0])?, None, false, 0)),
        T::EnhanceOfferedOneStar => {
            effects.push(blessing_reward(0, 0, 0, None, true, 0));
        }
        T::BattleRevival => {
            effects.push(CurioEffect::RevivePartyAndRestoreFullHp);
            effects.push(CurioEffect::DestroyAfterTriggers {
                triggers: turns(p[0])?,
            });
        }
        T::RandomBlessings => effects.push(CurioEffect::GrantRandomBlessings {
            path: None,
            minimum: turns(p[0])?,
            maximum: turns(p[1])?,
        }),
        T::PostBattleHealing => effects.push(CurioEffect::Battle(PathEffect::HealMaximumHpRatio {
            target: PathEffectTarget::AllAllies,
            ratio: p[0],
        })),
        T::GuaranteedThreeStar => {
            effects.push(blessing_reward(
                0,
                0,
                0,
                Some(CurioBlessingRarity::ThreeStar),
                false,
                0,
            ));
            effects.push(CurioEffect::DestroyAfterTriggers {
                triggers: turns(p[0])?,
            });
        }
        T::EntryEnemyDamage => {
            effects.push(CurioEffect::DamageEnemiesMaximumHpRatio { ratio: p[0] });
        }
        T::ResonanceRobe => {
            effects.push(CurioEffect::Battle(PathEffect::ConfigureResonanceEnergy {
                maximum: PathEffectValue::from_integral(100)?,
                gain_on_ally_turn_ratio: PathEffectValue::ZERO,
            }));
            effects.push(battle_stat(PathEffectStat::PathDamageRatio, p[0]));
        }
        T::FragmentDamage => effects.push(battle_stat(
            PathEffectStat::DamageRatio,
            p[1].checked_multiply_count(facts.cosmic_fragments / integral(p[0])?)?,
        )),
        T::BattleFragmentGain => effects.push(CurioEffect::ConfigureFragmentGain {
            ratio: p[0],
            battle_rewards_only: true,
        }),
        T::ServiceDiscount => effects.push(CurioEffect::ReduceRunServiceCost { ratio: p[0] }),
        T::DomainFragmentInterest | T::ImmediateFragmentInterest => {
            effects.push(CurioEffect::GrantFragmentsFromCurrent { ratio: p[0] });
        }
        T::DestructibleDamage => effects.push(battle_stat(
            PathEffectStat::DamageRatio,
            p[0].checked_multiply_count(facts.destructibles_destroyed)?,
        )),
        T::EntryProtection => {
            effects.push(CurioEffect::Battle(PathEffect::ApplyUntilAttackedStat {
                target: PathEffectTarget::AllAllies,
                stat: PathEffectStat::DamageTakenReductionRatio,
                value: PathEffectValue::ONE,
            }));
            effects.push(CurioEffect::Battle(PathEffect::ApplyTimedStat {
                target: PathEffectTarget::AllAllies,
                stat: PathEffectStat::EffectResistanceRatio,
                value: PathEffectValue::ONE,
                duration_turns: turns(p[1])?,
                maximum_stacks: 1,
            }));
        }
        T::DefeatCrown => {
            if !facts.final_domain {
                effects.push(CurioEffect::TreatNonFinalDefeatAsVictoryAndRestoreFullHp);
                effects.push(CurioEffect::DestroyAfterTriggers {
                    triggers: turns(p[0])?,
                });
            }
        }
        T::EidolonPrism => effects.push(CurioEffect::IncreaseEidolonResonance {
            levels: turns(p[0])?,
        }),
        T::CurioLotto => effects.push(lottery(CurioDestructibleReward::Curio, p[0], false)),
        T::BlessingLotto => effects.push(lottery(
            CurioDestructibleReward::Blessing,
            PathEffectValue::ZERO,
            true,
        )),
        T::DestructibleCapsule => effects.push(CurioEffect::ConfigureDestructibles {
            more_frequent: true,
            reward_multiplier: 2,
        }),
        T::WeaknessImplant => effects.push(CurioEffect::AddBattlefieldWeaknesses {
            fixed_chance: p[0],
            weakness_count: turns(p[1])?,
            duration_turns: turns(p[2])?,
        }),
        T::EnhanceOfferedRandom => {
            effects.push(blessing_reward(0, 0, 0, None, false, turns(p[0])?))
        }
    }
    Ok(effects
        .into_iter()
        .map(|effect| AppliedCurioEffect::new(&program.source_key, effect))
        .collect::<Vec<_>>()
        .into_boxed_slice())
}

fn battle_stat(stat: PathEffectStat, value: PathEffectValue) -> CurioEffect {
    CurioEffect::Battle(PathEffect::AddStat {
        target: PathEffectTarget::AllAllies,
        stat,
        value,
        cap: None,
    })
}

fn blessing_reward(
    extra_selections: u8,
    offer_count_delta: i8,
    free_rerolls: u8,
    guaranteed_rarity: Option<CurioBlessingRarity>,
    enhance_all_one_star: bool,
    enhance_random_count: u8,
) -> CurioEffect {
    CurioEffect::ConfigureBlessingReward {
        extra_selections,
        offer_count_delta,
        free_rerolls,
        guaranteed_rarity,
        enhance_all_one_star,
        enhance_random_count,
    }
}

fn lottery(
    reward: CurioDestructibleReward,
    loss_ratio: PathEffectValue,
    loses_resources: bool,
) -> CurioEffect {
    CurioEffect::ConfigureDestructibleLottery {
        reward,
        released_small_chance: true,
        failure_current_hp_loss_ratio: loss_ratio,
        failure_loses_energy_and_technique_points: loses_resources,
    }
}

fn integral(value: PathEffectValue) -> Result<u32, CurioEffectRuntimeError> {
    let raw = value.raw_six_decimal();
    if raw < 0 || raw % 1_000_000 != 0 {
        return Err(CurioEffectRuntimeError::InvalidParameter);
    }
    u32::try_from(raw / 1_000_000).map_err(|_| CurioEffectRuntimeError::InvalidParameter)
}

fn signed(value: PathEffectValue) -> Result<i8, CurioEffectRuntimeError> {
    i8::try_from(integral(value)?).map_err(|_| CurioEffectRuntimeError::InvalidParameter)
}

fn u8_count(value: u32) -> Result<u8, CurioEffectRuntimeError> {
    u8::try_from(value).map_err(|_| CurioEffectRuntimeError::InvalidFacts)
}

fn registry(effect: &str) -> Option<(CurioTemplate, usize, Option<&'static str>)> {
    use CurioTemplate as T;
    let path = |template, arity, key| Some((template, arity, Some(key)));
    match effect {
        "73" => path(T::SealingWax, 0, "universe.path.erudition"),
        "22" => path(T::SealingWax, 1, "universe.path.preservation"),
        "23" => path(T::SealingWax, 1, "universe.path.elation"),
        "24" => path(T::SealingWax, 1, "universe.path.hunt"),
        "25" => path(T::SealingWax, 1, "universe.path.destruction"),
        "26" => path(T::SealingWax, 1, "universe.path.remembrance"),
        "27" => path(T::SealingWax, 1, "universe.path.nihility"),
        "28" => path(T::SealingWax, 1, "universe.path.abundance"),
        "72" => path(T::SealingWax, 0, "universe.path.propagation"),
        "75" => Some((T::DestroyedCurioDamage, 1, None)),
        "76" => Some((T::FullHpFragments, 1, None)),
        "82" => Some((T::FragmentGainNoBlessing, 1, None)),
        "83" => Some((T::TechniqueDamage, 2, None)),
        "84" => Some((T::DomainFragmentsThreshold, 2, None)),
        "85" => Some((T::SpendFragmentsCritical, 1, None)),
        "87" => Some((T::TurnHealing, 1, None)),
        "88" => Some((T::SelectedPathBlessings, 2, None)),
        "89" => Some((T::ToxiFlame, 3, None)),
        "90" => Some((T::DifferentPathBreakEffect, 1, None)),
        "1" => Some((T::ExtraBlessingSelection, 3, None)),
        "20" => Some((T::EnhanceBlessings, 1, None)),
        "2" => Some((T::FreeBlessingReroll, 1, None)),
        "3" => Some((T::EnhanceOfferedOneStar, 1, None)),
        "4" => Some((T::BattleRevival, 1, None)),
        "5" => Some((T::RandomBlessings, 2, None)),
        "6" => Some((T::PostBattleHealing, 2, None)),
        "7" => Some((T::GuaranteedThreeStar, 1, None)),
        "8" => Some((T::EntryEnemyDamage, 2, None)),
        "11" => Some((T::ResonanceRobe, 2, None)),
        "14" => Some((T::FragmentDamage, 3, None)),
        "12" => Some((T::BattleFragmentGain, 1, None)),
        "13" => Some((T::ServiceDiscount, 1, None)),
        "15" => Some((T::DomainFragmentInterest, 1, None)),
        "74" => Some((T::ImmediateFragmentInterest, 1, None)),
        "58" => Some((T::DestructibleDamage, 1, None)),
        "19" => Some((T::EntryProtection, 3, None)),
        "61" => Some((T::DefeatCrown, 1, None)),
        "62" => Some((T::EidolonPrism, 1, None)),
        "63" => Some((T::CurioLotto, 1, None)),
        "77" => Some((T::BlessingLotto, 1, None)),
        "64" => Some((T::DestructibleCapsule, 1, None)),
        "68" => Some((T::WeaknessImplant, 3, None)),
        "69" => Some((T::EnhanceOfferedRandom, 1, None)),
        _ => None,
    }
}

fn catalog_digest(programs: &[CompiledCurioProgram]) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock-universe-curio-effect-runtime-catalog-v1");
    encoder.text(CURIO_EFFECT_RUNTIME_REVISION);
    encoder.u32(programs.len() as u32);
    for program in programs {
        encoder.u32(program.curio.get());
        encoder.text(&program.source_key);
        encoder.u8(program.template as u8);
        encoder.u32(program.path.map_or(0, PathId::get));
        encoder.u32(program.parameters.len() as u32);
        for parameter in &program.parameters {
            encoder.i64(parameter.raw_six_decimal());
        }
    }
    encoder.finish()
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CurioEffectRuntimeError {
    InvalidDefinition,
    InvalidDenominator,
    InvalidParameter,
    InvalidFacts,
    UnknownCurio,
    Overflow,
}

impl From<crate::path_effect_runtime::PathEffectRuntimeError> for CurioEffectRuntimeError {
    fn from(value: crate::path_effect_runtime::PathEffectRuntimeError) -> Self {
        match value {
            crate::path_effect_runtime::PathEffectRuntimeError::Overflow => Self::Overflow,
            _ => Self::InvalidParameter,
        }
    }
}
