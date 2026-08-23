use std::sync::Arc;

use starclock_activity::{
    ActivityCause, ActivityConfigDigest, ActivityDefinitionDigest, ActivityDefinitionId,
    ActivityDefinitionIdentity, ActivityInstanceId, ActivityMasterSeed, ActivityRngContext,
    ActivityRngStreams, ActivityTransactionOutcome, ActivityTransactionState,
};
use starclock_combat::{
    CombatantSpecDigest, Energy, Hp, ResolvedCombatantSpec, ResolvedDefinitionBindings, Speed,
    StatValue, TeamSide, UnitLevel, catalog::action::AbilityKind,
};

use crate::battle_materialization::{UniverseBattleRoster, player_participants};

use super::{
    GOLD_AND_GEARS_BATTLE_SNAPSHOT_REVISION,
    GoldAndGearsBattleAssemblyContext, GoldAndGearsEncounterSelection, GoldAndGearsEntryError,
    GoldAndGearsRuntimeInstance,
};
use super::{tests};

#[test]
fn current_activity_snapshot_materializes_a_real_validated_battle() {
    let instance = tests::compiled_battle_fixture(tests::shared_factory());
    let (state, selection) = selected_combat(&instance, 0x1406_0201);
    let roster = roster(&instance);
    let context = GoldAndGearsBattleAssemblyContext::new(Vec::new(), false);
    let snapshot = instance.compile_battle_snapshot(&state, &context).unwrap();
    assert_eq!(
        player_participants(
            &instance.content_runtime.standard,
            &roster,
            &snapshot.shared,
            None,
            &[],
        )
        .unwrap()
        .len(),
        4
    );

    let first = instance
        .materialize_current_battle(&state, &selection, &roster, &context)
        .unwrap();
    let second = instance
        .materialize_current_battle(&state, &selection, &roster, &context)
        .unwrap();

    assert_eq!(first.digest(), second.digest());
    assert_eq!(
        digest_hex(first.digest()),
        "a8a1352250fa1d2f8679b7c4afa86b062696ac521af0945aeeb5411fdbbea886"
    );
    assert_eq!(
        digest_hex(first.battle_spec().combat_input_digest().bytes()),
        "d0c5f0e1041ff3a94952031d9b434af7b54e260796d11bee2d729896085bd3b7"
    );
    assert_eq!(
        digest_hex(first.enemy_definition_digest()),
        "6a463b116ffaa5462f16795b5f90ace8d07daa334573a1864f159e820789ad09"
    );
    assert_eq!(first.battle_spec(), second.battle_spec());
    assert_eq!(first.participant_lock(), instance.participants().digest());
    assert_eq!(first.enemy_definition_count(), 90);
    assert_eq!(first.mode_owned_enemy_definition_count(), 23);
    assert_eq!(first.reviewed_stat_source_count(), 10);
    assert_eq!(first.fallback_stat_source_count(), 80);
    assert_eq!(
        first.reviewed_stat_source_count() + first.fallback_stat_source_count(),
        90
    );
    assert_eq!(first.enemy_definitions().len(), 90);
    assert!(first.enemy_definitions().windows(2).all(|pair| {
        pair[0].stable_key() < pair[1].stable_key()
    }));
    assert!(first.enemy_definitions().iter().all(|binding| {
        !binding.behavior_source_key().is_empty()
            && first
                .combat_catalog()
                .enemy(binding.combat_enemy())
                .is_some()
    }));
    assert_eq!(
        first
            .enemy_definitions()
            .iter()
            .filter(|binding| binding.mode_owned())
            .count(),
        23
    );

    let expected_enemies = selection
        .waves()
        .iter()
        .map(|wave| wave.slots().len())
        .sum::<usize>();
    assert_eq!(
        first
            .battle_spec()
            .participants()
            .iter()
            .filter(|participant| participant.side() == TeamSide::Player)
            .count(),
        4
    );
    assert_eq!(
        first
            .battle_spec()
            .participants()
            .iter()
            .filter(|participant| participant.side() == TeamSide::Enemy)
            .count(),
        expected_enemies
    );
    assert_eq!(
        GOLD_AND_GEARS_BATTLE_SNAPSHOT_REVISION,
        "gold-and-gears-battle-snapshot-v1"
    );
}

