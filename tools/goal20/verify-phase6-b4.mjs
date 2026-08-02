#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const matrixPath = "evidence/swarm-disaster-runtime-v1/foundation/coverage-matrix.json";
const evidence = json(
  "evidence/swarm-disaster-runtime-v1/topology/seeded-run-matrix.json",
);
const matrix = json(matrixPath);
assert(evidence.schema_revision
  === "starclock.swarm-disaster-seeded-run-matrix.v1"
  && evidence.goal_id === "swarm-disaster-runtime-v1"
  && evidence.batch === "G20-P6-B4"
  && evidence.result === "Pass",
"Goal 20 P6-B4 evidence drift");

const frozen = evidence.frozen_matrix;
assert(frozen.path === matrixPath
  && frozen.sha256 === sha256File(matrixPath)
  && frozen.sha256 === "5c29836618f2e8a391f15ca536bdf5082ab4b1d9d64d34a2258d2e5eafae812f"
  && matrix.matrix_revision === "swarm-disaster-seeded-matrix-v1"
  && matrix.runs.length === 16
  && new Set(matrix.runs.map((row) => row.area_id)).size === 5
  && new Set(matrix.runs.map((row) => row.path_id)).size === 8
  && new Set(matrix.runs.map((row) => row.audience_die_id)).size === 8
  && new Set(matrix.runs.flatMap((row) => row.face_ids)).size === 42
  && matrix.runs.filter((row) => row.boundary_case !== null).length === 8
  && matrix.runs.every((row) => row.countdown_initial === 20
    && row.expected_planes === 3
    && row.expected_terminal === "Complete"
    && row.replay_verification === "FreshFactoryRequired"),
"P6-B4 frozen matrix coverage drift");
const probes = matrix.runs.flatMap((row) => row.policy_probes)
  .map((probe) => probe.id);
assert(probes.length === 31 && new Set(probes).size === 31,
  "P6-B4 policy probes must cover all 31 boundaries exactly once");

const execution = evidence.execution;
assert(execution.revision === "swarm-disaster-seeded-run-v1"
  && execution.driver_visibility === "PrivateTestOnlyNoPublicModeSurface"
  && execution.completed_runs === 16
  && execution.fresh_factory_verified_runs === 16
  && execution.primary_nested_battles === 202
  && execution.fresh_verification_nested_battles === 202
  && execution.total_executor_invocations === 404
  && execution.minimum_battles_per_run === 12
  && execution.maximum_battles_per_run === 13
  && execution.minimum_steps_per_run === 48
  && execution.maximum_steps_per_run === 50
  && execution.terminal === "Completed"
  && execution.matrix_roster_accuracy
    === "SyntheticBalanceIndependentNotObservedNumericParity"
  && execution.runtime_json_reads === 0,
"P6-B4 execution summary drift");
assert(evidence.boundaries.initial_countdown === 20
  && evidence.boundaries.maximum_uncapped_disarray_observed === 22
  && Object.entries(evidence.boundaries)
    .filter(([key]) => !["initial_countdown", "maximum_uncapped_disarray_observed"].includes(key))
    .every(([, value]) => value === true),
"P6-B4 Countdown/Disarray boundary evidence drift");
assert(evidence.runs.length === 16
  && evidence.runs.every((run, index) => run.id === matrix.runs[index].id
    && Number.isInteger(run.battle_count)
    && run.battle_count >= 12 && run.battle_count <= 13
    && Number.isInteger(run.step_count)
    && run.step_count >= 48 && run.step_count <= 50
    && Number.isInteger(run.maximum_disarray_level)
    && /^[0-9a-f]{64}$/u.test(run.final_state_hash)
    && /^[0-9a-f]{64}$/u.test(run.transcript_digest))
  && evidence.runs.reduce((sum, run) => sum + run.battle_count, 0) === 202,
"P6-B4 per-run golden evidence drift");
assert(Object.values(evidence.validation).every(Boolean),
  "P6-B4 validation evidence is incomplete");

