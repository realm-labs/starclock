#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const evidence = json(
  "evidence/swarm-disaster-runtime-v1/runtime/countdown-disarray.json",
);

assert(evidence.schema_revision === "starclock.swarm-disaster-countdown-disarray.v1"
  && evidence.goal_id === "swarm-disaster-runtime-v1"
  && evidence.batch === "G20-P2-B4"
  && evidence.result === "Pass",
"Goal 20 Countdown/Disarray evidence drift");
const input = evidence.catalog_input;
assert(input.candidate_bundle_sha256
  === "385727a8a5875795b29c996102040f7f4419c6adac7b5e10ee6b09c084409362"
  && input.mode_constants === 19
  && input.boss_decay_rows === 42
  && input.enabled_swarm_boss_decay_rows === 15
  && input.disabled_unproven_shared_dlc_rows === 27
  && input.assigned_source_obligations === 61,
"Countdown/Disarray input denominator drift");

const countdown = evidence.countdown_lifecycle;
assert(countdown.initial === 20
  && countdown.warning_threshold === 5
  && countdown.movement_delta === -1
  && countdown.slot_scope === "Activity"
  && countdown.carry_policy === "CarryExact"
  && countdown.movement_precedes_adjustments === true
  && countdown.adjustment_order === "stable-operation-id"
  && countdown.duplicate_adjustment_ids_rejected === true
  && countdown.stale_program_rejected_atomically === true
  && countdown.rng_draws === 0,
"Countdown lifecycle contract drift");
const disarray = evidence.planar_disarray;
assert(disarray.transition_boundary === "AcceptedMoveWhenPreMoveCountdownIsZero"
  && disarray.transition_countdown === -1
  && disarray.transition_level === 1
  && disarray.stored_level_is_uncapped === true
  && disarray.modifier_cap_level === 20
  && disarray.modifier_units === "integer-percent",
"Planar Disarray transition contract drift");
assert(disarray.cumulative_modifier_vectors.map((row) =>
  `${row.level}:${row.damage_dealt}:${row.damage_received_reduction}:${row.speed}`)
  .join(",") === [
    "1:5:4:0",
    "5:25:20:0",
    "6:35:24:5",
    "10:75:40:25",
    "11:95:44:35",
    "20:275:80:125",
    "21:275:80:125",
  ].join(","),
"Planar Disarray modifier vectors drift");
const decay = evidence.boss_decay_contributions;
assert(decay.addressing === "stable-starclock-key"
  && decay.selection_order === "stable-boss-decay-id"
  && decay.maximum_selected === 2
  && decay.maximum_per_plane_threshold === 1
  && decay.selected_rows_coexist === true
  && decay.effect_parameters_retained_as_canonical_sora_values === true
  && decay.unproven_shared_dlc_rows_fail_closed === true
  && decay.battle_spec_application_state === "PendingG20P6B3",
"boss-decay contribution contract drift");

const expectedPolicies = new Map([
  ["swarm-disaster.research-gap.source-goal09-project-policy-boss-decay-levels", 46],
  ["swarm-disaster.research-gap.source-goal09-project-policy-countdown-and-disarray", 7],
  ["swarm-disaster.research-gap.source-goal09-public-hoyolab-swarm-progression-countdown", 51],
]);
assert(evidence.policy_boundaries.length === 3,
  "Countdown/Disarray policy count drift");
for (const policy of evidence.policy_boundaries) {
  assert(expectedPolicies.get(policy.boundary_id) === policy.affected_record_count
    && policy.state === "InheritedPolicy"
    && policy.remaining_owner === "G20-P6-B3"
    && nonEmpty(policy.implemented_revision),
  `Countdown/Disarray policy is mislabeled: ${policy.boundary_id}`);
}
const deferred = evidence.deferred_semantics;
assert(deferred.mechanic_rules === 3
  && deferred.mechanic_rule_batch === "G20-P5-M03"
  && deferred.semantic_fixtures === 3
  && deferred.semantic_fixture_batch === "G20-P5-B1"
  && deferred.all_remain_pending === true,
"P2-B4 overclaimed deferred semantic completion");
const validation = evidence.validation;
assert(validation.external_runtime_json_reads === 0
  && validation.embedded_sora_json_lowered_once_at_factory_load === true
  && validation.authoritative_float_fields === 0
  && validation.new_public_mode_types === 0
  && validation.public_reexports_added === 0
  && validation.source_policy_public_reexports === 72,
"Countdown/Disarray validation evidence drift");
const tests = evidence.tests;
assert(tests.entry_and_lifecycle_unit_passed === 12
  && tests.swarm_unit_passed === 23
  && tests.identity_integration_passed === 5
  && tests.clippy_passed === true
  && nonEmpty(tests.quick_gate_result)
  && nonEmpty(tests.quick_gate_seconds)
  && Number.isInteger(tests.quick_deferred_inputs)
  && tests.full_gate_passed === true
  && nonEmpty(tests.full_gate_seconds)
  && tests.full_generated_checks === 33
  && tests.full_source_cache_skips === 4
  && tests.full_workspace_harnesses === 34
  && nonEmpty(tests.full_workspace_tests_seconds),
"Countdown/Disarray test evidence drift");

