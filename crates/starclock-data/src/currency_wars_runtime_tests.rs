use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use starclock_activity::{
    ActivityBattleHandoff, ActivityConfigDigest, ActivityDecisionKind, ActivityDefinitionDigest,
    ActivityDefinitionId, ActivityDefinitionIdentity, ActivityInstanceId, ActivityMasterSeed,
    ActivityTerminalOutcome, AttemptId, BattleOutcome, BattleResult, EventDigest, MetricValue,
    ParticipantBattleState, ProjectedValue,
};
use starclock_combat::{
    ActionEventData, Battle, BattleClockSpec, BattleEventKind, BattleSeed, BattleStateHash,
    Command, LifeState, LinkedEntityKind, ParticipantSource, PresenceState, Scalar, UnitEventData,
    UnitLevel,
};
use starclock_mode_currency_wars::{
    CURRENCY_WARS_ACTION_VALUE_REMAINING_KEY, CURRENCY_WARS_BATTLE_PROGRESS_KEY,
    CurrencyWarsBattleAssembler, CurrencyWarsBondId, CurrencyWarsBondResolutionContext,
    CurrencyWarsDeployment, CurrencyWarsEncounterSelectionReceipt,
    CurrencyWarsEnemyAffixDefinition, CurrencyWarsEnemyAffixSelectionSource,
    CurrencyWarsEnemySlotDefinition, CurrencyWarsEntryState, CurrencyWarsEquipmentLoadout,
    CurrencyWarsGambit, CurrencyWarsNodeKind, CurrencyWarsPosition, CurrencyWarsPositionKind,
    CurrencyWarsRoleId, CurrencyWarsRoleState, CurrencyWarsRoster, CurrencyWarsRun,
    CurrencyWarsRunDefinition, CurrencyWarsRunSetup,
};

use crate::{
    currency_wars::load_currency_wars_catalog_candidate, load_currency_wars_battle_resources,
};

