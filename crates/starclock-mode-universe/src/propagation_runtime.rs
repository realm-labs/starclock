//! Closed executor for the released Standard Universe Propagation partition.

use crate::{
    catalog::UniverseCatalog,
    digest::Encoder,
    id::{BlessingId, PathId, ResonanceId},
    path_effect_runtime::{
        AppliedPathEffect, PathBattleEvent, PathEffect, PathEffectFacts, PathEffectRuntimeError,
        PathEffectStat, PathEffectTarget, PathEffectValue, count, exact_parameters, turns,
    },
};

pub const PROPAGATION_RUNTIME_REVISION: &str = "standard-universe-propagation-runtime-v1";
const PROPAGATION_PATH_KEY: &str = "universe.path.propagation";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
enum PropagationTemplate {
    SporesOnSpend = 0,
    SporesOnRecovery = 1,
    UltimateSkillPointAccounting = 2,
    SporePropagation = 3,
    SporeBurstDamage = 4,
    SporeBurstSustain = 5,
    EmptyBasicSkillPoint = 6,
    BasicAttackSplash = 7,
    SpendDamageReduction = 8,
    NonAttackSkillDamage = 9,
    BlessingBasicDamage = 10,
    BasicCriticalRate = 11,
    BasicCriticalDamage = 12,
    BasicDefense = 13,
    BasicSpeed = 14,
    EntrySkillPoints = 15,
    SpendEnergy = 16,
    SpendHealing = 17,
    ResonanceMetamorphosis = 18,
    ResonanceProboscis = 19,
    ResonancePhenol = 20,
    ResonanceCrystalPincers = 21,
}

