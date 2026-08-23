//! Line-limit exception: this test-only state-machine corpus shares one production definition fixture.
use std::{collections::BTreeMap, sync::Arc};

use starclock_activity::{
    ActivityBattleHandoff, ActivityConfigDigest, ActivityDecisionKind, ActivityDefinitionDigest,
    ActivityDefinitionId, ActivityDefinitionIdentity, ActivityInstanceId, ActivityMasterSeed,
    ActivityTerminalOutcome, AttemptId, BattleOutcome, BattleResult, EventDigest, MetricValue,
    ParticipantBattleState, ProjectedValue,
};
use starclock_combat::{
    AbilityId, ActionValue, AssemblyDigest, BattleClockSpec, BattleSpec, BattleStateHash,
    CombatantSpecDigest, ConcedePolicy, EncounterId, EnemyDefinitionId, Energy, FormationIndex, Hp,
    LifeState, ParticipantSource, ParticipantSpec, PresenceState, ResolvedCombatantSpec,
    ResolvedDefinitionBindings, Speed, TeamResourceSpec, TeamSide, UnitDefinitionId, UnitLevel,
};

use crate::{
    CURRENCY_WARS_ACTION_VALUE_REMAINING_KEY, CURRENCY_WARS_BATTLE_PROGRESS_KEY,
    CurrencyWarsAppliedReward, CurrencyWarsAugmentQuality, CurrencyWarsBondId, CurrencyWarsCatalog,
    CurrencyWarsDeployment, CurrencyWarsEntryState, CurrencyWarsEquipmentId,
    CurrencyWarsEquipmentSlot, CurrencyWarsGambit, CurrencyWarsInvestmentId,
    CurrencyWarsInvestmentOfferFamily, CurrencyWarsInvestmentOfferSpec, CurrencyWarsItemId,
    CurrencyWarsPosition, CurrencyWarsPositionKind, CurrencyWarsRoleId, CurrencyWarsRoleState,
    CurrencyWarsRoster, CurrencyWarsRun, CurrencyWarsRunDefinition, CurrencyWarsRunPosition,
    CurrencyWarsRunSetup, CurrencyWarsSelectedEnhancementId, catalog::tests_support,
};

#[test]
fn run_starts_at_the_first_route_encounter() {
    let run = CurrencyWarsRun::start(
        definition(10),
        ActivityInstanceId::new(1).unwrap(),
        ActivityMasterSeed::from_u64(7),
    )
    .unwrap();

    assert_eq!(run.gold(), 10);
    assert_eq!(run.team_level(), 1);
    assert_eq!(run.back_capacity(), 6);
    assert_eq!(run.squad_hp(), 100);
    assert_eq!(
        run.player_view().decision().unwrap().kind(),
        ActivityDecisionKind::Encounter
    );
}

#[test]
fn refresh_and_purchase_are_atomic_activity_boundaries() {
    let mut run = CurrencyWarsRun::start(
        definition(10),
        ActivityInstanceId::new(1).unwrap(),
        ActivityMasterSeed::from_u64(7),
    )
    .unwrap();
    let offered = run.refresh_shop().unwrap();

    assert_eq!(offered.len(), 5);
    assert!(
        offered
            .iter()
            .all(|offer| offer.role() == offered[0].role())
    );
    assert_eq!(offered[0].role(), CurrencyWarsRoleId::new(1001).unwrap());
    assert_eq!(run.gold(), 8);
    run.buy_shop_offer(offered[0]).unwrap();
    assert_eq!(run.gold(), 7);
    assert_eq!(
        run.roster()
            .unwrap()
            .count(CurrencyWarsRoleState::new(offered[0].role(), 1).unwrap()),
        2,
    );
}

#[test]
fn shop_excludes_roles_until_the_authored_cost_threshold_is_reached() {
    let catalog = Arc::new(tests_support::catalog_with_role_cost_threshold(
        CurrencyWarsRunPosition::new(2, 1).unwrap(),
    ));
    let mut run = CurrencyWarsRun::start(
        definition_with_catalog(10, catalog),
        ActivityInstanceId::new(81).unwrap(),
        ActivityMasterSeed::from_u64(181),
    )
    .unwrap();
    let hash = run.state_hash();
    let rng = run.debug_view().rng().to_vec();

    assert!(run.refresh_shop().is_err());
    assert_eq!(run.state_hash(), hash);
    assert_eq!(run.debug_view().rng(), rng);

    win_current_battle(&mut run, 70_000_001, 1);
    run.continue_supply().unwrap();
    run.continue_plane().unwrap();
    assert_eq!(run.current_plane(), Some(2));
    assert_eq!(run.refresh_shop().unwrap().len(), 5);
}

#[test]
fn reward_pools_apply_typed_rewards_and_empty_fallback_preserves_state() {
    let mut run = CurrencyWarsRun::start(
        definition(10),
        ActivityInstanceId::new(60).unwrap(),
        ActivityMasterSeed::from_u64(160),
    )
    .unwrap();

    let resolution = run.resolve_reward_pool(1).unwrap();
    assert_eq!(resolution.selected_reward_ids.as_ref(), &[1]);
    assert_eq!(resolution.remaining_value, 0);
    assert_eq!(run.gold(), 11);

    let hash = run.state_hash();
    let empty = run.resolve_reward_pool(2).unwrap();
    assert!(empty.selected_reward_ids.is_empty());
    assert_eq!(empty.remaining_value, 1);
    assert_eq!(run.state_hash(), hash);

    run.resolve_reward_pool(3).unwrap();
    assert_eq!(run.free_refreshes(), 1);
    let gold = run.gold();
    run.refresh_shop().unwrap();
    assert_eq!(run.free_refreshes(), 0);
    assert_eq!(run.gold(), gold);
}