#[test]
fn production_encounter_and_enemy_inputs_close_the_released_denominators() {
    let candidate = load_currency_wars_catalog_candidate().unwrap();
    let catalog = candidate.into_catalog();
    let encounters = catalog.encounter_catalog();
    let stages = encounters.released_stages().collect::<Vec<_>>();
    assert_eq!(stages.len(), 840);
    assert_eq!(encounters.enemy_scalings().count(), 603);
    assert!((1_u8..=5).all(|maximum| {
        encounters
            .formation_wave(maximum)
            .is_some_and(|wave| wave.maximum_teammates == maximum)
    }));
    assert!(encounters.formation_wave(0).is_none());
    assert!(encounters.formation_wave(6).is_none());
    let star_one = encounters.enemy_star_scaling(100_203_025, 1).unwrap();
    let star_four = encounters.enemy_star_scaling(100_203_025, 4).unwrap();
    assert_eq!(star_one.hp.scaled(), 1_000_000);
    assert_eq!(star_one.attack.scaled(), 1_000_000);
    assert_eq!(star_four.hp.scaled(), 8_000_000);
    assert_eq!(star_four.attack.scaled(), 1_600_000);
    assert_eq!(star_four.speed.scaled(), 1_500_000);
    assert!(encounters.enemy_star_scaling(100_203_025, 0).is_none());
    assert!(encounters.enemy_star_scaling(100_203_025, 5).is_none());

    let stage_levels = stages
        .iter()
        .map(|stage| stage.level)
        .collect::<BTreeSet<_>>();
    let stable_keys = encounters
        .enemy_slots
        .iter()
        .filter_map(|slot| match &slot.definition {
            CurrencyWarsEnemySlotDefinition::Monster {
                shared_enemy_key, ..
            } => Some(shared_enemy_key.as_ref()),
            CurrencyWarsEnemySlotDefinition::EliteScaling { .. } => None,
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(stable_keys.len(), 160);
    let expected_inputs = stable_keys
        .iter()
        .flat_map(|stable_key| stage_levels.iter().map(move |level| (*stable_key, *level)))
        .collect::<BTreeSet<_>>();
    let resources = load_currency_wars_battle_resources(&catalog).unwrap();
    assert_eq!(resources.enemy_input_count(), expected_inputs.len());
    assert!(expected_inputs.iter().all(|(stable_key, level)| {
        resources.contains_enemy_input(stable_key, UnitLevel::new(*level).unwrap())
    }));
    assert!(resources.behavior_fallback_input_count() > 0);
    assert_eq!(
        resources.behavior_fallback_input_count(),
        resources.enemy_input_count()
    );
    assert_eq!(resources.same_family_behavior_input_count(), 1_020);
    assert_eq!(resources.generic_behavior_fallback_input_count(), 1_380);
    assert!(resources.stat_fallback_input_count() > 0);
    assert!(resources.stat_fallback_input_count() < resources.enemy_input_count());
}

#[test]
fn production_enemy_affix_reaction_families_materialize_as_executable_rule_ir() {
    let candidate = load_currency_wars_catalog_candidate().unwrap();
    let identity = ActivityDefinitionIdentity::new(
        ActivityDefinitionId::new(24).unwrap(),
        ActivityDefinitionDigest::new(candidate.identity().content_digest().bytes()).unwrap(),
        ActivityConfigDigest::new(candidate.identity().configuration_digest().bytes()).unwrap(),
    );
    let catalog = Arc::new(candidate.into_catalog());
    let difficulty = catalog
        .difficulties()
        .iter()
        .find(|difficulty| difficulty.division_level == 9)
        .unwrap();
    let resources = Arc::new(load_currency_wars_battle_resources(&catalog).unwrap());
    let mut assembler = CurrencyWarsBattleAssembler::new(resources, 8).unwrap();
    let encounters = catalog.encounter_catalog();
    let expected_affixes = encounters
        .enemy_affix_definitions()
        .filter_map(|affix| match affix.definition {
            CurrencyWarsEnemyAffixDefinition::Affix { source_id, .. } => Some(source_id),
            CurrencyWarsEnemyAffixDefinition::MazeBuff { .. }
            | CurrencyWarsEnemyAffixDefinition::Scaling(_) => None,
        })
        .collect::<BTreeSet<_>>();
    let mut executed_affixes = BTreeSet::new();
    let mut saw_time_assassin_spawn = false;
    let mut saw_time_assassin_no_spawn = false;

    let probes = [
        (1_u64, [2002, 2005, 4001, 4003]),
        (2_u64, [3006, 4007, 4010, 4017]),
        (3_u64, [4006, 4016, 4017, 4021]),
        (4_u64, [2003, 2004, 4002, 4022]),
        (5_u64, [4005, 4012, 4015, 4026]),
        (6_u64, [3008, 4008, 4023, 4024]),
        (7_u64, [3001, 40140, 40141, 40142]),
        (13_u64, [1001, 2006, 3002, 3003]),
        (14_u64, [1002, 3004, 40143, 40144]),
        (15_u64, [1003, 3005, 40145, 40146]),
        (16_u64, [1004, 4018, 4019, 4020]),
        (17_u64, [1005, 4025, 4027, 4028]),
    ]
    .into_iter()
    .chain((19_u64..=50).map(|probe| (probe, [3007, 4009, 4011, 4013])));

    for (probe, affixes) in probes {
        executed_affixes.extend(affixes);
        let roles = [1004, 1001, 1003, 1508].map(|raw| {
            CurrencyWarsRoleState::new(CurrencyWarsRoleId::new(raw).unwrap(), 1).unwrap()
        });
        let roster =
            CurrencyWarsRoster::new(&catalog, roles.into_iter().map(|state| (state, 1))).unwrap();
        let deployment = CurrencyWarsDeployment::new(
            &catalog,
            &roster,
            4,
            [
                position(CurrencyWarsPositionKind::Front, 1),
                position(CurrencyWarsPositionKind::Front, 2),
                position(CurrencyWarsPositionKind::Front, 3),
                position(CurrencyWarsPositionKind::Front, 4),
            ]
            .into_iter()
            .zip(roles),
        )
        .unwrap();
        let definition = catalog
            .routes()
            .iter()
            .find_map(|route| {
                CurrencyWarsRunDefinition::new(
                    identity,
                    Arc::clone(&catalog),
                    route.id,
                    difficulty.source_id,
                    CurrencyWarsGambit::Standard,
                    CurrencyWarsEntryState::new(21, true, 9),
                    CurrencyWarsRunSetup {
                        initial_gold: 0,
                        initial_team_level: 4,
                        initial_experience: 0,
                        roster: roster.clone(),
                        deployment: deployment.clone(),
                        enemy_affix_ids: affixes.into(),
                        owned_builds: BTreeMap::new(),
                    },
                )
                .ok()
            })
            .map(Arc::new)
            .unwrap_or_else(|| panic!("Affix probe {probe} has no legal released route"));
        let mut run = CurrencyWarsRun::start(
            definition,
            ActivityInstanceId::new(24 + probe).unwrap(),
            ActivityMasterSeed::from_u64(24_000_500 + probe),
        )
        .unwrap();
        while run.player_view().decision().unwrap().kind() != ActivityDecisionKind::Encounter {
            match run.player_view().decision().unwrap().kind() {
                ActivityDecisionKind::Shop => run.continue_supply().unwrap(),
                ActivityDecisionKind::Route => run.continue_plane().unwrap(),
                kind => panic!("unexpected Affix probe decision: {kind:?}"),
            }
        }
        let snapshot = run.contribution_snapshot().unwrap();
        let difficulty_level = snapshot.augment_enemy_difficulty_add.iter().fold(
            snapshot.difficulty.enemy_scaling.enemy_difficulty_level,
            |level, (_, additional)| level + u16::from(*additional),
        );
        let scaling = encounters
            .enemy_scaling(snapshot.node.plane, difficulty_level)
            .unwrap();
        let materialized = assembler
            .materialize(&snapshot, encounters, scaling)
            .unwrap();
        let selection = materialized.selection();
        assert_eq!(selection.selected_enemy_affix_ids.as_ref(), affixes);
        if affixes.contains(&4013) {
            let spawned = selection.selected_monster_ids.contains(&4_032_028);
            saw_time_assassin_spawn |= spawned;
            saw_time_assassin_no_spawn |= !spawned;
            assert_eq!(
                selection
                    .formation_wave_limits
                    .iter()
                    .map(|limit| usize::from(*limit))
                    .sum::<usize>(),
                selection.selected_monster_ids.len()
            );
            assert_eq!(
                materialized
                    .battle_spec()
                    .participants()
                    .iter()
                    .filter(|participant| {
                        matches!(participant.source(), ParticipantSource::EncounterEnemy(_))
                    })
                    .count(),
                selection.selected_monster_ids.len()
            );
            assert_eq!(
                selection.time_assassin_policy_id(),
                Some("currency-wars.time-assassin-spawn-policy.v1")
            );
            assert!(selection.time_assassin_replacement_condition().is_some());
        }
        let mut battle = Battle::create(
            Arc::clone(materialized.combat_catalog()),
            materialized.battle_spec().clone(),
            BattleSeed::new([u8::try_from(probe).unwrap(); 32]),
        )
        .unwrap();
        let resolution = battle
            .apply(Command::StartBattle {
                decision: battle.decision().unwrap().id(),
            })
            .unwrap();
        if matches!(probe, 1 | 4 | 5 | 6 | 7) || probe >= 8 {
            assert!(
                !resolution
                    .events()
                    .iter()
                    .any(|event| matches!(event.kind(), BattleEventKind::Fault(_)))
            );
        } else {
            assert!(
                resolution
                    .events()
                    .iter()
                    .any(|event| matches!(event.kind(), BattleEventKind::HpConsumption(_))),
                "{:#?}",
                resolution.events()
            );
        }
    }
    assert!(saw_time_assassin_spawn);
    assert!(saw_time_assassin_no_spawn);
    assert_eq!(executed_affixes, expected_affixes);
}

#[test]
fn production_standard_route_executes_economy_roster_battles_and_terminal_settlement() {
    let candidate = load_currency_wars_catalog_candidate().unwrap();
    let identity = ActivityDefinitionIdentity::new(
        ActivityDefinitionId::new(21).unwrap(),
        ActivityDefinitionDigest::new(candidate.identity().content_digest().bytes()).unwrap(),
        ActivityConfigDigest::new(candidate.identity().configuration_digest().bytes()).unwrap(),
    );
    let catalog = Arc::new(candidate.into_catalog());
    let route = catalog
        .routes()
        .iter()
        .find(|route| route.stable_key.as_ref() == "currency-wars.area.route.100")
        .unwrap();
    let difficulty = catalog
        .difficulties()
        .iter()
        .find(|difficulty| difficulty.division_level == 3)
        .unwrap();
    let selected_roles = [1301, 1306, 1014, 1015]
        .map(|raw| CurrencyWarsRoleState::new(CurrencyWarsRoleId::new(raw).unwrap(), 1).unwrap());
    let roster =
        CurrencyWarsRoster::new(&catalog, selected_roles.into_iter().map(|state| (state, 1)))
            .unwrap();
    let positions = [
        position(CurrencyWarsPositionKind::Front, 1),
        position(CurrencyWarsPositionKind::Back, 1),
        position(CurrencyWarsPositionKind::Back, 2),
        position(CurrencyWarsPositionKind::Back, 3),
    ];
    let deployment = CurrencyWarsDeployment::new(
        &catalog,
        &roster,
        4,
        positions.into_iter().zip(selected_roles),
    )
    .unwrap();
    assert!(!deployment.bond_levels(&catalog).is_empty());
    let definition = Arc::new(
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
                roster,
                deployment,
                enemy_affix_ids: Box::new([2005]),
                owned_builds: BTreeMap::new(),
            },
        )
        .unwrap(),
    );
    let mut run = CurrencyWarsRun::start(
        definition,
        ActivityInstanceId::new(21).unwrap(),
        ActivityMasterSeed::from_u64(21_000_501),
    )
    .unwrap();
    let resources = Arc::new(load_currency_wars_battle_resources(&catalog).unwrap());
    let mut assembler = CurrencyWarsBattleAssembler::new(Arc::clone(&resources), 32).unwrap();

    let front_one = position(CurrencyWarsPositionKind::Front, 1);
    let back_four = position(CurrencyWarsPositionKind::Back, 4);
    run.undeploy(front_one).unwrap();
    run.deploy(back_four, selected_roles[0]).unwrap();
    let rejected_state = run.state_hash();
    let rejected_rng = run.debug_view().rng().to_vec();
    let rejected_cache = assembler.cache_stats();
    assert!(
        run.engage_current_node(AttemptId::new(999).unwrap(), &mut assembler)
            .is_err()
    );
    assert_eq!(run.state_hash(), rejected_state);
    assert_eq!(run.debug_view().rng(), rejected_rng);
    assert_eq!(assembler.cache_stats(), rejected_cache);
    run.undeploy(back_four).unwrap();
    run.deploy(front_one, selected_roles[0]).unwrap();

    let initial_hash = run.state_hash();
    let offers = run.refresh_shop().unwrap();
    let purchased = offers[0];
    let copies_before = run
        .roster()
        .unwrap()
        .base_copy_count(&catalog, purchased.role())
        .unwrap();
    assert!(copies_before < 2);
    run.buy_shop_offer(purchased).unwrap();
    assert_eq!(
        run.roster()
            .unwrap()
            .base_copy_count(&catalog, purchased.role())
            .unwrap(),
        copies_before + 1
    );
    assert_ne!(run.state_hash(), initial_hash);

    run.undeploy(position(CurrencyWarsPositionKind::Back, 3))
        .unwrap();
    run.deploy(
        position(CurrencyWarsPositionKind::Back, 4),
        selected_roles[3],
    )
    .unwrap();
    assert!(!run.deployment().unwrap().bond_levels(&catalog).is_empty());
    run.undeploy(position(CurrencyWarsPositionKind::Front, 1))
        .unwrap();
    run.undeploy(position(CurrencyWarsPositionKind::Back, 1))
        .unwrap();
    run.deploy(
        position(CurrencyWarsPositionKind::Front, 1),
        selected_roles[1],
    )
    .unwrap();
    run.deploy(
        position(CurrencyWarsPositionKind::Back, 1),
        selected_roles[0],
    )
    .unwrap();
    let front_role = catalog.role(selected_roles[1].role()).unwrap();
    let equipment = catalog
        .build_catalog()
        .equipment()
        .iter()
        .filter_map(|definition| definition.runtime.as_ref())
        .find(|equipment| !equipment.properties.is_empty() && equipment.eligible_for(front_role))
        .unwrap();
    run.receive_equipment(equipment.id).unwrap();
    run.equip(front_role.id, equipment.id, None).unwrap();

    let mut battles = 0_u32;
    let mut supplies = 0_u32;
    let mut loss_seen = false;
    let mut planes = BTreeSet::new();
    while run.player_view().terminal().is_none() {
        planes.extend(run.current_plane());
        match run.player_view().decision().unwrap().kind() {
            ActivityDecisionKind::Encounter => {
                battles += 1;
                if battles == 1 {
                    let snapshot = run.contribution_snapshot().unwrap();
                    let difficulty_level = snapshot.augment_enemy_difficulty_add.iter().fold(
                        snapshot.difficulty.enemy_scaling.enemy_difficulty_level,
                        |level, (_, additional)| level + u16::from(*additional),
                    );
                    let encounters = catalog.encounter_catalog();
                    let scaling = encounters
                        .enemy_scaling(snapshot.node.plane, difficulty_level)
                        .unwrap();
                    let rejected_cache = assembler.cache_stats();
                    let mismatched_scaling =
                        starclock_mode_currency_wars::CurrencyWarsEnemyScaling {
                            chapter: snapshot.node.plane.saturating_add(1),
                            ..scaling
                        };
                    assert!(
                        assembler
                            .materialize(&snapshot, encounters, mismatched_scaling)
                            .is_err()
                    );
                    assert_eq!(assembler.cache_stats(), rejected_cache);
                    let first = assembler
                        .materialize(&snapshot, encounters, scaling)
                        .unwrap();
                    let second = assembler
                        .materialize(&snapshot, encounters, scaling)
                        .unwrap();
                    assert_eq!(first.battle_spec(), second.battle_spec());
                    assert_eq!(first.contribution_receipt(), second.contribution_receipt());
                    let contribution_receipt = first.contribution_receipt();
                    assert_eq!(contribution_receipt.front_role_count, 1);
                    assert!(contribution_receipt.modifier_binding_count > 0);
                    assert!(contribution_receipt.star_property_count > 0);
                    assert!(contribution_receipt.equipment_property_count > 0);
                    assert!(contribution_receipt.bond_property_count > 0);
                    assert!(contribution_receipt.entry_energy_role_count > 0);
                    assert!(contribution_receipt.empowerment_skill_count > 0);
                    assert!(contribution_receipt.character_override_count > 0);
                    assert_eq!(first.selection().selected_enemy_affix_ids.as_ref(), [2005]);
                    assert!(first.selection().enemy_affix_policy_id().is_none());
                    assert_eq!(first.battle_spec().clock(), snapshot.battle_clock);
                    assert_eq!(
                        first.battle_spec().enemy_defeat_energy().unwrap().scaled(),
                        5_000_000
                    );
                    assert_eq!(
                        first.battle_spec().player_lethal_rescue().is_some(),
                        matches!(snapshot.battle_clock, Some(BattleClockSpec::ActionValue(_))),
                    );
                    let technique = snapshot.battle_overrides.automatic_techniques[0].ability;
                    let mut battle = Battle::create(
                        Arc::clone(first.combat_catalog()),
                        first.battle_spec().clone(),
                        BattleSeed::new([0x45; 32]),
                    )
                    .unwrap();
                    let resolution = battle
                        .apply(Command::StartBattle {
                            decision: battle.decision().unwrap().id(),
                        })
                        .unwrap();
                    assert!(resolution.events().iter().any(|event| {
                        matches!(
                            event.kind(),
                            BattleEventKind::Action(ActionEventData::Queued { ability, .. })
                                if *ability == technique
                        )
                    }));
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
                        snapshot.battle_overrides.back_battle_events.len()
                    );
                    assert_eq!(
                        battle.view().links().count(),
                        snapshot.battle_overrides.back_battle_events.len()
                    );
                    let back_event = battle
                        .view()
                        .units_by_id()
                        .find(|unit| matches!(unit.source(), ParticipantSource::Linked(_)))
                        .unwrap();
                    assert_eq!(back_event.maximum_hp().get(), 90);
                    assert_eq!(back_event.base_attack().scaled(), 100_000_000);
                    assert_eq!(back_event.base_defense().scaled(), 100_000_000);
                    assert_eq!(assembler.cache_stats().misses, 1);
                    assert_eq!(assembler.cache_stats().hits, 1);
                    let alternate_scaling = encounters
                        .enemy_scalings()
                        .find(|candidate| {
                            candidate.chapter == scaling.chapter
                                && candidate.difficulty_level > scaling.difficulty_level
                        })
                        .unwrap();
                    let alternate = assembler
                        .materialize(&snapshot, encounters, alternate_scaling)
                        .unwrap();
                    assert_ne!(first.battle_spec(), alternate.battle_spec());
                    assert_eq!(assembler.cache_stats().misses, 2);
                    assert_eq!(assembler.cache_stats().hits, 1);
                    let mut bounded =
                        CurrencyWarsBattleAssembler::new(Arc::clone(&resources), 1).unwrap();
                    bounded.materialize(&snapshot, encounters, scaling).unwrap();
                    bounded
                        .materialize(&snapshot, encounters, alternate_scaling)
                        .unwrap();
                    bounded.materialize(&snapshot, encounters, scaling).unwrap();
                    assert_eq!(bounded.cache_stats().entries, 1);
                    assert_eq!(bounded.cache_stats().hits, 0);
                    assert_eq!(bounded.cache_stats().misses, 3);
                }
                let lose_this_battle =
                    !loss_seen && run.current_battle_boundary().unwrap().clock().is_some();
                let node = run.current_node().unwrap().clone();
                let (handoff, selection) = start_battle(&mut run, &mut assembler, battles);
                let group = catalog
                    .encounter_catalog()
                    .groups()
                    .iter()
                    .find(|group| group.source_id == selection.group_source_id)
                    .unwrap();
                let selected_battle_area = selection.stage_id / 100;
                match node.kind {
                    CurrencyWarsNodeKind::Boss => {
                        assert_eq!(selection.enemy_star, 4);
                        assert!(selection.initial_phase_slots > 0);
                        assert_eq!(group.boss_battle_area_id, Some(selected_battle_area));
                        assert_eq!(
                            selection.boss_pool_source_id,
                            catalog
                                .encounter_catalog()
                                .boss_pool(selected_battle_area)
                                .map(|pool| pool.source_id)
                        );
                    }
                    CurrencyWarsNodeKind::Monster
                    | CurrencyWarsNodeKind::CampMonster
                    | CurrencyWarsNodeKind::EliteBranch => {
                        assert_eq!(selection.enemy_star, node.plane);
                        assert!(group.battle_area_ids.contains(&selected_battle_area));
                        assert_ne!(group.boss_battle_area_id, Some(selected_battle_area));
                        assert_eq!(selection.boss_pool_source_id, None);
                    }
                    CurrencyWarsNodeKind::Supply => unreachable!("battle decision at Supply node"),
                }
                assert!(selection.multi_phase_slots <= selection.initial_phase_slots);
                assert!(!selection.formation_wave_limits.is_empty());
                assert!(
                    selection
                        .formation_wave_limits
                        .iter()
                        .all(|maximum| (1..=5).contains(maximum))
                );
                assert_eq!(
                    selection
                        .formation_wave_limits
                        .iter()
                        .map(|maximum| usize::from(*maximum))
                        .sum::<usize>(),
                    selection.selected_monster_ids.len()
                );
                assert!(selection.selected_monster_ids.iter().all(|source_id| {
                    catalog.encounter_catalog().enemy_slot(*source_id).is_some()
                }));
                assert_eq!(
                    selection.formation_wave_policy_id(),
                    "currency-wars.formation-wave-selection-policy.v1"
                );
                assert_eq!(
                    selection.enemy_star_policy_id(),
                    "currency-wars.enemy-star-selection-policy.v1"
                );
                assert_eq!(
                    selection.enemy_roster_policy_id(),
                    "currency-wars.camp-enemy-roster-policy.v1"
                );
                if battles == 1 {
                    assert_eq!(handoff.participants().len(), 1);
                    assert_eq!(
                        handoff.participants()[0].participant().get(),
                        selected_roles[1].role().get()
                    );
                    assert_eq!(
                        selection.policy_id(),
                        "currency-wars.encounter-selection-policy.v1"
                    );
                    assert_eq!(
                        selection.team_resource_policy_id(),
                        "currency-wars.initial-skill-point-policy.v1"
                    );
                }
                let outcome = if lose_this_battle {
                    BattleOutcome::Lost
                } else {
                    BattleOutcome::Won
                };
                if battles == 1 {
                    let before_rejection = run.state_hash();
                    assert!(
                        run.submit_battle_result(incomplete_result(&handoff, outcome))
                            .is_err()
                    );
                    assert_eq!(run.state_hash(), before_rejection);
                }
                run.submit_battle_result(result(&handoff, outcome))
                    .unwrap_or_else(|error| {
                        panic!("production battle {battles} ({outcome:?}) failed: {error}")
                    });
                if lose_this_battle {
                    loss_seen = true;
                    assert!(run.squad_hp() > 0);
                    assert!(run.last_squad_hp_loss() > 0);
                }
            }
            ActivityDecisionKind::Shop => {
                supplies += 1;
                run.continue_supply().unwrap();
            }
            ActivityDecisionKind::Route => run.continue_plane().unwrap(),
            kind => panic!("unexpected production Currency Wars decision: {kind:?}"),
        }
    }

    assert_eq!(
        run.player_view().terminal(),
        Some(ActivityTerminalOutcome::Completed)
    );
    assert_eq!(run.player_view().completed_battle_count(), 20);
    assert_eq!(battles, 20);
    assert_eq!(supplies, 3);
    assert!(loss_seen);
    assert_eq!(planes, BTreeSet::from([1, 2, 3]));
    assert_eq!(
        catalog.flow_catalog().classify_settlement(100).rank_type(),
        Some("SSS")
    );
}

