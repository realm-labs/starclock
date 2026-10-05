#!/usr/bin/env node

// Current source taxonomy and capability inventory, never an execution receipt.
import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const ref = "content-reference/divergent-universe-v1";
const output = "content-manifests/divergent-universe-runtime-v1/grand-miracle-gamble-execution.json";
const inputs = {
  hex_references: `${ref}/weighted-curios.json`,
  hex_eligibility: `${ref}/weighted-curio-eligibility.json`,
  hex_states: `${ref}/weighted-curio-states.json`,
  gamble_groups: `${ref}/gamble-groups.json`,
  gamble_units: `${ref}/gamble-units.json`,
  curio_catalog: "crates/starclock-data/src/divergent_universe_curio_catalog.rs",
  curio_lowering: "crates/starclock-data/src/divergent_universe_curio.rs",
  taxonomy_tests: "crates/starclock-data/src/divergent_universe_tests.rs",
  mode_facade: "crates/starclock-mode-universe/src/divergent_universe/mod.rs",
  weighted_loadout: "crates/starclock-mode-universe/src/divergent_universe/weighted_curio.rs",
  weighted_loadout_tests: "crates/starclock-mode-universe/src/divergent_universe/tests/weighted_curio.rs",
  weighted_equipment_room: "crates/starclock-mode-universe/src/divergent_universe/weighted_curio_room.rs",
  weighted_equipment_room_binding: "crates/starclock-mode-universe/src/divergent_universe/weighted_curio_room_binding.rs",
  weighted_equipment_room_tests: "crates/starclock-mode-universe/src/divergent_universe/tests/weighted_curio_room.rs",
  weighted_equipment_position_binding: "crates/starclock-mode-universe/src/divergent_universe/position_weighted_curio.rs",
  weighted_equipment_profile_binding: "crates/starclock-mode-universe/src/divergent_universe/battle_room_binding.rs",
  weighted_equipment_controller: "crates/starclock-mode-universe/src/divergent_universe/baseline_runtime.rs",
  weighted_equipment_profile_tests: "crates/starclock-mode-universe/src/divergent_universe/tests/weighted_curio_room_profile.rs",
  weighted_equipment_profile_fixture: "crates/starclock-mode-universe/src/divergent_universe/tests/weighted_curio_room_profile_fixture.rs",
  weighted_overflow: "crates/starclock-mode-universe/src/divergent_universe/weighted_curio_overflow.rs",
  weighted_overflow_data: "config/divergent-universe-decisions-generated/debug-json/DuWeightedCurioOverflows.json",
  weighted_footstep: "crates/starclock-mode-universe/src/divergent_universe/weighted_curio_footstep.rs",
  weighted_footstep_data: "config/divergent-universe-decisions-generated/debug-json/DuWeightedCurioFootsteps.json",
  weighted_deflagration: "crates/starclock-mode-universe/src/divergent_universe/weighted_curio_deflagration.rs",
  weighted_deflagration_native: "crates/starclock-mode-universe/src/divergent_universe/weighted_curio_deflagration/native.rs",
  weighted_deflagration_equipment: "crates/starclock-mode-universe/src/divergent_universe/weighted_curio_deflagration/equipment.rs",
  weighted_deflagration_data: "config/divergent-universe-decisions-generated/debug-json/DuWeightedCurioDeflagrations.json",
  weighted_splash: "crates/starclock-mode-universe/src/divergent_universe/weighted_curio_splash.rs",
  weighted_splash_tests: "crates/starclock-mode-universe/src/divergent_universe/tests/weighted_curio_splash.rs",
  weighted_splash_data: "config/divergent-universe-decisions-generated/debug-json/DuWeightedCurioSplashes.json",
  weighted_shield: "crates/starclock-mode-universe/src/divergent_universe/weighted_curio_shield.rs",
  weighted_shield_tests: "crates/starclock-mode-universe/src/divergent_universe/tests/weighted_curio_shield.rs",
  weighted_shield_data: "config/divergent-universe-decisions-generated/debug-json/DuWeightedCurioShields.json",
  weighted_attack_debuff: "crates/starclock-mode-universe/src/divergent_universe/weighted_curio_attack_debuff.rs",
  weighted_attack_debuff_tests: "crates/starclock-mode-universe/src/divergent_universe/tests/weighted_curio_attack_debuff.rs",
  weighted_attack_debuff_data: "config/divergent-universe-decisions-generated/debug-json/DuWeightedCurioAttackDebuffs.json",
  weighted_support_attack: "crates/starclock-mode-universe/src/divergent_universe/weighted_curio_support_attack.rs",
  weighted_support_attack_tests: "crates/starclock-mode-universe/src/divergent_universe/tests/weighted_curio_support_attack.rs",
  weighted_support_attack_data: "config/divergent-universe-decisions-generated/debug-json/DuWeightedCurioSupportAttacks.json",
  weighted_prayer: "crates/starclock-mode-universe/src/divergent_universe/weighted_curio_prayer.rs",
  weighted_prayer_tests: "crates/starclock-mode-universe/src/divergent_universe/tests/weighted_curio_prayer.rs",
  weighted_prayer_data: "config/divergent-universe-decisions-generated/debug-json/DuWeightedCurioPrayers.json",
  weighted_retaliation: "crates/starclock-mode-universe/src/divergent_universe/weighted_curio_retaliation.rs",
  weighted_retaliation_tests: "crates/starclock-mode-universe/src/divergent_universe/tests/weighted_curio_retaliation.rs",
  weighted_counter_composition_tests: "crates/starclock-mode-universe/src/divergent_universe/tests/weighted_curio_counter_composition.rs",
  weighted_retaliation_data: "config/divergent-universe-decisions-generated/debug-json/DuWeightedCurioRetaliations.json",
  weighted_break_effect: "crates/starclock-mode-universe/src/divergent_universe/weighted_curio_break_effect.rs",
  weighted_break_effect_tests: "crates/starclock-mode-universe/src/divergent_universe/tests/weighted_curio_break_effect.rs",
  weighted_break_effect_data: "config/divergent-universe-decisions-generated/debug-json/DuWeightedCurioBreakEffects.json",
  weighted_necrosis: "crates/starclock-mode-universe/src/divergent_universe/weighted_curio_necrosis.rs",
  weighted_necrosis_tests: "crates/starclock-mode-universe/src/divergent_universe/tests/weighted_curio_necrosis.rs",
  weighted_burn_fixture: "crates/starclock-mode-universe/src/divergent_universe/tests/weighted_curio_burn_fixture.rs",
  weighted_necrosis_data: "config/divergent-universe-decisions-generated/debug-json/DuWeightedCurioNecroses.json",
  weighted_elation: "crates/starclock-mode-universe/src/divergent_universe/weighted_curio_elation.rs",
  weighted_elation_tests: "crates/starclock-mode-universe/src/divergent_universe/tests/weighted_curio_elation.rs",
  weighted_elation_fixture: "crates/starclock-mode-universe/src/divergent_universe/tests/weighted_curio_elation_fixture.rs",
  weighted_elation_data: "config/divergent-universe-decisions-generated/debug-json/DuWeightedCurioElations.json",
  weighted_excitation_authoring: "crates/starclock-data/src/divergent_universe_weighted_curio_excitation_data.rs",
  weighted_excitation_data: "config/divergent-universe-decisions-generated/debug-json/DuWeightedCurioExcitations.json",
  weighted_excitation: "crates/starclock-mode-universe/src/divergent_universe/weighted_curio_excitation.rs",
  weighted_excitation_tests: "crates/starclock-mode-universe/src/divergent_universe/tests/weighted_curio_excitation.rs",
  weighted_excitation_fixture: "crates/starclock-mode-universe/src/divergent_universe/tests/weighted_curio_excitation_fixture.rs",
  weighted_encouragement_authoring: "crates/starclock-data/src/divergent_universe_weighted_curio_encouragement_data.rs",
  weighted_encouragement_tests: "crates/starclock-data/src/divergent_universe_weighted_curio_encouragement_tests.rs",
  weighted_encouragement_data: "config/divergent-universe-decisions-generated/debug-json/DuWeightedCurioEncouragements.json",
  weighted_encouragement: "crates/starclock-mode-universe/src/divergent_universe/weighted_curio_encouragement.rs",
  weighted_encouragement_runtime_tests: "crates/starclock-mode-universe/src/divergent_universe/tests/weighted_curio_encouragement.rs",
  weighted_encouragement_fixture: "crates/starclock-mode-universe/src/divergent_universe/tests/weighted_curio_encouragement_fixture.rs",
  weighted_transfer_authoring: "crates/starclock-data/src/divergent_universe_weighted_curio_transfer_data.rs",
  weighted_transfer_data: "config/divergent-universe-decisions-generated/debug-json/DuWeightedCurioTransfers.json",
  weighted_transfer: "crates/starclock-mode-universe/src/divergent_universe/weighted_curio_transfer.rs",
  weighted_transfer_tests: "crates/starclock-mode-universe/src/divergent_universe/tests/weighted_curio_transfer.rs",
  weighted_transfer_fixture: "crates/starclock-mode-universe/src/divergent_universe/tests/weighted_curio_transfer_fixture.rs",
  contribution_snapshot: "crates/starclock-mode-universe/src/divergent_universe/contribution_snapshot.rs",
  gamble_runtime: "crates/starclock-mode-universe/src/divergent_universe/gamble_runtime.rs",
  tests: "crates/starclock-mode-universe/src/divergent_universe/tests/gamble_runtime.rs",
};
const revision = "fd978d6ef09f941fba644c731ab54abd6f7c3568";

