use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

use starclock_activity::{
    ActivityConfigDigest, ActivityDecisionKind, ActivityDefinitionDigest, ActivityDefinitionId,
    ActivityDefinitionIdentity, ActivityInstanceId, ActivityMasterSeed,
};
use starclock_combat::{
    ActionEventData, AssemblyDigest, Battle, BattleEventKind, BattleSeed, BattleSpec,
    CombatantSpecDigest, Command, ConcedePolicy, DecisionKind, EncounterId, FormationIndex, Hp,
    LinkedEntityKind, ParticipantSource, ParticipantSpec, ResolvedCombatantSpec,
    ResolvedDefinitionBindings, Speed, TeamResourceSpec, TeamSide, UnitEventData,
    catalog::{builder::CombatCatalogBuilder, definition::EncounterDefinition},
};
use starclock_mode_currency_wars::{
    CurrencyWarsAugmentQuality, CurrencyWarsAvatarBattleBehaviorArchetype,
    CurrencyWarsAvatarBattleBehaviorBindingPolicy, CurrencyWarsBattleAssembler,
    CurrencyWarsBattleConfigurationArchetype, CurrencyWarsBattleMaterialization,
    CurrencyWarsBattleOverrideRoleBuild, CurrencyWarsBattleProgramBinding,
    CurrencyWarsBattleProgramBindingArchetype, CurrencyWarsBattleResources,
    CurrencyWarsBondBattleBehaviorArchetype, CurrencyWarsCatalog, CurrencyWarsDeployment,
    CurrencyWarsEnemyBehaviorSource, CurrencyWarsEntryState, CurrencyWarsGambit,
    CurrencyWarsInvestmentKind, CurrencyWarsMechanicProgramDisposition, CurrencyWarsPosition,
    CurrencyWarsPositionKind, CurrencyWarsRoleId, CurrencyWarsRoleState, CurrencyWarsRoster,
    CurrencyWarsRun, CurrencyWarsRunDefinition, CurrencyWarsRunSetup,
};

use crate::{
    currency_wars::load_currency_wars_catalog_candidate, load_currency_wars_battle_resources,
};

#[test]
fn every_m01_policy_binding_executes_a_real_enemy_ai_action() {
    let catalog = load_currency_wars_catalog_candidate()
        .unwrap()
        .into_catalog();
    let resources = load_currency_wars_battle_resources(&catalog).unwrap();
    let programs = resources.battle_behavior_programs();

    assert_eq!(programs.len(), 9);
    assert_eq!(
        programs
            .iter()
            .filter(|program| {
                program.behavior_source == CurrencyWarsEnemyBehaviorSource::SameReleasedFamilyPolicy
            })
            .count(),
        4
    );
    assert_eq!(
        programs
            .iter()
            .filter(|program| {
                program.behavior_source
                    == CurrencyWarsEnemyBehaviorSource::GenericRankFallbackPolicy
            })
            .count(),
        5
    );

    let mut executed = BTreeSet::new();
    for (index, program) in programs.iter().enumerate() {
        let encounter = EncounterId::new(
            0x7d60_0000_u32
                .checked_add(u32::try_from(index).unwrap())
                .unwrap(),
        )
        .unwrap();
        let mut catalog_builder =
            CombatCatalogBuilder::from_catalog(resources.combat(), probe_digest(0x51, index));
        catalog_builder.add_encounter(EncounterDefinition::new(
            encounter,
            vec![program.definition],
            Vec::new(),
        ));
        let combat = catalog_builder.build().unwrap();
        let enemy_definition = combat.enemy(program.definition).unwrap();
        assert!(enemy_definition.ai_graph().is_some());
        assert!(!enemy_definition.abilities().is_empty());
        assert_eq!(enemy_definition.abilities(), program.combatant.abilities());
        let enemy_abilities = enemy_definition.abilities().to_vec();
        let player = probe_player(program.combatant.clone(), index);
        let battle_spec = BattleSpec::new(
            AssemblyDigest::new(probe_digest(0x52, index)).unwrap(),
            encounter,
            vec![
                ParticipantSpec::new(
                    TeamSide::Player,
                    FormationIndex::new(0).unwrap(),
                    ParticipantSource::Player,
                    player,
                ),
                ParticipantSpec::new(
                    TeamSide::Enemy,
                    FormationIndex::new(4).unwrap(),
                    ParticipantSource::EncounterEnemy(program.definition),
                    program.combatant.clone(),
                ),
            ],
            TeamResourceSpec::new(0, 0).unwrap(),
            TeamResourceSpec::new(0, 0).unwrap(),
            ConcedePolicy::Allowed,
        )
        .unwrap();
        let mut battle = Battle::create(
            combat,
            battle_spec,
            BattleSeed::new(probe_digest(0x53, index)),
        )
        .unwrap();
        let enemy = battle
            .view()
            .units_by_id()
            .find(|unit| unit.source() == ParticipantSource::EncounterEnemy(program.definition))
            .unwrap()
            .id();
        let mut command = Command::StartBattle {
            decision: battle.decision().unwrap().id(),
        };
        let mut resolved = false;
        for _ in 0..128 {
            let resolution = battle.apply(command).unwrap();
            assert!(
                resolution
                    .events()
                    .iter()
                    .all(|event| !matches!(event.kind(), BattleEventKind::Fault(_))),
                "{} emitted a battle fault: {:#?}",
                program.source_path,
                resolution.events()
            );
            if resolution.events().iter().any(|event| {
                matches!(
                    event.kind(),
                    BattleEventKind::Action(ActionEventData::Resolved {
                        actor,
                        ability,
                        ..
                    }) if *actor == enemy && enemy_abilities.contains(ability)
                )
            }) {
                resolved = true;
                break;
            }
            command = next_progress_command(&battle);
        }
        assert!(
            resolved,
            "{} never executed its selected AI",
            program.source_path
        );
        executed.insert(program.stable_key.as_ref());
    }
    assert_eq!(executed.len(), programs.len());
}

