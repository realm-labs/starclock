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
  weighted_retaliation_data: "config/divergent-universe-decisions-generated/debug-json/DuWeightedCurioRetaliations.json",
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
      weighted_curio_accepted_loadout_maximum: 3,
      weighted_curio_loadout_accuracy: "VersionedProjectPolicyAcceptedCurrentCatalogLoadout",
      weighted_curio_unsupported_equipment_rejects_battle_contribution: true,
      weighted_curio_battle_effects_implemented: false,
      weighted_curio_battle_effect_definitions: json(inputs.weighted_splash_data).table.rows.length
        + json(inputs.weighted_shield_data).table.rows.length
        + json(inputs.weighted_attack_debuff_data).table.rows.length
        + json(inputs.weighted_support_attack_data).table.rows.length
        + json(inputs.weighted_prayer_data).table.rows.length
        + json(inputs.weighted_retaliation_data).table.rows.length,
      weighted_curio_battle_effect_accuracies: ["VersionedProjectPolicyHitCalculatedCopyAdjacentTrueDamage",
        "VersionedProjectPolicyAllyActionResolvedReplaceTargetTurnShield",
        "VersionedProjectPolicyAttackResolvedAdvanceAndTargetTurnFinalReduction",
        "VersionedProjectPolicyRosterCountCritAndAttackResolvedAdditional",
        "VersionedProjectPolicyEntryHpAndTurnStartConsumeShield",
        "VersionedProjectPolicyPhysicalAggroAndOwnerAdditional"],
      grand_miracle_runtime_implemented: false,
      forge_room_payload_implemented: false,
      terminal_coverage_credit: 0,
      required_next_work: "Bind accepted equipment to actual Forge offers and slot-level admission; implement the remaining eleven Weighted Curio effects and equipment-command replay. Establish Grand Miracle selectors independently and repair its separate source/fixture audit labels without shrinking obligations.",
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
