//! Closed executor for the released Standard Universe Erudition partition.

use crate::{
    catalog::UniverseCatalog,
    digest::Encoder,
    id::{BlessingId, PathId, ResonanceId},
    path_effect_runtime::{
        AppliedPathEffect, PathBattleEvent, PathEffect, PathEffectFacts, PathEffectRuntimeError,
        PathEffectStat, PathEffectTarget, PathEffectValue, count, exact_parameters, turns,
    },
};

pub const ERUDITION_RUNTIME_REVISION: &str = "standard-universe-erudition-runtime-v1";
const ERUDITION_PATH_KEY: &str = "universe.path.erudition";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
enum EruditionTemplate {
    BrainCharge = 0,
    DefeatCharge = 1,
    UltimateResistancePenetration = 2,
    BrainUltimateCriticalDamage = 3,
    OverflowCharge = 4,
    BrainUltimateShield = 5,
    AttackedTargetDamage = 6,
    EntryUltimateEnergy = 7,
    AoeSingleTargetDamage = 8,
    BrokenUltimateDelay = 9,
    BlessingUltimateDamage = 10,
    UltimateCriticalRate = 11,
    UltimateCriticalDamage = 12,
    UltimateNextAttack = 13,
    AoeAttack = 14,
    AoeDefense = 15,
    UltimateHealing = 16,
    LethalEnergyHealing = 17,
    ResonanceSynapse = 18,
    ResonanceMeltCore = 19,
    ResonanceChainContagion = 20,
    ResonanceMemeticInversion = 21,
}