#[test]
fn production_transition_battles_reconstruct_from_fresh_state_and_seed() {
    let candidate = load_currency_wars_catalog_candidate().unwrap();
    let identity = ActivityDefinitionIdentity::new(
        ActivityDefinitionId::new(23).unwrap(),
        ActivityDefinitionDigest::new(candidate.identity().content_digest().bytes()).unwrap(),
        ActivityConfigDigest::new(candidate.identity().configuration_digest().bytes()).unwrap(),
    );
    let catalog = Arc::new(candidate.into_catalog());
    let route = catalog
        .routes()
        .iter()
        .find(|route| route.stable_key.as_ref() == "currency-wars.area.route.100")
        .unwrap();
    let difficulty = catalog
        .difficulties()
        .iter()
        .find(|difficulty| difficulty.division_level == 3)
        .unwrap();
    let role = CurrencyWarsRoleState::new(CurrencyWarsRoleId::new(1301).unwrap(), 1).unwrap();
    let roster = CurrencyWarsRoster::new(&catalog, [(role, 1)]).unwrap();
    let deployment = CurrencyWarsDeployment::new(
        &catalog,
        &roster,
        1,
        [(position(CurrencyWarsPositionKind::Front, 1), role)],
    )
    .unwrap();
    let definition = Arc::new(
        CurrencyWarsRunDefinition::new(
            identity,
            Arc::clone(&catalog),
            route.id,
            difficulty.source_id,
            CurrencyWarsGambit::Standard,
            CurrencyWarsEntryState::new(21, true, 9),
            CurrencyWarsRunSetup {
                initial_gold: 10,
                initial_team_level: 1,
                initial_experience: 0,
                roster,
                deployment,
                enemy_affix_ids: Box::new([]),
                owned_builds: BTreeMap::new(),
            },
        )
        .unwrap(),
    );
    let mut left = CurrencyWarsRun::start(
        Arc::clone(&definition),
        ActivityInstanceId::new(23).unwrap(),
        ActivityMasterSeed::from_u64(23_000_501),
    )
    .unwrap();
    let mut right = CurrencyWarsRun::start(
        definition,
        ActivityInstanceId::new(23).unwrap(),
        ActivityMasterSeed::from_u64(23_000_501),
    )
    .unwrap();
    let resources = Arc::new(load_currency_wars_battle_resources(&catalog).unwrap());
    let mut left_assembler = CurrencyWarsBattleAssembler::new(Arc::clone(&resources), 4).unwrap();
    let mut right_assembler = CurrencyWarsBattleAssembler::new(resources, 4).unwrap();

    let left_first = left
        .engage_current_node(AttemptId::new(1).unwrap(), &mut left_assembler)
        .unwrap();
    let right_first = right
        .engage_current_node(AttemptId::new(1).unwrap(), &mut right_assembler)
        .unwrap();
    assert_eq!(
        left_first.resolution().state_hash(),
        right_first.resolution().state_hash()
    );
    assert_eq!(
        left_first.resolution().events(),
        right_first.resolution().events()
    );
    assert_eq!(
        left_first.materialization().battle_spec(),
        right_first.materialization().battle_spec()
    );
    assert_eq!(
        left_first.materialization().selection(),
        right_first.materialization().selection()
    );
    assert_eq!(
        left_first.materialization().contribution_receipt(),
        right_first.materialization().contribution_receipt()
    );
    left.choose_prepared_battle().unwrap();
    right.choose_prepared_battle().unwrap();
    let left_handoff = left.start_pending_battle().unwrap();
    let right_handoff = right.start_pending_battle().unwrap();
    assert_eq!(left_handoff, right_handoff);
    let left_settlement = left
        .submit_battle_result(result(&left_handoff, BattleOutcome::Won))
        .unwrap();
    let right_settlement = right
        .submit_battle_result(result(&right_handoff, BattleOutcome::Won))
        .unwrap();
    assert_eq!(left_settlement.settlement(), right_settlement.settlement());
    assert_eq!(left_settlement.events(), right_settlement.events());
    assert_eq!(left_settlement.state_hash(), right_settlement.state_hash());
    assert_eq!(left.player_view(), right.player_view());
    assert_eq!(left.debug_view().rng(), right.debug_view().rng());

    while left.player_view().decision().unwrap().kind() != ActivityDecisionKind::Encounter {
        assert_eq!(left.player_view(), right.player_view());
        match left.player_view().decision().unwrap().kind() {
            ActivityDecisionKind::Shop => {
                left.continue_supply().unwrap();
                right.continue_supply().unwrap();
            }
            ActivityDecisionKind::Route => {
                left.continue_plane().unwrap();
                right.continue_plane().unwrap();
            }
            kind => panic!("unexpected transition decision: {kind:?}"),
        }
    }
    let left_second = left
        .engage_current_node(AttemptId::new(2).unwrap(), &mut left_assembler)
        .unwrap();
    let right_second = right
        .engage_current_node(AttemptId::new(2).unwrap(), &mut right_assembler)
        .unwrap();
    assert_eq!(
        left_second.resolution().state_hash(),
        right_second.resolution().state_hash()
    );
    assert_eq!(
        left_second.resolution().events(),
        right_second.resolution().events()
    );
    assert_eq!(
        left_second.materialization().battle_spec(),
        right_second.materialization().battle_spec()
    );
    assert_eq!(left_assembler.cache_stats(), right_assembler.cache_stats());
}