#[test]
fn every_avatar_policy_binding_reaches_a_real_battle_owned_execution_path() {
    let candidate = load_currency_wars_catalog_candidate().unwrap();
    let identity = candidate_identity(&candidate);
    let catalog = Arc::new(candidate.into_catalog());
    let resources = Arc::new(load_currency_wars_battle_resources(&catalog).unwrap());
    let programs = resources.avatar_battle_behavior_programs();
    assert_eq!(programs.len(), 82);
    assert_eq!(
        programs
            .iter()
            .filter(|program| program.archetype
                == CurrencyWarsAvatarBattleBehaviorArchetype::RoleBattleEvent)
            .count(),
        81
    );
    assert_eq!(
        programs
            .iter()
            .filter(|program| program.archetype
                == CurrencyWarsAvatarBattleBehaviorArchetype::AugmentBattleEvent)
            .count(),
        1
    );
    assert_eq!(
        programs
            .iter()
            .filter(|program| program.binding_policy
                == CurrencyWarsAvatarBattleBehaviorBindingPolicy::ExactBattleEvent)
            .count(),
        77
    );
    assert_eq!(
        programs
            .iter()
            .filter(|program| program.binding_policy
                == CurrencyWarsAvatarBattleBehaviorBindingPolicy::SameFamilyBattleEventFallback)
            .count(),
        4
    );

    let (run, _) = first_materializable_encounter_with(identity, &catalog, &resources, 61, |_| {});
    let base = run.contribution_snapshot().unwrap();
    let deployment = run.deployment().unwrap();
    let builds = base
        .roles
        .iter()
        .map(|role| CurrencyWarsBattleOverrideRoleBuild {
            role: role.role.id,
            technique_ability: catalog
                .build_catalog()
                .trial_build(role.role.id)
                .unwrap()
                .technique_ability,
            eidolon: role.build.eidolon().get(),
        })
        .collect::<Vec<_>>();
    let encounters = catalog.encounter_catalog();
    let difficulty_level = base.augment_enemy_difficulty_add.iter().fold(
        base.difficulty.enemy_scaling.enemy_difficulty_level,
        |level, (_, additional)| level + u16::from(*additional),
    );
    let scaling = encounters
        .enemy_scaling(base.node.plane, difficulty_level)
        .unwrap();
    let mut executed_events = BTreeSet::new();
    for (index, program) in programs
        .iter()
        .filter(|program| {
            program.archetype == CurrencyWarsAvatarBattleBehaviorArchetype::RoleBattleEvent
        })
        .enumerate()
    {
        let overrides = catalog
            .battle_override_snapshot(
                &deployment,
                &builds,
                &program.battle_event_ids,
                base.difficulty.season_id,
                base.battle_overrides.lethal_rescue_action_value,
            )
            .unwrap();
        assert!(program.battle_event_ids.iter().all(|event| {
            overrides
                .back_battle_events
                .iter()
                .any(|resolved| resolved.event_id == *event)
        }));
        let expected_link_count = overrides.back_battle_events.len();
        let mut snapshot = base.clone();
        snapshot.battle_overrides = overrides;
        let mut assembler = CurrencyWarsBattleAssembler::new(Arc::clone(&resources), 1).unwrap();
        let materialized = assembler
            .materialize(&snapshot, encounters, scaling)
            .unwrap_or_else(|error| panic!("{} failed assembly: {error}", program.source_path));
        let mut battle = Battle::create(
            Arc::clone(materialized.combat_catalog()),
            materialized.battle_spec().clone(),
            BattleSeed::new(probe_digest(0x62, index)),
        )
        .unwrap();
        let resolution = battle
            .apply(Command::StartBattle {
                decision: battle.decision().unwrap().id(),
            })
            .unwrap();
        assert!(
            resolution
                .events()
                .iter()
                .all(|event| !matches!(event.kind(), BattleEventKind::Fault(_))),
            "{} emitted a battle fault: {:#?}",
            program.source_path,
            resolution.events()
        );
        assert_eq!(
            resolution
                .events()
                .iter()
                .filter(|event| matches!(
                    event.kind(),
                    BattleEventKind::Unit(UnitEventData::Summoned {
                        kind: LinkedEntityKind::SharedActor,
                        ..
                    })
                ))
                .count(),
            expected_link_count
        );
        assert_eq!(battle.view().links().count(), expected_link_count);
        executed_events.extend(program.battle_event_ids.iter().copied());
    }
    assert_eq!(executed_events.len(), 81);

    let mut selected_augment = None;
    let (augment_run, materialized) =
        first_materializable_encounter_with(identity, &catalog, &resources, 62, |run| {
            let offered = run
                .offer_augments(CurrencyWarsAugmentQuality::Silver)
                .unwrap();
            run.choose_augment(offered[0], None).unwrap();
            selected_augment = Some(offered[0]);
        });
    let selected_augment = selected_augment.unwrap();
    let augment_snapshot = augment_run.contribution_snapshot().unwrap();
    assert!(augment_snapshot.investments.iter().any(|investment| {
        investment.id == selected_augment && investment.kind == CurrencyWarsInvestmentKind::Augment
    }));
    let receipt = materialized.contribution_receipt();
    assert_eq!(
        receipt.augment_policy_modifier_count,
        receipt.front_role_count
    );
    assert_eq!(
        receipt.augment_policy_id(),
        Some("currency-wars.augment-controller-contribution-policy.v1")
    );
    assert!(receipt.augment_policy_replacement_condition().is_some());
    let mut battle = Battle::create(
        Arc::clone(materialized.combat_catalog()),
        materialized.battle_spec().clone(),
        BattleSeed::new([0x63; 32]),
    )
    .unwrap();
    let resolution = battle
        .apply(Command::StartBattle {
            decision: battle.decision().unwrap().id(),
        })
        .unwrap();
    assert!(
        resolution
            .events()
            .iter()
            .all(|event| !matches!(event.kind(), BattleEventKind::Fault(_)))
    );
}

