//! Closed executor for the released Standard Universe Hunt partition.

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

pub const HUNT_RUNTIME_REVISION: &str = "standard-universe-hunt-runtime-v1";
const HUNT_PATH_KEY: &str = "universe.path.hunt";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
enum HuntTemplate {
    TurnCriticalBoost = 0,
    DefeatAdvanceBoost = 1,
    BreakAdvance = 2,
    ExcessCriticalDamage = 3,
    CriticalBoostTurnHealing = 4,
    CriticalBoostTurnUltimateHealing = 5,
    InheritUltimateBoost = 6,
    InheritUltimateFollowUpBoost = 7,
    ConsecutiveActionAttack = 8,
    ConsecutiveActionAttackSkillPoint = 9,
    DefeatEnergy = 10,
    TurnAdvanceCounter = 11,
    DefeatHealing = 12,
    DefeatBreakHealing = 13,
    BlessingSpeed = 14,
    CriticalRate = 15,
    CriticalDamage = 16,
    BreakDelay = 17,
    EntrySpeed = 18,
    TurnEndAdvance = 19,
    TurnEnergy = 20,
    LastAllyAttack = 21,
    ResonanceDamage = 22,
    ResonanceArrow = 23,
    ResonanceCritical = 24,
    ResonanceEnergy = 25,
}