#[test]
fn item_inventory_and_equipment_crafting_commit_atomically() {
    let mut run = CurrencyWarsRun::start(
        definition(10),
        ActivityInstanceId::new(61).unwrap(),
        ActivityMasterSeed::from_u64(161),
    )
    .unwrap();
    let item = CurrencyWarsItemId::new(350_101).unwrap();
    run.receive_item(item, 2).unwrap();
    assert_eq!(run.item_inventory().unwrap().get(&item), Some(&2));

    let equipment = CurrencyWarsEquipmentId::new(1).unwrap();
    run.receive_equipment(equipment).unwrap();
    run.receive_equipment(equipment).unwrap();
    run.craft_equipment(1).unwrap();
    run.equip(CurrencyWarsRoleId::new(1001).unwrap(), equipment, None)
        .unwrap();
    let hash = run.state_hash();
    assert!(run.craft_equipment(1).is_err());
    assert_eq!(run.state_hash(), hash);
}

#[test]
fn workbench_and_forge_services_execute_their_typed_inventory_lifecycle() {
    let role = CurrencyWarsRoleId::new(1001).unwrap();
    let source = CurrencyWarsEquipmentId::new(1).unwrap();

    let mut reroll = CurrencyWarsRun::start(
        definition(10),
        ActivityInstanceId::new(62).unwrap(),
        ActivityMasterSeed::from_u64(162),
    )
    .unwrap();
    let reroll_item = CurrencyWarsItemId::new(350_103).unwrap();
    reroll.receive_item(reroll_item, 1).unwrap();
    reroll.receive_equipment(source).unwrap();
    let output = reroll.use_reroll_equipment(reroll_item, source).unwrap();
    assert_eq!(output, CurrencyWarsEquipmentId::new(2).unwrap());
    reroll.equip(role, output, None).unwrap();

    let mut recommended = CurrencyWarsRun::start(
        definition(10),
        ActivityInstanceId::new(63).unwrap(),
        ActivityMasterSeed::from_u64(163),
    )
    .unwrap();
    let recommended_item = CurrencyWarsItemId::new(350_107).unwrap();
    recommended.receive_item(recommended_item, 1).unwrap();
    let granted = recommended
        .use_recommended_equipment(recommended_item, role)
        .unwrap();
    assert_eq!(granted.len(), 1);
    recommended.equip(role, granted[0], None).unwrap();

    let mut forge = CurrencyWarsRun::start(
        definition(10),
        ActivityInstanceId::new(64).unwrap(),
        ActivityMasterSeed::from_u64(164),
    )
    .unwrap();
    let forge_item = CurrencyWarsItemId::new(99_999).unwrap();
    forge.receive_item(forge_item, 1).unwrap();
    let offers = forge.open_forge(forge_item).unwrap();
    assert_eq!(offers.len(), 1);
    forge.choose_forge_offer(offers[0]).unwrap();
    assert!(forge.current_forge_offers().unwrap().is_empty());
    assert!(!forge.item_inventory().unwrap().contains_key(&forge_item));
}

#[test]
fn special_good_offer_purchase_and_activation_are_one_atomic_node_lifecycle() {
    let mut run = CurrencyWarsRun::start(
        definition(10),
        ActivityInstanceId::new(70).unwrap(),
        ActivityMasterSeed::from_u64(170),
    )
    .unwrap();

    run.offer_special_good(101).unwrap();
    assert_eq!(run.current_special_good_offer().unwrap(), Some(101));
    let activation = run.purchase_special_good(101).unwrap();
    assert_eq!(activation.id, 101);
    assert_eq!(activation.activation_count, 1);
    assert_eq!(activation.price_paid, 1);
    assert_eq!(run.gold(), 9);
    assert_eq!(run.special_good_activations().unwrap().get(&101), Some(&1));
    assert!(run.offer_special_good(107).is_err());

    let snapshot = run.contribution_snapshot().unwrap();
    assert_eq!(snapshot.special_goods.len(), 1);
    assert_eq!(snapshot.special_goods[0].definition.id, 101);
    assert_eq!(snapshot.special_goods[0].activation_count, 1);

    let mut free = CurrencyWarsRun::start(
        definition(10),
        ActivityInstanceId::new(71).unwrap(),
        ActivityMasterSeed::from_u64(171),
    )
    .unwrap();
    free.offer_special_good(107).unwrap();
    let activation = free.purchase_special_good(107).unwrap();
    assert_eq!(activation.price_paid, 0);
    assert_eq!(free.gold(), 10);

    let mut three_star = CurrencyWarsRun::start(
        definition(10),
        ActivityInstanceId::new(72).unwrap(),
        ActivityMasterSeed::from_u64(172),
    )
    .unwrap();
    assert!(three_star.offer_special_good(201).is_err());
    assert_eq!(
        three_star
            .activate_cyrene_three_star_goods()
            .unwrap()
            .as_ref(),
        &[201]
    );
    let hash = three_star.state_hash();
    assert!(three_star.activate_cyrene_three_star_goods().is_err());
    assert_eq!(three_star.state_hash(), hash);
}

#[test]
fn direct_reward_executor_covers_every_authored_operation_family() {
    let role = CurrencyWarsRoleId::new(1001).unwrap();
    let mut run = CurrencyWarsRun::start(
        definition(10),
        ActivityInstanceId::new(65).unwrap(),
        ActivityMasterSeed::from_u64(165),
    )
    .unwrap();
    assert_eq!(
        run.apply_reward(4).unwrap(),
        CurrencyWarsAppliedReward::Experience(5)
    );
    assert!(run.team_level() > 1);
    assert_eq!(
        run.apply_reward(5).unwrap(),
        CurrencyWarsAppliedReward::Item {
            item: CurrencyWarsItemId::new(350_101).unwrap(),
            count: 1,
        }
    );
    assert!(matches!(
        run.apply_reward(6).unwrap(),
        CurrencyWarsAppliedReward::Investment(_)
    ));
    assert_eq!(
        run.apply_reward(7).unwrap(),
        CurrencyWarsAppliedReward::Role { role, star: 1 }
    );
    assert_eq!(
        run.apply_reward(8).unwrap(),
        CurrencyWarsAppliedReward::Role { role, star: 1 }
    );

    let hash = run.state_hash();
    assert_eq!(
        run.apply_reward(13).unwrap(),
        CurrencyWarsAppliedReward::NoLegalResult
    );
    assert_eq!(run.state_hash(), hash);

    for (instance, reward_id) in [(66, 9), (67, 10), (68, 11), (69, 12)] {
        let mut equipment_run = CurrencyWarsRun::start(
            definition(10),
            ActivityInstanceId::new(instance).unwrap(),
            ActivityMasterSeed::from_u64(instance),
        )
        .unwrap();
        let applied = equipment_run.apply_reward(reward_id).unwrap();
        let equipment = match applied {
            CurrencyWarsAppliedReward::Equipment(values)
            | CurrencyWarsAppliedReward::RoleWithEquipment {
                equipment: values, ..
            } => values[0],
            other => panic!("unexpected fixture reward: {other:?}"),
        };
        equipment_run.equip(role, equipment, None).unwrap();
    }
}