export function buildGrandMiracleGambleExecution() {
  const hex = json(inputs.hex_references);
  const eligibility = json(inputs.hex_eligibility);
  const states = json(inputs.hex_states);
  const groups = json(inputs.gamble_groups);
  const units = json(inputs.gamble_units);
  const current = eligibility.filter((row) => row.selector_scope === "Tourn3");
  const otherMode = eligibility.filter((row) => row.selector_scope !== "Tourn3");
  const coins = units.filter((row) => row.outcome_program.operation === "GainRunCurrency");
  assert(hex.length === 17 && current.length === 17 && otherMode.length === 57
    && states.length === 34, "Hex reference denominator drift");
  assert(hex.every((row) => row.content_kind === "WeightedCurio"
    && row.kind === "DivergentUniverseWeightedCurio"
    && row.id.startsWith("divergent-universe.weighted-curio.")
    && !row.runtime_lowered && row.source_refs.some((source) =>
    source.path === "ExcelOutput/RogueTournHex.json" && source.revision === revision)),
  "Hex source/reference boundary drift");
  assert(groups.length === 126 && groups.every((row) => row.unit_ids.length === 0
    && row.fallback === "RejectWithoutMutation" && !row.runtime_lowered),
  "unresolved Gamble groups must remain fail-closed");
  assert(units.length === 89 && coins.length === 2
    && equal(coins.map((row) => row.outcome_program.amount), ["20", "40"]),
  "Gamble Coin source drift");
  assert(text(inputs.curio_lowering).includes(
    "content_kind(r.content_kind, &v.content_kind)?"),
  "owned Hex lowering must bind the authored column and payload classification");
  assert(text(inputs.curio_catalog).includes(
    "Hex references are Weighted Curios, not Grand Miracles"),
  "owned catalog must reject Grand Miracle classification of Hex references");
  assert(!text(inputs.mode_facade).includes("grand_miracle_runtime"),
    "reference flags must not be exposed as a Grand Miracle runtime");
  const probes = [
    ...[
      "equipment_room_all_seventeen_toggle_through_authenticated_offers_without_rng",
      "equipment_room_capacity_clear_canonical_order_and_independent_leave_reconstruct",
      "equipment_room_exact_change_budget_keeps_a_safe_leave_and_rejections_inert",
      "equipment_room_foreign_dirty_and_next_entry_failure_restore_the_entire_activity",
      "equipment_room_binding_rejects_missing_entry_lifecycle_and_spoofed_programs",
      "equipment_room_binding_rejects_menu_entry_bypass_and_wrong_logical_room",
      "equipment_room_compiles_each_current_reforge_in_all_nine_authored_decks",
    ].map(test => ({file: inputs.weighted_equipment_room_tests, test})),
    ...[
      "weighted_curio_profile_authenticates_all_decks_and_rejects_altered_attachments",
      "weighted_curio_profile_raw_unbound_foreign_stale_and_hidden_choices_are_inert",
      "weighted_curio_profile_selected_equipment_reconstructs_and_executes_real_battle_effects",
      "weighted_curio_profile_unsupported_menu_selection_still_rejects_controller_battle_atomically",
    ].map(test => ({file: inputs.weighted_equipment_profile_tests, test})),
    ...[
      "weighted_curio_transfer_binds_all_original_recipients_only_with_preservation_party",
      "weighted_curio_transfer_preserves_threshold_precision_and_heals_only_actual_decay",
      "weighted_curio_transfer_uses_recipient_turn_and_live_maximum_hp_not_other_turns",
      "weighted_curio_transfer_absorption_and_explicit_removal_do_not_heal_or_recurse",
      "weighted_curio_transfer_fresh_commands_are_deterministic_and_stale_commands_inert",
      "weighted_curio_transfer_provider_defeat_absence_and_inherited_linked_bundles_do_not_change_scope",
      "weighted_curio_transfer_unmodified_production_battle_starts_with_four_zero_capacity_markers",
    ].map(test => ({file: inputs.weighted_transfer_tests, test})),
    {file: inputs.weighted_loadout_tests,
      test: "weighted_curio_excitation_authoring_admits_only_the_bound_sora_definition"},
    {file: inputs.weighted_loadout_tests,
      test: "weighted_curio_encouragement_authoring_admits_only_the_bound_sora_definition"},
    ...[
      "weighted_curio_encouragement_mixed_calculators_and_native_follow_up_do_not_retag_actions",
      "weighted_curio_encouragement_shared_crit_reuses_decision_not_scoped_critical_damage",
      "weighted_curio_encouragement_real_linked_units_with_inherited_bundles_and_forced_effect_are_excluded",
      "weighted_curio_encouragement_unitless_timeline_actor_cannot_borrow_owner_critical_bonus_or_alias",
      "weighted_curio_encouragement_refresh_wave_and_fresh_battle_keep_one_owner_contribution",
      "weighted_curio_encouragement_unequip_and_empty_eligible_roster_have_no_effect",
      "weighted_curio_encouragement_fresh_reconstruction_and_stale_rejection_preserve_events_state_rng",
    ].map(test => ({file: inputs.weighted_encouragement_runtime_tests, test})),
    ...[
      "production_weighted_curio_encouragement_preserves_operand_without_execution_credit",
      "weighted_curio_encouragement_rejects_wrong_joins_noncanonical_operands_and_false_parity",
      "weighted_curio_encouragement_rejects_forged_provenance_at_every_required_source",
      "weighted_curio_encouragement_key_cannot_alias_an_existing_effect_family",
    ].map(test => ({file: inputs.weighted_encouragement_tests, test})),
    ...[
      "weighted_curio_excitation_effective_gain_excludes_overflow_cap_spend_and_nonquantum",
      "weighted_curio_excitation_partial_roster_and_cap_use_recipient_local_counts",
      "weighted_curio_excitation_multitarget_additional_crit_is_independent_of_nevercrit_hits",
      "weighted_curio_excitation_resistance_and_lethal_additional_do_not_apply_a_marker",
      "weighted_curio_excitation_uncertain_crit_and_effect_chance_use_independent_reproducible_draws",
      "weighted_curio_excitation_empty_quantum_roster_and_unequip_have_no_contribution",
      "weighted_curio_excitation_entanglement_refresh_hits_cleanse_and_expiry_are_native",
      "weighted_curio_excitation_stacks_survive_provider_defeat_and_wave_but_not_fresh_battle",
      "weighted_curio_excitation_inherited_linked_and_shared_actors_cannot_gain_or_consume",
      "weighted_curio_excitation_fresh_reconstruction_and_rejections_preserve_payloads_rng_and_state",
      "weighted_curio_excitation_unmodified_production_quantum_basic_executes_and_reconstructs",
      "weighted_curio_excitation_gain_is_per_effective_event_and_consumes_once_per_multihit_action",
    ].map(test => ({file: inputs.weighted_excitation_tests, test})),
    ...[
      "weighted_curio_elation_production_basic_and_nonattack_skill_execute_both_families",
      "weighted_curio_elation_multihit_refresh_stat_query_and_overflow_are_real_mutations",
      "weighted_curio_elation_recipient_turn_expiry_survives_caster_defeat",
      "weighted_curio_elation_presence_and_excluded_owners_do_not_invent_recipients",
      "weighted_curio_elation_fresh_reconstruction_rejections_and_unequip_are_inert",
      "weighted_curio_elation_dispel_and_wave_transition_retain_recipient_scope_and_meter",
      "weighted_curio_elation_queued_non_basic_skill_actions_never_trigger",
      "weighted_curio_elation_actual_linked_and_shared_actors_cannot_inherit_original_owner_trigger",
    ].map((test) => ({file: inputs.weighted_elation_tests, test})),
    { file: inputs.weighted_counter_composition_tests,
      test: "production_clara_counter_exhaustion_does_not_fault_with_weighted_curio_retaliation" },
    { file: inputs.taxonomy_tests,
      test: "production_hex_source_references_are_weighted_curios_not_grand_miracles" },
    ...[
      "weighted_curio_all_current_selections_replace_unequip_and_reconstruct_both_families",
      "weighted_curio_capacity_canonical_order_and_invalid_requests_are_atomic",
      "weighted_curio_unlowered_and_dirty_loadouts_reject_contribution_without_mutation_or_rng",
      "weighted_curio_completed_activity_rejects_equipment_without_events",
      "weighted_curio_unequip_restores_real_battle_execution_not_effect_credit",
    ].map((test) => ({ file: inputs.weighted_loadout_tests, test })),
    ...[
      "weighted_curio_splash_preserves_build_and_executes_each_hit_adjacent_copy_in_both_families",
      "weighted_curio_splash_unequip_removes_binding_and_stale_snapshot_is_rejected",
    ].map((test) => ({ file: inputs.weighted_splash_tests, test })),
    ...[
      "weighted_curio_shield_executes_per_recipient_hp_once_for_all_three_ally_actions_both_families",
      "weighted_curio_shield_refreshes_without_capacity_stacking_then_expires_on_recipient_turns",
    ].map((test) => ({ file: inputs.weighted_shield_tests, test })),
    ...[
      "weighted_curio_attack_debuff_executes_both_paths_once_per_attack_target_and_refreshes",
      "weighted_curio_attack_debuff_reduces_actual_enemy_damage_then_restores_after_target_turn",
      "weighted_curio_attack_debuff_rejects_non_attack_allied_noneligible_and_dead_targets",
      "weighted_curio_attack_debuff_preserves_build_activity_and_reconstructs_fresh_battles",
    ].map((test) => ({ file: inputs.weighted_attack_debuff_tests, test })),
    ...[
      "weighted_curio_support_attack_executes_all_paths_once_per_target_for_multihit_and_ultimate",
      "weighted_curio_support_attack_crit_stats_scale_with_roster_and_additional_cannot_crit",
      "weighted_curio_support_attack_rejects_noneligible_nonattack_allied_and_defeated_targets",
      "weighted_curio_support_attack_preserves_build_activity_fresh_battles_and_unequip",
      "weighted_curio_support_attack_uses_maximum_not_current_hp_and_accepts_skill_attack",
      "weighted_curio_support_attack_composes_with_real_ordinary_curio_damage_modifier",
    ].map((test) => ({ file: inputs.weighted_support_attack_tests, test })),
    ...[
      "weighted_curio_prayer_changes_real_capacity_only_for_both_released_paths",
      "weighted_curio_prayer_consumes_current_hp_before_maximum_shield_and_preserves_one_hp",
      "weighted_curio_prayer_refreshes_one_battle_lifetime_shield_once_per_owner_turn_not_action",
      "weighted_curio_prayer_preserves_fresh_battles_rejected_commands_and_unequip",
      "weighted_curio_prayer_real_battle_handoff_exposes_the_new_maxima_in_verified_result",
      "weighted_curio_prayer_uses_live_resource_capacity_not_hp_stat_or_entry_capacity",
    ].map((test) => ({ file: inputs.weighted_prayer_tests, test })),
    ...[
      "weighted_curio_retaliation_binds_only_physical_owners_and_materialized_enemy_primaries",
      "weighted_curio_retaliation_uses_each_victims_atk_once_per_action_with_nonlethal_owner_credit",
      "weighted_curio_retaliation_rejects_nonattack_damage_and_inactive_owners_without_recursive_damage",
      "weighted_curio_retaliation_sampling_has_path_weights_physical_bonus_and_fresh_hashes",
      "weighted_curio_retaliation_preserves_fresh_handoffs_rejected_commands_and_unequip",
    ].map((test) => ({ file: inputs.weighted_retaliation_tests, test })),
    ...[
      "weighted_curio_break_effect_captures_one_team_maximum_for_both_elements_without_compounding",
      "weighted_curio_break_effect_skips_inactive_members_and_uses_next_living_producer",
      "weighted_curio_break_effect_respects_reordered_roster_and_no_eligible_party",
      "weighted_curio_break_effect_changes_real_break_damage_and_keeps_later_buffs_live",
      "weighted_curio_break_effect_unequip_preserves_activity_and_reconstructs_production_handoffs",
    ].map((test) => ({ file: inputs.weighted_break_effect_tests, test })),
    ...[
      "weighted_curio_necrosis_executes_abundance_multihit_and_multitarget_once_per_action",
      "weighted_curio_necrosis_chance_uses_current_hit_rate_resistance_and_labeled_draws",
      "weighted_curio_necrosis_ticks_three_turns_then_expires_without_reapplication",
      "weighted_curio_necrosis_detonates_other_burns_in_both_stores_without_self_recursion",
      "weighted_curio_necrosis_replaces_cross_caster_source_and_recaptures_attack",
      "weighted_curio_necrosis_external_detonation_produces_one_bounded_other_burn_reaction",
      "weighted_curio_necrosis_preserves_rejections_fresh_handoffs_and_unequip",
    ].map((test) => ({ file: inputs.weighted_necrosis_tests, test })),
    ...[
      "gamble_catalogs_compile_exact_policy_boundaries_without_hex_runtime_admission",
      "gamble_exact_coin_units_execute_and_unresolved_outcomes_fail_closed",
      "all_gamble_groups_preserve_state_and_rng",
    ].map((test) => ({ file: inputs.tests, test })),
  ];
  for (const probe of probes)
    assert(text(probe.file).includes(`fn ${probe.test}(`), `missing test ${probe.test}`);
  return {
    status: "HexWeightedCurioReferencesRuntimePending",
    input_digests: Object.fromEntries(Object.entries(inputs).map(([name, file]) => [
      name, { path: file, sha256: sha256(file) },
    ])),
    source_taxonomy: {
      game_version: "4.4",
      repository: "https://gitlab.com/Dimbreath/turnbasedgamedata.git",
      revision, access_date: "2026-09-27", evidence_quality: "ExactStructured",
      semantic_kind: "WeightedCurio", selector: "RogueTournHex.TournMode=Tourn3",
      source_table: { path: "ExcelOutput/RogueTournHex.json",
        sha256: "51e91a6f53ab0545330e477d806a165108f6c7e5e70ad52b02f92c1dc4d36455" },
      meaning_evidence: { path: "ExcelOutput/RogueTournWorkbenchFunc.json",
        sha256: "b430ce650040a1b2cf5c262f8f69f8e9d15bf6632b81c4947408026e33363078",
        locator: "FuncID=11; FuncType=HexEquipment",
        title_hash: "1823313480930548529", description_hash: "14848339980219349693",
        summary_en: "The HexEquipment service equips and adjusts Weighted Curios.",
        summary_zh_cn: "HexEquipment 服务用于装备及调整加权奇物。" },
      text_map: { path: "TextMap/TextMapEN.json",
        sha256: "afc6d0ff9eeb20d09042e61c311e60d4ed56e69757f1ac42466ca4840e6ed789" },
    },
    current_boundary: {
      reference_transport_names_aligned: true,
      source_obligation_and_fixture_labels_aligned: false,
      owned_hex_semantic_classification: true,
      grand_miracles_admitted_from_hex: 0,
      weighted_curio_loadout_implemented: false,
      weighted_curio_accepted_loadout_boundary: true,
      weighted_curio_source_position_equipment_service_implemented: true,
      weighted_curio_equipment_service_accuracy: "VersionedProjectPolicyExplicitReforgeCapacityCanonicalToggle64Changes",
      weighted_curio_equipment_service_change_limit: 64,
      weighted_curio_equipment_service_flow_controller_bound: true,
      weighted_curio_equipment_service_encoded_replay_implemented: false,
      weighted_curio_accepted_loadout_maximum: 3,
      weighted_curio_loadout_accuracy: "VersionedProjectPolicyAcceptedCurrentCatalogLoadout",
      weighted_curio_unsupported_equipment_rejects_battle_contribution: true,
      weighted_curio_battle_effects_implemented: false,
      weighted_curio_battle_effect_definitions: json(inputs.weighted_transfer_data).table.rows.length
        + json(inputs.weighted_splash_data).table.rows.length
        + json(inputs.weighted_shield_data).table.rows.length
        + json(inputs.weighted_attack_debuff_data).table.rows.length
        + json(inputs.weighted_support_attack_data).table.rows.length
        + json(inputs.weighted_prayer_data).table.rows.length
        + json(inputs.weighted_retaliation_data).table.rows.length
        + json(inputs.weighted_break_effect_data).table.rows.length
        + json(inputs.weighted_necrosis_data).table.rows.length
        + json(inputs.weighted_elation_data).table.rows.length
        + json(inputs.weighted_excitation_data).table.rows.length
        + json(inputs.weighted_encouragement_data).table.rows.length
        + json(inputs.weighted_overflow_data).table.rows.length
        + json(inputs.weighted_footstep_data).table.rows.length
        + json(inputs.weighted_deflagration_data).table.rows.length,
      weighted_curio_encouragement_operands_authored: json(inputs.weighted_encouragement_data).table.rows.length === 1,
      weighted_curio_encouragement_battle_effect_implemented: true,
      weighted_curio_transfer_operands_authored: json(inputs.weighted_transfer_data).table.rows.length === 1,
      weighted_curio_transfer_battle_effect_implemented: true,
      weighted_curio_battle_effect_accuracies: ["VersionedProjectPolicyHitCalculatedCopyAdjacentTrueDamage",
        "VersionedProjectPolicyAllyActionResolvedReplaceTargetTurnShield",
        "VersionedProjectPolicyAttackResolvedAdvanceAndTargetTurnFinalReduction",
        "VersionedProjectPolicyRosterCountCritAndAttackResolvedAdditional",
        "VersionedProjectPolicyEntryHpAndTurnStartConsumeShield",
        "VersionedProjectPolicyPhysicalAggroAndOwnerAdditional",
        "VersionedProjectPolicyEntryHighestTeamBreakEffectCapture",
        "VersionedProjectPolicyAttackResolvedNecrosisAndDotDamageBurnDetonation",
        "VersionedProjectPolicyActionResolvedOriginalPartyRefresh",
        "VersionedProjectPolicyEffectiveGainActionResolvedTeamConsumption",
        "VersionedProjectPolicyOriginalElationFollowUpDamage",
        "VersionedProjectPolicyOtherShieldAppliedOwnerTurnExcessDecay",
        "VersionedProjectPolicyHitEndedOriginalActorConfirmedDeathReadinessEffect",
        "VersionedProjectPolicyEffectiveHpLossAfterSkillOriginalDamage",
        "VersionedProjectPolicyOriginalFireAfterActionNaturalTickBurns"],
      grand_miracle_runtime_implemented: false,
      forge_room_payload_implemented: false,
      terminal_coverage_credit: 0,
      required_next_work: "Implement encoded source-position equipment replay and default complete-position integration; establish original Forge slot-level selection and automatic admission, and implement the remaining two effects and other Forge services. Establish Grand Miracle selectors independently and repair its separate source/fixture audit labels without shrinking obligations.",
    },
    summary: { weighted_curio_references: hex.length,
      current_hex_eligibility_rules: current.length,
      other_mode_eligibility_references: otherMode.length,
      reference_state_boundaries: states.length, unresolved_gamble_groups: groups.length,
      gamble_units: units.length, accepted_coin_units: coins.length,
      unresolved_gamble_units: units.length - coins.length },
    // Existence is checked here; only native Cargo execution verifies behavior.
    native_test_targets: probes,
  };
}

function json(file) { return JSON.parse(text(file)); }
function text(file) { return fs.readFileSync(path.join(root, file), "utf8"); }
function sha256(file) {
  return crypto.createHash("sha256").update(fs.readFileSync(path.join(root, file))).digest("hex");
}
function pretty(value) { return `${JSON.stringify(value, null, 2)}\n`; }
function equal(left, right) { return JSON.stringify(left) === JSON.stringify(right); }
function assert(condition, message) { if (!condition) throw new Error(message); }

if (fileURLToPath(import.meta.url) === path.resolve(process.argv[1])) {
  const serialized = pretty(buildGrandMiracleGambleExecution());
  if (process.argv.includes("--check")) {
    assert(text(output) === serialized, `${output} is stale`);
    console.log("Divergent Universe Hex taxonomy/Gamble inventory is current.");
  } else {
    fs.writeFileSync(path.join(root, output), serialized);
    console.log("Generated current Hex taxonomy/Gamble inventory, without execution credit.");
  }
}
