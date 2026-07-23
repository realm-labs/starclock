//! Closed executor for the released Standard Universe Elation partition.

use crate::{
    catalog::UniverseCatalog,
    digest::Encoder,
    id::{BlessingId, PathId, ResonanceId},
    path_effect_runtime::{
        AppliedPathEffect, PathBattleEvent, PathEffect, PathEffectFacts, PathEffectRuntimeError,
        PathEffectStat, PathEffectTarget, PathEffectValue, count, exact_parameters, turns,
    },
};

pub const ELATION_RUNTIME_REVISION: &str = "standard-universe-elation-runtime-v1";
const ELATION_PATH_KEY: &str = "universe.path.elation";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
enum ElationTemplate {
    RandomAftertaste = 0,
    BrokenAftertaste = 1,
    UltimateAsFollowUp = 2,
    ExtraAftertaste = 3,
    AftertasteVulnerability = 4,
    AftertasteAttackReduction = 5,
    FollowUpTargetDamage = 6,
    FollowUpRamp = 7,
    FollowUpDelay = 8,
    FollowUpSkillPoint = 9,
    BlessingFollowUpDamage = 10,
    FollowUpBreakEfficiency = 11,
    FollowUpDamage = 12,
    FollowUpCriticalRate = 13,
    FollowUpEnergyRegeneration = 14,
    FollowUpHealing = 15,
    FollowUpDefense = 16,
    FollowUpSpeed = 17,
    ResonanceDamage = 18,
    ResonanceSensoryPursuit = 19,
    ResonanceVariableEnergy = 20,
    ResonanceEnergyGain = 21,
}