#[test]
fn every_m04_configuration_policy_binds_a_real_materialization_controller() {
    let candidate = load_currency_wars_catalog_candidate().unwrap();
    let identity = candidate_identity(&candidate);
    let catalog = Arc::new(candidate.into_catalog());
    let resources = Arc::new(load_currency_wars_battle_resources(&catalog).unwrap());
    let policies = catalog
        .encounter_catalog()
        .mechanic_programs()
        .iter()
        .filter_map(|program| match &program.disposition {
            CurrencyWarsMechanicProgramDisposition::ExecutedBattleConfigurationPolicy(policy) => {
                Some(policy)
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(policies.len(), 9);

    let materialized = first_materializable_encounter(identity, &catalog, &resources, 63);
    let baseline = materialized.configuration_execution_receipt();
    assert_eq!(baseline.executions().len(), policies.len());
    assert!(baseline.executions().iter().all(|execution| {
        matches!(
            execution.archetype,
            CurrencyWarsBattleConfigurationArchetype::CurrentEquipmentController
        ) || execution.active_binding_count > 0
    }));

    {
        let archetype = CurrencyWarsBattleConfigurationArchetype::CurrentEquipmentController;
        let policy = policies
            .iter()
            .find(|policy| policy.archetype == archetype)
            .unwrap();
        let candidate = catalog
            .build_catalog()
            .equipment()
            .iter()
            .filter_map(|definition| definition.runtime.as_ref())
            .find_map(|equipment| {
                let ability = equipment.ability_name.as_deref()?;
                if !policy
                    .ability_names
                    .iter()
                    .any(|candidate| candidate.as_ref() == ability)
                {
                    return None;
                }
                [1301, 1306, 1014, 1015].into_iter().find_map(|raw| {
                    let role = CurrencyWarsRoleId::new(raw).unwrap();
                    equipment
                        .eligible_for(catalog.role(role).unwrap())
                        .then_some((role, equipment.id))
                })
            })
            .unwrap_or_else(|| panic!("{archetype:?} has no equippable released ability"));
        let (_, equipment_materialized) =
            first_materializable_encounter_with(identity, &catalog, &resources, 64, |run| {
                run.receive_equipment(candidate.1).unwrap();
                run.equip(candidate.0, candidate.1, None).unwrap();
            });
        assert!(
            equipment_materialized
                .configuration_execution_receipt()
                .executions()
                .iter()
                .any(|execution| {
                    execution.archetype == archetype && execution.active_binding_count > 0
                })
        );
    }
}

#[test]
fn every_m05_bond_policy_binds_a_released_bond_materialization_controller() {
    let candidate = load_currency_wars_catalog_candidate().unwrap();
    let identity = candidate_identity(&candidate);
    let catalog = Arc::new(candidate.into_catalog());
    let resources = Arc::new(load_currency_wars_battle_resources(&catalog).unwrap());
    let policies = catalog
        .encounter_catalog()
        .mechanic_programs()
        .iter()
        .filter_map(|program| match &program.disposition {
            CurrencyWarsMechanicProgramDisposition::ExecutedBondBattlePolicy(policy) => {
                Some(policy)
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(policies.len(), 31);
    assert_eq!(
        policies
            .iter()
            .filter(|policy| matches!(
                policy.archetype,
                CurrencyWarsBondBattleBehaviorArchetype::BondStageAbilityController
            ))
            .count(),
        25
    );

    let materialized = first_materializable_encounter(identity, &catalog, &resources, 65);
    let executions = materialized.bond_execution_receipt().executions();
    assert_eq!(executions.len(), policies.len());
    assert!(executions.iter().all(|execution| {
        execution.registered_binding_count > 0
            && usize::from(execution.registered_binding_count) == execution.bond_ids.len()
            && execution
                .bond_ids
                .iter()
                .all(|bond| catalog.bond_catalog().bond(*bond).is_some())
    }));
    assert!(
        executions
            .iter()
            .any(|execution| execution.active_binding_count > 0)
    );
}

#[test]
fn every_m06_program_policy_binds_a_released_runtime_controller() {
    let candidate = load_currency_wars_catalog_candidate().unwrap();
    let identity = candidate_identity(&candidate);
    let catalog = Arc::new(candidate.into_catalog());
    let resources = Arc::new(load_currency_wars_battle_resources(&catalog).unwrap());
    let m06_inputs = resources
        .battle_program_bindings()
        .iter()
        .filter(|input| {
            !is_m07_avatar_binding_source(&input.source_path)
                && !is_m08_avatar_or_battle_event_binding_source(&input.source_path)
                && !is_m09_battle_event_configuration_source(&input.source_path)
        })
        .collect::<Vec<_>>();
    assert_eq!(m06_inputs.len(), 26);
    for (archetype, expected) in [
        (
            CurrencyWarsBattleProgramBindingArchetype::CoreAvatarAbility,
            7,
        ),
        (CurrencyWarsBattleProgramBindingArchetype::ServantAbility, 1),
        (
            CurrencyWarsBattleProgramBindingArchetype::RoleBattleEvent,
            10,
        ),
        (
            CurrencyWarsBattleProgramBindingArchetype::BondStageAbility,
            4,
        ),
        (
            CurrencyWarsBattleProgramBindingArchetype::AugmentStageAbility,
            1,
        ),
        (
            CurrencyWarsBattleProgramBindingArchetype::MonsterTagController,
            2,
        ),
        (
            CurrencyWarsBattleProgramBindingArchetype::EquipmentController,
            1,
        ),
    ] {
        assert_eq!(
            m06_inputs
                .iter()
                .filter(|input| input.archetype == archetype)
                .count(),
            expected
        );
    }
    assert!(
        m06_inputs
            .iter()
            .all(|input| input.runtime_definition_count > 0)
    );

    let materialized = first_materializable_encounter(identity, &catalog, &resources, 65);
    let executions = materialized
        .program_binding_execution_receipt()
        .executions()
        .iter()
        .filter(|execution| {
            !is_m07_avatar_binding_source(&execution.source_path)
                && !is_m08_avatar_or_battle_event_binding_source(&execution.source_path)
                && !is_m09_battle_event_configuration_source(&execution.source_path)
        })
        .collect::<Vec<_>>();
    assert_eq!(executions.len(), 26);
    assert!(executions.iter().all(|execution| {
        execution.registered_binding_count > 0
            && execution.runtime_definition_count > 0
            && usize::from(execution.registered_binding_count) == execution.bindings.len()
    }));
    assert!(
        executions
            .iter()
            .any(|execution| execution.active_binding_count > 0)
    );
}

#[test]
fn every_m07_avatar_program_policy_binds_a_released_runtime_controller() {
    let candidate = load_currency_wars_catalog_candidate().unwrap();
    let identity = candidate_identity(&candidate);
    let catalog = Arc::new(candidate.into_catalog());
    let resources = Arc::new(load_currency_wars_battle_resources(&catalog).unwrap());
    let inputs = resources
        .battle_program_bindings()
        .iter()
        .filter(|input| is_m07_avatar_binding_source(&input.source_path))
        .collect::<Vec<_>>();
    assert_eq!(inputs.len(), 29);
    assert!(
        inputs
            .iter()
            .all(|input| input.runtime_definition_count > 0)
    );

    let materialized = first_materializable_encounter(identity, &catalog, &resources, 97);
    let executions = materialized
        .program_binding_execution_receipt()
        .executions()
        .iter()
        .filter(|execution| is_m07_avatar_binding_source(&execution.source_path))
        .collect::<Vec<_>>();
    assert_eq!(executions.len(), 29);
    assert!(executions.iter().all(|execution| {
        execution.registered_binding_count > 0
            && execution.runtime_definition_count > 0
            && usize::from(execution.registered_binding_count) == execution.bindings.len()
    }));
    assert!(
        executions
            .iter()
            .any(|execution| execution.active_binding_count > 0)
    );
    let common = materialized
        .configuration_execution_receipt()
        .executions()
        .iter()
        .find(|execution| execution.source_path.as_ref() == M07_COMMON_AVATAR_SOURCE)
        .expect("M07 common Avatar controller emits a materialization receipt");
    assert_eq!(
        common.archetype,
        CurrencyWarsBattleConfigurationArchetype::CommonBattleKernel
    );
    assert_eq!(common.active_binding_count, 1);
}

#[test]
fn every_m08_avatar_and_battle_event_program_binds_a_released_runtime_controller() {
    let candidate = load_currency_wars_catalog_candidate().unwrap();
    let identity = candidate_identity(&candidate);
    let catalog = Arc::new(candidate.into_catalog());
    let resources = Arc::new(load_currency_wars_battle_resources(&catalog).unwrap());
    let inputs = resources
        .battle_program_bindings()
        .iter()
        .filter(|input| is_m08_avatar_or_battle_event_binding_source(&input.source_path))
        .collect::<Vec<_>>();
    assert_eq!(inputs.len(), 35);
    assert!(
        inputs
            .iter()
            .all(|input| input.runtime_definition_count > 0)
    );
    let partner = inputs
        .iter()
        .find(|input| {
            input
                .source_path
                .ends_with("BattleEvent_GridFight_Cocolia_Partner_00_Config.json")
        })
        .unwrap();
    assert_eq!(partner.runtime_definition_count, 1);
    let summon = inputs
        .iter()
        .find(|input| {
            input
                .source_path
                .ends_with("BattleEvent_GridFight_DanHengPT_00_BE_Config.json")
        })
        .unwrap();
    assert!(summon.bindings.iter().any(|binding| matches!(
        binding,
        CurrencyWarsBattleProgramBinding::BattleEvent(11_414)
    )));

    let materialized = first_materializable_encounter(identity, &catalog, &resources, 129);
    let executions = materialized
        .program_binding_execution_receipt()
        .executions()
        .iter()
        .filter(|execution| is_m08_avatar_or_battle_event_binding_source(&execution.source_path))
        .collect::<Vec<_>>();
    assert_eq!(executions.len(), 35);
    assert!(executions.iter().all(|execution| {
        execution.registered_binding_count > 0
            && execution.runtime_definition_count > 0
            && usize::from(execution.registered_binding_count) == execution.bindings.len()
    }));
    assert!(
        executions
            .iter()
            .any(|execution| execution.active_binding_count > 0)
    );
}

#[test]
fn every_m09_battle_event_configuration_binds_a_released_runtime_controller() {
    let candidate = load_currency_wars_catalog_candidate().unwrap();
    let identity = candidate_identity(&candidate);
    let catalog = Arc::new(candidate.into_catalog());
    let resources = Arc::new(load_currency_wars_battle_resources(&catalog).unwrap());
    let inputs = resources
        .battle_program_bindings()
        .iter()
        .filter(|input| is_m09_battle_event_configuration_source(&input.source_path))
        .collect::<Vec<_>>();
    assert_eq!(inputs.len(), 42);
    assert!(
        inputs
            .iter()
            .all(|input| input.runtime_definition_count > 0)
    );
    let no_action_delay = inputs
        .iter()
        .find(|input| {
            input
                .source_path
                .ends_with("BattleEvent_GridFight_NoActionDelay_Config.json")
        })
        .unwrap();
    assert_eq!(no_action_delay.runtime_definition_count, 43);
    assert_eq!(
        no_action_delay
            .bindings
            .iter()
            .filter(|binding| matches!(binding, CurrencyWarsBattleProgramBinding::BattleEvent(_)))
            .count(),
        43
    );
    let summoner = inputs
        .iter()
        .find(|input| {
            input
                .source_path
                .ends_with("BattleEvent_GridFight_TheHerta_00_Summoner01_Config.json")
        })
        .unwrap();
    assert_eq!(summoner.runtime_definition_count, 1);

    let materialized = first_materializable_encounter(identity, &catalog, &resources, 161);
    let executions = materialized
        .program_binding_execution_receipt()
        .executions()
        .iter()
        .filter(|execution| is_m09_battle_event_configuration_source(&execution.source_path))
        .collect::<Vec<_>>();
    assert_eq!(executions.len(), 42);
    assert!(executions.iter().all(|execution| {
        execution.registered_binding_count > 0
            && execution.runtime_definition_count > 0
            && usize::from(execution.registered_binding_count) == execution.bindings.len()
    }));
    assert!(
        executions
            .iter()
            .any(|execution| execution.active_binding_count > 0)
    );
}

#[test]
fn every_m10_enemy_character_configuration_reaches_battle_assembly() {
    let candidate = load_currency_wars_catalog_candidate().unwrap();
    let identity = candidate_identity(&candidate);
    let catalog = Arc::new(candidate.into_catalog());
    let resources = Arc::new(load_currency_wars_battle_resources(&catalog).unwrap());
    let inputs = resources.enemy_character_configurations();
    assert_eq!(inputs.len(), 11);
    assert!(inputs.iter().all(|input| {
        input.bindings.len() == 1
            && input.bindings.iter().all(|binding| {
                resources
                    .combat()
                    .enemy(binding.definition)
                    .is_some_and(|definition| {
                        definition.ai_graph().is_some() && !definition.abilities().is_empty()
                    })
            })
    }));
    let materialized = first_materializable_encounter(identity, &catalog, &resources, 193);
    let executions = materialized
        .enemy_character_configuration_execution_receipt()
        .executions();
    assert_eq!(executions.len(), 11);
    assert!(executions.iter().all(|execution| {
        execution.registered_binding_count == 1 && execution.runtime_definition_count == 1
    }));
    assert!(
        executions
            .iter()
            .all(|execution| execution.active_binding_count <= 1)
    );
}

#[test]
fn every_m12_enemy_ai_configuration_reaches_battle_assembly() {
    let candidate = load_currency_wars_catalog_candidate().unwrap();
    let identity = candidate_identity(&candidate);
    let catalog = Arc::new(candidate.into_catalog());
    let resources = Arc::new(load_currency_wars_battle_resources(&catalog).unwrap());
    let inputs = resources.enemy_ai_configurations();
    assert_eq!(inputs.len(), 3);
    assert_eq!(
        inputs
            .iter()
            .map(|input| input.bindings.len())
            .sum::<usize>(),
        4
    );
    assert!(inputs.iter().all(|input| {
        input.bindings.iter().all(|binding| {
            resources
                .combat()
                .enemy(binding.definition)
                .is_some_and(|definition| {
                    definition.ai_graph().is_some() && !definition.abilities().is_empty()
                })
        })
    }));
    let materialized = first_materializable_encounter(identity, &catalog, &resources, 211);
    let executions = materialized
        .enemy_ai_configuration_execution_receipt()
        .executions();
    assert_eq!(executions.len(), 3);
    assert_eq!(
        executions
            .iter()
            .map(|execution| usize::from(execution.registered_binding_count))
            .sum::<usize>(),
        4
    );
    assert!(executions.iter().all(|execution| {
        execution.registered_binding_count == execution.runtime_definition_count
            && execution.active_binding_count <= execution.registered_binding_count
    }));
}

const M07_AVATAR_PREFIX: &str = "Config/ConfigAbility/GridFight/3.5/Avatar_GridFight_";
const M07_COMMON_AVATAR_SOURCE: &str =
    "Config/ConfigAbility/GridFight/3.5/Avatar_GridFight_Common_00_Ability.json";

fn is_m07_avatar_binding_source(source_path: &str) -> bool {
    const STEMS: [&str; 29] = [
        "Boothill_00",
        "Bronya_00",
        "Castorice_00",
        "Cerydra_00",
        "Cipher_00",
        "Cyrene_00",
        "DanHengIL_00",
        "Dr_Ratio_00",
        "Evernight_00",
        "Feixiao_00",
        "Gallagher_00",
        "Gepard_00",
        "Guinaifen_00",
        "Harscyline_00",
        "Herta_00",
        "Huohuo_00",
        "Hyacine_00",
        "HyacineServant_00",
        "Jiaoqiu_00",
        "JingYuan_00",
        "Kafka_00",
        "Lingsha_00",
        "Mydeimos_00",
        "Natasha_00",
        "Phainon_00",
        "PlayerBoy_30",
        "PlayerBoyServant_30",
        "Qingque_00",
        "Rappa_00",
    ];
    source_path
        .strip_prefix(M07_AVATAR_PREFIX)
        .and_then(|value| value.strip_suffix("_Ability.json"))
        .is_some_and(|stem| STEMS.contains(&stem))
}

fn is_m08_avatar_or_battle_event_binding_source(source_path: &str) -> bool {
    let avatar = source_path
        .strip_prefix("Config/ConfigAbility/GridFight/")
        .and_then(|remainder| remainder.split_once('/'))
        .and_then(|(version, file)| {
            file.strip_prefix("Avatar_GridFight_")
                .and_then(|value| value.strip_suffix("_Ability.json"))
                .map(|stem| (version, stem))
        });
    const AVATARS: [(&str, &str); 15] = [
        ("3.5", "Saber_00"),
        ("3.5", "Sam_00"),
        ("3.5", "Sampo_00"),
        ("3.5", "Seele_00"),
        ("3.5", "Silwolf_00"),
        ("3.5", "Sunday_10"),
        ("3.5", "TheHerta_00"),
        ("3.5", "Tribbie_00"),
        ("3.5", "Welt_00"),
        ("4.0", "Constance_00"),
        ("4.0", "Sparxie_00"),
        ("4.0", "YaoGuang_00"),
        ("4.2", "Ashveil_00"),
        ("4.2", "Evanescia_00"),
        ("4.2", "SilverWolf999_00"),
    ];
    if avatar.is_some_and(|value| AVATARS.contains(&value)) {
        return true;
    }
    let battle_event = source_path
        .strip_prefix(
            "Config/ConfigCharacter/BattleEvent/GridFight/3.5/AvatarConfig/BattleEvent_GridFight_",
        )
        .and_then(|value| value.strip_suffix("_Config.json"));
    const BATTLE_EVENTS: [&str; 20] = [
        "Anaxa_00",
        "BlackSwan_00",
        "BloodTrait_Attack_00",
        "BloodTrait_Start_00",
        "Cerydra_00",
        "Cocolia_00",
        "Cocolia_Partner_00",
        "DanHengPT_00_BE",
        "DanHengPT_00",
        "Evernight_00",
        "EvernightServant_00",
        "Fugue_00",
        "FuXuan_00",
        "Gallagher_00",
        "Guinaifen_00",
        "Herta_00",
        "Himeko_00",
        "Huohuo_00",
        "Jade_00",
        "Jingliu_00",
    ];
    battle_event.is_some_and(|stem| BATTLE_EVENTS.contains(&stem))
}

fn is_m09_battle_event_configuration_source(source_path: &str) -> bool {
    let Some(remainder) = source_path.strip_prefix("Config/ConfigCharacter/BattleEvent/GridFight/")
    else {
        return false;
    };
    let Some((version, remainder)) = remainder.split_once('/') else {
        return false;
    };
    let Some((category, file)) = remainder.split_once('/') else {
        return false;
    };
    let Some(stem) = file
        .strip_prefix("BattleEvent_GridFight_")
        .and_then(|value| value.strip_suffix("_Config.json"))
    else {
        return false;
    };
    match (version, category) {
        ("3.5", "AvatarConfig") => [
            "Luocha_00",
            "Mar_7th_00",
            "Moze_00",
            "Mydeimos_00",
            "Natasha_00",
            "NoActionDelay",
            "Pela_00",
            "PlayerBoy_30",
            "PlayerGirl_30",
            "Ren_00",
            "Robin_00",
            "RuanMei_00",
            "Saber_00",
            "Sampo_00",
            "Silwolf_00",
            "Sparkle_00",
            "SPTraitMonster_00",
            "TheHerta_00_Summoner01",
            "Tingyun_00",
            "Topaz_00_BE",
            "Topaz_00",
            "Tribbie_00",
            "Yanqing_00",
            "Yunli_00",
        ]
        .contains(&stem),
        ("3.5", "OriginConfig") => [
            "Origin_1001",
            "Origin_1005",
            "Origin_1007_Augment_35402041",
            "Origin_1007",
            "Origin_1008_00",
            "Origin_1008_01",
            "Origin_1008_02",
            "Origin_1008_03",
        ]
        .contains(&stem),
        ("4.0", "AvatarConfig") => [
            "Argenti_00",
            "Constance_00",
            "Sparxie_ExtraElation",
            "YaoGuang_00",
        ]
        .contains(&stem),
        ("4.2", "AvatarConfig") => [
            "Ashveil_00",
            "Evanescia_00",
            "Kafka_00",
            "PlayerBoy_40",
            "PlayerGirl_40",
        ]
        .contains(&stem),
        ("4.2", "OriginConfig") => stem == "Augment_35402045",
        _ => false,
    }
}

fn first_materializable_encounter(
    identity: ActivityDefinitionIdentity,
    catalog: &Arc<CurrencyWarsCatalog>,
    resources: &Arc<CurrencyWarsBattleResources>,
    first_seed: u64,
) -> CurrencyWarsBattleMaterialization {
    first_materializable_encounter_with(identity, catalog, resources, first_seed, |_| {}).1
}

fn first_materializable_encounter_with(
    identity: ActivityDefinitionIdentity,
    catalog: &Arc<CurrencyWarsCatalog>,
    resources: &Arc<CurrencyWarsBattleResources>,
    first_seed: u64,
    mut prepare: impl FnMut(&mut CurrencyWarsRun),
) -> (CurrencyWarsRun, CurrencyWarsBattleMaterialization) {
    (0..catalog.routes().len())
        .find_map(|route_offset| {
            let seed = first_seed + u64::try_from(route_offset).ok()?;
            let mut run =
                production_run_at_route(identity, Arc::clone(catalog), seed, route_offset);
            prepare(&mut run);
            advance_to_encounter(&mut run);
            let snapshot = run.contribution_snapshot().ok()?;
            let encounters = catalog.encounter_catalog();
            let scaling = encounters.enemy_scaling(
                snapshot.node.plane,
                snapshot.difficulty.enemy_scaling.enemy_difficulty_level,
            )?;
            let mut assembler = CurrencyWarsBattleAssembler::new(Arc::clone(resources), 1).ok()?;
            let materialized = assembler.materialize(&snapshot, encounters, scaling).ok()?;
            Some((run, materialized))
        })
        .expect("bounded deterministic probes contain a materializable encounter")
}

fn candidate_identity(
    candidate: &crate::currency_wars::CurrencyWarsCatalogCandidate,
) -> ActivityDefinitionIdentity {
    ActivityDefinitionIdentity::new(
        ActivityDefinitionId::new(62).unwrap(),
        ActivityDefinitionDigest::new(candidate.identity().content_digest().bytes()).unwrap(),
        ActivityConfigDigest::new(candidate.identity().configuration_digest().bytes()).unwrap(),
    )
}

fn production_run_at_route(
    identity: ActivityDefinitionIdentity,
    catalog: Arc<CurrencyWarsCatalog>,
    seed: u64,
    route_offset: usize,
) -> CurrencyWarsRun {
    let difficulty = catalog
        .difficulties()
        .iter()
        .find(|difficulty| difficulty.division_level == 3)
        .unwrap();
    let roles = [1301, 1306, 1014, 1015]
        .map(|raw| CurrencyWarsRoleState::new(CurrencyWarsRoleId::new(raw).unwrap(), 1).unwrap());
    let roster = CurrencyWarsRoster::new(&catalog, roles.map(|role| (role, 1))).unwrap();
    let deployment = CurrencyWarsDeployment::new(
        &catalog,
        &roster,
        4,
        [
            position(CurrencyWarsPositionKind::Front, 1),
            position(CurrencyWarsPositionKind::Back, 1),
            position(CurrencyWarsPositionKind::Back, 2),
            position(CurrencyWarsPositionKind::Back, 3),
        ]
        .into_iter()
        .zip(roles),
    )
    .unwrap();
    let routes = catalog.routes();
    let definition = routes
        .iter()
        .cycle()
        .skip(route_offset % routes.len())
        .take(routes.len())
        .find_map(|route| {
            CurrencyWarsRunDefinition::new(
                identity,
                Arc::clone(&catalog),
                route.id,
                difficulty.source_id,
                CurrencyWarsGambit::Standard,
                CurrencyWarsEntryState::new(21, true, 9),
                CurrencyWarsRunSetup {
                    initial_gold: 100,
                    initial_team_level: 4,
                    initial_experience: 0,
                    roster: roster.clone(),
                    deployment: deployment.clone(),
                    enemy_affix_ids: Box::new([]),
                    owned_builds: BTreeMap::new(),
                },
            )
            .ok()
        })
        .map(Arc::new)
        .unwrap();
    CurrencyWarsRun::start(
        definition,
        ActivityInstanceId::new(seed).unwrap(),
        ActivityMasterSeed::from_u64(seed),
    )
    .unwrap()
}

fn advance_to_encounter(run: &mut CurrencyWarsRun) {
    while run.player_view().decision().unwrap().kind() != ActivityDecisionKind::Encounter {
        match run.player_view().decision().unwrap().kind() {
            ActivityDecisionKind::Shop => run.continue_supply().unwrap(),
            ActivityDecisionKind::Route => run.continue_plane().unwrap(),
            kind => panic!("unexpected avatar policy probe decision: {kind:?}"),
        }
    }
}

fn position(kind: CurrencyWarsPositionKind, slot: u8) -> CurrencyWarsPosition {
    CurrencyWarsPosition::new(kind, slot).unwrap()
}

fn probe_player(template: ResolvedCombatantSpec, index: usize) -> ResolvedCombatantSpec {
    ResolvedCombatantSpec::new(
        template.form(),
        template.level(),
        Hp::new(1_000_000_000).unwrap(),
        Speed::from_scaled(1).unwrap(),
        ResolvedDefinitionBindings::new(template.abilities().to_vec(), Vec::new(), Vec::new())
            .unwrap(),
        CombatantSpecDigest::new(probe_digest(0x54, index)).unwrap(),
    )
    .unwrap()
}

fn next_progress_command(battle: &Battle) -> Command {
    if let Some(command) = battle.advance_command() {
        return command;
    }
    let decision = battle
        .decision()
        .expect("nonterminal policy probe exposes a decision");
    match decision.kind() {
        DecisionKind::NormalAction => decision
            .legal_commands()
            .iter()
            .find(|command| matches!(command, Command::UseAbility { .. })),
        DecisionKind::PreparedAction => decision
            .legal_commands()
            .iter()
            .find(|command| matches!(command, Command::CommitPreparedAction { .. })),
        DecisionKind::ActionFrame | DecisionKind::BattleChoice => decision
            .legal_commands()
            .iter()
            .find(|command| matches!(command, Command::CommitActionFrame { .. })),
        DecisionKind::BattleStart => decision
            .legal_commands()
            .iter()
            .find(|command| matches!(command, Command::StartBattle { .. })),
    }
    .cloned()
    .expect("policy probe exposes a deterministic progress command")
}

fn probe_digest(domain: u8, index: usize) -> [u8; 32] {
    let mut digest = [domain; 32];
    digest[0] = domain;
    digest[1..9].copy_from_slice(&u64::try_from(index + 1).unwrap().to_le_bytes());
    digest
}