#[test]
fn production_bonds_resolve_parent_thresholds_explicit_and_module_subtraits() {
    let catalog = load_currency_wars_catalog_candidate()
        .unwrap()
        .into_catalog();
    let roles = [1301, 1306, 1014, 1015]
        .map(|raw| CurrencyWarsRoleState::new(CurrencyWarsRoleId::new(raw).unwrap(), 1).unwrap());
    let roster = CurrencyWarsRoster::new(&catalog, roles.map(|state| (state, 1))).unwrap();
    let deployment = CurrencyWarsDeployment::new(
        &catalog,
        &roster,
        4,
        [
            position(CurrencyWarsPositionKind::Front, 1),
            position(CurrencyWarsPositionKind::Front, 2),
            position(CurrencyWarsPositionKind::Back, 1),
            position(CurrencyWarsPositionKind::Back, 2),
        ]
        .into_iter()
        .zip(roles),
    )
    .unwrap();
    let parent_festival = CurrencyWarsBondId::new(1010).unwrap();
    let festival_star = CurrencyWarsBondId::new(2501).unwrap();
    let parent_grail = CurrencyWarsBondId::new(1013).unwrap();
    let default_grail = CurrencyWarsBondId::new(10131).unwrap();
    let projection_grail = CurrencyWarsBondId::new(10132).unwrap();
    let mut context = CurrencyWarsBondResolutionContext {
        module_id: catalog.flow_catalog().profile_module_source_id(),
        ..CurrencyWarsBondResolutionContext::default()
    };
    context
        .selected_subtraits
        .insert(parent_festival, festival_star);
    let snapshot = catalog.bond_catalog().resolve(
        &deployment,
        &CurrencyWarsEquipmentLoadout::default(),
        &context,
    );
    assert!(
        snapshot.active_bonds.iter().any(|bond| {
            bond.id == parent_festival && bond.member_count == 2 && bond.level == 2
        })
    );
    assert!(
        snapshot
            .active_bonds
            .iter()
            .any(|bond| { bond.id == festival_star && bond.member_count == 2 && bond.level == 2 })
    );
    assert!(
        snapshot
            .active_bonds
            .iter()
            .any(|bond| { bond.id == parent_grail && bond.member_count == 2 && bond.level == 2 })
    );
    assert!(
        snapshot
            .active_bonds
            .iter()
            .any(|bond| bond.id == default_grail)
    );
    assert!(
        !snapshot
            .active_bonds
            .iter()
            .any(|bond| bond.id == projection_grail)
    );
    for expected in [".1010.2.", ".2501.2.", ".1013.2.", ".10131.2."] {
        assert!(snapshot.contributions.iter().any(|contribution| {
            contribution.level == Some(2) && contribution.stable_key.contains(expected)
        }));
    }
    assert!(
        snapshot.properties.iter().all(|property| {
            !property.targets.is_empty() && property.property.value.scaled() != 0
        })
    );

    context.module_id = 7_110_501;
    let projection = catalog.bond_catalog().resolve(
        &deployment,
        &CurrencyWarsEquipmentLoadout::default(),
        &context,
    );
    assert!(
        projection
            .active_bonds
            .iter()
            .any(|bond| bond.id == projection_grail)
    );
    assert!(
        !projection
            .active_bonds
            .iter()
            .any(|bond| bond.id == default_grail)
    );
}

