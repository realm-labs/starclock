//! Closed executor for the released Standard Universe Nihility partition.

use crate::{
    catalog::UniverseCatalog,
    digest::Encoder,
    id::{BlessingId, PathId, ResonanceId},
    path_effect_runtime::{
        AppliedPathEffect, PathBattleEvent, PathDotSelection, PathEffect, PathEffectFacts,
        PathEffectRuntimeError, PathEffectStat, PathEffectTarget, PathEffectValue, count,
        exact_parameters, turns,
    },
};

pub const NIHILITY_RUNTIME_REVISION: &str = "standard-universe-nihility-runtime-v1";
const NIHILITY_PATH_KEY: &str = "universe.path.nihility";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
enum NihilityTemplate {
    DotSuspicion = 0,
    DotSuspicionPersistent = 1,
    AppliedDotSuspicion = 2,
    TurnStartDots = 3,
    DefeatedSuspicionSpread = 4,
    SuspicionAmplification = 5,
    SuspicionStatReduction = 6,
    WeaknessBreakEfficiency = 7,
    WeaknessBreakSpread = 8,
    BrokenEnemyRandomDot = 9,
    AttackTriggerRandomDot = 10,
    BlessingDotDamage = 11,
    BreakEffect = 12,
    EffectResistance = 13,
    DotDamageTaken = 14,
    ExtendDots = 15,
    DotCountDamageTaken = 16,
    DotHeal = 17,
    DotEnergy = 18,
    ResonanceDots = 19,
    ResonanceApplication = 20,
    ResonanceConfusionDevoid = 21,
    ResonanceEnergy = 22,
}