impl ElationTemplate {
    const fn event(self) -> PathBattleEvent {
        match self {
            Self::RandomAftertaste
            | Self::BrokenAftertaste
            | Self::FollowUpTargetDamage
            | Self::FollowUpDelay
            | Self::FollowUpSkillPoint
            | Self::FollowUpHealing
            | Self::FollowUpDefense
            | Self::FollowUpSpeed => PathBattleEvent::FollowUpAttackUsed,
            Self::UltimateAsFollowUp => PathBattleEvent::UltimateUsed,
            Self::ExtraAftertaste
            | Self::AftertasteVulnerability
            | Self::AftertasteAttackReduction => PathBattleEvent::AftertasteDamageDealt,
            Self::FollowUpRamp | Self::FollowUpEnergyRegeneration => {
                PathBattleEvent::FollowUpDamageDealt
            }
            Self::BlessingFollowUpDamage
            | Self::FollowUpBreakEfficiency
            | Self::FollowUpDamage
            | Self::FollowUpCriticalRate => PathBattleEvent::StatQueried,
            Self::ResonanceDamage
            | Self::ResonanceSensoryPursuit
            | Self::ResonanceVariableEnergy => PathBattleEvent::PathResonanceActivated,
            Self::ResonanceEnergyGain => PathBattleEvent::BattleStarted,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CompiledProgram {
    source_key: Box<str>,
    template: ElationTemplate,
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
pub struct ElationRuntimeCatalog {
    path: PathId,
    blessings: Box<[BlessingPrograms]>,
    resonances: Box<[ResonanceProgram]>,
    digest: [u8; 32],
}

impl ElationRuntimeCatalog {
    pub fn compile(catalog: &UniverseCatalog) -> Result<Self, PathEffectRuntimeError> {
        let path = catalog
            .paths()
            .iter()
            .find(|path| path.stable_key() == ELATION_PATH_KEY)
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
        ElationTemplate::RandomAftertaste => effects.push(PathEffect::DealAftertaste {
            target: PathEffectTarget::PrimaryEnemy,
            minimum_hits: turns(p[0])?,
            maximum_hits: turns(p[1])?,
            amount_per_hit: facts.actor_base_attack.checked_multiply_ratio(p[2])?,
            damage_bonus_ratio: p[3],
            random_element_each_hit: true,
        }),
        ElationTemplate::BrokenAftertaste => {
            let extra = if facts.enemy_is_weakness_broken {
                turns(p[1])?
            } else {
                0
            };
            effects.push(PathEffect::DealAftertaste {
                target: PathEffectTarget::PrimaryEnemy,
                minimum_hits: 1,
                maximum_hits: 1_u8
                    .checked_add(extra)
                    .ok_or(PathEffectRuntimeError::Overflow)?,
                amount_per_hit: facts.actor_base_attack.checked_multiply_ratio(p[0])?,
                damage_bonus_ratio: PathEffectValue::ZERO,
                random_element_each_hit: true,
            });
        }
        ElationTemplate::UltimateAsFollowUp => {
            effects.push(PathEffect::TreatUltimateDamageAsFollowUp {
                follow_up_damage_ratio: p[0],
            });
        }
        ElationTemplate::ExtraAftertaste => {
            effects.push(PathEffect::ExtraAftertasteFromOriginal {
                target: PathEffectTarget::PrimaryEnemy,
                hits: turns(p[0])?,
                original_damage_ratio: p[1],
                different_element: true,
            });
        }
        ElationTemplate::AftertasteVulnerability => {
            effects.push(aftertaste_modifier(
                PathEffectStat::DamageTakenRatio,
                p[0],
                facts,
            )?);
        }
        ElationTemplate::AftertasteAttackReduction => {
            effects.push(aftertaste_modifier(
                PathEffectStat::AttackReductionRatio,
                p[0],
                facts,
            )?);
        }
        ElationTemplate::FollowUpTargetDamage => effects.push(PathEffect::Damage {
            target: PathEffectTarget::HitEnemies,
            amount: facts
                .actor_base_attack
                .checked_multiply_ratio(p[0])?
                .checked_multiply_count(facts.follow_up_targets_hit)?,
            kind: crate::path_effect_runtime::PathEffectDamageKind::PathAdditional,
            element: crate::path_effect_runtime::PathEffectElement::InheritActor,
            can_defeat: true,
            force_critical: false,
            critical_damage_ratio: PathEffectValue::ZERO,
        }),
        ElationTemplate::FollowUpRamp => {
            effects.push(PathEffect::IncreaseFollowUpDamageWithinAttack {
                ratio_per_damage_instance: p[0],
            });
        }
        ElationTemplate::FollowUpDelay => {
            effects.push(PathEffect::DelayAction {
                target: PathEffectTarget::HitEnemies,
                ratio: p[0],
            });
            if p[1] != PathEffectValue::ZERO {
                effects.push(PathEffect::ApplyImprisonment {
                    target: PathEffectTarget::HitEnemies,
                    base_chance: p[1],
                    duration_turns: turns(p[2])?,
                    speed_reduction_ratio: p[4],
                    action_delay_ratio: p[5],
                });
            }
        }
        ElationTemplate::FollowUpSkillPoint => effects.push(PathEffect::GainSkillPoint {
            fixed_chance: p[0],
            amount: 1,
            once_per_action: true,
        }),
        ElationTemplate::BlessingFollowUpDamage => effects.push(follow_up_modifier(
            PathEffectStat::DamageRatio,
            p[0].checked_multiply_count(facts.path_blessing_count.min(count(p[1])?))?,
        )),
        ElationTemplate::FollowUpBreakEfficiency => effects.push(follow_up_modifier(
            PathEffectStat::WeaknessBreakEfficiencyRatio,
            p[0],
        )),
        ElationTemplate::FollowUpDamage => {
            effects.push(follow_up_modifier(PathEffectStat::DamageRatio, p[0]));
        }
        ElationTemplate::FollowUpCriticalRate => {
            effects.push(follow_up_modifier(PathEffectStat::CriticalRateRatio, p[0]))
        }
        ElationTemplate::FollowUpEnergyRegeneration => effects.push(follow_up_modifier(
            PathEffectStat::EnergyRegenerationRateRatio,
            p[0],
        )),
        ElationTemplate::FollowUpHealing => effects.push(PathEffect::HealMaximumHpRatio {
            target: PathEffectTarget::Actor,
            ratio: p[0],
        }),
        ElationTemplate::FollowUpDefense => effects.push(PathEffect::ApplyTimedStat {
            target: PathEffectTarget::Actor,
            stat: PathEffectStat::DefenseRatio,
            value: p[0],
            duration_turns: turns(p[1])?,
            maximum_stacks: 1,
        }),
        ElationTemplate::FollowUpSpeed => effects.push(PathEffect::ApplyTimedStat {
            target: PathEffectTarget::Actor,
            stat: PathEffectStat::SpeedRatio,
            value: p[0],
            duration_turns: turns(p[1])?,
            maximum_stacks: 1,
        }),
        ElationTemplate::ResonanceDamage => {
            effects.push(PathEffect::RandomElementFollowUpDamage {
                target: PathEffectTarget::AllEnemies,
                amount_per_hit: facts.path_base_damage,
                minimum_hits: turns(p[1])?,
                maximum_hits: turns(p[2])?,
            });
        }
        ElationTemplate::ResonanceSensoryPursuit => {
            effects.push(PathEffect::ApplySensoryPursuit {
                target: PathEffectTarget::AllEnemies,
                base_chance: PathEffectValue::from_raw_six_decimal(1_500_000),
                duration_turns: 1,
                follow_up_damage_taken_ratio: p[0],
            });
        }
        ElationTemplate::ResonanceVariableEnergy => {
            effects.push(PathEffect::ConfigureVariableResonanceEnergy {
                maximum: PathEffectValue::from_integral(200)?,
                consume_all_energy: true,
                excess_energy_ratio_per_extra_hit: p[4],
            });
        }
        ElationTemplate::ResonanceEnergyGain => {
            effects.push(PathEffect::ConfigureResonanceEnergyGain {
                battle_start_maximum_ratio: p[5],
                follow_up_attack_maximum_ratio: p[6],
            });
        }
    }
    Ok(effects
        .into_iter()
        .map(|effect| AppliedPathEffect::new(&program.source_key, effect))
        .collect::<Vec<_>>()
        .into_boxed_slice())
}

fn aftertaste_modifier(
    stat: PathEffectStat,
    value_per_element: PathEffectValue,
    facts: PathEffectFacts,
) -> Result<PathEffect, PathEffectRuntimeError> {
    Ok(PathEffect::ApplyAftertasteTypeModifier {
        target: PathEffectTarget::PrimaryEnemy,
        stat,
        value_per_element,
        element_count: u8::try_from(facts.aftertaste_element_count)
            .map_err(|_| PathEffectRuntimeError::InvalidFacts)?,
        until_end_of_next_action: true,
    })
}

fn follow_up_modifier(stat: PathEffectStat, value: PathEffectValue) -> PathEffect {
    PathEffect::AddFollowUpModifier { stat, value }
}

fn registry(key: &str) -> Result<(ElationTemplate, usize), PathEffectRuntimeError> {
    use ElationTemplate as T;
    Ok(match key {
        "StageAbility_612630" => (T::RandomAftertaste, 4),
        "StageAbility_612631" => (T::BrokenAftertaste, 2),
        "StageAbility_612632" => (T::UltimateAsFollowUp, 1),
        "StageAbility_612640" => (T::ExtraAftertaste, 2),
        "StageAbility_612641" => (T::AftertasteVulnerability, 1),
        "StageAbility_612642" => (T::AftertasteAttackReduction, 1),
        "StageAbility_612643" => (T::FollowUpTargetDamage, 1),
        "StageAbility_612644" => (T::FollowUpRamp, 1),
        "StageAbility_612645" => (T::FollowUpDelay, 6),
        "StageAbility_612646" => (T::FollowUpSkillPoint, 1),
        "StageAbility_612650" => (T::BlessingFollowUpDamage, 2),
        "StageAbility_612651" => (T::FollowUpBreakEfficiency, 1),
        "StageAbility_612652" => (T::FollowUpDamage, 1),
        "StageAbility_612653" => (T::FollowUpCriticalRate, 1),
        "StageAbility_612654" => (T::FollowUpEnergyRegeneration, 1),
        "StageAbility_612655" => (T::FollowUpHealing, 1),
        "StageAbility_612656" => (T::FollowUpDefense, 2),
        "StageAbility_612657" => (T::FollowUpSpeed, 2),
        "StageAbility_612620" => (T::ResonanceDamage, 7),
        "StageAbility_612621" => (T::ResonanceSensoryPursuit, 7),
        "StageAbility_612622" => (T::ResonanceVariableEnergy, 7),
        "StageAbility_612623" => (T::ResonanceEnergyGain, 7),
        _ => return Err(PathEffectRuntimeError::UnknownSource),
    })
}

fn catalog_digest(
    path: PathId,
    blessings: &[BlessingPrograms],
    resonances: &[ResonanceProgram],
) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock-universe-elation-runtime-catalog-v1");
    encoder.text(ELATION_RUNTIME_REVISION);
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