#[test]
fn production_battle_overrides_join_role_star_and_cyrene_provider_exactly() {
    let candidate = load_currency_wars_catalog_candidate().unwrap();
    let identity = ActivityDefinitionIdentity::new(
        ActivityDefinitionId::new(22).unwrap(),
        ActivityDefinitionDigest::new(candidate.identity().content_digest().bytes()).unwrap(),
        ActivityConfigDigest::new(candidate.identity().configuration_digest().bytes()).unwrap(),
    );
    let catalog = Arc::new(candidate.into_catalog());
    let route = &catalog.routes()[0];
    let difficulty = catalog
        .difficulties()
        .iter()
        .find(|difficulty| difficulty.division_level == 9)
        .unwrap();
    let roles = [1415, 1403]
        .map(|raw| CurrencyWarsRoleState::new(CurrencyWarsRoleId::new(raw).unwrap(), 1).unwrap());
    let roster = CurrencyWarsRoster::new(&catalog, roles.map(|state| (state, 1))).unwrap();
    let deployment = CurrencyWarsDeployment::new(
        &catalog,
        &roster,
        2,
        [
            position(CurrencyWarsPositionKind::Front, 1),
            position(CurrencyWarsPositionKind::Front, 2),
        ]
        .into_iter()
        .zip(roles),
    )
    .unwrap();
    let definition = Arc::new(
        CurrencyWarsRunDefinition::new(
            identity,
            Arc::clone(&catalog),
            route.id,
            difficulty.source_id,
            CurrencyWarsGambit::Standard,
            CurrencyWarsEntryState::new(21, true, 9),
            CurrencyWarsRunSetup {
                initial_gold: 0,
                initial_team_level: 2,
                initial_experience: 0,
                roster,
                deployment,
                enemy_affix_ids: Box::new([]),
                owned_builds: BTreeMap::new(),
            },
        )
        .unwrap(),
    );
    assert_eq!(definition.enemy_affixes().source_ids().len(), 4);
    assert_eq!(
        definition.enemy_affixes().source(),
        CurrencyWarsEnemyAffixSelectionSource::DeterministicProjectPolicy
    );
    assert!(definition.enemy_affixes().policy_id().is_some());
    let run = CurrencyWarsRun::start(
        definition,
        ActivityInstanceId::new(22).unwrap(),
        ActivityMasterSeed::from_u64(22_000_501),
    )
    .unwrap();
    let snapshot = run.battle_override_snapshot().unwrap();
    let contributions = run.contribution_snapshot().unwrap();

    assert_eq!(snapshot.automatic_techniques.len(), 2);
    assert!(
        snapshot
            .back_battle_events
            .iter()
            .any(|event| event.event_id == 62375)
    );
    assert!(snapshot.cyrene_skill_overrides.iter().any(|value| {
        value.provider_role == CurrencyWarsRoleId::new(1415).unwrap()
            && value.role == CurrencyWarsRoleId::new(1403).unwrap()
    }));
    assert_eq!(contributions.roles.len(), 2);
    assert_eq!(contributions.team_level.level, 2);
    assert_eq!(contributions.team_level.properties.len(), 2);
    assert_eq!(contributions.parameter_registry.len(), 254);
    assert_eq!(contributions.influence_properties.len(), 7);
    assert_eq!(contributions.bond_registry.len(), 683);
    let tribbie = contributions
        .roles
        .iter()
        .find(|role| role.role.id == CurrencyWarsRoleId::new(1403).unwrap())
        .unwrap();
    let passive = tribbie
        .empowerment
        .iter()
        .find(|skill| skill.skill_id == 14039901)
        .unwrap();
    assert_eq!(
        passive.parameters.as_ref(),
        [
            Scalar::from_scaled(120_000),
            Scalar::from_scaled(10_060_000)
        ]
    );
    assert_eq!(contributions.summon_battle_event_overrides.len(), 1);
    assert_eq!(
        contributions.summon_battle_event_overrides[0]
            .source_path
            .as_ref(),
        "Config/ConfigCharacter/GridFight/3.5/Avatar_GridFight_Lingsha_00_BE_Config.json"
    );
    assert!(contributions.roles.iter().all(|role| {
        !role.effective_ability_levels.is_empty()
            && !role.empowerment.is_empty()
            && role.star_state.rank_attachments.len() == 6
            && role.role.id == role.role_state.role()
            && role.character_override.is_some()
    }));
}

