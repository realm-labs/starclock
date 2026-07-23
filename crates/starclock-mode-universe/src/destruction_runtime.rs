//! Closed executor for the released Standard Universe Destruction partition.

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

pub const DESTRUCTION_RUNTIME_REVISION: &str = "standard-universe-destruction-runtime-v1";
const DESTRUCTION_PATH_KEY: &str = "universe.path.destruction";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
enum DestructionTemplate {
    VirtualGrit = 0,
    GritOnHit = 1,
    DamageShare = 2,
    GritRetaliation = 3,
    AttackHpConsumption = 4,
    GritMitigation = 5,
    LostHpAttackDefense = 6,
    LowHpDamage = 7,
    LowHpHealing = 8,
    UltimateShield = 9,
    BlessingAttack = 10,
    DefeatPrevention = 11,
    MaximumHp = 12,
    HitEnergy = 13,
    EntryShield = 14,
    LowHpShield = 15,
    LostHpDefense = 16,
    LostHpEffectResistance = 17,
    ResonanceDamage = 18,
    ResonanceHpShield = 19,
    ResonanceEntropic = 20,
    ResonanceAutoActivation = 21,
}

impl DestructionTemplate {
    const fn event(self) -> PathBattleEvent {
        match self {
            Self::VirtualGrit
            | Self::LostHpAttackDefense
            | Self::LowHpDamage
            | Self::BlessingAttack
            | Self::MaximumHp
            | Self::LostHpDefense
            | Self::LostHpEffectResistance => PathBattleEvent::StatQueried,
            Self::GritOnHit
            | Self::GritRetaliation
            | Self::LowHpHealing
            | Self::HitEnergy
            | Self::LowHpShield
            | Self::ResonanceAutoActivation => PathBattleEvent::CharacterAttacked,
            Self::DamageShare | Self::GritMitigation => PathBattleEvent::DamageCalculated,
            Self::AttackHpConsumption => PathBattleEvent::AttackStarted,
            Self::UltimateShield => PathBattleEvent::UltimateUsed,
            Self::DefeatPrevention => PathBattleEvent::LethalDamageReceived,
            Self::EntryShield => PathBattleEvent::BattleStarted,
            Self::ResonanceDamage | Self::ResonanceHpShield | Self::ResonanceEntropic => {
                PathBattleEvent::PathResonanceActivated
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CompiledProgram {
    source_key: Box<str>,
    template: DestructionTemplate,
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
pub struct DestructionRuntimeCatalog {
    path: PathId,
    blessings: Box<[BlessingPrograms]>,
    resonances: Box<[ResonanceProgram]>,
    digest: [u8; 32],
}

impl DestructionRuntimeCatalog {
    pub fn compile(catalog: &UniverseCatalog) -> Result<Self, PathEffectRuntimeError> {
        let path = catalog
            .paths()
            .iter()
            .find(|path| path.stable_key() == DESTRUCTION_PATH_KEY)
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
    let alternate = matches!(
        program.template,
        DestructionTemplate::GritOnHit
            | DestructionTemplate::LowHpHealing
            | DestructionTemplate::HitEnergy
            | DestructionTemplate::LowHpShield
    ) && event == PathBattleEvent::HpLost
        || program.template == DestructionTemplate::GritOnHit
            && event == PathBattleEvent::TurnEnded;
    if program.template.event() != event && !alternate {
        return Ok(Box::new([]));
    }
    let p = &program.parameters;
    let mut effects = Vec::with_capacity(2);
    match program.template {
        DestructionTemplate::VirtualGrit => effects.push(PathEffect::ApplyVirtualGrit {
            target: PathEffectTarget::Actor,
            below_hp_ratio: p[5],
            base_stacks: turns(p[4])?,
            additional_hp_loss_interval_ratio: p.get(7).copied().unwrap_or(PathEffectValue::ZERO),
            additional_stacks_per_interval: p.get(8).copied().map(turns).transpose()?.unwrap_or(0),
            maximum_stacks: turns(p[2])?,
            attack_ratio_per_stack: p[0],
            defense_ratio_per_stack: p[1],
        }),
        DestructionTemplate::GritOnHit => {
            let stacks = if event == PathBattleEvent::TurnEnded {
                -signed_stacks(p[4])?
            } else {
                signed_stacks(p[5])?
            };
            effects.push(PathEffect::ModifyGrit {
                target: PathEffectTarget::Actor,
                stacks,
                adjacent_stacks: if event == PathBattleEvent::TurnEnded {
                    0
                } else {
                    p.get(6).copied().map(turns).transpose()?.unwrap_or(0)
                },
                maximum_stacks: turns(p[2])?,
                attack_ratio_per_stack: p[0],
                defense_ratio_per_stack: p[1],
                once_per_action: event != PathBattleEvent::TurnEnded,
            });
        }
        DestructionTemplate::DamageShare => effects.push(PathEffect::DistributeIncomingDamage {
            target: PathEffectTarget::AllAllies,
            damage_reduction_ratio: p.first().copied().unwrap_or(PathEffectValue::ZERO),
        }),
        DestructionTemplate::GritRetaliation => {
            let attack = facts
                .actor_base_attack
                .checked_multiply_ratio(p[0])?
                .checked_multiply_count(facts.grit_stacks)?;
            let loss = p
                .get(1)
                .copied()
                .map(|ratio| {
                    facts
                        .hp_lost
                        .checked_multiply_ratio(ratio)?
                        .checked_multiply_count(facts.grit_stacks)
                })
                .transpose()?
                .unwrap_or(PathEffectValue::ZERO);
            effects.push(PathEffect::RetaliateFromGrit {
                target: PathEffectTarget::Attacker,
                amount: attack.checked_add(loss)?,
                can_defeat: false,
            });
        }
        DestructionTemplate::AttackHpConsumption => {
            if facts.grit_stacks > 0 {
                let hp_loss = facts.actor_current_hp.checked_multiply_ratio(p[0])?;
                let ratio = p[1].checked_add(
                    p.get(2)
                        .copied()
                        .unwrap_or(PathEffectValue::ZERO)
                        .checked_multiply_count(facts.grit_stacks)?,
                )?;
                effects.push(PathEffect::ConsumeCurrentHpAndDamage {
                    target: PathEffectTarget::HitEnemies,
                    hp_cost_ratio: p[0],
                    damage_amount: hp_loss.checked_multiply_ratio(ratio)?,
                });
            }
        }
        DestructionTemplate::GritMitigation => {
            effects.push(PathEffect::AddStat {
                target: PathEffectTarget::Actor,
                stat: PathEffectStat::DamageTakenReductionRatio,
                value: p[0].checked_multiply_count(facts.grit_stacks)?,
                cap: None,
            });
            if let Some(maximum) = p.get(1).copied() {
                effects.push(PathEffect::SetGritMaximum {
                    maximum_stacks: turns(maximum)?,
                });
            }
        }
        DestructionTemplate::LostHpAttackDefense => {
            effects.push(lost_hp_stat(PathEffectStat::AttackRatio, p[0], facts)?);
            if let Some(defense) = p.get(1).copied() {
                effects.push(lost_hp_stat(PathEffectStat::DefenseRatio, defense, facts)?);
            }
        }
        DestructionTemplate::LowHpDamage => {
            let mut value = PathEffectValue::ZERO;
            if facts.actor_current_hp_ratio < p[0] {
                value = p[1];
            }
            if facts.actor_current_hp_ratio < p[2] {
                value = value.checked_add(p[3])?;
            }
            if value != PathEffectValue::ZERO {
                effects.push(stat(PathEffectStat::DamageRatio, value));
            }
        }
        DestructionTemplate::LowHpHealing => {
            if facts.actor_current_hp_ratio < p[0] {
                effects.push(PathEffect::HealMaximumHpRatioCappedPerAction {
                    target: PathEffectTarget::Actor,
                    ratio: p[1],
                    cap_ratio: p[2],
                });
            }
        }
        DestructionTemplate::UltimateShield => {
            let amount = facts.hp_lost.checked_multiply_ratio(p[0])?.checked_add(
                p.get(2)
                    .copied()
                    .map(|ratio| facts.actor_maximum_hp.checked_multiply_ratio(ratio))
                    .transpose()?
                    .unwrap_or(PathEffectValue::ZERO),
            )?;
            effects.push(shield(amount, turns(p[1])?));
        }
        DestructionTemplate::BlessingAttack => effects.push(stat(
            PathEffectStat::AttackRatio,
            p[0].checked_multiply_count(facts.path_blessing_count.min(count(p[1])?))?,
        )),
        DestructionTemplate::DefeatPrevention => {
            effects.push(PathEffect::PreventDefeatAndHeal {
                target: PathEffectTarget::Actor,
                heal_maximum_hp_ratio: p[0],
                maximum_team_triggers_per_battle: turns(p[1])?,
            });
        }
        DestructionTemplate::MaximumHp => {
            effects.push(stat(PathEffectStat::MaximumHpRatio, p[0]));
        }
        DestructionTemplate::HitEnergy => effects.push(PathEffect::GainEnergy {
            target: PathEffectTarget::Actor,
            amount: p[0],
            once_per_action: true,
        }),
        DestructionTemplate::EntryShield => effects.push(shield(
            facts.hp_lost.checked_multiply_ratio(p[0])?,
            turns(p[1])?,
        )),
        DestructionTemplate::LowHpShield => {
            if facts.actor_current_hp_ratio < p[0] {
                effects.push(PathEffect::ShieldOnLowHp {
                    target: PathEffectTarget::Actor,
                    trigger_below_hp_ratio: p[0],
                    maximum_hp_ratio: p[1],
                    duration_turns: turns(p[2])?,
                    maximum_triggers_per_character_per_battle: turns(p[3])?,
                });
            }
        }
        DestructionTemplate::LostHpDefense => {
            effects.push(lost_hp_stat(PathEffectStat::DefenseRatio, p[0], facts)?);
        }
        DestructionTemplate::LostHpEffectResistance => effects.push(lost_hp_stat(
            PathEffectStat::EffectResistanceRatio,
            p[0],
            facts,
        )?),
        DestructionTemplate::ResonanceDamage => effects.push(PathEffect::Damage {
            target: PathEffectTarget::AllEnemies,
            amount: facts.party_hp_lost.checked_multiply_ratio(p[0])?,
            kind: PathEffectDamageKind::PathResonance,
            element: PathEffectElement::Fire,
            can_defeat: true,
            force_critical: false,
            critical_damage_ratio: PathEffectValue::ZERO,
        }),
        DestructionTemplate::ResonanceHpShield => {
            effects.push(PathEffect::ConsumePartyHpForResonance {
                remaining_hp_ratio: p[2],
                resonance_damage_bonus_ratio: p[3],
                shield_duration_turns: turns(p[4])?,
            });
        }
        DestructionTemplate::ResonanceEntropic => {
            effects.push(PathEffect::ApplyEntropicRetribution {
                target: PathEffectTarget::AllEnemies,
                base_chance: PathEffectValue::from_raw_six_decimal(1_500_000),
                duration_turns: turns(p[5])?,
                defense_reduction_ratio: p[6],
                party_hp_lost_damage_ratio: p[7],
            });
        }
        DestructionTemplate::ResonanceAutoActivation => {
            if facts.actor_current_hp_ratio < p[8] {
                effects.push(PathEffect::AutoActivateResonance {
                    trigger_below_hp_ratio: p[8],
                    maximum_triggers_per_battle: turns(p[9])?,
                    cannot_repeat_for_same_attack: true,
                    consume_energy: false,
                });
            }
        }
    }
    Ok(effects
        .into_iter()
        .map(|effect| AppliedPathEffect::new(&program.source_key, effect))
        .collect::<Vec<_>>()
        .into_boxed_slice())
}

fn signed_stacks(value: PathEffectValue) -> Result<i8, PathEffectRuntimeError> {
    i8::try_from(turns(value)?).map_err(|_| PathEffectRuntimeError::InvalidParameter)
}

fn lost_hp_stat(
    stat: PathEffectStat,
    ratio_per_percent: PathEffectValue,
    facts: PathEffectFacts,
) -> Result<PathEffect, PathEffectRuntimeError> {
    Ok(stat_effect(
        stat,
        ratio_per_percent
            .checked_multiply_ratio(facts.actor_hp_lost_ratio)?
            .checked_multiply_count(100)?,
    ))
}

fn stat(stat: PathEffectStat, value: PathEffectValue) -> PathEffect {
    stat_effect(stat, value)
}

fn stat_effect(stat: PathEffectStat, value: PathEffectValue) -> PathEffect {
    PathEffect::AddStat {
        target: PathEffectTarget::Actor,
        stat,
        value,
        cap: None,
    }
}

fn shield(amount: PathEffectValue, duration_turns: u8) -> PathEffect {
    PathEffect::Shield {
        target: PathEffectTarget::Actor,
        amount,
        duration_turns,
        special: false,
        fixed_chance: PathEffectValue::ONE,
    }
}

fn registry(key: &str) -> Result<(DestructionTemplate, usize), PathEffectRuntimeError> {
    use DestructionTemplate as T;
    Ok(match key {
        "StageAbility_61253001" => (T::VirtualGrit, 7),
        "StageAbility_61253002" => (T::VirtualGrit, 9),
        "StageAbility_61253101" => (T::GritOnHit, 6),
        "StageAbility_61253102" => (T::GritOnHit, 7),
        "StageAbility_61253201" => (T::DamageShare, 0),
        "StageAbility_61253202" => (T::DamageShare, 1),
        "StageAbility_61254001" => (T::GritRetaliation, 1),
        "StageAbility_61254002" => (T::GritRetaliation, 2),
        "StageAbility_61254101" => (T::AttackHpConsumption, 2),
        "StageAbility_61254102" => (T::AttackHpConsumption, 3),
        "StageAbility_61254201" => (T::GritMitigation, 1),
        "StageAbility_61254202" => (T::GritMitigation, 2),
        "StageAbility_61254301" => (T::LostHpAttackDefense, 1),
        "StageAbility_61254302" => (T::LostHpAttackDefense, 2),
        "StageAbility_61254401" => (T::LowHpDamage, 4),
        "StageAbility_61254501" | "StageAbility_61254502" => (T::LowHpHealing, 3),
        "StageAbility_61254601" => (T::UltimateShield, 2),
        "StageAbility_61254602" => (T::UltimateShield, 3),
        "StageAbility_61255001" => (T::BlessingAttack, 2),
        "StageAbility_61255101" => (T::DefeatPrevention, 2),
        "StageAbility_61255201" => (T::MaximumHp, 1),
        "StageAbility_61255301" => (T::HitEnergy, 1),
        "StageAbility_61255401" => (T::EntryShield, 2),
        "StageAbility_61255501" => (T::LowHpShield, 4),
        "StageAbility_61255601" => (T::LostHpDefense, 1),
        "StageAbility_61255701" => (T::LostHpEffectResistance, 1),
        "StageAbility_612520" => (T::ResonanceDamage, 10),
        "StageAbility_612521" => (T::ResonanceHpShield, 10),
        "StageAbility_612522" => (T::ResonanceEntropic, 10),
        "StageAbility_612523" => (T::ResonanceAutoActivation, 10),
        _ => return Err(PathEffectRuntimeError::UnknownSource),
    })
}

fn catalog_digest(
    path: PathId,
    blessings: &[BlessingPrograms],
    resonances: &[ResonanceProgram],
) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock-universe-destruction-runtime-catalog-v1");
    encoder.text(DESTRUCTION_RUNTIME_REVISION);
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