impl PropagationTemplate {
    const fn event(self) -> PathBattleEvent {
        match self {
            Self::SporesOnSpend
            | Self::SpendDamageReduction
            | Self::SpendEnergy
            | Self::SpendHealing => PathBattleEvent::SkillPointConsumed,
            Self::SporesOnRecovery => PathBattleEvent::SkillPointRecovered,
            Self::UltimateSkillPointAccounting => PathBattleEvent::UltimateUsed,
            Self::SporePropagation | Self::SporeBurstDamage | Self::SporeBurstSustain => {
                PathBattleEvent::SporeBurst
            }
            Self::EmptyBasicSkillPoint | Self::BasicDefense | Self::BasicSpeed => {
                PathBattleEvent::BasicAttackUsed
            }
            Self::BasicAttackSplash => PathBattleEvent::BasicAttackDamageDealt,
            Self::NonAttackSkillDamage => PathBattleEvent::NonAttackSkillUsed,
            Self::BlessingBasicDamage | Self::BasicCriticalRate | Self::BasicCriticalDamage => {
                PathBattleEvent::StatQueried
            }
            Self::EntrySkillPoints
            | Self::ResonanceProboscis
            | Self::ResonancePhenol
            | Self::ResonanceCrystalPincers => PathBattleEvent::BattleStarted,
            Self::ResonanceMetamorphosis => PathBattleEvent::PathResonanceActivated,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CompiledProgram {
    source_key: Box<str>,
    template: PropagationTemplate,
    parameters: Box<[PathEffectValue]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct BlessingPrograms {
    blessing: BlessingId,
    levels: [CompiledProgram; 2],
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ResonanceProgram {
    resonance: ResonanceId,
    program: CompiledProgram,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PropagationRuntimeCatalog {
    path: PathId,
    blessings: Box<[BlessingPrograms]>,
    resonances: Box<[ResonanceProgram]>,
    digest: [u8; 32],
}

impl PropagationRuntimeCatalog {
    pub fn compile(catalog: &UniverseCatalog) -> Result<Self, PathEffectRuntimeError> {
        let path = catalog
            .paths()
            .iter()
            .find(|path| path.stable_key() == PROPAGATION_PATH_KEY)
            .ok_or(PathEffectRuntimeError::InvalidDefinition)?;
        if path.blessings().len() != 18 || path.formations().len() != 3 {
            return Err(PathEffectRuntimeError::InvalidDefinition);
        }
        let mut blessings = Vec::with_capacity(18);
        for blessing_id in path.blessings() {
            let blessing = catalog
                .blessing(*blessing_id)
                .ok_or(PathEffectRuntimeError::InvalidDefinition)?;
            let mut levels = blessing
                .levels()
                .iter()
                .map(|level_id| {
                    let level = catalog
                        .blessing_level(*level_id)
                        .ok_or(PathEffectRuntimeError::InvalidDefinition)?;
                    let (template, arity) = registry(level.source_binding_key())?;
                    if level.parameters().len() != arity || level.blessing() != blessing.id() {
                        return Err(PathEffectRuntimeError::InvalidDefinition);
                    }
                    Ok((
                        level.level(),
                        CompiledProgram {
                            source_key: level.stable_key().into(),
                            template,
                            parameters: exact_parameters(level.parameters())?,
                        },
                    ))
                })
                .collect::<Result<Vec<_>, PathEffectRuntimeError>>()?;
            levels.sort_by_key(|(level, _)| *level);
            let [(_, first), (_, second)] = levels
                .try_into()
                .map_err(|_| PathEffectRuntimeError::InvalidDefinition)?;
            blessings.push(BlessingPrograms {
                blessing: blessing.id(),
                levels: [first, second],
            });
        }
        blessings.sort_by_key(|entry| entry.blessing);
        let mut resonance_ids = Vec::from(path.formations());
        resonance_ids.push(path.resonance());
        let mut resonances = Vec::with_capacity(4);
        for resonance_id in resonance_ids {
            let resonance = catalog
                .resonance(resonance_id)
                .ok_or(PathEffectRuntimeError::InvalidDefinition)?;
            let (template, arity) = registry(resonance.source_binding_key())?;
            if resonance.path() != path.id() || resonance.parameters().len() != arity {
                return Err(PathEffectRuntimeError::InvalidDefinition);
            }
            resonances.push(ResonanceProgram {
                resonance: resonance.id(),
                program: CompiledProgram {
                    source_key: resonance.stable_key().into(),
                    template,
                    parameters: exact_parameters(resonance.parameters())?,
                },
            });
        }
        resonances.sort_by_key(|entry| entry.resonance);
        let digest = catalog_digest(path.id(), &blessings, &resonances);
        Ok(Self {
            path: path.id(),
            blessings: blessings.into_boxed_slice(),
            resonances: resonances.into_boxed_slice(),
            digest,
        })
    }

    #[must_use]
    pub const fn path(&self) -> PathId {
        self.path
    }
    #[must_use]
    pub const fn digest(&self) -> [u8; 32] {
        self.digest
    }
    #[must_use]
    pub const fn content_count(&self) -> usize {
        59
    }
    #[must_use]
    pub const fn rule_count(&self) -> usize {
        58
    }
    #[must_use]
    pub fn blessing_ids(&self) -> impl ExactSizeIterator<Item = BlessingId> + '_ {
        self.blessings.iter().map(|entry| entry.blessing)
    }
    #[must_use]
    pub fn resonance_ids(&self) -> impl ExactSizeIterator<Item = ResonanceId> + '_ {
        self.resonances.iter().map(|entry| entry.resonance)
    }

    pub fn execute_blessing(
        &self,
        blessing: BlessingId,
        level: u8,
        event: PathBattleEvent,
        facts: PathEffectFacts,
    ) -> Result<Box<[AppliedPathEffect]>, PathEffectRuntimeError> {
        let entry = self
            .blessings
            .binary_search_by_key(&blessing, |entry| entry.blessing)
            .ok()
            .map(|index| &self.blessings[index])
            .ok_or(PathEffectRuntimeError::UnknownSource)?;
        let program = entry
            .levels
            .get(usize::from(
                level
                    .checked_sub(1)
                    .ok_or(PathEffectRuntimeError::InvalidParameter)?,
            ))
            .ok_or(PathEffectRuntimeError::InvalidParameter)?;
        execute(program, event, facts)
    }

    pub fn execute_resonance(
        &self,
        resonance: ResonanceId,
        event: PathBattleEvent,
        facts: PathEffectFacts,
    ) -> Result<Box<[AppliedPathEffect]>, PathEffectRuntimeError> {
        let program = self
            .resonances
            .binary_search_by_key(&resonance, |entry| entry.resonance)
            .ok()
            .map(|index| &self.resonances[index].program)
            .ok_or(PathEffectRuntimeError::UnknownSource)?;
        execute(program, event, facts)
    }
}

fn execute(
    program: &CompiledProgram,
    event: PathBattleEvent,
    facts: PathEffectFacts,
) -> Result<Box<[AppliedPathEffect]>, PathEffectRuntimeError> {
    let facts = facts.validate()?;
    if program.template.event() != event {
        return Ok(Box::new([]));
    }
    let p = &program.parameters;
    let mut effects = Vec::with_capacity(2);
    match program.template {
        PropagationTemplate::SporesOnSpend => effects.push(PathEffect::ApplySpores {
            target: PathEffectTarget::AllEnemies,
            stacks_per_skill_point: turns(p[0])?,
            random_target_count: 0,
            maximum_stacks: None,
        }),
        PropagationTemplate::SporesOnRecovery => effects.push(PathEffect::ApplySpores {
            target: PathEffectTarget::RandomEnemy,
            stacks_per_skill_point: 1,
            random_target_count: turns(p[0])?,
            maximum_stacks: (p[2] != PathEffectValue::ONE)
                .then(|| turns(p[1]))
                .transpose()?,
        }),
        PropagationTemplate::UltimateSkillPointAccounting => {
            effects.push(PathEffect::ConfigureNextSkillPointAccounting {
                after_ultimate: true,
                includes_recovery: p[2] != PathEffectValue::ONE,
                extra_points: 1,
                critical_damage_ratio_per_point: p[0],
                maximum_stacks: turns(p[1])?,
                expires_after_attack: true,
            });
        }
        PropagationTemplate::SporePropagation => {
            effects.push(PathEffect::ConfigureSporePropagation {
                spread_count: turns(p[0])?,
                may_return_to_original: true,
            });
        }
        PropagationTemplate::SporeBurstDamage => effects.push(PathEffect::ModifySporeBurst {
            damage_bonus_ratio: p[0],
            spread_on_defeat: if p[1] == PathEffectValue::ONE {
                PathEffectTarget::AdjacentEnemies
            } else {
                PathEffectTarget::OtherEnemies
            },
        }),
        PropagationTemplate::SporeBurstSustain => {
            effects.push(PathEffect::HealPerSporeBurst {
                target: PathEffectTarget::LowestHpAlly,
                maximum_hp_ratio_per_spore: p[0],
                spore_count: u8_count(facts.spores_burst)?,
            });
            if p[2] != PathEffectValue::ONE {
                effects.push(PathEffect::AddPartyDamageReductionPerSpore {
                    value_per_spore: p[1],
                    spore_count: facts.all_enemy_spore_count,
                });
            }
        }
        PropagationTemplate::EmptyBasicSkillPoint => {
            if facts.skill_points_available == 0 {
                effects.push(PathEffect::ConditionalBasicAttackSkillPoint {
                    required_available_points: 0,
                    guaranteed_amount: 1,
                    extra_amount: u8::from(p[0] != PathEffectValue::ZERO),
                    extra_fixed_chance: p[0],
                });
            }
        }
        PropagationTemplate::BasicAttackSplash => effects.push(PathEffect::BasicAttackSplash {
            target: if p[1] == PathEffectValue::ONE {
                PathEffectTarget::RandomOtherEnemies
            } else {
                PathEffectTarget::AdjacentEnemies
            },
            original_damage_ratio: p[0],
        }),
        PropagationTemplate::SpendDamageReduction => {
            effects.push(PathEffect::ApplySkillPointConsumedStat {
                stat: PathEffectStat::DamageTakenReductionRatio,
                value_per_point: p[0],
                points: u8_count(facts.skill_points_consumed)?,
                duration_turns: turns(p[1])?,
                maximum_stacks: turns(p[2])?,
            });
        }
        PropagationTemplate::NonAttackSkillDamage => {
            effects.push(PathEffect::ApplyNonAttackSkillTeamDamage {
                value_per_stack: p[0],
                duration_turns: turns(p[1])?,
                maximum_stacks: turns(p[2])?,
            });
        }
        PropagationTemplate::BlessingBasicDamage => effects.push(basic_modifier(
            PathEffectStat::DamageRatio,
            p[0].checked_multiply_count(facts.path_blessing_count.min(count(p[1])?))?,
        )),
        PropagationTemplate::BasicCriticalRate => {
            effects.push(basic_modifier(PathEffectStat::CriticalRateRatio, p[0]));
        }
        PropagationTemplate::BasicCriticalDamage => {
            effects.push(basic_modifier(PathEffectStat::CriticalDamageRatio, p[0]));
        }
        PropagationTemplate::BasicDefense => effects.push(PathEffect::ApplyTimedStat {
            target: PathEffectTarget::Actor,
            stat: PathEffectStat::DefenseRatio,
            value: p[0],
            duration_turns: turns(p[1])?,
            maximum_stacks: 1,
        }),
        PropagationTemplate::BasicSpeed => effects.push(PathEffect::ApplyTimedStat {
            target: PathEffectTarget::Actor,
            stat: PathEffectStat::SpeedRatio,
            value: p[0],
            duration_turns: turns(p[1])?,
            maximum_stacks: 1,
        }),
        PropagationTemplate::EntrySkillPoints => {
            effects.push(PathEffect::ConfigureEntrySkillPointRecovery {
                amount_after_ally_turn: 1,
                maximum_triggers_per_battle: turns(p[0])?,
            });
        }
        PropagationTemplate::SpendEnergy => effects.push(PathEffect::GainEnergy {
            target: PathEffectTarget::Actor,
            amount: p[0].checked_multiply_count(facts.skill_points_consumed)?,
            once_per_action: false,
        }),
        PropagationTemplate::SpendHealing => {
            let ratio = p[0].checked_multiply_count(facts.skill_points_consumed)?;
            effects.push(PathEffect::HealMaximumHpRatio {
                target: PathEffectTarget::Actor,
                ratio,
            });
        }
        PropagationTemplate::ResonanceMetamorphosis => {
            effects.push(PathEffect::ApplyMetamorphosis {
                target: PathEffectTarget::Actor,
                action_advance_ratio: PathEffectValue::ONE,
                skill_points: turns(p[0])?,
                duration_turns: 1,
            });
        }
        PropagationTemplate::ResonanceProboscis => {
            effects.push(PathEffect::ConfigureMetamorphosis {
                duration_bonus_turns: 1,
                defeated_enemy_energy_ratio: p[0],
            });
        }
        PropagationTemplate::ResonancePhenol => {
            effects.push(PathEffect::ConfigureSkillPointResonanceEnergy {
                maximum: PathEffectValue::from_integral(200)?,
                energy_ratio_per_consumed_or_recovered_point: p[0],
            });
        }
        PropagationTemplate::ResonanceCrystalPincers => {
            effects.push(PathEffect::ConfigureMetamorphosisSporeBurst {
                damage_ratio: p[0],
                basic_attack_ratio_per_spore: p[1],
                maximum_triggers_per_target: turns(p[2])?,
            });
        }
    }
    Ok(effects
        .into_iter()
        .map(|effect| AppliedPathEffect::new(&program.source_key, effect))
        .collect::<Vec<_>>()
        .into_boxed_slice())
}

fn basic_modifier(stat: PathEffectStat, value: PathEffectValue) -> PathEffect {
    PathEffect::AddBasicAttackModifier { stat, value }
}

fn u8_count(value: u32) -> Result<u8, PathEffectRuntimeError> {
    u8::try_from(value).map_err(|_| PathEffectRuntimeError::InvalidFacts)
}

fn registry(key: &str) -> Result<(PropagationTemplate, usize), PathEffectRuntimeError> {
    use PropagationTemplate as T;
    Ok(match key {
        "StageAbility_612730" => (T::SporesOnSpend, 4),
        "StageAbility_612731" => (T::SporesOnRecovery, 3),
        "StageAbility_612732" => (T::UltimateSkillPointAccounting, 3),
        "StageAbility_612740" => (T::SporePropagation, 2),
        "StageAbility_612741" => (T::SporeBurstDamage, 2),
        "StageAbility_612742" => (T::SporeBurstSustain, 3),
        "StageAbility_612743" => (T::EmptyBasicSkillPoint, 1),
        "StageAbility_612744" => (T::BasicAttackSplash, 2),
        "StageAbility_612745" => (T::SpendDamageReduction, 3),
        "StageAbility_612746" => (T::NonAttackSkillDamage, 3),
        "StageAbility_612750" => (T::BlessingBasicDamage, 2),
        "StageAbility_612751" => (T::BasicCriticalRate, 1),
        "StageAbility_612752" => (T::BasicCriticalDamage, 1),
        "StageAbility_612753" => (T::BasicDefense, 2),
        "StageAbility_612754" => (T::BasicSpeed, 2),
        "StageAbility_612755" => (T::EntrySkillPoints, 1),
        "StageAbility_612756" => (T::SpendEnergy, 1),
        "StageAbility_612757" => (T::SpendHealing, 1),
        "StageAbility_612720" => (T::ResonanceMetamorphosis, 3),
        "StageAbility_612721" => (T::ResonanceProboscis, 1),
        "StageAbility_612722" => (T::ResonancePhenol, 1),
        "StageAbility_612723" => (T::ResonanceCrystalPincers, 3),
        _ => return Err(PathEffectRuntimeError::UnknownSource),
    })
}

fn catalog_digest(
    path: PathId,
    blessings: &[BlessingPrograms],
    resonances: &[ResonanceProgram],
) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock-universe-propagation-runtime-catalog-v1");
    encoder.text(PROPAGATION_RUNTIME_REVISION);
    encoder.u32(path.get());
    encoder.u32(blessings.len() as u32);
    for blessing in blessings {
        encoder.u32(blessing.blessing.get());
        for program in &blessing.levels {
            encode_program(&mut encoder, program);
        }
    }
    encoder.u32(resonances.len() as u32);
    for resonance in resonances {
        encoder.u32(resonance.resonance.get());
        encode_program(&mut encoder, &resonance.program);
    }
    encoder.finish()
}

fn encode_program(encoder: &mut Encoder, program: &CompiledProgram) {
    encoder.text(&program.source_key);
    encoder.u8(program.template as u8);
    encoder.u32(program.parameters.len() as u32);
    for parameter in &program.parameters {
        encoder.i64(parameter.raw_six_decimal());
    }
}