const seeded = text(
  "crates/starclock-mode-universe/src/swarm_disaster_entry/seeded_run.rs",
);
const tests = text(
  "crates/starclock-mode-universe/src/swarm_disaster_entry/seeded_run_tests.rs",
);
const moduleSource = text(
  "crates/starclock-mode-universe/src/swarm_disaster_entry/mod.rs",
);
const materializationTests = text(
  "crates/starclock-mode-universe/src/swarm_disaster_entry/battle_materialization_tests.rs",
);
for (const literal of [
  "SWARM_DISASTER_SEEDED_RUN_REVISION",
  "execute_seeded_run",
  "verify_seeded_run",
  "UniverseBattleRoster",
  "execute_current_battle",
  "ReplayDivergence",
  "transcript_digest",
  "InitialCountdown",
  "FinalBossDecay",
  "selected_boss_decay",
]) assert(seeded.includes(literal), `missing seeded-run contract ${literal}`);
for (const literal of [
  "frozen_matrix_completes_real_battles_and_verifies_from_fresh_factories",
  "SwarmDisasterRuntimeFactory::load_candidate",
  "seeded_matrix_roster",
  "assert_eq!(total_battles, 202)",
]) assert(tests.includes(literal), `missing seeded matrix regression ${literal}`);
for (const run of evidence.runs) {
  assert(tests.includes(run.id)
    && tests.includes(run.final_state_hash)
    && tests.includes(run.transcript_digest),
  `${run.id}: test golden drift`);
}
assert(moduleSource.includes("#[cfg(test)]\nmod seeded_run;")
  && !moduleSource.includes("pub mod seeded_run")
  && materializationTests.includes("seeded_matrix_roster")
  && materializationTests.includes("1_000_000_000_000"),
"P6-B4 private driver or balance-independent roster drift");
for (const source of [seeded, tests])
  for (const forbidden of [
    "serde_json", "std::fs", "read_to_string", "SystemTime", "thread_rng", "f32", "f64",
  ]) assert(forbidden === "f32" || forbidden === "f64"
    ? !new RegExp(`\\b${forbidden}\\b`, "u").test(source)
    : !source.includes(forbidden),
    `seeded matrix runtime gained forbidden dependency ${forbidden}`);
assert(seeded.split(/\r?\n/u).length <= 800,
  "P6-B4 seeded runner should be split before 800 lines");

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
assert(status.includes("| Active phase | Phase 7 — Replay, controllers and external surfaces |")
  && status.includes("| Active batch | None |")
  && status.includes("| Next unblocked batch | `G20-P7-B1` |")
  && status.includes("| `G20-P6-B4` | `Complete` |")
  && status.includes("- [x] The seeded matrix completes and freshly verifies every replay."),
"G20-P6-B4 ledger is incomplete");
const testEvidence = evidence.tests;
assert(testEvidence.focused_matrix_tests_passed === 1
  && testEvidence.swarm_entry_suite_passed === 138
  && testEvidence.aggregate_swarm_suite_passed === 149
  && testEvidence.swarm_integration_tests_passed === 5
  && testEvidence.activity_suite_passed === 63
  && testEvidence.clippy_passed === true
  && testEvidence.dependency_policy_passed === true
  && testEvidence.source_policy_passed === true
  && testEvidence.handwritten_rust_files === 948
  && testEvidence.public_reexport_declarations === 72
  && testEvidence.goal_verifier_passed === true
  && testEvidence.quick_gate_passed === true
  && Number(testEvidence.quick_gate_seconds) > 0
  && Number.isInteger(testEvidence.quick_selected_harnesses)
  && Number.isInteger(testEvidence.quick_deferred_inputs)
  && Number(testEvidence.final_tree_quick_gate_seconds) > 0
  && testEvidence.final_tree_quick_rust_receipt === "CacheHit"
  && testEvidence.full_gate_required === true
  && testEvidence.full_gate_passed === true
  && Number(testEvidence.full_gate_seconds) > 0
  && testEvidence.full_generated_checks === 33
  && testEvidence.full_source_cache_skips === 4
  && testEvidence.full_workspace_harnesses === 34,
"P6-B4 test evidence drift");

console.log(
  "Goal 20 P6-B4 verified (16/16 fresh-verified runs; 202 primary and 202 "
  + "fresh-verification nested battles; all frozen axes and boundaries).",
);

function text(relative) {
  return fs.readFileSync(path.join(root, relative), "utf8");
}
function json(relative) {
  return JSON.parse(text(relative));
}
function sha256File(relative) {
  return crypto.createHash("sha256").update(
    fs.readFileSync(path.join(root, relative)),
  ).digest("hex");
}
function captureGit(args) {
  return execFileSync("git", args, {
    cwd: root,
    encoding: "utf8",
    maxBuffer: 16 * 1024 * 1024,
  });
}
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
