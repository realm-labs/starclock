//! Closed executor for negative Curios, Error Codes and replacement effects.

use crate::{
    curio::CurioStateKind,
    curio_effect_runtime::{AppliedCurioEffect, CurioEffect, CurioEnergyChange, CurioHpChange},
    curio_runtime::{CurioContribution, CurioRuntimeCatalog},
    digest::Encoder,
    id::{CurioId, CurioStateId},
    path_effect_runtime::{
        PathEffect, PathEffectStat, PathEffectTarget, PathEffectValue, exact_parameters, turns,
    },
};

pub const NEGATIVE_CURIO_RUNTIME_REVISION: &str = "standard-universe-negative-curio-runtime-v1";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum NegativeCurioEvent {
    Acquired = 0,
    BattleWon = 1,
    BlessingRewardOffered = 2,
    DomainEntered = 3,
    BattleStarted = 4,
    WeaknessBroken = 5,
    UltimateUsed = 6,
    DamageTakenCalculated = 7,
    SkillUsed = 8,
    EnemyDefeated = 9,
    BasicAttackUsed = 10,
    ActionEnded = 11,
    BlessingServicePriced = 12,
    StatQueried = 13,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
enum NegativeCurioTemplate {
    RepairDestroyedCurios = 0,
    ReplaceOwnedCurios = 1,
    ErrorEnergy = 2,
    ErrorHp = 3,
    ErrorDamageTaken = 4,
    ErrorActionAdvance = 5,
    ErrorDamageDealt = 6,
    ErrorSkillPoints = 7,
    PostActionAdvance = 8,
    Parasitized = 9,
    FragmentDebt = 10,
    ReducedBlessingOffers = 11,
    MajorAggro = 12,
    DomainFragmentLoss = 13,
    BlessingServiceInflation = 14,
    EntrySkillPointLoss = 15,
    Fission = 16,
    ReplaceBlessings = 17,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CompiledNegativeCurioProgram {
    curio: CurioId,
    state: CurioStateId,
    kind: CurioStateKind,
    source_key: Box<str>,
    source_effect_id: Box<str>,
    template: NegativeCurioTemplate,
    parameters: Box<[PathEffectValue]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NegativeCurioRuntimeCatalog {
    programs: Box<[CompiledNegativeCurioProgram]>,
    digest: [u8; 32],
}

impl NegativeCurioRuntimeCatalog {
    pub fn compile(runtime: &CurioRuntimeCatalog) -> Result<Self, NegativeCurioRuntimeError> {
        let mut programs = Vec::new();
        for definition in runtime.definitions() {
            for state in definition.states() {
                let Some((template, arity)) = registry(state.source_effect_id()) else {
                    continue;
                };
                if state.parameters().len() != arity {
                    return Err(NegativeCurioRuntimeError::InvalidDefinition);
                }
                programs.push(CompiledNegativeCurioProgram {
                    curio: definition.curio(),
                    state: state.id(),
                    kind: state.kind(),
                    source_key: format!(
                        "{}.state.{}",
                        definition.stable_key(),
                        state_suffix(state.kind())
                    )
                    .into(),
                    source_effect_id: state.source_effect_id().into(),
                    template,
                    parameters: exact_parameters(state.parameters())
                        .map_err(|_| NegativeCurioRuntimeError::InvalidParameter)?,
                });
            }
        }
        programs.sort_by_key(|program| (program.curio, program.state));
        let mut curios = programs
            .iter()
            .map(|program| program.curio)
            .collect::<Vec<_>>();
        curios.dedup();
        if programs.len() != 24
            || curios.len() != 18
            || programs
                .windows(2)
                .any(|pair| (pair[0].curio, pair[0].state) == (pair[1].curio, pair[1].state))
            || !programs.iter().all(valid_kind)
        {
            return Err(NegativeCurioRuntimeError::InvalidDenominator);
        }
        let digest = catalog_digest(&programs);
        Ok(Self {
            programs: programs.into_boxed_slice(),
            digest,
        })
    }

    #[must_use]
    pub const fn content_count(&self) -> usize {
        42
    }

    #[must_use]
    pub const fn rule_count(&self) -> usize {
        42
    }

    #[must_use]
    pub const fn state_program_count(&self) -> usize {
        24
    }

    #[must_use]
    pub const fn curio_count(&self) -> usize {
        18
    }

    #[must_use]
    pub const fn digest(&self) -> [u8; 32] {
        self.digest
    }

    #[must_use]
    pub fn contains_curio(&self, curio: CurioId) -> bool {
        self.programs
            .binary_search_by_key(&curio, |program| program.curio)
            .is_ok()
    }

    pub fn execute(
        &self,
        contribution: &CurioContribution,
        event: NegativeCurioEvent,
    ) -> Result<Box<[AppliedCurioEffect]>, NegativeCurioRuntimeError> {
        let key = (contribution.curio(), contribution.state().id());
        let program = self
            .programs
            .binary_search_by_key(&key, |program| (program.curio, program.state))
            .ok()
            .map(|index| &self.programs[index])
            .ok_or(NegativeCurioRuntimeError::UnknownCurioState)?;
        if program.kind != contribution.state().kind()
            || program.source_effect_id.as_ref() != contribution.state().source_effect_id()
        {
            return Err(NegativeCurioRuntimeError::ContributionMismatch);
        }
        execute(program, event)
    }
}

fn execute(
    program: &CompiledNegativeCurioProgram,
    event: NegativeCurioEvent,
) -> Result<Box<[AppliedCurioEffect]>, NegativeCurioRuntimeError> {
    use NegativeCurioEvent as E;
    use NegativeCurioTemplate as T;
    let p = &program.parameters;
    let mut effects = Vec::with_capacity(2);
    match (program.template, program.kind, event) {
        (T::RepairDestroyedCurios, CurioStateKind::Active, E::Acquired) => {
            effects.push(CurioEffect::RepairRandomDestroyedCurios {
                maximum: turns(p[0])?,
                restore_default_charges: true,
            });
        }
        (T::ReplaceOwnedCurios, CurioStateKind::Active, E::Acquired) => {
            effects.push(CurioEffect::ReplaceAllOwnedCuriosRandomly {
                include_source: true,
            });
        }
        (T::ReplaceBlessings, CurioStateKind::Active, E::Acquired) => {
            effects.push(CurioEffect::ReplaceAllBlessingsRandomly {
                retain_enhancement: true,
                released_higher_rarity_chance: true,
            });
        }
        (T::ErrorEnergy, CurioStateKind::Repairing, E::WeaknessBroken) => {
            effects.push(CurioEffect::ChangeActorEnergy {
                change: CurioEnergyChange::Clear,
            });
        }
        (T::ErrorEnergy, CurioStateKind::Fixed, E::WeaknessBroken) => {
            effects.push(CurioEffect::ChangeActorEnergy {
                change: CurioEnergyChange::RestoreMaximum,
            });
        }
        (
            T::ErrorHp,
            kind @ (CurioStateKind::Repairing | CurioStateKind::Fixed),
            E::UltimateUsed,
        ) => {
            effects.push(CurioEffect::ChangeActorCurrentHpRatio {
                change: if kind == CurioStateKind::Repairing {
                    CurioHpChange::Consume
                } else {
                    CurioHpChange::Restore
                },
                ratio: p[0],
                can_defeat: false,
            });
        }
        (T::ErrorDamageTaken, CurioStateKind::Repairing, E::DamageTakenCalculated) => {
            effects.push(CurioEffect::BattleStatWhileCurrentHpBelow {
                target: PathEffectTarget::Actor,
                stat: PathEffectStat::DamageTakenRatio,
                value: p[1],
                threshold: p[0],
            });
        }
        (T::ErrorDamageTaken, CurioStateKind::Fixed, E::DamageTakenCalculated) => {
            effects.push(battle_stat(
                PathEffectTarget::AllAllies,
                PathEffectStat::DamageTakenReductionRatio,
                p[2],
            ));
        }
        (
            T::ErrorActionAdvance,
            kind @ (CurioStateKind::Repairing | CurioStateKind::Fixed),
            E::SkillUsed,
        ) => {
            effects.push(CurioEffect::Battle(PathEffect::ActionAdvance {
                target: if kind == CurioStateKind::Repairing {
                    PathEffectTarget::RandomEnemy
                } else {
                    PathEffectTarget::Actor
                },
                ratio: p[0],
                cannot_repeat_for_same_actor: false,
            }));
        }
        (
            T::ErrorDamageDealt,
            kind @ (CurioStateKind::Repairing | CurioStateKind::Fixed),
            E::EnemyDefeated,
        ) => {
            effects.push(battle_stat(
                if kind == CurioStateKind::Repairing {
                    PathEffectTarget::AllEnemies
                } else {
                    PathEffectTarget::AllAllies
                },
                PathEffectStat::DamageRatio,
                p[0],
            ));
        }
        (T::ErrorSkillPoints, CurioStateKind::Repairing, E::SkillUsed) => {
            effects.push(CurioEffect::ModifySkillPoints {
                delta: -signed_count(p[0])?,
            });
        }
        (T::ErrorSkillPoints, CurioStateKind::Fixed, E::BasicAttackUsed) => {
            effects.push(CurioEffect::ModifySkillPoints {
                delta: signed_count(p[0])?,
            });
        }
        (T::PostActionAdvance, CurioStateKind::Active, E::ActionEnded) => {
            effects.push(CurioEffect::Battle(PathEffect::ActionAdvance {
                target: PathEffectTarget::Actor,
                ratio: p[1],
                cannot_repeat_for_same_actor: false,
            }));
        }
        (T::Parasitized, CurioStateKind::Active, E::BattleStarted) => {
            effects.push(CurioEffect::ConfigureParasitized {
                attack_ratio: p[0],
                turn_current_hp_cost_ratio: p[1],
                transfer_to_random_ally_when_downed: true,
            });
        }
        (T::FragmentDebt, CurioStateKind::Active, E::BattleWon) => {
            effects.push(CurioEffect::SuppressBattleFragmentsThenDoubleCurrent {
                triggers: turns(p[0])?,
            });
        }
        (T::ReducedBlessingOffers, CurioStateKind::Active, E::BlessingRewardOffered) => {
            effects.push(CurioEffect::ConfigureBlessingReward {
                extra_selections: 0,
                offer_count_delta: -signed_count(p[0])?,
                free_rerolls: 0,
                guaranteed_rarity: None,
                enhance_all_one_star: false,
                enhance_random_count: 0,
            });
        }
        (T::MajorAggro, CurioStateKind::Active, E::BattleStarted) => {
            effects.push(CurioEffect::ApplyReleasedMajorAggro {
                target: PathEffectTarget::RandomAlly,
                target_count: turns(p[0])?,
                duration_turns: turns(p[1])?,
            });
        }
        (T::DomainFragmentLoss, CurioStateKind::Active, E::DomainEntered) => {
            effects.push(CurioEffect::LoseCosmicFragmentsRatio { ratio: p[0] });
        }
        (T::BlessingServiceInflation, CurioStateKind::Active, E::BlessingServicePriced) => {
            effects.push(CurioEffect::IncreaseBlessingServiceCost {
                ratio: p[0],
                affects_enhance: true,
                affects_reset: true,
            });
        }
        (T::EntrySkillPointLoss, CurioStateKind::Active, E::BattleStarted) => {
            effects.push(CurioEffect::ConsumeSkillPoints {
                amount: turns(p[0])?,
            });
        }
        (T::Fission, CurioStateKind::Active, E::StatQueried) => {
            effects.push(battle_stat(
                PathEffectTarget::AllAllies,
                PathEffectStat::AttackRatio,
                negate(p[0])?,
            ));
        }
        (T::Fission, CurioStateKind::Active, E::BattleWon) => {
            effects.push(CurioEffect::ConfigureCurioFission {
                released_chance: true,
                maximum_concurrent_copies: turns(p[1])?,
            });
        }
        _ => {}
    }
    Ok(effects
        .into_iter()
        .map(|effect| AppliedCurioEffect::new(&program.source_key, effect))
        .collect::<Vec<_>>()
        .into_boxed_slice())
}

fn battle_stat(
    target: PathEffectTarget,
    stat: PathEffectStat,
    value: PathEffectValue,
) -> CurioEffect {
    CurioEffect::Battle(PathEffect::AddStat {
        target,
        stat,
        value,
        cap: None,
    })
}

fn signed_count(value: PathEffectValue) -> Result<i8, NegativeCurioRuntimeError> {
    i8::try_from(turns(value)?).map_err(|_| NegativeCurioRuntimeError::InvalidParameter)
}

fn negate(value: PathEffectValue) -> Result<PathEffectValue, NegativeCurioRuntimeError> {
    value
        .raw_six_decimal()
        .checked_neg()
        .map(PathEffectValue::from_raw_six_decimal)
        .ok_or(NegativeCurioRuntimeError::Overflow)
}

fn state_suffix(kind: CurioStateKind) -> &'static str {
    match kind {
        CurioStateKind::Active => "active",
        CurioStateKind::Repairing => "repairing",
        CurioStateKind::Fixed => "fixed",
    }
}

fn valid_kind(program: &CompiledNegativeCurioProgram) -> bool {
    match program.template {
        NegativeCurioTemplate::ErrorEnergy
        | NegativeCurioTemplate::ErrorHp
        | NegativeCurioTemplate::ErrorDamageTaken
        | NegativeCurioTemplate::ErrorActionAdvance
        | NegativeCurioTemplate::ErrorDamageDealt
        | NegativeCurioTemplate::ErrorSkillPoints => matches!(
            program.kind,
            CurioStateKind::Repairing | CurioStateKind::Fixed
        ),
        _ => program.kind == CurioStateKind::Active,
    }
}

fn registry(effect: &str) -> Option<(NegativeCurioTemplate, usize)> {
    use NegativeCurioTemplate as T;
    Some(match effect {
        "17" => (T::RepairDestroyedCurios, 1),
        "21" => (T::ReplaceOwnedCurios, 1),
        "45" => (T::ErrorEnergy, 1),
        "47" => (T::ErrorHp, 1),
        "49" => (T::ErrorDamageTaken, 3),
        "51" => (T::ErrorActionAdvance, 1),
        "53" => (T::ErrorDamageDealt, 1),
        "55" => (T::ErrorSkillPoints, 1),
        "57" => (T::PostActionAdvance, 2),
        "59" => (T::Parasitized, 2),
        "60" => (T::FragmentDebt, 1),
        "65" => (T::ReducedBlessingOffers, 1),
        "66" => (T::MajorAggro, 2),
        "67" => (T::DomainFragmentLoss, 1),
        "70" => (T::BlessingServiceInflation, 1),
        "71" => (T::EntrySkillPointLoss, 1),
        "78" => (T::Fission, 2),
        "86" => (T::ReplaceBlessings, 0),
        _ => return None,
    })
}

fn catalog_digest(programs: &[CompiledNegativeCurioProgram]) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock-universe-negative-curio-runtime-catalog-v1");
    encoder.text(NEGATIVE_CURIO_RUNTIME_REVISION);
    encoder.u32(programs.len() as u32);
    for program in programs {
        encoder.u32(program.curio.get());
        encoder.u32(program.state.get());
        encoder.u8(program.kind as u8);
        encoder.text(&program.source_key);
        encoder.text(&program.source_effect_id);
        encoder.u8(program.template as u8);
        encoder.u32(program.parameters.len() as u32);
        for parameter in &program.parameters {
            encoder.i64(parameter.raw_six_decimal());
        }
    }
    encoder.finish()
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NegativeCurioRuntimeError {
    InvalidDefinition,
    InvalidDenominator,
    InvalidParameter,
    UnknownCurioState,
    ContributionMismatch,
    Overflow,
}

impl From<crate::path_effect_runtime::PathEffectRuntimeError> for NegativeCurioRuntimeError {
    fn from(value: crate::path_effect_runtime::PathEffectRuntimeError) -> Self {
        match value {
            crate::path_effect_runtime::PathEffectRuntimeError::Overflow => Self::Overflow,
            _ => Self::InvalidParameter,
        }
    }
}