fn position(kind: CurrencyWarsPositionKind, index: u8) -> CurrencyWarsPosition {
    CurrencyWarsPosition::new(kind, index).unwrap()
}

fn start_battle(
    run: &mut CurrencyWarsRun,
    assembler: &mut CurrencyWarsBattleAssembler,
    attempt: u32,
) -> (ActivityBattleHandoff, CurrencyWarsEncounterSelectionReceipt) {
    let preparation = run
        .engage_current_node(AttemptId::new(attempt).unwrap(), assembler)
        .unwrap();
    let selection = preparation.materialization().selection();
    run.choose_prepared_battle().unwrap();
    (run.start_pending_battle().unwrap(), selection)
}

fn result(handoff: &ActivityBattleHandoff, outcome: BattleOutcome) -> BattleResult {
    let progress = if outcome == BattleOutcome::Lost {
        900_000
    } else {
        1_000_000
    };
    let mut values = vec![
        ProjectedValue::Outcome(outcome),
        ProjectedValue::FinalStateHash(BattleStateHash::from_bytes([0x71; 32])),
        ProjectedValue::EventDigest(EventDigest::new([0x72; 32]).unwrap()),
        ProjectedValue::TerminalFault(None),
    ];
    values.extend(handoff.participant_carry().iter().map(|carry| {
        ProjectedValue::ParticipantState(
            ParticipantBattleState::new(
                carry.participant(),
                carry.current_hp(),
                carry.maximum_hp(),
                carry.current_energy(),
                carry.maximum_energy(),
                LifeState::Alive,
                PresenceState::Present,
            )
            .unwrap(),
        )
    }));
    values.extend([
        ProjectedValue::Metric {
            key: CURRENCY_WARS_ACTION_VALUE_REMAINING_KEY.into(),
            value: MetricValue::ActionValue(0),
        },
        ProjectedValue::Metric {
            key: CURRENCY_WARS_BATTLE_PROGRESS_KEY.into(),
            value: MetricValue::Ratio(progress),
        },
    ]);
    BattleResult::seal(handoff.identity(), values)
}

fn incomplete_result(handoff: &ActivityBattleHandoff, outcome: BattleOutcome) -> BattleResult {
    BattleResult::seal(
        handoff.identity(),
        vec![
            ProjectedValue::Outcome(outcome),
            ProjectedValue::FinalStateHash(BattleStateHash::from_bytes([0x71; 32])),
            ProjectedValue::EventDigest(EventDigest::new([0x72; 32]).unwrap()),
            ProjectedValue::TerminalFault(None),
        ],
    )
}