#[test]
fn augment_offer_and_explicit_replacement_are_atomic() {
    let mut run = CurrencyWarsRun::start(
        definition(10),
        ActivityInstanceId::new(40).unwrap(),
        ActivityMasterSeed::from_u64(100),
    )
    .unwrap();
    let offered = run
        .offer_augments(CurrencyWarsAugmentQuality::Silver)
        .unwrap();
    assert_eq!(offered.len(), 3);
    assert_eq!(run.current_augment_offers().unwrap(), offered);

    let invalid = CurrencyWarsInvestmentId::new(5).unwrap();
    let hash = run.state_hash();
    assert!(run.choose_augment(invalid, None).is_err());
    assert_eq!(run.state_hash(), hash);

    run.choose_augment(offered[0], None).unwrap();
    assert!(run.current_augment_offers().unwrap().is_empty());
    let replacement_offer = run
        .offer_augments(CurrencyWarsAugmentQuality::Silver)
        .unwrap();
    run.choose_augment(replacement_offer[0], Some(offered[0]))
        .unwrap();
    let contribution = run.contribution_snapshot().unwrap();
    assert_eq!(contribution.investments.len(), 1);
    assert_eq!(contribution.investments[0].id, replacement_offer[0]);
}

#[test]
fn selected_enhancement_honors_condition_cost_and_rejection_atomicity() {
    let mut run = CurrencyWarsRun::start(
        definition(10),
        ActivityInstanceId::new(41).unwrap(),
        ActivityMasterSeed::from_u64(101),
    )
    .unwrap();
    let id = CurrencyWarsSelectedEnhancementId::new(1).unwrap();
    assert_eq!(run.eligible_selected_enhancements(30_021).unwrap().len(), 1);
    assert_eq!(
        run.offer_selected_enhancements(30_021).unwrap().as_ref(),
        &[id]
    );
    run.choose_selected_enhancement(id, 30_021, None).unwrap();
    assert_eq!(run.gold(), 5);

    let hash = run.state_hash();
    assert!(run.choose_selected_enhancement(id, 30_021, None).is_err());
    assert_eq!(run.state_hash(), hash);
}

#[test]
fn typed_investments_enforce_explicit_eligibility_and_talent_graphs() {
    let mut run = CurrencyWarsRun::start(
        definition(10),
        ActivityInstanceId::new(42).unwrap(),
        ActivityMasterSeed::from_u64(102),
    )
    .unwrap();
    let portal = CurrencyWarsInvestmentId::new(4_000_001).unwrap();
    let orb = CurrencyWarsInvestmentId::new(3_000_001).unwrap();
    let projection = CurrencyWarsInvestmentId::new(5_000_001).unwrap();
    let talent_root = CurrencyWarsInvestmentId::new(6_000_001).unwrap();
    let talent_child = CurrencyWarsInvestmentId::new(6_000_002).unwrap();

    run.choose_investment(portal).unwrap();
    run.choose_investment(orb).unwrap();
    run.choose_investment(projection).unwrap();
    let hash = run.state_hash();
    assert!(run.choose_talent(talent_child, true).is_err());
    assert_eq!(run.state_hash(), hash);
    assert!(run.choose_talent(talent_root, false).is_err());
    assert_eq!(run.state_hash(), hash);
    run.choose_talent(talent_root, true).unwrap();
    run.choose_talent(talent_child, true).unwrap();

    assert!(run.choose_season_talent(2021, true).is_err());
    run.choose_season_talent(2011, true).unwrap();
    run.choose_season_talent(2021, true).unwrap();
    let contribution = run.contribution_snapshot().unwrap();
    assert_eq!(contribution.typed_investments.len(), 5);
    assert_eq!(contribution.season_talents.len(), 2);
}

#[test]
fn cross_family_offer_reroll_replacement_and_contribution_are_atomic() {
    let mut run = CurrencyWarsRun::start(
        definition(10),
        ActivityInstanceId::new(43).unwrap(),
        ActivityMasterSeed::from_u64(103),
    )
    .unwrap();
    let spec = CurrencyWarsInvestmentOfferSpec::new(
        vec![
            CurrencyWarsInvestmentOfferFamily::Augment,
            CurrencyWarsInvestmentOfferFamily::Enhancement,
            CurrencyWarsInvestmentOfferFamily::Orb,
        ],
        None,
        3,
        1,
    )
    .unwrap();
    let first = run.offer_investments(spec).unwrap();
    assert_eq!(first.len(), 3);
    let rerolled = run.reroll_investments().unwrap();
    assert_eq!(rerolled.len(), 3);
    let hash = run.state_hash();
    assert!(run.reroll_investments().is_err());
    assert_eq!(run.state_hash(), hash);

    let selected = rerolled[0];
    let wrong_family = rerolled
        .iter()
        .copied()
        .find(|candidate| candidate.get() / 1_000_000 != selected.get() / 1_000_000);
    if let Some(wrong) = wrong_family {
        let hash = run.state_hash();
        assert!(
            run.choose_offered_investment(selected, Some(wrong), true)
                .is_err()
        );
        assert_eq!(run.state_hash(), hash);
    }
    run.choose_offered_investment(selected, None, true).unwrap();
    assert!(run.current_investment_offers().unwrap().is_empty());
    let contribution = run.contribution_snapshot().unwrap();
    assert_eq!(contribution.investments.len(), 1);
    assert_eq!(contribution.investments[0].id, selected);
}

