//! Closed executor for the released Standard Universe Abundance partition.

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

pub const ABUNDANCE_RUNTIME_REVISION: &str = "standard-universe-abundance-runtime-v1";
const ABUNDANCE_PATH_KEY: &str = "universe.path.abundance";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
enum AbundanceTemplate {
    HealingDewdrop = 0,
    TurnDewdrop = 1,
    SharedHealing = 2,
    DewdropRuptureHealing = 3,
    FullHpDewdropEfficiency = 4,
    DewdropDispel = 5,
    HealingAttack = 6,
    HpAdditionalDamage = 7,
    FullHpDefense = 8,
    AllyHealingBonus = 9,
    BlessingMaximumHp = 10,
    HealingReceived = 11,
    EntryHealing = 12,
    BreakHealing = 13,
    HealedDefense = 14,
    ProviderHealing = 15,
    HealedSpeed = 16,
    HealingSkillPoint = 17,
    ResonanceHealing = 18,
    ResonancePreventDefeat = 19,
    ResonanceSubduingEvils = 20,
    ResonanceAction = 21,
}

impl AbundanceTemplate {
    const fn event(self) -> PathBattleEvent {
        match self {
            Self::HealingDewdrop
            | Self::AllyHealingBonus
            | Self::HealedDefense
            | Self::HealedSpeed => PathBattleEvent::HealingReceived,
            Self::TurnDewdrop => PathBattleEvent::TurnStarted,
            Self::SharedHealing
            | Self::HealingAttack
            | Self::ProviderHealing
            | Self::HealingSkillPoint => PathBattleEvent::HealingProvided,
            Self::DewdropRuptureHealing | Self::DewdropDispel => PathBattleEvent::DewdropRuptured,
            Self::FullHpDewdropEfficiency
            | Self::FullHpDefense
            | Self::BlessingMaximumHp
            | Self::HealingReceived => PathBattleEvent::StatQueried,
            Self::HpAdditionalDamage => PathBattleEvent::AttackCompleted,
            Self::EntryHealing => PathBattleEvent::BattleStarted,
            Self::BreakHealing => PathBattleEvent::WeaknessBroken,
            Self::ResonanceHealing | Self::ResonanceSubduingEvils | Self::ResonanceAction => {
                PathBattleEvent::PathResonanceActivated
            }
            Self::ResonancePreventDefeat => PathBattleEvent::LethalDamageReceived,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CompiledProgram {
    source_key: Box<str>,
    template: AbundanceTemplate,
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
pub struct AbundanceRuntimeCatalog {
    path: PathId,
    blessings: Box<[BlessingPrograms]>,
    resonances: Box<[ResonanceProgram]>,
    digest: [u8; 32],
}

impl AbundanceRuntimeCatalog {
    pub fn compile(catalog: &UniverseCatalog) -> Result<Self, PathEffectRuntimeError> {
        let path = catalog
            .paths()
            .iter()
            .find(|path| path.stable_key() == ABUNDANCE_PATH_KEY)
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
    if program.template.event() != event {
        return Ok(Box::new([]));
    }
    let p = &program.parameters;
    let mut effects = Vec::with_capacity(2);
    match program.template {
        AbundanceTemplate::HealingDewdrop => effects.push(PathEffect::ChargeDewdrop {
            target: PathEffectTarget::Actor,
            amount: facts.healing_amount.checked_multiply_ratio(p[0])?,
            maximum_hp_cap_ratio: PathEffectValue::ONE,
            damage_bonus_ratio: p[1],
            ruptures_after_attack: true,
        }),
        AbundanceTemplate::TurnDewdrop => {
            let basis = match turns(p[1])? {
                1 => facts.actor_current_hp,
                2 => facts.actor_maximum_hp,
                _ => return Err(PathEffectRuntimeError::InvalidParameter),
            };
            effects.push(PathEffect::ChargeDewdrop {
                target: PathEffectTarget::Actor,
                amount: basis.checked_multiply_ratio(p[0])?,
                maximum_hp_cap_ratio: PathEffectValue::ONE,
                damage_bonus_ratio: PathEffectValue::ZERO,
                ruptures_after_attack: true,
            });
        }
        AbundanceTemplate::SharedHealing => {
            effects.push(PathEffect::HealAmount {
                target: PathEffectTarget::OtherAllies,
                amount: facts.healing_amount.checked_multiply_ratio(p[0])?,
                once_per_action: false,
            });
            if p[1] != PathEffectValue::ZERO {
                effects.push(PathEffect::ScaleAttackFromHealing {
                    target: PathEffectTarget::AllAllies,
                    healing_ratio: p[1],
                    base_attack_cap_ratio: p[2],
                    until_next_turn_end: true,
                });
            }
        }
        AbundanceTemplate::DewdropRuptureHealing => {
            let raw = facts.dewdrop_charge.checked_multiply_ratio(p[0])?;
            let maximum = facts.actor_maximum_hp.checked_multiply_ratio(p[1])?;
            let minimum = facts.actor_maximum_hp.checked_multiply_ratio(p[2])?;
            effects.push(PathEffect::HealAmount {
                target: PathEffectTarget::Actor,
                amount: raw.max(minimum).min(maximum),
                once_per_action: false,
            });
        }
        AbundanceTemplate::FullHpDewdropEfficiency => {
            if facts.actor_is_full_hp {
                effects.push(PathEffect::ModifyDewdropChargeEfficiency {
                    target: PathEffectTarget::Actor,
                    value: p[0],
                });
            }
        }
        AbundanceTemplate::DewdropDispel => effects.push(PathEffect::DispelDebuff {
            target: PathEffectTarget::Actor,
            fixed_chance: p[0],
            count: 1,
        }),
        AbundanceTemplate::HealingAttack => effects.push(PathEffect::ApplyTimedStat {
            target: if p[2] == PathEffectValue::ONE {
                PathEffectTarget::AllAllies
            } else {
                PathEffectTarget::HealerAndHealed
            },
            stat: PathEffectStat::AttackRatio,
            value: p[0],
            duration_turns: turns(p[1])?,
            maximum_stacks: 1,
        }),
        AbundanceTemplate::HpAdditionalDamage => {
            let basis = if p[1] == PathEffectValue::ONE {
                facts.actor_maximum_hp
            } else {
                facts.actor_current_hp
            };
            effects.push(PathEffect::Damage {
                target: PathEffectTarget::HitEnemies,
                amount: basis.checked_multiply_ratio(p[0])?,
                kind: PathEffectDamageKind::PathAdditional,
                element: PathEffectElement::InheritActor,
                can_defeat: true,
                force_critical: false,
                critical_damage_ratio: PathEffectValue::ZERO,
            });
        }
        AbundanceTemplate::FullHpDefense => {
            if facts.actor_is_full_hp {
                effects.push(stat(
                    PathEffectTarget::Actor,
                    PathEffectStat::DamageTakenReductionRatio,
                    p[0],
                ));
                if p[1] != PathEffectValue::ZERO {
                    effects.push(stat(
                        PathEffectTarget::Actor,
                        PathEffectStat::EffectResistanceRatio,
                        p[1],
                    ));
                }
            }
        }
        AbundanceTemplate::AllyHealingBonus => {
            if facts.healing_was_from_ally {
                effects.push(PathEffect::HealAmount {
                    target: PathEffectTarget::Actor,
                    amount: facts.healing_amount.checked_multiply_ratio(p[0])?,
                    once_per_action: false,
                });
            }
        }
        AbundanceTemplate::BlessingMaximumHp => effects.push(stat(
            PathEffectTarget::AllAllies,
            PathEffectStat::MaximumHpRatio,
            p[0].checked_multiply_count(facts.path_blessing_count.min(count(p[1])?))?,
        )),
        AbundanceTemplate::HealingReceived => effects.push(stat(
            PathEffectTarget::AllAllies,
            PathEffectStat::HealingReceivedRatio,
            p[0],
        )),
        AbundanceTemplate::EntryHealing => effects.push(PathEffect::HealMaximumHpRatio {
            target: PathEffectTarget::AllAllies,
            ratio: p[0],
        }),
        AbundanceTemplate::BreakHealing => effects.push(PathEffect::HealMaximumHpRatio {
            target: PathEffectTarget::Actor,
            ratio: p[0],
        }),
        AbundanceTemplate::HealedDefense => effects.push(timed_stat(
            PathEffectTarget::Actor,
            PathEffectStat::DefenseRatio,
            p[0],
            turns(p[1])?,
        )),
        AbundanceTemplate::ProviderHealing => effects.push(PathEffect::HealAmount {
            target: PathEffectTarget::Actor,
            amount: facts.actor_maximum_hp.checked_multiply_ratio(p[0])?,
            once_per_action: true,
        }),
        AbundanceTemplate::HealedSpeed => effects.push(timed_stat(
            PathEffectTarget::Actor,
            PathEffectStat::SpeedRatio,
            p[0],
            turns(p[1])?,
        )),
        AbundanceTemplate::HealingSkillPoint => effects.push(PathEffect::GainSkillPoint {
            fixed_chance: p[0],
            amount: 1,
            once_per_action: true,
        }),
        AbundanceTemplate::ResonanceHealing => {
            effects.push(PathEffect::HealMaximumHpRatio {
                target: PathEffectTarget::AllAllies,
                ratio: p[0],
            });
            effects.push(timed_stat(
                PathEffectTarget::AllAllies,
                PathEffectStat::MaximumHpRatio,
                p[1],
                turns(p[2])?,
            ));
        }
        AbundanceTemplate::ResonancePreventDefeat => {
            effects.push(PathEffect::PreventDefeatAndActivateResonance {
                target: PathEffectTarget::Actor,
                maximum_triggers_per_battle: turns(p[8])?,
                consume_all_energy: true,
            });
        }
        AbundanceTemplate::ResonanceSubduingEvils => {
            effects.push(PathEffect::DispelDebuff {
                target: PathEffectTarget::AllAllies,
                fixed_chance: PathEffectValue::ONE,
                count: u8::MAX,
            });
            effects.push(PathEffect::ApplySubduingEvils {
                target: PathEffectTarget::AllAllies,
                stacks: turns(p[3])?,
                maximum_stacks: turns(p[6])?,
                duration_turns: turns(p[4])?,
                blocked_debuffs_per_stack: 1,
                heal_maximum_hp_ratio_on_block: p[5],
            });
        }
        AbundanceTemplate::ResonanceAction => {
            effects.push(PathEffect::InstallResonanceAction {
                healing_reduction_ratio: p[7],
                activate_after_first_manual_use: true,
            });
        }
    }
    Ok(effects
        .into_iter()
        .map(|effect| AppliedPathEffect::new(&program.source_key, effect))
        .collect::<Vec<_>>()
        .into_boxed_slice())
}

fn stat(target: PathEffectTarget, stat: PathEffectStat, value: PathEffectValue) -> PathEffect {
    PathEffect::AddStat {
        target,
        stat,
        value,
        cap: None,
    }
}

fn timed_stat(
    target: PathEffectTarget,
    stat: PathEffectStat,
    value: PathEffectValue,
    duration_turns: u8,
) -> PathEffect {
    PathEffect::ApplyTimedStat {
        target,
        stat,
        value,
        duration_turns,
        maximum_stacks: 1,
    }
}

fn registry(key: &str) -> Result<(AbundanceTemplate, usize), PathEffectRuntimeError> {
    use AbundanceTemplate as T;
    Ok(match key {
        "StageAbility_612330" => (T::HealingDewdrop, 2),
        "StageAbility_612331" => (T::TurnDewdrop, 2),
        "StageAbility_612332" => (T::SharedHealing, 3),
        "StageAbility_612340" => (T::DewdropRuptureHealing, 3),
        "StageAbility_612341" => (T::FullHpDewdropEfficiency, 1),
        "StageAbility_612342" => (T::DewdropDispel, 1),
        "StageAbility_612343" => (T::HealingAttack, 3),
        "StageAbility_612344" => (T::HpAdditionalDamage, 2),
        "StageAbility_612345" => (T::FullHpDefense, 2),
        "StageAbility_612346" => (T::AllyHealingBonus, 1),
        "StageAbility_612350" => (T::BlessingMaximumHp, 2),
        "StageAbility_612351" => (T::HealingReceived, 1),
        "StageAbility_612352" => (T::EntryHealing, 1),
        "StageAbility_612353" => (T::BreakHealing, 1),
        "StageAbility_612354" => (T::HealedDefense, 2),
        "StageAbility_612355" => (T::ProviderHealing, 1),
        "StageAbility_612356" => (T::HealedSpeed, 2),
        "StageAbility_612357" => (T::HealingSkillPoint, 1),
        "StageAbility_612320" => (T::ResonanceHealing, 9),
        "StageAbility_612321" => (T::ResonancePreventDefeat, 9),
        "StageAbility_612322" => (T::ResonanceSubduingEvils, 9),
        "StageAbility_612323" => (T::ResonanceAction, 9),
        _ => return Err(PathEffectRuntimeError::UnknownSource),
    })
}

fn catalog_digest(
    path: PathId,
    blessings: &[BlessingPrograms],
    resonances: &[ResonanceProgram],
) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock-universe-abundance-runtime-catalog-v1");
    encoder.text(ABUNDANCE_RUNTIME_REVISION);
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