impl HuntTemplate {
    const fn event(self) -> PathBattleEvent {
        match self {
            Self::TurnCriticalBoost
            | Self::CriticalBoostTurnHealing
            | Self::TurnAdvanceCounter
            | Self::TurnEnergy
            | Self::LastAllyAttack => PathBattleEvent::TurnStarted,
            Self::DefeatAdvanceBoost | Self::DefeatEnergy | Self::DefeatHealing => {
                PathBattleEvent::EnemyDefeated
            }
            Self::BreakAdvance | Self::BreakDelay => PathBattleEvent::WeaknessBroken,
            Self::ExcessCriticalDamage
            | Self::BlessingSpeed
            | Self::CriticalRate
            | Self::CriticalDamage => PathBattleEvent::StatQueried,
            Self::CriticalBoostTurnUltimateHealing | Self::InheritUltimateBoost => {
                PathBattleEvent::UltimateUsed
            }
            Self::InheritUltimateFollowUpBoost => PathBattleEvent::UltimateUsed,
            Self::ConsecutiveActionAttack | Self::ConsecutiveActionAttackSkillPoint => {
                PathBattleEvent::ConsecutiveActionStarted
            }
            Self::DefeatBreakHealing => PathBattleEvent::EnemyDefeated,
            Self::EntrySpeed => PathBattleEvent::BattleStarted,
            Self::TurnEndAdvance => PathBattleEvent::TurnEnded,
            Self::ResonanceDamage
            | Self::ResonanceArrow
            | Self::ResonanceCritical
            | Self::ResonanceEnergy => PathBattleEvent::PathResonanceActivated,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CompiledProgram {
    source_key: Box<str>,
    template: HuntTemplate,
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
pub struct HuntRuntimeCatalog {
    path: PathId,
    blessings: Box<[BlessingPrograms]>,
    resonances: Box<[ResonanceProgram]>,
    digest: [u8; 32],
}

impl HuntRuntimeCatalog {
    pub fn compile(catalog: &UniverseCatalog) -> Result<Self, PathEffectRuntimeError> {
        let path = catalog
            .paths()
            .iter()
            .find(|path| path.stable_key() == HUNT_PATH_KEY)
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
    let alternate = matches!(program.template, HuntTemplate::InheritUltimateFollowUpBoost)
        && event == PathBattleEvent::FollowUpAttackUsed
        || matches!(
            program.template,
            HuntTemplate::CriticalBoostTurnUltimateHealing
        ) && event == PathBattleEvent::TurnStarted
        || matches!(program.template, HuntTemplate::DefeatBreakHealing)
            && event == PathBattleEvent::WeaknessBroken;
    if program.template.event() != event && !alternate {
        return Ok(Box::new([]));
    }
    let p = &program.parameters;
    let mut effects = Vec::with_capacity(3);
    match program.template {
        HuntTemplate::TurnCriticalBoost => effects.push(critical_boost(p, turns(p[2])?, false)?),
        HuntTemplate::DefeatAdvanceBoost => {
            effects.push(advance(PathEffectTarget::Actor, PathEffectValue::ONE, true));
            effects.push(critical_boost(p, turns(p[2])?, true)?);
        }
        HuntTemplate::BreakAdvance => {
            effects.push(advance(
                PathEffectTarget::Actor,
                PathEffectValue::ONE,
                false,
            ));
            effects.push(PathEffect::IncreaseNextAttackDamage {
                target: PathEffectTarget::Actor,
                ratio: p[0],
            });
            if p[1] == PathEffectValue::ONE && facts.weakness_broken_enemy_is_elite {
                effects.push(advance(
                    PathEffectTarget::AllAllies,
                    PathEffectValue::ONE,
                    false,
                ));
            }
        }
        HuntTemplate::ExcessCriticalDamage => {
            effects.push(PathEffect::CriticalDamageFromExcessRate {
                target: PathEffectTarget::Actor,
                excess_rate_multiplier: p[0],
                per_critical_boost_stack: p.get(2).copied().unwrap_or(PathEffectValue::ZERO),
                cap: p[1],
            })
        }
        HuntTemplate::CriticalBoostTurnHealing | HuntTemplate::CriticalBoostTurnUltimateHealing => {
            effects.push(PathEffect::HealMaximumHpRatio {
                target: PathEffectTarget::Actor,
                ratio: p[0].checked_multiply_count(facts.critical_boost_stacks)?,
            })
        }
        HuntTemplate::InheritUltimateBoost | HuntTemplate::InheritUltimateFollowUpBoost => {
            effects.push(PathEffect::InheritCriticalBoost {
                target: PathEffectTarget::Actor,
                extra_stacks: turns(p[0])?,
                maximum_stacks: turns(p[1])?,
            });
        }
        HuntTemplate::ConsecutiveActionAttack | HuntTemplate::ConsecutiveActionAttackSkillPoint => {
            effects.push(PathEffect::ApplyTimedStat {
                target: PathEffectTarget::Actor,
                stat: PathEffectStat::AttackRatio,
                value: p[0],
                duration_turns: 1,
                maximum_stacks: turns(p[1])?,
            });
            if program.template == HuntTemplate::ConsecutiveActionAttackSkillPoint {
                effects.push(PathEffect::GainSkillPoint {
                    fixed_chance: p[2],
                    amount: 1,
                    once_per_action: true,
                });
            }
        }
        HuntTemplate::DefeatEnergy => effects.push(PathEffect::GainEnergyMaximumRatio {
            target: PathEffectTarget::Actor,
            ratio: p[0],
        }),
        HuntTemplate::TurnAdvanceCounter => effects.push(PathEffect::ConfigureTurnAdvanceCounter {
            target: PathEffectTarget::Actor,
            turn_interval: turns(p[0])?,
            initial_turns: p.get(1).copied().map(turns).transpose()?.unwrap_or(0),
            cannot_repeat_for_same_actor: true,
        }),
        HuntTemplate::DefeatHealing | HuntTemplate::DefeatBreakHealing => {
            let ratio = if event == PathBattleEvent::WeaknessBroken {
                p[1]
            } else {
                p[0]
            };
            effects.push(PathEffect::HealMaximumHpRatio {
                target: PathEffectTarget::Actor,
                ratio,
            });
        }
        HuntTemplate::BlessingSpeed => effects.push(stat(
            PathEffectStat::SpeedRatio,
            p[0].checked_multiply_count(facts.path_blessing_count.min(count(p[1])?))?,
        )),
        HuntTemplate::CriticalRate => effects.push(stat(PathEffectStat::CriticalRateRatio, p[0])),
        HuntTemplate::CriticalDamage => {
            effects.push(stat(PathEffectStat::CriticalDamageRatio, p[0]))
        }
        HuntTemplate::BreakDelay => effects.push(PathEffect::DelayAction {
            target: PathEffectTarget::PrimaryEnemy,
            ratio: p[0],
        }),
        HuntTemplate::EntrySpeed => effects.push(PathEffect::ApplyUntilAttackedStat {
            target: PathEffectTarget::AllAllies,
            stat: PathEffectStat::SpeedRatio,
            value: p[0],
        }),
        HuntTemplate::TurnEndAdvance => effects.push(advance(PathEffectTarget::Actor, p[0], false)),
        HuntTemplate::TurnEnergy => effects.push(PathEffect::GainEnergy {
            target: PathEffectTarget::Actor,
            amount: p[0],
            once_per_action: false,
        }),
        HuntTemplate::LastAllyAttack => effects.push(PathEffect::ScaleAttackFromLastAlly {
            target: PathEffectTarget::Actor,
            source_attack: facts.last_acting_ally_attack,
            ratio: p[0],
            until_next_turn_start: true,
        }),
        HuntTemplate::ResonanceDamage => effects.push(PathEffect::Damage {
            target: PathEffectTarget::AllEnemies,
            amount: facts.highest_ally_attack.checked_multiply_ratio(p[1])?,
            kind: PathEffectDamageKind::PathResonance,
            element: PathEffectElement::Wind,
            can_defeat: true,
            force_critical: false,
            critical_damage_ratio: PathEffectValue::ZERO,
        }),
        HuntTemplate::ResonanceArrow => {
            effects.push(advance(
                PathEffectTarget::HighestAttackAlly,
                PathEffectValue::ONE,
                false,
            ));
            effects.push(PathEffect::ApplyLightHuntingCelestialArrow {
                target: PathEffectTarget::HighestAttackAlly,
                critical_damage_from_critical_rate_ratio: p[0],
                extra_turn_after_defeat: true,
                cannot_repeat: true,
                expires_after_ability: true,
            });
        }
        HuntTemplate::ResonanceCritical => effects.push(PathEffect::ModifyResonanceCritical {
            guaranteed_critical_below_hp_ratio: p[2],
            critical_damage_ratio: p[4],
            defeated_energy_maximum_ratio: p[5],
        }),
        HuntTemplate::ResonanceEnergy => effects.push(PathEffect::ConfigureResonanceEnergy {
            maximum: PathEffectValue::from_integral(200)?,
            gain_on_ally_turn_ratio: p[6],
        }),
    }
    Ok(effects
        .into_iter()
        .map(|effect| AppliedPathEffect::new(&program.source_key, effect))
        .collect::<Vec<_>>()
        .into_boxed_slice())
}

fn critical_boost(
    p: &[PathEffectValue],
    stacks: u8,
    at_next_turn_start: bool,
) -> Result<PathEffect, PathEffectRuntimeError> {
    Ok(PathEffect::ApplyCriticalBoost {
        target: PathEffectTarget::Actor,
        stacks,
        maximum_stacks: turns(p[3])?,
        critical_rate_ratio_per_stack: p[0],
        critical_damage_ratio_per_stack: p[1],
        at_next_turn_start,
    })
}

fn advance(
    target: PathEffectTarget,
    ratio: PathEffectValue,
    cannot_repeat_for_same_actor: bool,
) -> PathEffect {
    PathEffect::ActionAdvance {
        target,
        ratio,
        cannot_repeat_for_same_actor,
    }
}

fn stat(stat: PathEffectStat, value: PathEffectValue) -> PathEffect {
    PathEffect::AddStat {
        target: PathEffectTarget::AllAllies,
        stat,
        value,
        cap: None,
    }
}

fn registry(key: &str) -> Result<(HuntTemplate, usize), PathEffectRuntimeError> {
    use HuntTemplate as T;
    Ok(match key {
        "StageAbility_61243001" | "StageAbility_61243002" => (T::TurnCriticalBoost, 4),
        "StageAbility_61243101" => (T::DefeatAdvanceBoost, 4),
        "StageAbility_61243201" => (T::BreakAdvance, 2),
        "StageAbility_61244001" | "StageAbility_61244002" => (
            T::ExcessCriticalDamage,
            if key.ends_with("02") { 3 } else { 2 },
        ),
        "StageAbility_61244101" => (T::CriticalBoostTurnHealing, 1),
        "StageAbility_61244102" => (T::CriticalBoostTurnUltimateHealing, 1),
        "StageAbility_61244201" => (T::InheritUltimateBoost, 4),
        "StageAbility_61244202" => (T::InheritUltimateFollowUpBoost, 4),
        "StageAbility_61244301" => (T::ConsecutiveActionAttack, 2),
        "StageAbility_61244302" => (T::ConsecutiveActionAttackSkillPoint, 3),
        "StageAbility_61244401" | "StageAbility_61244402" => {
            (T::DefeatEnergy, if key.ends_with("02") { 2 } else { 1 })
        }
        "StageAbility_61244501" | "StageAbility_61244502" => (
            T::TurnAdvanceCounter,
            if key.ends_with("02") { 2 } else { 1 },
        ),
        "StageAbility_61244601" => (T::DefeatHealing, 1),
        "StageAbility_61244602" => (T::DefeatBreakHealing, 2),
        "StageAbility_61245001" => (T::BlessingSpeed, 2),
        "StageAbility_61245101" => (T::CriticalRate, 1),
        "StageAbility_61245201" => (T::CriticalDamage, 1),
        "StageAbility_61245301" => (T::BreakDelay, 1),
        "StageAbility_61245401" => (T::EntrySpeed, 1),
        "StageAbility_61245501" => (T::TurnEndAdvance, 1),
        "StageAbility_61245601" => (T::TurnEnergy, 1),
        "StageAbility_61245701" => (T::LastAllyAttack, 1),
        "StageAbility_612420" => (T::ResonanceDamage, 7),
        "StageAbility_612421" => (T::ResonanceArrow, 7),
        "StageAbility_612422" => (T::ResonanceCritical, 7),
        "StageAbility_612423" => (T::ResonanceEnergy, 7),
        _ => return Err(PathEffectRuntimeError::UnknownSource),
    })
}

fn catalog_digest(
    path: PathId,
    blessings: &[BlessingPrograms],
    resonances: &[ResonanceProgram],
) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock-universe-hunt-runtime-catalog-v1");
    encoder.text(HUNT_RUNTIME_REVISION);
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