#[test]
fn enhancement_selection_charges_gold_and_enters_the_snapshot() {
    let mut run = CurrencyWarsRun::start(
        definition(10),
        ActivityInstanceId::new(44).unwrap(),
        ActivityMasterSeed::from_u64(104),
    )
    .unwrap();
    let enhancement = CurrencyWarsInvestmentId::new(2_000_001).unwrap();
    run.choose_investment(enhancement).unwrap();
    assert_eq!(run.gold(), 5);
    let snapshot = run.contribution_snapshot().unwrap();
    assert_eq!(snapshot.enhancements.len(), 1);
    assert_eq!(snapshot.enhancements[0].investment, enhancement);
}

#[test]
fn equipment_replacement_and_teardown_are_atomic_activity_boundaries() {
    let mut run = CurrencyWarsRun::start(
        definition(10),
        ActivityInstanceId::new(30).unwrap(),
        ActivityMasterSeed::from_u64(90),
    )
    .unwrap();
    let role = CurrencyWarsRoleId::new(1001).unwrap();
    let equipment = CurrencyWarsEquipmentId::new(1).unwrap();
    run.receive_equipment(equipment).unwrap();
    run.equip(role, equipment, None).unwrap();
    assert_eq!(run.equipment_loadout().unwrap().for_role(role).count(), 1);

    let hash = run.state_hash();
    assert!(run.equip(role, equipment, None).is_err());
    assert_eq!(run.state_hash(), hash);

    run.receive_equipment(equipment).unwrap();
    let first = CurrencyWarsEquipmentSlot::new(role, 1).unwrap();
    run.equip(role, equipment, Some(first)).unwrap();
    assert_eq!(run.equipment_loadout().unwrap().for_role(role).count(), 1);
    run.unequip(first).unwrap();
    assert_eq!(run.equipment_loadout().unwrap().for_role(role).count(), 0);
}

#[test]
fn relocation_refreshes_and_tears_down_character_empowerment() {
    let mut run = CurrencyWarsRun::start(
        definition(10),
        ActivityInstanceId::new(31).unwrap(),
        ActivityMasterSeed::from_u64(91),
    )
    .unwrap();
    let front = CurrencyWarsPosition::new(CurrencyWarsPositionKind::Front, 1).unwrap();
    let back = CurrencyWarsPosition::new(CurrencyWarsPositionKind::Back, 1).unwrap();
    let initial = run.empowerment_snapshot().unwrap();
    assert_eq!(initial.active().len(), 1);
    assert_eq!(initial.active()[0].position, front);
    assert_eq!(initial.active()[0].skills.len(), 1);

    run.relocate(front, back).unwrap();
    assert!(run.empowerment_snapshot().unwrap().active().is_empty());

    run.relocate(back, front).unwrap();
    assert_eq!(run.empowerment_snapshot().unwrap(), initial);

    run.undeploy(front).unwrap();
    assert!(run.empowerment_snapshot().unwrap().active().is_empty());
}

#[test]
fn rejected_relocation_preserves_state_and_empowerment() {
    let mut run = CurrencyWarsRun::start(
        definition(10),
        ActivityInstanceId::new(32).unwrap(),
        ActivityMasterSeed::from_u64(92),
    )
    .unwrap();
    let front = CurrencyWarsPosition::new(CurrencyWarsPositionKind::Front, 1).unwrap();
    let occupied = CurrencyWarsPosition::new(CurrencyWarsPositionKind::Front, 1).unwrap();
    let hash = run.state_hash();
    let snapshot = run.empowerment_snapshot().unwrap();

    assert!(run.relocate(front, occupied).is_err());
    assert_eq!(run.state_hash(), hash);
    assert_eq!(run.empowerment_snapshot().unwrap(), snapshot);
}

#[test]
fn battle_override_snapshot_executes_automatic_energy_rescue_and_typed_contributions() {
    let mut run = CurrencyWarsRun::start(
        definition(10),
        ActivityInstanceId::new(34).unwrap(),
        ActivityMasterSeed::from_u64(94),
    )
    .unwrap();
    let snapshot = run.battle_override_snapshot().unwrap();

    assert_eq!(snapshot.automatic_techniques.len(), 1);
    assert_eq!(
        snapshot.automatic_techniques[0].ability,
        AbilityId::new(1).unwrap()
    );
    assert_eq!(snapshot.back_battle_events[0].event_id, 12);
    assert!(snapshot.external_battle_event_ids.is_empty());
    assert_eq!(snapshot.special_resources.len(), 1);
    assert_eq!(snapshot.role_global_modifiers.len(), 1);
    assert_eq!(snapshot.rank_skill_overrides.len(), 1);
    assert_eq!(snapshot.summon_battle_event_overrides.len(), 1);
    assert_eq!(snapshot.cyrene_skill_overrides.len(), 1);
    assert_eq!(
        snapshot
            .scale_defeat_energy(Energy::from_scaled(10_000_000).unwrap())
            .unwrap(),
        Energy::from_scaled(5_000_000).unwrap(),
    );
    let rescued = snapshot
        .resolve_lethal_damage(
            Hp::new(1_000).unwrap(),
            ActionValue::from_scaled(20_000_000).unwrap(),
        )
        .unwrap();
    assert_eq!(rescued.restored_hp, Hp::new(1_000).unwrap());
    assert_eq!(
        rescued.deducted_action_value,
        ActionValue::from_scaled(20_000_000).unwrap()
    );
    assert_eq!(rescued.remaining_action_value, ActionValue::ZERO);
    assert!(rescued.countdown_expired);

    let front = CurrencyWarsPosition::new(CurrencyWarsPositionKind::Front, 1).unwrap();
    let back = CurrencyWarsPosition::new(CurrencyWarsPositionKind::Back, 1).unwrap();
    run.relocate(front, back).unwrap();
    let refreshed = run.battle_override_snapshot().unwrap();
    assert!(refreshed.automatic_techniques.is_empty());
    assert!(refreshed.special_resources.is_empty());
}

