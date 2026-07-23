//! Closed executor for the released Standard Universe Remembrance partition.

use crate::{
    catalog::UniverseCatalog,
    digest::Encoder,
    id::{BlessingId, PathId, ResonanceId},
    path_effect_runtime::{
        AppliedPathEffect, PathBattleEvent, PathEffect, PathEffectDamageKind, PathEffectElement,
        PathEffectFacts, PathEffectRuntimeError, PathEffectStat, PathEffectTarget, PathEffectValue,
        count, exact_parameters, turns,
    },
};

pub const REMEMBRANCE_RUNTIME_REVISION: &str = "standard-universe-remembrance-runtime-v1";
const REMEMBRANCE_PATH_KEY: &str = "universe.path.remembrance";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
enum RemembranceTemplate {
    FrozenAttackDissociation = 0,
    BreakDissociation = 1,
    RepeatedAttackFreeze = 2,
    DetonateDissociation = 3,
    DissociationVulnerability = 4,
    RemovedDissociationFreeze = 5,
    IceDamageSplash = 6,
    DamageFreeze = 7,
    UltimateIceWeakness = 8,
    EntryFreeze = 9,
    BlessingFreezeResistance = 10,
    EffectHitRate = 11,
    HalfHpFreeze = 12,
    FrozenSkillDamage = 13,
    FrozenCriticalExposure = 14,
    FrozenDamageTaken = 15,
    FreezeEnergy = 16,
    FreezeShield = 17,
    ResonanceDamageFreeze = 18,
    ResonanceFreezeResistance = 19,
    ResonanceEonianRiver = 20,
    ResonanceEnergy = 21,
}

