//! Closed executor for the released Standard Universe Preservation partition.

use crate::{
    catalog::UniverseCatalog,
    digest::Encoder,
    id::{BlessingId, PathId, ResonanceId},
    path::ExactParameter,
    path_effect_runtime::{
        AppliedPathEffect, PathBattleEvent, PathEffect, PathEffectDamageKind, PathEffectElement,
        PathEffectFacts, PathEffectRuntimeError, PathEffectStat, PathEffectTarget, PathEffectValue,
    },
};

pub const PRESERVATION_RUNTIME_REVISION: &str = "standard-universe-preservation-runtime-v1";
const PRESERVATION_PATH_KEY: &str = "universe.path.preservation";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
enum PreservationTemplate {
    AttackQuake = 0,
    RetaliatoryQuake = 1,
    Macrosegregation = 2,
    QuakeSplash = 3,
    QuakeBleed = 4,
    DefenseQuake = 5,
    ShieldAttack = 6,
    TurnEndShield = 7,
    ShieldCapacity = 8,
    ProviderShield = 9,
    BlessingDefense = 10,
    EntryShield = 11,
    LostHpShield = 12,
    BreakShield = 13,
    ShieldDamageReduction = 14,
    ShieldDispel = 15,
    ShieldCriticalDamage = 16,
    ShieldCriticalRate = 17,
    ResonanceDamage = 18,
    ResonanceCritical = 19,
    ResonanceShieldAmber = 20,
    ResonanceEnergy = 21,
}