#[test]
fn contribution_snapshot_is_immutable_and_binds_current_activity_state() {
    let mut run = CurrencyWarsRun::start(
        definition(10),
        ActivityInstanceId::new(35).unwrap(),
        ActivityMasterSeed::from_u64(95),
    )
    .unwrap();
    let first = run.contribution_snapshot().unwrap();

    assert_eq!(first.roles.len(), 1);
    assert_eq!(first.team_level.level, 1);
    assert_eq!(first.roles[0].role.id, first.roles[0].role_state.role());
    assert!(!first.parameter_registry.is_empty());
    assert_eq!(first.roles[0].role_state.star(), 1);
    assert!(!first.roles[0].empowerment.is_empty());
    assert_eq!(first.roles[0].empowerment[0].level, 1);
    assert_eq!(first.battle_overrides.automatic_techniques.len(), 1);

    let front = CurrencyWarsPosition::new(CurrencyWarsPositionKind::Front, 1).unwrap();
    let back = CurrencyWarsPosition::new(CurrencyWarsPositionKind::Back, 1).unwrap();
    run.relocate(front, back).unwrap();
    let second = run.contribution_snapshot().unwrap();

    assert_ne!(first.digest, second.digest);
    assert_eq!(first.roles[0].position, front);
    assert_eq!(second.roles[0].position, back);
    assert_eq!(first.battle_overrides.automatic_techniques.len(), 1);
    assert!(second.battle_overrides.automatic_techniques.is_empty());
}

#[test]
fn explicit_subtrait_selection_commits_and_invalid_selection_preserves_state() {
    let mut run = CurrencyWarsRun::start(
        definition(10),
        ActivityInstanceId::new(33).unwrap(),
        ActivityMasterSeed::from_u64(93),
    )
    .unwrap();
    let parent = CurrencyWarsBondId::new(1).unwrap();
    let child = CurrencyWarsBondId::new(2).unwrap();
    assert_eq!(run.bond_snapshot().unwrap().active_bonds.len(), 1);

    run.select_bond_subtrait(parent, child).unwrap();
    let selected = run.bond_snapshot().unwrap();
    assert_eq!(selected.active_bonds.len(), 2);
    assert_eq!(selected.selected_subtraits.as_ref(), &[(parent, child)]);
    assert_eq!(selected.contributions.len(), 2);
    assert_eq!(selected.trait_effect_ids.as_ref(), &[11, 21, 30_021]);
    assert_eq!(selected.battle_event_ids.as_ref(), &[12, 22]);

    let hash = run.state_hash();
    let unknown = CurrencyWarsBondId::new(99).unwrap();
    assert!(run.select_bond_subtrait(parent, unknown).is_err());
    assert_eq!(run.state_hash(), hash);

    let front = CurrencyWarsPosition::new(CurrencyWarsPositionKind::Front, 1).unwrap();
    run.undeploy(front).unwrap();
    let torn_down = run.bond_snapshot().unwrap();
    assert!(torn_down.active_bonds.is_empty());
    assert!(torn_down.selected_subtraits.is_empty());
    assert!(torn_down.contributions.is_empty());
}

#[test]
fn maximum_star_purchase_removes_same_role_offers_and_tears_down_old_state() {
    let mut run = CurrencyWarsRun::start(
        definition(100),
        ActivityInstanceId::new(24).unwrap(),
        ActivityMasterSeed::from_u64(84),
    )
    .unwrap();
    let role = CurrencyWarsRoleId::new(1001).unwrap();
    for purchase in 0..8 {
        let offers = run.current_shop_offers().unwrap();
        if offers.is_empty() {
            run.refresh_shop().unwrap();
        }
        let offer = run.current_shop_offers().unwrap()[0];
        assert_eq!(offer.role(), role);
        run.buy_shop_offer(offer).unwrap();
        if purchase == 4 {
            run.refresh_shop().unwrap();
        }
    }

    let maximum = CurrencyWarsRoleState::new(role, 3).unwrap();
    assert_eq!(run.roster().unwrap().count(maximum), 1);
    let front = CurrencyWarsPosition::new(CurrencyWarsPositionKind::Front, 1).unwrap();
    assert_eq!(
        run.deployment().unwrap().positions().get(&front),
        Some(&maximum)
    );
    assert!(run.current_shop_offers().unwrap().is_empty());

    let hash = run.state_hash();
    let gold = run.gold();
    let rng = run.debug_view().rng().to_vec();
    assert!(run.refresh_shop().is_err());
    assert_eq!(run.state_hash(), hash);
    assert_eq!(run.gold(), gold);
    assert_eq!(run.debug_view().rng(), rng);

    run.sell_role(maximum).unwrap();
    assert_eq!(run.current_shop_offers().unwrap().len(), 0);
    assert_eq!(run.refresh_shop().unwrap().len(), 5);
}

#[test]
fn synthesis_preserves_the_earliest_deployed_copy_as_the_upgraded_state() {
    let mut run = CurrencyWarsRun::start(
        definition(100),
        ActivityInstanceId::new(25).unwrap(),
        ActivityMasterSeed::from_u64(85),
    )
    .unwrap();
    let role = CurrencyWarsRoleId::new(1001).unwrap();
    let base = CurrencyWarsRoleState::new(role, 1).unwrap();
    let upgraded = CurrencyWarsRoleState::new(role, 2).unwrap();
    let front_one = CurrencyWarsPosition::new(CurrencyWarsPositionKind::Front, 1).unwrap();
    let front_two = CurrencyWarsPosition::new(CurrencyWarsPositionKind::Front, 2).unwrap();

    let first = run.current_shop_offers().unwrap()[0];
    run.buy_shop_offer(first).unwrap();
    run.buy_experience().unwrap();
    assert_eq!(run.team_level(), 3);
    run.deploy(front_two, base).unwrap();
    let third = run.current_shop_offers().unwrap()[0];
    run.buy_shop_offer(third).unwrap();

    let deployment = run.deployment().unwrap();
    assert_eq!(deployment.positions().get(&front_one), Some(&upgraded));
    assert!(!deployment.positions().contains_key(&front_two));
    assert_eq!(run.roster().unwrap().total_units(), 1);
}