impl RemembranceTemplate {
    const fn event(self) -> PathBattleEvent {
        match self {
            Self::FrozenAttackDissociation
            | Self::RepeatedAttackFreeze
            | Self::DetonateDissociation
            | Self::HalfHpFreeze => PathBattleEvent::AttackHit,
            Self::BreakDissociation => PathBattleEvent::WeaknessBroken,
            Self::DissociationVulnerability | Self::FrozenSkillDamage | Self::FrozenDamageTaken => {
                PathBattleEvent::DamageCalculated
            }
            Self::RemovedDissociationFreeze => PathBattleEvent::DissociationRemoved,
            Self::IceDamageSplash => PathBattleEvent::IceDamageDealt,
            Self::DamageFreeze => PathBattleEvent::DamageDealt,
            Self::UltimateIceWeakness => PathBattleEvent::UltimateUsed,
            Self::EntryFreeze | Self::BlessingFreezeResistance | Self::EffectHitRate => {
                PathBattleEvent::BattleStarted
            }
            Self::FrozenCriticalExposure | Self::FreezeEnergy | Self::FreezeShield => {
                PathBattleEvent::EnemyFrozen
            }
            Self::ResonanceDamageFreeze
            | Self::ResonanceFreezeResistance
            | Self::ResonanceEonianRiver => PathBattleEvent::PathResonanceActivated,
            Self::ResonanceEnergy => PathBattleEvent::BattleStarted,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CompiledProgram {
    source_key: Box<str>,
    template: RemembranceTemplate,
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

/// Immutable executor for 18 Blessings, their enhanced levels, and four Path actions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RemembranceRuntimeCatalog {
    path: PathId,
    blessings: Box<[BlessingPrograms]>,
    resonances: Box<[ResonanceProgram]>,
    digest: [u8; 32],
}

impl RemembranceRuntimeCatalog {
    pub fn compile(catalog: &UniverseCatalog) -> Result<Self, PathEffectRuntimeError> {
        let path = catalog
            .paths()
            .iter()
            .find(|path| path.stable_key() == REMEMBRANCE_PATH_KEY)
            .ok_or(PathEffectRuntimeError::InvalidDefinition)?;
        if path.blessings().len() != 18 || path.formations().len() != 3 {
            return Err(PathEffectRuntimeError::InvalidDefinition);
        }

        let mut blessings = Vec::with_capacity(18);
        for blessing_id in path.blessings() {
            let blessing = catalog
                .blessing(*blessing_id)
                .ok_or(PathEffectRuntimeError::InvalidDefinition)?;
            if blessing.path() != path.id() || blessing.levels().len() != 2 {
                return Err(PathEffectRuntimeError::InvalidDefinition);
            }
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
        if blessings.len() != 18 || resonances.len() != 4 {
            return Err(PathEffectRuntimeError::InvalidDefinition);
        }
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
    if program.template.event() != event
        && !(program.template == RemembranceTemplate::DamageFreeze
            && event == PathBattleEvent::StatQueried)
        && !(program.template == RemembranceTemplate::ResonanceEnergy
            && event == PathBattleEvent::EnemyFrozen)
    {
        return Ok(Box::new([]));
    }
    let p = &program.parameters;
    let mut effects = Vec::with_capacity(2);
    match program.template {
        RemembranceTemplate::FrozenAttackDissociation => {
            if facts.enemy_is_frozen {
                effects.push(PathEffect::ApplyDissociation {
                    target: PathEffectTarget::PrimaryEnemy,
                    base_chance: p[0],
                    duration_turns: turns(p[1])?,
                    maximum_hp_damage_ratio: PathEffectValue::from_raw_six_decimal(300_000),
                    removal_damage_bonus_ratio: p[2],
                    ignore_freeze_resistance: false,
                });
            }
        }
        RemembranceTemplate::BreakDissociation => effects.push(PathEffect::ApplyDissociation {
            target: PathEffectTarget::PrimaryEnemy,
            base_chance: p[0],
            duration_turns: turns(p[1])?,
            maximum_hp_damage_ratio: PathEffectValue::from_raw_six_decimal(300_000),
            removal_damage_bonus_ratio: PathEffectValue::ZERO,
            ignore_freeze_resistance: p[2] == PathEffectValue::ONE,
        }),
        RemembranceTemplate::RepeatedAttackFreeze => {
            if facts.enemy_attack_count >= count(p[0])? {
                effects.push(freeze(PathEffectTarget::PrimaryEnemy, p[1], turns(p[2])?));
            }
        }
        RemembranceTemplate::DetonateDissociation => {
            if facts.enemy_is_dissociated {
                effects.push(PathEffect::RemoveDissociation {
                    target: PathEffectTarget::PrimaryEnemy,
                    removal_damage_multiplier: p[0],
                });
            }
        }
        RemembranceTemplate::DissociationVulnerability => {
            if facts.enemy_is_dissociated || facts.enemy_has_dissociation_vulnerability {
                effects.push(stat(
                    PathEffectTarget::PrimaryEnemy,
                    PathEffectStat::DamageTakenRatio,
                    p[0],
                ));
            }
        }
        RemembranceTemplate::RemovedDissociationFreeze => {
            effects.push(freeze(PathEffectTarget::PrimaryEnemy, p[0], turns(p[1])?))
        }
        RemembranceTemplate::IceDamageSplash => effects.push(damage(
            if count(p[1])? == 1 {
                PathEffectTarget::AdjacentEnemies
            } else {
                PathEffectTarget::OtherEnemies
            },
            facts.damage_dealt.checked_multiply_ratio(p[0])?,
        )),
        RemembranceTemplate::DamageFreeze => {
            if event == PathBattleEvent::StatQueried {
                if p[2] != PathEffectValue::ZERO {
                    effects.push(stat(
                        PathEffectTarget::AllEnemies,
                        PathEffectStat::FreezeResistanceReductionRatio,
                        p[2],
                    ));
                }
            } else {
                effects.push(freeze(PathEffectTarget::PrimaryEnemy, p[0], turns(p[1])?));
            }
        }
        RemembranceTemplate::UltimateIceWeakness => {
            effects.push(PathEffect::ApplyIceWeakness {
                target: if count(p[2])? == 2 {
                    PathEffectTarget::RandomEnemyWithoutIceWeakness
                } else {
                    PathEffectTarget::RandomEnemy
                },
                base_chance: p[0],
                duration_turns: turns(p[1])?,
            });
        }
        RemembranceTemplate::EntryFreeze => effects.push(PathEffect::ApplyFreeze {
            target: PathEffectTarget::AllEnemies,
            base_chance: p[0],
            duration_turns: turns(p[1])?,
            speed_reduction_ratio: p[2],
            ignore_freeze_resistance: false,
        }),
        RemembranceTemplate::BlessingFreezeResistance => effects.push(stat(
            PathEffectTarget::AllEnemies,
            PathEffectStat::FreezeResistanceReductionRatio,
            p[0].checked_multiply_count(facts.path_blessing_count.min(count(p[1])?))?,
        )),
        RemembranceTemplate::EffectHitRate => effects.push(stat(
            PathEffectTarget::AllAllies,
            PathEffectStat::EffectHitRateRatio,
            p[0],
        )),
        RemembranceTemplate::HalfHpFreeze => {
            if facts.enemy_crossed_hp_threshold_first_time && facts.enemy_current_hp_ratio < p[0] {
                effects.push(freeze(PathEffectTarget::PrimaryEnemy, p[1], turns(p[2])?));
            }
        }
        RemembranceTemplate::FrozenSkillDamage => {
            if facts.enemy_is_frozen && facts.action_is_skill_or_ultimate {
                effects.push(stat(
                    PathEffectTarget::Actor,
                    PathEffectStat::DamageRatio,
                    p[0],
                ));
            }
        }
        RemembranceTemplate::FrozenCriticalExposure => {
            effects.push(PathEffect::MarkCriticalExposure {
                target: PathEffectTarget::PrimaryEnemy,
                attacks: turns(p[0])?,
                critical_rate_ratio: PathEffectValue::ONE,
            })
        }
        RemembranceTemplate::FrozenDamageTaken => {
            if facts.enemy_is_frozen {
                effects.push(stat(
                    PathEffectTarget::PrimaryEnemy,
                    PathEffectStat::DamageTakenRatio,
                    p[0],
                ));
            }
        }
        RemembranceTemplate::FreezeEnergy => effects.push(PathEffect::GainEnergy {
            target: PathEffectTarget::Actor,
            amount: p[0],
            once_per_action: true,
        }),
        RemembranceTemplate::FreezeShield => effects.push(PathEffect::Shield {
            target: PathEffectTarget::Actor,
            amount: facts.actor_maximum_hp.checked_multiply_ratio(p[0])?,
            duration_turns: turns(p[1])?,
            special: false,
            fixed_chance: PathEffectValue::ONE,
        }),
        RemembranceTemplate::ResonanceDamageFreeze => {
            effects.push(resonance_damage(
                facts.path_base_damage.checked_multiply_ratio(p[0])?,
            ));
            effects.push(freeze(PathEffectTarget::AllEnemies, p[1], turns(p[2])?));
        }
        RemembranceTemplate::ResonanceFreezeResistance => {
            effects.push(PathEffect::ApplyFreezeResistanceReduction {
                target: PathEffectTarget::AllEnemies,
                base_chance: PathEffectValue::from_raw_six_decimal(1_500_000),
                value: PathEffectValue::ONE,
                duration_turns: turns(p[3])?,
            });
        }
        RemembranceTemplate::ResonanceEonianRiver => effects.push(PathEffect::ApplyEonianRiver {
            target: PathEffectTarget::AllEnemies,
            base_chance: PathEffectValue::from_raw_six_decimal(1_500_000),
            duration_turns: 1,
        }),
        RemembranceTemplate::ResonanceEnergy => {
            effects.push(PathEffect::GainResonanceEnergy {
                maximum_ratio: if event == PathBattleEvent::BattleStarted {
                    p[4]
                } else {
                    p[5]
                },
            });
        }
    }
    Ok(effects
        .into_iter()
        .map(|effect| AppliedPathEffect::new(&program.source_key, effect))
        .collect::<Vec<_>>()
        .into_boxed_slice())
}

fn freeze(
    target: PathEffectTarget,
    base_chance: PathEffectValue,
    duration_turns: u8,
) -> PathEffect {
    PathEffect::ApplyFreeze {
        target,
        base_chance,
        duration_turns,
        speed_reduction_ratio: PathEffectValue::ZERO,
        ignore_freeze_resistance: false,
    }
}

fn damage(target: PathEffectTarget, amount: PathEffectValue) -> PathEffect {
    PathEffect::Damage {
        target,
        amount,
        kind: PathEffectDamageKind::PathAdditional,
        element: PathEffectElement::Ice,
        can_defeat: true,
        force_critical: false,
        critical_damage_ratio: PathEffectValue::ZERO,
    }
}

fn resonance_damage(amount: PathEffectValue) -> PathEffect {
    PathEffect::Damage {
        target: PathEffectTarget::AllEnemies,
        amount,
        kind: PathEffectDamageKind::PathResonance,
        element: PathEffectElement::Ice,
        can_defeat: true,
        force_critical: false,
        critical_damage_ratio: PathEffectValue::ZERO,
    }
}

fn stat(target: PathEffectTarget, stat: PathEffectStat, value: PathEffectValue) -> PathEffect {
    PathEffect::AddStat {
        target,
        stat,
        value,
        cap: None,
    }
}

/// The only stable source-binding dispatch for this partition.
fn registry(key: &str) -> Result<(RemembranceTemplate, usize), PathEffectRuntimeError> {
    use RemembranceTemplate as T;
    Ok(match key {
        "StageAbility_612130" => (T::FrozenAttackDissociation, 3),
        "StageAbility_612131" => (T::BreakDissociation, 3),
        "StageAbility_612132" => (T::RepeatedAttackFreeze, 3),
        "StageAbility_612140" => (T::DetonateDissociation, 1),
        "StageAbility_612141" => (T::DissociationVulnerability, 1),
        "StageAbility_612142" => (T::RemovedDissociationFreeze, 2),
        "StageAbility_612143" => (T::IceDamageSplash, 2),
        "StageAbility_612144" => (T::DamageFreeze, 3),
        "StageAbility_612145" => (T::UltimateIceWeakness, 3),
        "StageAbility_612146" => (T::EntryFreeze, 4),
        "StageAbility_612150" => (T::BlessingFreezeResistance, 2),
        "StageAbility_612151" => (T::EffectHitRate, 1),
        "StageAbility_612152" => (T::HalfHpFreeze, 3),
        "StageAbility_612153" => (T::FrozenSkillDamage, 1),
        "StageAbility_612154" => (T::FrozenCriticalExposure, 1),
        "StageAbility_612155" => (T::FrozenDamageTaken, 1),
        "StageAbility_612156" => (T::FreezeEnergy, 1),
        "StageAbility_612157" => (T::FreezeShield, 2),
        "StageAbility_612120" => (T::ResonanceDamageFreeze, 6),
        "StageAbility_612121" => (T::ResonanceFreezeResistance, 6),
        "StageAbility_612122" => (T::ResonanceEonianRiver, 6),
        "StageAbility_612123" => (T::ResonanceEnergy, 6),
        _ => return Err(PathEffectRuntimeError::UnknownSource),
    })
}

fn catalog_digest(
    path: PathId,
    blessings: &[BlessingPrograms],
    resonances: &[ResonanceProgram],
) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock-universe-remembrance-runtime-catalog-v1");
    encoder.text(REMEMBRANCE_RUNTIME_REVISION);
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