#[test]
fn repeated_current_battle_resolution_uses_bounded_non_authoritative_cache() {
    let instance = tests::compiled_battle_fixture(tests::shared_factory());
    let (state, selection) = selected_combat(&instance, 0x1408_0201);
    let roster = roster(&instance);
    let context = GoldAndGearsBattleAssemblyContext::new(Vec::new(), false);
    let state_hash = starclock_activity::ActivityStateHash::new([0x82; 32]).unwrap();
    let before = instance.battle_assembly_cache_metrics();
    let first = instance
        .resolve_current_battle(state_hash, &state, &selection, &roster, &context)
        .unwrap();
    let after_miss = instance.battle_assembly_cache_metrics();
    let second = instance
        .resolve_current_battle(state_hash, &state, &selection, &roster, &context)
        .unwrap();
    let after_hit = instance.battle_assembly_cache_metrics();

    assert!(Arc::ptr_eq(&first, &second));
    assert_eq!(first.digest(), second.digest());
    assert_eq!(after_miss.misses - before.misses, 1);
    assert_eq!(after_miss.entries - before.entries, 1);
    assert_eq!(after_hit.hits - after_miss.hits, 1);
    assert_eq!(after_hit.misses, after_miss.misses);
    assert_eq!(after_hit.evictions, after_miss.evictions);
}

#[test]
fn owned_blessings_and_curios_change_the_immutable_contribution_snapshot() {
    let instance = tests::compiled_battle_fixture(tests::shared_factory());
    let (mut state, selection) = selected_combat(&instance, 0x1406_0202);
    let roster = roster(&instance);
    let context = GoldAndGearsBattleAssemblyContext::new(Vec::new(), true);
    instance.compile_battle_snapshot(&state, &context).unwrap();
    let empty = instance
        .materialize_current_battle(&state, &selection, &roster, &context)
        .unwrap();

    let blessing = instance.content_runtime.blessings.definitions()[0].blessing();
    let shared_curio = instance
        .curio_definitions()
        .iter()
        .find(|definition| definition.shared_curio().is_some())
        .unwrap()
        .id();
    let gold_curio = instance
        .curio_definitions()
        .iter()
        .find(|definition| definition.shared_curio().is_none())
        .unwrap()
        .id();
    commit(
        &instance,
        &mut state,
        instance.compile_blessing_acquisition(blessing).unwrap(),
    );
    commit(
        &instance,
        &mut state,
        instance.compile_curio_acquisition(shared_curio).unwrap(),
    );
    commit(
        &instance,
        &mut state,
        instance.compile_curio_acquisition(gold_curio).unwrap(),
    );

    let populated = instance
        .materialize_current_battle(&state, &selection, &roster, &context)
        .unwrap();
    assert_eq!(populated.contributions().blessing_count(), 1);
    assert_eq!(populated.contributions().curio_count(), 2);
    assert_eq!(populated.contributions().gold_curio_count(), 1);
    assert_ne!(empty.contributions().digest(), populated.contributions().digest());
    assert_ne!(empty.digest(), populated.digest());
    assert_ne!(
        empty.battle_spec().assembly_digest(),
        populated.battle_spec().assembly_digest()
    );
}

#[test]
fn stale_encounter_selection_is_rejected_without_mutating_activity_state() {
    let instance = tests::compiled_battle_fixture(tests::shared_factory());
    let (mut state, selection) = selected_combat(&instance, 0x1406_0203);
    let roster = roster(&instance);
    let node = state.current_node();
    commit(
        &instance,
        &mut state,
        instance
            .compile_node_replacement(node, "gold-gears.domain.reward", None)
            .unwrap(),
    );
    let rng = activity_rng(&instance, 0x1406_0204);
    let before = state.state_hash(
        activity_identity(),
        instance.graph_definition(),
        ActivityInstanceId::new(1).unwrap(),
        &rng,
    );
    let sequence = state.command_sequence();

    assert!(matches!(
        instance.materialize_current_battle(
            &state,
            &selection,
            &roster,
            &GoldAndGearsBattleAssemblyContext::new(Vec::new(), false),
        ),
        Err(GoldAndGearsEntryError::InvalidBattleMaterialization)
    ));
    assert_eq!(state.command_sequence(), sequence);
    assert_eq!(
        state.state_hash(
            activity_identity(),
            instance.graph_definition(),
            ActivityInstanceId::new(1).unwrap(),
            &rng,
        ),
        before
    );
}