#[test]
fn battle_entry_rejects_a_back_only_deployment_without_mutating_the_run() {
    let mut run = CurrencyWarsRun::start(
        definition(10),
        ActivityInstanceId::new(26).unwrap(),
        ActivityMasterSeed::from_u64(86),
    )
    .unwrap();
    let role = CurrencyWarsRoleId::new(1001).unwrap();
    let state = CurrencyWarsRoleState::new(role, 1).unwrap();
    let front = CurrencyWarsPosition::new(CurrencyWarsPositionKind::Front, 1).unwrap();
    let back = CurrencyWarsPosition::new(CurrencyWarsPositionKind::Back, 1).unwrap();
    run.undeploy(front).unwrap();
    run.deploy(back, state).unwrap();

    let hash = run.state_hash();
    let rng = run.debug_view().rng().to_vec();
    assert!(
        run.engage_current_node_fixture(AttemptId::new(1).unwrap(), battle(70_000_001, 1))
            .is_err()
    );
    assert_eq!(run.state_hash(), hash);
    assert_eq!(run.debug_view().rng(), rng);
}

#[test]
fn finite_shop_pool_allows_duplicates_and_empty_refresh_refunds_state_and_rng() {
    let mut run = CurrencyWarsRun::start(
        definition(100),
        ActivityInstanceId::new(20).unwrap(),
        ActivityMasterSeed::from_u64(80),
    )
    .unwrap();

    let offered = run.current_shop_offers().unwrap();
    assert_eq!(offered.len(), 5);
    assert_eq!(
        offered.iter().map(|offer| offer.slot()).collect::<Vec<_>>(),
        vec![1, 2, 3, 4, 5],
    );
    assert!(
        offered
            .iter()
            .all(|offer| offer.role() == offered[0].role())
    );
    for _ in 0..8 {
        if run.current_shop_offers().unwrap().is_empty() {
            run.refresh_shop().unwrap();
        }
        let offer = run.current_shop_offers().unwrap()[0];
        run.buy_shop_offer(offer).unwrap();
    }

    let state = run.state_hash();
    let gold = run.gold();
    let rng = run.debug_view().rng().to_vec();
    assert!(run.refresh_shop().is_err());
    assert_eq!(run.state_hash(), state);
    assert_eq!(run.gold(), gold);
    assert_eq!(run.debug_view().rng(), rng);
}

#[test]
fn locked_shop_carries_the_exact_remaining_cards_across_node_entry() {
    let mut run = CurrencyWarsRun::start(
        definition(20),
        ActivityInstanceId::new(21).unwrap(),
        ActivityMasterSeed::from_u64(81),
    )
    .unwrap();
    let first = run.current_shop_offers().unwrap()[0];
    run.buy_shop_offer(first).unwrap();
    let locked = run.current_shop_offers().unwrap();
    run.set_shop_locked(true).unwrap();

    win_current_battle(&mut run, 70_000_001, 1);

    assert!(run.is_shop_locked());
    assert_eq!(run.current_shop_offers().unwrap(), locked);
    run.set_shop_locked(false).unwrap();
    assert!(!run.is_shop_locked());
}

#[test]
fn battle_income_interest_and_direct_experience_use_authored_boundaries() {
    let mut run = CurrencyWarsRun::start(
        definition(50),
        ActivityInstanceId::new(22).unwrap(),
        ActivityMasterSeed::from_u64(82),
    )
    .unwrap();
    win_current_battle(&mut run, 70_000_001, 1);
    assert_eq!(run.gold(), 58);
    assert_eq!(run.team_level(), 2);
    assert_eq!(run.experience(), 0);

    let mut maximum = CurrencyWarsRun::start(
        definition(100),
        ActivityInstanceId::new(23).unwrap(),
        ActivityMasterSeed::from_u64(83),
    )
    .unwrap();
    for _ in 0..5 {
        maximum.buy_experience().unwrap();
    }
    assert_eq!(maximum.team_level(), 10);
    let state = maximum.state_hash();
    let rng = maximum.debug_view().rng().to_vec();
    assert!(maximum.buy_experience().is_err());
    assert_eq!(maximum.state_hash(), state);
    assert_eq!(maximum.debug_view().rng(), rng);
}

#[test]
fn three_plane_flow_carries_run_state_and_resets_node_offers() {
    let definition = definition(10);
    let mut run = CurrencyWarsRun::start(
        Arc::clone(&definition),
        ActivityInstanceId::new(1).unwrap(),
        ActivityMasterSeed::from_u64(7),
    )
    .unwrap();
    assert_eq!(
        run.progression_projection().unwrap().unwrap().rule.position,
        CurrencyWarsRunPosition::new(1, 1).unwrap(),
    );
    let rejected_hash = run.state_hash();
    assert!(run.continue_plane().is_err());
    assert_eq!(run.state_hash(), rejected_hash);

    for (plane, encounter, attempt) in [(1, 70_000_001, 1), (2, 70_000_003, 2), (3, 70_000_005, 3)]
    {
        assert_eq!(run.current_plane(), Some(plane));
        win_current_battle(&mut run, encounter, attempt);
        assert_eq!(
            run.player_view().decision().unwrap().kind(),
            ActivityDecisionKind::Shop,
        );
        let offered = run.refresh_shop().unwrap();
        let carried_gold = run.gold();
        run.continue_supply().unwrap();
        assert_eq!(run.gold(), carried_gold);
        let rejected_hash = run.state_hash();
        assert!(run.buy_shop_offer(offered[0]).is_err());
        assert_eq!(run.state_hash(), rejected_hash);
        if plane < 3 {
            assert_eq!(
                run.player_view().decision().unwrap().kind(),
                ActivityDecisionKind::Route,
            );
            assert_eq!(run.current_plane(), Some(plane));
            run.continue_plane().unwrap();
            assert_eq!(run.current_plane(), Some(plane + 1));
        }
    }

    assert_eq!(
        run.player_view().terminal(),
        Some(ActivityTerminalOutcome::Completed),
    );
    assert_eq!(
        run.progression_projection().unwrap().unwrap().rule.position,
        CurrencyWarsRunPosition::new(3, 2).unwrap(),
    );
    let fresh = CurrencyWarsRun::start(
        definition,
        ActivityInstanceId::new(2).unwrap(),
        ActivityMasterSeed::from_u64(9),
    )
    .unwrap();
    assert_eq!(fresh.gold(), 10);
    assert_eq!(fresh.current_plane(), Some(1));
}