impl NihilityTemplate {
    const fn event(self) -> PathBattleEvent {
        match self {
            Self::DotSuspicion | Self::DotSuspicionPersistent | Self::DotHeal | Self::DotEnergy => {
                PathBattleEvent::DotDamageTaken
            }
            Self::AppliedDotSuspicion => PathBattleEvent::DotApplied,
            Self::TurnStartDots => PathBattleEvent::EnemyTurnStarted,
            Self::DefeatedSuspicionSpread => PathBattleEvent::EnemyDefeated,
            Self::SuspicionAmplification => PathBattleEvent::SuspicionApplying,
            Self::SuspicionStatReduction
            | Self::WeaknessBreakEfficiency
            | Self::BlessingDotDamage
            | Self::BreakEffect
            | Self::EffectResistance
            | Self::DotDamageTaken => PathBattleEvent::StatQueried,
            Self::WeaknessBreakSpread => PathBattleEvent::WeaknessBroken,
            Self::BrokenEnemyRandomDot | Self::AttackTriggerRandomDot => PathBattleEvent::AttackHit,
            Self::ExtendDots => PathBattleEvent::BattleStarted,
            Self::DotCountDamageTaken => PathBattleEvent::DamageCalculated,
            Self::ResonanceDots | Self::ResonanceApplication | Self::ResonanceConfusionDevoid => {
                PathBattleEvent::PathResonanceActivated
            }
            Self::ResonanceEnergy => PathBattleEvent::BattleStarted,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CompiledProgram {
    source_key: Box<str>,
    template: NihilityTemplate,
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
pub struct NihilityRuntimeCatalog {
    path: PathId,
    blessings: Box<[BlessingPrograms]>,
    resonances: Box<[ResonanceProgram]>,
    digest: [u8; 32],
}

impl NihilityRuntimeCatalog {
    pub fn compile(catalog: &UniverseCatalog) -> Result<Self, PathEffectRuntimeError> {
        let path = catalog
            .paths()
            .iter()
            .find(|path| path.stable_key() == NIHILITY_PATH_KEY)
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
        && !(program.template == NihilityTemplate::AppliedDotSuspicion
            && event == PathBattleEvent::DotRefreshed)
        && !(program.template == NihilityTemplate::ResonanceEnergy
            && event == PathBattleEvent::DotDamageTaken)
    {
        return Ok(Box::new([]));
    }
    let p = &program.parameters;
    let mut effects = Vec::with_capacity(2);
    match program.template {
        NihilityTemplate::DotSuspicion | NihilityTemplate::DotSuspicionPersistent => {
            effects.push(suspicion(
                turns(p[0])?,
                program.template == NihilityTemplate::DotSuspicionPersistent,
            ));
        }
        NihilityTemplate::AppliedDotSuspicion => {
            let stacks = if event == PathBattleEvent::DotRefreshed {
                turns(p[1])?
            } else {
                turns(p[0])?
            };
            if stacks != 0 {
                effects.push(suspicion(stacks, false));
            }
        }
        NihilityTemplate::TurnStartDots => effects.push(PathEffect::TriggerDots {
            target: PathEffectTarget::PrimaryEnemy,
            selection: PathDotSelection::All,
            times: turns(p[0])?,
            damage_ratio: p[1],
        }),
        NihilityTemplate::DefeatedSuspicionSpread => {
            if facts.suspicion_stacks != 0 {
                effects.push(PathEffect::SpreadSuspicion {
                    target: PathEffectTarget::RandomOtherEnemies,
                    target_count: turns(p[0])?,
                    stacks: u8::try_from(facts.suspicion_stacks.min(99))
                        .map_err(|_| PathEffectRuntimeError::InvalidFacts)?,
                });
            }
        }
        NihilityTemplate::SuspicionAmplification => {
            effects.push(PathEffect::ModifySuspicionApplication {
                extra_stacks: turns(p[0])?,
                multiplier: if p[1] == PathEffectValue::ONE { 2 } else { 1 },
            });
        }
        NihilityTemplate::SuspicionStatReduction => {
            let stacks = facts.suspicion_stacks;
            effects.push(capped_stat(
                PathEffectTarget::AllEnemies,
                PathEffectStat::AttackReductionRatio,
                p[0].checked_multiply_count(stacks)?,
                p[1],
            ));
            if p[2] != PathEffectValue::ZERO {
                effects.push(capped_stat(
                    PathEffectTarget::AllEnemies,
                    PathEffectStat::EffectResistanceReductionRatio,
                    p[2].checked_multiply_count(stacks)?,
                    p[3],
                ));
            }
        }
        NihilityTemplate::WeaknessBreakEfficiency => effects.push(stat(
            PathEffectTarget::AllAllies,
            PathEffectStat::WeaknessBreakEfficiencyRatio,
            p[0],
        )),
        NihilityTemplate::WeaknessBreakSpread => {
            effects.push(PathEffect::SpreadWeaknessBreak {
                target: if p[0] == PathEffectValue::ONE {
                    PathEffectTarget::AllEnemies
                } else {
                    PathEffectTarget::AdjacentEnemies
                },
            });
        }
        NihilityTemplate::BrokenEnemyRandomDot => {
            if facts.enemy_is_weakness_broken {
                effects.push(PathEffect::ApplyRandomBreakDot {
                    target: PathEffectTarget::PrimaryEnemy,
                    base_chance: p[0],
                    duration_turns: turns(p[1])?,
                    wind_shear_stacks: turns(p[2])?,
                    burn_shock_attack_ratio: p[3],
                    bleed_maximum_hp_ratio: p[4],
                    dispel_attacker_debuff: p[5] == PathEffectValue::ONE,
                });
            }
        }
        NihilityTemplate::AttackTriggerRandomDot => {
            if facts.enemy_has_dot {
                effects.push(PathEffect::TriggerDots {
                    target: PathEffectTarget::PrimaryEnemy,
                    selection: PathDotSelection::RandomOne,
                    times: 1,
                    damage_ratio: p[0],
                });
            }
        }
        NihilityTemplate::BlessingDotDamage => effects.push(stat(
            PathEffectTarget::AllAllies,
            PathEffectStat::DotDamageRatio,
            p[0].checked_multiply_count(facts.path_blessing_count.min(count(p[1])?))?,
        )),
        NihilityTemplate::BreakEffect => effects.push(stat(
            PathEffectTarget::AllAllies,
            PathEffectStat::BreakEffectRatio,
            p[0],
        )),
        NihilityTemplate::EffectResistance => effects.push(stat(
            PathEffectTarget::AllEnemies,
            PathEffectStat::EffectResistanceReductionRatio,
            p[0],
        )),
        NihilityTemplate::DotDamageTaken => effects.push(stat(
            PathEffectTarget::AllEnemies,
            PathEffectStat::DotDamageTakenRatio,
            p[0],
        )),
        NihilityTemplate::ExtendDots => effects.push(PathEffect::ExtendStandardDots {
            target: PathEffectTarget::AllEnemies,
            duration_turns: turns(p[0])?,
        }),
        NihilityTemplate::DotCountDamageTaken => effects.push(stat(
            PathEffectTarget::PrimaryEnemy,
            PathEffectStat::DamageTakenRatio,
            p[0].checked_multiply_count(facts.dot_count.min(count(p[1])?))?,
        )),
        NihilityTemplate::DotHeal => effects.push(PathEffect::HealMaximumHpRatio {
            target: PathEffectTarget::AllAllies,
            ratio: p[0],
        }),
        NihilityTemplate::DotEnergy => effects.push(PathEffect::GainEnergy {
            target: PathEffectTarget::RandomAlly,
            amount: p[0],
            once_per_action: false,
        }),
        NihilityTemplate::ResonanceDots => effects.push(PathEffect::ApplyResonanceDots {
            target: PathEffectTarget::AllEnemies,
            base_chance: p[0],
            duration_turns: turns(p[3])?,
            wind_shear_stacks: turns(p[4])?,
            burn_shock_attack_ratio: p[1],
            bleed_maximum_hp_ratio: p[2],
        }),
        NihilityTemplate::ResonanceApplication => {
            effects.push(PathEffect::ModifyResonanceDotApplication {
                base_chance_bonus: PathEffectValue::ONE,
                duration_bonus_turns: 1,
                stackable_status_bonus: 1,
            });
        }
        NihilityTemplate::ResonanceConfusionDevoid => {
            effects.push(PathEffect::ApplyConfusionAndDevoid {
                target: PathEffectTarget::AllEnemies,
                base_chance: PathEffectValue::ONE,
                confusion_stacks: 2,
                confusion_dot_trigger_ratio: p[5],
                devoid_stacks: 2,
                toughness_recovery_reduction_per_stack: p[6],
                duration_turns: turns(p[4])?,
            });
        }
        NihilityTemplate::ResonanceEnergy => effects.push(PathEffect::GainResonanceEnergy {
            maximum_ratio: if event == PathBattleEvent::BattleStarted {
                p[7]
            } else {
                p[8]
            },
        }),
    }
    Ok(effects
        .into_iter()
        .map(|effect| AppliedPathEffect::new(&program.source_key, effect))
        .collect::<Vec<_>>()
        .into_boxed_slice())
}

fn suspicion(stacks: u8, prevent_decay: bool) -> PathEffect {
    PathEffect::ApplySuspicion {
        target: PathEffectTarget::PrimaryEnemy,
        stacks,
        maximum_stacks: 99,
        dot_vulnerability_per_stack: PathEffectValue::from_raw_six_decimal(10_000),
        decay_per_turn: 2,
        prevent_decay,
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

fn capped_stat(
    target: PathEffectTarget,
    stat: PathEffectStat,
    value: PathEffectValue,
    cap: PathEffectValue,
) -> PathEffect {
    PathEffect::AddStat {
        target,
        stat,
        value,
        cap: Some(cap),
    }
}

fn registry(key: &str) -> Result<(NihilityTemplate, usize), PathEffectRuntimeError> {
    use NihilityTemplate as T;
    Ok(match key {
        "StageAbility_612230" => (T::DotSuspicion, 1),
        "StageAbility_612230_2" => (T::DotSuspicionPersistent, 1),
        "StageAbility_612231" => (T::AppliedDotSuspicion, 2),
        "StageAbility_612232" => (T::TurnStartDots, 2),
        "StageAbility_612240" => (T::DefeatedSuspicionSpread, 1),
        "StageAbility_612241" => (T::SuspicionAmplification, 2),
        "StageAbility_612242" => (T::SuspicionStatReduction, 4),
        "StageAbility_612243" => (T::WeaknessBreakEfficiency, 1),
        "StageAbility_612244" => (T::WeaknessBreakSpread, 1),
        "StageAbility_612245" => (T::BrokenEnemyRandomDot, 6),
        "StageAbility_612246" => (T::AttackTriggerRandomDot, 1),
        "StageAbility_612250" => (T::BlessingDotDamage, 2),
        "StageAbility_612251" => (T::BreakEffect, 1),
        "StageAbility_612252" => (T::EffectResistance, 1),
        "StageAbility_612253" => (T::DotDamageTaken, 1),
        "StageAbility_612254" => (T::ExtendDots, 1),
        "StageAbility_612255" => (T::DotCountDamageTaken, 2),
        "StageAbility_612256" => (T::DotHeal, 1),
        "StageAbility_612257" => (T::DotEnergy, 1),
        "StageAbility_612220" => (T::ResonanceDots, 9),
        "StageAbility_612221" => (T::ResonanceApplication, 9),
        "StageAbility_612222" => (T::ResonanceConfusionDevoid, 9),
        "StageAbility_612223" => (T::ResonanceEnergy, 9),
        _ => return Err(PathEffectRuntimeError::UnknownSource),
    })
}

fn catalog_digest(
    path: PathId,
    blessings: &[BlessingPrograms],
    resonances: &[ResonanceProgram],
) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock-universe-nihility-runtime-catalog-v1");
    encoder.text(NIHILITY_RUNTIME_REVISION);
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