impl PreservationTemplate {
    const fn event(self) -> PathBattleEvent {
        match self {
            Self::AttackQuake => PathBattleEvent::AttackHit,
            Self::RetaliatoryQuake | Self::LostHpShield => PathBattleEvent::CharacterAttacked,
            Self::Macrosegregation | Self::EntryShield | Self::BlessingDefense => {
                PathBattleEvent::BattleStarted
            }
            Self::QuakeSplash | Self::QuakeBleed | Self::DefenseQuake => {
                PathBattleEvent::PathDamageDealt
            }
            Self::ShieldAttack | Self::ShieldCriticalDamage | Self::ShieldCriticalRate => {
                PathBattleEvent::StatQueried
            }
            Self::ShieldDamageReduction => PathBattleEvent::DamageCalculated,
            Self::TurnEndShield => PathBattleEvent::TurnEnded,
            Self::ShieldCapacity | Self::ShieldDispel => PathBattleEvent::ShieldGranted,
            Self::ProviderShield => PathBattleEvent::ShieldGrantedToAlly,
            Self::BreakShield => PathBattleEvent::WeaknessBroken,
            Self::ResonanceDamage | Self::ResonanceCritical | Self::ResonanceShieldAmber => {
                PathBattleEvent::PathResonanceActivated
            }
            Self::ResonanceEnergy => PathBattleEvent::ShieldGranted,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CompiledProgram {
    source_key: Box<str>,
    template: PreservationTemplate,
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
pub struct PreservationRuntimeCatalog {
    path: PathId,
    blessings: Box<[BlessingPrograms]>,
    resonances: Box<[ResonanceProgram]>,
    digest: [u8; 32],
}

impl PreservationRuntimeCatalog {
    pub fn compile(catalog: &UniverseCatalog) -> Result<Self, PathEffectRuntimeError> {
        let path = catalog
            .paths()
            .iter()
            .find(|path| path.stable_key() == PRESERVATION_PATH_KEY)
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
        && !(program.template == PreservationTemplate::Macrosegregation
            && event == PathBattleEvent::ShieldGranted)
        && !(program.template == PreservationTemplate::ResonanceEnergy
            && event == PathBattleEvent::BattleStarted)
    {
        return Ok(Box::new([]));
    }
    let p = &program.parameters;
    let mut effects = Vec::with_capacity(2);
    match program.template {
        PreservationTemplate::AttackQuake => effects.push(damage(
            PathEffectTarget::HitEnemies,
            facts
                .actor_current_shield
                .checked_multiply_ratio(p[0])?
                .checked_add(facts.teammate_shield_total.checked_multiply_ratio(p[1])?)?,
            true,
        )),
        PreservationTemplate::RetaliatoryQuake => effects.push(damage(
            PathEffectTarget::Attacker,
            if p[1].raw_six_decimal() == 2_000_000 {
                facts.actor_shield_before_hit
            } else {
                facts.actor_current_shield
            }
            .checked_multiply_ratio(p[0])?,
            false,
        )),
        PreservationTemplate::Macrosegregation => {
            let turns = turns(p[2])?;
            if event == PathBattleEvent::BattleStarted {
                effects.push(PathEffect::Shield {
                    target: PathEffectTarget::AllAllies,
                    amount: facts.actor_maximum_hp.checked_multiply_ratio(p[0])?,
                    duration_turns: turns,
                    special: true,
                    fixed_chance: PathEffectValue::ONE,
                });
            } else {
                effects.push(PathEffect::StrengthenSpecialShield {
                    target: PathEffectTarget::Actor,
                    amount: facts.provided_shield.checked_multiply_ratio(p[1])?,
                    cycle_turns: turns,
                });
            }
        }
        PreservationTemplate::QuakeSplash => {
            effects.push(PathEffect::AddStat {
                target: PathEffectTarget::Actor,
                stat: PathEffectStat::PathDamageRatio,
                value: p[2],
                cap: None,
            });
            effects.push(damage(
                if count(p[1])? == 1 {
                    PathEffectTarget::AdjacentEnemies
                } else {
                    PathEffectTarget::OtherEnemies
                },
                facts.path_damage.checked_multiply_ratio(p[0])?,
                true,
            ));
        }
        PreservationTemplate::QuakeBleed => effects.push(PathEffect::ApplyBleed {
            target: PathEffectTarget::PrimaryEnemy,
            base_chance: p[0],
            maximum_hp_ratio: p[1],
            damage_cap_ratio: p[2],
            duration_turns: turns(p[3])?,
        }),
        PreservationTemplate::DefenseQuake => effects.push(damage(
            PathEffectTarget::PrimaryEnemy,
            facts.actor_defense.checked_multiply_ratio(p[0])?,
            true,
        )),
        PreservationTemplate::ShieldAttack => {
            if facts.actor_is_shielded {
                effects.push(PathEffect::AddStat {
                    target: PathEffectTarget::Actor,
                    stat: PathEffectStat::AttackFlat,
                    value: facts.actor_current_shield.checked_multiply_ratio(p[0])?,
                    cap: Some(facts.actor_base_attack.checked_multiply_ratio(p[1])?),
                });
            }
        }
        PreservationTemplate::TurnEndShield => effects.push(PathEffect::Shield {
            target: PathEffectTarget::Actor,
            amount: facts.actor_maximum_hp.checked_multiply_ratio(p[1])?,
            duration_turns: turns(p[2])?,
            special: p[0] == PathEffectValue::ONE,
            fixed_chance: p[0],
        }),
        PreservationTemplate::ShieldCapacity => effects.push(PathEffect::AddStat {
            target: PathEffectTarget::Actor,
            stat: PathEffectStat::ShieldCapacityRatio,
            value: p[0],
            cap: None,
        }),
        PreservationTemplate::ProviderShield => effects.push(PathEffect::Shield {
            target: PathEffectTarget::ShieldProvider,
            amount: facts.provided_shield.checked_multiply_ratio(p[0])?,
            duration_turns: turns(p[1])?,
            special: false,
            fixed_chance: PathEffectValue::ONE,
        }),
        PreservationTemplate::BlessingDefense => effects.push(PathEffect::AddStat {
            target: PathEffectTarget::AllAllies,
            stat: PathEffectStat::DefenseRatio,
            value: p[0].checked_multiply_count(facts.path_blessing_count.min(count(p[1])?))?,
            cap: None,
        }),
        PreservationTemplate::EntryShield => effects.push(PathEffect::Shield {
            target: PathEffectTarget::AllAllies,
            amount: facts.actor_maximum_hp.checked_multiply_ratio(p[0])?,
            duration_turns: turns(p[1])?,
            special: false,
            fixed_chance: PathEffectValue::ONE,
        }),
        PreservationTemplate::LostHpShield => effects.push(PathEffect::Shield {
            target: PathEffectTarget::Actor,
            amount: facts.hp_lost.checked_multiply_ratio(p[0])?,
            duration_turns: turns(p[1])?,
            special: false,
            fixed_chance: PathEffectValue::ONE,
        }),
        PreservationTemplate::BreakShield => effects.push(PathEffect::Shield {
            target: PathEffectTarget::AllAllies,
            amount: facts.actor_maximum_hp.checked_multiply_ratio(p[0])?,
            duration_turns: turns(p[1])?,
            special: false,
            fixed_chance: PathEffectValue::ONE,
        }),
        PreservationTemplate::ShieldDamageReduction => {
            conditional_stat(
                &mut effects,
                &facts,
                PathEffectStat::DamageTakenReductionRatio,
                p[0],
            );
        }
        PreservationTemplate::ShieldDispel => effects.push(PathEffect::DispelDebuff {
            target: PathEffectTarget::Actor,
            fixed_chance: p[0],
            count: 1,
        }),
        PreservationTemplate::ShieldCriticalDamage => {
            conditional_stat(
                &mut effects,
                &facts,
                PathEffectStat::CriticalDamageRatio,
                p[0],
            );
        }
        PreservationTemplate::ShieldCriticalRate => {
            conditional_stat(
                &mut effects,
                &facts,
                PathEffectStat::CriticalRateRatio,
                p[0],
            );
        }
        PreservationTemplate::ResonanceDamage => effects.push(resonance_damage(
            PathEffectTarget::AllEnemies,
            facts.party_shield_total.checked_multiply_ratio(p[0])?,
            true,
        )),
        PreservationTemplate::ResonanceCritical => effects.push(PathEffect::Damage {
            target: PathEffectTarget::AllEnemies,
            amount: PathEffectValue::ZERO,
            kind: PathEffectDamageKind::PathResonance,
            element: PathEffectElement::Physical,
            can_defeat: true,
            force_critical: true,
            critical_damage_ratio: p[3].checked_multiply_count(facts.shielded_allies)?,
        }),
        PreservationTemplate::ResonanceShieldAmber => {
            effects.push(PathEffect::Shield {
                target: PathEffectTarget::AllAllies,
                amount: facts.actor_maximum_hp.checked_multiply_ratio(p[7])?,
                duration_turns: 2,
                special: false,
                fixed_chance: PathEffectValue::ONE,
            });
            effects.push(PathEffect::ApplyAmber {
                target: PathEffectTarget::AllAllies,
            });
        }
        PreservationTemplate::ResonanceEnergy => effects.push(PathEffect::GainResonanceEnergy {
            maximum_ratio: if event == PathBattleEvent::BattleStarted {
                p[4]
            } else {
                p[5]
            },
        }),
    }
    Ok(effects
        .into_iter()
        .map(|effect| AppliedPathEffect::new(&program.source_key, effect))
        .collect::<Vec<_>>()
        .into_boxed_slice())
}

fn damage(target: PathEffectTarget, amount: PathEffectValue, can_defeat: bool) -> PathEffect {
    PathEffect::Damage {
        target,
        amount,
        kind: PathEffectDamageKind::PathAdditional,
        element: PathEffectElement::Physical,
        can_defeat,
        force_critical: false,
        critical_damage_ratio: PathEffectValue::ZERO,
    }
}

fn resonance_damage(
    target: PathEffectTarget,
    amount: PathEffectValue,
    can_defeat: bool,
) -> PathEffect {
    PathEffect::Damage {
        target,
        amount,
        kind: PathEffectDamageKind::PathResonance,
        element: PathEffectElement::Physical,
        can_defeat,
        force_critical: false,
        critical_damage_ratio: PathEffectValue::ZERO,
    }
}

fn conditional_stat(
    effects: &mut Vec<PathEffect>,
    facts: &PathEffectFacts,
    stat: PathEffectStat,
    value: PathEffectValue,
) {
    if facts.actor_is_shielded {
        effects.push(PathEffect::AddStat {
            target: PathEffectTarget::Actor,
            stat,
            value,
            cap: None,
        });
    }
}

fn count(value: PathEffectValue) -> Result<u32, PathEffectRuntimeError> {
    let raw = value.raw_six_decimal();
    if raw < 0 || raw % 1_000_000 != 0 {
        return Err(PathEffectRuntimeError::InvalidParameter);
    }
    u32::try_from(raw / 1_000_000).map_err(|_| PathEffectRuntimeError::InvalidParameter)
}

fn turns(value: PathEffectValue) -> Result<u8, PathEffectRuntimeError> {
    u8::try_from(count(value)?).map_err(|_| PathEffectRuntimeError::InvalidParameter)
}

fn exact_parameters(
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

/// The only stable source-binding dispatch for this partition.
fn registry(key: &str) -> Result<(PreservationTemplate, usize), PathEffectRuntimeError> {
    use PreservationTemplate as T;
    Ok(match key {
        "StageAbility_612030" => (T::AttackQuake, 2),
        "StageAbility_612031" => (T::RetaliatoryQuake, 2),
        "StageAbility_612032" => (T::Macrosegregation, 3),
        "StageAbility_612040" => (T::QuakeSplash, 3),
        "StageAbility_612041" => (T::QuakeBleed, 4),
        "StageAbility_612042" => (T::DefenseQuake, 1),
        "StageAbility_612043" => (T::ShieldAttack, 2),
        "StageAbility_612044" => (T::TurnEndShield, 3),
        "StageAbility_612045" => (T::ShieldCapacity, 2),
        "StageAbility_612046" => (T::ProviderShield, 2),
        "StageAbility_612050" => (T::BlessingDefense, 2),
        "StageAbility_612051" => (T::EntryShield, 2),
        "StageAbility_612052" => (T::LostHpShield, 2),
        "StageAbility_612053" => (T::BreakShield, 2),
        "StageAbility_612054" => (T::ShieldDamageReduction, 1),
        "StageAbility_612055" => (T::ShieldDispel, 1),
        "StageAbility_612056" => (T::ShieldCriticalDamage, 1),
        "StageAbility_612057" => (T::ShieldCriticalRate, 1),
        "StageAbility_612020" => (T::ResonanceDamage, 8),
        "StageAbility_612021" => (T::ResonanceCritical, 8),
        "StageAbility_612022" => (T::ResonanceShieldAmber, 8),
        "StageAbility_612023" => (T::ResonanceEnergy, 8),
        _ => return Err(PathEffectRuntimeError::UnknownSource),
    })
}

fn catalog_digest(
    path: PathId,
    blessings: &[BlessingPrograms],
    resonances: &[ResonanceProgram],
) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock-universe-preservation-runtime-catalog-v1");
    encoder.text(PRESERVATION_RUNTIME_REVISION);
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