pub(super) fn selected_combat(
    instance: &GoldAndGearsRuntimeInstance,
    seed: u64,
) -> (ActivityTransactionState, GoldAndGearsEncounterSelection) {
    let node = instance.encounter_runtime.node_at(1, 2).unwrap();
    let mut state = ActivityTransactionState::new(instance.state_definition().clone(), node);
    commit(
        instance,
        &mut state,
        instance
            .compile_node_replacement(node, "gold-gears.domain.monsternormal", None)
            .unwrap(),
    );
    let mut rng = activity_rng(instance, seed);
    let selection = instance.select_current_encounter(&state, &mut rng).unwrap();
    (state, selection)
}

pub(super) fn roster(instance: &GoldAndGearsRuntimeInstance) -> UniverseBattleRoster {
    roster_with_stats(instance, 100_000, 100_000_000, 100_000_000, 100_000_000)
}

/// Balance-independent roster for complete-run integration coverage. These
/// deliberately synthetic values are not an observed numeric-parity claim.
pub(super) fn seeded_matrix_roster(
    instance: &GoldAndGearsRuntimeInstance,
) -> UniverseBattleRoster {
    roster_with_stats(
        instance,
        1_000_000_000,
        1_000_000_000,
        1_000_000_000_000,
        1_000_000_000_000,
    )
}

fn roster_with_stats(
    instance: &GoldAndGearsRuntimeInstance,
    hp: i64,
    speed_scaled: i64,
    attack_scaled: i64,
    defense_scaled: i64,
) -> UniverseBattleRoster {
    let combat = instance
        .content_runtime
        .standard
        .simulation_catalog()
        .combat_catalog();
    let combatants = instance
        .participants()
        .entries()
        .iter()
        .map(|locked| {
            let unit = combat.unit(locked.character()).unwrap();
            let basic = unit
                .abilities()
                .iter()
                .copied()
                .find(|ability| {
                    combat
                        .ability(*ability)
                        .and_then(|definition| definition.action())
                        .is_some_and(|action| action.kind() == AbilityKind::Basic)
                })
                .unwrap();
            let spec = ResolvedCombatantSpec::new(
                locked.character(),
                UnitLevel::new(80).unwrap(),
                Hp::new(hp).unwrap(),
                Speed::from_scaled(speed_scaled).unwrap(),
                ResolvedDefinitionBindings::new(vec![basic], Vec::new(), Vec::new()).unwrap(),
                CombatantSpecDigest::new(locked.build().resolved_spec_digest().bytes()).unwrap(),
            )
            .unwrap()
            .with_base_attack_defense(
                StatValue::from_scaled(attack_scaled).unwrap(),
                StatValue::from_scaled(defense_scaled).unwrap(),
            )
            .with_energy(Energy::ZERO, Energy::from_scaled(100_000_000).unwrap())
            .unwrap();
            (locked.participant(), spec)
        })
        .collect();
    UniverseBattleRoster::new(instance.participants(), combatants).unwrap()
}

pub(super) fn commit(
    instance: &GoldAndGearsRuntimeInstance,
    state: &mut ActivityTransactionState,
    program: starclock_activity::ActivityProgramDefinition,
) {
    program
        .validate_against(instance.state_definition(), instance.graph_definition())
        .unwrap();
    let cause = ActivityCause::new(
        state.command_sequence() + 1,
        program.id(),
        state.current_node(),
    )
    .unwrap();
    assert!(matches!(
        state.apply_program(&program, cause, instance.graph_definition()),
        ActivityTransactionOutcome::Committed(_)
    ));
}

pub(super) fn activity_rng(instance: &GoldAndGearsRuntimeInstance, seed: u64) -> ActivityRngStreams {
    let identity = activity_identity();
    ActivityRngStreams::new(ActivityRngContext::new(
        ActivityMasterSeed::from_u64(seed),
        identity.id(),
        identity.definition_digest(),
        identity.config_digest(),
        instance.graph_definition().digest(),
        ActivityInstanceId::new(1).unwrap(),
        None,
        Some(instance.graph_definition().entry()),
        None,
        0,
    ))
}

pub(super) fn activity_identity() -> ActivityDefinitionIdentity {
    ActivityDefinitionIdentity::new(
        ActivityDefinitionId::new(14).unwrap(),
        ActivityDefinitionDigest::new([0x14; 32]).unwrap(),
        ActivityConfigDigest::new([0x62; 32]).unwrap(),
    )
}

fn digest_hex(digest: [u8; 32]) -> String {
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}