impl EruditionTemplate {
    const fn event(self) -> PathBattleEvent {
        match self {
            Self::BrainCharge | Self::EntryUltimateEnergy => PathBattleEvent::BattleStarted,
            Self::DefeatCharge => PathBattleEvent::EnemyDefeated,
            Self::UltimateResistancePenetration
            | Self::BlessingUltimateDamage
            | Self::UltimateCriticalRate
            | Self::UltimateCriticalDamage => PathBattleEvent::StatQueried,
            Self::BrainUltimateCriticalDamage | Self::BrainUltimateShield => {
                PathBattleEvent::UltimateViaBrainInVatUsed
            }
            Self::OverflowCharge => PathBattleEvent::EnergyOverflowed,
            Self::AttackedTargetDamage => PathBattleEvent::AttackCompleted,
            Self::AoeSingleTargetDamage | Self::AoeAttack | Self::AoeDefense => {
                PathBattleEvent::AoeAttackUsed
            }
            Self::BrokenUltimateDelay | Self::UltimateNextAttack | Self::UltimateHealing => {
                PathBattleEvent::UltimateUsed
            }
            Self::LethalEnergyHealing => PathBattleEvent::LethalDamageReceived,
            Self::ResonanceSynapse => PathBattleEvent::PathResonanceActivated,
            Self::ResonanceMeltCore
            | Self::ResonanceChainContagion
            | Self::ResonanceMemeticInversion => PathBattleEvent::BattleStarted,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CompiledProgram {
    source_key: Box<str>,
    template: EruditionTemplate,
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
pub struct EruditionRuntimeCatalog {
    path: PathId,
    blessings: Box<[BlessingPrograms]>,
    resonances: Box<[ResonanceProgram]>,
    digest: [u8; 32],
}

impl EruditionRuntimeCatalog {
    pub fn compile(catalog: &UniverseCatalog) -> Result<Self, PathEffectRuntimeError> {
        let path = catalog
            .paths()
            .iter()
            .find(|path| path.stable_key() == ERUDITION_PATH_KEY)
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
    let alternate = program.template == EruditionTemplate::BrainCharge
        && matches!(
            event,
            PathBattleEvent::WeaknessBroken | PathBattleEvent::AttackHit
        );
    if program.template.event() != event && !alternate {
        return Ok(Box::new([]));
    }
    let p = &program.parameters;
    let mut effects = Vec::with_capacity(2);
    match program.template {
        EruditionTemplate::BrainCharge => {
            let ratio = match event {
                PathBattleEvent::BattleStarted => p[1],
                PathBattleEvent::WeaknessBroken => p[0],
                PathBattleEvent::AttackHit if facts.enemy_is_weakness_broken => p[3],
                _ => PathEffectValue::ZERO,
            };
            if ratio != PathEffectValue::ZERO {
                effects.push(PathEffect::ChargeBrainInVat {
                    ratio,
                    once_per_enemy_per_attack: event == PathBattleEvent::AttackHit,
                });
            }
        }
        EruditionTemplate::DefeatCharge => {
            effects.push(PathEffect::ChargeBrainInVat {
                ratio: p[0],
                once_per_enemy_per_attack: false,
            });
            if p[1] != PathEffectValue::ZERO {
                effects.push(PathEffect::ApplyBrainFullSpeed {
                    speed_ratio: p[1],
                    duration_turns: turns(p[2])?,
                });
            }
        }
        EruditionTemplate::UltimateResistancePenetration => {
            let hit_count = if p[2] == PathEffectValue::ZERO {
                facts.ultimate_targets_hit
            } else {
                facts.maximum_ultimate_targets_hit
            };
            effects.push(PathEffect::AddUltimateModifier {
                stat: PathEffectStat::AllTypeResistancePenetrationRatio,
                value: p[0].checked_add(p[1].checked_multiply_count(hit_count)?)?,
                until_next_ultimate: p[2] == PathEffectValue::ZERO,
            });
        }
        EruditionTemplate::BrainUltimateCriticalDamage => {
            effects.push(PathEffect::AddUltimateModifier {
                stat: PathEffectStat::CriticalDamageRatio,
                value: p[0],
                until_next_ultimate: p[1] != PathEffectValue::ZERO,
            });
        }
        EruditionTemplate::OverflowCharge => {
            effects.push(PathEffect::ChargeBrainInVat {
                ratio: facts.excess_energy.checked_multiply_ratio(p[0])?,
                once_per_enemy_per_attack: false,
            });
        }
        EruditionTemplate::BrainUltimateShield => effects.push(PathEffect::Shield {
            target: PathEffectTarget::Actor,
            amount: facts.actor_maximum_hp.checked_multiply_ratio(p[0])?,
            duration_turns: turns(p[1])?,
            special: false,
            fixed_chance: PathEffectValue::ONE,
        }),
        EruditionTemplate::AttackedTargetDamage => {
            let defeated = if p[1] == PathEffectValue::ZERO {
                0
            } else {
                facts.defeated_enemy_count.min(5)
            };
            effects.push(PathEffect::AdditionalDamagePerAttackedEnemy {
                target: PathEffectTarget::HitEnemies,
                attack_ratio_per_enemy: p[0],
                enemy_count: u8_count(
                    facts
                        .attacked_enemy_count
                        .checked_add(defeated)
                        .ok_or(PathEffectRuntimeError::Overflow)?,
                )?,
                include_defeated_enemies_up_to: if p[1] == PathEffectValue::ZERO { 0 } else { 5 },
            });
        }
        EruditionTemplate::EntryUltimateEnergy => {
            effects.push(PathEffect::AddUltimateModifier {
                stat: PathEffectStat::DamageRatio,
                value: p[0],
                until_next_ultimate: p[2] == PathEffectValue::ZERO,
            });
            effects.push(PathEffect::RegenerateMaximumEnergyRatio {
                target: PathEffectTarget::Actor,
                ratio: p[1],
            });
        }
        EruditionTemplate::AoeSingleTargetDamage => {
            if facts.attacked_enemy_count == 1 {
                effects.push(PathEffect::AoeSingleTargetRepeatDamage {
                    target: PathEffectTarget::PrimaryEnemy,
                    original_damage_ratio: p[0],
                });
            }
        }
        EruditionTemplate::BrokenUltimateDelay => {
            if facts.enemy_is_weakness_broken {
                effects.push(PathEffect::UltimateWeaknessBrokenDelay {
                    target: PathEffectTarget::HitEnemies,
                    action_delay_ratio: p[0],
                    maximum_triggers_per_break: turns(p[1])?,
                });
            }
        }
        EruditionTemplate::BlessingUltimateDamage => effects.push(ultimate_modifier(
            PathEffectStat::DamageRatio,
            p[0].checked_multiply_count(facts.path_blessing_count.min(count(p[1])?))?,
        )),
        EruditionTemplate::UltimateCriticalRate => {
            effects.push(ultimate_modifier(PathEffectStat::CriticalRateRatio, p[0]));
        }
        EruditionTemplate::UltimateCriticalDamage => {
            effects.push(ultimate_modifier(PathEffectStat::CriticalDamageRatio, p[0]));
        }
        EruditionTemplate::UltimateNextAttack => {
            effects.push(PathEffect::IncreaseNextAttackDamage {
                target: PathEffectTarget::Actor,
                ratio: p[0],
            });
        }
        EruditionTemplate::AoeAttack => {
            effects.push(timed_stat(PathEffectStat::AttackRatio, p[0], turns(p[1])?))
        }
        EruditionTemplate::AoeDefense => {
            effects.push(timed_stat(PathEffectStat::DefenseRatio, p[0], turns(p[1])?))
        }
        EruditionTemplate::UltimateHealing => effects.push(PathEffect::HealMaximumHpRatio {
            target: PathEffectTarget::Actor,
            ratio: p[0],
        }),
        EruditionTemplate::LethalEnergyHealing => {
            effects.push(PathEffect::PreventDefeatConsumeEnergyHeal {
                target: PathEffectTarget::Actor,
                healing_per_energy_ratio: p[0],
                maximum_team_triggers_per_battle: 1,
            });
        }
        EruditionTemplate::ResonanceSynapse => effects.push(PathEffect::ApplySynapseResonance {
            target: PathEffectTarget::AllEnemies,
            damage_ratio_to_linked_targets: p[0],
            maximum_triggers: turns(p[2])?,
        }),
        EruditionTemplate::ResonanceMeltCore => {
            effects.push(PathEffect::ConfigureSynapseResonance {
                ultimate_attack_ratio: p[0],
                extra_triggers_on_defeat: 0,
                enemy_appearance_energy_maximum_ratio: PathEffectValue::ZERO,
            });
        }
        EruditionTemplate::ResonanceChainContagion => {
            effects.push(PathEffect::ConfigureSynapseResonance {
                ultimate_attack_ratio: PathEffectValue::ZERO,
                extra_triggers_on_defeat: turns(p[0])?,
                enemy_appearance_energy_maximum_ratio: PathEffectValue::ZERO,
            });
        }
        EruditionTemplate::ResonanceMemeticInversion => {
            effects.push(PathEffect::ConfigureSynapseResonance {
                ultimate_attack_ratio: PathEffectValue::ZERO,
                extra_triggers_on_defeat: 0,
                enemy_appearance_energy_maximum_ratio: p[0],
            });
        }
    }
    Ok(effects
        .into_iter()
        .map(|effect| AppliedPathEffect::new(&program.source_key, effect))
        .collect::<Vec<_>>()
        .into_boxed_slice())
}

fn ultimate_modifier(stat: PathEffectStat, value: PathEffectValue) -> PathEffect {
    PathEffect::AddUltimateModifier {
        stat,
        value,
        until_next_ultimate: false,
    }
}

fn timed_stat(stat: PathEffectStat, value: PathEffectValue, duration_turns: u8) -> PathEffect {
    PathEffect::ApplyTimedStat {
        target: PathEffectTarget::Actor,
        stat,
        value,
        duration_turns,
        maximum_stacks: 1,
    }
}

fn u8_count(value: u32) -> Result<u8, PathEffectRuntimeError> {
    u8::try_from(value).map_err(|_| PathEffectRuntimeError::InvalidFacts)
}

fn registry(key: &str) -> Result<(EruditionTemplate, usize), PathEffectRuntimeError> {
    use EruditionTemplate as T;
    Ok(match key {
        "StageAbility_612830" => (T::BrainCharge, 4),
        "StageAbility_612831" => (T::DefeatCharge, 3),
        "StageAbility_612832" => (T::UltimateResistancePenetration, 3),
        "StageAbility_612840" => (T::BrainUltimateCriticalDamage, 2),
        "StageAbility_612841" => (T::OverflowCharge, 1),
        "StageAbility_612842" => (T::BrainUltimateShield, 2),
        "StageAbility_612843" => (T::AttackedTargetDamage, 2),
        "StageAbility_612844" => (T::EntryUltimateEnergy, 3),
        "StageAbility_612845" => (T::AoeSingleTargetDamage, 3),
        "StageAbility_612846" => (T::BrokenUltimateDelay, 3),
        "StageAbility_612850" => (T::BlessingUltimateDamage, 2),
        "StageAbility_612851" => (T::UltimateCriticalRate, 1),
        "StageAbility_612852" => (T::UltimateCriticalDamage, 1),
        "StageAbility_612853" => (T::UltimateNextAttack, 1),
        "StageAbility_612854" => (T::AoeAttack, 2),
        "StageAbility_612855" => (T::AoeDefense, 2),
        "StageAbility_612856" => (T::UltimateHealing, 1),
        "StageAbility_612857" => (T::LethalEnergyHealing, 1),
        "StageAbility_612820" => (T::ResonanceSynapse, 3),
        "StageAbility_612821" => (T::ResonanceMeltCore, 1),
        "StageAbility_612822" => (T::ResonanceChainContagion, 1),
        "StageAbility_612823" => (T::ResonanceMemeticInversion, 1),
        _ => return Err(PathEffectRuntimeError::UnknownSource),
    })
}

fn catalog_digest(
    path: PathId,
    blessings: &[BlessingPrograms],
    resonances: &[ResonanceProgram],
) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock-universe-erudition-runtime-catalog-v1");
    encoder.text(ERUDITION_RUNTIME_REVISION);
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