#[test]
fn battle_boundary_orders_victory_timeout_loss_checkpoint_and_run_failure() {
    let mut timeout = CurrencyWarsRun::start(
        definition(10),
        ActivityInstanceId::new(10).unwrap(),
        ActivityMasterSeed::from_u64(70),
    )
    .unwrap();
    assert_eq!(
        timeout
            .current_battle_boundary()
            .unwrap()
            .clock()
            .and_then(|clock| match clock {
                BattleClockSpec::ActionValue(clock) => {
                    Some(clock.remaining().scaled())
                }
                BattleClockSpec::Cycles(_) => None,
            }),
        Some(180_000_000),
    );
    let handoff = start_current_battle(&mut timeout, 70_000_001, 1);
    let rejected_hash = timeout.state_hash();
    assert!(
        timeout
            .submit_battle_result(battle_result(&handoff, BattleOutcome::Lost, 0, 1,))
            .is_err()
    );
    assert_eq!(timeout.state_hash(), rejected_hash);
    timeout
        .submit_battle_result(battle_result(&handoff, BattleOutcome::Lost, 0, 0))
        .unwrap();
    assert_eq!(timeout.squad_hp(), 0);
    assert_eq!(timeout.last_squad_hp_loss(), 105);
    assert_eq!(timeout.last_battle_progress().scaled(), 0);
    assert_eq!(timeout.last_action_value().scaled(), 0);
    assert_eq!(
        timeout.player_view().terminal(),
        Some(ActivityTerminalOutcome::Failed),
    );
    assert_eq!(
        timeout
            .progression_projection()
            .unwrap()
            .unwrap()
            .rule
            .position,
        CurrencyWarsRunPosition::new(1, 1).unwrap(),
    );

    let mut recovered = CurrencyWarsRun::start(
        definition(10),
        ActivityInstanceId::new(12).unwrap(),
        ActivityMasterSeed::from_u64(72),
    )
    .unwrap();
    let handoff = start_current_battle(&mut recovered, 70_000_001, 1);
    recovered
        .submit_battle_result(battle_result(&handoff, BattleOutcome::Lost, 100_000, 0))
        .unwrap();
    assert_eq!(recovered.squad_hp(), 5);
    assert_eq!(recovered.last_squad_hp_loss(), 95);
    assert_eq!(
        recovered.player_view().decision().unwrap().kind(),
        ActivityDecisionKind::Shop,
    );

    let mut same_boundary_victory = CurrencyWarsRun::start(
        definition(10),
        ActivityInstanceId::new(11).unwrap(),
        ActivityMasterSeed::from_u64(71),
    )
    .unwrap();
    let handoff = start_current_battle(&mut same_boundary_victory, 70_000_001, 1);
    same_boundary_victory
        .submit_battle_result(battle_result(&handoff, BattleOutcome::Won, 1_000_000, 0))
        .unwrap();
    assert_eq!(same_boundary_victory.squad_hp(), 100);
    assert_eq!(same_boundary_victory.last_squad_hp_loss(), 0);
    assert_eq!(
        same_boundary_victory
            .player_view()
            .decision()
            .unwrap()
            .kind(),
        ActivityDecisionKind::Shop,
    );
}

#[test]
fn battle_settlement_carries_participant_state_rewards_and_next_node_atomically() {
    let mut run = CurrencyWarsRun::start(
        definition(10),
        ActivityInstanceId::new(13).unwrap(),
        ActivityMasterSeed::from_u64(73),
    )
    .unwrap();
    let handoff = start_current_battle(&mut run, 70_000_001, 1);
    let carry = handoff.participant_carry()[0];
    let carried_hp = Hp::new(carry.maximum_hp().get() / 2).unwrap();
    let carried_energy = Energy::from_scaled(carry.maximum_energy().scaled() / 2).unwrap();
    let before = run.state_hash();

    let result = battle_result_with_carry(
        &handoff,
        BattleOutcome::Won,
        1_000_000,
        100_000_000,
        carried_hp,
        carried_energy,
    );
    let stale_result = result.clone();
    let resolution = run.submit_battle_result(result).unwrap();

    assert_ne!(run.state_hash(), before);
    assert_eq!(resolution.state_hash(), run.state_hash());
    assert_eq!(run.gold(), 14);
    assert_eq!(run.team_level(), 2);
    assert_eq!(run.experience(), 0);
    assert_eq!(run.last_battle_progress().scaled(), 1_000_000);
    assert_eq!(run.last_action_value().scaled(), 100_000_000);
    assert_eq!(
        run.player_view().decision().unwrap().kind(),
        ActivityDecisionKind::Shop,
    );
    let settled = run.player_view().participant_carry()[0];
    assert_eq!(settled.current_hp(), carried_hp);
    assert_eq!(settled.current_energy(), carried_energy);
    let settled_state = run.state_hash();
    let settled_rng = run.debug_view().rng().to_vec();
    assert!(run.submit_battle_result(stale_result).is_err());
    assert_eq!(run.state_hash(), settled_state);
    assert_eq!(run.debug_view().rng(), settled_rng);

    run.continue_supply().unwrap();
    run.continue_plane().unwrap();
    let next = start_current_battle(&mut run, 70_000_003, 2);
    assert_eq!(next.participant_carry()[0].current_hp(), carried_hp);
    assert_eq!(next.participant_carry()[0].current_energy(), carried_energy);
}