const source = text(
  "crates/starclock-mode-universe/src/swarm_disaster_entry/countdown.rs",
);
const instance = text(
  "crates/starclock-mode-universe/src/swarm_disaster_entry/instance.rs",
);
const state = text(
  "crates/starclock-mode-universe/src/swarm_disaster_entry/state.rs",
);
for (const literal of [
  "MOVE_PROGRAM_ID: u32 = 0x5350_0001",
  "ADJUSTMENT_PROGRAM_ID: u32 = 0x5350_0002",
  "BOSS_DECAY_PROGRAM_ID: u32 = 0x5350_0003",
  "DISARRAY_LEVEL_KEY: u64 = 1",
  "BOSS_DECAY_KEY_BASE: u64 = 0x1000_0000",
  "level.clamp(0, 20)",
  "sort_unstable_by_key(|(operation_id, _)| *operation_id)",
  "ActivityOperation::Require",
]) assert(source.includes(literal), `missing Countdown contract ${literal}`);
for (const literal of [
  "compile_countdown_move",
  "compile_countdown_adjustments",
  "compile_boss_decay_selection",
  "countdown_warning_active",
  "disarray_modifiers",
]) assert(instance.includes(literal), `missing Countdown API ${literal}`);
assert(state.includes("pub(super) const COUNTDOWN: u32 = 0x5344_0008")
  && state.includes("pub(super) const DISARRAY: u32 = 0x5344_0009")
  && state.includes("SlotCarryPolicy::CarryExact"),
"Countdown state ownership/carry drift");
assert(!source.includes("rand::") && !source.includes("thread_rng")
  && !source.includes("SystemTime") && !source.includes("f32")
  && !source.includes("f64"),
"Countdown execution introduced nondeterminism or floats");

for (const relative of [
  "crates/starclock-mode-universe/src/swarm_disaster_entry/countdown.rs",
  "crates/starclock-mode-universe/src/swarm_disaster_entry/countdown_tests.rs",
  "crates/starclock-mode-universe/src/swarm_disaster_entry/factory.rs",
  "crates/starclock-mode-universe/src/swarm_disaster_entry/instance.rs",
  "crates/starclock-mode-universe/src/swarm_disaster_entry/mod.rs",
  "crates/starclock-mode-universe/src/swarm_disaster_unique/runtime_access.rs",
]) assert(text(relative).split(/\r?\n/u).length <= 800,
  `Swarm runtime source should be split before 800 lines: ${relative}`);
assert(text("crates/starclock-mode-universe/src/swarm_disaster_entry/mod.rs")
  .split(/\r?\n/u).length <= 200,
"Swarm entry facade exceeds the 200-line limit");

const dispositions = json(
  "content-manifests/swarm-disaster-runtime-v1/runtime-dispositions.json",
);
const assigned = dispositions.source_obligations.filter((row) =>
  row.execution_batch === "G20-P2-B4");
assert(assigned.length === 61, "P2-B4 source-obligation assignment drift");
const categories = counts(assigned.map((row) => row.category));
assert(categories.mode_constants === 19 && categories.boss_decay_levels === 42,
  "P2-B4 category denominator drift");
const rules = dispositions.mechanic_rules.filter((row) =>
  row.implementation_batch === "G20-P5-M03");
assert(rules.length === 3 && rules.every((row) => row.current_state === "Pending"),
  "P2-B4 mechanic-rule partition drift");
const fixtures = dispositions.semantic_fixtures.filter((row) =>
  row.implementation_owner_batch === "G20-P2-B4");
assert(fixtures.length === 3
  && fixtures.every((row) => row.execution_batch === "G20-P5-B1"
    && row.current_state === "Pending"),
"P2-B4 semantic-fixture assignment drift");
for (const [id, affected] of expectedPolicies) {
  const boundary = dispositions.policy_boundaries.find((row) => row.id === id);
  assert(boundary?.current_state === "InheritedPolicy"
    && boundary.affected_record_count === affected
    && boundary.implementation_batches.join(",") === "G20-P2-B4,G20-P6-B3",
  `frozen P0 policy assignment drift: ${id}`);
}

for (const protectedRoot of [
  "evidence/swarm-disaster-reference-v1",
  "content-manifests/swarm-disaster-v1",
  "content-reference/swarm-disaster-v1",
  "config/swarm-disaster/data",
  "config/swarm-disaster-generated",
]) assert(captureGit([
  "status", "--porcelain=v1", "--untracked-files=all", "--", protectedRoot,
]).trim() === "", `protected root has worktree changes: ${protectedRoot}`);

const status = text("docs/goals/20-swarm-disaster-runtime-status.md");
assert(status.includes("| `G20-P2-B4` | `Complete` |")
  && status.includes("| Active batch | None |")
  && status.includes("| Next unblocked batch | `G20-P2-B5` |")
  && status.includes("28 inherited / 3 terminal / 28 pending"),
"Goal 20 did not advance after P2-B4");

console.log(
  "Goal 20 P2-B4 verified (61 inputs; Countdown 20/-1; "
    + "Disarray level-20 cap; 15/27 boss-decay boundary).",
);

function text(relative) {
  return fs.readFileSync(path.join(root, relative), "utf8");
}
function json(relative) {
  return JSON.parse(text(relative));
}
function captureGit(args) {
  return execFileSync("git", args, {
    cwd: root,
    encoding: "utf8",
    maxBuffer: 16 * 1024 * 1024,
  });
}
function counts(values) {
  return values.reduce((result, value) => {
    result[value] = (result[value] ?? 0) + 1;
    return result;
  }, {});
}
function nonEmpty(value) {
  return typeof value === "string" && value.length > 0;
}
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
