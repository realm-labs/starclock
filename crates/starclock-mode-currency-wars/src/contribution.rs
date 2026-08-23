//! Immutable Activity-to-battle contribution boundary.

use sha2::{Digest, Sha256};
use starclock_build::{
    ability::AbilityInvestment, spec::CombatantBuildSpec, substitution::BuildSubstitutionReceipt,
};
use starclock_combat::{BattleClockSpec, ResolvedCombatantSpec, Scalar};

use crate::{
    CurrencyWarsAugmentQuality, CurrencyWarsBackBattleEvent, CurrencyWarsBattleOverrideRoleBuild,
    CurrencyWarsBattleOverrideSnapshot, CurrencyWarsBondContribution, CurrencyWarsBondMember,
    CurrencyWarsBondSnapshot, CurrencyWarsCharacterOverridePolicy,
    CurrencyWarsCharacterOverrideProgram, CurrencyWarsContributionParameter,
    CurrencyWarsCyreneSkillOverride, CurrencyWarsDecimal, CurrencyWarsDeployment,
    CurrencyWarsDifficulty, CurrencyWarsEnemyAffix, CurrencyWarsEnemyAffixBehavior,
    CurrencyWarsEnemyAffixSelectionSource, CurrencyWarsEnhancement,
    CurrencyWarsEquipmentDefinition, CurrencyWarsEquipmentSlot, CurrencyWarsGambit,
    CurrencyWarsInfluenceProperty, CurrencyWarsInvestment, CurrencyWarsInvestmentId,
    CurrencyWarsInvestmentMazeBuff, CurrencyWarsLethalRescueHpPolicy,
    CurrencyWarsMazeBuffEnhancement, CurrencyWarsNode, CurrencyWarsNodeId,
    CurrencyWarsOffFieldContributionSnapshot, CurrencyWarsPosition, CurrencyWarsPositionKind,
    CurrencyWarsRankSkillOverride, CurrencyWarsRole, CurrencyWarsRoleGlobalModifier,
    CurrencyWarsRoleState, CurrencyWarsRouteId, CurrencyWarsRuntimeEquipment,
    CurrencyWarsSelectedEnhancement, CurrencyWarsSelectedEnhancementId,
    CurrencyWarsSharedBattleBase, CurrencyWarsSkillParameterEdit,
    CurrencyWarsSkillParameterOperator, CurrencyWarsSpecialGood, CurrencyWarsSpecialResourceKind,
    CurrencyWarsStarState, CurrencyWarsStarStateOwner, CurrencyWarsSummonBattleEventOverride,
    CurrencyWarsTalentDefinition, CurrencyWarsTalentKind, CurrencyWarsTeamLevel,
    CurrencyWarsTrialBuild, CurrencyWarsTypedInvestment,
    runtime::{
        CurrencyWarsRun, CurrencyWarsRuntimeError, INVESTMENTS, SEASON_TALENTS,
        SELECTED_ENHANCEMENTS, debug_error, error,
    },
};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CurrencyWarsContributionDigest([u8; 32]);