#[test]
fn rejected_generated_settlement_follow_up_restores_activity_and_rng() {
    let mut run = CurrencyWarsRun::start(
        definition(10),
        ActivityInstanceId::new(14).unwrap(),
        ActivityMasterSeed::from_u64(74),
    )
    .unwrap();
    let handoff = start_current_battle(&mut run, 70_000_001, 1);
    let result = battle_result(&handoff, BattleOutcome::Won, 1_000_000, 100_000_000);
    let retry = result.clone();
    let before = run.state_hash();
    let rng = run.debug_view().rng().to_vec();

    assert!(
        run.submit_battle_result_with_rejected_follow_up_fixture(result)
            .is_err()
    );
    assert_eq!(run.state_hash(), before);
    assert_eq!(run.debug_view().rng(), rng);

    run.submit_battle_result(retry).unwrap();
    assert_ne!(run.state_hash(), before);
}

fn definition(initial_gold: u32) -> Arc<CurrencyWarsRunDefinition> {
    definition_with_catalog(initial_gold, Arc::new(tests_support::catalog()))
}

fn definition_with_catalog(
    initial_gold: u32,
    catalog: Arc<CurrencyWarsCatalog>,
) -> Arc<CurrencyWarsRunDefinition> {
    let role = CurrencyWarsRoleId::new(1001).unwrap();
    let state = CurrencyWarsRoleState::new(role, 1).unwrap();
    let roster = CurrencyWarsRoster::new(&catalog, [(state, 1)]).unwrap();
    let deployment = CurrencyWarsDeployment::new(
        &catalog,
        &roster,
        1,
        [(
            CurrencyWarsPosition::new(CurrencyWarsPositionKind::Front, 1).unwrap(),
            state,
        )],
    )
    .unwrap();
    Arc::new(
        CurrencyWarsRunDefinition::new(
            identity(),
            Arc::clone(&catalog),
            catalog.routes()[0].id,
            catalog.difficulties()[0].source_id,
            CurrencyWarsGambit::Standard,
            CurrencyWarsEntryState::new(21, false, 1),
            CurrencyWarsRunSetup {
                initial_gold,
                initial_team_level: 1,
                initial_experience: 0,
                roster,
                deployment,
                enemy_affix_ids: Box::new([]),
                owned_builds: BTreeMap::new(),
            },
        )
        .unwrap(),
    )
}

fn identity() -> ActivityDefinitionIdentity {
    ActivityDefinitionIdentity::new(
        ActivityDefinitionId::new(1).unwrap(),
        ActivityDefinitionDigest::new([1; 32]).unwrap(),
        ActivityConfigDigest::new([2; 32]).unwrap(),
    )
}

fn win_current_battle(run: &mut CurrencyWarsRun, encounter: u32, attempt: u32) {
    let handoff = start_current_battle(run, encounter, attempt);
    run.submit_battle_result(battle_result(
        &handoff,
        BattleOutcome::Won,
        1_000_000,
        100_000_000,
    ))
    .unwrap();
}

fn start_current_battle(
    run: &mut CurrencyWarsRun,
    encounter: u32,
    attempt: u32,
) -> ActivityBattleHandoff {
    run.engage_current_node_fixture(
        AttemptId::new(attempt).unwrap(),
        battle(encounter, u8::try_from(attempt).unwrap()),
    )
    .unwrap();
    run.choose_prepared_battle().unwrap();
    run.start_pending_battle().unwrap()
}

fn battle_result(
    handoff: &ActivityBattleHandoff,
    outcome: BattleOutcome,
    progress: i64,
    remaining_action_value: i64,
) -> BattleResult {
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
            value: MetricValue::ActionValue(remaining_action_value),
        },
        ProjectedValue::Metric {
            key: CURRENCY_WARS_BATTLE_PROGRESS_KEY.into(),
            value: MetricValue::Ratio(progress),
        },
    ]);
    BattleResult::seal(handoff.identity(), values)
}

fn battle_result_with_carry(
    handoff: &ActivityBattleHandoff,
    outcome: BattleOutcome,
    progress: i64,
    remaining_action_value: i64,
    current_hp: Hp,
    current_energy: Energy,
) -> BattleResult {
    let mut values = vec![
        ProjectedValue::Outcome(outcome),
        ProjectedValue::FinalStateHash(BattleStateHash::from_bytes([0x73; 32])),
        ProjectedValue::EventDigest(EventDigest::new([0x74; 32]).unwrap()),
        ProjectedValue::TerminalFault(None),
    ];
    values.extend(handoff.participant_carry().iter().map(|carry| {
        ProjectedValue::ParticipantState(
            ParticipantBattleState::new(
                carry.participant(),
                current_hp,
                carry.maximum_hp(),
                current_energy,
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
            value: MetricValue::ActionValue(remaining_action_value),
        },
        ProjectedValue::Metric {
            key: CURRENCY_WARS_BATTLE_PROGRESS_KEY.into(),
            value: MetricValue::Ratio(progress),
        },
    ]);
    BattleResult::seal(handoff.identity(), values)
}

fn battle(encounter: u32, digest: u8) -> BattleSpec {
    BattleSpec::new(
        AssemblyDigest::new([digest; 32]).unwrap(),
        EncounterId::new(encounter).unwrap(),
        vec![
            ParticipantSpec::new(
                TeamSide::Player,
                FormationIndex::new(0).unwrap(),
                ParticipantSource::Player,
                combatant(1, 3),
            ),
            ParticipantSpec::new(
                TeamSide::Enemy,
                FormationIndex::new(0).unwrap(),
                ParticipantSource::EncounterEnemy(EnemyDefinitionId::new(1).unwrap()),
                combatant(2, digest.wrapping_add(0x20)),
            ),
        ],
        TeamResourceSpec::new(3, 5).unwrap(),
        TeamResourceSpec::new(0, 0).unwrap(),
        ConcedePolicy::Allowed,
    )
    .unwrap()
}

fn combatant(form: u32, digest: u8) -> ResolvedCombatantSpec {
    ResolvedCombatantSpec::new(
        UnitDefinitionId::new(form).unwrap(),
        UnitLevel::new(80).unwrap(),
        Hp::new(1_000).unwrap(),
        Speed::from_scaled(100_000_000).unwrap(),
        ResolvedDefinitionBindings::new(vec![AbilityId::new(form).unwrap()], vec![], vec![])
            .unwrap(),
        CombatantSpecDigest::new([digest; 32]).unwrap(),
    )
    .unwrap()
}