impl CurrencyWarsContributionDigest {
    #[must_use]
    pub const fn bytes(self) -> [u8; 32] {
        self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsSelectedEmpowermentSkill {
    pub skill_id: u32,
    pub source_skill_id: Option<u32>,
    pub level: u8,
    pub stable_key: Box<str>,
    pub parameters: Box<[Scalar]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsRoleContribution {
    pub position: CurrencyWarsPosition,
    pub role_state: CurrencyWarsRoleState,
    pub role: CurrencyWarsRole,
    pub star_state: CurrencyWarsStarState,
    pub servant_star_states: Box<[CurrencyWarsStarState]>,
    pub character_override: Option<CurrencyWarsCharacterOverrideProgram>,
    pub character_override_policy: Option<CurrencyWarsCharacterOverridePolicy>,
    pub servant_overrides: Box<[CurrencyWarsCharacterOverrideProgram]>,
    pub servant_override_policies: Box<[CurrencyWarsCharacterOverridePolicy]>,
    pub build: CombatantBuildSpec,
    pub combatant: ResolvedCombatantSpec,
    pub substitution: BuildSubstitutionReceipt,
    pub effective_ability_levels: Box<[AbilityInvestment]>,
    pub equipment: Box<[CurrencyWarsSelectedEquipment]>,
    pub inactive_bond_count: u8,
    pub off_field: CurrencyWarsOffFieldContributionSnapshot,
    pub empowerment: Box<[CurrencyWarsSelectedEmpowermentSkill]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsSelectedEquipment {
    pub slot: CurrencyWarsEquipmentSlot,
    pub definition: CurrencyWarsEquipmentDefinition,
    pub runtime: CurrencyWarsRuntimeEquipment,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsActivatedSpecialGood {
    pub definition: CurrencyWarsSpecialGood,
    pub activation_count: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsContributionSnapshot {
    pub digest: CurrencyWarsContributionDigest,
    pub route: CurrencyWarsRouteId,
    pub difficulty: CurrencyWarsDifficulty,
    pub enemy_battle_base: CurrencyWarsSharedBattleBase,
    pub gambit: CurrencyWarsGambit,
    pub node: CurrencyWarsNode,
    pub team_level: CurrencyWarsTeamLevel,
    pub squad_hp: u32,
    pub unequipped_equipment_count: u32,
    pub roles: Box<[CurrencyWarsRoleContribution]>,
    pub bonds: CurrencyWarsBondSnapshot,
    pub bond_registry: Box<[CurrencyWarsBondContribution]>,
    pub investments: Box<[CurrencyWarsInvestment]>,
    pub typed_investments: Box<[CurrencyWarsTypedInvestment]>,
    pub enhancements: Box<[CurrencyWarsEnhancement]>,
    pub augment_maze_buffs: Box<[CurrencyWarsInvestmentMazeBuff]>,
    pub augment_enemy_difficulty_add: Box<[(CurrencyWarsAugmentQuality, u8)]>,
    pub binary_enemy_difficulty_add: u8,
    pub enemy_affixes: Box<[CurrencyWarsEnemyAffix]>,
    pub enemy_affix_behaviors: Box<[CurrencyWarsEnemyAffixBehavior]>,
    pub enemy_affix_selection_source: CurrencyWarsEnemyAffixSelectionSource,
    pub maze_buff_enhancements: Box<[CurrencyWarsMazeBuffEnhancement]>,
    pub season_talents: Box<[CurrencyWarsTalentDefinition]>,
    pub selected_enhancements: Box<[CurrencyWarsSelectedEnhancement]>,
    pub special_goods: Box<[CurrencyWarsActivatedSpecialGood]>,
    pub influence_properties: Box<[CurrencyWarsInfluenceProperty]>,
    pub parameter_registry: Box<[CurrencyWarsContributionParameter]>,
    pub summon_battle_event_overrides: Box<[CurrencyWarsCharacterOverrideProgram]>,
    pub battle_clock: Option<BattleClockSpec>,
    pub battle_overrides: CurrencyWarsBattleOverrideSnapshot,
}

pub(super) struct SelectedRoleBuild {
    pub override_build: CurrencyWarsBattleOverrideRoleBuild,
    spec: CombatantBuildSpec,
    combatant: ResolvedCombatantSpec,
    receipt: BuildSubstitutionReceipt,
    effective_ability_levels: Box<[AbilityInvestment]>,
}

pub(super) fn selected_role_builds(
    run: &CurrencyWarsRun,
    deployment: &CurrencyWarsDeployment,
) -> Result<Vec<SelectedRoleBuild>, CurrencyWarsRuntimeError> {
    deployment
        .positions()
        .values()
        .map(|state| {
            let role = state.role();
            let trial = run
                .definition
                .catalog
                .build_catalog()
                .trial_build(role)
                .ok_or_else(|| error("Currency Wars deployed role has no trial Build"))?;
            if let Some(owned) = run.definition.owned_builds.get(&role) {
                return Ok(SelectedRoleBuild {
                    override_build: CurrencyWarsBattleOverrideRoleBuild {
                        role,
                        technique_ability: trial.technique_ability,
                        eidolon: owned.spec().eidolon().get(),
                    },
                    spec: owned.spec().clone(),
                    combatant: owned.combatant().clone(),
                    receipt: owned.receipt(),
                    effective_ability_levels: owned.effective_ability_levels().into(),
                });
            }
            let selected = trial.substitute_owned(None).map_err(debug_error)?;
            let receipt = selected.receipt();
            let spec = selected.into_spec();
            Ok(SelectedRoleBuild {
                override_build: CurrencyWarsBattleOverrideRoleBuild {
                    role,
                    technique_ability: trial.technique_ability,
                    eidolon: spec.eidolon().get(),
                },
                spec,
                combatant: trial.combatant.clone(),
                receipt,
                effective_ability_levels: trial.effective_ability_levels.clone(),
            })
        })
        .collect()
}

pub(super) fn materialize(
    run: &CurrencyWarsRun,
) -> Result<CurrencyWarsContributionSnapshot, CurrencyWarsRuntimeError> {
    let deployment = run.deployment()?;
    let loadout = run.equipment_loadout()?;
    let unequipped_equipment_count = run
        .equipment_inventory()?
        .values()
        .try_fold(0_u32, |total, count| total.checked_add(*count))
        .ok_or_else(|| error("Currency Wars equipment inventory count overflow"))?;
    let selected = selected_role_builds(run, &deployment)?;
    let bonds = run.bond_snapshot()?;
    let difficulty = run
        .definition
        .catalog
        .difficulties()
        .iter()
        .find(|value| value.source_id == run.definition.difficulty)
        .cloned()
        .ok_or_else(|| error("Currency Wars contribution difficulty is missing"))?;
    let division_level = difficulty.division_level;
    let investment_ids = run
        .ordered_ids(INVESTMENTS)?
        .iter()
        .map(|raw| {
            CurrencyWarsInvestmentId::new(*raw)
                .ok_or_else(|| error("Currency Wars selected investment ID is invalid"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let investments = investment_ids
        .iter()
        .map(|id| {
            run.definition
                .catalog
                .investment(*id)
                .cloned()
                .ok_or_else(|| error("Currency Wars selected investment definition is missing"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let selected_enhancements = run
        .ordered_ids(SELECTED_ENHANCEMENTS)?
        .iter()
        .map(|raw| {
            u32::try_from(*raw)
                .ok()
                .and_then(CurrencyWarsSelectedEnhancementId::new)
                .and_then(|id| {
                    run.definition
                        .catalog
                        .augment_catalog()
                        .selected_enhancement(id)
                })
                .cloned()
                .ok_or_else(|| error("Currency Wars selected Enhancement definition is missing"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let special_goods = run
        .special_good_activations()?
        .into_iter()
        .map(|(id, activation_count)| {
            run.definition
                .catalog
                .service_catalog()
                .special_good(id)
                .cloned()
                .map(|definition| CurrencyWarsActivatedSpecialGood {
                    definition,
                    activation_count,
                })
                .ok_or_else(|| error("Currency Wars activated special good is missing"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let typed_investments = investment_ids
        .iter()
        .filter_map(|id| {
            run.definition
                .catalog
                .cross_investment_catalog()
                .investment(*id)
        })
        .collect::<Vec<_>>();
    let enhancements = investment_ids
        .iter()
        .filter_map(|id| {
            run.definition
                .catalog
                .augment_catalog()
                .enhancement(*id)
                .cloned()
        })
        .collect::<Vec<_>>();
    let augment_maze_buffs = run
        .definition
        .catalog
        .augment_catalog()
        .maze_buffs()
        .to_vec();
    let augment_enemy_difficulty_add = [
        CurrencyWarsAugmentQuality::Silver,
        CurrencyWarsAugmentQuality::Gold,
        CurrencyWarsAugmentQuality::Prismatic,
    ]
    .into_iter()
    .map(|quality| {
        (
            quality,
            run.definition
                .catalog
                .augment_catalog()
                .enemy_difficulty_add(quality, division_level),
        )
    })
    .collect::<Vec<_>>();
    let enemy_affixes = run
        .definition
        .enemy_affixes
        .source_ids()
        .iter()
        .map(|source_id| {
            run.definition
                .catalog
                .encounter_catalog()
                .enemy_affix_definition(*source_id)
                .cloned()
                .ok_or_else(|| error("Currency Wars selected enemy Affix is missing"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let binary_enemy_difficulty_add = match difficulty.binary_difficulty_rule {
        Some(rule) => run
            .definition
            .catalog
            .flow_catalog()
            .binary_difficulty_addition(
                rule,
                u8::try_from(enemy_affixes.len()).map_err(debug_error)?,
            )
            .ok_or_else(|| error("Currency Wars binary enemy-difficulty rule is missing"))?,
        None if enemy_affixes.is_empty() => 0,
        None => {
            return Err(error(
                "Currency Wars selected enemy Affixes without a binary difficulty rule",
            ));
        }
    };
    let season_talents = run
        .ordered_ids(SEASON_TALENTS)?
        .iter()
        .map(|raw| {
            u32::try_from(*raw)
                .ok()
                .and_then(|id| {
                    run.definition
                        .catalog
                        .cross_investment_catalog()
                        .talent(CurrencyWarsTalentKind::Season, id)
                })
                .cloned()
                .ok_or_else(|| error("Currency Wars selected season Talent is missing"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let maze_buff_enhancements = run
        .definition
        .catalog
        .blessing_formula_catalog()
        .maze_buff_enhancements()
        .to_vec();
    let mut roles = Vec::with_capacity(selected.len());
    for ((position, role_state), selected) in deployment.positions().iter().zip(selected) {
        let trial = run
            .definition
            .catalog
            .build_catalog()
            .trial_build(role_state.role())
            .expect("selected role has a validated trial Build");
        let equipment = loadout
            .for_role(role_state.role())
            .map(|(slot, id)| {
                run.definition
                    .catalog
                    .build_catalog()
                    .runtime_equipment_definition(id)
                    .cloned()
                    .and_then(|definition| {
                        definition
                            .runtime
                            .clone()
                            .map(|runtime| CurrencyWarsSelectedEquipment {
                                slot,
                                definition,
                                runtime,
                            })
                    })
                    .ok_or_else(|| error("Currency Wars equipped definition is missing"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let off_field = run
            .definition
            .catalog
            .build_catalog()
            .resolve_off_field_contributions(role_state.role(), position.kind(), &selected.spec);
        let inactive_bond_count = u8::try_from(
            run.definition
                .catalog
                .bonds()
                .iter()
                .filter(|bond| {
                    bond.members.iter().any(|member| {
                        matches!(member, CurrencyWarsBondMember::RosterRole(role)
                            if *role == role_state.role())
                    })
                })
                .filter(|bond| !bonds.active_bonds.iter().any(|active| active.id == bond.id))
                .count(),
        )
        .map_err(debug_error)?;
        let empowerment = selected_empowerment(
            run,
            *position,
            *role_state,
            trial,
            &selected.effective_ability_levels,
        )?;
        let role = run
            .definition
            .catalog
            .role(role_state.role())
            .cloned()
            .ok_or_else(|| error("Currency Wars deployed role definition is missing"))?;
        let star_state = selected_star_state(run, *role_state)?.clone();
        let servant_star_states = run
            .definition
            .catalog
            .economy_catalog()
            .star_states()
            .iter()
            .filter(|state| {
                matches!(state.owner, CurrencyWarsStarStateOwner::Servant { avatar_id, .. }
                    if avatar_id == role_state.role().get())
                    && state.star == role_state.star()
            })
            .cloned()
            .collect::<Vec<_>>();
        let character_override = run
            .definition
            .catalog
            .role_override_catalog()
            .for_star_state(&star_state)
            .map_err(debug_error)?
            .cloned();
        let character_override_policy = run
            .definition
            .catalog
            .role_override_catalog()
            .policy_for_star_state(&star_state);
        let servant_overrides = servant_star_states
            .iter()
            .map(|state| {
                run.definition
                    .catalog
                    .role_override_catalog()
                    .for_star_state(state)
            })
            .collect::<Result<Vec<_>, _>>()
            .map_err(debug_error)?
            .into_iter()
            .flatten()
            .cloned()
            .collect::<Vec<_>>();
        let servant_override_policies = servant_star_states
            .iter()
            .filter_map(|state| {
                run.definition
                    .catalog
                    .role_override_catalog()
                    .policy_for_star_state(state)
            })
            .collect::<Vec<_>>();
        roles.push(CurrencyWarsRoleContribution {
            position: *position,
            role_state: *role_state,
            role,
            star_state,
            servant_star_states: servant_star_states.into_boxed_slice(),
            character_override,
            character_override_policy,
            servant_overrides: servant_overrides.into_boxed_slice(),
            servant_override_policies: servant_override_policies.into_boxed_slice(),
            build: selected.spec,
            combatant: selected.combatant,
            substitution: selected.receipt,
            effective_ability_levels: selected.effective_ability_levels,
            equipment: equipment.into_boxed_slice(),
            inactive_bond_count,
            off_field,
            empowerment,
        });
    }
    let battle_overrides = run.battle_override_snapshot()?;
    apply_skill_parameter_overrides(&mut roles, &battle_overrides)?;
    let battle_clock = run.current_battle_boundary()?.clock();
    let node = run
        .current_node()
        .cloned()
        .ok_or_else(|| error("Currency Wars contribution snapshot has no current node"))?;
    let enemy_battle_base = run
        .definition
        .catalog
        .flow_catalog()
        .stage_battle_base(node.encounter)
        .or_else(|| {
            run.definition
                .catalog
                .flow_catalog()
                .level_battle_base(node.plane, node.ordinal)
        })
        .ok_or_else(|| error("Currency Wars shared enemy battle base is missing"))?;
    let summon_battle_event_overrides = run
        .definition
        .catalog
        .role_override_catalog()
        .summon_battle_events(difficulty.season_id)
        .cloned()
        .collect::<Vec<_>>();
    let team_level = run
        .definition
        .catalog
        .team_level(run.team_level())
        .cloned()
        .ok_or_else(|| error("Currency Wars contribution team level is missing"))?;
    let influence_properties = run
        .definition
        .catalog
        .economy_catalog()
        .influence_properties()
        .to_vec();
    let parameter_registry = run
        .definition
        .catalog
        .economy_catalog()
        .contribution_parameters()
        .to_vec();
    let bond_registry = run
        .definition
        .catalog
        .bond_catalog()
        .contributions()
        .to_vec();
    let digest = digest(
        run,
        node.id,
        DigestComponents {
            roles: &roles,
            bonds: &bonds,
            investments: &investment_ids,
            enemy_affixes: run.definition.enemy_affixes.source_ids(),
            selected_enhancements: &selected_enhancements,
            season_talents: &season_talents,
            summon_overrides: &summon_battle_event_overrides,
            battle_clock,
            overrides: &battle_overrides,
            unequipped_equipment_count,
        },
    );
    Ok(CurrencyWarsContributionSnapshot {
        digest,
        route: run.definition.route,
        difficulty,
        enemy_battle_base,
        gambit: run.definition.gambit,
        node,
        team_level,
        squad_hp: run.squad_hp(),
        unequipped_equipment_count,
        roles: roles.into_boxed_slice(),
        bonds,
        bond_registry: bond_registry.into_boxed_slice(),
        investments: investments.into_boxed_slice(),
        typed_investments: typed_investments.into_boxed_slice(),
        enhancements: enhancements.into_boxed_slice(),
        augment_maze_buffs: augment_maze_buffs.into_boxed_slice(),
        augment_enemy_difficulty_add: augment_enemy_difficulty_add.into_boxed_slice(),
        binary_enemy_difficulty_add,
        enemy_affixes: enemy_affixes.into_boxed_slice(),
        enemy_affix_behaviors: run.definition.enemy_affixes.behaviors().into(),
        enemy_affix_selection_source: run.definition.enemy_affixes.source(),
        maze_buff_enhancements: maze_buff_enhancements.into_boxed_slice(),
        season_talents: season_talents.into_boxed_slice(),
        selected_enhancements: selected_enhancements.into_boxed_slice(),
        special_goods: special_goods.into_boxed_slice(),
        influence_properties: influence_properties.into_boxed_slice(),
        parameter_registry: parameter_registry.into_boxed_slice(),
        summon_battle_event_overrides: summon_battle_event_overrides.into_boxed_slice(),
        battle_clock,
        battle_overrides,
    })
}

fn selected_empowerment(
    run: &CurrencyWarsRun,
    position: CurrencyWarsPosition,
    role_state: CurrencyWarsRoleState,
    trial: &CurrencyWarsTrialBuild,
    effective_levels: &[AbilityInvestment],
) -> Result<Box<[CurrencyWarsSelectedEmpowermentSkill]>, CurrencyWarsRuntimeError> {
    let star = selected_star_state(run, role_state)?;
    let (skill_ids, source_ids): (&[u32], &[u32]) = match position.kind() {
        CurrencyWarsPositionKind::Front => (
            &star.front_execution_skill_ids,
            &star.skill_override_source_ids,
        ),
        CurrencyWarsPositionKind::Back => (&star.back_execution_skill_ids, &[]),
    };
    skill_ids
        .iter()
        .enumerate()
        .map(|(index, &skill_id)| {
            let source_skill_id = source_ids.get(index).copied().filter(|id| *id != 0);
            let level = match source_skill_id {
                Some(source) => {
                    let binding = trial
                        .source_ability_bindings
                        .binary_search_by_key(&source, |binding| binding.source_skill_id)
                        .ok()
                        .map(|index| trial.source_ability_bindings[index])
                        .ok_or_else(|| error("Currency Wars source skill has no shared binding"))?;
                    effective_levels
                        .iter()
                        .find(|entry| entry.family() == binding.shared_ability)
                        .map(|entry| entry.invested().get())
                        .ok_or_else(|| error("Currency Wars source skill level is missing"))?
                }
                None => run
                    .definition
                    .catalog
                    .empowerment_catalog()
                    .maximum_skill_level(position.kind(), skill_id)
                    .ok_or_else(|| error("Currency Wars mode-defined skill family is missing"))?,
            };
            let row = run
                .definition
                .catalog
                .empowerment_catalog()
                .skill_row(position.kind(), skill_id, level)
                .ok_or_else(|| error("Currency Wars selected Empowerment level is missing"))?;
            Ok(CurrencyWarsSelectedEmpowermentSkill {
                skill_id,
                source_skill_id,
                level,
                stable_key: row.stable_key.clone(),
                parameters: row
                    .parameter_values
                    .iter()
                    .map(|value| value.scalar().map_err(debug_error))
                    .collect::<Result<Vec<_>, _>>()?
                    .into_boxed_slice(),
            })
        })
        .collect::<Result<Vec<_>, _>>()
        .map(Vec::into_boxed_slice)
}

fn apply_skill_parameter_overrides(
    roles: &mut [CurrencyWarsRoleContribution],
    overrides: &CurrencyWarsBattleOverrideSnapshot,
) -> Result<(), CurrencyWarsRuntimeError> {
    for role in roles {
        for skill in &mut role.empowerment {
            for override_ in overrides
                .rank_skill_overrides
                .iter()
                .filter(|override_| override_.skill_id == skill.skill_id)
            {
                apply_skill_parameter_edits(&mut skill.parameters, &override_.edits)?;
            }
            for override_ in overrides.cyrene_skill_overrides.iter().filter(|override_| {
                override_.role == role.role.id && override_.skill_id == skill.skill_id
            }) {
                apply_skill_parameter_edits(&mut skill.parameters, &override_.edits)?;
            }
        }
    }
    Ok(())
}

fn apply_skill_parameter_edits(
    parameters: &mut [Scalar],
    edits: &[CurrencyWarsSkillParameterEdit],
) -> Result<(), CurrencyWarsRuntimeError> {
    for edit in edits {
        let parameter = parameters
            .get_mut(usize::from(edit.index))
            .ok_or_else(|| error("Currency Wars skill parameter edit index is out of bounds"))?;
        *parameter = edit.apply(*parameter).map_err(debug_error)?;
    }
    Ok(())
}

fn selected_star_state(
    run: &CurrencyWarsRun,
    role_state: CurrencyWarsRoleState,
) -> Result<&CurrencyWarsStarState, CurrencyWarsRuntimeError> {
    run.definition
        .catalog
        .economy_catalog()
        .star_states()
        .iter()
        .find(|state| {
            state.owner == CurrencyWarsStarStateOwner::Role(role_state.role())
                && state.star == role_state.star()
        })
        .ok_or_else(|| error("Currency Wars contribution star state is missing"))
}

struct DigestComponents<'a> {
    roles: &'a [CurrencyWarsRoleContribution],
    bonds: &'a CurrencyWarsBondSnapshot,
    investments: &'a [CurrencyWarsInvestmentId],
    enemy_affixes: &'a [u32],
    selected_enhancements: &'a [CurrencyWarsSelectedEnhancement],
    season_talents: &'a [CurrencyWarsTalentDefinition],
    summon_overrides: &'a [CurrencyWarsCharacterOverrideProgram],
    battle_clock: Option<BattleClockSpec>,
    overrides: &'a CurrencyWarsBattleOverrideSnapshot,
    unequipped_equipment_count: u32,
}

fn digest(
    run: &CurrencyWarsRun,
    node: CurrencyWarsNodeId,
    components: DigestComponents<'_>,
) -> CurrencyWarsContributionDigest {
    let DigestComponents {
        roles,
        bonds,
        investments,
        enemy_affixes,
        selected_enhancements,
        season_talents,
        summon_overrides,
        battle_clock,
        overrides,
        unequipped_equipment_count,
    } = components;
    let identity = run.definition.activity.identity();
    let mut hash = Sha256::new();
    hash.update(b"starclock-currency-wars-contribution-v2");
    hash.update(identity.definition_digest().bytes());
    hash.update(identity.config_digest().bytes());
    hash.update(run.state_hash().bytes());
    hash.update(run.definition.route.get().to_le_bytes());
    hash.update(run.definition.difficulty.to_le_bytes());
    hash.update([gambit_tag(run.definition.gambit)]);
    hash.update(node.get().to_le_bytes());
    hash.update(unequipped_equipment_count.to_le_bytes());
    encode_len(&mut hash, roles.len());
    for role in roles {
        hash.update(role.position.encode().to_le_bytes());
        hash.update(role.role_state.encode().to_le_bytes());
        encode_build(&mut hash, &role.build);
        hash.update(role.combatant.digest().bytes());
        encode_receipt(&mut hash, role.substitution);
        encode_len(&mut hash, role.effective_ability_levels.len());
        for level in &role.effective_ability_levels {
            hash.update(level.family().get().to_le_bytes());
            hash.update([level.invested().get()]);
        }
        encode_len(&mut hash, role.equipment.len());
        for equipment in &role.equipment {
            hash.update(equipment.slot.encode().to_le_bytes());
            hash.update(equipment.runtime.id.get().to_le_bytes());
        }
        hash.update([role.inactive_bond_count]);
        encode_len(&mut hash, role.empowerment.len());
        for skill in &role.empowerment {
            hash.update(skill.skill_id.to_le_bytes());
            hash.update(skill.source_skill_id.unwrap_or_default().to_le_bytes());
            hash.update([skill.level]);
            encode_len(&mut hash, skill.parameters.len());
            for parameter in &skill.parameters {
                hash.update(parameter.scaled().to_le_bytes());
            }
        }
        encode_override(&mut hash, role.character_override.as_ref());
        encode_override_policy(&mut hash, role.character_override_policy.as_ref());
        encode_len(&mut hash, role.servant_overrides.len());
        for override_ in &role.servant_overrides {
            encode_override(&mut hash, Some(override_));
        }
        encode_len(&mut hash, role.servant_override_policies.len());
        for policy in &role.servant_override_policies {
            encode_override_policy(&mut hash, Some(policy));
        }
    }
    encode_len(&mut hash, bonds.active_bonds.len());
    for bond in &bonds.active_bonds {
        hash.update(bond.id.get().to_le_bytes());
        hash.update([bond.member_count, bond.level]);
    }
    encode_len(&mut hash, investments.len());
    for investment in investments {
        hash.update(investment.get().to_le_bytes());
    }
    encode_len(&mut hash, enemy_affixes.len());
    for source_id in enemy_affixes {
        hash.update(source_id.to_le_bytes());
    }
    encode_len(&mut hash, selected_enhancements.len());
    for enhancement in selected_enhancements {
        hash.update(enhancement.id.get().to_le_bytes());
    }
    encode_len(&mut hash, season_talents.len());
    for talent in season_talents {
        hash.update(talent.source_id.to_le_bytes());
    }
    encode_len(&mut hash, summon_overrides.len());
    for override_ in summon_overrides {
        encode_override(&mut hash, Some(override_));
    }
    match battle_clock {
        None => hash.update([0]),
        Some(BattleClockSpec::ActionValue(clock)) => {
            hash.update([1]);
            hash.update(clock.remaining().scaled().to_le_bytes());
            hash.update([clock.expiry() as u8]);
        }
        Some(BattleClockSpec::Cycles(clock)) => {
            hash.update([2]);
            hash.update(clock.remaining_cycles().to_le_bytes());
            hash.update(clock.first_window().scaled().to_le_bytes());
            hash.update(clock.later_window().scaled().to_le_bytes());
            hash.update([u8::from(clock.reset_window_on_wave())]);
            hash.update([clock.expiry() as u8]);
        }
    }
    encode_len(&mut hash, overrides.automatic_techniques.len());
    for technique in &overrides.automatic_techniques {
        hash.update(technique.position.encode().to_le_bytes());
        hash.update(technique.role_state.encode().to_le_bytes());
        hash.update(technique.ability.get().to_le_bytes());
    }
    hash.update(overrides.defeat_energy_ratio.scaled().to_le_bytes());
    hash.update([lethal_rescue_hp_policy_tag(
        overrides.lethal_rescue_hp_policy,
    )]);
    hash.update(overrides.lethal_rescue_action_value.scaled().to_le_bytes());
    encode_len(&mut hash, overrides.back_battle_events.len());
    for event in &overrides.back_battle_events {
        encode_back_battle_event(&mut hash, event);
    }
    encode_len(&mut hash, overrides.external_battle_event_ids.len());
    for event in &overrides.external_battle_event_ids {
        hash.update(event.to_le_bytes());
    }
    encode_len(&mut hash, overrides.special_resources.len());
    for resource in &overrides.special_resources {
        hash.update(resource.position.encode().to_le_bytes());
        hash.update(resource.role_state.encode().to_le_bytes());
        hash.update([special_resource_kind_tag(resource.kind)]);
        encode_decimal(&mut hash, resource.maximum);
    }
    encode_len(&mut hash, overrides.role_global_modifiers.len());
    for modifier in &overrides.role_global_modifiers {
        encode_role_global_modifier(&mut hash, modifier);
    }
    encode_len(&mut hash, overrides.rank_skill_overrides.len());
    for override_ in &overrides.rank_skill_overrides {
        encode_rank_skill_override(&mut hash, override_);
    }
    encode_len(&mut hash, overrides.summon_battle_event_overrides.len());
    for override_ in &overrides.summon_battle_event_overrides {
        encode_summon_battle_event_override(&mut hash, override_);
    }
    encode_len(&mut hash, overrides.cyrene_skill_overrides.len());
    for override_ in &overrides.cyrene_skill_overrides {
        encode_cyrene_skill_override(&mut hash, override_);
    }
    CurrencyWarsContributionDigest(hash.finalize().into())
}

fn encode_back_battle_event(hash: &mut Sha256, event: &CurrencyWarsBackBattleEvent) {
    hash.update(event.event_id.to_le_bytes());
    hash.update([
        event.kind.canonical_tag(),
        event.team.canonical_tag(),
        u8::from(event.hard_level),
    ]);
    encode_len(hash, event.abilities.len());
    for ability in &event.abilities {
        encode_bytes(hash, ability.as_bytes());
    }
    match event.speed {
        None => hash.update([0]),
        Some(speed) => {
            hash.update([1]);
            encode_decimal(hash, speed);
        }
    }
    encode_len(hash, event.values.len());
    for value in &event.values {
        encode_decimal(hash, *value);
    }
    encode_len(hash, event.properties.len());
    for property in &event.properties {
        hash.update([property.kind.canonical_tag()]);
        encode_decimal(hash, property.value);
    }
}

fn encode_role_global_modifier(hash: &mut Sha256, modifier: &CurrencyWarsRoleGlobalModifier) {
    hash.update(modifier.role.get().to_le_bytes());
    match &modifier.saved_value {
        None => hash.update([0]),
        Some(value) => {
            hash.update([1]);
            encode_bytes(hash, value.as_bytes());
        }
    }
    encode_len(hash, modifier.values.len());
    for value in &modifier.values {
        encode_decimal(hash, *value);
    }
}

fn encode_rank_skill_override(hash: &mut Sha256, override_: &CurrencyWarsRankSkillOverride) {
    hash.update(override_.rank_id.to_le_bytes());
    hash.update(override_.skill_id.to_le_bytes());
    encode_skill_parameter_edits(hash, &override_.edits);
}

fn encode_summon_battle_event_override(
    hash: &mut Sha256,
    override_: &CurrencyWarsSummonBattleEventOverride,
) {
    hash.update(override_.season_id.to_le_bytes());
    hash.update(override_.battle_event_id.to_le_bytes());
    encode_optional_bytes(hash, override_.front_config.as_deref());
    encode_optional_bytes(hash, override_.back_config.as_deref());
}

fn encode_cyrene_skill_override(hash: &mut Sha256, override_: &CurrencyWarsCyreneSkillOverride) {
    hash.update(override_.provider_role.get().to_le_bytes());
    hash.update(override_.role.get().to_le_bytes());
    hash.update(override_.skill_id.to_le_bytes());
    encode_bytes(hash, override_.multiple_value_key.as_bytes());
    encode_skill_parameter_edits(hash, &override_.edits);
}

fn encode_skill_parameter_edits(hash: &mut Sha256, edits: &[CurrencyWarsSkillParameterEdit]) {
    encode_len(hash, edits.len());
    for edit in edits {
        hash.update([edit.index, skill_parameter_operator_tag(edit.operator)]);
        encode_decimal(hash, edit.value);
    }
}

fn encode_optional_bytes(hash: &mut Sha256, value: Option<&str>) {
    match value {
        None => hash.update([0]),
        Some(value) => {
            hash.update([1]);
            encode_bytes(hash, value.as_bytes());
        }
    }
}

fn encode_decimal(hash: &mut Sha256, value: CurrencyWarsDecimal) {
    hash.update(value.significand().to_le_bytes());
    hash.update([value.decimal_places()]);
}

fn encode_override(hash: &mut Sha256, override_: Option<&CurrencyWarsCharacterOverrideProgram>) {
    let Some(override_) = override_ else {
        hash.update([0]);
        return;
    };
    hash.update([1]);
    encode_bytes(hash, override_.source_path.as_bytes());
    encode_bytes(hash, override_.source_sha256.as_bytes());
    encode_bytes(hash, override_.mechanical_shape_sha256.as_bytes());
}

fn encode_override_policy(hash: &mut Sha256, policy: Option<&CurrencyWarsCharacterOverridePolicy>) {
    let Some(policy) = policy else {
        hash.update([0]);
        return;
    };
    hash.update([1]);
    encode_bytes(hash, policy.policy_id.as_bytes());
    encode_bytes(hash, policy.source_path.as_bytes());
    encode_bytes(hash, policy.selected_behavior.as_bytes());
    encode_bytes(hash, policy.replacement_condition.as_bytes());
}

fn encode_bytes(hash: &mut Sha256, value: &[u8]) {
    encode_len(hash, value.len());
    hash.update(value);
}

fn encode_build(hash: &mut Sha256, spec: &CombatantBuildSpec) {
    hash.update(spec.form().get().to_le_bytes());
    hash.update([
        spec.level().get(),
        spec.promotion().get(),
        spec.eidolon().get(),
    ]);
    encode_len(hash, spec.ability_levels().len());
    for level in spec.ability_levels() {
        hash.update(level.family().get().to_le_bytes());
        hash.update([level.invested().get()]);
    }
    encode_len(hash, spec.traces().len());
    for trace in spec.traces() {
        hash.update(trace.get().to_le_bytes());
    }
    if let Some(cone) = spec.light_cone() {
        hash.update([1]);
        hash.update(cone.definition().get().to_le_bytes());
        hash.update([
            cone.level().get(),
            cone.promotion().get(),
            cone.superimposition().get(),
        ]);
    } else {
        hash.update([0]);
    }
    encode_len(hash, spec.contributions().len());
    for contribution in spec.contributions() {
        hash.update(contribution.get().to_le_bytes());
    }
    let relics = spec.relic_stats();
    for value in relics
        .base_flats()
        .into_iter()
        .chain(relics.base_ratios())
        .chain(relics.secondary())
        .chain(relics.element_damage_boosts())
    {
        hash.update(value.scaled().to_le_bytes());
    }
}

fn encode_receipt(hash: &mut Sha256, receipt: BuildSubstitutionReceipt) {
    hash.update([
        substitution_kind_tag(receipt.kind()),
        field_source_tag(receipt.progression()),
        field_source_tag(receipt.abilities()),
        field_source_tag(receipt.traces()),
        field_source_tag(receipt.eidolon()),
        field_source_tag(receipt.light_cone()),
        field_source_tag(receipt.contributions()),
    ]);
}

const fn substitution_kind_tag(kind: starclock_build::substitution::BuildSubstitutionKind) -> u8 {
    match kind {
        starclock_build::substitution::BuildSubstitutionKind::Trial => 0,
        starclock_build::substitution::BuildSubstitutionKind::StrengthenedOwned => 1,
    }
}

const fn field_source_tag(source: starclock_build::substitution::BuildFieldSource) -> u8 {
    match source {
        starclock_build::substitution::BuildFieldSource::Owned => 0,
        starclock_build::substitution::BuildFieldSource::MappedMinimum => 1,
        starclock_build::substitution::BuildFieldSource::Combined => 2,
    }
}

fn encode_len(hash: &mut Sha256, length: usize) {
    hash.update(
        u64::try_from(length)
            .expect("snapshot length fits u64")
            .to_le_bytes(),
    );
}

const fn gambit_tag(gambit: CurrencyWarsGambit) -> u8 {
    match gambit {
        CurrencyWarsGambit::Standard => 0,
        CurrencyWarsGambit::Overclock => 1,
    }
}

const fn special_resource_kind_tag(kind: CurrencyWarsSpecialResourceKind) -> u8 {
    match kind {
        CurrencyWarsSpecialResourceKind::EnergyBar => 0,
        CurrencyWarsSpecialResourceKind::MaximumEnergy => 1,
    }
}

const fn skill_parameter_operator_tag(operator: CurrencyWarsSkillParameterOperator) -> u8 {
    match operator {
        CurrencyWarsSkillParameterOperator::Add => 0,
        CurrencyWarsSkillParameterOperator::Multiply => 1,
        CurrencyWarsSkillParameterOperator::Set => 2,
    }
}

const fn lethal_rescue_hp_policy_tag(policy: CurrencyWarsLethalRescueHpPolicy) -> u8 {
    match policy {
        CurrencyWarsLethalRescueHpPolicy::FullMaximumHp => 0,
    }
}
